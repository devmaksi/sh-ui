// story: e03s01
// scenario: SC-e03s01-P1-01
mod common;

use common::TempRegistry;
use sh_ui_lib::metrics::{ProcReader, ProcfsReader};
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
fn proc_reader_reads_live_pid() {
    let temp = TempRegistry::new("proc-reader");
    let script = temp.write_script("busy.sh", "#!/bin/sh\nwhile :; do :; done\n");

    let mut child = Command::new(&script)
        .stdin(Stdio::null())
        .spawn()
        .expect("the busy script must spawn");

    let reader = ProcfsReader::new();

    let first = reader
        .sample(child.id())
        .expect("a live pid must yield a sample");
    assert!(
        first.rss_bytes > 0,
        "a live process must report resident memory"
    );

    std::thread::sleep(Duration::from_millis(300));

    let second = reader
        .sample(child.id())
        .expect("the pid must still be live");
    assert!(
        second.cpu_ticks > first.cpu_ticks,
        "a busy loop must consume cpu ticks between two samples"
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn proc_reader_returns_none_for_dead_pid() {
    let temp = TempRegistry::new("proc-reader-dead");
    let script = temp.write_script("quick.sh", "#!/bin/sh\nexit 0\n");

    let mut child = Command::new(&script)
        .stdin(Stdio::null())
        .spawn()
        .expect("the quick script must spawn");
    let pid = child.id();
    let _ = child.wait();

    let reader = ProcfsReader::new();
    assert!(
        reader.sample(pid).is_none(),
        "a reaped pid must report no sample"
    );
}
