// story: e01s02
import { useState } from "react";

interface AddScriptFormProps {
  onAdd: (path: string) => Promise<void>;
  pickPath: () => Promise<string | null>;
}

export function AddScriptForm({ onAdd, pickPath }: AddScriptFormProps) {
  const [path, setPath] = useState("");
  const [busy, setBusy] = useState(false);

  async function chooseFile() {
    const picked = await pickPath();
    if (picked) {
      setPath(picked);
    }
  }

  async function submit() {
    const trimmed = path.trim();
    if (!trimmed) {
      return;
    }

    setBusy(true);
    try {
      await onAdd(trimmed);
    } finally {
      setBusy(false);
    }
  }

  return (
    <form
      className="flex flex-col gap-3"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <label htmlFor="script-path" className="text-sm font-medium">
        Script path
      </label>
      <div className="flex gap-2">
        <input
          id="script-path"
          type="text"
          value={path}
          onChange={(event) => setPath(event.target.value)}
          className="flex-1 border border-neutral-300 px-2 py-1 font-mono text-sm"
        />
        <button
          type="button"
          onClick={() => void chooseFile()}
          className="border border-neutral-300 px-3 py-1"
        >
          Choose file
        </button>
        <button
          type="submit"
          disabled={busy || !path.trim()}
          className="border border-neutral-900 bg-neutral-900 px-3 py-1 text-white disabled:opacity-40"
        >
          Add script
        </button>
      </div>

      {path.trim() ? (
        <p className="text-sm text-neutral-600">
          About to register: <span className="font-mono">{path.trim()}</span>
        </p>
      ) : null}
    </form>
  );
}
