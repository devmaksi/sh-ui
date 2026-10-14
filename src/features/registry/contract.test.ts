// story: e01s01
// scenario: SC-e01s01-P1-03
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import type { RegistryList } from "./types";

const FIXTURE = resolve(
  process.cwd(),
  "src-tauri/tests/fixtures/registry-list.json",
);

function parseRegistryList(text: string): RegistryList {
  const parsed = JSON.parse(text) as RegistryList;
  if (!Array.isArray(parsed.entries)) {
    throw new Error("registry list payload must carry an entries array");
  }
  return parsed;
}

describe("registry list contract", () => {
  it("accepts the payload the Rust host produces", () => {
    const list = parseRegistryList(readFileSync(FIXTURE, "utf8"));

    expect(list.entries).toHaveLength(2);
    expect(list.entries[0]).toEqual({
      id: expect.any(String),
      path: expect.any(String),
      config: { args: [], env: {}, workingDir: null },
    });
  });
});
