//! Per-workspace project.yaml and AGENTS.md sync tests.

use std::fs;

use cli_ux::project_config::{
    agent_prompt_context, authoring_language_guidance, ensure_project_config, load_project_config,
    project_config_path, prompt_context_block, save_project_config, sync_agents_md, AgentLanguage,
    ProjectConfig,
};

fn temp_workspace() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "popsicle-project-config-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join(".popsicle")).unwrap();
    dir
}

#[test]
fn ensure_writes_default_yaml_and_agents_md() {
    let root = temp_workspace();
    let cfg = ensure_project_config(&root).unwrap();
    assert_eq!(cfg.paths.products_dir, "products");
    assert!(project_config_path(&root).is_file());
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.contains("popsicle:project-config:start"));
    assert!(agents.contains("products/"));
    assert!(agents.contains("Command Reference"));
    assert!(agents.contains("MANDATORY: Before Starting"));
    let localized = match cfg.agent.language {
        AgentLanguage::ZhCn => agents.contains("产品文档目录"),
        AgentLanguage::En => agents.contains("Products directory"),
    };
    assert!(localized);
}

#[test]
fn sync_upgrades_legacy_stub_agents_md() {
    let root = temp_workspace();
    fs::write(
        root.join("AGENTS.md"),
        "# Agent Instructions\n\n<!-- popsicle:project-config:start -->\nold\n<!-- popsicle:project-config:end -->\n",
    )
    .unwrap();
    let cfg = ProjectConfig::default();
    sync_agents_md(&root, &cfg).unwrap();
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.contains("Command Reference"));
    assert!(!agents.contains("\nold\n"));
}

#[test]
fn sync_replaces_marker_block_on_full_agents_md() {
    let root = temp_workspace();
    fs::write(
        root.join("AGENTS.md"),
        "# Existing\n\n## Command Reference (complete)\n\npopsicle issue list\n\n## ⛔ MANDATORY: Before Starting ANY Development Task\n\n<!-- popsicle:project-config:start -->\nold\n<!-- popsicle:project-config:end -->\n",
    )
    .unwrap();
    let mut cfg = ProjectConfig::default();
    cfg.agent.language = AgentLanguage::En;
    save_project_config(&root, &cfg).unwrap();
    sync_agents_md(&root, &cfg).unwrap();
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.contains("English"));
    assert!(!agents.contains("old"));
    assert!(agents.contains("# Existing"));
}

#[test]
fn default_spec_alias_deserializes_as_default_product() {
    let root = temp_workspace();
    fs::write(
        root.join(".popsicle/project.yaml"),
        "paths:\n  default_spec: cli-ux\n",
    )
    .unwrap();
    let cfg = load_project_config(&root).unwrap();
    assert_eq!(cfg.paths.default_product, "cli-ux");
}

#[test]
fn zh_cn_prompt_context_requires_chinese_issue_titles() {
    let mut cfg = ProjectConfig::default();
    cfg.agent.language = AgentLanguage::ZhCn;
    let block = prompt_context_block(&cfg);
    assert!(block.contains("简体中文"));
    assert!(block.contains(authoring_language_guidance(AgentLanguage::ZhCn)));
}

#[test]
fn agent_prompt_context_empty_when_inject_disabled() {
    let root = temp_workspace();
    let mut cfg = ProjectConfig::default();
    cfg.workflow.inject_on_run = false;
    save_project_config(&root, &cfg).unwrap();
    assert!(agent_prompt_context(&root).is_empty());
}

#[test]
fn load_returns_defaults_when_missing() {
    let root = temp_workspace();
    let cfg = load_project_config(&root).unwrap();
    assert_eq!(cfg.version, 1);
    assert_eq!(cfg.paths.products_dir, "products");
}
