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
pub struct CreateIssueFormOptions {
    pub default_spec: String,
    pub spec_options: Vec<String>,
    pub pipeline_options: Vec<String>,
    pub default_pipeline_by_type: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfigDto {
    pub language: String,
    pub products_dir: String,
    pub default_spec: String,
    pub sync_agents_md: bool,
    pub inject_on_run: bool,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectsList {
    pub projects: Vec<ProjectInfo>,
    pub default_path: Option<String>,
    pub global_config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueInfo {
    pub key: String,
    pub title: String,
    pub issue_type: String,
    pub priority: String,
    pub status: String,
    pub spec_id: String,
    pub pipeline: Option<String>,
    pub description: String,
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

pub use crate::workspace_readers::{
    IntentBlockDetail, IntentFileFull, IntentRef, IssueGuidance, TaskFull,
};
