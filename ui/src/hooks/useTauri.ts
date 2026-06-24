import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

export interface WorkspaceInfo {
  root: string;
  project_name: string;
  storage_backend: string;
  binary_match: boolean;
  executable_path: string;
}

export interface ProjectInfo {
  name: string;
  path: string;
  last_opened_at: number | null;
  is_default: boolean;
  is_valid: boolean;
}

export interface ProjectsList {
  projects: ProjectInfo[];
  default_path: string | null;
  global_config_path: string;
}

export type ApprovalMode = "manual" | "auto" | "delegate-dangerous";

export interface ProjectConfigDto {
  language: string;
  products_dir: string;
  default_product: string;
  product_options: string[];
  workflow_profile: string;
  sync_agents_md: boolean;
  inject_on_run: boolean;
  approval_mode: ApprovalMode;
  track_workspace: boolean;
  config_path: string;
}

export interface TaskOption {
  task_id: string;
  title: string;
  journey_stage: string;
}

export interface CreateIssueFormOptions {
  default_product: string;
  product_options: string[];
  pipeline_options: string[];
  default_pipeline_by_type: Record<string, string>;
  workflow_profile: string;
  task_options: TaskOption[];
}

export interface SaveProjectConfigInput {
  language: string;
  products_dir: string;
  default_product: string;
  workflow_profile: string;
  sync_agents_md: boolean;
  inject_on_run: boolean;
  approval_mode: ApprovalMode;
  track_workspace: boolean;
}

const DANGEROUS_STAGES = new Set(["cutover", "living-docs"]);

/** Mirrors `project_config::stage_needs_explicit_confirm` for UI gating. */
export function stageNeedsExplicitConfirm(
  mode: ApprovalMode,
  stageName: string,
  requiresApproval: boolean
): boolean {
  if (!requiresApproval) return false;
  if (mode === "auto") return false;
  if (mode === "delegate-dangerous") return DANGEROUS_STAGES.has(stageName);
  return true;
}

export interface IssueTaskLink {
  role: string;
  task_id: string | null;
  proposed_title: string | null;
  journey_stage: string | null;
  source: string;
}

export interface IssueInfo {
  key: string;
  title: string;
  issue_type: string;
  priority: string;
  status: string;
  product_id: string;
  pipeline: string | null;
  description: string;
  /** @deprecated use task_links */
  epic_task_id: string | null;
  task_links: IssueTaskLink[];
  active_run_id: string | null;
  run_ids: string[];
}

export interface ProductHealthReport {
  product: string;
  task_count: number;
  intent_block_count: number;
  journey_stages: string[];
  unverified_tasks: number;
  broken_refs: number;
  orphan_intents: number;
  has_product_md: boolean;
  has_architecture_md: boolean;
  health: string;
  hints: string[];
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

export interface IntentDiagramView {
  id: string;
  label: string;
  description: string;
  mermaid: string;
}

export interface IntentGraph {
  blocks: IntentBlockNode[];
  mermaid: string | null;
  diagrams: IntentDiagramView[];
  source: string;
  parse_error: string | null;
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

export async function getActiveProject(): Promise<ProjectInfo | null> {
  return invoke("get_active_project");
}

export async function resolveStartupProject(
  cliProject?: string
): Promise<string | null> {
  return invoke("resolve_startup_project", { cliProject: cliProject ?? null });
}

export async function workspaceNeedsBootstrap(path: string): Promise<boolean> {
  return invoke("workspace_needs_bootstrap_cmd", { path });
}

export async function openProject(
  path: string,
  confirmBootstrap = false
): Promise<ProjectInfo> {
  return invoke("open_project_cmd", { path, confirmBootstrap });
}

export async function listRegisteredProjects(): Promise<ProjectsList> {
  return invoke("list_registered_projects");
}

export async function removeRegisteredProject(name: string): Promise<void> {
  return invoke("remove_registered_project", { name });
}

export async function pickProjectDirectory(): Promise<string | null> {
  return invoke("pick_project_directory");
}

export function useProjectSession() {
  const [project, setProject] = useState<ProjectInfo | null>(null);
  const [bootstrapPromptPath, setBootstrapPromptPath] = useState<string | null>(
    null
  );
  const bootstrapResolverRef = useRef<
    ((info: ProjectInfo | null) => void) | null
  >(null);

  const syncProject = useCallback(async (info: ProjectInfo) => {
    setProject(info);
  }, []);

  const finishBootstrapPrompt = useCallback((confirmed: boolean) => {
    const path = bootstrapPromptPath;
    setBootstrapPromptPath(null);
    const resolve = bootstrapResolverRef.current;
    bootstrapResolverRef.current = null;
    if (!path || !resolve) return;
    if (!confirmed) {
      resolve(null);
      return;
    }
    openProject(path, true)
      .then((info) => {
        setProject(info);
        resolve(info);
      })
      .catch(() => resolve(null));
  }, [bootstrapPromptPath]);

  const openProjectDir = useCallback(async (path: string) => {
    const trimmed = path.trim();
    if (!trimmed) return null;
    const needs = await workspaceNeedsBootstrap(trimmed);
    if (needs) {
      return new Promise<ProjectInfo | null>((resolve) => {
        bootstrapResolverRef.current = resolve;
        setBootstrapPromptPath(trimmed);
      });
    }
    const info = await openProject(trimmed, false);
    setProject(info);
    return info;
  }, []);

  const closeProject = useCallback(() => {
    setProject(null);
  }, []);

  return {
    project,
    dir: project?.path ?? null,
    projectName: project?.name ?? null,
    syncProject,
    openProjectDir,
    closeProject,
    bootstrapPromptPath,
    finishBootstrapPrompt,
  };
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
  productId: string;
  pipeline?: string;
  priority?: string;
  description?: string;
  /** @deprecated use linkedTaskIds */
  epicTaskId?: string;
  linkedTaskIds?: string[];
  proposedTasks?: [string, string | null][];
}): Promise<IssueInfo> {
  return invoke("create_issue", {
    issueType: params.issueType,
    title: params.title,
    productId: params.productId,
    pipeline: params.pipeline,
    priority: params.priority,
    description: params.description,
    epicTaskId: params.epicTaskId ?? null,
    linkedTaskIds: params.linkedTaskIds ?? null,
    proposedTasks: params.proposedTasks ?? null,
  });
}

export async function getProductHealth(
  product: string
): Promise<ProductHealthReport> {
  return invoke("get_product_health", { product });
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

export interface ProposedTaskHint {
  title: string;
  journey_stage: string | null;
  source: string;
}

export interface IssueGuidance {
  product: string | null;
  pipeline_stage: string | null;
  hint: string;
  linked_tasks: TaskNode[];
  proposed_tasks: ProposedTaskHint[];
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

export async function getCreateIssueFormOptions(): Promise<CreateIssueFormOptions> {
  return invoke("get_create_issue_form_options");
}

export async function getProjectConfig(): Promise<ProjectConfigDto> {
  return invoke("get_project_config");
}

export async function saveProjectConfig(
  input: SaveProjectConfigInput
): Promise<ProjectConfigDto> {
  return invoke("save_project_config_cmd", { input });
}

export interface ProjectContextDto {
  path: string;
  content: string;
  exists: boolean;
}

export async function getProjectContextMd(): Promise<ProjectContextDto> {
  return invoke("get_project_context_md");
}

export async function saveProjectContextMd(
  content: string
): Promise<ProjectContextDto> {
  return invoke("save_project_context_md", { input: { content } });
}

export interface StageCatalogEntry {
  name: string;
  skills: string[];
  description: string;
  depends_on: string[];
  requires_approval: boolean;
}

export interface PipelineCatalogEntry {
  name: string;
  description: string;
  scale: string;
  keywords: string[];
  category: string;
  stage_count: number;
  approval_count: number;
  stages: StageCatalogEntry[];
  recommended: boolean;
}

export interface SkillArtifactEntry {
  artifact_type: string;
  description: string;
}

export interface SkillStateEntry {
  name: string;
  requires_approval: boolean;
  is_initial: boolean;
  is_final: boolean;
}

export interface SkillCatalogEntry {
  name: string;
  version: string;
  description: string;
  artifacts: SkillArtifactEntry[];
  workflow_states: SkillStateEntry[];
  used_in_pipelines: string[];
  standalone: boolean;
  guide_path: string;
}

export interface WorkflowCatalog {
  workflow_profile: string;
  workflow_profile_label: string;
  default_pipeline_by_type: Record<string, string>;
  pipelines: PipelineCatalogEntry[];
  skills: SkillCatalogEntry[];
}

export async function getWorkflowCatalog(): Promise<WorkflowCatalog> {
  return invoke("get_workflow_catalog");
}

export type IssueGroupBy = "none" | "product" | "pipeline" | "epic";
