# e02s02 — Stop a running script

## 1. Identity

Story ID: e02s02. Title: Stop a running script. Epic: e02-configure-and-control. Status:
planned.

## 2. Summary

The user presses Stop on a live run. The host sends SIGTERM to the run's whole process
group, waits the bounded grace (`TERM_GRACE`, 2 s), and SIGKILLs whatever is still alive.
The stop path is the same `terminate_group` the host already uses at shutdown, so one call
reaches every descendant of a script that forks (DEC-07). The run view gains a Stop control
that is enabled only while the run is live.

## 3. Classification

type: feat · risk: P1 · context: domain · bcps: 4

## 4. Value

A launcher that cannot stop what it started is a half product. Scripts that fork, loop, or
watch must be stoppable with one press.

## 5. Users and triggers

A live run in the run view. The user presses Stop; the run and everything it spawned are
gone within the grace bound.

## 6. Domain language

**Stop** — a user-requested termination of one live run through the host's group-termination
path. **Grace** — the bounded wait (`TERM_GRACE` = 2 s) between SIGTERM and SIGKILL.
**Process group** — every process in the run's group; the run is the group leader because
`start` calls `setsid` (DEC-07).

## 7. Requirements

#### ADDED: The host stops one live run on request
`Supervisor::stop_run(run_id)` sends SIGTERM to the run's process group, waits
`TERM_GRACE`, and SIGKILLs the group if the leader is still alive. It uses the same
`terminate_group` path as shutdown — no second termination implementation. An unknown or
already-finished run id is refused with `StopError::runNotFound`.

#### MODIFIED: The Ended event carries the stop request
**Before:** `RunEvent::Ended { run_id, code }`; a stopped run looked identical to one that
exited.
**After:** `RunEvent::Ended { run_id, code, stopped: bool }`; `stopped` is true when the stop
was user-requested. e02s03 turns this into the full end status.

#### ADDED: The run view offers Stop, enabled only for a live run
A Stop control per live run. It is disabled once the run has ended, and pressing it twice
does not create a second termination.

## 8. Behaviour before

Runs can only end by themselves or when the whole app exits. There is no stop control.

## 9. Behaviour after

Pressing Stop ends the run and its whole process tree within `TERM_GRACE + KILL_GRACE`,
and the UI shows the run as ended with `stopped` recorded.

## 10. Interface and data contracts

Rust (`src-tauri/src/supervisor.rs`):

```rust
pub enum StopError { RunNotFound(String) }   // kind() -> "runNotFound"
impl Supervisor { pub fn stop_run(&mut self, run_id: &str) -> Result<(), StopError> }
```

`LiveRun` gains `stopped: Arc<AtomicBool>` shared with `watch_exit`. New command
(`commands.rs`): `stop_run(supervisor, run_id: String) -> Result<(), StopErrorPayload>`.
`run://ended` payload gains `stopped: boolean`; `EndedEvent` in
`src/features/runs/host.ts` mirrors it. `RunHost` gains `stopRun(runId): Promise<void>`.

Zoom-out — `supervisor.rs`: purpose = own every live run and guarantee the group dies;
callers = `commands.rs::start_script`, `lib.rs` exit handler, now also `stop_run`; contracts
= group-leader spawn discipline, `TERM_GRACE` bound, `Ended` emitted only after both readers
hit EOF and the child is reaped.

## 11. Affected modules

`src-tauri/src/supervisor.rs` — `stop_run`, `LiveRun.stopped`, `Ended.stopped`.
`src-tauri/src/commands.rs` — `stop_run` command. `src-tauri/src/lib.rs` — registration.
`src/features/runs/` — `RunControls.tsx` (Stop button), `useRuns.ts`, `host.ts`, `types.ts`.

## 12. Implementation steps

1. `Supervisor::stop_run` reusing `terminate_group`; `runNotFound` for unknown ids →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml stop_sends_signal`
2. Escalation: a child that ignores SIGTERM is SIGKILLed after `TERM_GRACE` →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml stop_escalates`
3. `stop_run` command + `RunControls` Stop button, disabled when the run is not live →
   verify: `npm run test -- RunControls`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e02s02-P1-01 | U-R | Stop on a run whose script forked a grandchild ends both; `/proc` entries disappear |
| SC-e02s02-P1-02 | U-R | A script trapping SIGTERM is gone within `TERM_GRACE + KILL_GRACE` |
| SC-e02s02-P1-03 | U-R | Unknown run id answers `runNotFound`; stopping an ended run answers `runNotFound` |
| SC-e02s02-P2-04 | U-T | Stop button is enabled only while state is running; one press disables further presses |
| SC-e02s02-P1-M1 | M | A human stops a looping script and confirms with `pgrep` that nothing survives |

## 14. Verification Script

1. Register and start a script that loops forever and spawns a `sleep` child.
2. In another terminal confirm both pids exist (`pgrep`).
3. Press Stop in the app.
4. Within ~3 seconds confirm `pgrep` finds neither pid.
5. Confirm the Stop control is now disabled.

## 15. Out of scope

Per-run SIGINT/interactive Ctrl+C (out of scope in SCOPE_LATEST — no pty), stop with a custom
signal, stop timeout configuration, end-status rendering (e02s03).

## 16. Risks

- A script that traps SIGTERM and hangs longer than the grace is SIGKILLed — intended, but
  it may leave its own temp files. Accepted: the product promise is "nothing keeps running".
- `stop_run` on a run whose readers are still draining: `Ended` still waits for reader EOF,
  so output is not truncated by the stop.

## 17. Acceptance criteria

```gherkin
# SC-e02s02-P1-01
Scenario: The whole group stops
  Given a live run whose script spawned a grandchild
  When the user stops the run
  Then both pids are gone within the grace bound

# SC-e02s02-P1-02
Scenario: SIGTERM-ignoring script is forced down
  Given a live run that traps and ignores SIGTERM
  When the user stops the run
  Then the process is gone within TERM_GRACE + KILL_GRACE

# SC-e02s02-P2-04
Scenario: Stop only while live
  Given a run that has already ended
  When the run view renders
  Then the Stop control is disabled
```

## 18. Dependencies and prior art

No new packages. `terminate_group` (SIGTERM → 2 s → SIGKILL via `killpg`) already exists in
`supervisor.rs` from e01 and is the shutdown path; this story exposes it per-run. Same
semantics as systemd's `Terminate=` grace sequence.

## 19. Error surfaces

`runNotFound` for an unknown or finished run id — the UI keeps the button disabled in the
normal path, so this is a race guard (run ended between render and press), surfaced as a
toast, not a crash.

## 20. Follow-ups

e02s03 renders the stopped state distinctly; e02s04 routes shutdown through the same
`stop_run` path.
