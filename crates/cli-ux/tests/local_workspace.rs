//! Integration tests for the real local workspace backend (not mock helpers).
//! Fresh temp workspaces exercise the SQLite Phase 2 default (PROJ-11);
//! dedicated tests cover TSV legacy compatibility and migration.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cli_ux::{LocalWorkspace, StateBackend, Workspace};
use storage::WorkspaceStore;

fn temp_workspace() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEQ: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("popsicle-local-test-{nanos}-{seq}"));
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
    fs::create_dir_all(root.join("products/cli-ux")).expect("product dir");
    fs::create_dir_all(root.join("products/other-product")).expect("product dir");
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

fn write_approval_mode(root: &Path, mode: &str) {
    fs::create_dir_all(root.join(".popsicle")).expect("popsicle dir");
    fs::write(
        root.join(".popsicle/project.yaml"),
        format!("workflow:\n  approval_mode: {mode}\n"),
    )
    .expect("write project.yaml");
}

#[test]
fn tsv_roundtrip_persists_issue_and_doc() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    store.init().expect("init");

    let issue = store
        .create_issue(
            "bug",
            "roundtrip",
            "cli-ux",
            Some("test-open"),
            "medium",
            "desc",
            None,
            &[],
            &[],
        )
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-open")
        .expect("start issue");
    let doc = store
        .create_doc("shadow-implementer", "artifact", &run.run_id)
        .expect("create doc");

    drop(store);
    let reloaded = LocalWorkspace::open_at(root.clone()).expect("reload workspace");
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
fn fresh_workspace_defaults_to_sqlite_backend() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    assert_eq!(store.backend(), StateBackend::Sqlite);
    store
        .create_issue(
            "bug",
            "sqlite native",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create issue");
    assert!(root.join(".popsicle/state.db").is_file());
    assert!(!root.join(".popsicle/state.tsv").is_file());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn legacy_tsv_imports_to_sqlite_on_open() {
    let root = temp_workspace();
    let dir = root.join(".popsicle/self-host");
    fs::create_dir_all(&dir).expect("layout");
    fs::write(
        dir.join("state.tsv"),
        "meta\tnext_issue_num\t2\nmeta\tnext_run_num\t1\nmeta\tnext_doc_num\t1\nissue\tPROJ-1\tbug\tmedium\topen\tlegacy issue\tcli-ux\ttest-open\tdesc\n",
    )
    .expect("write tsv");

    let store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    assert_eq!(store.backend(), StateBackend::Sqlite);
    assert_eq!(
        store.get_issue("PROJ-1").expect("legacy issue").title,
        "legacy issue"
    );
    assert!(root.join(".popsicle/state.db").is_file());
    assert!(root.join(".popsicle/state.tsv.migrated").is_file());
    assert!(!root.join(".popsicle/state.tsv").is_file());
    assert!(!dir.exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn legacy_self_host_layout_relocates_on_open() {
    let root = temp_workspace();
    {
        let mut store = LocalWorkspace::open_at(root.clone()).expect("seed workspace");
        store
            .create_issue(
                "bug",
                "relocate me",
                "cli-ux",
                Some("test-open"),
                "medium",
                "",
                None,
                &[],
                &[],
            )
            .expect("create issue");
    }
    let legacy = root.join(".popsicle/self-host");
    fs::create_dir_all(&legacy).expect("legacy dir");
    fs::rename(root.join(".popsicle/state.db"), legacy.join("state.db")).expect("move db");
    if root.join(".popsicle/runs").is_dir() {
        fs::rename(root.join(".popsicle/runs"), legacy.join("runs")).expect("move runs");
    }

    let store = LocalWorkspace::open_at(root.clone()).expect("open relocates layout");
    assert!(root.join(".popsicle/state.db").is_file());
    assert!(!legacy.exists());
    assert_eq!(store.list_issues().expect("issues").len(), 1);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn migrate_to_sqlite_preserves_rows_and_is_idempotent() {
    let root = temp_workspace();
    let dir = root.join(".popsicle/self-host");
    fs::create_dir_all(&dir).expect("layout");
    fs::write(
        dir.join("state.tsv"),
        "meta\tnext_issue_num\t3\nmeta\tnext_run_num\t1\nmeta\tnext_doc_num\t1\nissue\tPROJ-1\tbug\tmedium\topen\tfirst\tslice-3-cli-ux\ttest-open\t\nissue\tPROJ-2\ttechnical\thigh\tdone\tsecond\tslice-3-cli-ux\t\t\n",
    )
    .expect("write tsv");

    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    // open_at auto-imports TSV; migrate is idempotent cleanup.
    assert_eq!(store.backend(), StateBackend::Sqlite);
    assert!(!store.migrate_to_sqlite().expect("idempotent migrate"));
    assert!(root.join(".popsicle/state.db").is_file());
    assert!(root.join(".popsicle/state.tsv.migrated").is_file());
    assert!(!root.join(".popsicle/state.tsv").is_file());
    assert!(!dir.exists());

    drop(store);
    let mut reloaded = LocalWorkspace::open_at(root.clone()).expect("reload");
    assert_eq!(reloaded.backend(), StateBackend::Sqlite);
    assert_eq!(reloaded.list_issues().expect("list").len(), 2);
    assert_eq!(reloaded.get_issue("PROJ-2").expect("issue").status, "done");
    // Counter preserved: next issue gets PROJ-3.
    let next = reloaded
        .create_issue(
            "bug",
            "post-migration",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create issue");
    assert_eq!(next.key, "PROJ-3");
    // Idempotent: second migrate reports false.
    assert!(!reloaded.migrate_to_sqlite().expect("re-migrate"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_issue_close_requires_completed_run() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "bug",
            "close me",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
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
    let reloaded = LocalWorkspace::open_at(root.clone()).expect("reload");
    assert_eq!(reloaded.get_issue(&issue.key).expect("get").status, "done");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_doc_check_fails_stub_and_passes_filled_doc() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "bug",
            "doc check",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
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

/// Regression: `inject_on_run` must not write the multi-line project context
/// (which may contain `---`) into the artifact frontmatter — it prematurely
/// closed the line-oriented YAML frontmatter and made `doc check` report
/// `frontmatter_complete:false`. The context is now returned in the CLI
/// response instead, and the artifact frontmatter stays parseable.
#[test]
fn inject_on_run_keeps_artifact_frontmatter_parseable() {
    use cli_ux::project_config::ensure_project_config;
    use cli_ux::project_context::save_project_context;

    let root = temp_workspace();
    ensure_project_config(&root).expect("project config (inject_on_run default true)");
    // PROJECT_CONTEXT.md with a `---` horizontal rule inside the engineering profile.
    save_project_context(
        &root,
        "# Project Context\n\n## 工程画像\n\nRust workspace.\n\n---\n\nMore facts.\n",
    )
    .expect("save project context");

    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "bug",
            "inject frontmatter",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-open")
        .expect("start issue");
    let doc = store
        .create_doc("shadow-implementer", "artifact", &run.run_id)
        .expect("create doc");

    // The multi-line preferences are surfaced in the CLI response, not the file.
    assert!(doc.agent_context.contains("[Project context]"));
    assert!(doc.agent_context.contains("Rust workspace"));

    let content = fs::read_to_string(root.join(&doc.file_path)).expect("read artifact");
    assert!(
        !content.contains("[Project context]"),
        "injected block must not be written into the artifact: {content}"
    );

    // Frontmatter stays intact: fresh stub parses cleanly (just needs a body).
    let stub = store.check_doc(&doc.doc_id).expect("check stub");
    assert!(
        stub.frontmatter_complete,
        "frontmatter should parse even with `---` in PROJECT_CONTEXT.md"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_pipeline_status_uses_stable_status_strings() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "bug",
            "status strings",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
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
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");

    let open_issue = store
        .create_issue(
            "bug",
            "open stage",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
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
            "cli-ux",
            Some("test-gated"),
            "medium",
            "",
            None,
            &[],
            &[],
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
fn auto_approval_mode_completes_gated_stage_without_confirm() {
    let root = temp_workspace();
    write_approval_mode(&root, "auto");
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");

    let issue = store
        .create_issue(
            "bug",
            "auto gated",
            "cli-ux",
            Some("test-gated"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-gated")
        .expect("start run");
    let stage = store
        .pipeline_status(&run.run_id)
        .expect("status")
        .current_stage;
    store
        .complete_stage(&stage, &run.run_id, false)
        .expect("auto mode completes gated stage without confirm");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn delegate_dangerous_mode_splits_gated_and_cutover() {
    let root = temp_workspace();
    write_approval_mode(&root, "delegate-dangerous");
    write_pipeline(
        &root,
        "test-cutover",
        r#"name: test-cutover
description: dangerous gated stage
stages:
  - name: cutover
    skill: cutover-author
    description: cutover gate
    requires_approval: true
"#,
    );
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");

    let review_issue = store
        .create_issue(
            "bug",
            "delegate review",
            "cli-ux",
            Some("test-gated"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create review issue");
    let review_run = store
        .start_issue(&review_issue.key, "", "test-gated")
        .expect("start review run");
    let review_stage = store
        .pipeline_status(&review_run.run_id)
        .expect("review status")
        .current_stage;
    store
        .complete_stage(&review_stage, &review_run.run_id, false)
        .expect("delegate completes non-dangerous gated stage");

    let cutover_issue = store
        .create_issue(
            "bug",
            "delegate cutover",
            "cli-ux",
            Some("test-cutover"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create cutover issue");
    let cutover_run = store
        .start_issue(&cutover_issue.key, "", "test-cutover")
        .expect("start cutover run");
    let cutover_stage = store
        .pipeline_status(&cutover_run.run_id)
        .expect("cutover status")
        .current_stage;
    let err = store
        .complete_stage(&cutover_stage, &cutover_run.run_id, false)
        .expect_err("cutover still requires confirm in delegate mode");
    assert!(err.to_string().contains("lock:"));
    store
        .complete_stage(&cutover_stage, &cutover_run.run_id, true)
        .expect("cutover with confirm");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn auto_mode_cannot_bypass_a_failing_machine_gate() {
    // feedback #19/P4/H6: the machine gate axis runs in EVERY approval_mode.
    let root = temp_workspace();
    write_approval_mode(&root, "auto");
    write_pipeline(
        &root,
        "test-gate-fail",
        r#"name: test-gate-fail
description: stage with a failing machine gate
stages:
  - name: implement
    skill: shadow-implementer
    description: gated by a command that fails
    requires_approval: false
    gate:
      - command_exit_zero: "exit 7"
"#,
    );
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "technical",
            "gate fail",
            "cli-ux",
            Some("test-gate-fail"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-gate-fail")
        .expect("start run");
    let stage = store
        .pipeline_status(&run.run_id)
        .expect("status")
        .current_stage;
    let err = store
        .complete_stage(&stage, &run.run_id, false)
        .expect_err("auto mode must NOT bypass a failing gate");
    let msg = err.to_string();
    assert!(msg.contains("gate:"), "expected gate error, got: {msg}");
    assert!(msg.contains("command_exit_zero"), "detail missing: {msg}");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn passing_machine_gate_allows_completion() {
    let root = temp_workspace();
    write_approval_mode(&root, "auto");
    write_pipeline(
        &root,
        "test-gate-pass",
        r#"name: test-gate-pass
description: stage with a passing machine gate
stages:
  - name: implement
    skill: shadow-implementer
    description: gated by a command that passes
    requires_approval: false
    gate:
      - command_exit_zero: "true"
"#,
    );
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "technical",
            "gate pass",
            "cli-ux",
            Some("test-gate-pass"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create issue");
    let run = store
        .start_issue(&issue.key, "", "test-gate-pass")
        .expect("start run");
    let stage = store
        .pipeline_status(&run.run_id)
        .expect("status")
        .current_stage;
    store
        .complete_stage(&stage, &run.run_id, false)
        .expect("passing gate completes the stage");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fresh_workspace_bootstrap_installs_pipelines_and_numbers_from_one() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("popsicle-bootstrap-test-{nanos}"));
    fs::create_dir_all(root.join("products/demo-proj")).expect("create fresh dir");

    let workspace = Workspace::at(root.clone());
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open fresh workspace");
    store.init().expect("init fresh workspace");
    let installed = workspace
        .install_bundled_pipelines()
        .expect("install bundled pipelines");
    assert!(installed.iter().any(|n| n == "product-greenfield-spec"));
    assert!(installed.iter().any(|n| n == "migration-bootstrap"));

    // Second install is a no-op (existing files are preserved).
    let second = workspace
        .install_bundled_pipelines()
        .expect("reinstall is idempotent");
    assert!(second.is_empty());

    let issue = store
        .create_issue(
            "technical",
            "first issue in fresh workspace",
            "demo-proj",
            Some("migration-bootstrap"),
            "medium",
            "",
            None,
            &[],
            &[],
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

fn write_slice_delivery_gate_product(root: &Path) {
    let task_dir = root.join("products/cli-ux/tasks/daily-ops");
    fs::create_dir_all(&task_dir).expect("task dir");
    fs::create_dir_all(root.join("products/cli-ux/intents")).expect("intents");
    fs::write(
        task_dir.join("T-CU-0098-gate-local.md"),
        r#"---
task_id: T-CU-0098
title: "local gate task"
journey_stage: daily-ops
related_intents:
  - acceptance.intent#LocalGateIntent
---
# local gate
"#,
    )
    .expect("task");
    fs::write(
        root.join("products/cli-ux/intents/acceptance.intent"),
        r#"type LocalGateResult { ok: Bool }
intent LocalGateIntent(r: LocalGateResult) {
  require true
  ensure r.ok' == true
}
"#,
    )
    .expect("intent");
    write_pipeline(
        root,
        "slice-delivery",
        r#"name: slice-delivery
description: gate test
stages:
  - name: implement
    skill: shadow-implementer
    description: impl
    requires_approval: false
"#,
    );
}

#[test]
fn bugfix_create_rejects_intent_spec_mismatch() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open");
    let err = store
        .create_issue(
            "product",
            "补 realized_by",
            "cli-ux",
            Some("bugfix"),
            "medium",
            "改 products/cli-ux/intents/contracts.intent",
            None,
            &[],
            &[],
        )
        .expect_err("expected bugfix-gate");
    let msg = err.to_string();
    assert!(
        msg.contains("fix-regression-gate:product-type")
            || msg.contains("fix-regression-gate:intent-content")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn slice_delivery_create_rejects_proposed_with_delivery_pipeline() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open");
    let err = store.create_issue(
        "technical",
        "bad combo",
        "cli-ux",
        Some("slice-delivery"),
        "medium",
        "desc",
        None,
        &[],
        &[("new thing".into(), Some("daily-ops".into()))],
    );
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("create-proposed"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn slice_delivery_start_requires_task_id_in_description() {
    let root = temp_workspace();
    write_slice_delivery_gate_product(&root);
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open");
    let issue = store
        .create_issue(
            "technical",
            "gate",
            "cli-ux",
            Some("slice-delivery"),
            "medium",
            "no task id in body",
            None,
            &["T-CU-0098"],
            &[],
        )
        .expect("create");
    let err = store.start_issue(&issue.key, "", "");
    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .to_string()
        .contains("description-missing-task"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn slice_delivery_start_passes_with_valid_links_and_description() {
    let root = temp_workspace();
    write_slice_delivery_gate_product(&root);
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open");
    let issue = store
        .create_issue(
            "technical",
            "gate ok",
            "cli-ux",
            Some("slice-delivery"),
            "medium",
            "交付 T-CU-0098 能力",
            None,
            &["T-CU-0098"],
            &[],
        )
        .expect("create");
    store
        .start_issue(&issue.key, "", "")
        .expect("start should pass gate");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn issue_link_replace_and_drop_proposed() {
    let root = temp_workspace();
    write_slice_delivery_gate_product(&root);
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open");
    let issue = store
        .create_issue(
            "technical",
            "link me",
            "cli-ux",
            Some("test-open"),
            "medium",
            "proposed only",
            None,
            &[],
            &[("New task".into(), Some("daily-ops".into()))],
        )
        .expect("create");
    let links = store
        .link_issue_tasks(&issue.key, &["T-CU-0098"], true, true)
        .expect("link");
    assert_eq!(links.iter().filter(|l| l.role == "linked").count(), 1);
    assert!(links.iter().all(|l| l.role != "proposed"));
    assert_eq!(links[0].task_id.as_deref(), Some("T-CU-0098"));
    assert_eq!(links[0].source, "issue-link");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn tsv_start_issue_rejects_duplicate_active_run_and_spec_mismatch() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");
    let issue = store
        .create_issue(
            "bug",
            "guards",
            "cli-ux",
            Some("test-open"),
            "medium",
            "",
            None,
            &[],
            &[],
        )
        .expect("create issue");

    let epic = store
        .create_issue(
            "technical",
            "epic link",
            "cli-ux",
            Some("bugfix"),
            "medium",
            "",
            Some("T-CU-0001"),
            &[],
            &[],
        )
        .expect("create epic issue");
    assert_eq!(epic.epic_task_id.as_deref(), Some("T-CU-0001"));
    let links = store.list_issue_tasks(&epic.key).expect("task links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].role, "linked");
    assert_eq!(links[0].task_id.as_deref(), Some("T-CU-0001"));

    let mismatch = store.start_issue(&issue.key, "other-product", "test-open");
    assert!(mismatch.is_err());
    assert!(mismatch.unwrap_err().to_string().contains("product-lock"));

    let first = store
        .start_issue(&issue.key, "", "test-open")
        .expect("first start");

    let duplicate = store.start_issue(&issue.key, "", "test-open");
    assert!(duplicate.is_err());
    assert!(duplicate.unwrap_err().to_string().contains("active-run"));

    let reloaded = LocalWorkspace::open_at(root.clone()).expect("reload");
    let shown = reloaded.get_issue(&issue.key).expect("get issue");
    assert_eq!(shown.status, "in_progress");
    assert_eq!(shown.product_id, "cli-ux");

    let runs = reloaded.run_ids_for_issue(&issue.key);
    assert_eq!(runs, vec![first.run_id]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn issue_tasks_multi_linked_and_proposed_persist() {
    let root = temp_workspace();
    let mut store = LocalWorkspace::open_at(root.clone()).expect("open workspace");

    let issue = store
        .create_issue(
            "technical",
            "task links",
            "cli-ux",
            None,
            "medium",
            "multi task",
            None,
            &["T-CU-0001", "T-CU-0002"],
            &[("新旅程".into(), Some("daily-ops".into()))],
        )
        .expect("create issue");

    let links = store.list_issue_tasks(&issue.key).expect("links");
    assert_eq!(links.len(), 3);
    assert_eq!(links[0].role, "linked");
    assert_eq!(links[0].task_id.as_deref(), Some("T-CU-0001"));
    assert_eq!(links[1].task_id.as_deref(), Some("T-CU-0002"));
    assert_eq!(links[2].role, "proposed");
    assert_eq!(links[2].proposed_title.as_deref(), Some("新旅程"));

    drop(store);
    let reloaded = LocalWorkspace::open_at(root.clone()).expect("reload");
    let again = reloaded.list_issue_tasks(&issue.key).expect("reload links");
    assert_eq!(again.len(), 3);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn admin_backfill_removes_deprecated_pipeline_installs() {
    use cli_ux::{parse_args, run_command, WorkspaceDomain};

    let root = temp_workspace();
    let pipelines_dir = root.join(".popsicle/pipelines");
    fs::create_dir_all(&pipelines_dir).expect("pipelines dir");
    fs::write(
        pipelines_dir.join("bugfix.pipeline.yaml"),
        "name: bugfix\nstages: []\n",
    )
    .expect("deprecated yaml");

    let mut domain = WorkspaceDomain::open_with(Some(root.to_str().unwrap())).expect("open");

    let command = parse_args(["admin", "backfill-pipeline-names", "--dry-run"]).expect("parse");
    let resp = run_command(&mut domain, command).expect("dry-run");
    assert_eq!(
        resp.fields.get("deprecated_files_removed"),
        Some(&"bugfix".to_string())
    );
    assert!(pipelines_dir.join("bugfix.pipeline.yaml").is_file());

    let command = parse_args(["admin", "backfill-pipeline-names"]).expect("parse");
    run_command(&mut domain, command).expect("apply");
    assert!(!pipelines_dir.join("bugfix.pipeline.yaml").exists());

    let _ = fs::remove_dir_all(root);
}
