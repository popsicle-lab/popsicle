use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceInfo {
    pub root: String,
    pub project_name: String,
    pub storage_backend: String,
    pub binary_match: bool,
    pub executable_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub last_opened_at: Option<u64>,
    pub is_default: bool,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskOptionDto {
    pub task_id: String,
    pub title: String,
    pub journey_stage: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateIssueFormOptions {
    pub default_product: String,
    pub product_options: Vec<String>,
    pub pipeline_options: Vec<String>,
    pub default_pipeline_by_type: std::collections::BTreeMap<String, String>,
    pub workflow_profile: String,
    pub task_options: Vec<TaskOptionDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfigDto {
    pub language: String,
    pub products_dir: String,
    pub default_product: String,
    pub product_options: Vec<String>,
    pub workflow_profile: String,
    pub sync_agents_md: bool,
    pub inject_on_run: bool,
    pub approval_mode: String,
    pub track_workspace: bool,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectContextDto {
    pub path: String,
    pub content: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectsList {
    pub projects: Vec<ProjectInfo>,
    pub default_path: Option<String>,
    pub global_config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueTaskLinkDto {
    pub role: String,
    pub task_id: Option<String>,
    pub proposed_title: Option<String>,
    pub journey_stage: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueInfo {
    pub key: String,
    pub title: String,
    pub issue_type: String,
    pub priority: String,
    pub status: String,
    pub product_id: String,
    pub pipeline: Option<String>,
    pub description: String,
    /// Deprecated: first linked task id; prefer `task_links`.
    pub epic_task_id: Option<String>,
    pub task_links: Vec<IssueTaskLinkDto>,
    pub active_run_id: Option<String>,
    pub run_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocInfo {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocFull {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub file_path: String,
    pub body: String,
    pub check_passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageStatusInfo {
    pub name: String,
    pub state: String,
    pub skills: Vec<String>,
    pub description: String,
    pub depends_on: Vec<String>,
    pub documents: Vec<DocInfo>,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineStatusFull {
    pub id: String,
    pub pipeline_name: String,
    pub issue_key: String,
    pub run_status: String,
    pub current_stage: String,
    pub stages: Vec<StageStatusInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageCompleteResult {
    pub current_stage: String,
    pub downstream_ready: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryRunDetailDto {
    pub report: telemetry::RunReport,
    pub spans: Vec<telemetry::WalLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentRuntimeConfigDto {
    pub server_url: String,
    pub runtime_id: String,
    pub config_path: String,
    pub cursor_agent_installed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CursorAgentStatusDto {
    pub installed: bool,
    pub binary_path: String,
    pub logged_in: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DaemonStatusDto {
    pub online: bool,
    pub poll_running: bool,
    pub pid: Option<u32>,
    pub workspace: String,
    pub detected_clis: Vec<String>,
    pub note: Option<String>,
    pub last_error: Option<String>,
    pub log_path: String,
    pub foreground_hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DaemonControlResultDto {
    pub poll_running: bool,
    pub pid: Option<u32>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentRuntimeServerStatusDto {
    pub server_url: String,
    pub runtime_id: String,
    pub server_ok: bool,
    pub storage: String,
    pub runtime_state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DispatchIssueResultDto {
    pub accepted: bool,
    pub state: String,
    pub reason: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoteRunMirrorDto {
    pub run_id: String,
    pub issue_key: Option<String>,
    pub pipeline: String,
    pub run_status: String,
    pub current_stage: String,
}

pub use crate::workspace_readers::{
    IntentBlockDetail, IntentFileFull, IntentRef, IssueGuidance, TaskFull,
};
