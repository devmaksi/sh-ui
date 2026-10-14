// story: e01s03
import type { OutputLine } from "./types";

interface OutputPanelProps {
  lines: OutputLine[];
  droppedCount: number;
}

/**
 * Read-only by design: no input, no send, no signal control. Output is appended in arrival
 * order and stderr is marked so it is never mistaken for normal output.
 */
export function OutputPanel({ lines, droppedCount }: OutputPanelProps) {
  return (
    <section
      aria-label="Script output"
      aria-live="polite"
      className="border border-neutral-200 bg-neutral-950 p-3 text-neutral-100"
    >
      {droppedCount > 0 ? (
        <p className="mb-2 text-xs text-neutral-400">
          {droppedCount} lines hidden — showing the last {lines.length}.
        </p>
      ) : null}

      {lines.length === 0 ? (
        <p className="text-sm text-neutral-400">No output yet.</p>
      ) : (
        <ol className="max-h-96 overflow-y-auto font-mono text-sm">
          {lines.map((line) => (
            <li
              key={line.id}
              data-testid="output-line"
              data-stream={line.stream}
              className={
                line.stream === "stderr"
                  ? "text-red-400"
                  : "text-neutral-100"
              }
            >
              {line.chunk}
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
