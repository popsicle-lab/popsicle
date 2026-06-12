//! mermaid-diagram bundled tool resolution and scaffold action.

use std::collections::BTreeMap;
use std::fs;

use cli_ux::{run_tool, Workspace};

#[test]
fn mermaid_diagram_tool_yaml_exists_in_repo() {
    let root = std::env::current_dir().expect("cwd");
    if !root
        .join("intent-coder/tools/mermaid-diagram/tool.yaml")
        .is_file()
    {
        return;
    }
    let ws = Workspace::at(root);
    let mut args = BTreeMap::new();
    args.insert("action".into(), "scaffold".into());
    args.insert("type".into(), "flowchart".into());
    args.insert("title".into(), "test".into());
    args.insert("format".into(), "text".into());
    let code = run_tool(&ws, "mermaid-diagram", &args).expect("run scaffold");
    assert_eq!(code, 0, "scaffold should succeed");
}

#[test]
fn mermaid_diagram_guide_action_prints_guide() {
    let root = std::env::current_dir().expect("cwd");
    if !root
        .join("intent-coder/tools/mermaid-diagram/guide.md")
        .is_file()
    {
        return;
    }
    let ws = Workspace::at(root);
    let mut args = BTreeMap::new();
    args.insert("action".into(), "guide".into());
    let code = run_tool(&ws, "mermaid-diagram", &args).expect("run guide");
    assert_eq!(code, 0);
}

#[test]
fn parse_tool_spec_reads_required_and_defaults() {
    let root = std::env::current_dir().expect("cwd");
    let path = root.join("intent-coder/tools/mermaid-diagram/tool.yaml");
    if !path.is_file() {
        return;
    }
    let content = fs::read_to_string(&path).expect("read tool.yaml");
    assert!(content.contains("name: mermaid-diagram"));
    assert!(content.contains("action"));
    assert!(content.contains("command: |"));
}
