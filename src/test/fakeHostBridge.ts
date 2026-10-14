// story: e01s02 e02s01
// Named fake of the host interface, per the e01 test plan. Tests use this instead of
// inline stubs so the fake can record calls and replay scripted failures.
import type { RegistryHost } from "../features/registry/host";
import type { ConfigDraft, RegistryErrorKind, RegistryFailure, ScriptEntry } from "../features/registry/types";

let sequence = 0;

export class FakeHostBridge implements RegistryHost {
  private entries: ScriptEntry[] = [];
  private nextAddFailure: RegistryFailure | null = null;
  private nextUpdateFailure: RegistryFailure | null = null;

  readonly addCalls: string[] = [];
  readonly removeCalls: string[] = [];
  readonly updateCalls: { id: string; draft: ConfigDraft }[] = [];

  seed(entries: ScriptEntry[]): void {
    this.entries = [...entries];
  }

  rejectNextAdd(kind: RegistryErrorKind, detail: string): void {
    this.nextAddFailure = { kind, detail };
  }

  rejectNextUpdate(kind: RegistryErrorKind, detail: string): void {
    this.nextUpdateFailure = { kind, detail };
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
    const entry: ScriptEntry = {
      id: `fake-${sequence}`,
      path,
      config: { args: [], env: {}, workingDir: null },
    };
    this.entries.push(entry);
    return entry;
  }

  // story: e02s01 — mirrors the host contract: the draft is parsed here the way Rust
  // parses it: empty lines skipped, env split at the first =.
  async updateScriptConfig(id: string, draft: ConfigDraft): Promise<ScriptEntry> {
    this.updateCalls.push({ id, draft });

    if (this.nextUpdateFailure) {
      const failure = this.nextUpdateFailure;
      this.nextUpdateFailure = null;
      throw failure;
    }

    const entry = this.entries.find((candidate) => candidate.id === id);
    if (!entry) {
      throw { kind: "notFound", detail: id } satisfies RegistryFailure;
    }

    entry.config = {
      args: draft.argsLines.filter((line) => line !== ""),
      env: Object.fromEntries(
        draft.envLines
          .filter((line) => line !== "")
          .map((line) => {
            const eq = line.indexOf("=");
            return [line.slice(0, eq), line.slice(eq + 1)];
          }),
      ),
      workingDir: draft.workingDir === "" ? null : draft.workingDir,
    };
    return { ...entry };
  }

  async removeScript(id: string): Promise<void> {
    this.removeCalls.push(id);
    this.entries = this.entries.filter((entry) => entry.id !== id);
  }
}
