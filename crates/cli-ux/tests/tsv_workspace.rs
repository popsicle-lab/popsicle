//! Integration tests for the real TSV workspace backend (not mock helpers).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cli_ux::{TsvWorkspace, Workspace};
use storage::WorkspaceStore;

fn temp_workspace() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("popsicle-tsv-test-{nanos}"));
    write_pipeline(
        &root,
        "test-open",
        r#"name: test-open
description: one stage without approval
stages:
  - name: implement
    skill: shadow-implementer
    description: open stage
    requires_approval: false
"#,
    );
    write_pipeline(
        &root,
        "test-gated",
        r#"name: test-gated
description: one stage with approval
stages:
  - name: review
    skill: shadow-implementer
    description: gated stage
    requires_approval: true
"#,
    );
    root
}

fn write_pipeline(root: &Path, name: &str, content: &str) {
    let dir = root.join(".popsicle/pipelines");
    fs::create_dir_all(&dir).expect("create pipeline dir");
    fs::write(dir.join(format!("{name}.pipeline.yaml")), content).expect("write pipeline");
}

#[test]
fn tsv_roundtrip_persists_issue_and_doc() {
    let root = temp_workspace();
    let mut store = TsvWorkspace::open_at(root.clone()).expect("open workspace");
    store.init().expect("init");

    let issue = store
        .create_issue(
            "bug",
            "roundtrip",
            "slice-3-cli-ux",
            Some("test-open"),
            "medium",
            "desc",
        )
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-open")
        .expect("start issue");
    let doc = store
        .create_doc("shadow-implementer", "artifact", &run.run_id)
        .expect("create doc");

    drop(store);
    let reloaded = TsvWorkspace::open_at(root.clone()).expect("reload workspace");
    let issues = reloaded.list_issues().expect("list issues");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].key, issue.key);

    let docs = reloaded
        .list_docs(Some(&run.run_id))
        .expect("list docs for run");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].id, doc.doc_id);
    assert!(root.join(&doc.file_path).is_file());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_issue_close_requires_completed_run() {
    let root = temp_workspace();
    let mut store = TsvWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue("bug", "close me", "slice-3-cli-ux", Some("test-open"), "medium", "")
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-open")
        .expect("start issue");

    let err = store.close_issue(&issue.key).unwrap_err();
    assert!(format!("{err}").contains("active-run"));

    store
        .complete_stage("implement", &run.run_id, false)
        .expect("complete stage");
    let closed = store.close_issue(&issue.key).expect("close issue");
    assert_eq!(closed.status, "done");

    drop(store);
    let reloaded = TsvWorkspace::open_at(root.clone()).expect("reload");
    assert_eq!(reloaded.get_issue(&issue.key).expect("get").status, "done");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_doc_check_fails_stub_and_passes_filled_doc() {
    let root = temp_workspace();
    let mut store = TsvWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue("bug", "doc check", "slice-3-cli-ux", Some("test-open"), "medium", "")
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-open")
        .expect("start issue");
    let doc = store
        .create_doc("shadow-implementer", "artifact", &run.run_id)
        .expect("create doc");

    // Fresh stub: frontmatter ok, body is just the heading.
    let stub = store.check_doc(&doc.doc_id).expect("check stub");
    assert!(stub.file_exists);
    assert!(stub.frontmatter_complete);
    assert!(!stub.body_filled);
    assert!(!stub.passed);

    // Placeholders keep the check failing even with prose.
    let abs = root.join(&doc.file_path);
    let content = fs::read_to_string(&abs).expect("read doc");
    fs::write(&abs, format!("{content}\nSome prose. [TBD: fill]\n")).expect("write");
    let with_placeholder = store.check_doc(&doc.doc_id).expect("check placeholder");
    assert!(with_placeholder.body_filled);
    assert_eq!(with_placeholder.placeholder_count, 1);
    assert!(!with_placeholder.passed);

    // Real content with checkboxes passes and reports counts.
    let content = fs::read_to_string(&abs).expect("read doc");
    fs::write(
        &abs,
        content.replace("[TBD: fill]", "done.\n\n- [x] fixed\n- [ ] follow-up"),
    )
    .expect("write");
    let filled = store.check_doc(&doc.doc_id).expect("check filled");
    assert!(filled.passed);
    assert_eq!(filled.checkboxes_total, 2);
    assert_eq!(filled.checkboxes_checked, 1);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_pipeline_status_uses_stable_status_strings() {
    let root = temp_workspace();
    let mut store = TsvWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "bug",
            "status strings",
            "slice-3-cli-ux",
            Some("test-open"),
            "medium",
            "",
        )
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-open")
        .expect("start issue");

    let status = store.pipeline_status(&run.run_id).expect("pipeline status");
    assert_eq!(status.run_status, "in_progress");
    assert_eq!(status.stages[0]["status"], "in_progress");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_gated_stage_requires_confirm_but_open_stage_does_not() {
    let root = temp_workspace();
    let mut store = TsvWorkspace::open_at(root.clone()).expect("open workspace");

    let open_issue = store
        .create_issue(
            "bug",
            "open stage",
            "slice-3-cli-ux",
            Some("test-open"),
            "medium",
            "",
        )
        .expect("create open issue");
    let open_run = store
        .start_issue(&open_issue.key, "", "test-open")
        .expect("start open run");
    let open_stage = store
        .pipeline_status(&open_run.run_id)
        .expect("open status")
        .current_stage;
    store
        .complete_stage(&open_stage, &open_run.run_id, false)
        .expect("complete open stage without confirm");

    let gated_issue = store
        .create_issue(
            "bug",
            "gated stage",
            "slice-3-cli-ux",
            Some("test-gated"),
            "medium",
            "",
        )
        .expect("create gated issue");
    let gated_run = store
        .start_issue(&gated_issue.key, "", "test-gated")
        .expect("start gated run");
    let gated_stage = store
        .pipeline_status(&gated_run.run_id)
        .expect("gated status")
        .current_stage;
    let err = store
        .complete_stage(&gated_stage, &gated_run.run_id, false)
        .expect_err("gated stage should require confirm");
    assert!(err.to_string().contains("lock:"));
    store
        .complete_stage(&gated_stage, &gated_run.run_id, true)
        .expect("complete gated stage with confirm");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fresh_workspace_bootstrap_installs_pipelines_and_numbers_from_one() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("popsicle-bootstrap-test-{nanos}"));
    fs::create_dir_all(&root).expect("create fresh dir");

    let workspace = Workspace::at(root.clone());
    let mut store = TsvWorkspace::open_at(root.clone()).expect("open fresh workspace");
    store.init().expect("init fresh workspace");
    let installed = workspace
        .install_bundled_pipelines()
        .expect("install bundled pipelines");
    assert!(installed.contains(&"greenfield-product-spec"));
    assert!(installed.contains(&"migration-bootstrap"));

    // Second install is a no-op (existing files are preserved).
    let second = workspace
        .install_bundled_pipelines()
        .expect("reinstall is idempotent");
    assert!(second.is_empty());

    let issue = store
        .create_issue(
            "technical",
            "first issue in fresh workspace",
            "new-proj-slice-1",
            Some("migration-bootstrap"),
            "medium",
            "",
        )
        .expect("create first issue");
    assert_eq!(issue.key, "PROJ-1");

    // Bundled pipeline is loadable end-to-end via start_issue.
    let run = store
        .start_issue(&issue.key, "", "migration-bootstrap")
        .expect("start with bundled pipeline");
    let status = store.pipeline_status(&run.run_id).expect("status");
    assert_eq!(status.pipeline_name, "migration-bootstrap");
    assert_eq!(status.current_stage, "init");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_start_issue_rejects_duplicate_active_run_and_spec_mismatch() {
    let root = temp_workspace();
    let mut store = TsvWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "bug",
            "guards",
            "slice-3-cli-ux",
            Some("test-open"),
            "medium",
            "",
        )
        .expect("create issue");

    let mismatch = store.start_issue(&issue.key, "other-spec", "test-open");
    assert!(mismatch.is_err());
    assert!(mismatch.unwrap_err().to_string().contains("spec-lock"));

    let first = store
        .start_issue(&issue.key, "", "test-open")
        .expect("first start");

    let duplicate = store.start_issue(&issue.key, "", "test-open");
    assert!(duplicate.is_err());
    assert!(duplicate.unwrap_err().to_string().contains("active-run"));

    let reloaded = TsvWorkspace::open_at(root.clone()).expect("reload");
    let shown = reloaded.get_issue(&issue.key).expect("get issue");
    assert_eq!(shown.status, "in_progress");
    assert_eq!(shown.spec_id, "slice-3-cli-ux");

    let runs = reloaded.run_ids_for_issue(&issue.key);
    assert_eq!(runs, vec![first.run_id]);

    let _ = fs::remove_dir_all(root);
}
