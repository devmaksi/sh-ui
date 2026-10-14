// story: e01s01
import { invoke } from "@tauri-apps/api/core";
import type { RegistryHost } from "./host";
import type { RegistryList } from "./types";

export const tauriRegistryHost: RegistryHost = {
  listScripts: () => invoke<RegistryList>("list_scripts"),
};
