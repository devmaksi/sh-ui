# Verification record: e03s02 — One summed figure over the whole process tree

Date: 2026-08-14. Branch: `feat/e03s02-tree-cost`. Automated gates passed; manual UAT
confirmed by the user the same day.

## Automated evidence

`npm run preflight` — exit 0: ESLint, both TypeScript projects, `cargo clippy --all-targets
-- -D warnings`, 37 vitest tests (12 files), 44 cargo tests, `npm run build` clean.

| Task | Verify command | Result |
|---|---|---|
| 1 tree walk through /proc child lists | `cargo test … process_tree_walk` | pass — 2 tests (forker with 3 sleeps → 4 pids; childless pid → root alone) |
| 2 tree sum with named interval | `cargo test … tree_sum` | pass — 2 tests (3×100 ticks/1 s → 300%; new child contributes memory, zero cpu history) |
| 3 child exiting mid-refresh | `cargo test … tree_sum_handles_exit` | pass — dead pid drops exactly once, no carry into the next baseline |
| 4 headline + breakdown in UI | `npm run test -- CostReadoutTree` | pass — 2 tests; payload contract extended with capped `processes` (fixture updated) |
| 5 manual fork verification | script below | pass — confirmed by the user |

## TDD evidence

- `5ea1c99` test → `5b76d45` feat — tree walk (red: `tree_pids`/`children` unresolved).
- `ea94446` test → `9b9a1c0` feat — tree sum + poller re-walks the tree each interval
  (red: `tree_sum` unresolved).
- `334a25e` test → `362afc5` feat — breakdown in the payload (red: `ProcessCostPayload`
  missing from `MetricsPayload`).
- `841ae25` test → `2808d72` feat — headline + Details toggle (red: no toggle control).

Honest deviation, task 3: `dd435e6` is a post-hoc regression guard, not a red cycle — the
drop-on-`None` behavior was already the design of `tree_sum` from task 2, so no genuine
failure existed to reproduce. The guard pins it against future change.

Gate caveat, honestly stated: `verify-tdd-red-commit.sh` runs `npm test` in a fresh
worktree where `node_modules` is absent, so for Rust-only RED commits it passes vacuously.
The real isolation evidence for each Rust RED is the compile error shown before its commit
(`unresolved import`, `no method children`, `has no field named processes`).

## Manual UAT — confirmed by the user on 2026-08-14

1. `/tmp/fork-cost.sh` — shell forking three python3 children, each holding ~30 MB.
2. Registered and started in the app.
3. Headline RAM rose to at least the summed ~90 MiB of the three allocations.
4. Details showed the four processes with their shares.
5. One child killed externally; the headline dropped by roughly its share without overshoot.
6. Closing the app left no `fork-cost` process.

User confirmed: "Подтверждаю".

## Suite at land time

44 cargo tests, 37 vitest tests, preflight exit 0. Epic e03 is complete with this story.
