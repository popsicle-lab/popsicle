export type RuntimeState = "online" | "offline";

export interface HealthResponse {
  status: string;
  storage: string;
}

export interface RuntimeStatusResponse {
  runtime_id: string;
  state: RuntimeState;
}

export interface StageMirror {
  name: string;
  status: string;
}

export interface RunMirror {
  run_id: string;
  issue_key?: string | null;
  pipeline: string;
  run_status: string;
  current_stage: string;
  stages: StageMirror[];
  updated_at: number;
}

export interface DispatchRequest {
  workspace_id: string;
  runtime_id: string;
  issue_key: string;
  pipeline: string;
}

export interface DispatchTask {
  id: string;
  issue_key: string;
  pipeline: string;
  phase: string;
  run_id?: string | null;
}

export interface DispatchResult {
  accepted: boolean;
  state: "queued" | "rejected";
  reason?: string | null;
  task?: DispatchTask | null;
}

export interface ResumeResult {
  accepted: boolean;
  state: "queued" | "rejected";
  reason?: string | null;
  task?: DispatchTask | null;
}

export interface ApproveResult {
  confirm_task_created: boolean;
}

export interface RunLogEntry {
  ts: number;
  level: string;
  message: string;
}

export interface RuntimeEvent {
  type: string;
  run_id?: string;
  session_id?: string;
  mirror?: RunMirror;
  stage?: string;
  entry?: RunLogEntry;
  session?: ChatSession;
  issue_key?: string;
}

export type ChatSessionStatus = "active" | "ready" | "bootstrapped" | "abandoned";

export interface ChatMessage {
  id: string;
  session_id: string;
  role: string;
  content: string;
  ts: number;
}

export interface ChatSession {
  id: string;
  workspace_id: string;
  runtime_id: string;
  product_id?: string | null;
  status: ChatSessionStatus;
  draft_title?: string | null;
  draft_pipeline?: string | null;
  draft_description?: string | null;
  linked_issue_key?: string | null;
  linked_run_id?: string | null;
  updated_at: number;
  messages?: ChatMessage[];
}

export interface ChatTurnResult {
  accepted: boolean;
  state: "queued" | "rejected";
  reason?: string | null;
  message?: ChatMessage | null;
}

export interface BootstrapResult {
  accepted: boolean;
  state: "queued" | "rejected";
  reason?: string | null;
}

export interface MobileConfig {
  serverUrl: string;
  runtimeId: string;
  workspaceId: string;
}

export const DEFAULT_CONFIG: MobileConfig = {
  serverUrl: "http://127.0.0.1:8787",
  runtimeId: "default",
  workspaceId: "/Users/narwal/Workspace/github/popsicle",
};

export const DANGEROUS_STAGES = new Set(["cutover", "living-docs"]);
