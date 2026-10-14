// story: e01s01 e02s01
export interface RunConfig {
  args: string[];
  env: Record<string, string>;
  workingDir: string | null;
}

export interface ScriptEntry {
  id: string;
  path: string;
  config: RunConfig;
}

export interface RegistryList {
  entries: ScriptEntry[];
}

/** story: e02s01 — the draft the UI sends for validation; parsing lives in Rust. */
export interface ConfigDraft {
  argsLines: string[];
  envLines: string[];
  workingDir: string;
}

/** Mirrors `RegistryError::kind` in src-tauri/src/registry.rs. */
export type RegistryErrorKind =
  | "notFound"
  | "notAFile"
  | "notShellScript"
  | "notReadable"
  | "corrupt"
  | "io"
  // story: e02s01
  | "badWorkingDir"
  | "badEnvVar";

export interface RegistryFailure {
  kind: RegistryErrorKind;
  detail: string;
}
