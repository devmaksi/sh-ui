# Refactor Plan — split supervisor.rs

Follow-up recorded in specs/verifications/AUDIT-e02-e02s04.md (§Code Style) and
specs/state.yaml known_findings. Convention violated: CONVENTIONS.md "Files: under
300 lines. One responsibility per module" and "Rust functions stay under 40 lines".

## Problem Statement

`src-tauri/src/supervisor.rs` is 619 lines and mixes five responsibilities: event
types, error types, process spawn/terminate, output reading, and the end-of-run
watch sequence. It is the project's churn hotspot (12 commits / 90 days), so every
future fix grows the largest file in the crate. `Supervisor::start` is ~90 lines and
does validation, spawning, record-keeping, output wiring, poller wiring, and exit
watching in one body.

## Solution

Split the file into six modules, one responsibility each (variant A):

| Module | Contents |
|---|---|
| `events.rs` | `OutputStream`, `RunEvent`, `RunStatus`, `EventSink`, `Recorder` |
| `errors.rs` | `StartError`, `StopError` |
| `output.rs` | `read_stream`, `split_chunk`, `OUTPUT_CHUNK_MAX_BYTES` |
| `spawn.rs` | `working_directory`, `spawn_capturing` + ETXTBSY retry constants, `terminate_group`, `wait_for_exit`, `TERM_GRACE`, `KILL_GRACE` |
| `watch_exit.rs` | `Completion`, `mark_reader_done`, `mark_exit`, `ExitWatch`, `watch_exit` |
| `supervisor.rs` (remainder) | `Supervisor`, `LiveRun`, `RunHandle`, `start`/`stop_run`/`shutdown`, `Drop` |

Then split `start()` into private helpers, each under 40 lines: validation,
spawn+record, wiring (output readers, poller, exit watch).

Test imports and internal `use` lines are repointed to the new modules in the same
commit as each extraction. No re-export shims are left behind.

**Preserved invariant (HARD GATE):** purely structural refactor. No behavior change.
String tags of `StartError::kind()` and `RunStatus::tag()` — part of the TypeScript
union contract — stay byte-identical. The event-ordering guarantee ("Ended is
structurally the last event of a run", e02s04) is carried over unchanged. `// story:`
comments travel with the code they annotate.

## Commits

Pre-step (separate chore branch, not part of the refactor diff):

0. `chore(tools): make dashboard.sh executable` — commit the pending mode change
   (644→755) in `tools/dashboard.sh` and land it. → verify: `bash -n tools/dashboard.sh`

All refactor commits happen on one branch (kickoff-branch), in dependency order so
each commit compiles:

1. `refactor(host): extract event types into events.rs` — move `OutputStream`,
   `RunEvent`, `RunStatus`, `EventSink`, `Recorder` and their doc comments into a new
   `events.rs`; declare it in `lib.rs`; repoint every consumer (`metrics.rs`,
   `runs.rs`, all test files importing these names from `supervisor::`). Delete the
   moved code from `supervisor.rs`. → verify: `cargo test --manifest-path src-tauri/Cargo.toml`

2. `refactor(host): extract start/stop errors into errors.rs` — move `StartError`
   and `StopError` (incl. `kind()`, `Display`) into `errors.rs`; repoint `commands.rs`
   and test imports. → verify: `cargo test --manifest-path src-tauri/Cargo.toml`

3. `refactor(host): extract output reading into output.rs` — move `read_stream`,
   `split_chunk`, `OUTPUT_CHUNK_MAX_BYTES` into `output.rs`; repoint `supervisor.rs`
   internals and the `output_chunk_cap` test import. → verify:
   `cargo test --manifest-path src-tauri/Cargo.toml`

4. `refactor(host): extract spawn and group termination into spawn.rs` — move
   `working_directory`, `spawn_capturing`, ETXTBSY retry constants, `terminate_group`,
   `wait_for_exit`, `TERM_GRACE`, `KILL_GRACE` into `spawn.rs`; repoint `lib.rs`
   (`terminate_group` call), `supervisor.rs`, and the `stop_escalates` test import of
   `TERM_GRACE`. → verify: `cargo test --manifest-path src-tauri/Cargo.toml`

5. `refactor(host): extract the exit-watch sequence into watch_exit.rs` — move
   `Completion`, `mark_reader_done`, `mark_exit`, `ExitWatch`, `watch_exit` into
   `watch_exit.rs`; repoint `supervisor.rs`. The join-poller-before-Ended ordering
   stays in one piece. → verify: `cargo test --manifest-path src-tauri/Cargo.toml`
   (includes `ended_event_once`, `final_state_event`, `metrics_emits_and_stops`)

6. `refactor(host): split Supervisor::start into three helpers` — private methods:
   validate the registered file (metadata, executable bit, working dir), spawn and
   record the live group, wire output readers + metrics poller + exit watch. Each
   helper under 40 lines; `start` becomes a short composition. → verify:
   `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path
   src-tauri/Cargo.toml -- -D warnings`

7. `refactor(host): verify module sizes and close the follow-up` — confirm every
   file under 300 lines (`wc -l src-tauri/src/*.rs`); mark the supervisor.rs item
   resolved in specs/state.yaml known_findings and append a resolution note to
   AUDIT-e02-e02s04.md. → verify: `npm run preflight`

Each commit leaves the crate compiling and the full suite green.

## Decision Document

- Six host modules replace the single supervisor module; module names: events,
  errors, output, spawn, watch_exit, supervisor.
- Extraction order follows the dependency graph: events → errors → output → spawn →
  watch_exit → supervisor remainder. No circular `use`.
- No `pub use` re-export shims: consumers (crate internals and integration tests)
  are repointed to the owning module in the same commit as the extraction.
- `terminate_group` stays `pub` (used by host startup reaping) but lives in `spawn.rs`.
- `Completion` and its helpers move with `watch_exit` because they exist only for the
  drain-before-Ended protocol.
- `start()` is decomposed into private helpers only; the public interface of
  `Supervisor` (`new`, `with_state_file`, `attach_gpu`, `live_run_ids`, `start`,
  `stop_run`, `shutdown`) does not change.
- Error tags (`entryNotFound`, `notExecutable`, `spawnFailed`, `badWorkingDir`,
  `runNotFound`) and status tags (`exited`, `failed`, `stopped`) are frozen — they
  are the IPC contract with the TypeScript union.
- Story-tag comments (`// story: eNNsNN`) move with the code blocks they annotate;
  traceability tooling keeps seeing them.

## Testing Decisions

- No new tests. The existing integration suite (stop_run, stop_escalates,
  ended_event_once, final_state_event, crash_recovery_reaps, shutdown_stops_all,
  output_chunk_cap, supervisor_busy_file, spawn_no_shell_interpretation, and the
  rest) is the safety net: it exercises public behavior through `Supervisor` +
  `Recorder` and is blind to where types live.
- Good test here = behavior-level test that compiles unchanged apart from import
  paths. If any test needs a logic edit to pass, the refactor has leaked behavior —
  stop and re-examine.
- Prior art: tests already import from `sh_ui_lib::supervisor::…`; after the refactor
  they import from `sh_ui_lib::events::…` etc. Same shape, new path.
- Gate: `npm run preflight` at the end (ESLint + tsc + clippy -D warnings + full
  Rust + Vitest suites + build).

## Out of Scope

- Any behavior change, including the ETXTBSY retry policy, grace durations, and
  chunk cap value.
- The known ETXTBSY flake source hunt (separate investigation if it resurfaces).
- Single-instance enforcement and reaped-orphans UI (separate change-request
  candidates).
- TS/UI code, IPC command signatures, registry, metrics, and gpu modules.
- Splitting `registry.rs` (344 lines, slightly over) — candidate for a later pass,
  not this one.

## Further Notes

- After extraction, `supervisor.rs` should land near ~180 lines; the largest new
  module near ~130. All under the 300-line convention.
- Run the refactor on a `kickoff-branch` branch (worktrees are disabled per DEV-02;
  check out in the primary checkout), land with `bash scripts/land-branch.sh`.
