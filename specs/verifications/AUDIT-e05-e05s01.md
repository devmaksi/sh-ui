# Audit — e05s01 (feat/e05s01-single-instance)

Diff: `main..HEAD` — new `instance.rs` (62 lines), new `tests/instance_lock.rs`,
`lib.rs` setup wiring, specs artifacts. Date: 2026-10-15.

## Checklist

### Supply Chain & Security — ✓
- No new dependencies (`libc` already present; flock call, no shell strings).
- No secrets in diff. Threat model: specs/security/epics/e05/THREAT_MODEL.md — 0 HIGH,
  2 LOW accepted with rationale.

### Provenance & Metadata — ✓
- Story spec, tasks YAML, threat model, and execution-status timestamps all reference
  e05s01; code carries `// story: e05s01`, tests carry scenario tags (SC-e05s01-P1-01/02/03).

### Law of Demeter — ✓
- `lib.rs` calls one function on one module. No message chains.

### CONVENTIONS.md Compliance — ✓
- Files under 300 lines; functions under 40 lines; story tags in Rust comments; specs
  output in specs/; no global CSS/TS touched.

### Scope — ✓
- Changes limited to e05s01 modules. No behavior change to reap semantics.

### Boy Scout Rule — ✓
- `InstanceError` gained `std::error::Error` when the wiring needed it.
- Setup order in `lib.rs` is now explicitly lock → reap → registry → supervisor with a
  comment stating why.

### Types and Safety — ✓
- No unwrap in production code; typed error enum; single `unsafe` block (flock on an
  owned File's fd — sound: the File outlives the call).

### Test Coverage — ✓
- New public items covered: `acquire_instance_lock` (exclusive, re-acquire after drop,
  release on SIGKILL of an external holder via flock(1) — the OS release path, not Drop).
- F.I.R.S.T: Fast (0.02 s), Independent (per-test temp dirs), Repeatable (bounded poll,
  no sleeps longer than 20 ms), Self-validating, Timely.
- Full suite green after every change: 42 Rust suites + 53 Vitest tests.

### SOLID & Heuristics — ✓
- `instance.rs` single responsibility (lock acquisition only). No new data clumps.

### Code Style — ✓
- Largest new file 75 lines. `acquire_instance_lock` ~25 lines.

## Accepted deviations
- `std::process::exit(0)` inside the setup closure for the AlreadyRunning path — abrupt
  by design (spec §16 mitigation: clean exit beats a Tauri builder panic). Documented in
  the story spec and in the code comment.
- The kill-release test depends on `flock(1)` (util-linux). Accepted: Linux-only product.

## Manual verification evidence (SC-e05s01-P1-M1)
Release binary, two real instances: first alive (pid 577261) while second exited 0 with
one stderr line "sh-ui: another instance is already running"; after TERM to the first,
a third launch acquired the lock and ran the full observation window (exit 124 from
timeout). No leftover processes.

## Verdict
PASS. F.I.R.S.T: no violations.
