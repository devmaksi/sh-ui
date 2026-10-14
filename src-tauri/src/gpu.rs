// story: e04s01
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The vendor tool that answered the probe, or none (DEC-14).
pub enum GpuSource {
    NvidiaSmi,
    AmdSmi,
    RocmSmi,
    IntelGpuTop,
    Unavailable,
}

impl GpuSource {
    /// Stable tag for the UI. Changing a tag here changes the TypeScript union.
    pub fn tag(&self) -> &'static str {
        match self {
            GpuSource::NvidiaSmi => "nvidia-smi",
            GpuSource::AmdSmi => "amd-smi",
            GpuSource::RocmSmi => "rocm-smi",
            GpuSource::IntelGpuTop => "intel_gpu_top",
            GpuSource::Unavailable => "unavailable",
        }
    }
}

pub type ToolRunner = dyn Fn(&[&str]) -> Result<String, std::io::Error> + Send + Sync;

/// DEC-14 order: nvidia-smi, amd-smi, rocm-smi (legacy), intel_gpu_top last and only
/// when it answers without elevated privileges — a root-only tool answers an error here.
/// The runner is injected so tests never execute real vendor tools.
pub fn probe_gpu_source(runner: &ToolRunner) -> GpuSource {
    if runner(&["nvidia-smi", "--query-gpu=name", "--format=csv,noheader"]).is_ok() {
        return GpuSource::NvidiaSmi;
    }
    if runner(&["amd-smi", "static"]).is_ok() {
        return GpuSource::AmdSmi;
    }
    if runner(&["rocm-smi", "--version"]).is_ok() {
        return GpuSource::RocmSmi;
    }
    if runner(&["intel_gpu_top", "--list"]).is_ok() {
        return GpuSource::IntelGpuTop;
    }
    GpuSource::Unavailable
}

/// Sums vendor per-process GPU memory (MiB) over the pids in the run's process tree.
/// A pid listed once per GPU it uses is summed per appearance — same semantics as the
/// vendor tool itself.
pub fn attribute_vram(apps: &[(u32, u64)], tree_pids: &[u32]) -> u64 {
    apps.iter()
        .filter(|(pid, _)| tree_pids.contains(pid))
        .map(|(_, mib)| *mib)
        .sum()
}

/// Asks the GPU source the two questions the readout needs. `None` means this source
/// cannot answer that question — the payload renders null, never zero.
pub trait GpuProbe: Send + Sync {
    /// Which tool this probe stands for; becomes the payload's gpuSource tag.
    fn source_tag(&self) -> &'static str;

    /// Vendor per-process GPU memory: pid → MiB, as the tool lists it.
    fn compute_apps(&self) -> Option<Vec<(u32, u64)>>;

    /// The busiest device's utilization percent, as context — not an attribution.
    fn device_util_max(&self) -> Option<f64>;
}

/// The source that answers nothing. Metrics stay null.
pub struct UnavailableGpuProbe;

impl GpuProbe for UnavailableGpuProbe {
    fn source_tag(&self) -> &'static str {
        "unavailable"
    }

    fn compute_apps(&self) -> Option<Vec<(u32, u64)>> {
        None
    }

    fn device_util_max(&self) -> Option<f64> {
        None
    }
}

/// nvidia-smi-backed probe. CSV with no header and no units: "pid, MiB" rows and one
/// percent per device.
pub struct NvidiaGpuProbe;

impl GpuProbe for NvidiaGpuProbe {
    fn source_tag(&self) -> &'static str {
        "nvidia-smi"
    }

    fn compute_apps(&self) -> Option<Vec<(u32, u64)>> {
        let csv = run_tool(&[
            "nvidia-smi",
            "--query-compute-apps=pid,used_gpu_memory",
            "--format=csv,noheader,nounits",
        ])
        .ok()?;

        let mut apps = Vec::new();
        for line in csv.lines() {
            let (pid, mib) = line.split_once(',')?;
            apps.push((pid.trim().parse().ok()?, mib.trim().parse().ok()?));
        }
        Some(apps)
    }

    fn device_util_max(&self) -> Option<f64> {
        let csv = run_tool(&[
            "nvidia-smi",
            "--query-gpu=utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .ok()?;

        let mut max: Option<f64> = None;
        for line in csv.lines() {
            let Ok(percent) = line.trim().parse() else { continue };
            max = Some(max.map_or(percent, |current: f64| current.max(percent)));
        }
        max
    }
}

/// Only nvidia per-process data is implemented for now; the other sources answer null,
/// which the UI renders as an honest gap (e04s02).
pub fn build_gpu_probe(source: &GpuSource) -> std::sync::Arc<dyn GpuProbe> {
    match source {
        GpuSource::NvidiaSmi => std::sync::Arc::new(NvidiaGpuProbe),
        _ => std::sync::Arc::new(UnavailableGpuProbe),
    }
}

/// Runs a vendor tool as an argv array with a short timeout. A tool that hangs, is
/// missing, or fails answers an io::Error — never a panic, never elevated privileges.
pub fn run_tool(argv: &[&str]) -> Result<String, std::io::Error> {
    let (tool, args) = argv
        .split_first()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty argv"))?;

    let mut child = Command::new(tool)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            if !status.success() {
                return Err(std::io::Error::other(format!(
                    "{tool} exited with {status}"
                )));
            }
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }

        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("{tool} timed out"),
            ));
        }

        std::thread::sleep(Duration::from_millis(20));
    }
}
