//! Per-workspace project.yaml and AGENTS.md sync tests.

use std::fs;

use cli_ux::project_config::{
    ensure_project_config, load_project_config, project_config_path, save_project_config,
    sync_agents_md, AgentLanguage, ProjectConfig,
};

fn temp_workspace() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "popsicle-project-config-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join(".popsicle/self-host")).unwrap();
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
    assert!(agents.contains("产品文档目录"));
}

#[test]
fn sync_replaces_marker_block() {
    let root = temp_workspace();
    fs::write(
        root.join("AGENTS.md"),
        "# Existing\n\n<!-- popsicle:project-config:start -->\nold\n<!-- popsicle:project-config:end -->\n",
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
fn load_returns_defaults_when_missing() {
    let root = temp_workspace();
    let cfg = load_project_config(&root).unwrap();
    assert_eq!(cfg.version, 1);
    assert_eq!(cfg.paths.products_dir, "products");
}
