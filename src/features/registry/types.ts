// story: e01s01
export interface ScriptEntry {
  id: string;
  path: string;
}

export interface RegistryList {
  entries: ScriptEntry[];
}

/** Mirrors `RegistryError::kind` in src-tauri/src/registry.rs. */
export type RegistryErrorKind =
  | "notFound"
  | "notAFile"
  | "notShellScript"
  | "notReadable"
  | "corrupt"
  | "io";

export interface RegistryFailure {
  kind: RegistryErrorKind;
  detail: string;
}
