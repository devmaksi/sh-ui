// story: e01s01
import { ScriptsList } from "./features/registry/ScriptsList";
import { tauriRegistryHost } from "./features/registry/tauriRegistryHost";
import { useRegistry } from "./features/registry/useRegistry";

export function App() {
  const { entries, error, reload } = useRegistry(tauriRegistryHost);

  return (
    <main className="mx-auto max-w-3xl p-6">
      <h1 className="text-2xl font-semibold">sh-ui</h1>
      <p className="text-sm text-neutral-500">Local shell script manager</p>

      {error ? (
        <div role="alert" className="mt-4 border border-red-300 p-3 text-red-700">
          <strong>list_scripts failed.</strong> {error}
        </div>
      ) : null}

      <div className="mt-6">
        <ScriptsList entries={entries} />
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
