// story: e01s01 e01s02
import type { RegistryList, ScriptEntry } from "./types";

/** The host capabilities the registry feature needs. Tests supply a fake. */
export interface RegistryHost {
  listScripts(): Promise<RegistryList>;
  addScript(path: string): Promise<ScriptEntry>;
  removeScript(id: string): Promise<void>;
}
