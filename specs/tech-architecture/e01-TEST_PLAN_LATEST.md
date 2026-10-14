# Test Design: e01-run-and-watch

Scope of this plan: epic e01 only. Risk class of the epic: P0. Produced by `plan-tests`
before implementation, consumed by `plan-work` story specs in §13 and §17.

## 1. Risk matrix and scenarios

Levels: **U-R** Rust unit, **U-T** TypeScript unit, **C** contract (Rust payload against the
TypeScript type), **G** gate (build or full-chain command), **M** manual UAT.

| Scenario ID | Behaviour | Risk | Level | Target file |
|---|---|---|---|---|
| SC-e01s01-P0-01 | `list_scripts` on an empty registry returns an empty list, not an error | P0 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s01-P0-02 | The UI renders an explicit empty state when entries are empty | P0 | U-T | `src/features/registry/ScriptsList.test.tsx` |
| SC-e01s01-P1-03 | The Rust payload and the TypeScript type describe the same shape | P1 | C | `tests/fixtures/registry-list.json` read by both suites |
| SC-e01s01-P1-04 | `npm run preflight` runs lint, Vitest, `cargo test`, and build | P1 | G | `npm run preflight` |
| SC-e01s01-P2-05 | The Linux release build links successfully | P2 | G | `cargo build --release --manifest-path src-tauri/Cargo.toml` |
| SC-e01s02-P0-01 | A valid readable `.sh` path is accepted, stored, and keeps its id | P0 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P0-02 | A directory path is rejected as `NotAFile` | P0 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P0-03 | A missing path is rejected as `NotFound` | P0 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P0-04 | A non-`.sh` file is rejected as `NotShellScript` | P0 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P0-05 | An unreadable file is rejected as `NotReadable` | P0 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P0-06 | Save then load returns identical entries, ids included | P0 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P1-07 | A corrupt registry file produces a named error, never a silent empty registry | P1 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P1-08 | Remove deletes the target entry and leaves the others | P1 | U-R | `src-tauri/src/registry.rs` tests |
| SC-e01s02-P1-09 | Each `RegistryError` variant renders its own message in the UI | P1 | U-T | `src/features/registry/RegistryErrors.test.tsx` |
| SC-e01s02-P2-10 | The dialog path lands in the form field and stays editable | P2 | U-T | `src/features/registry/AddScriptForm.test.tsx` |
| SC-e01s03-P0-01 | A spawned run appears in the supervisor with its pid | P0 | U-R | `src-tauri/src/supervisor.rs` tests |
| SC-e01s03-P0-02 | stdout and stderr arrive tagged with their stream | P0 | U-R | `src-tauri/src/supervisor.rs` tests |
| SC-e01s03-P0-03 | Host exit terminates every child it started | P0 | U-R | `src-tauri/src/supervisor.rs` tests |
| SC-e01s03-P0-04 | A path containing shell metacharacters is passed as one literal argument | P0 | U-R | `src-tauri/src/supervisor.rs` tests |
| SC-e01s03-P0-06 | The first output chunk arrives within 1000 ms of spawn | P0 | U-R | `src-tauri/src/supervisor.rs` tests |
| SC-e01s03-P1-05 | A burst of output respects the chunk cap | P1 | U-R | `src-tauri/src/supervisor.rs` tests |
| SC-e01s03-P1-06 | The panel appends in arrival order and marks stderr distinctly | P1 | U-T | `src/features/output/OutputPanel.test.tsx` |
| SC-e01s03-P1-07 | The panel renders no text input and no send control | P1 | U-T | `src/features/output/OutputPanel.test.tsx` |
| SC-e01s03-P1-08 | Each `StartError` variant renders its own message with the entry path | P1 | U-T | `src/features/registry/StartError.test.tsx` |
| SC-e01s03-P2-09 | A normally exiting script produces exactly one ended event | P2 | U-R | `src-tauri/src/supervisor.rs` tests |
| SC-e01s01-P0-M1 | A human opens the app and sees the empty registry | P0 | M | story §14 script |
| SC-e01s02-P0-M2 | A human adds a real script, restarts, and still sees it | P0 | M | story §14 script |
| SC-e01s03-P0-M3 | A human starts a looping script, reads output, closes the app, and finds no orphan | P0 | M | story §14 script |

Distribution: 15 Rust unit, 6 TypeScript unit, 1 contract, 2 gates, 3 manual. Nothing sits at
a higher level than it must.

## 2. Fixture architecture and isolation

**Registry path injection.** The registry module takes its file path as a parameter. No test
touches the real application data directory.

**`TempRegistry` factory.** Creates a throwaway directory, optionally prewrites a registry
JSON document, returns the path, and removes the directory on drop. Every registry test gets
its own instance.

**`FixtureScript` factory.** Writes a shell script of a given body into a throwaway directory
and returns its path. Bodies used: print once, print on a timer, write to stderr, fork three
children, exit non-zero, exit zero.

**Metacharacter fixture.** One `FixtureScript` body whose file name contains `$`, backticks,
`;`, and a space. The test asserts the supervisor received it as a single argv element.

**`FakeHostBridge`.** A named fake implementation of the host interface for UI tests. It
replays scripted events and records calls. No inline anonymous stubs.

**Shared contract fixture.** `tests/fixtures/registry-list.json` is the one payload sample.
The Rust test serialises its model and compares to this file. The Vitest contract test reads
the same file and asserts the TypeScript parser accepts it. A change to either side breaks
one of the two.

**Timing.** Latency tests use a real short script and a bounded wait, not a mocked clock. The
metrics interval is a parameter, so interval tests pass an explicit duration.

**Isolation rule.** No test depends on the order of another test, on the user's real scripts,
or on a network connection.

## 3. NFR verification

| NFR type | Requirement | Verification command |
|---|---|---|
| Latency | First output chunk within 1000 ms of spawn | `cargo test --manifest-path src-tauri/Cargo.toml output_first_chunk_latency` |
| Flood resistance | A 10000-line burst stays inside the chunk cap and does not grow host memory without bound | `cargo test --manifest-path src-tauri/Cargo.toml output_chunk_cap` |
| Clean exit | Zero processes from the app remain after host exit | `cargo test --manifest-path src-tauri/Cargo.toml shutdown_kills_children` |
| Durability | Registry save and reload produce identical content | `cargo test --manifest-path src-tauri/Cargo.toml registry_survives_reload` |
| Build health | Lint, both test suites, and build pass in one command | `npm run preflight` |

## 4. Out of scope

- Real-webview end-to-end automation. Driving the webview needs `tauri-driver` and WebDriver. Deferred until a later epic proves the manual scripts are a recurring cost.
- Visual and accessibility testing. Deferred.
- Duplicate-path registration behaviour. Not defined by the scope, so no test asserts it.
- Cross-distribution matrix. Tested on this machine only.
- GPU and VRAM measurement. Belongs to e04.
- Process-tree accounting. Belongs to e03.
