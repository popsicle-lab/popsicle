//! Unit tests for workspace_readers (PROJ-27 UI viz).

use std::path::PathBuf;

use cli_ux::workspace_readers::{
    guidance_for_issue, list_products, product_for_spec, read_intent_file, read_task, scan_tasks,
    task_graph_mermaid,
};

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

#[test]
fn product_for_spec_maps_slice_to_cli_ux() {
    let products = vec!["cli-ux".into(), "skill-runtime".into()];
    assert_eq!(
        product_for_spec("slice-3-cli-ux", &products).as_deref(),
        Some("cli-ux")
    );
}

#[test]
fn read_task_t_cu_0002_has_body() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let task = read_task(&ws, "T-CU-0002", Some("cli-ux")).expect("read_task");
    assert_eq!(task.task_id, "T-CU-0002");
    assert!(task.body.contains("issue create"));
    assert!(task.frontmatter.contains_key("journey_stage"));
}

#[test]
fn read_intent_acceptance_has_blocks() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let file = read_intent_file(&ws, "cli-ux", "acceptance.intent").expect("read_intent");
    assert!(file.content.contains("intent IssueStartCreatesRun"));
    assert!(
        file.blocks.iter().any(|b| b.name == "IssueStartCreatesRun"),
        "expected IssueStartCreatesRun block"
    );
}

#[test]
fn guidance_for_issue_recommends_tasks() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let products = list_products(&ws).expect("list_products");
    let product = product_for_spec("slice-3-cli-ux", &products);
    let g = guidance_for_issue(
        &ws,
        "cli-ux",
        "technical",
        "in_progress",
        Some("implement"),
        &[],
    )
    .expect("guidance");
    assert_eq!(g.product, product);
    assert!(!g.recommended_tasks.is_empty());
}
