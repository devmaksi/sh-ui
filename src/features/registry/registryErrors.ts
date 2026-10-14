// story: e01s02
import type { RegistryErrorKind, RegistryFailure } from "./types";

const REASONS: Record<RegistryErrorKind, string> = {
  notFound: "No file was found there.",
  notAFile: "That path is not a regular file.",
  notShellScript: "Only .sh files can be registered.",
  notReadable: "The file exists but cannot be read.",
  corrupt: "The saved registry cannot be read.",
  io: "The registry file could not be written.",
  // story: e02s01
  badWorkingDir: "That working directory does not exist or is not a directory.",
  badEnvVar: "Environment lines must be KEY=VALUE with a name that starts a letter or underscore.",
};

const UNKNOWN_REASON = "The host reported a registry problem.";

/** Turns a host error into one sentence a user can act on. */
export function describeRegistryError(failure: RegistryFailure): string {
  const reason = REASONS[failure.kind] ?? UNKNOWN_REASON;

  return `${reason} ${failure.detail}`;
}
