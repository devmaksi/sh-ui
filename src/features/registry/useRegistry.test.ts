// story: e01s02
// scenarios: SC-e01s02-P0-01, SC-e01s02-P1-09
import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FakeHostBridge } from "../../test/fakeHostBridge";
import { useRegistry } from "./useRegistry";

describe("useRegistry", () => {
  it("lists what the host reports", async () => {
    const host = new FakeHostBridge();
    host.seed([
      { id: "a1", path: "/home/user/build.sh" },
      { id: "b2", path: "/home/user/deploy.sh" },
    ]);

    const { result } = renderHook(() => useRegistry(host));

    await waitFor(() => expect(result.current.entries).toHaveLength(2));
  });

  it("adds through the host and shows the new entry", async () => {
    const host = new FakeHostBridge();
    const { result } = renderHook(() => useRegistry(host));

    await waitFor(() => expect(result.current.entries).toHaveLength(0));

    await act(async () => {
      await result.current.addScript("/home/user/build.sh");
    });

    expect(result.current.entries.map((entry) => entry.path)).toEqual([
      "/home/user/build.sh",
    ]);
    expect(result.current.error).toBeNull();
  });

  it("keeps the list unchanged and names the reason when the host rejects", async () => {
    const host = new FakeHostBridge();
    host.rejectNextAdd("notShellScript", "/home/user/notes.txt is not a .sh script");

    const { result } = renderHook(() => useRegistry(host));

    await act(async () => {
      await result.current.addScript("/home/user/notes.txt");
    });

    expect(result.current.entries).toHaveLength(0);
    expect(result.current.error).toContain("Only .sh files can be registered.");
  });

  it("removes through the host and drops the entry", async () => {
    const host = new FakeHostBridge();
    host.seed([{ id: "a1", path: "/home/user/build.sh" }]);

    const { result } = renderHook(() => useRegistry(host));
    await waitFor(() => expect(result.current.entries).toHaveLength(1));

    await act(async () => {
      await result.current.removeScript("a1");
    });

    expect(result.current.entries).toHaveLength(0);
  });
});
