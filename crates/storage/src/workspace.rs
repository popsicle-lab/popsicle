//! Workspace persistence trait (ADR-009 Phase 1 TSV → Phase 2 SQLite).

use std::collections::BTreeMap;

use crate::DocumentRow;

/// Errors from workspace-backed stores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    NotFound(String),
    InvalidState(String),
    Io(String),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "not found: {id}"),
            Self::InvalidState(msg) => write!(f, "invalid state: {msg}"),
            Self::Io(msg) => write!(f, "io: {msg}"),
        }
    }
}

impl std::error::Error for WorkspaceError {}

/// Issue row persisted in workspace storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueRow {
    pub key: String,
    pub issue_type: String,
    pub priority: String,
    pub status: String,
    pub title: String,
    pub spec_id: String,
    pub pipeline: Option<String>,
    pub description: String,
}

/// Result of starting a pipeline run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunStartRow {
    pub run_id: String,
    pub spec_locked: bool,
}

/// Snapshot returned by `pipeline_status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStatusRow {
    pub run_id: String,
    pub pipeline_name: String,
    pub run_status: String,
    pub current_stage: String,
    pub current_stage_index: i64,
    pub total_stages: i64,
    pub stages: Vec<BTreeMap<String, String>>,
}

/// Result of completing a pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageCompleteRow {
    pub current_stage: String,
    pub downstream_ready: bool,
}

/// Document create result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCreateRow {
    pub doc_id: String,
    pub file_path: String,
    pub artifact_file_exists: bool,
}

/// Result of `doc check` (PDR-001: the checklist replacement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCheckRow {
    pub doc_id: String,
    pub file_path: String,
    pub file_exists: bool,
    pub frontmatter_complete: bool,
    pub body_filled: bool,
    pub placeholder_count: u32,
    pub checkboxes_total: u32,
    pub checkboxes_checked: u32,
    pub passed: bool,
}

/// Phase 1 (TSV) and Phase 2 (SQLite) backends implement this trait.
pub trait WorkspaceStore {
    fn init(&mut self) -> Result<(), WorkspaceError>;

    fn create_issue(
        &mut self,
        issue_type: &str,
        title: &str,
        spec_id: &str,
        pipeline: Option<&str>,
        priority: &str,
        description: &str,
    ) -> Result<IssueRow, WorkspaceError>;

    fn list_issues(&self) -> Result<Vec<IssueRow>, WorkspaceError>;

    fn get_issue(&self, key: &str) -> Result<IssueRow, WorkspaceError>;

    /// Close an issue. Fails while the issue still has an active (incomplete) run.
    fn close_issue(&mut self, key: &str) -> Result<IssueRow, WorkspaceError>;

    fn start_issue(
        &mut self,
        key: &str,
        spec_id: &str,
        pipeline: &str,
    ) -> Result<RunStartRow, WorkspaceError>;

    fn create_doc(
        &mut self,
        skill: &str,
        title: &str,
        run_id: &str,
    ) -> Result<DocCreateRow, WorkspaceError>;

    fn list_docs(&self, run_id: Option<&str>) -> Result<Vec<DocumentRow>, WorkspaceError>;

    fn get_doc(&self, doc_id: &str) -> Result<DocumentRow, WorkspaceError>;

    /// Validate a document artifact: frontmatter, filled body, placeholders, checkboxes.
    fn check_doc(&self, doc_id: &str) -> Result<DocCheckRow, WorkspaceError>;

    fn pipeline_status(&self, run_id: &str) -> Result<PipelineStatusRow, WorkspaceError>;

    fn pipeline_next(&self, run_id: &str) -> Result<String, WorkspaceError>;

    fn complete_stage(
        &mut self,
        stage: &str,
        run_id: &str,
        confirm: bool,
    ) -> Result<StageCompleteRow, WorkspaceError>;
}
