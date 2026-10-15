# e02s04 — Leave nothing running when the app closes

## 1. Identity

Story ID: e02s04. Title: Leave nothing running when the app closes. Epic:
e02-configure-and-control. Status: planned.

## 2. Summary

The host records every live run's process group in a small state file in application data and
clears it on a clean exit. Closing the app stops every live run through the same `stop_run`
path a user press uses, so each run ends as `stopped`. If the host crashed before, the next
start reads the state file and reaps every recorded group that is still alive — verified by
pid start time, so a recycled pid is never killed — then clears the file.

## 3. Classification

type: feat · risk: P1 · context: infra · bcps: 4

## 4. Value

The product launches scripts on the user's machine. An app that can leak orphan processes on
close or crash is worse than a terminal. "Leave nothing running" is the trust promise of a
launcher.

## 5. Users and triggers

Closing the app window while runs are live; restarting the app after a crash or a kill -9 of
the host.

## 6. Domain language

**Live group record** — `{ pgid, pid, startTime }` written while a run is live; `pgid` equals
the leader pid because every run is a group leader (DEC-07). **Reap** — terminating a recorded
group that is still alive, through the same `terminate_group` path as a user stop. **Start
time match** — the recorded process start time (procfs `Task::starttime`) compared before
killing, so a pid recycled by an unrelated process is left alone.

## 7. Requirements

#### MODIFIED: Shutdown stops every live run through the user stop path
**Before:** `Supervisor::shutdown` iterated the runs map calling `terminate_group` directly;
stopped runs did not emit `Ended`, so the UI could not show them as stopped.
**After:** `shutdown` calls `stop_run` for every live run — same SIGTERM → `TERM_GRACE` →
SIGKILL path as a user press — and each run emits its final `Ended` with status `stopped`.

#### ADDED: The host records live process groups while runs are live
On spawn the supervisor appends a live group record (`pgid`, `pid`, `startTime` from procfs)
to `live-runs.json` in application data; when a run ends (any path) its record is removed. A
clean shutdown clears the file.

#### ADDED: On host start, stale recorded groups are reaped
Before any run can start, the host reads `live-runs.json`. For each record whose pid still
exists and whose start time matches the record, the host reaps that group through
`terminate_group`. Records that do not match (pid recycled, process gone) are dropped without
killing. The file is cleared after the sweep.

## 8. Behaviour before

`shutdown` kills groups directly on exit and on Drop; nothing is recorded, so a crashed host
leaks orphan process trees forever and nothing can find them again.

## 9. Behaviour after

Closing the app ends every live run as `stopped` with a visible final status. After a crash,
the next app start removes the orphans the crashed host left, and never touches a process
whose pid happens to have been reused.

## 10. Interface and data contracts

New Rust module `src-tauri/src/recovery.rs`:

```rust
pub struct LiveGroupRecord { pub pgid: u32, pub pid: u32, pub start_time: u64 }
pub fn load_records(path: &Path) -> Vec<LiveGroupRecord>;   // missing or corrupt file -> empty
pub struct LiveRunFile { /* path + records, insert/remove/clear, atomic write */ }
pub fn reap_stale(path: &Path, terminate: &dyn Fn(u32, u32)) -> Vec<LiveGroupRecord>;
// terminate(pgid, pid) is supervisor::terminate_group; returns the records it acted on
```

Reason for depth: after a crash the new host process has no in-memory way to find the groups
the old one left; one small file is the minimum record that makes reaping possible, and the
start-time match is the minimum guard against pid reuse. `Supervisor` gains
`with_state_file(path)`; `watch_exit` and `stop_run` remove records; `shutdown` clears the
file. `lib.rs` calls `reap_stale` in `setup` before managing the supervisor. No new IPC
command; no UI change required.

Zoom-out — `supervisor.rs`: purpose = own live runs and guarantee termination; callers =
`commands.rs`, `lib.rs`; contracts = group-leader spawn, one termination path
(`terminate_group`), `Ended` after reader drain. `lib.rs` setup order becomes: data dir →
reap stale groups → open registry → manage supervisor.

## 11. Affected modules

`src-tauri/src/recovery.rs` — new. `src-tauri/src/supervisor.rs` — state file wiring,
`shutdown` via `stop_run`. `src-tauri/src/lib.rs` — reap at setup. No feature-slice UI work
beyond e02s03's stopped label.

## 12. Implementation steps

1. `shutdown` routes every live run through `stop_run`; each emits `Ended` status `stopped` →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml shutdown_stops_all`
2. `recovery.rs`: record live groups on spawn, remove on end, clear on shutdown; reap stale
   records at startup with the start-time match →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml crash_recovery_reaps`
3. Manual: close the app with live runs and confirm no orphans (see §14) →
   verify-script: manual, recorded in §14

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e02s04-P1-01 | U-R | Supervisor shutdown over two live runs emits two Ended(status=stopped) and both process trees are gone |
| SC-e02s04-P1-02 | U-R | A live group spawned by the test, recorded, then "reaped" by a fresh host run of `reap_stale`, is dead; a record whose start time does not match is dropped without killing |
| SC-e02s04-P1-03 | U-R | Clean shutdown leaves `live-runs.json` empty; a run ending naturally removes only its own record |
| SC-e02s04-P1-M1 | M | The human closes the app with two infinite scripts running and `pgrep` finds neither |

## 14. Verification Script

1. Start two registered scripts that run indefinitely.
2. Close the app window.
3. Run `pgrep -f` for both script names. Confirm neither is running.
4. Start one infinite script, then kill the app with `kill -9`.
5. Start the app again. Confirm `pgrep` no longer finds the orphan and the app starts
   normally.

## 15. Out of scope

Reaping processes the user started outside the app; notifying the UI about reaped orphans
beyond a log line; a lock file preventing two app instances (a second instance would reap the
first's runs — deferred, single-instance enforcement is a follow-up).

## 16. Risks

- Pid reuse: a stale record whose pid now belongs to an unrelated process. Guarded by the
  start-time match before any kill; the mismatch path is unit-tested.
- Two app instances at once would treat each other's live runs as stale. Accepted for this
  version (single-instance desktop app); recorded as a follow-up.
- `startTime` comes from procfs (DEC-11 discipline) — no hand-parsing of `/proc/<pid>/stat`.

## 17. Acceptance criteria

```gherkin
# SC-e02s04-P1-01
Scenario: Closing the app stops every live run as a user stop
  Given two live runs
  When the supervisor shuts down
  Then each run emits one ended event with status stopped and its group is gone

# SC-e02s04-P1-02
Scenario: Stale groups reaped, recycled pids spared
  Given a state file with one live group and one record whose start time no longer matches
  When a fresh host reaps stale records
  Then the live group is dead and the mismatched record is dropped without any kill

# SC-e02s04-P1-03
Scenario: Clean exit leaves no record
  Given runs started and ended cleanly
  When the host shuts down
  Then the live-runs file is empty
```

## 18. Dependencies and prior art

No new packages: `procfs` (already [OK] per DEC-11) supplies `Task::starttime`; the state
file reuses the atomic temp+rename discipline from `Registry::save`. Prior art: systemd's
`Restart=` start-time identity check; `supervisor`-style pidfiles, corrected for pid reuse.

## 19. Error surfaces

A corrupt `live-runs.json` loads as empty (the sweep is skipped, logged) — a broken record
file must never block app start. `reap_stale` failures (EPERM on a foreign group) are logged
and the record dropped.

## 20. Follow-ups

Single-instance enforcement; surfacing "reaped N orphans" in the UI.
