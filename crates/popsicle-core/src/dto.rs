use serde::Serialize;

#[derive(Serialize)]
pub struct ProjectInfo {
    pub path: String,
    pub initialized: bool,
}

// ── Namespace entity ──

#[derive(Serialize)]
pub struct NamespaceEntityInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub status: String,
    pub tags: Vec<String>,
    pub spec_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct NamespaceEntityDetail {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub status: String,
    pub tags: Vec<String>,
    pub specs: Vec<SpecInfo>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub artifact_types: Vec<String>,
    pub doc_lifecycle: String,
    pub workflow_initial: String,
    pub inputs: Vec<SkillInputInfo>,
    pub workflow_states: Vec<WorkflowStateInfo>,
}

#[derive(Serialize)]
pub struct SkillInputInfo {
    pub from_skill: String,
    pub artifact_type: String,
    pub required: bool,
    pub relevance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct WorkflowStateInfo {
    pub name: String,
    pub is_final: bool,
    pub transitions: Vec<TransitionInfo>,
}

#[derive(Serialize)]
pub struct TransitionInfo {
    pub to: String,
    pub action: String,
    pub requires_approval: bool,
}

#[derive(Serialize)]
pub struct PipelineInfo {
    pub name: String,
    pub description: String,
    pub stages: Vec<StageInfo>,
}

#[derive(Serialize)]
pub struct StageInfo {
    pub name: String,
    pub skills: Vec<String>,
    pub description: String,
    pub depends_on: Vec<String>,
}

#[derive(Serialize)]
pub struct PipelineRunInfo {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub spec_id: String,
    pub issue_id: String,
    pub run_type: String,
}

#[derive(Serialize)]
pub struct PipelineStatusFull {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub stages: Vec<StageStatusInfo>,
}

#[derive(Serialize)]
pub struct StageStatusInfo {
    pub name: String,
    pub state: String,
    pub skills: Vec<String>,
    pub description: String,
    pub depends_on: Vec<String>,
    pub documents: Vec<DocInfo>,
    pub requires_approval: bool,
}

#[derive(Serialize)]
pub struct StageCompleteResult {
    pub stage: String,
    pub state: String,
    pub run_id: String,
    pub all_done: bool,
    pub auto_released: bool,
    pub unblocked: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct DocInfo {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub skill_name: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub checklist_checked: u32,
    pub checklist_total: u32,
}

#[derive(Serialize)]
pub struct DocFull {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub skill_name: String,
    pub pipeline_run_id: String,
    pub tags: Vec<String>,
    pub body: String,
    pub file_path: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub summary: String,
    pub doc_tags: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchDocResult {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub skill_name: String,
    pub pipeline_run_id: String,
    pub file_path: String,
    pub summary: String,
    pub doc_tags: Vec<String>,
    pub bm25_score: f64,
}

#[derive(Serialize)]
pub struct NextStepInfo {
    pub stage: String,
    pub skill: String,
    pub action: String,
    pub description: String,
    pub cli_command: String,
    pub prompt: Option<String>,
    pub blocked_by: Vec<String>,
    pub requires_approval: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_command: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
}

#[derive(Serialize)]
pub struct VerifyResult {
    pub run_id: String,
    pub verified: bool,
    pub issues: Vec<String>,
}

#[derive(Serialize)]
pub struct PromptInfo {
    pub skill: String,
    pub state: String,
    pub prompt: Option<String>,
    pub available_states: Vec<String>,
}

#[derive(Serialize)]
pub struct GitStatusInfo {
    pub branch: String,
    pub head: String,
    pub uncommitted_changes: bool,
    pub pipeline_run_id: Option<String>,
    pub total_commits: usize,
    pub pending_review: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Serialize)]
pub struct CommitLinkInfo {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub doc_id: Option<String>,
    pub pipeline_run_id: String,
    pub stage: Option<String>,
    pub skill: Option<String>,
    pub review_status: String,
    pub review_summary: Option<String>,
    pub linked_at: String,
}

#[derive(Serialize)]
pub struct IssueInfo {
    pub id: String,
    pub key: String,
    pub title: String,
    pub issue_type: String,
    pub priority: String,
    pub status: String,
    pub spec_id: String,
    pub pipeline: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct IssueFull {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub issue_type: String,
    pub priority: String,
    pub status: String,
    pub spec_id: String,
    pub pipeline: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ── Issue progress aggregation ──

#[derive(Serialize)]
pub struct IssueProgress {
    pub issue_key: String,
    pub spec_id: String,
    pub pipeline_runs: Vec<PipelineRunInfo>,
    pub stages_total: u32,
    pub stages_completed: u32,
    pub docs_total: u32,
    pub docs_final: u32,
    pub checklist_checked: u32,
    pub checklist_total: u32,
    pub current_stage: Option<String>,
    pub stage_summaries: Vec<StageSummary>,
}

#[derive(Clone, Serialize)]
pub struct StageSummary {
    pub name: String,
    pub state: String,
    pub docs: Vec<DocInfo>,
}

// ── Activity timeline ──

#[derive(Serialize)]
pub struct ActivityEvent {
    pub timestamp: String,
    pub event_type: String,
    pub title: String,
    pub detail: Option<String>,
    pub doc_id: Option<String>,
    pub stage: Option<String>,
}

// ── Project context ──

#[derive(Serialize)]
pub struct ProjectContextInfo {
    pub available: bool,
    pub content: Option<String>,
    pub path: Option<String>,
}

// ── WorkItem ──

#[derive(Serialize)]
pub struct WorkItemInfo {
    pub id: String,
    pub key: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub labels: Vec<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct WorkItemFull {
    pub id: String,
    pub key: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub labels: Vec<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub source_doc_id: Option<String>,
    pub fields: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

// ── Spec ──

#[derive(Serialize)]
pub struct SpecInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub namespace_id: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub run_count: u32,
    pub doc_count: u32,
}

#[derive(Serialize)]
pub struct SpecDetailInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub namespace_id: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub runs: Vec<PipelineRunInfo>,
    pub documents: Vec<DocInfo>,
    pub issues: Vec<IssueInfo>,
}

// ── Memory ──

#[derive(Serialize)]
pub struct MemoryInfo {
    pub id: u32,
    pub memory_type: String,
    pub summary: String,
    pub created: String,
    pub layer: String,
    pub refs: u32,
    pub tags: Vec<String>,
    pub files: Vec<String>,
    pub run: Option<String>,
    pub stale: bool,
    pub detail: String,
}

#[derive(Serialize)]
pub struct MemoryStatsInfo {
    pub line_count: usize,
    pub max_lines: usize,
    pub total: usize,
    pub long_term: usize,
    pub short_term: usize,
    pub bugs: usize,
    pub decisions: usize,
    pub patterns: usize,
    pub gotchas: usize,
    pub stale: usize,
}
