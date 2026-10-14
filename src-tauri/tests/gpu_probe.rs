// story: e04s01
// scenario: SC-e04s01-P1-01
use sh_ui_lib::gpu::{probe_gpu_source, GpuSource};
use std::collections::HashSet;

/// A fake runner: tools in the answering set exit 0, everything else fails. No real
/// vendor tool is ever executed by these tests.
fn runner_answering(answering: &[&str]) -> impl Fn(&[&str]) -> Result<String, std::io::Error> {
    let answering: HashSet<String> = answering.iter().map(|name| name.to_string()).collect();
    move |argv: &[&str]| {
        let tool = argv.first().copied().unwrap_or("");
        if answering.contains(tool) {
            Ok("gpu, ok".to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{tool} is not installed"),
            ))
        }
    }
}

#[test]
fn gpu_probe_first_answerer_is_nvidia() {
    let runner = runner_answering(&["nvidia-smi", "amd-smi"]);
    let outcome = probe_gpu_source(&runner);
    assert!(matches!(outcome.source, GpuSource::NvidiaSmi));
    assert_eq!(outcome.source.tag(), "nvidia-smi");
}

#[test]
fn gpu_probe_order_reaches_amd_when_nvidia_absent() {
    let runner = runner_answering(&["amd-smi", "rocm-smi"]);
    let outcome = probe_gpu_source(&runner);
    assert!(matches!(outcome.source, GpuSource::AmdSmi), "amd-smi outranks rocm-smi");
    assert_eq!(outcome.source.tag(), "amd-smi");
}

#[test]
fn gpu_probe_legacy_rocm_when_modern_amd_absent() {
    let runner = runner_answering(&["rocm-smi"]);
    let outcome = probe_gpu_source(&runner);
    assert!(matches!(outcome.source, GpuSource::RocmSmi));
}

#[test]
fn gpu_probe_none_answering_is_unavailable() {
    let runner = runner_answering(&[]);
    let outcome = probe_gpu_source(&runner);
    assert!(matches!(outcome.source, GpuSource::Unavailable));
    assert_eq!(outcome.source.tag(), "unavailable");
}
