// story: e02s04
// scenario: SC-e02s04-P1-02
mod common;

use common::TempRegistry;
use sh_ui_lib::recovery::{load_records, reap_stale, start_time_ticks, LiveGroupRecord, LiveRunFile};
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::supervisor::{Recorder, RunEvent, Supervisor};
use std::time::{Duration, Instant};

fn alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// A fresh host must reap the groups a crashed host left, and must spare a record whose
/// start time no longer matches — a recycled pid is never killed (SC-e02s04-P1-02).
#[test]
fn crash_recovery_reaps() {
    let temp = TempRegistry::new("crash-reap");
    let state_path = temp.dir_path().join("live-runs.json");

    // A live group spawned the way the host spawns runs: its own process group.
    let script = temp.write_script("hold.sh", "#!/bin/sh\necho up\nsleep 300\n");
    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());
    let run = supervisor
        .start("entry-hold", &script, &RunConfig::default())
        .expect("hold run must start");

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && !recorder.events().iter().any(|event| match event {
        RunEvent::Output { chunk, .. } => chunk == "up",
        _ => false,
    }) {
        std::thread::sleep(Duration::from_millis(20));
    }

    let pid = run.pid;
    let recorded_start = start_time_ticks(pid).expect("the live pid start time must be readable");

    let file = LiveRunFile::new(&state_path);
    file.insert(&LiveGroupRecord {
        pgid: pid,
        pid,
        start_time: recorded_start,
    });

    // A record whose pid exists but whose start time does not match: pid 1 is init, it
    // started long before this test. Reaping must drop it without any kill.
    file.insert(&LiveGroupRecord {
        pgid: 1,
        pid: 1,
        start_time: recorded_start + 999_999,
    });

    let acted = reap_stale(&state_path, &|pgid, _pid| {
        unsafe { libc::killpg(pgid as libc::pid_t, libc::SIGKILL) };
    });

    assert_eq!(acted.len(), 1, "only the start-time-matched group may be reaped");
    assert_eq!(acted[0].pid, pid);

    let reap_deadline = Instant::now() + Duration::from_secs(5);
    while alive(pid) && Instant::now() < reap_deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!alive(pid), "the recorded live group must be dead after the reap");
    assert!(alive(1), "a record with a mismatched start time must not be killed");
    assert!(
        load_records(&state_path).is_empty(),
        "the sweep must clear the state file"
    );

    supervisor.shutdown();
}
