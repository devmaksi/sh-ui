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

## Open questions

1. Application domain and top-level features — resolve with `scope-work`.
2. Server data access — plain fetch behind a slice-owned module, or TanStack Query.
3. Routing — not chosen.
4. Backend or API boundary — none defined yet. Defensive code covers retry, timeout, graceful degradation only.
