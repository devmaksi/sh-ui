// story: e01s03
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use crate::errors::{StartError, StopError};
use crate::events::{EventSink, OutputStream, RunEvent};
use crate::gpu::{build_gpu_probe, probe_gpu_source, run_tool, GpuProbe, UnavailableGpuProbe};
use crate::output::{mark_reader_done, read_stream, Completion};
use crate::spawn::{spawn_capturing, terminate_group, working_directory};
use crate::metrics::{spawn_metrics_poller, ProcfsReader, METRICS_REFRESH};
// story: e02s04 — live group records for crash recovery.
use crate::recovery::{self, LiveGroupRecord, LiveRunFile};
// story: e02s01
use crate::registry::RunConfig;
use crate::watch_exit::{watch_exit, ExitWatch};

#[derive(Debug, Clone)]
pub struct RunHandle {
    pub run_id: String,
    pub entry_id: String,
    pub pid: u32,
}

#[derive(Debug, Clone)]
struct LiveRun {
    pid: u32,
    finished: Arc<AtomicBool>,
    // story: e02s02 — set by stop_run before the group is terminated; watch_exit reports it.
    stopped: Arc<AtomicBool>,
}

/// Owns every live run of the host. Terminating a run kills its whole process group, because
/// each run is spawned as a group leader.
pub struct Supervisor {
    runs: HashMap<String, LiveRun>,
    sink: Arc<dyn EventSink>,
    gpu: Arc<dyn GpuProbe>,
    // story: e02s04 — live group records kept while runs are live.
    state_file: Option<LiveRunFile>,
    sequence: u64,
}

impl Supervisor {
    pub fn new(sink: impl EventSink) -> Self {
        Self {
            runs: HashMap::new(),
            sink: Arc::new(sink),
            gpu: Arc::new(UnavailableGpuProbe::default()),
            state_file: None,
            sequence: 0,
        }
    }

    /// story: e02s04 — the host keeps a live group record for every live run so a
    /// crashed host's groups can be reaped on the next start.
    pub fn with_state_file(mut self, path: impl AsRef<Path>) -> Self {
        self.state_file = Some(LiveRunFile::new(path.as_ref().to_path_buf()));
        self
    }

    /// The host probes vendor GPU tooling once at startup and hands the answer to every
    /// poller. Tests keep the default Unavailable probe.
    pub fn attach_gpu(&mut self) {
        self.gpu = build_gpu_probe(&probe_gpu_source(&run_tool));
    }

    pub fn live_run_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .runs
            .iter()
            .filter(|(_, run)| !run.finished.load(Ordering::SeqCst))
            .map(|(run_id, _)| run_id.clone())
            .collect();
        ids.sort();
        ids
    }

    pub fn start(
        &mut self,
        entry_id: &str,
        path: &Path,
        config: &RunConfig,
    ) -> Result<RunHandle, StartError> {
        validate_registered_file(path)?;

        // story: e02s01 — check the stored directory before spawning so the refusal is
        // typed, not a raw OS error from a failed chdir.
        let working_dir = working_directory(path, config)?;

        self.sequence += 1;
        let run_id = format!("run-{}", self.sequence);

        let (child, pid) = self.spawn_and_record(&run_id, path, config, &working_dir)?;
        self.wire_run(&run_id, entry_id, child, pid);

        Ok(RunHandle {
            run_id,
            entry_id: entry_id.to_string(),
            pid,
        })
    }

    /// Spawn the process group and register the run before anything can stop it.
    fn spawn_and_record(
        &mut self,
        run_id: &str,
        path: &Path,
        config: &RunConfig,
        working_dir: &Path,
    ) -> Result<(Child, u32), StartError> {
        let child = spawn_capturing(path, config, working_dir)
            .map_err(|error| StartError::SpawnFailed(format!("{}: {error}", path.display())))?;

        let pid = child.id();

        // story: e02s04 — record the live group before anything can end it.
        if let Some(file) = &self.state_file
            && let Some(start_time) = recovery::start_time_ticks(pid)
        {
            file.insert(&LiveGroupRecord {
                pgid: pid,
                pid,
                start_time,
            });
        }

        self.runs.insert(
            run_id.to_string(),
            LiveRun {
                pid,
                finished: Arc::new(AtomicBool::new(false)),
                stopped: Arc::new(AtomicBool::new(false)),
            },
        );

        Ok((child, pid))
    }

    /// Hand one spawned run to its readers, poller, and exit watch. Started is emitted
    /// here so it always precedes any Output event.
    fn wire_run(&mut self, run_id: &str, entry_id: &str, mut child: Child, pid: u32) {
        let (finished, stopped) = {
            let run = self.runs.get(run_id).expect("live run must exist");
            (run.finished.clone(), run.stopped.clone())
        };

        self.sink.emit(RunEvent::Started {
            run_id: run_id.to_string(),
            entry_id: entry_id.to_string(),
            pid,
        });

        let completion = Arc::new((
            Mutex::new(Completion {
                exit_code: None,
                readers_done: 0,
            }),
            Condvar::new(),
        ));

        read_child_output(&mut child, run_id, &self.sink, &completion);

        // story: e03s01
        // The poller ends with the run: it watches the same finished flag and stops when
        // the pid stops answering.
        let poller = spawn_metrics_poller(
            ProcfsReader::new(),
            run_id.to_string(),
            pid,
            self.sink.clone(),
            self.gpu.clone(),
            finished.clone(),
            METRICS_REFRESH,
        );

        watch_exit(ExitWatch {
            child,
            run_id: run_id.to_string(),
            sink: self.sink.clone(),
            finished,
            stopped,
            completion,
            state_file: self.state_file.clone(),
            // story: e02s04 — Ended must be the last event of a run: watch_exit stops the
            // poller and joins it before announcing the verdict, so no in-flight metrics
            // tick can land after Ended.
            poller,
        });
    }

    /// story: e02s02 — stop one live run: SIGTERM the whole group, then SIGKILL whatever
    /// is still alive. Same terminate_group path the host shutdown uses (DEC-07).
    pub fn stop_run(&mut self, run_id: &str) -> Result<(), StopError> {
        let run = self
            .runs
            .get(run_id)
            .filter(|run| !run.finished.load(Ordering::SeqCst))
            .ok_or_else(|| StopError::RunNotFound(run_id.to_string()))?;

        run.stopped.store(true, Ordering::SeqCst);
        terminate_group(run.pid, &run.finished);
        Ok(())
    }

    /// story: e02s04 — shutdown routes every live run through the same stop_run path a
    /// user press uses, so each run emits its final Ended with status stopped, and a
    /// clean exit leaves no live group record behind.
    pub fn shutdown(&mut self) {
        let ids = self.live_run_ids();

        for run_id in ids {
            let _ = self.stop_run(&run_id);
        }

        if let Some(file) = &self.state_file {
            file.clear();
        }
    }
}

impl Drop for Supervisor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// story: e02s01 — the registered path must be a regular file with the execute bit
/// (DEC-08).
fn validate_registered_file(path: &Path) -> Result<(), StartError> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| StartError::EntryNotFound(format!("{}: {error}", path.display())))?;

    if !metadata.is_file() {
        return Err(StartError::EntryNotFound(path.display().to_string()));
    }

    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(StartError::NotExecutable(path.display().to_string()));
    }

    Ok(())
}

/// Pipe both child streams to the sink; a missing stream counts as an already-done reader.
fn read_child_output(
    child: &mut Child,
    run_id: &str,
    sink: &Arc<dyn EventSink>,
    completion: &Arc<(Mutex<Completion>, Condvar)>,
) {
    if let Some(stdout) = child.stdout.take() {
        read_stream(
            stdout,
            OutputStream::Stdout,
            run_id.to_string(),
            sink.clone(),
            completion.clone(),
        );
    } else {
        mark_reader_done(completion);
    }

    if let Some(stderr) = child.stderr.take() {
        read_stream(
            stderr,
            OutputStream::Stderr,
            run_id.to_string(),
            sink.clone(),
            completion.clone(),
        );
    } else {
        mark_reader_done(completion);
    }
}

