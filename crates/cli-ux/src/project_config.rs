//! Per-workspace agent preferences at `.popsicle/project.yaml`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use storage::WorkspaceError;

pub const PROJECT_CONFIG_FILE: &str = "project.yaml";
const AGENTS_MD: &str = "AGENTS.md";
const GITIGNORE_FILE: &str = ".gitignore";
const MARKER_START: &str = "<!-- popsicle:project-config:start -->";
const MARKER_END: &str = "<!-- popsicle:project-config:end -->";
const GITIGNORE_MARKER_START: &str = "# popsicle:gitignore:start";
const GITIGNORE_MARKER_END: &str = "# popsicle:gitignore:end";

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
    /// Default product for new issues (`products/<name>/`). Legacy yaml key: `default_spec`.
    #[serde(default, alias = "default_spec")]
    pub default_product: String,
}

/// Workflow persona: adjusts default pipelines and UI emphasis (not RBAC).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowProfile {
    /// Daily bugfix / incremental delivery on existing spec.
    #[default]
    DailyDev,
    /// Large migration: spec chain before implement.
    Migration,
    /// PM / spec authoring; de-emphasize implement stages.
    PmSpecOnly,
    /// End-to-end OPC with delegated approvals.
    OpcFull,
}

impl WorkflowProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DailyDev => "daily-dev",
            Self::Migration => "migration",
            Self::PmSpecOnly => "pm-spec-only",
            Self::OpcFull => "opc-full",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "migration" | "migrate" => Self::Migration,
            "pm" | "pm-spec-only" | "pm_spec_only" | "product-manager" => Self::PmSpecOnly,
            "opc" | "opc-full" | "opc_full" | "full" => Self::OpcFull,
            _ => Self::DailyDev,
        }
    }

    pub fn label(self, lang: AgentLanguage) -> &'static str {
        match (self, lang) {
            (Self::DailyDev, AgentLanguage::ZhCn) => "日常开发",
            (Self::Migration, AgentLanguage::ZhCn) => "大型迁移",
            (Self::PmSpecOnly, AgentLanguage::ZhCn) => "产品经理 / Spec",
            (Self::OpcFull, AgentLanguage::ZhCn) => "OPC 全流程",
            (Self::DailyDev, AgentLanguage::En) => "Daily development",
            (Self::Migration, AgentLanguage::En) => "Large migration",
            (Self::PmSpecOnly, AgentLanguage::En) => "PM / spec authoring",
            (Self::OpcFull, AgentLanguage::En) => "OPC full pipeline",
        }
    }

    /// Default pipeline for `issue create` given issue type (ADR-029 taxonomy).
    pub fn default_pipeline(self, issue_type: &str) -> &'static str {
        if issue_type == "bug" {
            return "fix-regression";
        }
        match self {
            Self::DailyDev => match issue_type {
                "technical" => "feature-delivery",
                "product" => "product-greenfield-spec",
                _ => "arch-decision",
            },
            Self::Migration => match issue_type {
                "technical" => "migration-slice-spec",
                "product" => "product-greenfield-spec",
                _ => "arch-decision",
            },
            Self::PmSpecOnly => match issue_type {
                "technical" => "feature-spec",
                "product" => "product-greenfield-spec",
                _ => "feature-spec",
            },
            Self::OpcFull => "product-greenfield-spec",
        }
    }

    pub fn suggested_approval_mode(self) -> ApprovalMode {
        match self {
            Self::DailyDev => ApprovalMode::DelegateDangerous,
            Self::Migration | Self::PmSpecOnly => ApprovalMode::Manual,
            Self::OpcFull => ApprovalMode::Auto,
        }
    }
}

/// Pipeline stage completion policy for `requires_approval` gates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalMode {
    /// Every `requires_approval` stage needs explicit human `--confirm` (default).
    #[serde(alias = "required")]
    #[default]
    Manual,
    /// Agent may complete all stages; `--confirm` is implied when needed.
    #[serde(alias = "automatic", alias = "full")]
    Auto,
    /// Non-dangerous `requires_approval` stages may be delegated; dangerous ones still need human confirm.
    #[serde(alias = "delegate", alias = "dangerous-only")]
    DelegateDangerous,
}

impl ApprovalMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Auto => "auto",
            Self::DelegateDangerous => "delegate-dangerous",
        }
    }

    pub fn label_zh(self) -> &'static str {
        self.label(AgentLanguage::ZhCn)
    }

    pub fn label(self, lang: AgentLanguage) -> &'static str {
        match (self, lang) {
            (Self::Manual, AgentLanguage::ZhCn) => "必须人工审批",
            (Self::Auto, AgentLanguage::ZhCn) => "全自动",
            (Self::DelegateDangerous, AgentLanguage::ZhCn) => "危险操作需审批（其余代批）",
            (Self::Manual, AgentLanguage::En) => "Manual approval required",
            (Self::Auto, AgentLanguage::En) => "Fully automatic",
            (Self::DelegateDangerous, AgentLanguage::En) => {
                "Dangerous stages need approval (delegate others)"
            }
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" | "automatic" | "full" => Self::Auto,
            "delegate" | "delegate-dangerous" | "delegate_dangerous" | "dangerous-only"
            | "dangerous_only" => Self::DelegateDangerous,
            _ => Self::Manual,
        }
    }
}

/// Stages that remain human-gated under [`ApprovalMode::DelegateDangerous`].
pub const DANGEROUS_STAGES: &[&str] = &["cutover", "living-docs"];

pub fn is_dangerous_stage(stage: &str) -> bool {
    DANGEROUS_STAGES.contains(&stage)
}

/// Whether CLI/UI must collect explicit `--confirm` before completing this stage.
pub fn stage_needs_explicit_confirm(
    mode: ApprovalMode,
    stage: &str,
    requires_approval: bool,
) -> bool {
    if !requires_approval {
        return false;
    }
    match mode {
        ApprovalMode::Manual => true,
        ApprovalMode::Auto => false,
        ApprovalMode::DelegateDangerous => is_dangerous_stage(stage),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default)]
    pub profile: WorkflowProfile,
    #[serde(default = "default_true")]
    pub sync_agents_md: bool,
    #[serde(default = "default_true")]
    pub inject_on_run: bool,
    #[serde(default)]
    pub approval_mode: ApprovalMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GitConfig {
    /// When `false` (default), sync adds `.popsicle/` to the managed `.gitignore` block.
    /// When `true`, workspace files under `.popsicle/` may be committed to git.
    #[serde(default)]
    pub track_workspace: bool,
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
    #[serde(default)]
    pub git: GitConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: 1,
            agent: AgentConfig::default(),
            paths: PathConfig::default(),
            workflow: WorkflowConfig::default(),
            git: GitConfig::default(),
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
            default_product: String::new(),
        }
    }
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            profile: WorkflowProfile::default(),
            sync_agents_md: true,
            inject_on_run: true,
            approval_mode: ApprovalMode::default(),
        }
    }
}

/// Resolve default pipeline map for issue-create UI / CLI hints.
pub fn default_pipelines_by_type(profile: WorkflowProfile) -> BTreeMap<String, String> {
    ["product", "technical", "bug", "idea"]
        .into_iter()
        .map(|t| (t.to_string(), profile.default_pipeline(t).to_string()))
        .collect()
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
    sync_gitignore(workspace_root, &config)?;
    Ok(config)
}

/// Remind agents to match project language when filling `--title` / `--description`.
pub fn authoring_language_guidance(lang: AgentLanguage) -> &'static str {
    match lang {
        AgentLanguage::ZhCn => {
            "创建或更新 Issue / 文档时，`--title` 与 `--description` 使用简体中文（除非用户明确要求英文）。"
        }
        AgentLanguage::En => {
            "When creating issues or docs, write `--title` and `--description` in English unless the user asks otherwise."
        }
    }
}

pub fn approval_mode_guidance(mode: ApprovalMode, lang: AgentLanguage) -> &'static str {
    match (mode, lang) {
        (ApprovalMode::Manual, AgentLanguage::En) => {
            "STOP after each stage; wait for the user before `pipeline stage complete`. \
             `requires_approval` stages need human `--confirm`."
        }
        (ApprovalMode::Auto, AgentLanguage::En) => {
            "After `doc check` passes, you may run `pipeline stage complete` without waiting; \
             `--confirm` is implied for `requires_approval` stages."
        }
        (ApprovalMode::DelegateDangerous, AgentLanguage::En) => {
            "You may auto-complete non-dangerous `requires_approval` stages. \
             Dangerous stages (`cutover`, `living-docs`) still need explicit human `--confirm`."
        }
        (ApprovalMode::Manual, AgentLanguage::ZhCn) => {
            "每完成一个 stage 的文档后暂停，向用户汇报并等待确认后再执行 `pipeline stage complete`；\
             带 `requires_approval` 的阶段必须由用户亲自 `--confirm`。"
        }
        (ApprovalMode::Auto, AgentLanguage::ZhCn) => {
            "文档通过 `doc check` 后可直接 `pipeline stage complete`；\
             `requires_approval` 阶段由系统自动代批（无需 `--confirm`）。"
        }
        (ApprovalMode::DelegateDangerous, AgentLanguage::ZhCn) => {
            "非危险 `requires_approval` 阶段可由 agent 代批完成；\
             危险阶段（`cutover`、`living-docs`）仍需用户显式 `--confirm`。"
        }
    }
}

/// Inject into CLI JSON / artifact frontmatter when `workflow.inject_on_run` is enabled.
pub fn agent_prompt_context(workspace_root: &Path) -> String {
    let Some(config) = load_project_config(workspace_root)
        .ok()
        .filter(|c| c.workflow.inject_on_run)
    else {
        return String::new();
    };
    let mut out = prompt_context_block(&config);
    out.push_str(&crate::project_context::project_context_injection_block(
        workspace_root,
        crate::project_context::DEFAULT_INJECTION_MAX_BYTES,
    ));
    out
}

pub fn prompt_context_block(config: &ProjectConfig) -> String {
    let lang = config.agent.language;
    let mode = config.workflow.approval_mode;
    let product_line = if config.paths.default_product.is_empty() {
        String::new()
    } else {
        match lang {
            AgentLanguage::ZhCn => {
                format!("\n- 默认产品：`{}`", config.paths.default_product)
            }
            AgentLanguage::En => {
                format!("\n- Default product: `{}`", config.paths.default_product)
            }
        }
    };
    let authoring = authoring_language_guidance(lang);
    match lang {
        AgentLanguage::ZhCn => format!(
            "[Project preferences]\n- 界面 / Agent 语言：{}\n- 产品目录：`{}/`\n- ADR：`{}/<product>/decisions/adr/`\n- PDR：`{}/<product>/decisions/pdr/`\n- Pipeline 审批：{}（{}）{product_line}\n- {}\n- {}",
            lang.label(),
            config.paths.products_dir,
            config.paths.products_dir,
            config.paths.products_dir,
            mode.as_str(),
            mode.label(lang),
            authoring,
            approval_mode_guidance(mode, lang),
        ),
        AgentLanguage::En => format!(
            "[Project preferences]\n- UI / agent language: {}\n- Products directory: `{}/`\n- ADRs: `{}/<product>/decisions/adr/`\n- PDRs: `{}/<product>/decisions/pdr/`\n- Pipeline approval: {} ({}){product_line}\n- {}\n- {}",
            lang.label(),
            config.paths.products_dir,
            config.paths.products_dir,
            config.paths.products_dir,
            mode.as_str(),
            mode.label(lang),
            authoring,
            approval_mode_guidance(mode, lang),
        ),
    }
}

pub fn agents_md_section(config: &ProjectConfig) -> String {
    let lang = config.agent.language;
    let product_line = if config.paths.default_product.is_empty() {
        String::new()
    } else {
        match lang {
            AgentLanguage::ZhCn => {
                format!("\n- **默认产品**：`{}`", config.paths.default_product)
            }
            AgentLanguage::En => {
                format!(
                    "\n- **Default product**: `{}`",
                    config.paths.default_product
                )
            }
        }
    };
    let mode = config.workflow.approval_mode;
    let authoring = authoring_language_guidance(lang);
    match lang {
        AgentLanguage::ZhCn => format!(
            "{MARKER_START}\n## 本项目偏好\n\n- **界面 / Agent 语言**：{}\n- **产品文档目录**：`{}/`\n- **决策记录**：`{}/<product>/decisions/{{adr,pdr}}/`{product_line}\n- **Pipeline 审批模式**：`{}`（{}）\n- **Issue / 文档文案**：{}\n\n### 阶段完成策略\n\n{}\n{MARKER_END}",
            lang.label(),
            config.paths.products_dir,
            config.paths.products_dir,
            mode.as_str(),
            mode.label(lang),
            authoring,
            approval_mode_guidance(mode, lang),
        ),
        AgentLanguage::En => format!(
            "{MARKER_START}\n## Project preferences\n\n- **UI / agent language**: {}\n- **Products directory**: `{}/`\n- **Decisions**: `{}/<product>/decisions/{{adr,pdr}}/`{product_line}\n- **Pipeline approval**: `{}` ({})\n- **Issue / doc copy**: {}\n\n### Stage completion\n\n{}\n{MARKER_END}",
            lang.label(),
            config.paths.products_dir,
            config.paths.products_dir,
            mode.as_str(),
            mode.label(lang),
            authoring,
            approval_mode_guidance(mode, lang),
        ),
    }
}

/// True when `AGENTS.md` is missing or only contains the legacy init stub.
pub fn agents_md_needs_bootstrap(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return true;
    }
    !(trimmed.contains("Command Reference")
        && trimmed.contains("MANDATORY: Before Starting")
        && trimmed.contains("popsicle issue list"))
}

fn agents_md_bootstrap_body() -> String {
    crate::intent_coder_bundle::embedded_agents_md_bootstrap()
        .map(str::trim_end)
        .map(str::to_string)
        .unwrap_or_else(|| "# Agent Instructions\n".to_string())
}

pub fn sync_agents_md(workspace_root: &Path, config: &ProjectConfig) -> Result<(), WorkspaceError> {
    let agents_path = workspace_root.join(AGENTS_MD);
    let section = agents_md_section(config);
    let existing = if agents_path.is_file() {
        fs::read_to_string(&agents_path).map_err(io_err)?
    } else {
        String::new()
    };
    let base = if agents_md_needs_bootstrap(&existing) {
        agents_md_bootstrap_body()
    } else {
        existing
    };
    let content = upsert_marked_section(&base, &section);
    fs::write(&agents_path, content).map_err(io_err)
}

/// Maintain a popsicle-managed block in root `.gitignore` from `git.track_workspace`.
pub fn sync_gitignore(
    workspace_root: &Path,
    config: &ProjectConfig,
) -> Result<bool, WorkspaceError> {
    let path = workspace_root.join(GITIGNORE_FILE);
    let section = gitignore_managed_section(config.git.track_workspace);
    let content = if path.is_file() {
        let existing = fs::read_to_string(&path).map_err(io_err)?;
        let stripped = strip_legacy_popsicle_gitignore_lines(&existing);
        upsert_gitignore_section(&stripped, &section)
    } else {
        format!("{section}\n")
    };
    let previous = if path.is_file() {
        Some(fs::read_to_string(&path).map_err(io_err)?)
    } else {
        None
    };
    let changed = previous.as_deref() != Some(content.as_str());
    if changed {
        fs::write(&path, content).map_err(io_err)?;
    }
    Ok(changed)
}

pub fn gitignore_managed_section(track_workspace: bool) -> String {
    if track_workspace {
        format!(
            "{GITIGNORE_MARKER_START}\n# git.track_workspace: true — .popsicle/ is not ignored\n{GITIGNORE_MARKER_END}"
        )
    } else {
        format!("{GITIGNORE_MARKER_START}\n.popsicle/\n{GITIGNORE_MARKER_END}")
    }
}

fn strip_legacy_popsicle_gitignore_lines(content: &str) -> String {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == ".popsicle/" {
            continue;
        }
        if trimmed.starts_with("# popsicle workspace")
            || trimmed.starts_with("# (state 由 popsicle")
            || trimmed.starts_with("#  products/*/decisions/")
        {
            continue;
        }
        out.push(line);
    }
    let mut joined = out.join("\n");
    if !joined.is_empty() && !joined.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

fn upsert_gitignore_section(existing: &str, section: &str) -> String {
    upsert_marked_block(
        existing,
        GITIGNORE_MARKER_START,
        GITIGNORE_MARKER_END,
        section,
    )
}

fn upsert_marked_section(existing: &str, section: &str) -> String {
    upsert_marked_block(existing, MARKER_START, MARKER_END, section)
}

fn upsert_marked_block(existing: &str, start: &str, end: &str, section: &str) -> String {
    if let (Some(start_idx), Some(end_idx)) = (existing.find(start), existing.find(end)) {
        let before = &existing[..start_idx];
        let after = &existing[end_idx + end.len()..];
        let mut out = String::new();
        out.push_str(before.trim_end());
        if !before.trim().is_empty() {
            out.push('\n');
            if !before.ends_with("\n\n") {
                out.push('\n');
            }
        }
        out.push_str(section);
        out.push_str(after);
        out
    } else if existing.trim().is_empty() {
        format!("{section}\n")
    } else {
        format!("{}\n\n{section}\n", existing.trim_end())
    }
}

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_md_needs_bootstrap_detects_legacy_stub() {
        let stub = "# Agent Instructions\n\n<!-- popsicle:project-config:start -->\nx\n<!-- popsicle:project-config:end -->\n";
        assert!(super::agents_md_needs_bootstrap(stub));
        assert!(super::agents_md_needs_bootstrap(""));
    }

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
        assert_eq!(parsed.workflow.approval_mode, ApprovalMode::Manual);
    }

    #[test]
    fn workflow_profile_default_pipelines() {
        assert_eq!(
            WorkflowProfile::DailyDev.default_pipeline("bug"),
            "fix-regression"
        );
        assert_eq!(
            WorkflowProfile::DailyDev.default_pipeline("technical"),
            "feature-delivery"
        );
        assert_eq!(
            WorkflowProfile::Migration.default_pipeline("technical"),
            "migration-slice-spec"
        );
        assert_eq!(
            WorkflowProfile::PmSpecOnly.default_pipeline("technical"),
            "feature-spec"
        );
        let map = default_pipelines_by_type(WorkflowProfile::OpcFull);
        assert_eq!(map.get("bug").map(String::as_str), Some("fix-regression"));
    }

    #[test]
    fn sync_gitignore_writes_managed_block_when_missing() {
        let root = std::env::temp_dir().join(format!(
            "popsicle-gitignore-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let cfg = ProjectConfig::default();
        assert!(sync_gitignore(&root, &cfg).unwrap());
        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(content.contains(GITIGNORE_MARKER_START));
        assert!(content.contains(".popsicle/"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sync_gitignore_track_workspace_allows_popsicle_dir() {
        let root = std::env::temp_dir().join(format!(
            "popsicle-gitignore-track-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut cfg = ProjectConfig::default();
        cfg.git.track_workspace = true;
        sync_gitignore(&root, &cfg).unwrap();
        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(content.contains("git.track_workspace: true"));
        assert!(!content.lines().any(|l| l.trim() == ".popsicle/"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sync_gitignore_strips_legacy_standalone_entry() {
        let root = std::env::temp_dir().join(format!(
            "popsicle-gitignore-legacy-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join(".gitignore"),
            "target/\n.popsicle/\n# popsicle workspace legacy comment\n",
        )
        .unwrap();
        sync_gitignore(&root, &ProjectConfig::default()).unwrap();
        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert_eq!(content.matches(".popsicle/").count(), 1);
        assert!(content.contains(GITIGNORE_MARKER_START));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn approval_mode_policy() {
        assert!(stage_needs_explicit_confirm(
            ApprovalMode::Manual,
            "review",
            true
        ));
        assert!(!stage_needs_explicit_confirm(
            ApprovalMode::Auto,
            "review",
            true
        ));
        assert!(!stage_needs_explicit_confirm(
            ApprovalMode::DelegateDangerous,
            "review",
            true
        ));
        assert!(stage_needs_explicit_confirm(
            ApprovalMode::DelegateDangerous,
            "cutover",
            true
        ));
        assert!(!stage_needs_explicit_confirm(
            ApprovalMode::DelegateDangerous,
            "implement",
            false
        ));
    }
}
