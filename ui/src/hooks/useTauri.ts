import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

export interface WorkspaceInfo {
  root: string;
  storage_backend: string;
  binary_match: boolean;
  executable_path: string;
}

export interface IssueInfo {
  key: string;
  title: string;
  issue_type: string;
  priority: string;
  status: string;
  spec_id: string;
  pipeline: string | null;
  description: string;
  active_run_id: string | null;
  run_ids: string[];
}

export interface DocInfo {
  id: string;
  doc_type: string;
  title: string;
  status: string;
  file_path: string;
}

export interface DocFull extends DocInfo {
  body: string;
  check_passed: boolean;
}

export interface StageStatusInfo {
  name: string;
  state: string;
  skills: string[];
  description: string;
  depends_on: string[];
  documents: DocInfo[];
  requires_approval: boolean;
}

export interface PipelineStatusFull {
  id: string;
  pipeline_name: string;
  issue_key: string;
  run_status: string;
  current_stage: string;
  stages: StageStatusInfo[];
}

export interface StageCompleteResult {
  current_stage: string;
  downstream_ready: boolean;
}

export interface TaskNode {
  task_id: string;
  title: string;
  journey_stage: string;
  product: string;
  related_next_tasks: string[];
  related_intents: string[];
  file_path: string;
}

export interface TaskGraph {
  nodes: TaskNode[];
}

export interface IntentBlockNode {
  name: string;
  kind: string;
  task_id: string | null;
  product: string;
  file: string;
}

export interface IntentGraph {
  blocks: IntentBlockNode[];
  mermaid: string | null;
  source: string;
}

export function useRefresh(callback: () => void) {
  useEffect(() => {
    const unlisten = listen("popsicle://refresh", callback);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [callback]);
}

export async function getInitialDir(): Promise<string> {
  return invoke("get_initial_dir");
}

export function useProjectDir() {
  const [dir, setDir] = useState<string | null>(null);

  const setProjectDir = useCallback(async (path: string) => {
    await invoke("set_project_dir", { path });
    setDir(path);
  }, []);

  return { dir, setProjectDir };
}

export async function getWorkspaceInfo(): Promise<WorkspaceInfo> {
  return invoke("get_workspace_info");
}

export async function listIssues(): Promise<IssueInfo[]> {
  return invoke("list_issues");
}

export async function getIssue(key: string): Promise<IssueInfo> {
  return invoke("get_issue", { key });
}

export async function createIssue(params: {
  issueType: string;
  title: string;
  specId: string;
  pipeline?: string;
  priority?: string;
  description?: string;
}): Promise<IssueInfo> {
  return invoke("create_issue", {
    issueType: params.issueType,
    title: params.title,
    specId: params.specId,
    pipeline: params.pipeline,
    priority: params.priority,
    description: params.description,
  });
}

export async function startIssue(key: string): Promise<string> {
  return invoke("start_issue", { key });
}

export async function listDocsForRun(runId: string): Promise<DocInfo[]> {
  return invoke("list_docs_for_run", { runId });
}

export async function readDoc(docId: string): Promise<DocFull> {
  return invoke("read_doc", { docId });
}

export async function getPipelineStatus(
  runId: string
): Promise<PipelineStatusFull> {
  return invoke("get_pipeline_status", { runId });
}

export async function completeStage(
  runId: string,
  stageName: string,
  confirm: boolean
): Promise<StageCompleteResult> {
  return invoke("complete_stage", { runId, stageName, confirm });
}

export async function scanTaskGraph(): Promise<TaskGraph> {
  return invoke("scan_task_graph");
}

export async function listProductNames(): Promise<string[]> {
  return invoke("list_product_names");
}

export async function scanIntentGraph(product: string): Promise<IntentGraph> {
  return invoke("scan_intent_graph", { product });
}

export async function intentGraphMermaid(product: string): Promise<string> {
  return invoke("intent_graph_mermaid", { product });
}

export interface TaskFull {
  task_id: string;
  title: string;
  journey_stage: string;
  product: string;
  file_path: string;
  frontmatter: Record<string, unknown>;
  body: string;
}

export interface IntentBlockDetail {
  name: string;
  kind: string;
  task_id: string | null;
  start_line: number;
  end_line: number;
  snippet: string;
}

export interface IntentFileFull {
  product: string;
  file: string;
  content: string;
  blocks: IntentBlockDetail[];
}

export interface IntentRef {
  reference: string;
  file: string;
  block: string;
  product: string;
}

export interface IssueGuidance {
  product: string | null;
  pipeline_stage: string | null;
  hint: string;
  recommended_tasks: TaskNode[];
  related_intents: IntentRef[];
}

export async function scanProductTaskGraph(
  product: string
): Promise<TaskGraph> {
  return invoke("scan_product_task_graph", { product });
}

export async function readTaskContent(
  taskId: string,
  product?: string
): Promise<TaskFull> {
  return invoke("read_task_content", { taskId, product });
}

export async function readIntentFile(
  product: string,
  file: string
): Promise<IntentFileFull> {
  return invoke("read_intent_file_cmd", { product, file });
}

export async function resolveIntentRef(
  reference: string,
  product?: string
): Promise<IntentBlockDetail> {
  return invoke("resolve_intent_ref_cmd", { reference, product });
}

export async function getIssueGuidance(
  issueKey: string
): Promise<IssueGuidance> {
  return invoke("get_issue_guidance", { issueKey });
}
