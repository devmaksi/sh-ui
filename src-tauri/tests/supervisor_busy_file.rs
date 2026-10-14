// story: e01s03
// scenario: SC-e01s03-P0-01
mod common;

use common::TempRegistry;
use sh_ui_lib::supervisor::{Recorder, StartError, Supervisor};

#[test]
fn supervisor_retries_while_the_script_file_is_busy() {
    let temp = TempRegistry::new("busy-retry");
    let script = temp.write_script("busy.sh", "#!/bin/sh\nsleep 5\n");

    // A write handle on the script makes exec fail with ETXTBSY. This is what an editor
    // saving the file, or a backup tool copying it, looks like to the host. The handle is
    // released after 60 ms, well inside the host's retry window.
    let busy_path = script.clone();
    let holder = std::thread::spawn(move || {
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .open(&busy_path)
            .expect("write handle on the script");
        std::thread::sleep(std::time::Duration::from_millis(60));
        drop(writer);
    });

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let handle = supervisor
        .start("entry-1", &script)
        .expect("a briefly busy script must still start");

    assert!(handle.pid > 0);

    holder.join().expect("holder thread");
    supervisor.shutdown();
}

#[test]
fn supervisor_reports_a_script_that_stays_busy() {
    let temp = TempRegistry::new("busy-forever");
    let script = temp.write_script("stuck.sh", "#!/bin/sh\nsleep 5\n");

    let writer = std::fs::OpenOptions::new()
        .write(true)
        .open(&script)
        .expect("write handle on the script");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let error = supervisor
        .start("entry-1", &script)
        .expect_err("a script that never frees must be reported");

    match error {
        StartError::SpawnFailed(detail) => {
            assert!(
                detail.contains("Text file busy"),
                "the report must name the real reason, got {detail}"
            );
        }
        other => panic!("expected spawnFailed, got {other:?}"),
    }

    assert!(supervisor.live_run_ids().is_empty());
    assert!(recorder.events().is_empty(), "a failed start must emit no events");

    drop(writer);
}
