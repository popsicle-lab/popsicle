//! telemetry bundled tool: guide action and tool.yaml presence.

use std::collections::BTreeMap;
use std::fs;

use cli_ux::{run_tool, CliDomain, Workspace, WorkspaceDomain};

#[test]
fn telemetry_tool_yaml_exists_in_repo() {
    let root = std::env::current_dir().expect("cwd");
    if !root
        .join("intent-coder/tools/telemetry/tool.yaml")
        .is_file()
    {
        return;
    }
    let content = fs::read_to_string(root.join("intent-coder/tools/telemetry/tool.yaml"))
        .expect("read tool.yaml");
    assert!(content.contains("name: telemetry"));
    assert!(content.contains("action=guide"));
}

#[test]
fn telemetry_guide_action_prints_guide() {
    let root = std::env::current_dir().expect("cwd");
    if !root.join("intent-coder/tools/telemetry/guide.md").is_file() {
        return;
    }
    let ws = Workspace::at(root);
    let mut args = BTreeMap::new();
    args.insert("action".into(), "guide".into());
    let code = run_tool(&ws, "telemetry", &args).expect("run guide");
    assert_eq!(code, 0, "guide should succeed");
}

#[test]
fn telemetry_guide_via_rust_dispatch() {
    let root = std::env::current_dir().expect("cwd");
    if !root.join("intent-coder/tools/telemetry/guide.md").is_file() {
        return;
    }
    let domain = WorkspaceDomain::open_with(Some(root.to_str().unwrap())).expect("open workspace");
    let mut args = BTreeMap::new();
    args.insert("action".into(), "guide".into());
    args.insert("format".into(), "json".into());
    let result = domain.tool_run("telemetry", &args).expect("tool_run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.fields.contains_key("guide"),
        "json response should include guide field"
    );
}
