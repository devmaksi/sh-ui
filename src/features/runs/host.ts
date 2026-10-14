// story: e01s03
import type { OutputStream, RunHandle } from "./types";

export interface OutputEvent {
  runId: string;
  stream: OutputStream;
  chunk: string;
}

export interface EndedEvent {
  runId: string;
  code: number | null;
}

export type OutputListener = (event: OutputEvent) => void;
export type EndedListener = (event: EndedEvent) => void;

/** The host capabilities the run feature needs. Tests supply a fake. */
export interface RunHost {
  startScript(entryId: string): Promise<RunHandle>;
  onOutput(listener: OutputListener): () => void;
  onEnded(listener: EndedListener): () => void;
}
