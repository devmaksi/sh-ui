// story: e02s03
// scenario: SC-e02s03-P1-02
mod common;

use common::TempRegistry;
use sh_ui_lib::metrics::METRICS_REFRESH;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::supervisor::{Recorder, RunEvent, Supervisor};
use std::time::{Duration, Instant};

fn wait_for_ended(recorder: &Recorder, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if recorder
            .events()
            .iter()
            .any(|event| matches!(event, RunEvent::Ended { .. }))
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn final_state_event_arrives_once_and_no_metrics_follow_it() {
    let temp = TempRegistry::new("final_once");
    // Live long enough for the poller to report at least once (1 s cadence).
    let script = temp.write_script("slow.sh", "#!/bin/sh\nsleep 2\necho done\nexit 0\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    assert!(
        wait_for_ended(&recorder, Duration::from_secs(10)),
        "the run must end"
    );

    // Give a late metrics tick or a duplicate ended a chance to show up.
    std::thread::sleep(METRICS_REFRESH * 2);

    let events = recorder.events();
    let ended_count = events
        .iter()
        .filter(|event| matches!(event, RunEvent::Ended { .. }))
        .count();
    assert_eq!(ended_count, 1, "exactly one final state event per run");

    let ended_index = events
        .iter()
        .position(|event| matches!(event, RunEvent::Ended { .. }))
        .expect("ended event exists");
    let metrics_after = events[ended_index + 1..]
        .iter()
        .filter(|event| matches!(event, RunEvent::Metrics { .. }))
        .count();
    assert_eq!(
        metrics_after, 0,
        "the metrics poller must stop with the run: no metrics after the final state"
    );

    supervisor.shutdown();
}

#[test]
fn a_stopped_run_reports_its_final_state_once_too() {
    let temp = TempRegistry::new("final_stop");
    let script = temp.write_script("loop.sh", "#!/bin/sh\nsleep 300\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    supervisor.stop_run(&handle.run_id).expect("stop must succeed");
    assert!(wait_for_ended(&recorder, Duration::from_secs(10)));

    std::thread::sleep(METRICS_REFRESH * 2);

    let events = recorder.events();
    let ended_count = events
        .iter()
        .filter(|event| matches!(event, RunEvent::Ended { .. }))
        .count();
    assert_eq!(ended_count, 1, "a stopped run must also end exactly once");

    supervisor.shutdown();
}
