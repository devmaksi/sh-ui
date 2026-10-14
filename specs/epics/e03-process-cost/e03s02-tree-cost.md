# e03s02 — One summed figure over the whole process tree

## 1. Identity

Story ID: e03s02. Title: One summed figure over the whole process tree. Epic:
e03-process-cost. Status: planned.

## 2. Summary

A script that forks must still show one number. The host walks every live descendant of the
run pid, sums their CPU and RAM into one snapshot, and the UI shows that sum as the headline
figure with the per-process breakdown behind it.

## 3. Classification

type: feat · risk: P1 · context: domain · bcps: 8

## 4. Value

This is the epic's value test: one CPU number and one RAM number that include every process
the script started. That is the measurement the product exists to provide.

## 5. Users and triggers

A live run whose script spawns children. The headline figure updates on the refresh interval
and reflects the whole tree.

## 6. Domain language

**Process tree** — the run pid and every live descendant reached through `/proc` child lists.
**Tree sum** — one snapshot: summed CPU percent and summed resident bytes over the tree.
RAM is summed as RSS per process; shared pages count in each process, as `ps` reports them.
(Decision 2026-08-14, user chose summed RSS.)

## 7. Requirements

#### ADDED: The host walks the process tree
For a live run the host collects the run pid and every live descendant each refresh, using
the `procfs` child lists.

#### ADDED: One figure covers the whole tree
The headline CPU and RAM figures are the sum over the tree, not the script process alone.

#### ADDED: A child that exits mid-refresh does not corrupt the sum
A descendant that disappears between the walk and the read is dropped from that snapshot. It
is never counted twice and never leaves a stale contribution.

#### ADDED: The breakdown is available but not in the way
The headline is the sum. The per-process breakdown sits behind an explicit control.

## 8. Behaviour before

e03s01 shows CPU and RAM of the script process only. A forking script under-reports.

## 9. Behaviour after

A forking script shows one summed figure that includes its children, refreshed live, with a
breakdown behind a toggle.

## 10. Interface and data contracts

`ProcReader` gains `fn children(&self, pid: u32) -> Vec<u32>` — direct children of one pid.
`fn tree_pids(reader, root) -> Vec<u32>` walks depth-first. `fn tree_sum(reader, pids,
previous, wall_ms, clock)` returns one `TreeSnapshot { cpu_percent: f64, ram_bytes: u64 }`
plus the new baseline map for the next interval, reusing the e03s01 percent math and the same
`METRICS_REFRESH`.

`RunEvent::Metrics` gains `processes: Vec<ProcessCost>` — per-pid `cpu_percent` and
`ram_bytes`, capped at the named constant `BREAKDOWN_MAX_PROCESSES` (64). The headline sum
always covers the whole tree even when the breakdown is capped. App event:
`run://metrics { runId, cpuPercent, ramBytes, processes }`.

## 11. Affected modules

`src-tauri/src/metrics.rs` — tree walk and sum. `src-tauri/src/supervisor.rs` — the poller
samples the tree instead of one pid. `src/features/runs/` — headline plus breakdown control.

## 12. Implementation steps

1. Walk descendants depth-first through `ProcReader::children` (procfs `Task::children`) → verify: `cargo test --manifest-path src-tauri/Cargo.toml process_tree_walk`
2. Sum CPU and RAM across the tree into one snapshot, reusing the named interval constant → verify: `cargo test --manifest-path src-tauri/Cargo.toml tree_sum`
3. Drop a child that exits between walk and read without double counting → verify: `cargo test --manifest-path src-tauri/Cargo.toml tree_sum_handles_exit`
4. Show the summed figure as the headline with the breakdown behind a toggle → verify: `npm run test -- cost-readout-tree`
5. Manual verification with a script that forks known children → verify: the Verification Script below, recorded in `specs/verifications/`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e03s02-P1-01 | U-R | A spawned script that forks yields every live child in the walk |
| SC-e03s02-P1-02 | U-R | The sum equals the per-process samples added by hand |
| SC-e03s02-P1-03 | U-R | A child gone mid-refresh drops out; no double count, no panic |
| SC-e03s02-P2-04 | U-T | The headline shows the sum; the breakdown is behind a control |
| SC-e03s02-P1-M1 | M | A human forks three known children and reads at least their summed RAM |

## 14. Verification Script

1. Register a script that forks three children, each allocating a known memory size.
2. Start it.
3. Confirm the headline RAM figure is at least the sum of the three allocations.
4. Kill one child externally.
5. Confirm the figure drops by about that child's share without overshooting.

## 15. Out of scope

GPU and VRAM (e04), per-child historical graphs, cgroup accounting, threads counted
separately (the tree is processes, not tasks).

## 16. Risks

- A walk that races with re-parenting could miss or double count a fast-lived child. The
  snapshot rule (walk, then read, drop `None`) bounds the window; detect with the exit test.
- A runaway fork bomb grows the tree every interval. The walk is bounded by the refresh
  interval and reports what exists at that instant; no caching of stale pids.

## 17. Acceptance criteria

```gherkin
# SC-e03s02-P1-02
Scenario: One figure covers every child
  Given a run whose script forked three children with known samples
  When the host sums the tree
  Then the headline equals the hand-added sum

# SC-e03s02-P1-03
Scenario: A child that dies mid-refresh does not corrupt the figure
  Given a tree snapshot in progress
  When one child exits before it is read
  Then the next snapshot omits it exactly once

# SC-e03s02-P2-04
Scenario: The sum leads, the breakdown waits
  Given a live tree
  When the runs view renders
  Then the headline is the summed figure
  And the per-process breakdown appears only after the user opens it
```

## 18. Dependencies and prior art

`procfs 0.18.0` `[OK]`. Verified API: `Task::children(&self) -> ProcResult<Vec<u32>>` reads
`/proc/<pid>/task/<pid>/children`. Prior art: `ps --ppid` and `pstree` walk the same lists.

## 19. Error surfaces

A child that vanishes mid-walk is skipped silently. A `/proc` read failure on the root pid
ends the poller exactly as in e03s01.

## 20. Follow-ups

GPU and VRAM over the same tree arrive in e04.
