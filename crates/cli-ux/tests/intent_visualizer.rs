//! Intent visualizer integration (requires `ui` feature).

use std::path::PathBuf;

use cli_ux::workspace_readers::scan_intents;

#[test]
fn scan_intents_uses_visualizer_for_cli_ux() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let graph = scan_intents(&ws, "cli-ux").expect("scan_intents");
    assert_eq!(graph.source, "visualizer");
    assert!(!graph.diagrams.is_empty(), "expected at least one diagram");
    let goal = graph
        .diagrams
        .iter()
        .find(|d| d.id == "goal-graph")
        .expect("goal-graph");
    assert!(goal.mermaid.contains("graph TD"));
    assert!(
        goal.mermaid.contains("IssueStartCreatesRun") || goal.mermaid.contains("intentNode"),
        "goal graph should include intent nodes"
    );
    assert!(graph.mermaid.is_some());
}

#[test]
fn artifact_system_goal_graph_ids_are_ascii() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let graph = scan_intents(&ws, "artifact-system").expect("scan_intents");
    let goal = graph
        .diagrams
        .iter()
        .find(|d| d.id == "goal-graph")
        .expect("goal-graph");
    assert!(
        !goal.mermaid.contains("guard_upstream_判定经")
            && !goal.mermaid.contains("ContextLayer_运行时")
    );
    assert!(goal.mermaid.contains("n0["));
}
