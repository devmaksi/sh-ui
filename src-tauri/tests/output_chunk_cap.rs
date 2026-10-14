// story: e01s03
// scenario: SC-e01s03-P1-05
mod common;

use common::TempRegistry;
use sh_ui_lib::registry::RunConfig;
use sh_ui_lib::supervisor::{OutputStream, Recorder, RunEvent, Supervisor, OUTPUT_CHUNK_MAX_BYTES};
use std::time::{Duration, Instant};

fn chunks_for(recorder: &Recorder) -> Vec<String> {
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
fn output_chunk_cap_splits_one_oversized_line() {
    let long_line = "x".repeat(OUTPUT_CHUNK_MAX_BYTES * 12);
    let temp = TempRegistry::new("chunk-cap");
    let script = temp.write_script("long.sh", &format!("#!/bin/sh\necho '{long_line}'\n"));

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    supervisor.start("entry-1", &script, &RunConfig::default()).expect("spawn must succeed");
    assert!(
        wait_for_end(&recorder, Duration::from_secs(10)),
        "the run must end"
    );

    let chunks = chunks_for(&recorder);

    assert!(chunks.len() > 1, "an oversized line must arrive as several chunks");
    for chunk in &chunks {
        assert!(
            chunk.len() <= OUTPUT_CHUNK_MAX_BYTES,
            "a chunk of {} bytes passed the cap of {OUTPUT_CHUNK_MAX_BYTES}",
            chunk.len()
        );
    }
    assert_eq!(
        chunks.concat(),
        long_line,
        "splitting must not lose or reorder bytes"
    );
}

#[test]
fn output_chunk_cap_survives_a_fast_burst() {
    let temp = TempRegistry::new("chunk-burst");
    let script = temp.write_script(
        "burst.sh",
        "#!/bin/sh\nfor i in $(seq 1 5000); do echo \"line-$i\"; done\n",
    );

    let recorder = Recorder::default();
    let mut supervisor = Supervisor::new(recorder.clone());

    supervisor.start("entry-1", &script, &RunConfig::default()).expect("spawn must succeed");
    assert!(
        wait_for_end(&recorder, Duration::from_secs(20)),
        "the burst must finish"
    );

    let chunks = chunks_for(&recorder);
    let expected: Vec<String> = (1..=5000).map(|index| format!("line-{index}")).collect();

    assert_eq!(chunks.len(), 5000, "every line of the burst must arrive");
    assert_eq!(chunks, expected, "the burst must arrive in order, none dropped");
}
