// story: e01s03
// scenario: SC-e01s03-P1-08
import { describe, expect, it } from "vitest";
import { describeStartFailure } from "./startErrors";
import type { StartErrorKind } from "./types";

const kinds: StartErrorKind[] = ["entryNotFound", "notExecutable", "spawnFailed"];

describe("describeStartFailure", () => {
  it("names a different reason for every start error", () => {
    const messages = kinds.map((kind) =>
      describeStartFailure({ kind, detail: "/home/user/build.sh" }),
    );

    expect(new Set(messages).size).toBe(kinds.length);
  });

  it("keeps the entry path inside the message", () => {
    for (const kind of kinds) {
      const message = describeStartFailure({ kind, detail: "/home/user/build.sh" });
      expect(message).toContain("/home/user/build.sh");
    }
  });

  it("explains a missing execute bit as something the user can fix", () => {
    const message = describeStartFailure({
      kind: "notExecutable",
      detail: "/home/user/build.sh",
    });

    expect(message).toMatch(/execute/i);
  });

  it("still reports usefully when the host sends an unknown kind", () => {
    const message = describeStartFailure({
      kind: "brandNewKind" as StartErrorKind,
      detail: "odd host failure",
    });

    expect(message).toContain("odd host failure");
  });
});
