// story: e01s03
// scenario: SC-e01s03-P0-03
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::events::{Recorder, RunEvent};
use sh_ui_lib::supervisor::{Supervisor};
use std::time::{Duration, Instant};

/// Every process whose PPid chain leads to `root`, read before the root is killed.
fn descendants(root: u32) -> Vec<u32> {
    let mut parents: Vec<u32> = vec![root];
    let mut found: Vec<u32> = Vec::new();

    while let Some(parent) = parents.pop() {
        let Ok(entries) = std::fs::read_dir("/proc") else { continue };
        for entry in entries.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
            let Ok(status) = std::fs::read_to_string(format!("/proc/{pid}/status")) else { continue };
            let ppid = status.lines().find_map(|line| {
                line.strip_prefix("PPid:")
                    .map(|rest| rest.trim().parse::<u32>().unwrap_or(0))
            });
            if ppid == Some(parent) && pid != root {
                found.push(pid);
                parents.push(pid);
            }
        }
    }

    found
}

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

#[test]
fn shutdown_kills_children() {
    let temp = TempRegistry::new("shutdown-tree");
    let script = temp.write_script(
        "tree.sh",
        "#!/bin/sh\necho ready\nsh -c 'sleep 300 & sleep 300' &\nsleep 300\n",
    );

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let handle = supervisor.start("entry-1", &script, &RunConfig::default()).expect("spawn must succeed");
    assert!(wait_for_output(&recorder, "ready", Duration::from_secs(5)));

    let tree = descendants(handle.pid);
    assert!(
        tree.len() >= 2,
        "the fixture must actually build a process tree, found {tree:?}"
    );

    supervisor.shutdown();

    assert!(!alive(handle.pid), "the run itself must not survive shutdown");
    for pid in &tree {
        assert!(!alive(*pid), "descendant {pid} survived shutdown");
    }
}

#[test]
fn shutdown_kills_a_child_that_ignores_sigterm() {
    let temp = TempRegistry::new("shutdown-stubborn");
    let script = temp.write_script("stubborn.sh", "#!/bin/sh\ntrap '' TERM\necho ready\nsleep 300\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let handle = supervisor.start("entry-1", &script, &RunConfig::default()).expect("spawn must succeed");
    assert!(wait_for_output(&recorder, "ready", Duration::from_secs(5)));

    supervisor.shutdown();

    assert!(
        !alive(handle.pid),
        "a run that ignores SIGTERM must still be killed"
    );
}
