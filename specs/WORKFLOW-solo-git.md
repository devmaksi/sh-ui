# sh-ui — Solo Git Workflow

Registered profile: `solo-git`. Source: bigpowers `profiles/solo-git.md`.
`specs/state.yaml` sets `workflow_mode: solo-git`. Skills read that key, not this file.

## Day-to-day arc

1. **On `main`:** `survey-context` → `scope-work` → `slice-tasks` → `plan-work` → `kickoff-branch`
2. **In the worktree:** `develop-tdd` / `execute-plan` → `verify-work` → `audit-code` → `commit-message`
3. **Integrate:** `release-branch` in **solo-local** mode → `bash scripts/land-branch.sh <branch> "<message>"`
4. **Done when:** `git branch --show-current` prints `main` and the feature worktree is gone

## Gates that stay mandatory

- Preflight green before forward work: `npm run lint && npm run test && npm run build`
- `bash scripts/run-verification-gates.sh` before every land
- Conventional Commits on every commit
- No commit on `main` outside `land-branch.sh`

## What is dropped

- Pull request ceremony. No `gh pr create`.
- Remote CI as a per-task blocker.

## Hooks

Install with `guard-git`. Hooks block commit and push on `main` unless `GIT_BIGPOWERS_LAND=1`, which only `land-branch.sh` sets.
