// story: e01s03
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::events::{EventSink, RunEvent};

pub const EVENT_STARTED: &str = "run://started";
pub const EVENT_OUTPUT: &str = "run://output";
pub const EVENT_ENDED: &str = "run://ended";
pub const EVENT_METRICS: &str = "run://metrics";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartedPayload {
    pub run_id: String,
    pub entry_id: String,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPayload {
    pub run_id: String,
    pub stream: &'static str,
    pub chunk: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndedPayload {
    pub run_id: String,
    pub code: Option<i32>,
    // story: e02s03
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCostPayload {
    pub pid: u32,
    pub cpu_percent: f64,
    pub ram_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsPayload {
    pub run_id: String,
    pub cpu_percent: f64,
    pub ram_bytes: u64,
    pub processes: Vec<ProcessCostPayload>,
    pub vram_bytes: Option<u64>,
    pub gpu_util_percent: Option<f64>,
    pub gpu_source: String,
    pub gpu_reason: Option<String>,
}

/// Turns supervisor events into app events. The supervisor itself never sees Tauri.
pub struct TauriRunSink {
    app: AppHandle,
}

impl TauriRunSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventSink for TauriRunSink {
    fn emit(&self, event: RunEvent) {
        match event {
            RunEvent::Started {
                run_id,
                entry_id,
                pid,
            } => {
                let _ = self.app.emit(
                    EVENT_STARTED,
                    StartedPayload {
                        run_id,
                        entry_id,
                        pid,
                    },
                );
            }
            RunEvent::Output {
                run_id,
                stream,
                chunk,
            } => {
                let _ = self.app.emit(
                    EVENT_OUTPUT,
                    OutputPayload {
                        run_id,
                        stream: stream.tag(),
                        chunk,
                    },
                );
            }
            RunEvent::Ended { run_id, code, status } => {
                let _ = self.app.emit(EVENT_ENDED, EndedPayload { run_id, code, status: status.tag() });
            }
            RunEvent::Metrics {
                run_id,
                cpu_percent,
                ram_bytes,
                processes,
                vram_bytes,
                gpu_util_percent,
                gpu_source,
                gpu_reason,
            } => {
                let _ = self.app.emit(
                    EVENT_METRICS,
                    MetricsPayload {
                        run_id,
                        cpu_percent,
                        ram_bytes,
                        vram_bytes,
                        gpu_util_percent,
                        gpu_source,
                        gpu_reason,
                        processes: processes
                            .into_iter()
                            .map(|cost| ProcessCostPayload {
                                pid: cost.pid,
                                cpu_percent: cost.cpu_percent,
                                ram_bytes: cost.ram_bytes,
                            })
                            .collect(),
                    },
                );
            }
        }
    }
}
