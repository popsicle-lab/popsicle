import { useCallback, useEffect, useState } from "react";

import type { RunMirror } from "@/api/types";
import { useConfig } from "@/hooks/useConfig";
import { sanitizeRunMirror } from "@/utils/run-mirror";

export function useRuns() {
  const { client } = useConfig();
  const [runs, setRuns] = useState<RunMirror[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(
    async (opts?: { silent?: boolean }) => {
      if (!opts?.silent) setLoading(true);
      setError(null);
      try {
        const list = await client.listRuns();
        setRuns(list.map((run) => sanitizeRunMirror(run)));
      } catch (e) {
        setError(String(e));
      } finally {
        if (!opts?.silent) setLoading(false);
      }
    },
    [client]
  );

  const refresh = useCallback(async () => {
    setRefreshing(true);
    await reload({ silent: true });
    setRefreshing(false);
  }, [reload]);

  useEffect(() => {
    reload();
  }, [reload]);

  useEffect(() => {
    const disconnect = client.connectEvents((event) => {
      if (event.type === "run_updated" && event.mirror) {
        setRuns((prev) => {
          const existing = prev.find((r) => r.run_id === event.mirror!.run_id);
          const merged = sanitizeRunMirror(event.mirror!, existing);
          const next = prev.filter((r) => r.run_id !== merged.run_id);
          return [merged, ...next].sort(
            (a, b) => b.updated_at - a.updated_at
          );
        });
      }
    });
    return disconnect;
  }, [client]);

  return { runs, loading, refreshing, error, reload, refresh };
}
