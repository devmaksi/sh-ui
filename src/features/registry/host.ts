// story: e01s01
import type { RegistryList } from "./types";

/** The host capabilities the registry feature needs. Tests supply a fake. */
export interface RegistryHost {
  listScripts(): Promise<RegistryList>;
}
