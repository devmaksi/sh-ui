// story: e03s01 e03s02
// scenario: SC-e03s02-P2-04
use sh_ui_lib::runs::{MetricsPayload, ProcessCostPayload, EVENT_METRICS};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("run-metrics.json")
}

#[test]
fn metrics_event_payload_matches_the_shared_fixture() {
    let fixture = std::fs::read_to_string(fixture_path())
        .expect("tests/fixtures/run-metrics.json must exist");

    let payload = MetricsPayload {
        run_id: "run-1".to_string(),
        cpu_percent: 12.5,
        ram_bytes: 1_048_576,
        processes: vec![ProcessCostPayload {
            pid: 4242,
            cpu_percent: 12.5,
            ram_bytes: 1_048_576,
        }],
        vram_bytes: None,
        gpu_util_percent: None,
        gpu_source: "unavailable".to_string(),
    };

    let produced: serde_json::Value =
        serde_json::to_value(&payload).expect("metrics payload must serialise");
    let expected: serde_json::Value =
        serde_json::from_str(&fixture).expect("fixture must be valid JSON");

    assert_eq!(
        produced, expected,
        "the metrics payload no longer matches the fixture the UI is tested against"
    );
}

#[test]
fn metrics_event_name_is_run_metrics() {
    assert_eq!(EVENT_METRICS, "run://metrics");
}
