# e03s01 — Live CPU and RAM of the running script

## 1. Identity

Story ID: e03s01. Title: Live CPU and RAM of the running script. Epic: e03-process-cost. Status: planned.

## 2. Summary

While a run is live, the host reads the script process from `/proc` through the `procfs`
crate on a fixed interval and emits a metrics event: CPU percent and resident RAM. The UI
renders a live readout next to the output panel. When the run ends, the events stop.

## 3. Classification

type: feat · risk: P1 · context: domain · bcps: 5

## 4. Value

The product's promise is "see what your scripts cost". This story delivers the first two
numbers of that promise for a single process.

## 5. Users and triggers

A run is live. The readout updates once per refresh interval without any user action.

## 6. Domain language

**Metrics sample** — one reading of one pid: CPU ticks consumed and resident bytes.
**CPU percent** — percent of one core: Δ(utime+stime) over Δwall, in the sense of `top`.
A multi-threaded script may exceed 100%. (Decision 2026-08-14, user chose per-core.)
**Refresh interval** — the named constant gap between two metrics events.

## 7. Requirements

#### ADDED: The host measures a live process
For every live run the host reads CPU ticks and resident memory of the run pid from `/proc`
via the `procfs` crate. The reading lives behind a `ProcReader` trait so tests can feed
deterministic samples.

#### ADDED: Metrics arrive on a fixed interval
While a run is alive, one metrics event per refresh interval reaches the UI. After the run
ends, no further metrics events are emitted for it.

#### ADDED: CPU percent is percent of one core
CPU percent = Δ(utime+stime) ticks ÷ (clock ticks per second × Δwall seconds) × 100.
Values above 100 mean the script used more than one core.

#### ADDED: The user reads live CPU and RAM
The runs view shows the latest CPU percent and resident RAM for the selected live run.

## 8. Behaviour before

e01 ships live output. A running script shows its text but no cost.

## 9. Behaviour after

A live run shows a CPU number and a RAM number that update on the refresh interval and
freeze at the last value when the run ends.

## 10. Interface and data contracts

New host module `metrics.rs`:
- `struct ProcessSample { cpu_ticks: u64, rss_bytes: u64 }`
- `trait ProcReader { fn sample(&self, pid: u32) -> Option<ProcessSample> }` — `None` when
  the pid is gone.
- `const METRICS_REFRESH_MS` — named constant, 1000 ms.
- Clock ticks per second read once via `sysconf(_SC_CLK_TCK)`.

New event variant `RunEvent::Metrics { run_id, cpu_percent: f64, ram_bytes: u64 }` mapped to
app event `run://metrics { runId, cpuPercent, ramBytes }`.

## 11. Affected modules

`src-tauri/src/metrics.rs` — new. `src-tauri/src/supervisor.rs` — starts and stops the
poller per run. `src-tauri/src/runs.rs` — metrics payload. `src/features/runs/` — cost
readout. New dependency `procfs 0.18.0` in `src-tauri/Cargo.toml` (user-approved, DEC-11).

## 12. Implementation steps

1. Add `procfs` and the metrics module: `ProcessSample`, `ProcReader`, a procfs-backed
   reader; read a real spawned pid → verify: `cargo test --manifest-path src-tauri/Cargo.toml proc_reader`
2. CPU percent math from tick deltas and wall deltas with the named interval constant → verify: `cargo test --manifest-path src-tauri/Cargo.toml cpu_percent_math`
3. Poller emits `RunEvent::Metrics` every interval while the run is alive and stops after
   it ends → verify: `cargo test --manifest-path src-tauri/Cargo.toml metrics_emits_and_stops`
4. Map the variant to `run://metrics` with a camelCase payload → verify: `cargo test --manifest-path src-tauri/Cargo.toml metrics_event_payload`
5. Render the live readout for the selected run → verify: `npm run test -- cost-readout`

## 13. Test strategy

No e03 test-plan artifact yet; scenarios are defined here and reuse e01 levels.

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e03s01-P1-01 | U-R | A real spawned pid yields rss > 0 and growing cpu ticks |
| SC-e03s01-P1-02 | U-R | Metrics events arrive on the interval while the run is alive |
| SC-e03s01-P1-03 | U-R | No metrics event arrives after `Ended` |
| SC-e03s01-P2-04 | U-R | A known tick delta over a known wall delta yields the exact percent |
| SC-e03s01-P2-05 | U-T | The readout renders CPU percent and RAM from a metrics event |
| SC-e03s01-P1-M1 | M | A human runs a busy-loop script and sees a non-zero CPU number |

## 14. Verification Script

1. Register `/tmp/busy-loop.sh` that burns CPU in one loop.
2. Start it.
3. Observe the CPU number rise above 0 and stay near 100 for one core.
4. Observe a plausible RAM figure (thousands of KiB or more, not 0).
5. Stop the script. Observe the numbers stop updating.

## 15. Out of scope

Child processes (e03s02), GPU and VRAM (e04), metric history and graphs (out of product
scope), per-user attribution, nice/priority.

## 16. Risks

- Reading `/proc` by field index is fragile across kernels. Mitigated by using `procfs`
  instead of hand parsing (DEC-11).
- A poller that outlives the run leaks a thread and emits stale metrics. Detect with the
  stop test, not by inspection.
- A dead pid between two polls must end the poller, not panic. `ProcReader` returns `None`.

## 17. Acceptance criteria

```gherkin
# SC-e03s01-P1-02
Scenario: Cost updates while the script runs
  Given a live run
  When the refresh interval passes
  Then a metrics event with cpu percent and ram arrives

# SC-e03s01-P1-03
Scenario: Cost stops when the script ends
  Given a run that just ended
  When any further interval passes
  Then no metrics event arrives for that run

# SC-e03s01-P2-04
Scenario: One core busy reads as about 100 percent
  Given a sample delta of 100 ticks over 1 wall second on a 100 Hz clock
  When the host computes the percent
  Then the result is 100

# SC-e03s01-P2-05
Scenario: The user sees the cost
  Given a metrics event for the selected run
  When the runs view renders
  Then it shows the CPU percent and the resident RAM
```

## 18. Dependencies and prior art

`procfs 0.18.0` (tokio-rs) `[OK]` — 86M downloads, updated 2025-08. Verified API:
`Process::stat()` → `Stat { utime, stime }` in clock ticks; `Process::status()` →
`Status.vm_rss` in KiB. Prior art: `top`/`htop` per-core percent semantics. No other new
package.

## 19. Error surfaces

A pid that vanishes between polls ends the poller silently; the readout keeps the last
value. A `/proc` read error on a live pid logs nothing to the UI — the next interval
retries.

## 20. Follow-ups

Tree sum over descendants arrives in e03s02. GPU and VRAM arrive in e04.
