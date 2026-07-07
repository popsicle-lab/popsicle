import { useCallback, useEffect, useState } from "react";

import type { HealthResponse, RuntimeStatusResponse } from "@/api/types";
import { useConfig } from "@/hooks/useConfig";

export function useRuntimeStatus() {
  const { client, loaded } = useConfig();
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [runtime, setRuntime] = useState<RuntimeStatusResponse | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (opts?: { silent?: boolean }) => {
    if (!opts?.silent) setRefreshing(true);
    try {
      const [h, r] = await Promise.all([client.health(), client.runtimeState()]);
      setHealth(h);
      setRuntime(r);
      setError(null);
    } catch (e) {
      setHealth(null);
      setRuntime(null);
      setError(String(e));
    } finally {
      if (!opts?.silent) setRefreshing(false);
    }
  }, [client]);

  useEffect(() => {
    if (loaded) {
      refresh({ silent: true });
    }
  }, [loaded, client, refresh]);

  return {
    health,
    runtime,
    refreshing,
    error,
    refresh,
    serverOk: health !== null,
    runtimeOnline: runtime?.state === "online",
  };
}
