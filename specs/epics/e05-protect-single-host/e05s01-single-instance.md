# e05s01 — A second instance must not fight the first

## 1. Identity

Story ID: e05s01. Title: A second instance must not fight the first. Epic:
e05-protect-single-host. Status: planned.

## 2. Summary

The host takes an exclusive OS-level lock (flock) on a file in application data before
anything else in setup runs. A second instance that cannot take the lock exits immediately
without reading `live-runs.json`, without reaping, and without constructing a Supervisor.
The lock is held by the open file descriptor for the whole life of the process and is
released by the OS on exit, including a crash — so a crashed host never leaves a stale
lock, exactly like it never leaves a stale live-group record.

## 3. Classification

type: fix · risk: P1 · context: host · bcps: 3

## 4. Value

e02s04 shipped crash recovery: at startup the host reaps every process group recorded in
`live-runs.json`. While a first instance is running, those records are live, not stale —
so a second launch today kills the first instance's running scripts and races it on the
same file. The recovery feature is a weapon against a parallel instance. One lock removes
the whole class.

## 5. Users and triggers

A user double-clicks the app icon while the app is already running. The second process
detects the held lock and exits. The first instance is untouched: its runs keep running,
its state file stays single-writer.

## 6. Domain language

**Instance lock** — an exclusive advisory lock (`flock LOCK_EX | LOCK_NB`) on
`<app_data>/app.lock`, held for the life of the host process. **Second instance** — a host
process that starts while the lock is already held. **Single host** — the invariant that at
most one sh-ui process supervises runs and reads or writes `live-runs.json` at a time.

## 7. Requirements

#### ADDED: The host takes the instance lock before any recovery work
Setup order in `lib.rs` becomes: acquire instance lock → reap stale groups → open registry
→ construct Supervisor. A host that does not hold the lock performs none of the later
steps.

#### ADDED: A second instance exits without side effects
When the lock is held, the host writes one clear line to stderr ("another instance is
already running") and terminates with a successful exit code before window creation. It
must not open `live-runs.json`, must not call `reap_stale`, must not spawn anything.

#### MODIFIED: Crash recovery keeps its promise, now against one host only
**Before:** startup reaping could target groups owned by a running sibling instance.
**After:** reaping runs only in the process that holds the lock, which is the only process
that can own live groups. The reap semantics themselves (DEC-07 path, `terminate_group`)
do not change.

## 8. Behaviour before

Any number of instances start. The later one reaps the earlier one's live groups.

## 9. Behaviour after

Exactly one instance runs. A second launch exits within milliseconds, visibly absent, with
one stderr line. After the first instance exits or crashes, the next launch acquires the
lock normally and reaps as before.

## 10. Interface and data contracts

Rust, new module `src-tauri/src/instance.rs`:

```rust
pub enum InstanceError { AlreadyRunning(String), LockFailed(String) }
/// Returns the locked file; the lock lives as long as the File is open.
/// The caller must keep it alive for the whole process (managed state).
pub fn acquire_instance_lock(path: &Path) -> Result<std::fs::File, InstanceError>;
```

Implementation: `open` the lock file (create if missing), `libc::flock(fd, LOCK_EX |
LOCK_NB)`; `EWOULDBLOCK` maps to `AlreadyRunning`. `lib.rs` setup calls it first and keeps
the returned `File` in a managed state for the process lifetime. On `AlreadyRunning`:
`eprintln!` one line, `app.exit(0)`-equivalent early return from setup that prevents
window creation (setup returns Err → builder aborts before windows show; verify the exact
Tauri 2 exit path during implementation). No new dependencies: `libc` is already a project
dependency.

## 11. Affected modules

`src-tauri/src/instance.rs` (new). `src-tauri/src/lib.rs` — setup order, lock kept alive.
`src-tauri/Cargo.toml` — no new deps (`libc` exists).

## 12. Implementation steps

1. `instance.rs` with `acquire_instance_lock` + unit tests → verify:
   `cargo test --manifest-path src-tauri/Cargo.toml --test instance_lock`
2. Wire into `lib.rs` setup before `reap_stale`; keep the File alive; second instance
   exits with one stderr line → verify: `cargo test --manifest-path src-tauri/Cargo.toml`
   (full suite green) + manual two-instance check (step 14).
3. Guard test that reap cannot run without the lock: setup order asserted by a test that
   a second `acquire_instance_lock` on the same path fails while the first File is alive →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml --test instance_lock`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e05s01-P1-01 | U-R | First acquire succeeds; second acquire on the same path fails with AlreadyRunning while the first File is alive |
| SC-e05s01-P1-02 | U-R | After the first File is dropped (simulated exit), a new acquire succeeds |
| SC-e05s01-P1-03 | U-R | A lock file left by a killed process does not block the next acquire (OS releases flock on process death) |
| SC-e05s01-P1-M1 | M | Two real app instances: the second exits within ~1 s, the first's running script survives (`pgrep`), one stderr line printed |

## 14. Verification Script

1. Start the app, launch a long-running script.
2. Start the app a second time.
3. Confirm the second process exits quickly and prints "another instance is already
   running".
4. Confirm the first window is intact and the script is still running (`pgrep`).
5. Close the first instance; start the app again; confirm normal start.

## 15. Out of scope

Raising/focusing the first window (needs single-instance plugin or window management —
deferred). Multi-window within one instance. Locking `registry.json` separately (the single
host invariant covers it). Any Windows/macOS lock semantics (Linux-only product).

## 16. Risks

- flock is advisory: a different application could open the same path. Accepted — the lock
  guards against sh-ui instances, not against arbitrary programs.
- If Tauri setup error handling shows a panic dialog instead of a clean exit, the second
  instance looks broken. Mitigation: verify the exact exit path in step 2 and prefer an
  explicit early `std::process::exit(0)` after the stderr line if setup-Err is not clean.

## 17. Acceptance criteria

```gherkin
Feature: Single host
  Scenario: Second instance refuses to start
    Given a sh-ui instance holds the instance lock
    When another sh-ui process starts
    Then the new process exits with code 0 within one second
    And it writes "another instance is already running" to stderr
    And it does not read or write live-runs.json

  Scenario: First instance is untouched
    Given the first instance has a live run
    When a second instance exits on the lock
    Then the live run's process group is still alive

  Scenario: Lock survives no process
    Given the holding process was killed
    When a new instance starts
    Then it acquires the lock and reaps stale groups as before
```
