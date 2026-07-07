import type {
  ApproveResult,
  DispatchRequest,
  DispatchResult,
  HealthResponse,
  MobileConfig,
  ResumeResult,
  RunLogEntry,
  RunMirror,
  RuntimeEvent,
  RuntimeStatusResponse,
} from "./types";

function normalizeBaseUrl(url: string): string {
  return url.trim().replace(/\/+$/, "");
}

async function parseJson<T>(resp: Response): Promise<T> {
  if (!resp.ok) {
    const text = await resp.text().catch(() => "");
    throw new Error(`HTTP ${resp.status}${text ? `: ${text}` : ""}`);
  }
  return resp.json() as Promise<T>;
}

export class AgentRuntimeClient {
  constructor(private cfg: MobileConfig) {}

  withConfig(cfg: MobileConfig): AgentRuntimeClient {
    return new AgentRuntimeClient(cfg);
  }

  private base(): string {
    return normalizeBaseUrl(this.cfg.serverUrl);
  }

  async health(): Promise<HealthResponse> {
    const resp = await fetch(`${this.base()}/health`);
    return parseJson(resp);
  }

  async runtimeState(): Promise<RuntimeStatusResponse> {
    const resp = await fetch(
      `${this.base()}/v1/runtimes/${encodeURIComponent(this.cfg.runtimeId)}`
    );
    return parseJson(resp);
  }

  async listRuns(): Promise<RunMirror[]> {
    const resp = await fetch(`${this.base()}/v1/runs`);
    return parseJson(resp);
  }

  async getRun(runId: string): Promise<RunMirror> {
    const resp = await fetch(`${this.base()}/v1/runs/${encodeURIComponent(runId)}`);
    return parseJson(resp);
  }

  async dispatch(body: Omit<DispatchRequest, "runtime_id" | "workspace_id"> & Partial<Pick<DispatchRequest, "runtime_id" | "workspace_id">>): Promise<DispatchResult> {
    const payload: DispatchRequest = {
      workspace_id: body.workspace_id ?? this.cfg.workspaceId,
      runtime_id: body.runtime_id ?? this.cfg.runtimeId,
      issue_key: body.issue_key,
      pipeline: body.pipeline,
    };
    const resp = await fetch(`${this.base()}/v1/dispatch`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    return parseJson(resp);
  }

  async approve(runId: string, stage: string): Promise<ApproveResult> {
    const resp = await fetch(
      `${this.base()}/v1/runs/${encodeURIComponent(runId)}/approve`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          runtime_id: this.cfg.runtimeId,
          stage,
        }),
      }
    );
    return parseJson(resp);
  }

  async resume(runId: string): Promise<ResumeResult> {
    const resp = await fetch(
      `${this.base()}/v1/runs/${encodeURIComponent(runId)}/resume`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          runtime_id: this.cfg.runtimeId,
          workspace_id: this.cfg.workspaceId,
        }),
      }
    );
    return parseJson(resp);
  }

  async listRunLogs(runId: string): Promise<RunLogEntry[]> {
    const resp = await fetch(
      `${this.base()}/v1/runs/${encodeURIComponent(runId)}/logs`
    );
    return parseJson(resp);
  }

  wsUrl(): string {
    const base = this.base();
    const wsBase = base.replace(/^http/, "ws");
    return `${wsBase}/v1/ws`;
  }

  connectEvents(onEvent: (event: RuntimeEvent) => void): () => void {
    const ws = new WebSocket(this.wsUrl());
    ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(String(msg.data)) as RuntimeEvent;
        onEvent(data);
      } catch {
        /* ignore malformed */
      }
    };
    return () => ws.close();
  }
}
