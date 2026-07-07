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
  mirror?: RunMirror;
  stage?: string;
  entry?: RunLogEntry;
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
