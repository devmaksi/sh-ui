// story: e01s02
// scenario: SC-e01s02-P1-09
import { describe, expect, it } from "vitest";
import { describeRegistryError } from "./registryErrors";
import type { RegistryErrorKind } from "./types";

const kinds: RegistryErrorKind[] = [
  "notFound",
  "notAFile",
  "notShellScript",
  "notReadable",
  "corrupt",
  "io",
];

describe("describeRegistryError", () => {
  it("names a different reason for every error kind", () => {
    const messages = kinds.map((kind) =>
      describeRegistryError({ kind, detail: "/home/user/thing" }),
    );

    const distinct = new Set(messages);
    expect(distinct.size).toBe(kinds.length);
  });

  it("keeps the path the host named inside the message", () => {
    const message = describeRegistryError({
      kind: "notShellScript",
      detail: "/home/user/notes.txt is not a .sh script",
    });

    expect(message).toContain("/home/user/notes.txt");
  });

  it("still says something useful when the host reports an unknown kind", () => {
    const message = describeRegistryError({
      kind: "brandNewKind" as RegistryErrorKind,
      detail: "unexpected host failure",
    });

    expect(message).toContain("unexpected host failure");
  });
});
