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

// story: e02s03 — the five states the run view renders. notRunning belongs to entries
// with no run record in this session; the four others come from the host.
export type RunEndStatus = "exited" | "failed" | "stopped";
export type RunState = "notRunning" | "running" | RunEndStatus;

/** One process inside the run's tree, for the breakdown behind the headline. */
export interface ProcessCost {
  pid: number;
  cpuPercent: number;
  ramBytes: number;
}

/** The newest metrics event for a run. Percent is of one core (DEC-12); the headline
 * sums cover the whole tree, the breakdown is capped by the host. Null gpu fields mean
 * not measured — never zero. */
export interface RunMetrics {
  cpuPercent: number;
  ramBytes: number;
  processes: ProcessCost[];
  vramBytes: number | null;
  gpuUtilPercent: number | null;
  gpuSource: string;
  gpuReason: string | null;
}

export interface Run {
  handle: RunHandle;
  state: RunState;
  exitCode: number | null;
  metrics: RunMetrics | null;
}

/** Mirrors `StartError::kind` in src-tauri/src/supervisor.rs. */
// story: e02s01
export type StartErrorKind =
  | "entryNotFound"
  | "notExecutable"
  | "spawnFailed"
  | "badWorkingDir";

export interface StartFailure {
  kind: StartErrorKind;
  detail: string;
}

/** A fast-printing script must not grow the webview without bound (ADR-0002). */
export const OUTPUT_LINE_BUFFER = 2000;
