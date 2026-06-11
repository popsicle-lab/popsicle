//! Per-workspace agent preferences at `.popsicle/project.yaml`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use storage::WorkspaceError;

pub const PROJECT_CONFIG_FILE: &str = "project.yaml";
const AGENTS_MD: &str = "AGENTS.md";
const MARKER_START: &str = "<!-- popsicle:project-config:start -->";
const MARKER_END: &str = "<!-- popsicle:project-config:end -->";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentLanguage {
    #[serde(alias = "zh-CN", alias = "zh_CN", alias = "chinese")]
    ZhCn,
    #[serde(alias = "en-US", alias = "english")]
    En,
}

impl AgentLanguage {
    pub fn as_str(self) -> &'static str {
        self.label_code()
    }

    fn label_code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ZhCn => "简体中文",
            Self::En => "English",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_cn" | "chinese" => Self::ZhCn,
            _ => Self::En,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_language")]
    pub language: AgentLanguage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathConfig {
    #[serde(default = "default_products_dir")]
    pub products_dir: String,
    #[serde(default)]
    pub default_spec: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default = "default_true")]
    pub sync_agents_md: bool,
    #[serde(default = "default_true")]
    pub inject_on_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub paths: PathConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: 1,
            agent: AgentConfig::default(),
            paths: PathConfig::default(),
            workflow: WorkflowConfig::default(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            language: detect_default_language(),
        }
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            products_dir: default_products_dir(),
            default_spec: String::new(),
        }
    }
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            sync_agents_md: true,
            inject_on_run: true,
        }
    }
}

fn default_version() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

fn default_products_dir() -> String {
    "products".into()
}

fn default_language() -> AgentLanguage {
    detect_default_language()
}

pub fn detect_default_language() -> AgentLanguage {
    let lang = std::env::var("LANG").unwrap_or_default();
    if lang.to_ascii_lowercase().starts_with("zh") {
        AgentLanguage::ZhCn
    } else {
        AgentLanguage::En
    }
}

pub fn products_dir_path(workspace_root: &Path) -> PathBuf {
    let rel = load_project_config(workspace_root)
        .map(|c| c.paths.products_dir)
        .unwrap_or_else(|_| default_products_dir());
    workspace_root.join(rel)
}

pub fn project_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".popsicle").join(PROJECT_CONFIG_FILE)
}

pub fn load_project_config(workspace_root: &Path) -> Result<ProjectConfig, WorkspaceError> {
    let path = project_config_path(workspace_root);
    if !path.is_file() {
        return Ok(ProjectConfig::default());
    }
    let content = fs::read_to_string(&path).map_err(io_err)?;
    serde_yaml::from_str(&content)
        .map_err(|e| WorkspaceError::InvalidState(format!("invalid project config: {e}")))
}

pub fn save_project_config(
    workspace_root: &Path,
    config: &ProjectConfig,
) -> Result<(), WorkspaceError> {
    let path = project_config_path(workspace_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_err)?;
    }
    let content = serde_yaml::to_string(config).map_err(|e| io_err(e.to_string()))?;
    fs::write(&path, content).map_err(io_err)
}

/// Write default `project.yaml` when missing; optionally sync `AGENTS.md`.
pub fn ensure_project_config(workspace_root: &Path) -> Result<ProjectConfig, WorkspaceError> {
    let path = project_config_path(workspace_root);
    let config = if path.is_file() {
        load_project_config(workspace_root)?
    } else {
        let config = ProjectConfig::default();
        save_project_config(workspace_root, &config)?;
        config
    };
    if config.workflow.sync_agents_md {
        sync_agents_md(workspace_root, &config)?;
    }
    Ok(config)
}

pub fn prompt_context_block(config: &ProjectConfig) -> String {
    let lang = config.agent.language;
    let spec_line = if config.paths.default_spec.is_empty() {
        String::new()
    } else {
        format!("\n- Default spec: `{}`", config.paths.default_spec)
    };
    format!(
        "[Project preferences]\n- Respond in: {}\n- Products directory: `{}/`\n- ADRs: `{}/<product>/decisions/adr/`\n- PDRs: `{}/<product>/decisions/pdr/`{spec_line}",
        lang.label(),
        config.paths.products_dir,
        config.paths.products_dir,
        config.paths.products_dir,
    )
}

pub fn agents_md_section(config: &ProjectConfig) -> String {
    let lang = config.agent.language;
    let spec_line = if config.paths.default_spec.is_empty() {
        String::new()
    } else {
        format!("\n- **默认 spec**：`{}`", config.paths.default_spec)
    };
    format!(
        "{MARKER_START}\n## 本项目 Agent 偏好\n\n- **回复语言**：{}\n- **产品文档目录**：`{}/`\n- **决策记录**：`{}/<product>/decisions/{{adr,pdr}}/`{spec_line}\n{MARKER_END}",
        lang.label(),
        config.paths.products_dir,
        config.paths.products_dir,
    )
}

pub fn sync_agents_md(workspace_root: &Path, config: &ProjectConfig) -> Result<(), WorkspaceError> {
    let agents_path = workspace_root.join(AGENTS_MD);
    let section = agents_md_section(config);
    let content = if agents_path.is_file() {
        let existing = fs::read_to_string(&agents_path).map_err(io_err)?;
        upsert_marked_section(&existing, &section)
    } else {
        format!("# Agent Instructions\n\n{section}\n")
    };
    fs::write(&agents_path, content).map_err(io_err)
}

fn upsert_marked_section(existing: &str, section: &str) -> String {
    if let (Some(start), Some(end)) = (existing.find(MARKER_START), existing.find(MARKER_END)) {
        let before = &existing[..start];
        let after = &existing[end + MARKER_END.len()..];
        let mut out = String::new();
        out.push_str(before.trim_end());
        if !before.trim().is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(section);
        out.push_str(after);
        out
    } else if existing.trim().is_empty() {
        format!("# Agent Instructions\n\n{section}\n")
    } else {
        format!("{existing}\n\n{section}\n")
    }
}

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_replaces_existing_marker_block() {
        let existing = "preamble\n\n<!-- popsicle:project-config:start -->\nold\n<!-- popsicle:project-config:end -->\nfooter";
        let section =
            "<!-- popsicle:project-config:start -->\nnew\n<!-- popsicle:project-config:end -->";
        let out = upsert_marked_section(existing, section);
        assert!(out.contains("preamble"));
        assert!(out.contains("new"));
        assert!(!out.contains("old"));
    }

    #[test]
    fn roundtrip_yaml() {
        let cfg = ProjectConfig::default();
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let parsed: ProjectConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.paths.products_dir, "products");
    }
}
