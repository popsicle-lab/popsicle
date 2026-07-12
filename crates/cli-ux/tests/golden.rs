//! Golden semantic baselines for the `cli-ux` slice.

use cli_ux::{
    complete_pipeline_stage, contains_removed_top_level_command, create_document_artifact,
    parse_args, start_issue_run, top_level_help,
};
use storage::MemoryDocumentStore;

#[test]
fn golden_001_help_exposes_idd_main_path_without_removed_commands() {
    let help = top_level_help();
    for command in ["init", "issue", "pipeline", "doc", "admin", "doctor"] {
        assert!(help.lines().any(|line| line == command));
    }
    assert!(!contains_removed_top_level_command(&help));
}

#[test]
fn golden_002_issue_start_returns_run_id_and_lock_signal() {
    let result = start_issue_run("PROJ-7", "slice-3-cli-ux", "slice-delivery", "run-7").unwrap();
    assert!(result.run_created);
    assert!(result.spec_locked);
    assert!(result.has_run_id);
    assert_eq!(result.run_id, "run-7");
}

#[test]
fn golden_003_doc_create_writes_artifact_and_document_row() {
    let mut store = MemoryDocumentStore::new();
    let result = create_document_artifact(
        &mut store,
        "doc-7",
        "shadow-implementer",
        "cli-ux implementation coverage",
        "run-7",
    )
    .unwrap();
    assert!(result.artifact_file_exists);
    assert!(result.document_row_exists);
    assert_eq!(store.len(), 1);
}

#[test]
fn golden_004_stage_complete_requires_confirm_then_advances() {
    let err = complete_pipeline_stage("current", "run-7", false).unwrap_err();
    assert_eq!(err.category, "lock");
    assert!(err.next_step.contains("--confirm"));

    let result = complete_pipeline_stage("current", "run-7", true).unwrap();
    assert!(result.previous_completed);
    assert!(result.downstream_ready);
    assert!(result.status_reflects_state);
}

#[test]
fn golden_005_admin_commands_are_nested_under_admin() {
    assert!(parse_args(["admin", "migrate", "--workspace", "/repo"]).is_ok());
    assert!(parse_args(["admin", "reinit", "--workspace", "/repo"]).is_ok());
    assert!(parse_args(["admin", "sync-intent-coder"]).is_ok());
    assert!(parse_args(["migrate"]).is_err());
    assert!(parse_args(["reinit"]).is_err());
}

#[test]
fn golden_006_removed_commands_return_actionable_errors() {
    for command in ["checklist", "item", "sync"] {
        let err = parse_args([command]).unwrap_err();
        assert_eq!(err.category, "invalid-args");
        assert_eq!(err.object_ref, command);
        assert!(err.has_category_object_and_next_step());
    }
}

// PROJ-17 command surface alignment goldens.

#[test]
fn golden_007_deferred_commands_return_actionable_errors() {
    for command in cli_ux::DEFERRED_TOP_LEVEL_COMMANDS {
        let err = parse_args([*command]).unwrap_err();
        assert_eq!(err.category, "deferred");
        assert_eq!(err.object_ref, *command);
        assert!(err.has_category_object_and_next_step());
    }
}

#[test]
fn golden_008_format_flag_is_global() {
    use cli_ux::Command;
    assert_eq!(
        parse_args(["issue", "list", "--format", "json"]).unwrap(),
        Command::IssueList
    );
    assert!(matches!(
        parse_args(["pipeline", "next", "--run", "run-1", "--format", "json"]).unwrap(),
        Command::PipelineNext { .. }
    ));
    assert!(matches!(
        parse_args(["--format", "json"]).unwrap(),
        Command::Help
    ));
}

#[test]
fn golden_008b_help_flag_works_in_any_position() {
    use cli_ux::Command;
    // `--help` / `-h` must win over required-arg validation and subcommand
    // routing so users can always discover usage (feedback #5).
    for argv in [
        vec!["issue", "create", "--help"],
        vec!["issue", "start", "--help"],
        vec!["issue", "--help"],
        vec!["pipeline", "--help"],
        vec!["doc", "-h"],
    ] {
        assert_eq!(
            parse_args(argv.clone()).unwrap(),
            Command::Help,
            "{argv:?} should route to help"
        );
    }
}

#[test]
fn golden_008c_version_flag_is_recognized() {
    use cli_ux::Command;
    for argv in [
        vec!["--version"],
        vec!["-V"],
        vec!["version"],
        vec!["issue", "--version"],
    ] {
        assert_eq!(
            parse_args(argv.clone()).unwrap(),
            Command::Version,
            "{argv:?} should route to version"
        );
    }
    let response = cli_ux::version_response();
    assert_eq!(
        response.fields.get("version").map(String::as_str),
        Some(env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn golden_008d_migration_bootstrap_allows_missing_product() {
    use cli_ux::Command;
    // migration-bootstrap creates products in its init stage, so --product may
    // be omitted (feedback #3).
    match parse_args([
        "issue",
        "create",
        "--type",
        "technical",
        "--title",
        "Bootstrap repo",
        "--pipeline",
        "migration-bootstrap",
    ])
    .unwrap()
    {
        Command::IssueCreate {
            product_id,
            pipeline,
            ..
        } => {
            assert_eq!(product_id, "");
            assert_eq!(pipeline.as_deref(), Some("migration-bootstrap"));
        }
        other => panic!("expected IssueCreate, got {other:?}"),
    }

    // Any other pipeline still requires --product.
    let err = parse_args([
        "issue",
        "create",
        "--type",
        "technical",
        "--title",
        "x",
        "--pipeline",
        "feature-spec",
    ])
    .unwrap_err();
    assert_eq!(err.object_ref, "product");
}

#[test]
fn golden_010_issue_type_default_pipelines_are_bundled() {
    use skill_runtime::IssueType;
    let bundled = cli_ux::bundled_pipeline_names();
    for issue_type in [
        IssueType::Product,
        IssueType::Technical,
        IssueType::Bug,
        IssueType::Idea,
    ] {
        let default = issue_type
            .default_pipeline()
            .expect("every issue type needs a default pipeline");
        assert!(
            bundled.iter().any(|n| n == default),
            "default pipeline `{default}` for {issue_type:?} is not bundled"
        );
    }
}

#[test]
fn golden_011_doc_check_and_issue_close_parse() {
    use cli_ux::Command;
    assert_eq!(
        parse_args(["doc", "check", "doc-1"]).unwrap(),
        Command::DocCheck {
            doc_id: "doc-1".into()
        }
    );
    assert_eq!(
        parse_args(["issue", "close", "PROJ-1"]).unwrap(),
        Command::IssueClose {
            key: "PROJ-1".into()
        }
    );
}

#[test]
fn golden_009_help_advertises_only_implemented_commands() {
    let help = top_level_help();
    for deferred in cli_ux::DEFERRED_TOP_LEVEL_COMMANDS {
        assert!(
            !help.lines().any(|line| line.trim() == *deferred),
            "help must not advertise deferred command `{deferred}`"
        );
    }
    let response = cli_ux::help_response();
    assert!(response.fields.contains_key("usage"));
    assert!(response.fields.contains_key("deferred_commands"));
    // Every advertised top-level command must lead somewhere in the parser:
    // either parse on its own or fail asking for a subcommand (not "unknown").
    for command in cli_ux::TOP_LEVEL_COMMANDS {
        if let Err(err) = parse_args([*command]) {
            assert_ne!(
                err.category, "deferred",
                "advertised command `{command}` must not be deferred"
            );
        }
    }
}
