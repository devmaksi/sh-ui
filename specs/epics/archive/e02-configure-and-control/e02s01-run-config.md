# e02s01 — Arguments, environment, and working directory per script

## 1. Identity

Story ID: e02s01. Title: Arguments, environment, and working directory per script. Epic:
e02-configure-and-control. Status: planned.

## 2. Summary

Each registry entry gains a run configuration: an argv list, environment overrides, and an
optional working directory. The user edits it in the registry view with line-based fields —
one argument per line, one `KEY=VALUE` per line, one directory path. The host validates the
configuration when it is saved and starts the script with it: `exec` gets the stored argv
array, the stored environment merged over the host environment, and the stored directory or
the directory holding the script (DEC-10) when none is set.

## 3. Classification

type: feat · risk: P1 · context: domain · bcps: 6

## 4. Value

The product promises "register a script once, run it the way you would from a terminal".
Without per-script argv, env, and directory, a script that reads sibling files or takes flags
cannot be registered at all.

## 5. Users and triggers

A user registers a script, opens its edit form, fills in arguments, environment variables,
and a directory, saves, and starts. The configuration survives an app restart.

## 6. Domain language

**Run configuration** — the argv list, environment overrides, and working directory stored on
one registry entry. **argv item** — one line in the arguments field; it becomes exactly one
element of the exec argv array. No shell parsing exists anywhere. **Environment override** —
a `KEY=VALUE` pair the host sets on the spawned process on top of the inherited host
environment. **Working directory override** — a directory that replaces the default of
"the directory holding the script" (DEC-10).

## 7. Requirements

#### ADDED: A registry entry stores a run configuration
`ScriptEntry` gains `config: RunConfig { args: Vec<String>, env: BTreeMap<String, String>,
workingDir: Option<String> }`. A missing `config` in a stored document deserializes to the
empty configuration.

#### ADDED: The user edits the configuration with line-based fields
The registry view gains an edit form per entry: arguments textarea (one argv item per line),
environment textarea (one `KEY=VALUE` per line), working directory text field (empty means
the directory holding the script). Empty lines are skipped. Argument lines are kept
verbatim — no trimming, no quoting rules, because there is no shell.

#### ADDED: The host validates the configuration at save time
Working directory: a non-empty value must exist and be a directory → `badWorkingDir`.
Environment: the key must match `^[A-Za-z_][A-Za-z0-9_]*$` and the line must contain `=` →
`badEnvVar`. Validation lives in Rust only; the UI displays what the host rejects.

#### MODIFIED: Start uses the stored configuration
**Before:** the host execs the script with argv `[path]`, the inherited environment, and cwd
set to the directory holding the script.
**After:** the host execs with argv `[path, ...config.args]`, the inherited environment with
`config.env` applied on top, and cwd set to `config.workingDir` or the directory holding the
script when it is unset. A stored working directory that no longer exists at start time is
refused with `StartError::badWorkingDir` before the spawn.

#### MODIFIED: The registry document version accepts 1 and 2
**Before:** `load` rejects any document whose version is not `REGISTRY_FILE_VERSION` (1).
**After:** `load` accepts version 1 (entries get the empty configuration through serde
default) and version 2; `save` writes version 2.

## 8. Behaviour before

`ScriptEntry` is `{ id, path }`. Every start runs the script with no arguments, the host's
environment, and the script's own directory. There is no edit affordance in the registry view.

## 9. Behaviour after

The user configures arguments, environment, and directory per script from the UI; the host
validates on save and honors the configuration on every start, across restarts.

## 10. Interface and data contracts

Rust (`src-tauri/src/registry.rs`):

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunConfig {
    #[serde(default)] pub args: Vec<String>,
    #[serde(default)] pub env: BTreeMap<String, String>,
    #[serde(default)] pub working_dir: Option<String>,
}
```

`ScriptEntry` gains `#[serde(default)] pub config: RunConfig`. New command:

```rust
#[tauri::command]
pub fn update_script_config(
    registry, id: String,
    args_lines: Vec<String>, env_lines: Vec<String>, working_dir: String,
) -> Result<ScriptEntry, RegistryErrorPayload>
```

The command parses the raw lines (skip empty; split env at the first `=`), validates, stores,
saves, and returns the updated entry. New `RegistryError` variants `BadWorkingDir`,
`BadEnvVar` with tags `badWorkingDir`, `badEnvVar`. New `StartError` variant
`BadWorkingDir` with tag `badWorkingDir`.

TypeScript mirrors (`src/features/registry/types.ts`): `ScriptEntry` gains
`config: { args: string[]; env: Record<string, string>; workingDir: string | null }`;
`RegistryErrorKind` gains `"badWorkingDir" | "badEnvVar"`. `RunHost`
(`src/features/runs/types.ts`) `StartErrorKind` gains `"badWorkingDir"`. `RegistryHost` gains
`updateScriptConfig(id, draft: { argsLines: string[]; envLines: string[]; workingDir: string })`.

Zoom-out — `registry.rs`: purpose = the persistent registry document; callers =
`commands.rs`, `lib.rs`; contracts = camelCase payloads, save-on-every-mutation, atomic
temp+rename write. `supervisor.rs`: purpose = own and supervise live runs; callers =
`commands.rs::start_script`, `lib.rs` exit handler; contracts = argv arrays only (AGENTS
"Never"), setsid group leader, `StartError` tag stability with the TS union.

## 11. Affected modules

`src-tauri/src/registry.rs` — RunConfig, validation, version 2. `src-tauri/src/commands.rs` —
`update_script_config`, start passes config. `src-tauri/src/supervisor.rs` — `start` gains a
`&RunConfig` argument. `src-tauri/src/lib.rs` — command registration.
`src/features/registry/` — `RunConfigForm.tsx`, edit affordance in `ScriptsList.tsx`, host
and types. `src/features/runs/` — start error union.

## 12. Implementation steps

1. Add `RunConfig` to `ScriptEntry` with serde default; accept document versions 1 and 2 →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml run_config_store`
2. `update_script_config` with `badWorkingDir` and `badEnvVar` validation in Rust →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml run_config_validates`
3. `Supervisor::start` execs with stored argv, env over the inherited environment, stored
   cwd with a pre-spawn existence check →
   verify: `cargo test --manifest-path src-tauri/Cargo.toml start_uses_config`
4. `RunConfigForm` edit form with inline host errors; edit affordance in the scripts list →
   verify: `npm run test -- RunConfigForm`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e02s01-P1-01 | U-R | Config round-trips save/load; a version 1 document loads with the empty config |
| SC-e02s01-P1-02 | U-R | A missing or non-directory working dir is refused with `badWorkingDir` |
| SC-e02s01-P1-03 | U-R | `1BAD=x`, a line without `=`, and an empty key are refused with `badEnvVar` |
| SC-e02s01-P1-04 | U-R | Spawned child receives argv items, env overrides, and the stored cwd; unset cwd defaults to the script directory |
| SC-e02s01-P2-05 | U-T | The form prefills from the entry, submits drafts, and renders `badWorkingDir`/`badEnvVar` inline |
| SC-e02s01-P1-M1 | M | A script that prints its args, one env var, and `pwd` shows the configured values |

## 14. Verification Script

1. Register a script that prints `"$@"`, the value of `DEMO_VAR`, and `pwd`.
2. Open its edit form; enter two arguments (one containing a space), `DEMO_VAR=hello`, and a
   working directory different from the script's own. Save.
3. Start the run. Confirm the output shows both arguments (the spaced one intact), `hello`,
   and the chosen directory.
4. Enter a nonexistent directory. Confirm the save is refused inline with a readable message.
5. Restart the app. Confirm the configuration is still there.

## 15. Out of scope

Shell-style quoting or a command-line parser; per-run (not per-entry) configuration; env
secret masking; a directory picker widget (the path field is editable; the native dialog may
come later).

## 16. Risks

- Pid/path drift: a stored directory deleted between save and start. Mitigated by the
  pre-spawn check returning `badWorkingDir` instead of a raw OS error.
- Version 2 documents opened by an older build fail as `corrupt` — accepted; the app is
  single-version on this machine.
- Environment overrides inherit the host environment; a stored `PATH` replaces the host one.
  That matches terminal semantics and is stated in the form's help text.

## 17. Acceptance criteria

```gherkin
# SC-e02s01-P1-02
Scenario: Missing directory refused
  Given a draft whose working directory does not exist
  When the user saves the configuration
  Then the host refuses with kind badWorkingDir

# SC-e02s01-P1-04
Scenario: Start honors the configuration
  Given an entry with args ["a b", "--flag"], env DEMO_VAR=hello, working dir /tmp
  When the run starts
  Then the child receives argv ["a b", "--flag"], sees DEMO_VAR=hello, and its cwd is /tmp

# SC-e02s01-P2-05
Scenario: Host errors shown inline
  Given a save rejected with badEnvVar
  When the form renders the failure
  Then the message appears next to the environment field
```

## 18. Dependencies and prior art

No new packages. `BTreeMap` from std keeps env order deterministic in the JSON document.
Line-split-at-first-`=` is the same rule `env`(1) and Docker's `--env` flag use. Prior art in
this repo: `Registry::save`'s atomic temp+rename already guarantees the persistence half.

## 19. Error surfaces

`badWorkingDir` and `badEnvVar` on save; `badWorkingDir` on start; existing start errors
unchanged. Every refusal names the offending value in `detail`.

## 20. Follow-ups

None required by this story. A native directory picker is a possible later polish.
