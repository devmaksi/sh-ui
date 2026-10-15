// story: e02s04
// scenario: SC-e02s04-P1-01
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::events::{Recorder, RunEvent, RunStatus};
use sh_ui_lib::supervisor::{Supervisor};
use std::time::{Duration, Instant};

fn alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

fn wait_for_output(recorder: &Recorder, needle: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let hit = recorder.events().iter().any(|event| match event {
            RunEvent::Output { chunk, .. } => chunk.contains(needle),
            _ => false,
        });
        if hit {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

/// Shutdown must end every live run the way a user stop does: one Ended event per run
/// with status stopped, and the process group gone (SC-e02s04-P1-01).
#[test]
fn shutdown_stops_all() {
    let temp = TempRegistry::new("shutdown-stops-all");
    let script_a = temp.write_script("a.sh", "#!/bin/sh\necho a-ready\nsleep 300\n");
    let script_b = temp.write_script("b.sh", "#!/bin/sh\necho b-ready\nsleep 300\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let run_a = supervisor
        .start("entry-a", &script_a, &RunConfig::default())
        .expect("run a must start");
    let run_b = supervisor
        .start("entry-b", &script_b, &RunConfig::default())
        .expect("run b must start");

    assert!(wait_for_output(&recorder, "a-ready", Duration::from_secs(5)));
    assert!(wait_for_output(&recorder, "b-ready", Duration::from_secs(5)));

    supervisor.shutdown();

    // Ended is emitted after both readers drain, so wait for both verdicts.
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if recorder
            .events()
            .iter()
            .filter(|event| matches!(event, RunEvent::Ended { .. }))
            .count()
            >= 2
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let ended: Vec<(String, RunStatus)> = recorder
        .events()
        .iter()
        .filter_map(|event| match event {
            RunEvent::Ended { run_id, status, .. } => Some((run_id.clone(), *status)),
            _ => None,
        })
        .collect();

    assert_eq!(ended.len(), 2, "each live run must emit exactly one Ended event");
    assert!(
        ended.iter().all(|(_, status)| *status == RunStatus::Stopped),
        "shutdown must end every run as stopped, got {ended:?}"
    );
    assert!(!alive(run_a.pid), "run a must not survive shutdown");
    assert!(!alive(run_b.pid), "run b must not survive shutdown");
}
