// story: e02s02
// scenario: SC-e02s02-P1-02
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::events::{Recorder, RunEvent};
use sh_ui_lib::spawn::TERM_GRACE;
use sh_ui_lib::supervisor::Supervisor;
use std::path::Path;
use std::time::{Duration, Instant};

#[test]
fn stop_escalates_to_sigkill_after_the_grace() {
    let temp = TempRegistry::new("stop_trap");
    // The script ignores SIGTERM, so only the escalation can end it.
    let script = temp.write_script(
        "stubborn.sh",
        "#!/bin/sh\ntrap '' TERM\nsleep 300\n",
    );

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    let started = Instant::now();
    supervisor
        .stop_run(&handle.run_id)
        .expect("stopping a live run must succeed");

    let deadline = started + TERM_GRACE + Duration::from_secs(3);
    // Ended follows the reap by design: watch_exit stops the metrics poller and joins
    // it before announcing the verdict, so wait for both, then assert each.
    let mut gone = false;
    let mut ended = false;
    while Instant::now() < deadline && !(gone && ended) {
        gone = !Path::new(&format!("/proc/{}", handle.pid)).exists();
        ended = recorder.events().into_iter().any(|event| {
            matches!(
                event,
                RunEvent::Ended {
                    status: sh_ui_lib::events::RunStatus::Stopped,
                    ..
                }
            )
        });
        if gone && ended {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    assert!(
        gone,
        "a SIGTERM-ignoring script must be gone within TERM_GRACE + KILL_GRACE, pid {} lived past {:?}",
        handle.pid,
        started.elapsed()
    );
    assert!(ended, "an escalated stop must still report its end");
}
