// story: e01s01 e01s02
import { open } from "@tauri-apps/plugin-dialog";
import { AddScriptForm } from "./features/registry/AddScriptForm";
import { ScriptsList } from "./features/registry/ScriptsList";
import { tauriRegistryHost } from "./features/registry/tauriRegistryHost";
import { useRegistry } from "./features/registry/useRegistry";

async function pickScriptPath(): Promise<string | null> {
  const picked = await open({
    multiple: false,
    filters: [{ name: "Shell scripts", extensions: ["sh"] }],
  });

  return typeof picked === "string" ? picked : null;
}

export function App() {
  const { entries, error, reload, addScript, removeScript } =
    useRegistry(tauriRegistryHost);

  return (
    <main className="mx-auto max-w-3xl p-6">
      <h1 className="text-2xl font-semibold">sh-ui</h1>
      <p className="text-sm text-neutral-500">Local shell script manager</p>

      {error ? (
        <div role="alert" className="mt-4 border border-red-300 p-3 text-red-700">
          {error}
        </div>
      ) : null}

      <div className="mt-6">
        <AddScriptForm onAdd={addScript} pickPath={pickScriptPath} />
      </div>

      <div className="mt-6">
        <ScriptsList entries={entries} onRemove={removeScript} />
      </div>

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
