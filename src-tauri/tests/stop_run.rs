// story: e02s02
// scenarios: SC-e02s02-P1-01, SC-e02s02-P1-03
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::supervisor::{Recorder, RunEvent, Supervisor};
use std::time::{Duration, Instant};

fn wait_for_end(recorder: &Recorder, timeout: Duration) -> Option<RunEvent> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(event) = recorder
            .events()
            .into_iter()
            .find(|event| matches!(event, RunEvent::Ended { .. }))
        {
            return Some(event);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    None
}

fn alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[test]
fn stop_sends_signal_to_the_whole_group() {
    let temp = TempRegistry::new("stop_group");
    // A loop that forks a grandchild, the way real watcher scripts do (DEC-07).
    let script = temp.write_script(
        "loop.sh",
        "#!/bin/sh\nsleep 300 & echo $! > grandchild.pid\nwait\n",
    );

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    let pid_file = temp.dir_path().join("grandchild.pid");
    let deadline = Instant::now() + Duration::from_secs(5);
    let grandchild = loop {
        if let Ok(text) = std::fs::read_to_string(&pid_file) {
            break text.trim().parse::<u32>().expect("pid fixture");
        }
        assert!(Instant::now() < deadline, "the script must fork");
        std::thread::sleep(Duration::from_millis(20));
    };

    assert!(alive(handle.pid) && alive(grandchild));

    supervisor
        .stop_run(&handle.run_id)
        .expect("stopping a live run must succeed");

    let ended = wait_for_end(&recorder, Duration::from_secs(6));
    assert!(
        ended.is_some(),
        "the stopped run must report its end like any other"
    );
    assert!(matches!(
        ended,
        Some(RunEvent::Ended {
            status: sh_ui_lib::supervisor::RunStatus::Stopped,
            ..
        })
    ));

    assert!(
        !alive(handle.pid) && !alive(grandchild),
        "one stop must reach every descendant of the script"
    );
}

#[test]
fn stop_run_refuses_unknown_and_finished_ids() {
    let temp = TempRegistry::new("stop_unknown");
    let script = temp.write_script("quick.sh", "#!/bin/sh\nexit 0\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let error = supervisor
        .stop_run("no-such-run")
        .expect_err("an unknown run id must be refused");
    assert_eq!(error.kind(), "runNotFound");

    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");
    assert!(
        wait_for_end(&recorder, Duration::from_secs(10)).is_some(),
        "the script must finish"
    );

    let error = supervisor
        .stop_run(&handle.run_id)
        .expect_err("a finished run must not be stoppable twice");
    assert_eq!(error.kind(), "runNotFound");
}
