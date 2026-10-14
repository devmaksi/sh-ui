# BUG-20260814-01 — guard-git lets a commit land on an unborn protected branch

status: logged
severity: medium
priority: P1
scope: bigpowers hooks (upstream, not sh-ui source)
discovered: 2026-08-14
discovered_during: seed-conventions on sh-ui

## Symptom

`git init -b main && git add -A && git commit -m "chore(seed): …"` ran through the guard and
created the repository's first commit directly on `main`. A later `git commit` on the same
branch was blocked as expected.

## Root cause

`hooks/pre-tool-use.sh` resolves the current branch before the tool command runs:

```bash
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
```

In a directory that is not a repository yet, or on an unborn branch, this returns `unknown`.
The protected-branch loop compares `CURRENT_BRANCH` against `main`/`master`, finds no match,
and allows the commit. The command that creates the protected branch in the same breath is
never inspected.

## Impact

Branch protection is silently skipped for the first commit of every new repository. Solo
developers who seed a project get an unprotected root commit and no signal that a gate was
missed.

## Reproduction

1. Create an empty directory outside any repository.
2. Run `git init -b main && git add -A && git commit -m "chore: seed"`.
3. Observe the commit lands on `main` with no block.
4. Run any further `git commit` on `main`. Observe the block.

## Proposed fix (upstream)

In `pre-tool-use.sh`, before the protected-branch loop:

1. If `CURRENT_BRANCH` is `unknown`, inspect the command string for `git init` with
   `-b main`, `-b master`, `--initial-branch=main`, or `--initial-branch=master`.
2. Treat a match as a protected-branch commit and deny unless `GIT_BIGPOWERS_LAND=1`.
3. Deny `git commit` when `CURRENT_BRANCH` is `unknown` and the command is not part of a
   repository bootstrap exemption.

## Local decision for sh-ui

The seed commit stays as the repository root commit. Reverting it would leave `main` with no
commits, which is worse. Every commit after it goes through `kickoff-branch` and
`scripts/land-branch.sh`. Recorded in `specs/state.yaml`.

## Not fixed here because

The file lives in the global bigpowers install (`~/.pi-env/dev/npm/node_modules/bigpowers/hooks/`).
Editing a shared global install from one project session would change behaviour for every
other project on this machine. Fix belongs upstream.
