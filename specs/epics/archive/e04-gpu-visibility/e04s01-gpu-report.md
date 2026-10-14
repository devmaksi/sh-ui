# e04s01 — Report GPU and VRAM when the machine can measure them

## 1. Identity

Story ID: e04s01. Title: Report GPU and VRAM when the machine can measure them. Epic:
e04-gpu-visibility. Status: planned.

## 2. Summary

At the start of a run the host probes vendor GPU tools in the fixed order of DEC-14 and
remembers which one answered. Each metrics interval the answering tool is asked for
per-process GPU memory; the numbers for pids inside the run's process tree are summed into
the run's VRAM figure, with the busiest device's utilization as context. The readout shows
them.

## 3. Classification

type: feat · risk: P1 · context: domain · bcps: 5

## 4. Value

The product promises CPU, RAM, VRAM and GPU. This story adds the two numbers the machine
can actually measure, without pretending to measure what it cannot.

## 5. Users and triggers

A live run on a machine with vendor GPU tooling. The GPU and VRAM figures refresh on the
same interval as CPU and RAM.

## 6. Domain language

**GPU probe** — one attempt to run a vendor tool and learn whether it answers, done once
per poller. **GPU source** — the tool that answered (`nvidia-smi`, `amd-smi`, `rocm-smi`,
`intel_gpu_top`) or `unavailable`. **VRAM attribution** — summing the vendor's per-process
GPU memory over the pids in the run's process tree. **Device utilization context** — the
maximum utilization across the machine's GPU devices; it is context, not an attribution.

## 7. Requirements

#### ADDED: The host probes GPU tools in a fixed order
nvidia-smi → amd-smi → rocm-smi → intel_gpu_top (the last only when it runs without root).
The first tool that answers becomes the GPU source for the run. No tool is ever invoked
with elevated privileges.

#### ADDED: VRAM is attributed to the run's process tree
Where the GPU source exposes per-process GPU memory, only pids inside the run's tree are
summed into the run's VRAM. Processes outside the tree are never counted.

#### ADDED: Device utilization is context, not attribution
The readout carries the busiest device's utilization percent alongside the run's VRAM. It
is labeled as device context because vendor tools do not attribute GPU time to processes
reliably (DEC-14, option A).

#### MODIFIED: The metrics snapshot carries GPU figures
Before: `run://metrics` carried `cpuPercent`, `ramBytes`, `processes`. After: it also
carries `vramBytes` (nullable), `gpuUtilPercent` (nullable) and `gpuSource` (the tool tag or
`unavailable`). Null means not measured — never zero.

## 8. Behaviour before

e03 shows summed CPU and RAM over the process tree. GPU and VRAM do not exist in any view.

## 9. Behaviour after

On a machine with GPU tooling, a live run shows a VRAM figure attributed to its tree and a
device utilization figure, refreshed on the same interval.

## 10. Interface and data contracts

New host module `gpu.rs`:
- `enum GpuSource { NvidiaSmi, AmdSmi, RocmSmi, IntelGpuTop, Unavailable }` with a `tag()`
  for the payload.
- `trait GpuProbe: Send + Sync { fn compute_apps(&self) -> Option<Vec<(u32, u64)>>; // pid → MiB
  fn device_util_max(&self) -> Option<f64>; }` — `None` means the source cannot answer that
  question, which the payload renders as null, never as zero.
- `fn probe_gpu_source(runner: &dyn Fn(&[&str]) -> Result<String, std::io::Error>) -> GpuSource`
  — the runner is injected so tests never execute real vendor tools.
- `fn attribute_vram(apps: &[(u32, u64)], tree_pids: &[u32]) -> u64` — summed MiB.

`spawn_metrics_poller` gains a `gpu: Arc<dyn GpuProbe>` argument; the supervisor builds it
once from `probe_gpu_source` at startup. `RunEvent::Metrics` and `run://metrics` gain
`vram_bytes: Option<u64>`, `gpu_util_percent: Option<f64>`, `gpu_source: String`.

## 11. Affected modules

`src-tauri/src/gpu.rs` — new. `src-tauri/src/metrics.rs` — poller queries attribution per
interval. `src-tauri/src/supervisor.rs` — carries the probe. `src-tauri/src/runs.rs` —
payload fields. `src/features/runs/` — readout columns.

## 12. Implementation steps

1. Probe with an injected runner: fixed order, first answerer wins, no answer → Unavailable → verify: `cargo test --manifest-path src-tauri/Cargo.toml gpu_probe`
2. Attribution: vendor per-process MiB summed over tree pids only → verify: `cargo test --manifest-path src-tauri/Cargo.toml gpu_attribution`
3. Poller carries `vramBytes`, `gpuUtilPercent`, `gpuSource` into `run://metrics` → verify: `cargo test --manifest-path src-tauri/Cargo.toml gpu_payload`
4. Readout shows GPU utilization and VRAM when measured → verify: `npm run test -- CostReadoutGpu`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e04s01-P1-01 | U-R | With a fake runner the probe picks nvidia-smi first; with nvidia failing it falls to amd-smi; with all failing the source is Unavailable |
| SC-e04s01-P1-02 | U-R | Only tree pids are summed; a foreign pid is excluded |
| SC-e04s01-P2-03 | U-R | Device utilization context is the max across devices |
| SC-e04s01-P2-04 | U-T | The readout renders GPU percent and VRAM MiB from a metrics event |
| SC-e04s01-P1-M1 | M | A human runs a GPU-using script and the VRAM figure roughly matches what the vendor tool reports for that pid |

## 14. Verification Script

1. On this machine (nvidia-smi present) register a script that uses the GPU, for example a
   python3 workload that allocates a device tensor, or any workload the user knows.
2. Start it.
3. Confirm the readout shows a VRAM figure above zero within a few intervals.
4. In another terminal run `nvidia-smi --query-compute-apps=pid,used_gpu_memory --format=csv`
   and confirm the readout figure matches the sum for the run's pids.
5. Confirm the GPU percent is labeled as device context, not as the script's own usage.

## 15. Out of scope

Per-process GPU time (pmon parsing), AMD/Intel live testing (tools absent on the dev
machine — the probe paths are unit-tested with fake runners), GPU history, cross-machine
measurement, anything requiring root.

## 16. Risks

- Vendor tools report only compute processes; a graphics-only script shows no VRAM. That is
  a vendor limit, surfaced honestly via e04s02, not papered over.
- Running `nvidia-smi` every interval costs ~100–300 ms. Acceptable at a 1 s cadence; the
  probe itself runs once, not per interval.
- A vendor tool present but broken (driver mismatch) must degrade to Unavailable, not hang
  the poller: every probe call has a short timeout and any failure answers None.

## 17. Acceptance criteria

```gherkin
# SC-e04s01-P1-01
Scenario: The first answering tool wins
  Given a fake runner where nvidia-smi answers and amd-smi would too
  When the host probes
  Then the gpu source is nvidia-smi

# SC-e04s01-P1-02
Scenario: Only the run's tree counts
  Given compute apps listing the run's two pids and one foreign pid
  When the host attributes vram
  Then the figure is the sum of the two tree pids only

# SC-e04s01-P2-04
Scenario: The readout carries the gpu figures
  Given a metrics event with vram and device utilization
  When the runs view renders
  Then both figures appear
```

## 18. Dependencies and prior art

`nvidia-smi` verified live on the dev machine: `--query-compute-apps=pid,used_gpu_memory
--format=csv` yields pid + MiB per GPU; `--query-gpu=utilization.gpu` yields device percent.
`amd-smi process` (ROCm 7+) and deprecated `rocm-smi` per the ROCm changelog;
`intel_gpu_top` needs CAP_PERFMON per the nvtop README. No new Rust dependency: vendor
tools are invoked as argv arrays through `Command`, same discipline as script spawning.
Prior art: telegraf's nvidia_smi input plugin parses the same CSV.

## 19. Error surfaces

A probe tool that errors, times out, or needs root answers Unavailable with a reason
(e04s02). A compute-apps query that fails mid-run yields null for that interval; the CPU
and RAM figures keep flowing.

## 20. Follow-ups

The explicit unavailable rendering arrives in e04s02.
