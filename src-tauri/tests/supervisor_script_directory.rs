// story: e01s03
// scenario: SC-e01s03-P0-01
mod common;

use common::TempRegistry;
use sh_ui_lib::supervisor::{OutputStream, Recorder, RunEvent, Supervisor};
use std::time::{Duration, Instant};

fn stdout_chunks(recorder: &Recorder) -> Vec<String> {
    recorder
        .events()
        .into_iter()
        .filter_map(|event| match event {
            RunEvent::Output {
                stream: OutputStream::Stdout,
                chunk,
                ..
            } => Some(chunk),
            _ => None,
        })
        .collect()
}

fn wait_for_end(recorder: &Recorder, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if recorder
            .events()
            .iter()
            .any(|event| matches!(event, RunEvent::Ended { .. }))
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn supervisor_starts_the_script_in_its_own_directory() {
    let temp = TempRegistry::new("cwd-dir");
    // A sibling file the script reads by a relative path, the way real scripts do.
    temp.write_file("data.txt", "read from the script directory\n");
    let script = temp.write_script("read.sh", "#!/bin/sh\ncat data.txt\npwd\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    supervisor.start("entry-1", &script).expect("spawn must succeed");
    assert!(
        wait_for_end(&recorder, Duration::from_secs(10)),
        "the script must finish"
    );

    let chunks = stdout_chunks(&recorder);

    assert_eq!(
        chunks,
        vec![
            "read from the script directory".to_string(),
            temp.dir_path().display().to_string(),
        ],
        "a relative path inside the script must resolve next to the script, not next to the host"
    );
}
