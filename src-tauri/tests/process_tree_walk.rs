// story: e03s02
// scenario: SC-e03s02-P1-01
mod common;

use common::TempRegistry;
use sh_ui_lib::metrics::{tree_pids, ProcReader, ProcfsReader};
use std::process::{Command, Stdio};
use std::time::Duration;

/// The trap makes the shell kill its own children when the test terminates it, so no
/// stray sleep survives the test run.
const FORKER: &str = concat!(
    "#!/bin/sh\n",
    "trap 'kill $(jobs -p) 2>/dev/null' EXIT INT TERM\n",
    "sleep 30 &\nsleep 30 &\nsleep 30 &\nwait\n",
);

#[test]
fn process_tree_walk_finds_live_children() {
    let temp = TempRegistry::new("tree-walk");
    let script = temp.write_script("forker.sh", FORKER);

    let mut child = Command::new(&script)
        .stdin(Stdio::null())
        .spawn()
        .expect("the forker must spawn");

    // Give the shell a moment to fork its three sleeps.
    std::thread::sleep(Duration::from_millis(400));

    let reader = ProcfsReader::new();
    let pids = tree_pids(&reader, child.id());

    let direct = reader.children(child.id());
    assert_eq!(
        direct.len(),
        3,
        "the forker must report its three live sleep children"
    );
    assert_eq!(
        pids.len(),
        4,
        "the walk must return the root plus every descendant"
    );
    assert!(pids.contains(&child.id()), "the walk must include the root");

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn process_tree_walk_returns_root_only_for_childless_pid() {
    let temp = TempRegistry::new("tree-walk-leaf");
    let script = temp.write_script("sleepy.sh", "#!/bin/sh\nsleep 30\n");

    let mut child = Command::new(&script)
        .stdin(Stdio::null())
        .spawn()
        .expect("the sleeper must spawn");

    let reader = ProcfsReader::new();
    let pids = tree_pids(&reader, child.id());

    assert_eq!(pids, vec![child.id()], "a childless pid walks alone");

    let _ = child.kill();
    let _ = child.wait();
}
