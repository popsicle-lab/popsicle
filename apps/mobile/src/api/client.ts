import type {
  ApproveResult,
  BootstrapResult,
  ChatSession,
  ChatTurnResult,
  DispatchRequest,
  DispatchResult,
  HealthResponse,
  MobileConfig,
  ResumeResult,
  RunLogEntry,
  RunMirror,
  RuntimeEvent,
  RuntimeStatusResponse,
  UpdateChatDraftRequest,
  WorkflowsResponse,
} from "./types";
import { sanitizeRunMirror } from "@/utils/run-mirror";

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
    const list = await parseJson<RunMirror[]>(resp);
    return list.map((run) => sanitizeRunMirror(run));
  }

  async getRun(runId: string): Promise<RunMirror> {
    const resp = await fetch(`${this.base()}/v1/runs/${encodeURIComponent(runId)}`);
    return sanitizeRunMirror(await parseJson<RunMirror>(resp));
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

  async createChatSession(productId?: string): Promise<ChatSession> {
    const resp = await fetch(`${this.base()}/v1/chat/sessions`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        workspace_id: this.cfg.workspaceId,
        runtime_id: this.cfg.runtimeId,
        product_id: productId ?? "agent-runtime",
      }),
    });
    return parseJson(resp);
  }

  async getChatSession(sessionId: string): Promise<ChatSession> {
    const resp = await fetch(
      `${this.base()}/v1/chat/sessions/${encodeURIComponent(sessionId)}`
    );
    return parseJson(resp);
  }

  async postChatMessage(
    sessionId: string,
    content: string
  ): Promise<ChatTurnResult> {
    const resp = await fetch(
      `${this.base()}/v1/chat/sessions/${encodeURIComponent(sessionId)}/messages`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ role: "user", content }),
      }
    );
    return parseJson(resp);
  }

  async bootstrapChatSession(sessionId: string): Promise<BootstrapResult> {
    const resp = await fetch(
      `${this.base()}/v1/chat/sessions/${encodeURIComponent(sessionId)}/bootstrap`,
      { method: "POST", headers: { "Content-Type": "application/json" } }
    );
    return parseJson(resp);
  }

  async updateChatDraft(
    sessionId: string,
    body: UpdateChatDraftRequest
  ): Promise<ChatSession> {
    const resp = await fetch(
      `${this.base()}/v1/chat/sessions/${encodeURIComponent(sessionId)}/draft`,
      {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }
    );
    return parseJson(resp);
  }

  async listWorkflows(workspaceId: string): Promise<WorkflowsResponse> {
    const params = new URLSearchParams({ workspace_id: workspaceId });
    const resp = await fetch(`${this.base()}/v1/workflows?${params.toString()}`);
    return parseJson(resp);
  }
}
