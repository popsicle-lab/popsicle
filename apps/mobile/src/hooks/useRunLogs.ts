import { useCallback, useEffect, useState } from "react";

import type { RunLogEntry } from "@/api/types";
import { useConfig } from "@/hooks/useConfig";

export function useRunLogs(runId: string | undefined) {
  const { client } = useConfig();
  const [logs, setLogs] = useState<RunLogEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    if (!runId) return;
    setLoading(true);
    try {
      const list = await client.listRunLogs(runId);
      setLogs(list);
    } catch {
      setLogs([]);
    } finally {
      setLoading(false);
    }
  }, [client, runId]);

  useEffect(() => {
    reload();
  }, [reload]);

  useEffect(() => {
    if (!runId) return;
    const disconnect = client.connectEvents((event) => {
      if (
        event.type === "run_log" &&
        event.run_id === runId &&
        event.entry
      ) {
        setLogs((prev) => [...prev, event.entry!].slice(-200));
      }
    });
    return disconnect;
  }, [client, runId]);

  return { logs, loading, reload };
}
