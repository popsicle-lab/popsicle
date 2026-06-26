use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct OtelConfig {
    #[serde(default)]
    pub exporter: ExporterConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExporterConfig {
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyConfig {
    #[serde(default = "default_true")]
    pub fail_open: bool,
    /// Background OTLP flush interval; 0 disables. Default 30 when endpoint set (see `effective_flush_interval`).
    #[serde(default = "default_flush_interval")]
    pub flush_interval_secs: u64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            fail_open: true,
            flush_interval_secs: default_flush_interval(),
        }
    }
}

fn default_flush_interval() -> u64 {
    30
}

pub fn effective_flush_interval(config: &OtelConfig) -> u64 {
    if config.exporter.endpoint.trim().is_empty() {
        0
    } else {
        config.policy.flush_interval_secs
    }
}

fn default_true() -> bool {
    true
}

/// Workspace `.popsicle/otel.yaml` overrides `~/.popsicle/otel.yaml`.
pub fn load_config(workspace_root: &Path) -> OtelConfig {
    let workspace_cfg = workspace_root.join(".popsicle/otel.yaml");
    if let Ok(c) = read_config_file(&workspace_cfg) {
        return c;
    }
    if let Some(home) = home_popsicle_dir() {
        let global_cfg = home.join("otel.yaml");
        if let Ok(c) = read_config_file(&global_cfg) {
            return c;
        }
    }
    OtelConfig::default()
}

fn read_config_file(path: &Path) -> Result<OtelConfig, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&content).map_err(|e| e.to_string())
}

fn home_popsicle_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".popsicle"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_fail_open() {
        let c = OtelConfig::default();
        assert!(c.policy.fail_open);
        assert!(c.exporter.endpoint.is_empty());
    }
}
