// story: e01s01 e01s02
import type { ScriptEntry } from "./types";

interface ScriptsListProps {
  entries: ScriptEntry[];
  onRemove?: (id: string) => void;
  onStart?: (id: string) => void;
}

export function ScriptsList({ entries, onRemove, onStart }: ScriptsListProps) {
  if (entries.length === 0) {
    return (
      <section aria-live="polite" className="py-8 text-neutral-500">
        No scripts are registered yet.
      </section>
    );
  }

  return (
    <ul className="divide-y divide-neutral-200">
      {entries.map((entry) => (
        <li key={entry.id} className="flex items-center gap-3 py-3">
          <span className="flex-1 font-mono text-sm">{entry.path}</span>
          {onStart ? (
            <button
              type="button"
              onClick={() => onStart(entry.id)}
              className="border border-neutral-900 bg-neutral-900 px-3 py-1 text-sm text-white"
            >
              Start {entry.path}
            </button>
          ) : null}

          {onRemove ? (
            <button
              type="button"
              onClick={() => onRemove(entry.id)}
              className="border border-neutral-300 px-2 py-1 text-sm"
            >
              Remove {entry.id}
            </button>
          ) : null}
        </li>
      ))}
    </ul>
  );
}
