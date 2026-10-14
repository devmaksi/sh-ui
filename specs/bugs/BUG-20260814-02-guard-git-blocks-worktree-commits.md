# BUG-20260814-02 — guard-git blocks commits made from a linked Git worktree

status: logged
severity: medium
priority: P1
scope: bigpowers hooks (upstream, not sh-ui source)
discovered: 2026-08-14
discovered_during: kickoff-branch for e01s01

## Symptom

`kickoff-branch` created a linked worktree at `../e01s01-app-shell` on branch
`feat/e01s01-app-shell`. Every `git commit` issued as
`cd /path/to/worktree && git commit -m "feat: …"` was refused with
"Direct commits to 'main' are forbidden", although the worktree HEAD was the feature branch.

## Root cause

`hooks/pre-tool-use.sh` resolves the branch once, in the hook's own working directory:

```bash
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
```

The hook never resolves the branch of the repository the command actually targets. When the
agent's shell starts in the primary checkout and changes directory inside the command string,
the hook still reads the primary checkout's HEAD. A `cd` inside the command does not change
what the hook inspected.

## Impact

The solo-git worktree workflow is unusable for agents: every commit from a worktree is
blocked, and the only way forward is to abandon the worktree and work on a branch in the
primary checkout. The guard produces a false block, not a false allow, so it is safe but
blocking.

## Reproduction

1. In a repository on `main`, run `git worktree add ../wt -b feat/x`.
2. From an agent session whose working directory is the primary checkout, run
   `cd ../wt && git commit -m "feat: x"`.
3. Observe the block. Run the same commit inside the worktree in a normal shell. Observe success.

## Proposed fix (upstream)

Resolve the target repository before judging the branch:

1. Parse a leading `cd <dir> &&` prefix, or a `-C <dir>` flag, from the command string.
2. Run `git rev-parse --abbrev-ref HEAD` with that directory as the working directory.
3. Fall back to the hook's own directory only when no target is present in the command.

## Local decision for sh-ui

Worktrees are not used in this project while the defect stands. Branches are checked out in
the primary checkout instead. Recorded in `specs/state.yaml` as DEV-02 and in
`specs/WORKFLOW-solo-git.md`.

## Not fixed here because

The hook lives in the global bigpowers install. Editing it from this project would change
behaviour for every other project on this machine.
