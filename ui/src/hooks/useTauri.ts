import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

export interface SkillInfo {
  name: string;
  description: string;
  version: string;
  artifact_types: string[];
  workflow_initial: string;
  inputs: {
    from_skill: string;
    artifact_type: string;
    required: boolean;
    relevance: string;
    sections?: string[];
  }[];
  workflow_states: {
    name: string;
    is_final: boolean;
    transitions: { to: string; action: string }[];
  }[];
}

export interface PipelineInfo {
  name: string;
  description: string;
  stages: StageInfo[];
}

export interface StageInfo {
  name: string;
  skills: string[];
  description: string;
  depends_on: string[];
}

export interface PipelineRunInfo {
  id: string;
  pipeline_name: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface PipelineStatusFull {
  id: string;
  pipeline_name: string;
  title: string;
  stages: StageStatusInfo[];
}

export interface StageStatusInfo {
  name: string;
  state: string;
  skills: string[];
  description: string;
  depends_on: string[];
  documents: DocInfo[];
}

export interface DocInfo {
  id: string;
  doc_type: string;
  title: string;
  status: string;
  skill_name: string;
  created_at: string | null;
  updated_at: string | null;
  checklist_checked: number;
  checklist_total: number;
}

export interface DocFull extends DocInfo {
  pipeline_run_id: string;
  tags: string[];
  body: string;
  file_path: string;
  summary: string;
  doc_tags: string[];
}

export interface SearchDocResult {
  id: string;
  doc_type: string;
  title: string;
  status: string;
  skill_name: string;
  pipeline_run_id: string;
  file_path: string;
  summary: string;
  doc_tags: string[];
  bm25_score: number;
}

export interface NextStepInfo {
  stage: string;
  skill: string;
  action: string;
  description: string;
  cli_command: string;
  prompt: string | null;
  blocked_by: string[];
  requires_approval: boolean;
  context_command?: string;
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

export async function listSkills(): Promise<SkillInfo[]> {
  return invoke("list_skills");
}

export async function listPipelines(): Promise<PipelineInfo[]> {
  return invoke("list_pipelines");
}

export async function listPipelineRuns(): Promise<PipelineRunInfo[]> {
  return invoke("list_pipeline_runs");
}

export async function getPipelineStatus(
  runId: string
): Promise<PipelineStatusFull> {
  return invoke("get_pipeline_status", { runId });
}

export async function listDocuments(filters?: {
  skill?: string;
  status?: string;
  runId?: string;
}): Promise<DocInfo[]> {
  return invoke("list_documents", filters || {});
}

export async function getDocument(docId: string): Promise<DocFull> {
  return invoke("get_document", { docId });
}

export async function searchDocuments(filters: {
  query: string;
  status?: string;
  skill?: string;
  excludeRun?: string;
  limit?: number;
}): Promise<SearchDocResult[]> {
  return invoke("search_documents", filters);
}

export async function getNextSteps(runId: string): Promise<NextStepInfo[]> {
  return invoke("get_next_steps", { runId });
}

export interface VerifyResult {
  run_id: string;
  verified: boolean;
  issues: string[];
}

export async function verifyPipelineRun(runId: string): Promise<VerifyResult> {
  return invoke("verify_pipeline_run", { runId });
}

export async function getProjectConfig(): Promise<Record<string, unknown>> {
  return invoke("get_project_config");
}

export interface GitStatusInfo {
  branch: string;
  head: string;
  uncommitted_changes: boolean;
  pipeline_run_id: string | null;
  total_commits: number;
  pending_review: number;
  passed: number;
  failed: number;
}

export interface CommitLinkInfo {
  sha: string;
  short_sha: string;
  message: string;
  author: string;
  timestamp: string;
  doc_id: string | null;
  pipeline_run_id: string;
  stage: string | null;
  skill: string | null;
  review_status: string;
  review_summary: string | null;
  linked_at: string;
}

export async function getGitStatus(): Promise<GitStatusInfo> {
  return invoke("get_git_status");
}

export async function getCommitLinks(filters?: {
  runId?: string;
  docId?: string;
}): Promise<CommitLinkInfo[]> {
  return invoke("get_commit_links", filters || {});
}

export interface DiscussionInfo {
  id: string;
  document_id: string | null;
  skill: string;
  pipeline_run_id: string;
  topic: string;
  status: string;
  user_confidence: number | null;
  message_count: number;
  created_at: string;
  concluded_at: string | null;
}

export interface DiscussionFull {
  id: string;
  document_id: string | null;
  skill: string;
  pipeline_run_id: string;
  topic: string;
  status: string;
  user_confidence: number | null;
  roles: DiscussionRoleInfo[];
  messages: DiscussionMessageInfo[];
  created_at: string;
  concluded_at: string | null;
}

export interface DiscussionRoleInfo {
  role_id: string;
  role_name: string;
  perspective: string | null;
  source: string;
}

export interface DiscussionMessageInfo {
  id: string;
  phase: string;
  role_id: string;
  role_name: string;
  content: string;
  message_type: string;
  reply_to: string | null;
  timestamp: string;
}

// ── Issue types ──

export interface IssueInfo {
  id: string;
  key: string;
  title: string;
  issue_type: string;
  priority: string;
  status: string;
  pipeline: string | null;
  pipeline_run_id: string | null;
  labels: string[];
  created_at: string;
  updated_at: string;
}

export interface IssueFull extends IssueInfo {
  description: string;
}

export async function listIssues(filters?: {
  issueType?: string;
  status?: string;
  label?: string;
}): Promise<IssueInfo[]> {
  return invoke("list_issues", filters || {});
}

export async function getIssue(key: string): Promise<IssueFull> {
  return invoke("get_issue", { key });
}

export async function createIssue(params: {
  issueType: string;
  title: string;
  description?: string;
  priority?: string;
  pipeline?: string;
  labels?: string[];
}): Promise<IssueInfo> {
  return invoke("create_issue", params);
}

export async function startIssue(key: string): Promise<IssueInfo> {
  return invoke("start_issue", { key });
}

export async function updateIssue(params: {
  key: string;
  status?: string;
  priority?: string;
  title?: string;
  labels?: string[];
}): Promise<IssueInfo> {
  return invoke("update_issue", params);
}

// ── Issue progress & activity ──

export interface IssueProgress {
  issue_key: string;
  pipeline_run_id: string | null;
  pipeline_name: string | null;
  stages_total: number;
  stages_completed: number;
  docs_total: number;
  docs_final: number;
  checklist_checked: number;
  checklist_total: number;
  current_stage: string | null;
  stage_summaries: StageSummary[];
}

export interface StageSummary {
  name: string;
  state: string;
  docs: DocInfo[];
}

export interface ActivityEvent {
  timestamp: string;
  event_type: string;
  title: string;
  detail: string | null;
  doc_id: string | null;
  stage: string | null;
}

export async function getIssueProgress(key: string): Promise<IssueProgress> {
  return invoke("get_issue_progress", { key });
}

export async function getActivity(runId: string): Promise<ActivityEvent[]> {
  return invoke("get_activity", { runId });
}

export async function findIssueByRun(runId: string): Promise<IssueInfo | null> {
  return invoke("find_issue_by_run", { runId });
}

// ── Project context ──

export interface ProjectContextInfo {
  available: boolean;
  content: string | null;
  path: string | null;
}

export async function getProjectContext(): Promise<ProjectContextInfo> {
  return invoke("get_project_context");
}

// ── Bug types ──

export interface BugInfo {
  id: string;
  key: string;
  title: string;
  severity: string;
  priority: string;
  status: string;
  source: string;
  issue_id: string | null;
  pipeline_run_id: string | null;
  labels: string[];
  created_at: string;
  updated_at: string;
}

export interface BugFull {
  id: string;
  key: string;
  title: string;
  description: string;
  severity: string;
  priority: string;
  status: string;
  steps_to_reproduce: string[];
  expected_behavior: string;
  actual_behavior: string;
  environment: string | null;
  stack_trace: string | null;
  source: string;
  related_test_case_id: string | null;
  related_commit_sha: string | null;
  fix_commit_sha: string | null;
  issue_id: string | null;
  pipeline_run_id: string | null;
  labels: string[];
  created_at: string;
  updated_at: string;
}

export async function listBugs(filters?: {
  severity?: string;
  status?: string;
  issueId?: string;
  runId?: string;
}): Promise<BugInfo[]> {
  return invoke("list_bugs", filters || {});
}

export async function getBug(key: string): Promise<BugFull> {
  return invoke("get_bug", { key });
}

// ── TestCase types ──

export interface TestCaseInfo {
  id: string;
  key: string;
  title: string;
  test_type: string;
  priority_level: string;
  status: string;
  source_doc_id: string | null;
  user_story_id: string | null;
  issue_id: string | null;
  pipeline_run_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface TestCaseFull {
  id: string;
  key: string;
  title: string;
  description: string;
  test_type: string;
  priority_level: string;
  status: string;
  preconditions: string[];
  steps: string[];
  expected_result: string;
  source_doc_id: string | null;
  user_story_id: string | null;
  issue_id: string | null;
  pipeline_run_id: string | null;
  labels: string[];
  created_at: string;
  updated_at: string;
}

export interface TestCoverageSummary {
  total: number;
  passed: number;
  failed: number;
  no_runs: number;
  pass_rate: number;
}

export async function listTestCases(filters?: {
  testType?: string;
  priority?: string;
  status?: string;
  runId?: string;
}): Promise<TestCaseInfo[]> {
  return invoke("list_test_cases", filters || {});
}

export async function getTestCase(key: string): Promise<TestCaseFull> {
  return invoke("get_test_case", { key });
}

export async function getTestCoverage(filters?: {
  runId?: string;
}): Promise<TestCoverageSummary> {
  return invoke("get_test_coverage", filters || {});
}

// ── UserStory types ──

export interface UserStoryInfo {
  id: string;
  key: string;
  title: string;
  persona: string;
  priority: string;
  status: string;
  issue_id: string | null;
  pipeline_run_id: string | null;
  ac_count: number;
  ac_verified: number;
  created_at: string;
  updated_at: string;
}

export interface UserStoryFull {
  id: string;
  key: string;
  title: string;
  description: string;
  persona: string;
  goal: string;
  benefit: string;
  priority: string;
  status: string;
  source_doc_id: string | null;
  issue_id: string | null;
  pipeline_run_id: string | null;
  acceptance_criteria: AcceptanceCriterionInfo[];
  created_at: string;
  updated_at: string;
}

export interface AcceptanceCriterionInfo {
  id: string;
  description: string;
  verified: boolean;
  test_case_ids: string[];
}

export async function listUserStories(filters?: {
  status?: string;
  issueId?: string;
  runId?: string;
}): Promise<UserStoryInfo[]> {
  return invoke("list_user_stories", filters || {});
}

export async function getUserStory(key: string): Promise<UserStoryFull> {
  return invoke("get_user_story", { key });
}

export async function listDiscussions(filters?: {
  runId?: string;
  skill?: string;
  status?: string;
}): Promise<DiscussionInfo[]> {
  return invoke("list_discussions", filters || {});
}

export async function getDiscussion(
  discussionId: string
): Promise<DiscussionFull> {
  return invoke("get_discussion", { discussionId });
}

// ── Memory types ──

export interface MemoryInfo {
  id: number;
  memory_type: string;
  summary: string;
  created: string;
  layer: string;
  refs: number;
  tags: string[];
  files: string[];
  run: string | null;
  stale: boolean;
  detail: string;
}

export interface MemoryStatsInfo {
  line_count: number;
  max_lines: number;
  total: number;
  long_term: number;
  short_term: number;
  bugs: number;
  decisions: number;
  patterns: number;
  gotchas: number;
  stale: number;
}

export async function listMemories(filters?: {
  layer?: string;
  memoryType?: string;
}): Promise<MemoryInfo[]> {
  return invoke("list_memories", filters || {});
}

export async function getMemoryStats(): Promise<MemoryStatsInfo> {
  return invoke("get_memory_stats");
}

export async function getMemory(id: number): Promise<MemoryInfo> {
  return invoke("get_memory", { id });
}
