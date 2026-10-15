# e02s03 — Show how the run ended

## 1. Identity

Story ID: e02s03. Title: Show how the run ended. Epic: e02-configure-and-control. Status:
planned.

## 2. Summary

When a run ends, the host derives one end status — `exited` (exit code 0), `failed` (non-zero
code or death by an unrequested signal), `stopped` (the user pressed Stop) — and emits it once
on `run://ended`, after both output readers have drained (the existing completion discipline).
The metrics poller for that run is already stopped by the same `finished` flag; this story
proves it. The run view renders five distinct states: not running, running, exited, failed,
stopped.

## 3. Classification

type: feat · risk: P1 · context: domain · bcps: 4

## 4. Value

The scope promises "the app shows the current state: not running, running, exited, or
failed". Without a distinct end status, a crashed script and a stopped script look identical,
and the user cannot trust the registry.

## 5. Users and triggers

Any run that ends — naturally, by failing, or by Stop. The status appears in the run view and
stays until the entry is started again.

## 6. Domain language

**End status** — the host-derived verdict for a finished run: `exited`, `failed`, or
`stopped`. **Final state event** — the single `Ended` carrying the end status, emitted after
reader drain. **Not running** — an entry with no run record in this session; distinct from
every ended status.

## 7. Requirements

#### ADDED: The host derives one end status per finished run
`stopped` when the termination was user-requested (the e02s02 flag); otherwise `exited` when
the exit code is 0; otherwise `failed` — a non-zero code or a death by signal (code `None`).

#### MODIFIED: The Ended event carries the status instead of the raw stop flag
**Before:** `RunEvent::Ended { run_id, code, stopped }` (e02s02 shape).
**After:** `RunEvent::Ended { run_id, code, status: RunStatus }` with `RunStatus ∈ {Exited,
Failed, Stopped}`; `run://ended` carries `status: "exited" | "failed" | "stopped"`. The event
is emitted exactly once per run, after both readers reach EOF and the child is reaped.

#### ADDED: No metrics arrive after the final state event
The metrics poller for a run stops on the same `finished` flag that gates `Ended`; no
`Metrics` event for a run id may follow its `Ended`. (Behavior exists from e03; this story
makes it a tested contract.)

#### MODIFIED: The run view renders five distinct states
**Before:** `RunState = "running" | "ended"` with the exit code shown.
**After:** the view renders `not running`, `running`, `exited`, `failed`, `stopped` as
distinct labels; `failed` shows the exit code when there is one.

## 8. Behaviour before

Every ended run renders as "ended" with a bare exit code; a stopped run is indistinguishable
from a crashed one.

## 9. Behaviour after

Each finished run shows why it ended. A script that exits 0 reads "exited"; one that exits 3
reads "failed (3)"; one the user stopped reads "stopped".

## 10. Interface and data contracts

Rust (`src-tauri/src/supervisor.rs`):

```rust
pub enum RunStatus { Exited, Failed, Stopped }   // tag(): "exited" | "failed" | "stopped"
RunEvent::Ended { run_id: String, code: Option<i32>, status: RunStatus }
```

`watch_exit` derives the status from the stopped flag and the wait result. `runs.rs`
`EndedPayload` gains `status: &'static str`. TypeScript (`src/features/runs/`):
`EndedEvent` gains `status: RunEndStatus`; `RunState = "notRunning" | "running" | "exited" |
"failed" | "stopped"`; `Run` keeps `exitCode: number | null`.

Zoom-out — `supervisor.rs`: purpose = own live runs and their termination; callers =
`commands.rs`, `lib.rs`; contracts = `Ended` exactly once and only after reader drain,
`StartError`/event tag stability with the TS unions, poller lifetime bound to `finished`.

## 11. Affected modules

`src-tauri/src/supervisor.rs` — `RunStatus`, `watch_exit` derivation. `src-tauri/src/runs.rs`
— payload field. `src/features/runs/` — `RunStatusBadge.tsx` (or equivalent in the run view),
`useRuns.ts`, `types.ts`, `host.ts`.

## 12. Implementation steps

1. `RunStatus` derivation in `watch_exit`; `Ended` payload carries `status` →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml run_exit_state`
2. Exactly-once `Ended` and no `Metrics` event after it for the same run id →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml final_state_event`
3. Five distinct state labels in the run view →
   verify: `npm run test -- RunStatus`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e02s03-P1-01 | U-R | exit 0 → exited; exit 7 → failed with code 7; user stop → stopped; signal death without stop → failed with code null |
| SC-e02s03-P1-02 | U-R | Exactly one Ended per run; the recorder shows no Metrics event after Ended for that run id |
| SC-e02s03-P2-03 | U-T | Each of the five states renders its own label; failed shows the code when present |
| SC-e02s03-P2-M1 | M | A human runs a script that exits 0, one that exits non-zero, and stops a loop; the three labels differ |

## 14. Verification Script

1. Register and start a script that prints "ok" and exits 0. Confirm the view reads "exited".
2. Start a script that exits 3. Confirm "failed" with the code 3.
3. Start a looping script and press Stop. Confirm "stopped".
4. Confirm no cost readout updates arrive after the status label appears.

## 15. Out of scope

End-time timestamps, run history across sessions, distinguishing signal kinds, restart
affordances.

## 16. Risks

- A run killed by an outside signal (OOM killer) reports `failed` with code `null` — honest,
  but the user may ask "by whom"; the detail is out of scope and stated in the UI copy as
  "killed".
- Status derivation must read the stopped flag before the flag is cleared; the flag lives in
  `LiveRun` and is only written by the stop path, so the ordering is safe.

## 17. Acceptance criteria

```gherkin
# SC-e02s03-P1-01
Scenario: End status derived from how the process ended
  Given a run whose script exits with code 7
  When the run ends
  Then the ended status is failed and the code is 7

# SC-e02s03-P1-02
Scenario: One final event, no metrics after it
  Given a run that produced metrics while live
  When it ends
  Then exactly one ended event arrives and no later metrics event names that run

# SC-e02s03-P2-03
Scenario: Five states render distinctly
  Given runs in each of the five states
  When the run view renders
  Then each run shows its own state label
```

## 18. Dependencies and prior art

No new packages. The completion discipline (readers drain before `Ended`) is from e01s03; the
poller's `finished` flag from e03s01. Status semantics match shell exit-code conventions
(0 = success) and systemd's unit states (exited/failed).

## 19. Error surfaces

None new: a finished run is data, not an error. `runNotFound` (e02s02) stays the only stop
error.

## 20. Follow-ups

Run history across sessions is deferred with metric history (SCOPE_LATEST out_of_scope).
