# sh-ui — Tech Stack

Status: seeded, not yet verified by code. Update via `map-codebase` once source exists.

## Stack

| Layer | Choice |
|-------|--------|
| Language | TypeScript, strict mode |
| UI | React + Vite |
| Runtime | Node 22 |
| Styling | Tailwind CSS (decision open — see `specs/state.yaml`) |
| Tests | Vitest + Testing Library |
| Package manager | npm |

## Structure

```
src/
  components/   shared presentational primitives, no feature imports
  features/     domain slices: components + hooks + data access
  hooks/        cross-feature hooks
  lib/          framework-free helpers
  styles/       Tailwind entry only
```

## Resolved by scope (2026-08-14)

- Domain: manage local `.sh` scripts — registry, run lifecycle, read-only output, resource accounting. See `specs/product/SCOPE_LATEST.yaml`.
- Locality: local machine only. No SSH, no remote execution.
- Terminal view: read-only stdout/stderr. No pty, no stdin.
- GPU: best effort through whatever vendor tool exists. Missing tooling is reported as unavailable.
- A local host process launches and supervises scripts. It is not a network service.

## Open questions

1. UI shell shape: desktop wrapper (Tauri) or local web app served by the Node host.
2. Process and metric source: `/proc` polling, or a supervisor library.
3. Registry persistence format: JSON file, SQLite, or YAML.
4. Routing and state library — decide in `slice-tasks`, not before.
5. Data-fetching between UI and local host — plain fetch, or TanStack Query.
