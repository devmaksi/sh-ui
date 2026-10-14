// story: e01s01
import { useCallback, useEffect, useState } from "react";
import type { RegistryHost } from "./host";
import type { RegistryList, ScriptEntry } from "./types";

function toMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

export function useRegistry(host: RegistryHost) {
  const [entries, setEntries] = useState<ScriptEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  const apply = useCallback(
    (list: RegistryList) => {
      setEntries(list.entries);
      setError(null);
    },
    [],
  );

  const reject = useCallback((cause: unknown) => {
    setError(toMessage(cause));
  }, []);

  const reload = useCallback(
    () => host.listScripts().then(apply).catch(reject),
    [host, apply, reject],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  return { entries, error, reload };
}
