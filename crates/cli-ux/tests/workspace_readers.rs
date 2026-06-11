//! Unit tests for workspace_readers (PROJ-27 UI viz).

use std::path::PathBuf;

use cli_ux::workspace_readers::{scan_tasks, task_graph_mermaid};

#[test]
fn scan_cli_ux_tasks_non_empty() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../products");
    let graph = scan_tasks(&root).expect("scan_tasks");
    assert!(
        !graph.nodes.is_empty(),
        "expected tasks under products/*/tasks"
    );
    let ids: Vec<_> = graph.nodes.iter().map(|n| n.task_id.as_str()).collect();
    assert!(
        ids.iter().any(|id| id.starts_with("T-CU-")),
        "expected cli-ux task ids, got {ids:?}"
    );
}

#[test]
fn task_graph_mermaid_contains_edges() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../products");
    let graph = scan_tasks(&root).expect("scan_tasks");
    let mm = task_graph_mermaid(&graph);
    assert!(mm.starts_with("flowchart"));
    if graph.nodes.iter().any(|n| !n.related_next_tasks.is_empty()) {
        assert!(mm.contains("-->"), "mermaid should include edges");
    }
}
