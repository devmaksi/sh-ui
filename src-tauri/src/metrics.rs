// story: e03s01 e03s02
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::supervisor::{EventSink, RunEvent};
use procfs::process::Process;

/// Gap between two metrics events. One named constant so host and UI agree on the cadence.
pub const METRICS_REFRESH: Duration = Duration::from_millis(1000);

/// One reading of one pid: cpu_ticks is utime+stime in clock ticks, rss_bytes is resident
/// memory. Shared by the poller and the tree sum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessSample {
    pub cpu_ticks: u64,
    pub rss_bytes: u64,
}

/// Reads process samples from /proc. The trait exists so poller and sum tests can feed
/// deterministic samples instead of racing real processes.
pub trait ProcReader: Send + Sync {
    /// `None` when the pid is gone — a dead process is not an error, it is an answer.
    fn sample(&self, pid: u32) -> Option<ProcessSample>;

    /// Direct children of one pid from the /proc child lists.
    fn children(&self, pid: u32) -> Vec<u32>;
}

/// Percent of one core (DEC-12): consumed ticks over elapsed wall time, top-style.
/// A multi-threaded run may exceed 100. A zero wall delta reports 0 instead of dividing.
pub fn cpu_percent(tick_delta: u64, wall_ms: u64, clock_ticks_per_sec: u64) -> f64 {
    if wall_ms == 0 || clock_ticks_per_sec == 0 {
        return 0.0;
    }
    let seconds = wall_ms as f64 / 1000.0;
    tick_delta as f64 / (clock_ticks_per_sec as f64 * seconds) * 100.0
}

/// One summed reading of the whole tree: percent of one core over the tree and summed
/// resident bytes (DEC-13).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeSnapshot {
    pub cpu_percent: f64,
    pub ram_bytes: u64,
}

/// One process inside the tree, for the breakdown behind the headline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessCost {
    pub pid: u32,
    pub cpu_percent: f64,
    pub ram_bytes: u64,
}

/// The breakdown array is capped so a fork bomb cannot grow one event without bound.
/// The headline sum always covers the whole tree.
pub const BREAKDOWN_MAX_PROCESSES: usize = 64;

/// Sums cpu tick deltas and resident bytes over the given pids. A pid that vanished
/// between the walk and the read is dropped from this snapshot — never counted, never
/// carried into the next baseline. A pid with no previous sample contributes memory but
/// no cpu history.
pub fn tree_sum(
    reader: &dyn ProcReader,
    pids: &[u32],
    previous: &HashMap<u32, ProcessSample>,
    wall_ms: u64,
    clock_ticks_per_sec: u64,
) -> (TreeSnapshot, HashMap<u32, ProcessSample>) {
    let mut tick_delta = 0u64;
    let mut ram_bytes = 0u64;
    let mut baseline = HashMap::new();

    for pid in pids {
        let Some(current) = reader.sample(*pid) else {
            continue;
        };
        let delta = previous
            .get(pid)
            .map_or(0, |prev| current.cpu_ticks.saturating_sub(prev.cpu_ticks));
        tick_delta += delta;
        ram_bytes += current.rss_bytes;
        baseline.insert(*pid, current);
    }

    (
        TreeSnapshot {
            cpu_percent: cpu_percent(tick_delta, wall_ms, clock_ticks_per_sec),
            ram_bytes,
        },
        baseline,
    )
}

fn clock_ticks_per_second() -> u64 {
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks > 0 {
        ticks as u64
    } else {
        100
    }
}

/// Emits one Metrics event per refresh while the run is alive and stops when the finished
/// flag is set or the root pid stops answering. Each interval re-walks the tree so children
/// born or gone since the last interval are reflected in the sum. Reason for depth: the
/// poller must end exactly with the run, and its tests must drive it with fake samples
/// instead of real processes.
pub fn spawn_metrics_poller(
    reader: impl ProcReader + 'static,
    run_id: String,
    pid: u32,
    sink: Arc<dyn EventSink>,
    finished: Arc<AtomicBool>,
    refresh: Duration,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let clock = clock_ticks_per_second();
        let Some(first) = reader.sample(pid) else {
            return;
        };
        let mut baseline: HashMap<u32, ProcessSample> = HashMap::new();
        baseline.insert(pid, first);
        let mut previous_at = Instant::now();

        while !finished.load(Ordering::SeqCst) {
            thread::sleep(refresh);
            if finished.load(Ordering::SeqCst) {
                break;
            }

            let pids = tree_pids(&reader, pid);
            let now = Instant::now();
            let wall_ms = now.duration_since(previous_at).as_millis() as u64;
            let (snapshot, next_baseline) = tree_sum(&reader, &pids, &baseline, wall_ms, clock);

            // The root stopped answering: the run is over, the poller ends with it.
            if !next_baseline.contains_key(&pid) {
                break;
            }

            let mut processes: Vec<ProcessCost> = next_baseline
                .iter()
                .map(|(child_pid, current)| {
                    let delta = baseline
                        .get(child_pid)
                        .map_or(0, |prev| current.cpu_ticks.saturating_sub(prev.cpu_ticks));
                    ProcessCost {
                        pid: *child_pid,
                        cpu_percent: cpu_percent(delta, wall_ms, clock),
                        ram_bytes: current.rss_bytes,
                    }
                })
                .collect();
            processes.sort_by_key(|cost| cost.pid);
            processes.truncate(BREAKDOWN_MAX_PROCESSES);

            sink.emit(RunEvent::Metrics {
                run_id: run_id.clone(),
                cpu_percent: snapshot.cpu_percent,
                ram_bytes: snapshot.ram_bytes,
                processes,
            });
            baseline = next_baseline;
            previous_at = now;
        }
    })
}

/// The /proc reader built on the procfs crate (DEC-11). Field parsing across kernel
/// versions is the crate's job, not ours.
pub struct ProcfsReader;

impl ProcfsReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProcfsReader {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcReader for ProcfsReader {
    fn sample(&self, pid: u32) -> Option<ProcessSample> {
        let process = Process::new(pid as i32).ok()?;
        let stat = process.stat().ok()?;
        let status = process.status().ok()?;
        let rss_kib = status.vmrss?;
        Some(ProcessSample {
            cpu_ticks: stat.utime + stat.stime,
            rss_bytes: rss_kib * 1024,
        })
    }

    fn children(&self, pid: u32) -> Vec<u32> {
        Process::new(pid as i32)
            .ok()
            .and_then(|process| process.task_main_thread().ok())
            .and_then(|task| task.children().ok())
            .unwrap_or_default()
    }
}

/// Every live pid in the tree: the root plus all descendants, depth-first through the
/// /proc child lists.
pub fn tree_pids(reader: &dyn ProcReader, root: u32) -> Vec<u32> {
    let mut all = vec![root];
    let mut queue = vec![root];

    while let Some(pid) = queue.pop() {
        for child in reader.children(pid) {
            all.push(child);
            queue.push(child);
        }
    }

    all
}
