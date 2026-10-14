# Verification record: e04s01 — Report GPU and VRAM when the machine can measure them

Date: 2026-08-14. Branch: `feat/e04s01-gpu-report`. Automated gates passed; manual UAT
confirmed by the user the same day.

## Automated evidence

`npm run preflight` — exit 0: ESLint, both TypeScript projects, `cargo clippy --all-targets
-- -D warnings`, 39 vitest tests (13 files), 51 cargo tests, `npm run build` clean.

| Task | Verify command | Result |
|---|---|---|
| 1 probe in DEC-14 order, injected runner | `cargo test … gpu_probe` | pass — 4 tests (nvidia first, amd before legacy rocm, none → Unavailable) |
| 2 vram attribution over the tree | `cargo test … gpu_attribution` | pass — 2 tests (foreign pid excluded, multi-GPU pid summed) |
| 3 gpu fields in `run://metrics` | `cargo test … gpu_payload` | pass — fixture `run-metrics-gpu.json` matched; base fixture carries explicit nulls |
| 4 readout shows the figures | `npm run test -- CostReadoutGpu` | pass — 2 tests (vram MiB, device percent, source tag) |

## TDD evidence

- `2f1b2d3` test → `d9c9ace` feat — probe order (red: `gpu` module absent).
- `4b43245` test → `953eda5` feat — attribution (red: `attribute_vram` unresolved).
- `d92281b` test → `7a5ff01` feat — payload fields (red: `MetricsPayload` had no `vram_bytes`).
- `999091a` test → `679cdcd` feat — readout (red: no `cost-gpu`/`cost-vram` testids).

Refactors inside GREEN commits: the poller's gpu-source tag moved from a parameter to
`GpuProbe::source_tag` (clippy `too_many_arguments`), and `gpu_source_tag` no longer
re-probes the vendor tool per interval — the tag comes from the probe object itself.

## Manual UAT — confirmed by the user on 2026-08-14

1. `/tmp/gpu-cube.sh` — `timeout 20 vkcube`, a Vulkan workload.
2. Registered and started; the readout showed the GPU device percent with the
   `nvidia-smi` source tag and a VRAM field.
3. Cross-checked against `nvidia-smi --query-compute-apps=pid,used_gpu_memory --format=csv`.
4. Closing the app left no cube process.

Honest note recorded during UAT: `query-compute-apps` lists compute (CUDA) processes; a
graphics-only process such as `vkcube` can measure as zero VRAM. That is a vendor
limitation surfaced by design, and e04s02 renders the non-measurable case as a labelled gap.

## Suite at land time

51 cargo tests, 39 vitest tests, preflight exit 0.
