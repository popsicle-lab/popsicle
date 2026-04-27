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
    #[serde(default)]
    pub module: ModuleSection,
    #[serde(default)]
    pub sync: SyncSection,
}

/// Cloud-sync configuration. All fields optional; sync is opt-in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSection {
    /// Base URL of a popsicle-cloud-compatible server (e.g.
    /// `https://api.popsicle.cloud`). When empty, sync is disabled.
    #[serde(default)]
    pub endpoint: String,
    /// Master switch. Sync requires both `enabled = true` and a non-empty
    /// `endpoint`.
    #[serde(default)]
    pub enabled: bool,
    /// Periodic full-reconcile interval for the sync daemon, in seconds.
    #[serde(default = "default_sync_interval")]
    pub interval_secs: u64,
    /// Whether `popsicle` should auto-spawn the sync daemon on relevant
    /// commands. Manual `popsicle sync daemon start` is always available.
    #[serde(default)]
    pub auto_start_daemon: bool,
}

impl Default for SyncSection {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            enabled: false,
            interval_secs: default_sync_interval(),
            auto_start_daemon: false,
        }
    }
}

impl SyncSection {
    /// Whether sync is fully configured and enabled.
    pub fn is_active(&self) -> bool {
        self.enabled && !self.endpoint.is_empty()
    }
}

fn default_sync_interval() -> u64 {
    300
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectSection {
    pub default_pipeline: Option<String>,
    /// Issue key prefix (e.g. "PROJ" produces keys like PROJ-1, PROJ-2).
    pub key_prefix: Option<String>,
}

impl ProjectSection {
    pub fn key_prefix_or_default(&self) -> &str {
        self.key_prefix.as_deref().unwrap_or("PROJ")
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
    #[serde(default)]
    pub targets: Vec<String>,
}

impl Default for AgentSection {
    fn default() -> Self {
        Self {
            install_instructions: true,
            targets: vec![],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleSection {
    /// Active module name (defaults to "official").
    pub name: Option<String>,
    /// Where the module was installed from (e.g. "builtin", "github:user/repo").
    pub source: Option<String>,
    /// Installed module version.
    pub version: Option<String>,
}

impl ModuleSection {
    pub fn name_or_default(&self) -> &str {
        self.name.as_deref().unwrap_or("official")
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

    pub fn save(&self, config_path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::error::PopsicleError::Storage(format!("Failed to serialize config: {}", e))
        })?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}
