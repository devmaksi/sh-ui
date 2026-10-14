// story: e01s03
// Named fake of the run host, per the e01 test plan.
import type {
  EndedListener,
  OutputListener,
  RunHost,
} from "../features/runs/host";
import type { RunHandle, StartErrorKind, StartFailure } from "../features/runs/types";

export class FakeRunHost implements RunHost {
  readonly startCalls: string[] = [];

  private handles = new Map<string, RunHandle>();
  private outputListeners = new Set<OutputListener>();
  private endedListeners = new Set<EndedListener>();
  private nextFailure: StartFailure | null = null;
  private sequence = 0;

  rejectNextStart(kind: StartErrorKind, detail: string): void {
    this.nextFailure = { kind, detail };
  }

  async startScript(entryId: string): Promise<RunHandle> {
    this.startCalls.push(entryId);

    if (this.nextFailure) {
      const failure = this.nextFailure;
      this.nextFailure = null;
      throw failure;
    }

    this.sequence += 1;
    const handle: RunHandle = {
      runId: `run-${this.sequence}`,
      entryId,
      pid: 4000 + this.sequence,
    };
    this.handles.set(handle.runId, handle);
    return handle;
  }

  onOutput(listener: OutputListener): () => void {
    this.outputListeners.add(listener);
    return () => this.outputListeners.delete(listener);
  }

  onEnded(listener: EndedListener): () => void {
    this.endedListeners.add(listener);
    return () => this.endedListeners.delete(listener);
  }

  listenerCount(): number {
    return this.outputListeners.size + this.endedListeners.size;
  }

  emitOutput(runId: string, stream: "stdout" | "stderr", chunk: string): void {
    for (const listener of this.outputListeners) {
      listener({ runId, stream, chunk });
    }
  }

  emitEnded(runId: string, code: number | null): void {
    for (const listener of this.endedListeners) {
      listener({ runId, code });
    }
  }
}
