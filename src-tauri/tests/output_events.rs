// story: e01s03
// scenario: SC-e01s03-P0-02
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::events::{OutputStream, Recorder, RunEvent};
use sh_ui_lib::supervisor::{Supervisor};
use std::time::{Duration, Instant};

fn outputs_for(recorder: &Recorder, stream: OutputStream) -> Vec<String> {
    recorder
        .events()
        .into_iter()
        .filter_map(|event| match event {
            RunEvent::Output { stream: kind, chunk, .. } if kind == stream => Some(chunk),
            _ => None,
        })
        .collect()
}

fn ended_count(recorder: &Recorder) -> usize {
    recorder
        .events()
        .into_iter()
        .filter(|event| matches!(event, RunEvent::Ended { .. }))
        .count()
}

fn wait_for_end(recorder: &Recorder, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if ended_count(recorder) > 0 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn output_events_tag_every_chunk_with_its_stream() {
    let temp = TempRegistry::new("output-events");
    let script = temp.write_script(
        "both-streams.sh",
        "#!/bin/sh\necho out-1\necho err-1 >&2\necho out-2\n",
    );

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    supervisor.start("entry-1", &script, &RunConfig::default()).expect("spawn must succeed");

    assert!(
        wait_for_end(&recorder, Duration::from_secs(5)),
        "a script that finishes must report an ended event"
    );

    assert_eq!(
        outputs_for(&recorder, OutputStream::Stdout),
        vec!["out-1", "out-2"],
        "stdout chunks keep their tag and their order"
    );
    assert_eq!(
        outputs_for(&recorder, OutputStream::Stderr),
        vec!["err-1"],
        "stderr chunks never arrive tagged as stdout"
    );
}

#[test]
fn output_events_report_the_exit_code_of_a_clean_run() {
    let temp = TempRegistry::new("output-events-code");
    let script = temp.write_script("clean.sh", "#!/bin/sh\necho done\n");

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    let handle = supervisor
        .start("entry-1", &script, &RunConfig::default())
        .expect("spawn must succeed");

    assert!(
        wait_for_end(&recorder, Duration::from_secs(5)),
        "a script that finishes must report an ended event"
    );

    let ended = recorder
        .events()
        .into_iter()
        .find(|event| matches!(event, RunEvent::Ended { .. }))
        .expect("ended event");

    match ended {
        RunEvent::Ended { run_id, code, .. } => {
            assert_eq!(run_id, handle.run_id);
            assert_eq!(code, Some(0), "a clean exit must report its code");
        }
        other => panic!("expected an ended event, got {other:?}"),
    }
}
