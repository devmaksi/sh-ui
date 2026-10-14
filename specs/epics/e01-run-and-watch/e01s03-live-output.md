# e01s03 — Start a script and read its live output

## 1. Identity

Story ID: e01s03. Title: Start a script and read its live output. Epic: e01-run-and-watch. Status: planned.

## 2. Summary

The user presses start on a registered script. The host spawns it, keeps it in a supervisor
map, captures stdout and stderr, and streams both to the UI as events. The UI appends them in
a read-only panel. When the app closes, the host terminates the process.

## 3. Classification

type: feat · risk: P0 · context: domain · bcps: 8

## 4. Value

This is the tracer bullet completing. After this story the product does the thing it exists
to do: run a script the user owns and show what it says.

## 5. Users and triggers

The user selects a registry entry and presses start. The host spawns the process. Output
arrives as the script writes it.

## 6. Domain language

**Run** — one execution of one entry, identified by a run id. **Supervisor** — the host
component that owns every live run. **Output stream** — the captured stdout or stderr of one
run, tagged by which one it is.

## 7. Requirements

#### ADDED: The user starts a registered script
The host spawns the registered file with an argv array. The host never builds a shell command
string from user data.

#### ADDED: The user reads live output
stdout and stderr reach the UI as they are produced, tagged by stream, appended in arrival order.

#### ADDED: The output view accepts no input
The panel has no text entry, no send action, and no signal control.

#### ADDED: The host terminates its children when the app exits
A run started by the host does not outlive the host.

## 8. Behaviour before

e01s02 ships a registry of entries. Nothing can be executed.

## 9. Behaviour after

Pressing start produces a live, appending, read-only output view for that run. Closing the
app leaves no process behind.

## 10. Interface and data contracts

Host command `start_script(entry_id: String) -> Result<RunHandle, StartError>`.
Host events: `run://started { run_id, entry_id, pid }`, `run://output { run_id, stream, chunk }`,
`run://ended { run_id, code }`. `StartError` is a closed set: `EntryNotFound`, `NotExecutable`,
`SpawnFailed`. Chunks are capped at a named constant so one burst cannot flood the UI.

## 11. Affected modules

`src-tauri/src/supervisor.rs` — new. `src-tauri/src/commands.rs` — start command.
`src/features/registry/` — start control. `src/features/output/` — new output panel.

## 12. Implementation steps

1. Add the supervisor map keyed by run id, with the spawn call using an argv array → verify: `cargo test --manifest-path src-tauri/Cargo.toml supervisor_tracks_run`
2. Capture stdout and stderr and emit them as tagged events → verify: `cargo test --manifest-path src-tauri/Cargo.toml output_events`
3. Enforce the chunk cap as a named constant → verify: `cargo test --manifest-path src-tauri/Cargo.toml output_chunk_cap`
4. Build the read-only output panel that appends and distinguishes the two streams → verify: `npm run test -- output-panel`
5. Terminate every live child when the host exits → verify: `cargo test --manifest-path src-tauri/Cargo.toml shutdown_kills_children`
6. Confirm the panel exposes no input control → verify: `npm run test -- output-panel-no-input`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e01s03-P0-01 | Rust unit | A spawned run appears in the supervisor with its pid |
| SC-e01s03-P0-02 | Rust unit | Output written by the script arrives tagged with its stream |
| SC-e01s03-P0-03 | Rust unit | No child process remains after host shutdown |
| SC-e01s03-P1-04 | UI unit | The panel appends chunks in arrival order and marks stderr distinctly |
| SC-e01s03-P1-05 | UI unit | The panel renders no text input and no send control |

## 14. Verification Script

1. Create `/tmp/echo-loop.sh` that prints one line per second to stdout and one warning line to stderr.
2. Register it in the app.
3. Press start.
4. Observe the first line within one second, and continuing.
5. Observe the stderr line rendered as a warning, not mixed with normal output.
6. Confirm the panel has no place to type.
7. Close the app.
8. Run `pgrep -f echo-loop.sh` and observe no process.

## 15. Out of scope

Interactive input, signals the user chooses, restart controls, scrollback limits, output
search, and log files on disk.

## 16. Risks

- Spawning through a shell string would let registry data execute arbitrary commands. The argv-array rule is the control. Detect with a test that registers a path containing shell metacharacters and asserts they are passed as one literal argument.
- A script that writes output very fast can flood the event channel. Detect with the chunk cap test.
- A child that outlives the host leaves an orphan. Detect with the shutdown test, not by manual inspection.

## 17. Acceptance criteria

```gherkin
Scenario: Start a script and watch it
  Given a registered script that prints once per second
  When the user presses start
  Then output appears within one second
  And output keeps appearing while the script runs

Scenario: Output is read only
  Given a run is live
  When the user looks at the output panel
  Then no input control exists

Scenario: Closing the app stops the script
  Given a run is live
  When the user closes the application
  Then no process from that run remains running
```

## 18. Dependencies and prior art

Tauri 2 shell execution and event API `[OK]`. No extra crate is introduced for spawning in
this story. Prior art is the standard `Command` spawn with an args array.

## 19. Error surfaces

`EntryNotFound`, `NotExecutable`, and `SpawnFailed` each render their own message with the
entry path included. A failed start never leaves the panel open and empty.

## 20. Follow-ups

User-triggered stop arrives in e02s02. Exit state and exit code arrive in e02s03. Crash
recovery of leftover processes arrives in e02s04. Metrics for the run arrive in e03s01.
