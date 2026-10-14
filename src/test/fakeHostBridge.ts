// story: e01s02
// Named fake of the host interface, per the e01 test plan. Tests use this instead of
// inline stubs so the fake can record calls and replay scripted failures.
import type { RegistryHost } from "../features/registry/host";
import type { RegistryErrorKind, RegistryFailure, ScriptEntry } from "../features/registry/types";

let sequence = 0;

export class FakeHostBridge implements RegistryHost {
  private entries: ScriptEntry[] = [];
  private nextAddFailure: RegistryFailure | null = null;

  readonly addCalls: string[] = [];
  readonly removeCalls: string[] = [];

  seed(entries: ScriptEntry[]): void {
    this.entries = [...entries];
  }

  rejectNextAdd(kind: RegistryErrorKind, detail: string): void {
    this.nextAddFailure = { kind, detail };
  }

  async listScripts() {
    return { entries: [...this.entries] };
  }

  async addScript(path: string): Promise<ScriptEntry> {
    this.addCalls.push(path);

    if (this.nextAddFailure) {
      const failure = this.nextAddFailure;
      this.nextAddFailure = null;
      throw failure;
    }

    sequence += 1;
    const entry: ScriptEntry = { id: `fake-${sequence}`, path };
    this.entries.push(entry);
    return entry;
  }

  async removeScript(id: string): Promise<void> {
    this.removeCalls.push(id);
    this.entries = this.entries.filter((entry) => entry.id !== id);
  }
}
