// story: e02s04
// scenario: SC-e02s04-P1-03
mod common;

use common::TempRegistry;
use sh_ui_lib::recovery::load_records;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::supervisor::{Recorder, RunEvent, Supervisor};
use std::time::{Duration, Instant};

/// A run ending naturally removes only its own record; a clean shutdown leaves the
/// live-runs file empty (SC-e02s04-P1-03).
#[test]
fn clean_exit_leaves_no_record() {
    let temp = TempRegistry::new("clean-exit-record");
    let state_path = temp.dir_path().join("live-runs.json");

    let short_script = temp.write_script("short.sh", "#!/bin/sh\necho done\nsleep 1\n");
    let long_script = temp.write_script("long.sh", "#!/bin/sh\necho up\nsleep 300\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone()).with_state_file(&state_path);

    let run_short = supervisor
        .start("entry-short", &short_script, &RunConfig::default())
        .expect("short run must start");
    let run_long = supervisor
        .start("entry-long", &long_script, &RunConfig::default())
        .expect("long run must start");

    // Both records exist while both runs are live.
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && load_records(&state_path).len() < 2 {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(load_records(&state_path).len(), 2);

    // The short run ends on its own; only its own record goes away.
    while Instant::now() < deadline
        && !recorder.events().iter().any(|event| match event {
            RunEvent::Ended { run_id, .. } => run_id == &run_short.run_id,
            _ => false,
        })
    {
        std::thread::sleep(Duration::from_millis(20));
    }

    let remaining = load_records(&state_path);
    assert_eq!(
        remaining.len(),
        1,
        "a natural end must remove only its own record, got {remaining:?}"
    );
    assert_eq!(remaining[0].pid, run_long.pid);

    supervisor.shutdown();

    while Instant::now() < deadline
        && !recorder.events().iter().any(|event| match event {
            RunEvent::Ended { run_id, .. } => run_id == &run_long.run_id,
            _ => false,
        })
    {
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(
        load_records(&state_path).is_empty(),
        "a clean shutdown must leave the live-runs file empty"
    );
}
