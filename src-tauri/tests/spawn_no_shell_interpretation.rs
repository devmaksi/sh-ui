// story: e01s03
// scenario: SC-e01s03-P0-04
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
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
fn spawn_no_shell_interpretation_of_a_hostile_path() {
    let temp = TempRegistry::new("hostile-path");
    let name = "ouch $HOME `id` ; touch pwned .sh";
    let script = temp.write_script(name, "#!/bin/sh\necho \"ARG0=$0\"\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    supervisor.start("entry-1", &script, &RunConfig::default()).expect("spawn must succeed");
    assert!(
        wait_for_end(&recorder, Duration::from_secs(10)),
        "the run must end"
    );

    let chunks = stdout_chunks(&recorder);

    assert_eq!(
        chunks,
        vec![format!("ARG0={}", script.display())],
        "the whole name must reach the process as one literal argument"
    );

    assert!(
        !temp.dir_path().join("pwned").exists(),
        "the semicolon in the name must not run a second command"
    );
}
