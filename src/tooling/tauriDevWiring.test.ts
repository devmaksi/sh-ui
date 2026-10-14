// story: e01s01
// scenario: SC-e01s01-P1-04
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

interface PackageJson {
  scripts: Record<string, string>;
}

interface TauriConfig {
  build: { beforeDevCommand: string; beforeBuildCommand: string };
}

function readJson<T>(path: string): T {
  return JSON.parse(readFileSync(resolve(process.cwd(), path), "utf8")) as T;
}

const scripts = readJson<PackageJson>("package.json").scripts;
const tauriBuild = readJson<TauriConfig>("src-tauri/tauri.conf.json").build;

function runsTauri(command: string): boolean {
  return /\btauri\b/.test(command);
}

/** Resolve `npm run <name>` to the body of that script. */
function scriptBody(command: string): string {
  const name = command.replace(/^npm run /, "");
  return scripts[name] ?? command;
}

describe("dev command wiring", () => {
  it("does not let tauri dev call a script that calls tauri dev", () => {
    const beforeDev = scriptBody(tauriBuild.beforeDevCommand);

    expect(runsTauri(beforeDev)).toBe(false);
  });

  it("starts the Vite dev server that tauri waits for on port 1420", () => {
    const beforeDev = scriptBody(tauriBuild.beforeDevCommand);

    expect(beforeDev).toContain("vite");
  });

  it("keeps the build gate free of tauri", () => {
    expect(runsTauri(scripts.build)).toBe(false);
    expect(runsTauri(scriptBody(tauriBuild.beforeBuildCommand))).toBe(false);
  });
});
