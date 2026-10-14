// story: e01s01
import type { ScriptEntry } from "./types";

interface ScriptsListProps {
  entries: ScriptEntry[];
}

export function ScriptsList({ entries }: ScriptsListProps) {
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
        <li key={entry.id} className="py-3">
          <span className="font-mono text-sm">{entry.path}</span>
        </li>
      ))}
    </ul>
  );
}
