// story: e01s03
use std::sync::{Arc, Mutex};

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
        // story: e02s03 — one honest verdict per finished run, derived in watch_exit.
        status: RunStatus,
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

/// story: e02s03 — how a run ended. exited: code 0. failed: non-zero code or death by
/// an unrequested signal (code None). stopped: the user pressed stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Exited,
    Failed,
    Stopped,
}

impl RunStatus {
    /// Stable tag for the UI payload union.
    pub fn tag(self) -> &'static str {
        match self {
            RunStatus::Exited => "exited",
            RunStatus::Failed => "failed",
            RunStatus::Stopped => "stopped",
        }
    }
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
