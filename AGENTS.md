# sh-ui — AI Agents

> **Multi-agent context** — This file is the canonical project context for **Cline**, **Aider**, **OpenCode**, and other AGENTS.md-native tools. Claude Code and Cursor read it via the `CLAUDE.md` symlink.

Read CONVENTIONS.md before any GitHub or git operation.

<!-- BEGIN bigpowers:context-routing -->
## Context Routing

Load subdirectory context by file glob — see project-specific routing table (seeded by `seed-conventions`).

| Glob | Load |
|------|------|
| `src/components/**` | CONVENTIONS.md § UI Conventions |
| `src/features/**` | CONVENTIONS.md § UI Conventions, specs/tech-architecture/tech-stack.md |
| `specs/epics/**` | specs/state.yaml, specs/release-plan.yaml |
| `specs/bugs/**` | CONVENTIONS.md § Discovered Defects |
<!-- END bigpowers:context-routing -->

<!-- BEGIN bigpowers:learned-preferences -->
## Learned User Preferences

- User writes and expects replies in Russian.
- Integration mode is solo-git. No pull request ceremony.

## Workspace Facts

- Project seeded with `seed-conventions` on 2026-08-14.
- `scripts/` is a symlink to the global bigpowers install. Do not replace it with a local copy.
<!-- END bigpowers:learned-preferences -->

<!-- BEGIN bigpowers:project -->
## Project

sh-ui is a Linux desktop application that manages a registry of shell scripts.
Stack: TypeScript / React + Vite / Tauri (Rust host) / Node 22 / Tailwind CSS / Vitest

## Commands

| Action | Command |
|--------|---------|
| Run | `npm run tauri dev` |
| Test | `npm run test && cargo test --manifest-path src-tauri/Cargo.toml` |
| Build | `npm run tauri build` |
| Lint | `npm run lint && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` |
| Preflight | `npm run lint && npm run test && cargo test --manifest-path src-tauri/Cargo.toml && npm run build` |
| Land | `bash scripts/land-branch.sh <branch> "<conventional message>"` |

## Test

`npm run test` (Vitest). Typecheck runs inside `npm run lint` via `tsc --noEmit`.

## Lint

`npm run lint` (ESLint + `tsc --noEmit`).

## Build

`npm run build` (`tsc -b && vite build`).

## Architecture

`src-tauri/` is the process host. It spawns scripts, supervises them, captures output, and reads metrics.
`src/features/` holds UI slices. Each slice owns its components, hooks, and IPC calls.
`src/components/` holds shared presentational primitives only. UI talks to the host over Tauri IPC, never a socket.

## Conventions

- Named exports. Default exports are forbidden.
- Name component files `PascalCase.tsx`. Name hooks `useThing.ts`.
- Style with Tailwind utility classes. Do not add global CSS files.
- Keep IPC calls inside the feature slice that owns the data.
- Keep process and metric logic in Rust. Keep presentation logic in TypeScript.
- Tag every story implementation with `// story: eNNsNN` or `// story: eNNsNN` in Rust comments.

## Never

- Never dismiss reproducible gate failures as pre-existing or out of scope.
- Never proceed on red Preflight or red CI — invoke quick-fix or fix-bug first.
- Never add a dependency without an explicit user request.
- Never edit `.env*` files or commit secrets.
- Never write feature code without a plan in `specs/`.
- Never push directly to `main` except via `land-branch.sh`.
- Never listen on a network port. Tauri IPC is the only seam between UI and host.
- Never run a script through a shell string built from user input. Pass argv arrays.

## Agent Rules

- **Workflow Mandate:** Use bigpowers skills (e.g. `plan-work`, `develop-tdd`) for structured work.
- **Always Green:** Preflight and CI must be green before forward work.
- Read specs/ and CONVENTIONS.md before writing code.
- Write the minimum code that solves the stated problem.
- Run tests after every change. Show evidence before declaring done.
- All planning output goes in specs/.

## Token Economy — Minimal Footprint

1. **Check existing dependencies first.** DO inspect what your current dependencies already do before adding a package or writing your own code.
2. **Prefer mature, maintained libraries.** DO NOT rewrite a capability a maintained library provides without a documented reason.
3. **Copy validated patterns.** DO study how established products solve the same problem before inventing a new approach.
4. **Keep the simplest working implementation.** DO write the least code that satisfies the stated requirement. NEVER add preventive abstraction or unused config layers.
<!-- END bigpowers:project -->
