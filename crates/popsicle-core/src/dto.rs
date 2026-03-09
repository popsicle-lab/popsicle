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
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
