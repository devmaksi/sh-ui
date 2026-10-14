// story: e01s03 e03s01
export type OutputStream = "stdout" | "stderr";

export interface OutputLine {
  id: number;
  stream: OutputStream;
  chunk: string;
}

export interface RunHandle {
  runId: string;
  entryId: string;
  pid: number;
}

export type RunState = "running" | "ended";

/** One process inside the run's tree, for the breakdown behind the headline. */
export interface ProcessCost {
  pid: number;
  cpuPercent: number;
  ramBytes: number;
}

/** The newest metrics event for a run. Percent is of one core (DEC-12); the headline
 * sums cover the whole tree, the breakdown is capped by the host. */
export interface RunMetrics {
  cpuPercent: number;
  ramBytes: number;
  processes: ProcessCost[];
}

export interface Run {
  handle: RunHandle;
  state: RunState;
  exitCode: number | null;
  metrics: RunMetrics | null;
}

/** Mirrors `StartError::kind` in src-tauri/src/supervisor.rs. */
export type StartErrorKind = "entryNotFound" | "notExecutable" | "spawnFailed";

export interface StartFailure {
  kind: StartErrorKind;
  detail: string;
}

/** A fast-printing script must not grow the webview without bound (ADR-0002). */
export const OUTPUT_LINE_BUFFER = 2000;
