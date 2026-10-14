# sh-ui — Conventions

Rules for every AI agent working in this repository. Read AGENTS.md for commands and architecture.

## Conventional Commits & Semantic Versioning

All commits MUST follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).

Format: `<type>(<scope>): <description>`. Space after the colon is mandatory.

- `feat`: Minor bump. `fix` / `perf`: Patch bump.
- `docs`, `chore`, `style`, `refactor`, `test`: no bump.
- `BREAKING CHANGE:` or `!` after the type: Major bump.
- NEVER include `Co-authored-by` or any AI-attribution footer.

## Git Operations (solo-git mode)

`specs/state.yaml` sets `workflow_mode: solo-git`. This is the canonical integrate-mode signal.

- NEVER work directly on `main`. Start every task on a worktree or branch via `kickoff-branch`.
- Land with `bash scripts/land-branch.sh <branch> "<conventional message>"`. Do NOT open a pull request.
- `git push origin <feature-branch>` is allowed for backup. NEVER push to `main` except through `land-branch.sh`.
- Run `bash scripts/run-verification-gates.sh` before any land.
- NEVER call the GitHub REST API directly. NEVER create GitHub issues from skills — write local `.md` files in `specs/`.

## Agent Workflow Mandates

Agents MUST NOT bypass the bigpowers workflow.

- DO NOT write feature code from a bare prompt like "build this feature".
- Route work through skills: `survey-context` → `plan-work` → `develop-tdd` / `execute-plan`.
- Route bug reports through `investigate-bug` before any fix.
- Every story MUST end with a manual verification script for the user. Wait for user confirmation before declaring done.
- Every story MUST carry at least one `story: eNNsNN` tag in its implementing code or tests.
- Code generation without a plan in `specs/` is forbidden.

## Always Green / Shift Left

Solo developers own the whole codebase. **Always Green** means Preflight is green before any forward work.

**Shift Left (1-10-100):** a defect costs 1× in development, 10× in integration, 100× in production. Fix a red gate now.

**Preflight** — `npm run lint && npm run test && npm run build`. It MUST pass before kickoff, develop, or verify phases advance.

## Discovered Defects

Any reproducible gate failure found during unrelated work is a discovered defect. It is not background noise.

fix-or-log ladder, in order:

1. **quick-fix** — trivial, data-only, single-file fix.
2. **fix-bug** — when quick-fix guardrails abort, or the failure needs investigation. Write `specs/bugs/BUG-*.md` and fix with TDD.
3. **Log** — only when reproduction is blocked after a good-faith attempt. Write the BUG spec and stop forward work.

Ship discovered fixes in separate commits. NEVER narrate a failure and continue.

### Banned dismissive phrases

| Banned phrase | Required behavior instead |
|---------------|---------------------------|
| Pre-existing / pre-existing issues | Run fix-or-log; if truly unrelated, prove it with a passing repro after revert |
| Unrelated to this session | Same — session boundaries do not waive green gates |
| Not introduced by my changes | Bisect or fix anyway; the solo owner owns the whole tree |
| Out of scope (ignoring a red gate) | Invoke quick-fix or fix-bug; scope minimization never overrides Always Green |

## specs/ — All Planning Output Goes Here

| Path | Content |
|------|---------|
| `specs/state.yaml` | session state, `workflow_mode`, active epic, handoff |
| `specs/release-plan.yaml` | release index and epic sequence |
| `specs/epics/eNN-*.yaml` | stories and tasks with runnable `verify:` |
| `specs/execution-status.yaml` | done/pending per story |
| `specs/product/` | SCOPE, VISION, GLOSSARY |
| `specs/tech-architecture/` | tech-stack, test, design, security plans |
| `specs/adr/` | architecture decision records |
| `specs/bugs/registry.yaml` | bug registry |
| `specs/verifications/` | UAT evidence |

Every task needs `verify: <runnable command>`. Evidence over claims.

## UI Conventions

- Named exports only. Default exports are forbidden.
- Component files use `PascalCase.tsx`. Hook files use `useThing.ts`. Types live in the owning slice.
- Feature slices live in `src/features/<name>/` and own their components, hooks, and data access.
- `src/components/` holds shared presentational primitives. It MUST NOT import from `src/features/`.
- Style with Tailwind utility classes. Do NOT add global CSS files or inline style objects.
- Keep components under 150 lines. Extract a subcomponent when a render body grows.
- Props: explicit interfaces. No `any`. No optional props with unused fallbacks.
- Do not introduce a state library before a concrete problem needs one. Start with local state.

## Code Style

- Functions: 4–20 lines. Split if longer.
- Files: under 300 lines. One responsibility per module.
- Names: specific and unique. Avoid `data`, `handler`, `Manager`, `Service`.
- Early returns over nested ifs. Max 2 indentation levels.
- No magic strings or numbers. Extract every literal used in logic to a named constant.
- Complex boolean expressions go into a named predicate function.
- Remove dead code. Never comment code out.
- Boy Scout Rule: leave every touched file at least as clean as you found it.

## Comments

- Write WHY, not WHAT.
- Non-obvious logic MUST include a provenance link (ADR filename, commit SHA, or bug ID).
- No comments that restate the code.

## Tests (F.I.R.S.T)

- Tests run headless with `npm run test`.
- Every new function gets a test. Every bug fix gets a regression test.
- Tests are Fast, Independent, Repeatable, Self-Validating, Timely.
- Test boundaries: empty input, minimum, maximum, off-by-one.
- Assert on observable outcomes only. Never assert on internal state.
- Use Testing Library for components. Do not test implementation details.
- Never skip or ignore a test without a written note on what stays unresolved.

## Dependencies

- Prefer mature, maintained libraries over custom code.
- Add a dependency only on explicit user request, and record the reason in `specs/`.
- Inject dependencies by parameter, not by global import.

## Defensive Code

The agent implements defensive code only for these categories:

- **Retry** with backoff for transient failures, such as a metric read that fails once.
- **Timeout** for long-running operations, including a script that never exits.
- **Graceful degradation** when a host capability is missing, such as absent GPU tooling. Report the metric as unavailable. NEVER report zero as a measurement.

Process supervision is mandatory, not optional:

- Every launched process MUST be tracked by the host that launched it.
- The app MUST terminate its child processes on shutdown and on crash recovery.
- A stop request MUST escalate from graceful signal to forced kill after a bounded wait.

Rate limiting and circuit breakers do NOT apply — sh-ui has no network service and no remote hosts.

## Formatting

- Use Prettier via `npm run lint`. No style debates beyond that.
