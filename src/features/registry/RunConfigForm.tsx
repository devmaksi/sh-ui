// story: e02s01
// scenario: SC-e02s01-P2-05
import { useState } from "react";
import type { ConfigDraft, ScriptEntry } from "./types";

function envLinesOf(entry: ScriptEntry): string[] {
  return Object.entries(entry.config.env).map(([key, value]) => `${key}=${value}`);
}

interface RunConfigFormProps {
  entry: ScriptEntry;
  onSave: (draft: ConfigDraft) => Promise<void>;
  error: string | null;
}

/**
 * One entry's run configuration. Arguments are one argv item per line and environment
 * lines are KEY=VALUE — the draft is sent raw because parsing and validation live in
 * the host (e02s01 spec §7). Empty lines are the user's separators, not data.
 */
export function RunConfigForm({ entry, onSave, error }: RunConfigFormProps) {
  const [argsText, setArgsText] = useState(entry.config.args.join("\n"));
  const [envText, setEnvText] = useState(envLinesOf(entry).join("\n"));
  const [workingDir, setWorkingDir] = useState(entry.config.workingDir ?? "");
  const [busy, setBusy] = useState(false);

  async function submit() {
    setBusy(true);
    try {
      await onSave({
        argsLines: argsText.split("\n"),
        envLines: envText.split("\n"),
        workingDir: workingDir.trim(),
      });
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
      <label htmlFor="config-args" className="text-sm font-medium">
        Arguments
      </label>
      <p className="-mt-2 text-xs text-neutral-500">One argv item per line, passed as-is.</p>
      <textarea
        id="config-args"
        rows={3}
        value={argsText}
        onChange={(event) => setArgsText(event.target.value)}
        className="border border-neutral-300 px-2 py-1 font-mono text-sm"
      />

      <label htmlFor="config-env" className="text-sm font-medium">
        Environment
      </label>
      <p className="-mt-2 text-xs text-neutral-500">One KEY=VALUE per line.</p>
      <textarea
        id="config-env"
        rows={3}
        value={envText}
        onChange={(event) => setEnvText(event.target.value)}
        className="border border-neutral-300 px-2 py-1 font-mono text-sm"
      />

      <label htmlFor="config-working-dir" className="text-sm font-medium">
        Working directory
      </label>
      <p className="-mt-2 text-xs text-neutral-500">Empty means the directory holding the script.</p>
      <input
        id="config-working-dir"
        type="text"
        value={workingDir}
        onChange={(event) => setWorkingDir(event.target.value)}
        className="border border-neutral-300 px-2 py-1 font-mono text-sm"
      />

      {error ? (
        <div role="alert" className="border border-red-300 p-2 text-sm text-red-700">
          {error}
        </div>
      ) : null}

      <div>
        <button
          type="submit"
          disabled={busy}
          className="border border-neutral-900 bg-neutral-900 px-3 py-1 text-white disabled:opacity-40"
        >
          Save configuration
        </button>
      </div>
    </form>
  );
}
