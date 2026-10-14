// story: e01s03 e03s01
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  EndedListener,
  MetricsListener,
  OutputListener,
  RunHost,
} from "./host";
import type { OutputStream, ProcessCost, RunHandle, StartFailure } from "./types";

interface RunOutputPayload {
  runId: string;
  stream: OutputStream;
  chunk: string;
}

interface RunEndedPayload {
  runId: string;
  code: number | null;
}

interface RunMetricsPayload {
  runId: string;
  cpuPercent: number;
  ramBytes: number;
  processes: ProcessCost[];
  vramBytes: number | null;
  gpuUtilPercent: number | null;
  gpuSource: string;
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

/** Subscribes once and hands the unsubscribe back to the caller. */
function subscribe<T>(
  name: string,
  onData: (payload: T) => void,
): () => void {
  let unlisten: UnlistenFn | null = null;
  let cancelled = false;

  void listen<T>(name, (event) => onData(event.payload)).then((off) => {
    if (cancelled) {
      off();
      return;
    }
    unlisten = off;
  });

  return () => {
    cancelled = true;
    unlisten?.();
  };
}

export const tauriRunHost: RunHost = {
  startScript: (entryId) =>
    invoke<RunHandle>("start_script", { entryId }).catch((cause: unknown) => {
      throw toStartFailure(cause);
    }),

  onOutput: (listener: OutputListener) =>
    subscribe<RunOutputPayload>("run://output", (payload) =>
      listener({
        runId: payload.runId,
        stream: payload.stream,
        chunk: payload.chunk,
      }),
    ),

  onEnded: (listener: EndedListener) =>
    subscribe<RunEndedPayload>("run://ended", (payload) =>
      listener({ runId: payload.runId, code: payload.code }),
    ),

  onMetrics: (listener: MetricsListener) =>
    subscribe<RunMetricsPayload>("run://metrics", (payload) =>
      listener({
        runId: payload.runId,
        cpuPercent: payload.cpuPercent,
        ramBytes: payload.ramBytes,
        processes: payload.processes,
        vramBytes: payload.vramBytes,
        gpuUtilPercent: payload.gpuUtilPercent,
        gpuSource: payload.gpuSource,
      }),
    ),
};
