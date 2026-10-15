// story: e01s03
// scenario: SC-e01s03-P2-09
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::events::{Recorder, RunEvent};
use sh_ui_lib::supervisor::{Supervisor};
use std::time::{Duration, Instant};

fn ended_events(recorder: &Recorder) -> Vec<RunEvent> {
    recorder
        .events()
        .into_iter()
        .filter(|event| matches!(event, RunEvent::Ended { .. }))
        .collect()
}

#[test]
fn ended_event_once_for_a_normal_exit() {
    let temp = TempRegistry::new("ended-once");
    let script = temp.write_script("quick.sh", "#!/bin/sh\necho a\necho b\nexit 0\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    supervisor.start("entry-1", &script, &RunConfig::default()).expect("spawn must succeed");

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && ended_events(&recorder).is_empty() {
        std::thread::sleep(Duration::from_millis(20));
    }

    // Give any duplicate a chance to show up before judging the count.
    std::thread::sleep(Duration::from_millis(300));

    let ended = ended_events(&recorder);
    assert_eq!(ended.len(), 1, "a normal exit must produce exactly one ended event");

    supervisor.shutdown();

    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(
        ended_events(&recorder).len(),
        1,
        "shutting down an already finished run must not emit a second ended event"
    );
}
