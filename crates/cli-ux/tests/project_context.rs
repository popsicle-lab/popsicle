//! `docs/PROJECT_CONTEXT.md` load/save/inject (ADR-026).

use std::fs;

use cli_ux::project_config::ensure_project_config;
use cli_ux::project_context::{
    load_project_context, project_context_for_injection, project_context_path,
    save_project_context, DEFAULT_INJECTION_MAX_BYTES,
};

fn temp_workspace() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "popsicle-project-context-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join(".popsicle")).unwrap();
    dir
}

#[test]
fn save_load_roundtrip() {
    let root = temp_workspace();
    let content = "# Project Context\n\n## 工程画像\n\nRust workspace.\n";
    save_project_context(&root, content).unwrap();
    assert_eq!(load_project_context(&root).unwrap(), content);
    assert!(project_context_path(&root).is_file());
}

#[test]
fn injection_prefers_engineering_section() {
    let root = temp_workspace();
    let md = "# Project Context\n\n## 工程画像\n\nAlpha.\n\n## 现在状态\n\nBeta.\n";
    save_project_context(&root, md).unwrap();
    let injected = project_context_for_injection(&root, DEFAULT_INJECTION_MAX_BYTES);
    assert!(injected.contains("Alpha"));
    assert!(!injected.contains("Beta"));
}

#[test]
fn agent_context_includes_project_context_when_inject_on() {
    let root = temp_workspace();
    ensure_project_config(&root).unwrap();
    let md = "# Project Context\n\n## 工程画像\n\nWorkspace crates/*.\n";
    save_project_context(&root, md).unwrap();
    let ctx = cli_ux::project_config::agent_prompt_context(&root);
    assert!(ctx.contains("[Project preferences]"));
    assert!(ctx.contains("[Project context]"));
    assert!(ctx.contains("Workspace crates"));
}

#[test]
fn bundled_weekly_health_pipeline_name() {
    assert!(cli_ux::bundled_pipeline_names()
        .iter()
        .any(|n| n == "doc-sync-weekly"));
}
