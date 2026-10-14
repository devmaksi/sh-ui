// story: e01s03
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

export interface Run {
  handle: RunHandle;
  state: RunState;
  exitCode: number | null;
}

/** Mirrors `StartError::kind` in src-tauri/src/supervisor.rs. */
export type StartErrorKind = "entryNotFound" | "notExecutable" | "spawnFailed";

export interface StartFailure {
  kind: StartErrorKind;
  detail: string;
}

/** A fast-printing script must not grow the webview without bound (ADR-0002). */
export const OUTPUT_LINE_BUFFER = 2000;
