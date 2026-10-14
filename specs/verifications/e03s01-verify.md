# Verification record: e03s01 — Live CPU and RAM of the running script

Date: 2026-08-14. Branch: `feat/e03s01-live-cost`. Automated gates passed; manual UAT
confirmed by the user the same day.

## Automated evidence

`npm run preflight` — exit 0: ESLint, both TypeScript projects, `cargo clippy --all-targets
-- -D warnings`, 35 vitest tests (11 files), 39 cargo tests, `npm run build` clean.

| Task | Verify command | Result |
|---|---|---|
| 1 procfs + metrics module, real-pid sample | `cargo test … proc_reader` | pass — 2 tests (live pid yields rss>0 and growing ticks; dead pid yields None) |
| 2 cpu percent math, named interval | `cargo test … cpu_percent_math` | pass — 4 tests (100%, 250%, 50%, zero-wall guard) |
| 3 poller emits per interval, stops at run end | `cargo test … metrics_emits_and_stops` | pass — 3 tests (supervisor end-to-end, sample-gone stop, finished-flag stop) |
| 4 `run://metrics` payload contract | `cargo test … metrics_event_payload` | pass — 2 tests against `tests/fixtures/run-metrics.json` |
| 5 live readout in the UI | `npm run test -- CostReadout` | pass — 2 tests; `useRuns` grew a metrics test (6 total) |

## TDD evidence

Every behavior is a test-only RED commit followed by a GREEN commit;
`bash scripts/verify-tdd-red-commit.sh` reported PASS for each RED in isolation:

- `7e0badb` test → `d1e9d22` feat — proc reader (red: `metrics` module did not exist).
- `6e9ccd7` test → `40cf300` feat — cpu percent math (red: `cpu_percent` unresolved).
- `dc334af` test → `6688809` feat — metrics event payload (red: `MetricsPayload`/`EVENT_METRICS` unresolved).
- `9cf568f` test → `69cc25f` feat — poller lifecycle (red: `spawn_metrics_poller` unresolved).
- `21122df` test → `6285153` feat — cost readout UI (red: `CostReadout`/`MetricsListener` missing).

Note: task 4 was executed before task 3 deliberately — implementing the poller first would
have made the payload test pass without a genuine red.

## Defects found and fixed on the way

1. `bp-timing.sh` overwrote `specs/state.yaml` with an empty skeleton because its YAML load
   silently swallowed a pre-existing parse error (unescaped apostrophe in `known_findings`).
   State restored from git; quoting fixed; logged as BUG-20260814-06.
2. Kebab-case vitest verify tokens (`cost-readout`, and the e01-era `output-panel`) match no
   file under vitest 5's case-sensitive filters. Active e03 task YAMLs now use exact PascalCase
   file stems. Logged as BUG-20260814-07.
3. Clippy `new_without_default` on `ProcfsReader` — added `Default`.
4. `RunEvent` lost its `Eq` derive because the metrics variant carries `f64`; `PartialEq`
   remains, all existing event comparisons still compile.

## Manual UAT — confirmed by the user on 2026-08-14

1. `/tmp/busy-loop.sh` — a one-core busy loop, mode 0755.
2. `npm run tauri dev`, register the file, press start.
3. CPU rose above 0 and held near 100%, refreshing once per second.
4. RAM showed a plausible resident figure, not 0.
5. Closing the app left the numbers frozen and `pgrep -f busy-loop` found no process.

User confirmed: "Подтверждаю".

## Suite at land time

39 cargo tests, 35 vitest tests, preflight exit 0.
