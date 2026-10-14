# sh-ui — Tech Stack

Status: seeded, decision ADR-0001 recorded. No source yet — update via `map-codebase` once code exists.

## Stack

| Layer | Choice |
|-------|--------|
| App shell | Tauri (ADR-0001) |
| Host (Rust) | process spawn, supervision, output capture, metric reading |
| UI | React + Vite, TypeScript strict |
| UI ↔ host | Tauri IPC: commands for calls, events for output and metrics |
| Runtime | Node 22 for the UI toolchain, cargo 1.98 for the host |
| Styling | Tailwind CSS (decision open — see `specs/state.yaml`) |
| Tests | Vitest + Testing Library (UI), `cargo test` (host) |
| Package manager | npm + cargo |

## Structure

```
src/
  components/   shared presentational primitives, no feature imports
  features/     UI slices: components + hooks + IPC calls
  hooks/        cross-feature hooks
  lib/          framework-free helpers
  styles/       Tailwind entry only
src-tauri/
  src/          host: registry, launcher, supervisor, metrics
  Cargo.toml
```

## Resolved

- Domain: manage local `.sh` scripts — registry, run lifecycle, read-only output, resource accounting. See `specs/product/SCOPE_LATEST.yaml`.
- Locality: local machine only. No SSH, no remote execution, no listening port.
- Terminal view: read-only stdout/stderr. No pty, no stdin.
- GPU: best effort through whatever vendor tool exists. Missing tooling is reported as unavailable.
- App shape: Tauri. Rust host inside the app process. See `specs/adr/0001-tauri-shell-with-rust-host.md`.

## Open questions

1. Metric source: `/proc` polling, or a crate that reads it.
2. GPU probe: which vendor tools to try, and in what order.
3. Registry persistence format: JSON file, SQLite, or YAML.
4. Routing and UI state library — decide in `slice-tasks`, not before.

## Environment verified on the dev machine (2026-08-14)

- node v22.23.2, npm 10.9.8
- cargo 1.98.0
- webkit2gtk 4.1 present
