// story: e02s03
// scenario: SC-e02s03-P2-03
import type { RunEndStatus, RunState } from "./types";

const END_LABELS: Record<RunEndStatus, string> = {
  exited: "exited",
  failed: "failed",
  stopped: "stopped",
};

interface RunStatusBadgeProps {
  state: RunState;
  exitCode?: number | null;
}

/** One honest label per state. A failed run names the code when the host has one. */
export function RunStatusBadge({ state, exitCode }: RunStatusBadgeProps) {
  if (state === "running") {
    return (
      <span className="font-medium text-neutral-900" data-state="running">
        running
      </span>
    );
  }

  if (state === "notRunning") {
    return <span className="text-neutral-500">not running</span>;
  }

  const code = exitCode === null || exitCode === undefined ? "" : ` with code ${exitCode}`;

  return (
    <span
      className={
        state === "failed"
          ? "font-medium text-red-700"
          : "font-medium text-neutral-700"
      }
    >
      {END_LABELS[state]}
      {state === "failed" && code ? code : ""}
    </span>
  );
}
