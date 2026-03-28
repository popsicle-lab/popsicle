use serde::Serialize;

#[derive(Serialize)]
pub struct DiscussionInfo {
    pub id: String,
    pub document_id: Option<String>,
    pub skill: String,
    pub pipeline_run_id: String,
    pub topic: String,
    pub status: String,
    pub user_confidence: Option<i32>,
    pub message_count: usize,
    pub created_at: String,
    pub concluded_at: Option<String>,
}

#[derive(Serialize)]
pub struct DiscussionFull {
    pub id: String,
    pub document_id: Option<String>,
    pub skill: String,
    pub pipeline_run_id: String,
    pub topic: String,
    pub status: String,
    pub user_confidence: Option<i32>,
    pub roles: Vec<DiscussionRoleInfo>,
    pub messages: Vec<DiscussionMessageInfo>,
    pub created_at: String,
    pub concluded_at: Option<String>,
}

#[derive(Serialize)]
pub struct DiscussionRoleInfo {
    pub role_id: String,
    pub role_name: String,
    pub perspective: Option<String>,
    pub source: String,
}

#[derive(Serialize)]
pub struct DiscussionMessageInfo {
    pub id: String,
    pub phase: String,
    pub role_id: String,
    pub role_name: String,
    pub content: String,
    pub message_type: String,
    pub reply_to: Option<String>,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub path: String,
    pub initialized: bool,
}

#[derive(Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub artifact_types: Vec<String>,
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
    pub topic_id: String,
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
    pub pipeline: Option<String>,
    pub pipeline_run_id: Option<String>,
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
    pub pipeline: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ── Issue progress aggregation ──

#[derive(Serialize)]
pub struct IssueProgress {
    pub issue_key: String,
    pub pipeline_run_id: Option<String>,
    pub pipeline_name: Option<String>,
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

// ── Bug ──

#[derive(Serialize)]
pub struct BugInfo {
    pub id: String,
    pub key: String,
    pub title: String,
    pub severity: String,
    pub priority: String,
    pub status: String,
    pub source: String,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct BugFull {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub priority: String,
    pub status: String,
    pub steps_to_reproduce: Vec<String>,
    pub expected_behavior: String,
    pub actual_behavior: String,
    pub environment: Option<String>,
    pub stack_trace: Option<String>,
    pub source: String,
    pub related_test_case_id: Option<String>,
    pub related_commit_sha: Option<String>,
    pub fix_commit_sha: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ── TestCase ──

#[derive(Serialize)]
pub struct TestCaseInfo {
    pub id: String,
    pub key: String,
    pub title: String,
    pub test_type: String,
    pub priority_level: String,
    pub status: String,
    pub source_doc_id: Option<String>,
    pub user_story_id: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct TestCaseFull {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub test_type: String,
    pub priority_level: String,
    pub status: String,
    pub preconditions: Vec<String>,
    pub steps: Vec<String>,
    pub expected_result: String,
    pub source_doc_id: Option<String>,
    pub user_story_id: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct TestRunInfo {
    pub id: String,
    pub test_case_id: String,
    pub passed: bool,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub commit_sha: Option<String>,
    pub run_at: String,
}

#[derive(Serialize)]
pub struct TestCoverageSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub no_runs: usize,
    pub pass_rate: f64,
}

// ── UserStory ──

#[derive(Serialize)]
pub struct UserStoryInfo {
    pub id: String,
    pub key: String,
    pub title: String,
    pub persona: String,
    pub priority: String,
    pub status: String,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub ac_count: usize,
    pub ac_verified: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct UserStoryFull {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub persona: String,
    pub goal: String,
    pub benefit: String,
    pub priority: String,
    pub status: String,
    pub source_doc_id: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub acceptance_criteria: Vec<AcceptanceCriterionInfo>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct AcceptanceCriterionInfo {
    pub id: String,
    pub description: String,
    pub verified: bool,
    pub test_case_ids: Vec<String>,
}

// ── Topic ──

#[derive(Serialize)]
pub struct TopicInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub run_count: u32,
    pub doc_count: u32,
}

#[derive(Serialize)]
pub struct TopicDetailInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub runs: Vec<PipelineRunInfo>,
    pub documents: Vec<DocInfo>,
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
