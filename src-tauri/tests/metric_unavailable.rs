// story: e04s02
// scenario: SC-e04s02-P0-01
use sh_ui_lib::gpu::{probe_gpu_source, GpuSource};
use sh_ui_lib::runs::MetricsPayload;

#[test]
fn metric_unavailable_serializes_as_null_with_a_reason() {
    let payload = MetricsPayload {
        run_id: "run-1".to_string(),
        cpu_percent: 10.0,
        ram_bytes: 1000,
        processes: vec![],
        vram_bytes: None,
        gpu_util_percent: None,
        gpu_source: "unavailable".to_string(),
        gpu_reason: Some("no gpu tool found".to_string()),
    };

    let produced: serde_json::Value =
        serde_json::to_value(&payload).expect("payload must serialise");

    assert_eq!(produced["vramBytes"], serde_json::Value::Null);
    assert_eq!(produced["gpuUtilPercent"], serde_json::Value::Null);
    assert_eq!(produced["gpuReason"], "no gpu tool found");
    assert!(
        produced.get("vramBytes") != Some(&serde_json::Value::from(0)),
        "unknown must never serialize as zero"
    );
}

#[test]
fn metric_unavailable_probe_names_a_permission_block() {
    let runner = |argv: &[&str]| -> Result<String, std::io::Error> {
        let tool = argv.first().copied().unwrap_or("");
        if tool == "intel_gpu_top" {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "needs cap_perfmon",
            ))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{tool} is not installed"),
            ))
        }
    };

    let outcome = probe_gpu_source(&runner);
    assert!(matches!(outcome.source, GpuSource::Unavailable));
    assert_eq!(
        outcome.reason,
        Some("gpu tool needs permissions this app does not have")
    );
}

#[test]
fn metric_unavailable_probe_names_absent_tools() {
    let runner = |_argv: &[&str]| -> Result<String, std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not installed",
        ))
    };

    let outcome = probe_gpu_source(&runner);
    assert_eq!(outcome.reason, Some("no gpu tool found"));
}
