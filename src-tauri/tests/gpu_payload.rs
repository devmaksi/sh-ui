// story: e04s01
// scenario: SC-e04s01-P1-01, SC-e04s01-P2-03
use sh_ui_lib::runs::{MetricsPayload, ProcessCostPayload};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("run-metrics-gpu.json")
}

#[test]
fn gpu_payload_matches_the_shared_fixture() {
    let fixture = std::fs::read_to_string(fixture_path())
        .expect("tests/fixtures/run-metrics-gpu.json must exist");

    let payload = MetricsPayload {
        run_id: "run-1".to_string(),
        cpu_percent: 12.5,
        ram_bytes: 1_048_576,
        processes: vec![ProcessCostPayload {
            pid: 4242,
            cpu_percent: 12.5,
            ram_bytes: 1_048_576,
        }],
        vram_bytes: Some(350 * 1024 * 1024),
        gpu_util_percent: Some(42.0),
        gpu_source: "nvidia-smi".to_string(),
        gpu_reason: None,
    };

    let produced: serde_json::Value =
        serde_json::to_value(&payload).expect("gpu payload must serialise");
    let expected: serde_json::Value =
        serde_json::from_str(&fixture).expect("fixture must be valid JSON");

    assert_eq!(
        produced, expected,
        "the gpu payload no longer matches the fixture the UI is tested against"
    );
}
