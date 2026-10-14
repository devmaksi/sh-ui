// story: e01s03
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::gpu::{build_gpu_probe, probe_gpu_source, run_tool, GpuProbe, UnavailableGpuProbe};
use crate::metrics::{spawn_metrics_poller, ProcfsReader, METRICS_REFRESH};

/// One event payload cannot exceed this. A script that writes fast must not flood the UI.
pub const OUTPUT_CHUNK_MAX_BYTES: usize = 8 * 1024;

const TERM_GRACE: Duration = Duration::from_secs(2);
const KILL_GRACE: Duration = Duration::from_secs(1);

/// ETXTBSY means the file is open for writing somewhere — an editor saving, a backup tool
/// copying. It clears in milliseconds, so the host tries a few times before reporting it.
const SPAWN_ATTEMPTS: u8 = 8;
const SPAWN_RETRY_WAIT: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

impl OutputStream {
    pub fn tag(self) -> &'static str {
        match self {
            OutputStream::Stdout => "stdout",
            OutputStream::Stderr => "stderr",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunEvent {
    Started {
        run_id: String,
        entry_id: String,
        pid: u32,
    },
    Output {
        run_id: String,
        stream: OutputStream,
        chunk: String,
    },
    Ended {
        run_id: String,
        code: Option<i32>,
    },
    Metrics {
        run_id: String,
        cpu_percent: f64,
        ram_bytes: u64,
        processes: Vec<crate::metrics::ProcessCost>,
        vram_bytes: Option<u64>,
        gpu_util_percent: Option<f64>,
        gpu_source: String,
        gpu_reason: Option<String>,
    },
}

/// What the supervisor does with output. The Tauri adapter turns events into app events;
/// tests use `Recorder`.
pub trait EventSink: Send + Sync + 'static {
    fn emit(&self, event: RunEvent);
}

#[derive(Clone, Default)]
pub struct Recorder(Arc<Mutex<Vec<RunEvent>>>);

impl Recorder {
    pub fn events(&self) -> Vec<RunEvent> {
        self.0.lock().expect("recorder lock").clone()
    }
}

impl EventSink for Recorder {
    fn emit(&self, event: RunEvent) {
        self.0.lock().expect("recorder lock").push(event);
    }
}

#[derive(Debug)]
pub enum StartError {
    EntryNotFound(String),
    NotExecutable(String),
    SpawnFailed(String),
}

impl StartError {
    /// Stable tag for the UI error union. Changing a tag here changes the TypeScript union.
    pub fn kind(&self) -> &'static str {
        match self {
            StartError::EntryNotFound(_) => "entryNotFound",
            StartError::NotExecutable(_) => "notExecutable",
            StartError::SpawnFailed(_) => "spawnFailed",
        }
    }
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::EntryNotFound(detail) => write!(f, "no registry entry: {detail}"),
            StartError::NotExecutable(path) => {
                write!(f, "{path} is not executable; add the execute bit or remove the entry")
            }
            StartError::SpawnFailed(detail) => write!(f, "the process could not be started: {detail}"),
        }
    }
}

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
}

/// When both output readers have reached EOF and the child has been reaped, the run is
/// complete. `Ended` is emitted only after that, so the UI never sees "finished" while
/// output is still in flight.
struct Completion {
    exit_code: Option<Option<i32>>,
    readers_done: u8,
}

fn mark_reader_done(state: &Arc<(Mutex<Completion>, Condvar)>) {
    let (lock, signal) = &**state;
    let mut completion = lock.lock().expect("completion lock");
    completion.readers_done += 1;
    signal.notify_all();
}

fn mark_exit(state: &Arc<(Mutex<Completion>, Condvar)>, code: Option<i32>) {
    let (lock, signal) = &**state;
    let mut completion = lock.lock().expect("completion lock");
    completion.exit_code = Some(code);
    signal.notify_all();
}

/// Owns every live run of the host. Terminating a run kills its whole process group, because
/// each run is spawned as a group leader.
pub struct Supervisor {
    runs: HashMap<String, LiveRun>,
    sink: Arc<dyn EventSink>,
    gpu: Arc<dyn GpuProbe>,
    sequence: u64,
}

impl Supervisor {
    pub fn new(sink: impl EventSink) -> Self {
        Self {
            runs: HashMap::new(),
            sink: Arc::new(sink),
            gpu: Arc::new(UnavailableGpuProbe::default()),
            sequence: 0,
        }
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

    pub fn start(&mut self, entry_id: &str, path: &Path) -> Result<RunHandle, StartError> {
        let metadata = std::fs::metadata(path)
            .map_err(|error| StartError::EntryNotFound(format!("{}: {error}", path.display())))?;

        if !metadata.is_file() {
            return Err(StartError::EntryNotFound(path.display().to_string()));
        }

        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(StartError::NotExecutable(path.display().to_string()));
        }

        self.sequence += 1;
        let run_id = format!("run-{}", self.sequence);

        let mut child = spawn_capturing(path)
            .map_err(|error| StartError::SpawnFailed(format!("{}: {error}", path.display())))?;

        let pid = child.id();
        let finished = Arc::new(AtomicBool::new(false));

        self.runs.insert(
            run_id.clone(),
            LiveRun {
                pid,
                finished: finished.clone(),
            },
        );

        self.sink.emit(RunEvent::Started {
            run_id: run_id.clone(),
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

        if let Some(stdout) = child.stdout.take() {
            read_stream(
                stdout,
                OutputStream::Stdout,
                run_id.clone(),
                self.sink.clone(),
                completion.clone(),
            );
        } else {
            mark_reader_done(&completion);
        }

        if let Some(stderr) = child.stderr.take() {
            read_stream(
                stderr,
                OutputStream::Stderr,
                run_id.clone(),
                self.sink.clone(),
                completion.clone(),
            );
        } else {
            mark_reader_done(&completion);
        }

        watch_exit(child, run_id.clone(), self.sink.clone(), finished.clone(), completion);

        // story: e03s01
        // The poller ends with the run: it watches the same finished flag and stops when
        // the pid stops answering.
        spawn_metrics_poller(
            ProcfsReader::new(),
            run_id.clone(),
            pid,
            self.sink.clone(),
            self.gpu.clone(),
            finished.clone(),
            METRICS_REFRESH,
        );

        Ok(RunHandle {
            run_id,
            entry_id: entry_id.to_string(),
            pid,
        })
    }

    /// SIGTERM the whole group, then SIGKILL whatever is still alive.
    pub fn shutdown(&mut self) {
        let runs: Vec<LiveRun> = self.runs.values().cloned().collect();
        self.runs.clear();

        for run in runs {
            terminate_group(run.pid, &run.finished);
        }
    }
}

impl Drop for Supervisor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn spawn_capturing(path: &Path) -> Result<Child, std::io::Error> {
    let mut attempt = 0;

    loop {
        // argv array only. The path is one argument to exec, never a shell command string.
        let mut command = Command::new(path);
        // A script reads its own siblings by relative path. Run it where it lives, not where
        // the host happens to have been started (ADR-0002).
        let working_dir = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        command
            .current_dir(working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(error) if error.raw_os_error() == Some(libc::ETXTBSY) && attempt < SPAWN_ATTEMPTS => {
                attempt += 1;
                thread::sleep(SPAWN_RETRY_WAIT);
            }
            Err(error) => return Err(error),
        }
    }
}

fn terminate_group(pid: u32, finished: &AtomicBool) {
    let group = pid as libc::pid_t;

    let term_accepted = unsafe { libc::killpg(group, libc::SIGTERM) } == 0;

    if term_accepted && wait_for_exit(pid, TERM_GRACE) {
        finished.store(true, Ordering::SeqCst);
        return;
    }

    unsafe { libc::killpg(group, libc::SIGKILL) };
    wait_for_exit(pid, KILL_GRACE);
    finished.store(true, Ordering::SeqCst);
}

fn wait_for_exit(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let proc = format!("/proc/{pid}");

    while Instant::now() < deadline {
        if !std::path::Path::new(&proc).exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }

    !std::path::Path::new(&proc).exists()
}

fn read_stream(
    stream: impl std::io::Read + Send + 'static,
    which: OutputStream,
    run_id: String,
    sink: Arc<dyn EventSink>,
    completion: Arc<(Mutex<Completion>, Condvar)>,
) {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            for chunk in split_chunk(&line) {
                sink.emit(RunEvent::Output {
                    run_id: run_id.clone(),
                    stream: which,
                    chunk,
                });
            }
        }
        mark_reader_done(&completion);
    });
}

/// A line longer than the cap becomes several chunks. No single event is bigger than the cap.
fn split_chunk(line: &str) -> Vec<String> {
    if line.len() <= OUTPUT_CHUNK_MAX_BYTES {
        return vec![line.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for character in line.chars() {
        if current.len() + character.len_utf8() > OUTPUT_CHUNK_MAX_BYTES {
            chunks.push(std::mem::take(&mut current));
        }
        current.push(character);
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn watch_exit(
    mut child: Child,
    run_id: String,
    sink: Arc<dyn EventSink>,
    finished: Arc<AtomicBool>,
    completion: Arc<(Mutex<Completion>, Condvar)>,
) {
    thread::spawn(move || {
        let status = child.wait();
        mark_exit(&completion, status.ok().and_then(|status| status.code()));

        let (lock, signal) = &*completion;
        let mut guard = lock.lock().expect("completion lock");
        while guard.readers_done < 2 {
            guard = signal.wait(guard).expect("completion lock not poisoned");
        }

        let code = guard.exit_code.flatten();
        drop(guard);

        sink.emit(RunEvent::Ended { run_id, code });
        finished.store(true, Ordering::SeqCst);
    });
}
