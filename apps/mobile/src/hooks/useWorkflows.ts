import { useFocusEffect } from "expo-router";
import { useCallback, useState } from "react";

import type { AgentRuntimeClient } from "@/api/client";
import type { WorkflowPipeline } from "@/api/types";

export function useWorkflows(
  client: AgentRuntimeClient,
  workspaceId: string,
  loaded: boolean
) {
  const [pipelines, setPipelines] = useState<WorkflowPipeline[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!loaded || !workspaceId.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const resp = await client.listWorkflows(workspaceId);
      setPipelines(resp.pipelines ?? []);
    } catch (e) {
      setError(String(e));
      setPipelines([]);
    } finally {
      setLoading(false);
    }
  }, [client, loaded, workspaceId]);

  useFocusEffect(
    useCallback(() => {
      refresh().catch(() => {});
    }, [refresh])
  );

  return { pipelines, loading, error, refresh };
}
