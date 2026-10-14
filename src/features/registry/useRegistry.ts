// story: e01s01 e01s02 e02s01
import { useCallback, useEffect, useState } from "react";
import { describeRegistryError } from "./registryErrors";
import type { RegistryHost } from "./host";
import type { ConfigDraft, RegistryFailure, ScriptEntry } from "./types";

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

  // story: e02s01 — the host validates; on success the stored entry replaces the old one.
  // The boolean lets the edit form stay open when the host refuses.
  const updateConfig = useCallback(
    async (id: string, draft: ConfigDraft): Promise<boolean> => {
      try {
        const updated = await host.updateScriptConfig(id, draft);
        setEntries((current) =>
          current.map((entry) => (entry.id === id ? updated : entry)),
        );
        setError(null);
        return true;
      } catch (cause) {
        applyError(cause);
        return false;
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

  return { entries, error, reload, addScript, removeScript, updateConfig };
}
