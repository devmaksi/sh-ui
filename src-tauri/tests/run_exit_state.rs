// story: e02s03
// scenario: SC-e02s03-P1-01
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::events::{Recorder, RunEvent};
use sh_ui_lib::supervisor::{Supervisor};
use std::time::{Duration, Instant};

fn wait_for_ended(recorder: &Recorder, timeout: Duration) -> (Option<i32>, &'static str) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        for event in recorder.events() {
            if let RunEvent::Ended { code, status, .. } = event {
                return (code, status.tag());
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("the run must end");
}

#[test]
fn exit_zero_is_exited() {
    let temp = TempRegistry::new("state_ok");
    let script = temp.write_script("ok.sh", "#!/bin/sh\nexit 0\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    assert_eq!(wait_for_ended(&recorder, Duration::from_secs(10)), (Some(0), "exited"));
}

#[test]
fn non_zero_exit_is_failed_with_the_code() {
    let temp = TempRegistry::new("state_fail");
    let script = temp.write_script("bad.sh", "#!/bin/sh\nexit 7\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    assert_eq!(wait_for_ended(&recorder, Duration::from_secs(10)), (Some(7), "failed"));
}

#[test]
fn user_stop_is_stopped() {
    let temp = TempRegistry::new("state_stop");
    let script = temp.write_script("loop.sh", "#!/bin/sh\nsleep 300\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    supervisor.stop_run(&handle.run_id).expect("stop must succeed");

    let (code, status) = wait_for_ended(&recorder, Duration::from_secs(10));
    assert_eq!(status, "stopped");
    assert!(code.is_none(), "a stopped run has no exit code to report");
}

#[test]
fn death_by_an_outside_signal_is_failed_without_a_code() {
    let temp = TempRegistry::new("state_signal");
    let script = temp.write_script("loop.sh", "#!/bin/sh\nsleep 300\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    // The whole group dies externally (the way an OOM-style killer reaches a setsid
    // leader): an orphan holding the output pipes would keep the run legitimately open.
    unsafe { libc::killpg(handle.pid as libc::pid_t, libc::SIGKILL) };

    let (code, status) = wait_for_ended(&recorder, Duration::from_secs(10));
    assert_eq!(status, "failed");
    assert!(code.is_none(), "a signal death has no exit code");
}
