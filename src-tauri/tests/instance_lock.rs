// story: e05s01
// scenario: SC-e05s01-P1-01, SC-e05s01-P1-02
mod common;

use common::TempRegistry;
use sh_ui_lib::instance::{InstanceError, acquire_instance_lock};

/// While one host holds the instance lock, no second host may take it — and after the
/// holder goes away (simulated exit: the File is dropped) the lock is acquirable again.
#[test]
fn lock_is_exclusive_while_held_and_free_after_release() {
    let temp = TempRegistry::new("instance-lock");
    let path = temp.dir_path().join("app.lock");

    let held = acquire_instance_lock(&path).expect("first acquire must succeed");

    let second = acquire_instance_lock(&path);
    match second {
        Err(InstanceError::AlreadyRunning(detail)) => {
            assert!(detail.contains("app.lock"), "refusal must name the lock path");
        }
        Err(other) => panic!("expected AlreadyRunning, got {other:?}"),
        Ok(_) => panic!("second acquire must fail while the lock is held"),
    }

    drop(held);
    acquire_instance_lock(&path).expect("acquire must succeed after the holder drops the file");
}

// story: e05s01
// scenario: SC-e05s01-P1-03
/// A lock held by a process that was SIGKILLed must not block the next host: the OS
/// releases flock when the holding process dies. The holder is a real external process
/// (flock(1)) so the release path is the OS one, not a Drop.
#[test]
fn lock_is_released_when_the_holding_process_is_killed() {
    let temp = TempRegistry::new("instance-lock-kill");
    let path = temp.dir_path().join("app.lock");

    let mut holder = std::process::Command::new("flock")
        .args([
            "-x",
            &path.display().to_string(),
            "-c",
            "sleep 30",
        ])
        .spawn()
        .expect("flock(1) must be available on Linux");

    // Wait until the external holder actually owns the lock.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        match acquire_instance_lock(&path) {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(InstanceError::AlreadyRunning(_)) => break,
            Err(other) => panic!("unexpected lock error: {other:?}"),
        }
        assert!(std::time::Instant::now() < deadline, "flock(1) never took the lock");
    }

    holder.kill().expect("killing the holder must work");
    holder.wait().expect("waiting for the killed holder must work");

    acquire_instance_lock(&path).expect("lock must be free after the holder was killed");
}
