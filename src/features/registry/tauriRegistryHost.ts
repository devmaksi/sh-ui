// story: e01s01 e01s02 e02s01
import { invoke } from "@tauri-apps/api/core";
import type { RegistryHost } from "./host";
import type { RegistryFailure, RegistryList, ScriptEntry } from "./types";

function toRegistryFailure(cause: unknown): RegistryFailure {
  if (
    typeof cause === "object" &&
    cause !== null &&
    "kind" in cause &&
    "detail" in cause
  ) {
    return cause as RegistryFailure;
  }

  return {
    kind: "io",
    detail: cause instanceof Error ? cause.message : String(cause),
  };
}

export const tauriRegistryHost: RegistryHost = {
  listScripts: () => invoke<RegistryList>("list_scripts"),

  addScript: (path) =>
    invoke<ScriptEntry>("add_script", { path }).catch((cause: unknown) => {
      throw toRegistryFailure(cause);
    }),

  removeScript: (id) =>
    invoke<void>("remove_script", { id }).catch((cause: unknown) => {
      throw toRegistryFailure(cause);
    }),

  // story: e02s01 — the draft goes raw; the host parses and validates.
  updateScriptConfig: (id, draft) =>
    invoke<ScriptEntry>("update_script_config", {
      id,
      argsLines: draft.argsLines,
      envLines: draft.envLines,
      workingDir: draft.workingDir,
    }).catch((cause: unknown) => {
      throw toRegistryFailure(cause);
    }),
};
