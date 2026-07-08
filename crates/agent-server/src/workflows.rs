//! Workspace pipeline catalog via skill-runtime (read from disk on each request).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use skill_runtime::{load_pipelines_dir, LoadError, PipelineDef};

use crate::AppState;

const LEGACY_PIPELINES_REL: &str = ".popsicle/pipelines";
const MODULE_REL: &str = ".popsicle/modules/intent-coder";
const LIVE_REL: &str = "intent-coder";

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowPipelineEntry {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowsResponse {
    pub workspace_id: String,
    pub pipelines: Vec<WorkflowPipelineEntry>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowsQuery {
    pub workspace_id: String,
}

#[derive(Debug)]
pub enum WorkflowListError {
    InvalidWorkspace(String),
    Load(LoadError),
}

impl std::fmt::Display for WorkflowListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWorkspace(msg) => write!(f, "{msg}"),
            Self::Load(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for WorkflowListError {}

fn has_live_intent_coder_root(root: &Path) -> bool {
    root.join(LIVE_REL).join("module.yaml").is_file()
}

/// Same search order as cli-ux `intent_coder_resolve::pipeline_search_dirs`.
pub fn pipeline_search_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if has_live_intent_coder_root(root) {
        let live = root.join(LIVE_REL).join("pipelines");
        if live.is_dir() {
            dirs.push(live);
        }
    }
    let legacy = root.join(LEGACY_PIPELINES_REL);
    if legacy.is_dir() {
        dirs.push(legacy);
    }
    let module = root.join(MODULE_REL).join("pipelines");
    if module.is_dir() {
        dirs.push(module);
    }
    dirs
}

/// Resolve workspace root: direct path (local server) or container mount fallback.
pub fn resolve_workspace_root(workspace_id: &str) -> Result<PathBuf, WorkflowListError> {
    let trimmed = workspace_id.trim();
    if trimmed.is_empty() {
        return Err(WorkflowListError::InvalidWorkspace(
            "workspace_id is empty".into(),
        ));
    }
    let requested = Path::new(trimmed);
    if requested.is_dir() {
        return Ok(requested.to_path_buf());
    }
    if let Ok(mount) = std::env::var("AGENT_RUNTIME_WORKSPACE_ROOT") {
        let mount = PathBuf::from(mount.trim());
        if mount.is_dir() {
            return Ok(mount);
        }
    }
    Err(WorkflowListError::InvalidWorkspace(format!(
        "workspace not found: {trimmed} (container 需挂载工作区并设置 AGENT_RUNTIME_WORKSPACE_ROOT)"
    )))
}

/// List installed pipeline templates for a workspace (first dir wins per name).
pub fn list_workspace_pipelines(
    root: &Path,
) -> Result<Vec<WorkflowPipelineEntry>, WorkflowListError> {
    if !root.is_dir() {
        return Err(WorkflowListError::InvalidWorkspace(format!(
            "workspace not found: {}",
            root.display()
        )));
    }
    let mut by_name: BTreeMap<String, WorkflowPipelineEntry> = BTreeMap::new();
    for dir in pipeline_search_dirs(root) {
        let defs = load_pipelines_dir(&dir).map_err(WorkflowListError::Load)?;
        for def in defs {
            by_name
                .entry(def.name.clone())
                .or_insert_with(|| pipeline_entry(&def));
        }
    }
    Ok(by_name.into_values().collect())
}

fn pipeline_entry(def: &PipelineDef) -> WorkflowPipelineEntry {
    WorkflowPipelineEntry {
        name: def.name.clone(),
        description: def.description.clone(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowsErrorResponse {
    pub error: String,
    pub detail: String,
}

pub async fn list_workflows(
    State(_state): State<AppState>,
    Query(query): Query<WorkflowsQuery>,
) -> Result<Json<WorkflowsResponse>, (StatusCode, Json<WorkflowsErrorResponse>)> {
    let workspace_id = query.workspace_id.trim();
    if workspace_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(WorkflowsErrorResponse {
                error: "bad_request".into(),
                detail: "workspace_id is required".into(),
            }),
        ));
    }
    let root = resolve_workspace_root(workspace_id).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(WorkflowsErrorResponse {
                error: "workspace_not_found".into(),
                detail: e.to_string(),
            }),
        )
    })?;
    let pipelines = list_workspace_pipelines(&root).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(WorkflowsErrorResponse {
                error: "workspace_not_readable".into(),
                detail: e.to_string(),
            }),
        )
    })?;
    Ok(Json(WorkflowsResponse {
        workspace_id: workspace_id.to_string(),
        pipelines,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn resolve_workspace_root_uses_mount_fallback() {
        let root = env::current_dir().expect("cwd");
        if !root.join("intent-coder/module.yaml").is_file() {
            return;
        }
        std::env::set_var("AGENT_RUNTIME_WORKSPACE_ROOT", &root);
        let resolved = resolve_workspace_root("/nonexistent/host/path").expect("mount fallback");
        assert_eq!(resolved, root);
        std::env::remove_var("AGENT_RUNTIME_WORKSPACE_ROOT");
    }

    #[test]
    fn list_workspace_pipelines_in_monorepo() {
        let root = env::current_dir().expect("cwd");
        if !root.join("intent-coder/module.yaml").is_file() {
            return;
        }
        let pipelines = list_workspace_pipelines(&root).expect("list");
        assert!(pipelines.iter().any(|p| p.name == "feature-delivery"));
        assert!(pipelines.iter().any(|p| p.name == "feature-spec"));
    }
}
