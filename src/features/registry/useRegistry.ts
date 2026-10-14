// story: e01s01 e01s02
import { useCallback, useEffect, useState } from "react";
import { describeRegistryError } from "./registryErrors";
import type { RegistryHost } from "./host";
import type { RegistryFailure, ScriptEntry } from "./types";

function toMessage(cause: unknown): string {
  if (
    typeof cause === "object" &&
    cause !== null &&
    "kind" in cause &&
    "detail" in cause
  ) {
    return describeRegistryError(cause as RegistryFailure);
  }

  return cause instanceof Error ? cause.message : String(cause);
}

export function useRegistry(host: RegistryHost) {
  const [entries, setEntries] = useState<ScriptEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  const applyList = useCallback((list: { entries: ScriptEntry[] }) => {
    setEntries(list.entries);
    setError(null);
  }, []);

  const applyError = useCallback((cause: unknown) => {
    setError(toMessage(cause));
  }, []);

  const reload = useCallback(
    () => host.listScripts().then(applyList).catch(applyError),
    [host, applyList, applyError],
  );

  const addScript = useCallback(
    async (path: string) => {
      try {
        const entry = await host.addScript(path);
        setEntries((current) => [...current, entry]);
        setError(null);
      } catch (cause) {
        applyError(cause);
      }
    },
    [host, applyError],
  );

  const removeScript = useCallback(
    async (id: string) => {
      try {
        await host.removeScript(id);
        setEntries((current) => current.filter((entry) => entry.id !== id));
        setError(null);
      } catch (cause) {
        applyError(cause);
      }
    },
    [host, applyError],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  return { entries, error, reload, addScript, removeScript };
}
