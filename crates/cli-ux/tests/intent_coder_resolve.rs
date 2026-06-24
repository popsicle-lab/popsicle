//! Tests for unified intent-coder path resolution.

use cli_ux::build_workflow_catalog;
use cli_ux::{install_intent_coder_module, Workspace};

#[test]
fn workflow_catalog_uses_live_intent_coder_pipelines_in_dogfood_layout() {
    let root = std::env::temp_dir().join(format!(
        "popsicle-wf-live-pipe-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("intent-coder/pipelines")).unwrap();
    std::fs::create_dir_all(root.join(".popsicle/pipelines")).unwrap();
    std::fs::create_dir_all(root.join(".popsicle")).unwrap();
    std::fs::write(
        root.join("intent-coder/module.yaml"),
        "name: intent-coder\nversion: \"9.9.9\"\n",
    )
    .unwrap();
    let ws = Workspace::at(root.clone());
    ws.install_bundled_pipelines().unwrap();
    install_intent_coder_module(&ws, false).unwrap();

    let live_fix =
        std::fs::read_to_string(root.join(".popsicle/pipelines/fix-regression.pipeline.yaml"))
            .unwrap()
            .replace("单点回归", "LIVE FIX MARKER");
    std::fs::write(
        root.join("intent-coder/pipelines/fix-regression.pipeline.yaml"),
        live_fix,
    )
    .unwrap();

    let cat = build_workflow_catalog(&ws).unwrap();
    let fix = cat
        .pipelines
        .iter()
        .find(|p| p.name == "fix-regression")
        .expect("fix-regression pipeline");
    assert!(
        fix.description.contains("LIVE FIX MARKER"),
        "catalog should prefer live intent-coder/pipelines over .popsicle/pipelines"
    );
}
