# e01s01 — App shell shows the script list

## 1. Identity

Story ID: e01s01. Title: App shell shows the script list. Epic: e01-run-and-watch. Status: planned.

## 2. Summary

Build the application once, end to end: a Tauri window, a React UI, a Rust host, and one
working call between them. The first working call returns the script registry, which is
empty at this point. The UI renders that empty registry as a clear empty state.

## 3. Classification

type: feat · risk: P0 · context: infra · bcps: 5

## 4. Value

Nothing else in the release can be built before this. It proves the seam between UI and host
carries typed data, so every later story builds on a path that already works.

## 5. Users and triggers

The user opens the application. The application asks the host for the registry and shows it.

## 6. Domain language

**Registry** — the set of scripts the user has added. **Host** — the Rust side of the app that
owns processes. **Run** — one execution of one registered script. This story introduces the
first two.

## 7. Requirements

#### ADDED: The application starts and shows the registry
The application opens a window, asks the host for the registry, and renders it. An empty
registry renders as an empty state with a call to add a script.

#### ADDED: One command runs the whole local verification stack
The project exposes a `preflight` script that runs lint, both test suites, and the build.

## 8. Behaviour before

No code exists. The repository holds conventions and specs only.

## 9. Behaviour after

`npm run tauri dev` opens a window that lists zero scripts with an empty state. `npm run preflight`
runs lint, Vitest, `cargo test`, and the build in one command.

## 10. Interface and data contracts

Host command `list_scripts() -> RegistryList`. `RegistryList { entries: Vec<ScriptEntry> }`.
At this story `ScriptEntry` carries `id` and `path` only. The UI calls it through
`@tauri-apps/api` `invoke` and renders the result. Payloads are typed on both sides. Untyped
JSON across the seam is not allowed.

## 11. Affected modules

`src/` — React app, `src/features/registry/`, `src/components/`. `src-tauri/` — host crate,
`src-tauri/src/registry.rs`, `src-tauri/src/commands.rs`. Root — `package.json`, ESLint,
Tailwind, Vitest config.

## 12. Implementation steps

1. Scaffold the app with `create-tauri-app` using the React + TypeScript template → verify: `npm run build && cargo build --manifest-path src-tauri/Cargo.toml`
2. Add npm scripts `dev`, `build`, `test`, `lint`, `tauri`, `preflight` matching AGENTS.md → verify: `npm run lint && npm run test`
3. Add the registry module in Rust with an empty list and a `list_scripts` command → verify: `cargo test --manifest-path src-tauri/Cargo.toml list_scripts`
4. Render the registry list with an explicit empty state → verify: `npm run test -- scripts-list`
5. Wire the `preflight` chain across both languages → verify: `npm run preflight`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e01s01-P0-01 | Rust unit | `list_scripts` returns an empty registry, not an error |
| SC-e01s01-P0-02 | UI unit | The list renders the empty state when entries are empty |
| SC-e01s01-P1-03 | Build gate | `npm run preflight` exits 0 |

Test files carry `// scenario: SC-…` and `// story: e01s01` comments.

## 14. Verification Script

1. Run `npm run tauri dev`.
2. Observe the window opens with no console errors.
3. Observe the registry area shows an empty state that says no scripts are registered.
4. Close the window.
5. Run `npm run preflight` and observe all four gates pass.

## 15. Out of scope

Adding scripts, running scripts, output display, metrics, and window chrome polish.

## 16. Risks

- Tauri Linux system dependencies missing, which fails the build at link time. Detect at step 1, not later.
- ESLint, Vitest, and Tailwind defaults fighting the Tauri template. Detect at step 2.
- The IPC contract drifts between Rust struct and TypeScript type. Detect by testing the payload shape at step 4.

## 17. Acceptance criteria

```gherkin
Scenario: First launch with no scripts
  Given the application has just been built
  When the user opens it
  Then the window renders
  And the registry area shows an empty state

Scenario: Preflight covers both languages
  Given a clean checkout
  When the user runs npm run preflight
  Then lint passes
  And Vitest passes
  And cargo test passes
  And the build passes
```

## 18. Dependencies and prior art

`@tauri-apps/cli` 2.12.2 `[OK]`, `@tauri-apps/api` 2.12.3 `[OK]`, `vitest` 5.0.4 `[OK]`,
`@testing-library/react` 16.3.3 `[OK]`, `tailwindcss` 4.3.3 `[OK]`. Scaffolding uses the
official `create-tauri-app` React + TypeScript template rather than a hand-built config.

## 19. Error surfaces

If the host command fails, the UI shows a named failure band with the command name and the
host error message. The UI never renders a blank area in place of an error.

## 20. Follow-ups

Registry mutation arrives in e01s02. Execution and output arrive in e01s03. Run metadata
(args, environment, working directory) arrives in e02s01.
