// story: e02s01
// scenario: SC-e02s01-P1-04
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::supervisor::{OutputStream, Recorder, RunEvent, Supervisor};
use std::collections::BTreeMap;
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
fn start_passes_stored_argv_as_separate_items() {
    let temp = TempRegistry::new("cfg_argv");
    let script = temp.write_script("args.sh", "#!/bin/sh\nprintf '%s\\n' \"$@\"\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let config = RunConfig {
        args: vec!["a b".to_string(), "--flag".to_string()],
        env: BTreeMap::new(),
        working_dir: None,
    };
    supervisor
        .start("entry-1", &script, &config)
        .expect("spawn must succeed");
    assert!(wait_for_end(&recorder, Duration::from_secs(10)));

    assert_eq!(
        stdout_chunks(&recorder),
        vec!["a b".to_string(), "--flag".to_string()],
        "argv items must arrive intact, spaces included, never as shell text"
    );
}

#[test]
fn start_applies_stored_environment_over_the_host_one() {
    let temp = TempRegistry::new("cfg_env");
    let script = temp.write_script("env.sh", "#!/bin/sh\necho \"$DEMO_VAR\"\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let config = RunConfig {
        args: vec![],
        env: BTreeMap::from([("DEMO_VAR".to_string(), "hello world".to_string())]),
        working_dir: None,
    };
    supervisor
        .start("entry-1", &script, &config)
        .expect("spawn must succeed");
    assert!(wait_for_end(&recorder, Duration::from_secs(10)));

    assert_eq!(
        stdout_chunks(&recorder),
        vec!["hello world".to_string()],
        "the stored environment must reach the script"
    );
}

#[test]
fn start_uses_the_stored_working_directory() {
    let temp = TempRegistry::new("cfg_cwd");
    let script_dir = temp.dir_path();
    let script = temp.write_script("where.sh", "#!/bin/sh\npwd\n");
    let other_dir = script_dir.join("elsewhere");
    std::fs::create_dir_all(&other_dir).expect("fixture dir must be creatable");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let config = RunConfig {
        args: vec![],
        env: BTreeMap::new(),
        working_dir: Some(other_dir.display().to_string()),
    };
    supervisor
        .start("entry-1", &script, &config)
        .expect("spawn must succeed");
    assert!(wait_for_end(&recorder, Duration::from_secs(10)));

    assert_eq!(
        stdout_chunks(&recorder),
        vec![other_dir.display().to_string()],
        "the stored directory must win over the script directory"
    );
}

#[test]
fn start_refuses_a_working_directory_deleted_after_the_save() {
    let temp = TempRegistry::new("cfg_cwd_gone");
    let script = temp.write_script("where.sh", "#!/bin/sh\npwd\n");
    let gone = temp.dir_path().join("gone");
    std::fs::create_dir_all(&gone).expect("fixture dir must be creatable");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    std::fs::remove_dir(&gone).expect("fixture dir must be removable");

    let config = RunConfig {
        args: vec![],
        env: BTreeMap::new(),
        working_dir: Some(gone.display().to_string()),
    };
    let error = supervisor
        .start("entry-1", &script, &config)
        .expect_err("a stored directory that vanished must be refused");

    assert_eq!(
        error.kind(),
        "badWorkingDir",
        "the refusal must be typed, not a raw spawn failure"
    );
}
