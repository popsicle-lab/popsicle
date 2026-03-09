import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

export interface SkillInfo {
  name: string;
  description: string;
  version: string;
  artifact_types: string[];
  workflow_initial: string;
  inputs: { from_skill: string; artifact_type: string; required: boolean }[];
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
}

export interface DocFull extends DocInfo {
  pipeline_run_id: string;
  tags: string[];
  body: string;
  file_path: string;
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
}

export function useRefresh(callback: () => void) {
  useEffect(() => {
    const unlisten = listen("popsicle://refresh", callback);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [callback]);
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
