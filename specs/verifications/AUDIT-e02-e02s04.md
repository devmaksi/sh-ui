# Audit — e02s04 (feat/e02s04-shutdown-cleanup)

Diff: `main..HEAD`, 11 commits. Churn hotspot: `src-tauri/src/supervisor.rs` (12 commits/90d).

## Checklist

### Supply Chain & Security — ✓
- No new dependencies (procfs, serde already [OK] per DEC-11/DEC-04).
- No secrets in diff.
- Security scan done: specs/security/REVIEW.md — 0 HIGH, 2 LOW accepted with rationale.

### Provenance & Metadata — ✓
- e02s04-verify.yaml and REVIEW.md reference commits (c45ebc2, 4410a39) and DEC-07/DEC-11.

### Law of Demeter — ✓
- No message chains. `reap_stale` takes a `terminate` closure — supervisor internals stay private.

### CONVENTIONS.md Compliance — ✓
- All planning output in specs/. No gh/REST calls. Tailwind/TS conventions untouched (no UI changes).

### Scope — ✓
- Changes limited to e02s04 modules. Two in-scope gate failures (metrics-after-Ended flake,
  tree-walk fixture race) and one spec-parse defect (release-plan wsjf indent) were
  reproducible red-gate findings — fixed in-scope, not dismissed.

### Boy Scout Rule — ✓
- `stop_escalates.rs`: racy single-condition poll replaced with two-condition wait + per-condition asserts.
- `process_tree_walk.rs`: fixture made genuinely childless (exec), comment records the host fact.
- `watch_exit`: 8-argument signature replaced with `ExitWatch` struct (clippy too-many-arguments clean).

### Types and Safety — ✓
- No `any`, no ts-ignore, no casts. Rust: typed serde struct, no unsafe added beyond existing killpg.

### Test Coverage — ✓
- New public items: `start_time_ticks`, `load_records`, `LiveRunFile` (insert/remove/clear),
  `reap_stale`, `Supervisor::with_state_file` — all covered by crash_recovery_reaps and
  clean_exit_leaves_no_record. Shutdown path: shutdown_stops_all.
- Both flake fixes carry regression coverage in the tests that caught them
  (final_state_event, process_tree_walk) — proven by 3/3 green full-suite runs.
- F.I.R.S.T: tests are independent, repeatable (temp dirs), self-validating, behavior-level.

### SOLID & Heuristics — ✓
- recovery.rs: single responsibility (record + reap), injected terminate closure (DIP).
- No new Middle Man / Feature Envy. Data clumps (child/run_id/sink/…) bundled into ExitWatch.

### Code Style — ✗ (1 item, pre-existing)
- **supervisor.rs is 619 lines** (convention: <300). It was already ~580 before this story;
  this story added ~40. Splitting it (events/errors vs supervisor vs exit-watch) is a
  mechanical refactor touching every test import — deliberately NOT done inside e02s04 to
  keep the story diff reviewable. Recorded as a follow-up: plan-refactor on supervisor.rs.
- `start()` is long (~90 lines, pre-existing) — same follow-up.

**Resolution (2026-08-14, refactor/supervisor-split):** supervisor.rs split into
events.rs / errors.rs / output.rs / spawn.rs / watch_exit.rs; supervisor.rs is 279
lines, `start()` is 24 (validate/spawn_and_record/wire_run). Plan: specs/REFACTOR_LATEST.md.

### Red Flags named
- I initially reached for "pre-existing, out of scope" on the two red-preflight flakes and
  on the wsjf indent — caught myself; all three were fixed in-scope.
- The supervisor.rs length item is the one I am choosing to log rather than fix; the
  justification is scope containment, stated above, not silence.

## Verdict
PASS with one logged deviation (supervisor.rs length → follow-up refactor).
