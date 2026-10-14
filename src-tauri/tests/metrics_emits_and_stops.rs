// story: e03s01
// scenario: SC-e03s01-P1-02, SC-e03s01-P1-03
mod common;

use common::TempRegistry;
use sh_ui_lib::metrics::{spawn_metrics_poller, ProcReader, ProcessSample};
use sh_ui_lib::supervisor::{Recorder, RunEvent, Supervisor};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn metrics_count(events: &[RunEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, RunEvent::Metrics { .. }))
        .count()
}

/// Deterministic samples so the poller tests never race real processes.
struct FakeReader(Mutex<VecDeque<ProcessSample>>);

impl FakeReader {
    fn new(samples: Vec<ProcessSample>) -> Self {
        Self(Mutex::new(samples.into()))
    }
}

impl ProcReader for FakeReader {
    fn sample(&self, _pid: u32) -> Option<ProcessSample> {
        self.0.lock().expect("fake reader lock").pop_front()
    }
}

#[test]
fn metrics_emits_and_stops_with_supervisor() {
    let temp = TempRegistry::new("metrics-supervisor");
    let script = temp.write_script("sleep.sh", "#!/bin/sh\nsleep 30\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    supervisor
        .start("entry-1", &script)
        .expect("the sleep script must spawn");

    std::thread::sleep(Duration::from_millis(1300));
    assert!(
        metrics_count(&recorder.events()) >= 1,
        "a live run must produce at least one metrics event per interval"
    );

    supervisor.shutdown();
    std::thread::sleep(Duration::from_millis(100));
    let settled = metrics_count(&recorder.events());

    std::thread::sleep(Duration::from_millis(1300));
    assert_eq!(
        metrics_count(&recorder.events()),
        settled,
        "no metrics event may arrive after the run ended"
    );
}

#[test]
fn metrics_emits_and_stops_poller_stops_when_sample_gone() {
    let recorder = Recorder::default();
    let finished = Arc::new(AtomicBool::new(false));
    let reader = FakeReader::new(vec![
        ProcessSample { cpu_ticks: 0, rss_bytes: 1000 },
        ProcessSample { cpu_ticks: 100, rss_bytes: 1000 },
        ProcessSample { cpu_ticks: 200, rss_bytes: 2000 },
    ]);

    let handle = spawn_metrics_poller(
        reader,
        "run-1".to_string(),
        4242,
        Arc::new(recorder.clone()),
        finished,
        Duration::from_millis(50),
    );
    handle.join().expect("poller must exit when the pid is gone");

    let events = recorder.events();
    let metrics: Vec<&RunEvent> = events
        .iter()
        .filter(|event| matches!(event, RunEvent::Metrics { .. }))
        .collect();
    assert_eq!(metrics.len(), 2, "one event per sample after the baseline");
    assert!(
        matches!(
            metrics[1],
            RunEvent::Metrics { ram_bytes: 2000, .. }
        ),
        "the last sample must carry the newest rss"
    );
}

#[test]
fn metrics_emits_and_stops_poller_stops_on_finished() {
    struct EndlessReader;
    impl ProcReader for EndlessReader {
        fn sample(&self, _pid: u32) -> Option<ProcessSample> {
            Some(ProcessSample { cpu_ticks: 10, rss_bytes: 1000 })
        }
    }

    let recorder = Recorder::default();
    let finished = Arc::new(AtomicBool::new(false));

    let handle = spawn_metrics_poller(
        EndlessReader,
        "run-1".to_string(),
        4242,
        Arc::new(recorder.clone()),
        finished.clone(),
        Duration::from_millis(50),
    );

    std::thread::sleep(Duration::from_millis(160));
    assert!(
        metrics_count(&recorder.events()) >= 2,
        "the poller must emit on every interval while the run is alive"
    );

    finished.store(true, Ordering::SeqCst);
    handle.join().expect("poller must exit when the run is finished");

    let settled = metrics_count(&recorder.events());
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(
        metrics_count(&recorder.events()),
        settled,
        "a finished run must not keep emitting metrics"
    );
}
