// story: e01s03
import type { StartErrorKind, StartFailure } from "./types";

const REASONS: Record<StartErrorKind, string> = {
  entryNotFound: "That script is no longer in the registry.",
  notExecutable: "The file has no execute bit, so the host will not run it.",
  spawnFailed: "The host could not start the process.",
  // story: e02s01
  badWorkingDir: "The configured working directory no longer exists. Edit the entry.",
};

const UNKNOWN_REASON = "The host reported a start problem.";

/** Turns a start failure into one sentence that names the script involved. */
export function describeStartFailure(failure: StartFailure): string {
  const reason = REASONS[failure.kind] ?? UNKNOWN_REASON;

  return `${reason} ${failure.detail}`;
}
