---
status: accepted
date: 2026-08-14
---

# Runs are spawned as process-group leaders, executed directly, and capped on the way to the UI

Three choices decide how sh-ui controls a script it started. They are recorded together
because they all answer the same question: what does the host promise about a process it owns.

**Process groups.** Each run is spawned with `setsid` in a `pre_exec` block, so the script
becomes the leader of a new process group. Stopping the run is one `killpg(SIGTERM)`, and one
`killpg(SIGKILL)` for anything still alive after a two-second grace. A script that forks — a
build script, a dev server, a loop that calls other programs — is terminated completely, not
just its first process.

**Direct exec.** The host runs the registered file itself. If the file has no execute bit, the
host reports `NotExecutable` and names the path.

**Working directory.** A run starts in the directory that holds the script, not in the
directory the host happened to be started from.

**Output caps.** A captured line longer than `OUTPUT_CHUNK_MAX_BYTES` (8 KiB) is split into
several events. The UI keeps a ring buffer of 2000 lines and reports how many lines it dropped.

## Considered options

- **Process groups (`setsid` + `killpg`)** — chosen. One call reaches every descendant. Cost:
  one `unsafe` block and a dependency on `libc`, which Tauri already pulls in.
- **Walking `/proc` for descendants and killing bottom-up** — rejected. It races: a grandchild
  can re-parent itself between the scan and the kill, and the host would report a clean stop
  while a process survives.
- **The `command-group` crate** — rejected. It wraps the same two syscalls this project now
  calls directly, and one more dependency is not worth it for two calls.
- **Inherit the host's working directory** — rejected. It was the behaviour until 2026-08-14, and
  it broke the first script that reads a sibling file: `cat ./config` resolved against
  `src-tauri/` under `cargo test` and against the project root under `npm run dev`. A script
  should find its own files without the user knowing where the app was launched.
- **Running scripts through `sh <path>`** — rejected. It would let entries without an execute
  bit run, which sounds convenient and is not: the host would no longer be running the file the
  user registered, and `NotExecutable` would disappear from the contract. The honest failure is
  better than the convenient one.
- **Unbounded output in the UI** — rejected. A script printing megabytes would grow the webview
  until the app stops responding.

## Consequences

- `libc` is now a direct dependency of the Rust host.
- A script's relative paths resolve next to the script. e02s01 adds an explicit working
  directory per entry that overrides this default.
- `src-tauri/src/supervisor.rs` owns live runs. Nothing else may keep a `Child`.
- A run that ignores `SIGTERM` is killed after 2 seconds. A run that ignores `SIGKILL` cannot
  exist on Linux, so the host's promise is total.
- `StartError` is a closed set: `EntryNotFound`, `NotExecutable`, `SpawnFailed`. The UI maps
  each tag to its own message.
- The UI must show dropped output as a count, never hide it silently.
