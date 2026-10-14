// story: e01s03 e03s01
import type { OutputStream, ProcessCost, RunHandle } from "./types";

export interface OutputEvent {
  runId: string;
  stream: OutputStream;
  chunk: string;
}

export interface EndedEvent {
  runId: string;
  code: number | null;
}

export interface MetricsEvent {
  runId: string;
  cpuPercent: number;
  ramBytes: number;
  processes: ProcessCost[];
}

export type OutputListener = (event: OutputEvent) => void;
export type EndedListener = (event: EndedEvent) => void;
export type MetricsListener = (event: MetricsEvent) => void;

/** The host capabilities the run feature needs. Tests supply a fake. */
export interface RunHost {
  startScript(entryId: string): Promise<RunHandle>;
  onOutput(listener: OutputListener): () => void;
  onEnded(listener: EndedListener): () => void;
  onMetrics(listener: MetricsListener): () => void;
}
