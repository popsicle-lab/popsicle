//! Product health scanner tests.

use cli_ux::workspace_readers::scan_product_health;

#[test]
fn scan_cli_ux_product_health_ok() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let report = scan_product_health(root, "cli-ux").expect("health scan");
    assert!(report.task_count > 0);
    assert!(report.intent_block_count > 0);
    assert!(report.has_product_md);
    assert!(!report.journey_stages.is_empty());
}
