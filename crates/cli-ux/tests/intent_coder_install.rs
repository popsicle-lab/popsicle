//! intent-coder module install into `.popsicle/modules/`.

use std::fs;

use cli_ux::{install_intent_coder_module, IntentCoderSource, Workspace};

#[test]
fn install_intent_coder_module_from_repo_root() {
    let root = std::env::current_dir().expect("cwd");
    if !root.join("intent-coder/module.yaml").is_file() {
        return;
    }
    let ws = Workspace::at(root);
    let result = install_intent_coder_module(&ws, true).expect("sync module");
    assert!(result.installed, "expected fresh install: {:?}", result);
    assert_eq!(result.source, Some(IntentCoderSource::WorkspaceRoot));
    assert!(ws
        .intent_coder_module_dir()
        .join("skills/issue-author/guide.md")
        .is_file());
    assert_eq!(result.version.as_deref(), Some("0.7.0"));
}

#[test]
fn install_intent_coder_module_embedded_without_repo_root() {
    let root = std::env::temp_dir().join(format!(
        "popsicle-embedded-module-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("temp root");
    let ws = Workspace::at(root.clone());
    let result = install_intent_coder_module(&ws, true).expect("embedded install");
    assert!(result.installed);
    assert_eq!(result.source, Some(IntentCoderSource::Embedded));
    assert!(ws
        .intent_coder_module_dir()
        .join("skills/shadow-implementer/skill.yaml")
        .is_file());
    assert!(ws
        .intent_coder_module_dir()
        .join("tools/intent-validate/tool.yaml")
        .is_file());
    assert!(ws
        .intent_coder_module_dir()
        .join("tools/mermaid-diagram/tool.yaml")
        .is_file());
    let _ = fs::remove_dir_all(&root);
}
