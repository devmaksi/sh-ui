# e05s02 — Reaped orphans are visible in the UI

## 1. Identity

Story ID: e05s02. Title: Reaped orphans are visible in the UI. Epic:
e05-protect-single-host. Status: planned.

## 2. Summary

When the host starts and reaps orphaned process groups left by a crashed instance, the
fact is currently printed with `eprintln!` to a terminal a desktop user never sees. This
story delivers the same fact to the UI: the reaped groups are stored in host state at
startup, exposed over one IPC command, and shown as a dismissible banner in the runs view
with the count and the pids.

## 3. Classification

type: feat · risk: P2 · context: domain · bcps: 3

## 4. Value

The product promises honesty about what the host did to processes (the e04 theme). A
silent kill of a user's still-warm processes after a crash is exactly the fact the user
should see: "your previous app crashed; these script processes were terminated."

## 5. Users and triggers

The user restarts sh-ui after a crash or a kill. On first paint of the runs view a banner
states how many orphaned groups were reaped and lists their pids. One press dismisses it
for the session.

## 6. Domain language

**Reaped orphan** — a process group recorded in `live-runs.json` by a previous host that no
longer exists, terminated at startup through the `terminate_group` path (DEC-07). **Reap
report** — the list of reaped groups the startup produced, delivered to the UI once.

## 7. Requirements

#### ADDED: The startup reap report is available over IPC
The `Vec<LiveGroupRecord>` returned by `recovery::reap_stale` in `lib.rs` setup is stored
in managed state. A command `take_reap_report()` returns it once (take semantics: a second
call returns an empty list) as `ReapedGroup { pgid, pid }` payloads.

#### ADDED: The runs view shows a dismissible reap banner
On mount the frontend calls `take_reap_report`. A non-empty report renders a banner:
"Collected N orphaned process group(s): pid, pid". A dismiss control hides it; it does not
return until the next host start.

#### MODIFIED: The reap fact keeps its stderr line
**Before:** `eprintln!` only. **After:** `eprintln!` stays (host log) and the same fact
gains the UI path. No behavior change to reaping itself.

## 8. Behaviour before

Reaping is invisible to a desktop user.

## 9. Behaviour after

After a crash-restart the user sees exactly which groups were terminated and can dismiss
the notice.

## 10. Interface and data contracts

Rust (`commands.rs`, `lib.rs`): `take_reap_report() -> Vec<ReapedGroupPayload>` where
`ReapedGroupPayload { pgid: u32, pid: u32 }`, backed by a `Mutex<Option<Vec<...>>>`
managed state filled in setup. Frontend (`src/features/runs/`): `host.ts` gains
`takeReapReport(): Promise<ReapedGroup[]>`; a `ReapBanner` component in the runs slice
renders and dismisses it. TS type `ReapedGroup { pgid: number; pid: number }`.

## 11. Affected modules

`src-tauri/src/lib.rs` — store the reap result. `src-tauri/src/commands.rs` —
`take_reap_report`. `src/features/runs/` — `ReapBanner.tsx` (new), `host.ts`, `types.ts`,
slice mount point.

## 12. Implementation steps

1. Store the reap report in managed state + `take_reap_report` command (take-once
   semantics) → verify: `cargo test --manifest-path src-tauri/Cargo.toml --test reap_report`
2. `ReapBanner` in the runs slice: renders count and pids, dismiss hides it → verify:
   `npm run test -- ReapBanner`
3. Wire `takeReapReport` into the slice mount; banner absent when the report is empty →
   verify: `npm run test -- ReapBanner`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e05s02-P2-01 | U-T | Banner renders count and pid list from a non-empty report; empty report renders nothing |
| SC-e05s02-P2-02 | U-T | Dismiss removes the banner and it does not re-render on re-mount within the session |
| SC-e05s02-P2-03 | U-R | `take_reap_report` returns the stored list once; the second call returns empty |
| SC-e05s02-P2-M1 | M | Kill the host while a script runs, restart, read the banner, confirm the pid matches the killed script |

## 14. Verification Script

1. Start the app, start a long-running script.
2. `kill -9` the app process (the script survives as an orphan).
3. Start the app again.
4. Confirm the banner names the orphaned pid and the script is gone (`pgrep`).
5. Dismiss the banner; confirm it stays gone for the session.

## 15. Out of scope

Persisting the report across host starts (it describes this start only). Per-orphan
details (script name, exit time). History of past reaps.

## 16. Risks

- A reaped pid may already have exited between crash and restart; the banner then names a
  pid that no longer exists. Accepted — the report says what the host tried to reap, which
  is what happened.

## 17. Acceptance criteria

```gherkin
Feature: Reap visibility
  Scenario: Crash restart reports the reaps
    Given a previous host left live group records and died
    When the host starts and reaps them
    And the runs view mounts
    Then a banner states how many groups were reaped and their pids

  Scenario: Empty start stays quiet
    Given no live group records exist
    When the runs view mounts
    Then no banner is rendered

  Scenario: Dismiss is durable for the session
    Given the banner is visible
    When the user dismisses it
    Then it does not appear again until the next host start
```
