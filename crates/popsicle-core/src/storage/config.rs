use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Project configuration loaded from `.popsicle/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: ProjectSection,
    #[serde(default)]
    pub git: GitSection,
    #[serde(default)]
    pub agent: AgentSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSection {
    pub default_pipeline: Option<String>,
}

impl Default for ProjectSection {
    fn default() -> Self {
        Self {
            default_pipeline: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSection {
    #[serde(default = "default_true")]
    pub auto_track: bool,
}

impl Default for GitSection {
    fn default() -> Self {
        Self { auto_track: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSection {
    #[serde(default = "default_true")]
    pub install_instructions: bool,
}

impl Default for AgentSection {
    fn default() -> Self {
        Self {
            install_instructions: true,
        }
    }
}

fn default_true() -> bool {
    true
}

impl ProjectConfig {
    pub fn load(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(config_path)?;
        let config: ProjectConfig = toml::from_str(&content).map_err(|e| {
            crate::error::PopsicleError::Storage(format!("Invalid config.toml: {}", e))
        })?;
        Ok(config)
    }
}
