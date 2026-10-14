// story: e01s03 e03s01
import type { OutputStream, ProcessCost, RunEndStatus, RunHandle } from "./types";

export interface OutputEvent {
  runId: string;
  stream: OutputStream;
  chunk: string;
}

export interface EndedEvent {
  runId: string;
  code: number | null;
  // story: e02s03 — the host-derived end verdict, once per run.
  status: RunEndStatus;
}

export interface MetricsEvent {
  runId: string;
  cpuPercent: number;
  ramBytes: number;
  processes: ProcessCost[];
  vramBytes: number | null;
  gpuUtilPercent: number | null;
  gpuSource: string;
  gpuReason: string | null;
}

export type OutputListener = (event: OutputEvent) => void;
export type EndedListener = (event: EndedEvent) => void;
export type MetricsListener = (event: MetricsEvent) => void;

/** The host capabilities the run feature needs. Tests supply a fake. */
export interface RunHost {
  startScript(entryId: string): Promise<RunHandle>;
  // story: e02s02
  stopRun(runId: string): Promise<void>;
  onOutput(listener: OutputListener): () => void;
  onEnded(listener: EndedListener): () => void;
  onMetrics(listener: MetricsListener): () => void;
}
