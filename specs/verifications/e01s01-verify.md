# Verification record: e01s01 — App shell shows the script list

Date: 2026-08-14. Branch: `feat/e01s01-app-shell`. Mode: automated gates passed, manual UAT
awaiting the user.

## Automated evidence

| Task | Verify command | Result |
|---|---|---|
| 1 scaffold | `npm run build && cargo build --manifest-path src-tauri/Cargo.toml` | pass — vite build ok, cargo finished dev profile in 45.89s |
| 2 npm scripts and preflight | `npm run preflight` | pass — exit 0 |
| 3 `list_scripts` | `cargo test --manifest-path src-tauri/Cargo.toml list_scripts` | pass — 1 passed |
| 4 UI list and empty state | `npm run test -- scripts-list` | pass — 2 passed |
| 5 contract fixture | `cargo test --manifest-path src-tauri/Cargo.toml contract_registry_list` | pass — 1 passed |
| 6 release target | `cargo build --release --manifest-path src-tauri/Cargo.toml` | pass — finished in 1m 28s |

Full suite at the time of writing: `cargo test` 2 passed, 0 failed. `npm run test` 2 files, 3
tests passed. `npm run lint` clean.

## TDD gate evidence

Two-commit red/green policy, checked with `verify-tdd-red-commit.sh`:

- `017b454 test(registry): specify that a fresh registry lists zero entries` →
  `PASS: test-only commit fails in isolation` (compile error: `could not find registry in sh_ui_lib`)
- `1f5a83c test(registry): specify empty state and listed script paths` → vitest failed on the
  missing `./ScriptsList` import before implementation landed.
- `b554a57 test(registry): pin the shared registry list payload for both sides` → both the Rust
  and the Vitest test failed before the fixture existed.

## Discovered defect during this story

`npm run preflight` went red on the first full run: test files import `node:fs` and `node:path`,
which the browser-only tsconfig could not typecheck. Fixed in `88533cd` by splitting type
checking into `tsconfig.json` (app, browser types) and `tsconfig.test.json` (node types, test
files only). Preflight then exited 0.

## Second discovered defect: `tauri dev` recursion

`npm run tauri dev` looped forever with `Waiting for your frontend dev server to start on
http://localhost:1420/`. Cause: the scaffold sets `build.beforeDevCommand` to `npm run dev`, and
the `dev` script had been changed to `tauri dev`. Tauri therefore re-launched itself instead of
starting Vite, so port 1420 never opened.

Fixed as a red/green pair:

- `d6b6837 test(tooling): specify that tauri dev must not call a tauri script` — 2 of 3 tests
  failed before the fix
- `fix(tooling): break tauri dev beforeDevCommand recursion` — `dev:client` runs Vite, and
  `beforeDevCommand` now points at it

Evidence after the fix, from `npm run dev`:

```
Running BeforeDevCommand (`npm run dev:client`)
VITE v8.3.4 ready in 148 ms
Local: http://localhost:1420/
Running DevCommand (`cargo run …`)
Running `target/debug/sh-ui`
```

One `BeforeDevCommand`, no repetition. The regression test `src/tooling/tauriDevWiring.test.ts`
resolves `npm run <name>` to the script body, so the loop cannot return silently.

## Manual UAT — confirmed by the user on 2026-08-14

Steps 1 to 5 were run by the user. Step 5 output was reported back in full:

- `npm run lint` clean
- `vitest run` — 3 files, 6 tests passed
- `cargo test` — 2 integration tests passed, 0 failed
- `vite build` — 21 modules transformed, built in 131 ms

The user confirmed the window opened, the empty state rendered, and closing the window behaved.

Story status: passing.

## Manual UAT script, for reruns

1. Run `npm run tauri dev`.
2. Observe the window opens with no console errors.
3. Observe the registry area shows "No scripts are registered yet."
4. Close the window.
5. Run `npm run preflight` and observe all four gates pass.
