// story: e01s03
// scenarios: SC-e01s03-P1-06, SC-e01s03-P1-08, SC-e03s01-P2-05
import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FakeRunHost } from "../../test/fakeRunHost";
import { OUTPUT_LINE_BUFFER } from "./types";
import { useRuns } from "./useRuns";

describe("useRuns", () => {
  it("appends output in arrival order and keeps the stream tag", async () => {
    const host = new FakeRunHost();
    const { result } = renderHook(() => useRuns(host));

    await act(async () => {
      await result.current.start("entry-1");
    });

    act(() => {
      host.emitOutput("run-1", "stdout", "out-1");
      host.emitOutput("run-1", "stderr", "warn-1");
      host.emitOutput("run-1", "stdout", "out-2");
    });

    const lines = result.current.linesFor("run-1");
    expect(lines.map((line) => line.chunk)).toEqual(["out-1", "warn-1", "out-2"]);
    expect(lines[1].stream).toBe("stderr");
  });

  it("caps the buffer and counts the lines it hid", async () => {
    const host = new FakeRunHost();
    const { result } = renderHook(() => useRuns(host));

    await act(async () => {
      await result.current.start("entry-1");
    });

    act(() => {
      for (let index = 1; index <= OUTPUT_LINE_BUFFER + 500; index += 1) {
        host.emitOutput("run-1", "stdout", `line-${index}`);
      }
    });

    const lines = result.current.linesFor("run-1");
    expect(lines.length).toBe(OUTPUT_LINE_BUFFER);
    expect(result.current.hiddenCountFor("run-1")).toBe(500);
    expect(lines[0].chunk).toBe("line-501");
    expect(lines[lines.length - 1].chunk).toBe(`line-${OUTPUT_LINE_BUFFER + 500}`);
  });

  it("names the reason when a start fails and adds no run", async () => {
    const host = new FakeRunHost();
    host.rejectNextStart("notExecutable", "/home/user/build.sh");

    const { result } = renderHook(() => useRuns(host));

    await act(async () => {
      await result.current.start("entry-1");
    });

    expect(result.current.runs).toHaveLength(0);
    expect(result.current.error).toContain("execute");
    expect(result.current.error).toContain("/home/user/build.sh");
  });

  it("attaches the newest metrics event to its run", async () => {
    const host = new FakeRunHost();
    const { result } = renderHook(() => useRuns(host));

    await act(async () => {
      await result.current.start("entry-1");
    });

    act(() => {
      host.emitMetrics("run-1", 12.5, 2097152, []);
      host.emitMetrics("run-1", 42.5, 1048576, [
        { pid: 4242, cpuPercent: 42.5, ramBytes: 1048576 },
      ]);
    });

    expect(result.current.runs[0].metrics).toEqual({
      cpuPercent: 42.5,
      ramBytes: 1048576,
      processes: [{ pid: 4242, cpuPercent: 42.5, ramBytes: 1048576 }],
    });
  });

  it("marks a run ended with its exit code", async () => {
    const host = new FakeRunHost();
    const { result } = renderHook(() => useRuns(host));

    await act(async () => {
      await result.current.start("entry-1");
    });

    act(() => {
      host.emitEnded("run-1", 0);
    });

    expect(result.current.runs[0].state).toBe("ended");
    expect(result.current.runs[0].exitCode).toBe(0);
  });

  it("unsubscribes from the host when the view goes away", async () => {
    const host = new FakeRunHost();
    const { unmount } = renderHook(() => useRuns(host));

    unmount();

    expect(host.listenerCount()).toBe(0);
  });
});
