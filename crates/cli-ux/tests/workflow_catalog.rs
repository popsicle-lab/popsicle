//! Integration tests for workflow catalog (UI help center).

use cli_ux::{build_workflow_catalog, install_intent_coder_module, Workspace};

fn temp_ws() -> Workspace {
    let root = std::env::temp_dir().join(format!(
        "popsicle-wf-catalog-it-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join(".popsicle/pipelines")).unwrap();
    std::fs::create_dir_all(root.join(".popsicle/self-host")).unwrap();
    Workspace::at(root)
}

#[test]
fn workflow_catalog_includes_pipelines_and_skills() {
    let ws = temp_ws();
    ws.install_bundled_pipelines().unwrap();
    install_intent_coder_module(&ws, false).unwrap();

    let cat = build_workflow_catalog(&ws).unwrap();
    assert!(cat.pipelines.iter().any(|p| p.name == "slice-delivery"));
    assert!(cat.skills.iter().any(|s| s.name == "prd-writer"));
    assert!(cat
        .skills
        .iter()
        .find(|s| s.name == "issue-author")
        .map(|s| s.standalone)
        .unwrap_or(false));
}

#[test]
fn skill_used_in_pipelines_is_populated() {
    let ws = temp_ws();
    ws.install_bundled_pipelines().unwrap();
    install_intent_coder_module(&ws, false).unwrap();

    let cat = build_workflow_catalog(&ws).unwrap();
    let prd = cat.skills.iter().find(|s| s.name == "prd-writer").unwrap();
    assert!(prd.used_in_pipelines.contains(&"slice-spec".to_string()));
}
