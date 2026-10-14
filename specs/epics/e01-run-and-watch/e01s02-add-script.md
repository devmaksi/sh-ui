# e01s02 — Add a script by path and keep it across restarts

## 1. Identity

Story ID: e01s02. Title: Add a script by path and keep it across restarts. Epic: e01-run-and-watch. Status: planned.

## 2. Summary

The user picks a `.sh` file, the host validates it, stores it in the registry, and writes the
registry to disk. On the next launch the host reads the file back and the list shows the same
entries. The user can also remove an entry.

## 3. Classification

type: feat · risk: P0 · context: domain · bcps: 5

## 4. Value

Without persistence the product is a per-session toy. The user builds a library of scripts
once and returns to it.

## 5. Users and triggers

The user opens the file picker or types a path, confirms, and the entry appears. The user
removes an entry with the remove action. The user restarts the app and the entries remain.

## 6. Domain language

**Script entry** — one registered script, identified by a stable entry id. **Registry file** —
the JSON document in application data that holds all entries.

## 7. Requirements

#### ADDED: The user registers a script by path
The user selects a file through the native dialog or types its path. The host accepts only an
existing readable file with a `.sh` suffix. A rejected path produces a typed error the UI can
render.

#### ADDED: The registry survives a restart
The host writes the registry to application data on every mutation and loads it at startup.

#### ADDED: The user removes an entry
Removal takes an entry id and deletes that entry only.

## 8. Behaviour before

e01s01 ships a host command that always returns an empty registry.

## 9. Behaviour after

The list shows real entries. Entries survive closing and reopening the app. Removing an entry
removes it from disk.

## 10. Interface and data contracts

Host commands: `add_script(path: String) -> Result<ScriptEntry, RegistryError>`,
`remove_script(id: String) -> Result<(), RegistryError>`, `list_scripts() -> RegistryList`.
`RegistryError` is a closed set: `NotFound`, `NotAFile`, `NotShellScript`, `NotReadable`,
`Persistence`. Each maps to one TypeScript error union member. The registry file is JSON:
`{ version: 1, entries: [{ id, path }] }`.

## 11. Affected modules

`src-tauri/src/registry.rs` — entry model, load, save. `src-tauri/src/commands.rs` — new
commands. `src/features/registry/` — add form, remove action, error band.

## 12. Implementation steps

1. Add the registry entry model and JSON load and save in Rust → verify: `cargo test --manifest-path src-tauri/Cargo.toml registry_persistence`
2. Add `add_script` with path validation → verify: `cargo test --manifest-path src-tauri/Cargo.toml registry_add`
3. Add `remove_script` by entry id → verify: `cargo test --manifest-path src-tauri/Cargo.toml registry_remove`
4. Add the native file dialog and the path field to the add form → verify: `npm run test -- registry-form`
5. Render every `RegistryError` variant as a distinct message → verify: `npm run test -- registry-errors`
6. Prove a write-reload cycle keeps entries identical → verify: `cargo test --manifest-path src-tauri/Cargo.toml registry_survives_reload`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e01s02-P0-01 | Rust unit | A valid `.sh` path is accepted and persisted |
| SC-e01s02-P0-02 | Rust unit | A directory, a missing file, and a non-`.sh` file are each rejected with the right error |
| SC-e01s02-P0-03 | Rust unit | Load after save returns identical entries |
| SC-e01s02-P1-04 | UI unit | The form shows the specific error text for each error variant |

## 14. Verification Script

1. Run `npm run tauri dev`.
2. Add a real `.sh` file through the dialog.
3. Observe the entry appears with its full path.
4. Try to add a directory. Observe a specific rejection message, not a generic one.
5. Close the app and start it again. Observe the entry is still listed.
6. Remove the entry. Observe it is gone, and still gone after another restart.

## 15. Out of scope

Editing run configuration, duplicate detection across different paths, importing a folder of
scripts, and any watcher that notices a file changed on disk.

## 16. Risks

- Entry ids regenerated at load would break removal and later run attribution. Detect by asserting id stability in the reload test.
- Writing the registry file non-atomically can corrupt it on a crash mid-write. Use write-to-temp-then-rename from the first commit.
- Application data path differs across Linux setups. Detect by resolving it through the Tauri path API, not by hand-building a path.

## 17. Acceptance criteria

```gherkin
Scenario: Register a script
  Given the application is open with an empty registry
  When the user selects an existing readable .sh file
  Then the entry appears in the list
  And the entry is present after the app restarts

Scenario: Reject a path that is not a script
  Given the user types the path of a directory
  When the user confirms the add
  Then the host rejects it with the NotAFile error
  And the UI names the problem in plain words

Scenario: Remove an entry
  Given the registry holds two entries
  When the user removes one
  Then only that entry disappears
```

## 18. Dependencies and prior art

`@tauri-apps/plugin-dialog` 2.8.2 `[OK]` for the native file picker. `@tauri-apps/plugin-store`
2.5.0 `[SUS]` is deliberately not used — persistence here is one Rust module with one file, and
a plugin would add a dependency without adding capability.

## 19. Error surfaces

Every rejection names the path and the reason. The UI never shows a generic "something went
wrong" for a registry error.

## 20. Follow-ups

Run metadata for each entry arrives in e02s01. Entry status while running arrives in e02s03.
