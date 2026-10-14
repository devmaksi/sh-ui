// story: e01s03 e03s01
import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import { describeStartFailure } from "./startErrors";
import type { RunHost } from "./host";
import {
  OUTPUT_LINE_BUFFER,
  type OutputLine,
  type Run,
  type StartFailure,
} from "./types";

interface BufferState {
  lines: Record<string, OutputLine[]>;
  hidden: Record<string, number>;
}

type BufferAction = { type: "output"; runId: string; line: OutputLine };

const emptyBuffer: BufferState = { lines: {}, hidden: {} };

/** Keeps the last OUTPUT_LINE_BUFFER lines per run and counts what it hid. */
function bufferReducer(state: BufferState, action: BufferAction): BufferState {
  if (action.type !== "output") {
    return state;
  }

  const current = state.lines[action.runId] ?? [];
  const next = [...current, action.line];

  if (next.length <= OUTPUT_LINE_BUFFER) {
    return { lines: { ...state.lines, [action.runId]: next }, hidden: state.hidden };
  }

  const overflow = next.length - OUTPUT_LINE_BUFFER;

  return {
    lines: { ...state.lines, [action.runId]: next.slice(overflow) },
    hidden: {
      ...state.hidden,
      [action.runId]: (state.hidden[action.runId] ?? 0) + overflow,
    },
  };
}

function toStartFailure(cause: unknown): StartFailure {
  if (
    typeof cause === "object" &&
    cause !== null &&
    "kind" in cause &&
    "detail" in cause
  ) {
    return cause as StartFailure;
  }

  return {
    kind: "spawnFailed",
    detail: cause instanceof Error ? cause.message : String(cause),
  };
}

export function useRuns(host: RunHost) {
  const [runs, setRuns] = useState<Run[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [buffer, applyOutput] = useReducer(bufferReducer, emptyBuffer);
  const lineSequence = useRef(0);

  const start = useCallback(
    async (entryId: string) => {
      try {
        const handle = await host.startScript(entryId);
        setRuns((current) => [
          ...current,
          { handle, state: "running", exitCode: null, metrics: null },
        ]);
        setError(null);
      } catch (cause) {
        setError(describeStartFailure(toStartFailure(cause)));
      }
    },
    [host],
  );

  useEffect(() => {
    const offOutput = host.onOutput((event) => {
      lineSequence.current += 1;
      applyOutput({
        type: "output",
        runId: event.runId,
        line: { id: lineSequence.current, stream: event.stream, chunk: event.chunk },
      });
    });

    const offEnded = host.onEnded((event) => {
      setRuns((current) =>
        current.map((run) =>
          run.handle.runId === event.runId
            ? { ...run, state: "ended", exitCode: event.code }
            : run,
        ),
      );
    });

    const offMetrics = host.onMetrics((event) => {
      setRuns((current) =>
        current.map((run) =>
          run.handle.runId === event.runId
            ? {
                ...run,
                metrics: {
                  cpuPercent: event.cpuPercent,
                  ramBytes: event.ramBytes,
                  processes: event.processes,
                  vramBytes: event.vramBytes,
                  gpuUtilPercent: event.gpuUtilPercent,
                  gpuSource: event.gpuSource,
                },
              }
            : run,
        ),
      );
    });

    return () => {
      offOutput();
      offEnded();
      offMetrics();
    };
  }, [host]);

  const linesFor = useCallback(
    (runId: string): OutputLine[] => buffer.lines[runId] ?? [],
    [buffer],
  );

  const hiddenCountFor = useCallback(
    (runId: string): number => buffer.hidden[runId] ?? 0,
    [buffer],
  );

  return { runs, error, start, linesFor, hiddenCountFor };
}
