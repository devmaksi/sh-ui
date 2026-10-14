// story: e01s03
// scenario: SC-e01s03-P0-01
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::supervisor::{Recorder, Supervisor};

#[test]
fn supervisor_tracks_run() {
    let temp = TempRegistry::new("supervisor-tracks");
    let script = temp.write_script("sleep.sh", "#!/bin/sh\nsleep 30\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("a registered executable must spawn");

    assert_eq!(supervisor.live_run_ids(), vec![handle.run_id.clone()]);
    assert!(handle.pid > 0, "the run must report a real pid");
    assert!(
        std::path::Path::new(&format!("/proc/{}", handle.pid)).exists(),
        "the reported pid must be a live process"
    );

    supervisor.shutdown();

    assert!(
        supervisor.live_run_ids().is_empty(),
        "shutdown must clear every run from the supervisor"
    );
}
