// story: e03s01
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

fn clock_ticks_per_second() -> u64 {
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks > 0 {
        ticks as u64
    } else {
        100
    }
}

/// Emits one Metrics event per refresh while the run is alive and stops when the finished
/// flag is set or the pid stops answering. Reason for depth: the poller must end exactly
/// with the run, and its tests must drive it with fake samples instead of real processes.
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
        let Some(mut previous) = reader.sample(pid) else {
            return;
        };
        let mut previous_at = Instant::now();

        while !finished.load(Ordering::SeqCst) {
            thread::sleep(refresh);
            if finished.load(Ordering::SeqCst) {
                break;
            }
            let Some(current) = reader.sample(pid) else {
                break;
            };
            let now = Instant::now();
            let wall_ms = now.duration_since(previous_at).as_millis() as u64;
            let percent = cpu_percent(
                current.cpu_ticks.saturating_sub(previous.cpu_ticks),
                wall_ms,
                clock,
            );
            sink.emit(RunEvent::Metrics {
                run_id: run_id.clone(),
                cpu_percent: percent,
                ram_bytes: current.rss_bytes,
            });
            previous = current;
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
}
