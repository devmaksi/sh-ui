# Verification record: e04s02 — Say what cannot be measured

Date: 2026-08-14. Branch: `feat/e04s02-honest-gaps`. Automated gates passed; manual UAT
confirmed by the user the same day.

## Automated evidence

`npm run preflight` — exit 0: ESLint, both TypeScript projects, `cargo clippy --all-targets
-- -D warnings`, 41 vitest tests (14 files), 54 cargo tests (two consecutive full runs),
`npm run build` clean.

| Task | Verify command | Result |
|---|---|---|
| 1 unavailable carries a reason; nulls never zero | `cargo test … metric_unavailable` | pass — 3 tests (null serialization + reason; permission block named; absent tools named) |
| 2 labelled gap in the readout | `npm run test -- CostReadoutUnavailable` | pass — 2 tests (gap text + reason; no `cost-gpu`/`cost-vram` zeros; CPU and RAM still render) |
| 3 manual with tooling hidden | script below | pass — confirmed by the user |

## TDD evidence

- `4992f15` test → `8bd4371` feat — reason through probe and payload (red: `gpu_reason` field
  absent, `GpuProbeOutcome` absent).
- `52cb03d` test → `1a974c9` feat — gap rendering (red: no `cost-gpu-gap` testid).

Refactor inside GREEN: `probe_gpu_source` now returns `GpuProbeOutcome { source, reason }`
with reasons from a closed set — "no gpu tool found", "gpu tool failed to run", "gpu tool
needs permissions this app does not have". Raw vendor stderr never reaches the UI.

Flake hardening (boy scout): `proc_reader` and `process_tree_walk` hit the known ETXTBSY
transient while spawning scripts directly; both now retry 8× at 25 ms like the host does.
Two consecutive full cargo runs: 54 passed, 0 failed.

## Manual UAT — confirmed by the user on 2026-08-14

1. `/usr/bin/nvidia-smi` sits in the merged-`/bin` directory, so PATH subtraction would
   break `sh` and `sleep` for the app's own scripts. The honest equivalent was chosen: a
   shadow tool earlier in PATH — `/tmp/gpu-off/nvidia-smi` exiting 127.
2. `PATH="/tmp/gpu-off:$PATH" npm run tauri dev`, registered and started `/tmp/busy-loop.sh`.
3. CPU and RAM stayed live and updated.
4. The readout showed "GPU: not measurable — gpu tool failed to run". No zero appeared for
   GPU or VRAM.
5. Closing the app left no orphan.

User confirmed: "Подтверждаю".

## Suite at land time

54 cargo tests, 41 vitest tests, preflight exit 0. Epic e04 is complete with this story.
