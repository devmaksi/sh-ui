# Verification record: e01s02 — Add a script by path and keep it across restarts

Date: 2026-08-14. Branch: `feat/e01s02-script-registry`. Automated gates passed, manual UAT
awaiting the user.

## Automated evidence

Suite at the time of writing: `cargo test` 14 tests passed, 0 failed. `npm run test` 6 files,
17 tests passed. `npm run lint` runs ESLint, both TypeScript projects, and `cargo clippy -D
warnings` — clean. `npm run preflight` exit 0.

| Task | Verify command | Result |
|---|---|---|
| 1 entry model, load, atomic save | `cargo test … registry_persistence` | pass |
| 2 accept a valid readable .sh | `cargo test … registry_add` | pass — 2 tests |
| 3 four distinct rejections | `cargo test … registry_rejects` | pass — 4 tests |
| 4 corrupt file is a named error | `cargo test … registry_corrupt` | pass — 2 tests |
| 5 remove by id | `cargo test … registry_remove` | pass — 2 tests |
| 6 write and reload identical | `cargo test … registry_survives_reload` | pass |
| 7 dialog plus editable path field | `npm run test -- registry-form` | pass — 3 tests |
| 8 every error variant its own message | `npm run test -- registry-errors` | pass — 3 tests |

Extra coverage added beyond the task list: `registry_store` (1 test) proves every mutation is
already on disk, and `useRegistry` (4 tests) proves the UI hook adds, removes, and names a
rejection instead of silently dropping it.

## TDD gate evidence

Red/green pairs, each verified with `verify-tdd-red-commit.sh` using
`TDD_VERIFY_CMD='cargo test --manifest-path src-tauri/Cargo.toml'`:

- `f78c830` test → `704d833` feat — persistence
- `4b2302d` test → `3422fd8` feat — accepting a path and assigning an id
- `10b93bc` test → `85e69e7` feat — four rejection reasons
- `51eb69c` test → `e328b14` feat — removal by id
- `18b2fd3` test → `cf84323` feat — store that saves on every mutation
- `a9055ad` test → `a6d03bb` feat — error sentences
- `7cf30a5` test → `16e0d53` feat — add form
- `e4feb57` test → `8a66ec7` feat — add and remove through the hook
- `1354f00` test → `9d999c0` feat — remove action per entry

One test-only commit, `ee2272f`, has no matching red: it covers corrupt-document handling that
was already implemented during task 1. It adds coverage, not a behaviour change.

## Discovered defects during this story

1. `npm run preflight` went red: `tauriRegistryHost.ts` used `ScriptEntry` without importing it.
   Fixed in `96b83e4`.
2. React Testing Library did not clean the DOM between tests because Vitest runs with globals
   disabled, so queries matched elements from earlier tests. Fixed in `f4dcce0` by registering
   `afterEach(cleanup)` in the vitest setup file.
3. `npm run lint` did not run `cargo clippy`, although CONVENTIONS says clippy is part of Lint.
   The lint script now runs ESLint, both tsconfigs, and clippy with `-D warnings`.

## Manual UAT — confirmed by the user on 2026-08-14

1. Run `npm run dev`.
2. Add a real `.sh` file through the dialog. Observe the path in the field, the
   "About to register:" line, and the entry appearing in the list.
3. Type the path of a directory and confirm. Observe a specific rejection naming the problem.
4. Close the app and start it again. Observe the entry is still listed.
5. Remove the entry. Observe it is gone, and still gone after another restart.
6. Run `npm run preflight` and observe all four gates pass.

The user reported all six steps passed. Story status is `done` in `specs/execution-status.yaml`.
