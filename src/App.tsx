// story: e01s01 e01s02 e01s03 e03s01 e02s01
import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { AddScriptForm } from "./features/registry/AddScriptForm";
import { RunConfigForm } from "./features/registry/RunConfigForm";
import { ScriptsList } from "./features/registry/ScriptsList";
import { tauriRegistryHost } from "./features/registry/tauriRegistryHost";
import { useRegistry } from "./features/registry/useRegistry";
import { CostReadout } from "./features/runs/CostReadout";
import { OutputPanel } from "./features/runs/OutputPanel";
import { RunControls } from "./features/runs/RunControls";
import { tauriRunHost } from "./features/runs/tauriRunHost";
import { useRuns } from "./features/runs/useRuns";

async function pickScriptPath(): Promise<string | null> {
  const picked = await open({
    multiple: false,
    filters: [{ name: "Shell scripts", extensions: ["sh"] }],
  });

  return typeof picked === "string" ? picked : null;
}

export function App() {
  const { entries, error: registryError, reload, addScript, removeScript, updateConfig } =
    useRegistry(tauriRegistryHost);
  const { runs, error: runError, start, stop, linesFor, hiddenCountFor } =
    useRuns(tauriRunHost);

  // story: e02s01 — one entry open in the edit form at a time.
  const [editingId, setEditingId] = useState<string | null>(null);
  const editingEntry = entries.find((entry) => entry.id === editingId);

  const activeRun = runs[runs.length - 1];
  const activeEntry = activeRun
    ? entries.find((entry) => entry.id === activeRun.handle.entryId)
    : undefined;

  return (
    <main className="mx-auto max-w-4xl p-6">
      <h1 className="text-2xl font-semibold">sh-ui</h1>
      <p className="text-sm text-neutral-500">Local shell script manager</p>

      {registryError || runError ? (
        <div role="alert" className="mt-4 border border-red-300 p-3 text-red-700">
          {registryError ?? runError}
        </div>
      ) : null}

      <div className="mt-6">
        <AddScriptForm onAdd={addScript} pickPath={pickScriptPath} />
      </div>

      <div className="mt-6">
        <ScriptsList
          entries={entries}
          onStart={start}
          onRemove={removeScript}
          onEdit={setEditingId}
        />
      </div>

      {editingEntry ? (
        <section className="mt-6 border border-neutral-200 p-4">
          <h2 className="font-mono text-sm">{editingEntry.path}</h2>
          <div className="mt-3">
            <RunConfigForm
              key={editingEntry.id}
              entry={editingEntry}
              error={registryError}
              onSave={async (draft) => {
                const saved = await updateConfig(editingEntry.id, draft);
                if (saved) {
                  setEditingId(null);
                }
              }}
            />
          </div>
        </section>
      ) : null}

      {activeRun ? (
        <section className="mt-6">
          <h2 className="text-sm font-medium">
            {activeEntry?.path ?? activeRun.handle.entryId} · pid{" "}
            {activeRun.handle.pid} ·{" "}
            {activeRun.state === "ended"
              ? `ended with code ${activeRun.exitCode ?? "unknown"}`
              : "running"}
          </h2>
          {/* story: e02s02 */}
          <div className="mt-2">
            <RunControls
              running={activeRun.state === "running"}
              onStop={() => void stop(activeRun.handle.runId)}
            />
          </div>
          <div className="mt-2">
            <CostReadout metrics={activeRun.metrics} />
          </div>
          <div className="mt-2">
            <OutputPanel
              lines={linesFor(activeRun.handle.runId)}
              droppedCount={hiddenCountFor(activeRun.handle.runId)}
            />
          </div>
        </section>
      ) : null}

      <button
        type="button"
        onClick={() => void reload()}
        className="mt-6 border border-neutral-300 px-3 py-1"
      >
        Reload
      </button>
    </main>
  );
}
