# BUG-20260814-03 — run-verification-gates.sh runs the bigpowers self-test inside a consumer project

status: logged
severity: medium
priority: P1
scope: bigpowers scripts (upstream, not sh-ui source)
discovered: 2026-08-14
discovered_during: release-branch gates for e01s01

## Symptom

`bash scripts/run-verification-gates.sh` in sh-ui reported 17 passed, 1 skipped, 25 failed,
verdict FAIL. sh-ui has no golden gates of its own. Its real gate is `npm run preflight`, which
passes.

## Root cause

The suite is a self-test of the bigpowers repository. Each gate asserts artifacts that only
exist in that repository. Run from a consumer project, they fail for the wrong reason.

Observed while reproducing:

```
$ bash scripts/golden-g04-selftest.sh
FAIL cannot read skills-lock.json

$ bash scripts/golden-g09-yaml-roundtrip.sh
ERROR: fixture not found: /home/devmaksi/Project/sh-ui/specs/verifications/fixtures/yaml-roundtrip/sample.yaml
```

`scripts/` in this project is a symlink to the global bigpowers install, so the gate scripts are
found. Their fixtures and expected repository layout are not. One gate reported
`script not found: tests/run-golden-migrate-version.sh`, which is the same problem in a
different shape.

## Impact

A consumer project gets a red verdict it cannot act on. An agent following CONVENTIONS
§ Always Green either stops work on a false failure or learns to ignore gate output, which is
the behaviour the banned-phrases table exists to prevent.

## Proposed fix (upstream)

1. Detect whether the current repository is the bigpowers repository — presence of
   `skills-lock.json` and `skills/` is enough.
2. When it is not, exit with a clear message: this suite tests bigpowers itself, run the
   project Preflight recorded in `AGENTS.md` instead.
3. Alternatively, resolve fixtures relative to the bigpowers install rather than the caller.

## Local decision for sh-ui

The project gate is `npm run preflight`, plus `verify-tdd-red-commit.sh` with an explicit
`TDD_VERIFY_CMD` during `develop-tdd`. `run-verification-gates.sh` is not run in this project
while the defect stands. Recorded in `specs/state.yaml` as DEV-03.

## Not fixed here because

The scripts live in the global bigpowers install. Editing them from this project would change
behaviour for every other project on this machine.
