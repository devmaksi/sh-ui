// story: e01s03
// scenario: SC-e01s03-P0-06
mod common;

use common::TempRegistry;
use sh_ui_lib::supervisor::{Recorder, RunEvent, Supervisor};
use std::time::{Duration, Instant};

fn output_count(recorder: &Recorder) -> usize {
    recorder
        .events()
        .iter()
        .filter(|event| matches!(event, RunEvent::Output { .. }))
        .count()
}

fn ended_count(recorder: &Recorder) -> usize {
    recorder
        .events()
        .iter()
        .filter(|event| matches!(event, RunEvent::Ended { .. }))
        .count()
}

#[test]
fn output_first_chunk_arrives_within_one_second() {
    let temp = TempRegistry::new("first-chunk");
    let script = temp.write_script("fast.sh", "#!/bin/sh\necho first\nsleep 3\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let started = Instant::now();
    supervisor.start("entry-1", &script).expect("spawn must succeed");

    let deadline = started + Duration::from_millis(1000);
    let mut latency: Option<Duration> = None;

    while Instant::now() < deadline {
        if output_count(&recorder) > 0 {
            latency = Some(started.elapsed());
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    let latency = latency.expect("the first chunk must arrive within 1000 ms");
    supervisor.shutdown();

    assert!(
        latency <= Duration::from_millis(1000),
        "first chunk took {latency:?} to arrive"
    );
}

#[test]
fn output_keeps_arriving_while_the_script_runs() {
    let temp = TempRegistry::new("keep-arriving");
    let script = temp.write_script(
        "ticks.sh",
        "#!/bin/sh\nfor i in 1 2 3 4; do echo \"tick-$i\"; sleep 0.4; done\n",
    );

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    supervisor.start("entry-1", &script).expect("spawn must succeed");

    std::thread::sleep(Duration::from_millis(700));
    let early = output_count(&recorder);

    assert!(
        early >= 2,
        "output must still be arriving while the script is running, got {early}"
    );
    assert_eq!(ended_count(&recorder), 0, "a live run must not report ended");
    assert_eq!(supervisor.live_run_ids().len(), 1, "the run must stay live");

    supervisor.shutdown();
}
