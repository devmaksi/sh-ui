# Verification record: e01s03 — Start a script and read its live output

Date: 2026-08-14. Branch: `feat/e01s03-live-output`. Automated gates passed, manual UAT
awaiting the user.

## Automated evidence

`cargo test` — 27 tests passed, 0 failed. `npm run test` — 10 files, 32 tests passed.
`npm run lint` — ESLint, both TypeScript projects, `cargo clippy --all-targets -- -D warnings`,
all clean. `npm run preflight` — exit 0.

| Task | Verify command | Result |
|---|---|---|
| 1 supervisor map, argv spawn | `cargo test … supervisor_tracks_run` | pass |
| 2 stdout and stderr as tagged events | `cargo test … output_events` | pass — 2 tests |
| 3 chunk cap as a named constant | `cargo test … output_chunk_cap` | pass — 2 tests |
| 4 no shell interpretation of a path | `cargo test … spawn_no_shell_interpretation` | pass |
| 5 first chunk within 1000 ms | `cargo test … output_first_chunk_latency` | pass — 2 tests |
| 6 exactly one ended event | `cargo test … ended_event_once` | pass |
| 7 shutdown kills every live child | `cargo test … shutdown_kills_children` | pass — 2 tests |
| 8 read-only output panel | `npm run test -- OutputPanel` | pass — 4 tests |
| 9 panel exposes no input | `npm run test -- OutputPanel.noInput` | pass |
| 10 each StartError renders its own message | `npm run test -- startErrors` | pass — 4 tests |

Extra coverage beyond the task list: `supervisor_busy_file` (2 tests) proves the host retries a
script that is briefly open for writing and reports it when the block never clears; `useRuns`
(5 tests) proves the UI buffer caps at 2000 lines, counts hidden lines, keeps stream tags, names
a start failure, and unsubscribes on unmount.

## TDD evidence, honestly classified

True red-then-green:

- `66d06ae` test → `3a55a8e` feat — supervisor tracking (red: module did not exist).
- `ea87d79` test → `041c3cc` feat — stream tagging and busy-file retry (red: `ended` arrived
  before the output pipes had drained, and a script held open for writing failed to start).
- `3e232ac` test → `f6dd082` feat — output panel and start error text (red: modules missing).
- `35c9b1a` test → `2ce031c` feat — run buffering over IPC (red: hook missing).
- `02b7f46` test → `1fa49a0` feat — start action per listed script (red: no start button).

Coverage tests with no red, because the behaviour was already correct when the test was written:

- `a774bc6` chunk cap, `1924d9c` hostile path stays one argument, `d33f95d` first-chunk latency,
  `452b9f6` single ended event, `14f3a47` shutdown kills the tree. These lock in promises the
  ADR makes; they did not drive the design.

## Flakiness found and fixed

`output_events` failed about once in ten runs with `Text file busy (os error 26)`. Confirmed
deterministically that holding a write handle on a script makes `exec` fail with ETXTBSY. The
exact source of the transient write handle in the test harness was not traced. The host now
retries a busy script 8 times at 25 ms intervals and, if it stays busy, reports `spawnFailed`
with the OS reason in the message. `output_events` then passed 20 consecutive runs.

Two further defects found while getting here:

1. `Ended` was emitted before both output pipes had drained, so a UI that closes on `ended`
   would lose the tail of the output. Fixed by gating `Ended` on both readers reaching EOF and
   the child being reaped.
2. Clippy flagged a nested `if` in `terminate_group`; rewritten as a named boolean
   `term_accepted` plus one condition, which reads better than what clippy wanted.

## Working directory fix (added after user review)

The user asked where a script actually runs. Measured answer at the time: nowhere was set, so
the run inherited the host's directory — `src-tauri/` under `cargo test`, the project root under
`npm run dev`. A script reading a sibling file by relative path therefore failed.

- `2b161f1` test → `94c2641` feat — the run now starts in the directory holding the script.
  The red was real and showed the defect: `cat data.txt` produced nothing and `pwd` reported
  `/home/devmaksi/Project/sh-ui/src-tauri`.
- Recorded as DEC-10 and as a rejected option in ADR-0002. e02s01 adds a per-entry override.

Suite after the fix: 28 Rust tests, 32 TypeScript tests, `npm run preflight` exit 0.

## Manual UAT — confirmed by the user on 2026-08-14

1. Create `/tmp/echo-loop.sh`: prints one line per second to stdout and one warning to stderr.
2. `chmod +x /tmp/echo-loop.sh`.
3. `npm run dev`, register the file, press Start.
4. Observe the first line within one second, and continuing.
5. Observe the stderr line rendered as a warning, not mixed with normal output.
6. Confirm the panel has no place to type.
7. Close the app while the script is still running.
8. Run `pgrep -f echo-loop.sh` and observe no process.

The user confirmed the run. Story status is `done` in `specs/execution-status.yaml`, and epic e01
is complete.
