//! Per-machine agent-runtime settings at `~/.popsicle/agent-runtime.json`.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use storage::WorkspaceError;

pub const AGENT_RUNTIME_CONFIG_FILE: &str = "agent-runtime.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AgentRuntimeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_id: Option<String>,
}

pub fn agent_runtime_config_path() -> Result<PathBuf, WorkspaceError> {
    Ok(crate::global_config::global_home()?.join(AGENT_RUNTIME_CONFIG_FILE))
}

pub fn load_agent_runtime_config() -> Result<AgentRuntimeConfig, WorkspaceError> {
    let path = agent_runtime_config_path()?;
    if !path.is_file() {
        return Ok(AgentRuntimeConfig::default());
    }
    let content = fs::read_to_string(&path).map_err(io_err)?;
    serde_json::from_str(&content)
        .map_err(|e| WorkspaceError::InvalidState(format!("invalid agent-runtime.json: {e}")))
}

pub fn save_agent_runtime_config(cfg: &AgentRuntimeConfig) -> Result<(), WorkspaceError> {
    let path = agent_runtime_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_err)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(cfg).map_err(|e| {
            WorkspaceError::InvalidState(format!("serialize agent-runtime.json: {e}"))
        })?,
    )
    .map_err(io_err)
}

pub fn effective_runtime_id(cfg: &AgentRuntimeConfig) -> String {
    cfg.runtime_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("default")
        .to_string()
}

pub fn server_client_from_config(
    cfg: &AgentRuntimeConfig,
) -> Result<agent_daemon::ServerClient, String> {
    let base = cfg
        .server_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "未配置 Agent Runtime Server URL".to_string())?;
    Ok(agent_daemon::ServerClient::new(
        base,
        effective_runtime_id(cfg),
    ))
}

fn io_err(e: std::io::Error) -> WorkspaceError {
    WorkspaceError::InvalidState(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_runtime_id_is_default() {
        let cfg = AgentRuntimeConfig::default();
        assert_eq!(effective_runtime_id(&cfg), "default");
    }
}
