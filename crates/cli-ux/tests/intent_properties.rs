//! Property tests mirroring `products/cli-ux/intents/*`.

use cli_ux::{
    complete_pipeline_stage, contains_removed_top_level_command, create_document_artifact,
    parse_args, run_command, start_issue_run, top_level_help, AdminCommand, AdminResult, CliDomain,
    CliError, Command, InitResult,
};
use storage::MemoryDocumentStore;

#[derive(Default)]
struct TestDomain {
    store: MemoryDocumentStore,
}

impl CliDomain for TestDomain {
    fn init_workspace(&mut self) -> Result<InitResult, CliError> {
        Ok(InitResult {
            workspace_ready: true,
            has_next_step: true,
            next_step: "popsicle issue create --spec <spec> --pipeline slice-spec".into(),
        })
    }

    fn start_issue(
        &mut self,
        key: &str,
        spec_id: &str,
        pipeline: &str,
    ) -> Result<cli_ux::IssueStartResult, CliError> {
        start_issue_run(key, spec_id, pipeline, "run-1")
    }

    fn create_doc(
        &mut self,
        skill: &str,
        title: &str,
        run_id: &str,
    ) -> Result<cli_ux::DocCreateResult, CliError> {
        create_document_artifact(&mut self.store, "doc-1", skill, title, run_id)
    }

    fn complete_stage(
        &mut self,
        stage: &str,
        run_id: &str,
        confirm: bool,
    ) -> Result<cli_ux::StageAdvanceResult, CliError> {
        complete_pipeline_stage(stage, run_id, confirm)
    }

    fn admin_migrate(&mut self, workspace: &str) -> Result<AdminResult, CliError> {
        Ok(AdminResult {
            under_admin_tree: true,
            explicit_workspace: !workspace.is_empty(),
            workspace: workspace.to_string(),
        })
    }

    fn admin_reinit(&mut self, workspace: &str) -> Result<AdminResult, CliError> {
        Ok(AdminResult {
            under_admin_tree: true,
            explicit_workspace: !workspace.is_empty(),
            workspace: workspace.to_string(),
        })
    }
}

#[test]
fn init_shows_next_step() {
    let mut domain = TestDomain::default();
    let response = run_command(&mut domain, Command::Init).unwrap();
    assert_eq!(response.fields["workspace_ready"], "true");
    assert_eq!(response.fields["has_next_step"], "true");
    assert!(response
        .next_step
        .unwrap()
        .contains("popsicle issue create"));
}

#[test]
fn issue_start_creates_run() {
    let mut domain = TestDomain::default();
    let command = parse_args([
        "issue",
        "start",
        "PROJ-7",
        "--spec",
        "slice-3-cli-ux",
        "--pipeline",
        "slice-delivery",
    ])
    .unwrap();
    let response = run_command(&mut domain, command).unwrap();
    assert_eq!(response.fields["run_created"], "true");
    assert_eq!(response.fields["spec_locked"], "true");
    assert_eq!(response.fields["has_run_id"], "true");
    assert_eq!(response.fields["run_id"], "run-1");
}

#[test]
fn doc_command_writes_artifact_and_row() {
    let mut domain = TestDomain::default();
    let command = parse_args([
        "doc",
        "create",
        "shadow-implementer",
        "--title",
        "cli-ux coverage",
        "--run",
        "run-1",
    ])
    .unwrap();
    let response = run_command(&mut domain, command).unwrap();
    assert_eq!(response.fields["artifact_file_exists"], "true");
    assert_eq!(response.fields["document_row_exists"], "true");
    assert_eq!(response.fields["has_doc_id"], "true");
    assert!(response.fields["file_path"].contains(".popsicle/artifacts/run-1"));
}

#[test]
fn stage_advance_reflects_state() {
    let mut domain = TestDomain::default();
    let command = parse_args([
        "pipeline",
        "stage",
        "complete",
        "current",
        "--run",
        "run-1",
        "--confirm",
    ])
    .unwrap();
    let response = run_command(&mut domain, command).unwrap();
    assert_eq!(response.fields["previous_completed"], "true");
    assert_eq!(response.fields["downstream_ready"], "true");
    assert_eq!(response.fields["status_reflects_state"], "true");
    assert_eq!(response.fields["current_stage"], "next");
}

#[test]
fn errors_are_actionable() {
    let err = parse_args(["pipeline", "stage", "complete", "current", "--run", "run-1"])
        .and_then(|command| run_command(&mut TestDomain::default(), command))
        .unwrap_err();
    assert_eq!(err.category, "lock");
    assert!(err.object_ref.contains("current"));
    assert!(err.next_step.contains("--confirm"));
    assert!(err.has_category_object_and_next_step());
}

#[test]
fn admin_commands_are_explicit() {
    let command = parse_args(["admin", "migrate", "--workspace", "/tmp/popsicle"]).unwrap();
    assert!(matches!(
        command,
        Command::Admin(AdminCommand::Migrate { ref workspace }) if workspace == "/tmp/popsicle"
    ));

    let mut domain = TestDomain::default();
    let response = run_command(&mut domain, command).unwrap();
    assert_eq!(response.fields["under_admin_tree"], "true");
    assert_eq!(response.fields["explicit_workspace"], "true");

    let top_level = parse_args(["migrate"]).unwrap_err();
    assert_eq!(top_level.category, "invalid-args");
    assert!(top_level.next_step.contains("admin migrate"));
}

#[test]
fn render_top_level_help_keeps_removed_commands_removed() {
    let help = top_level_help();
    assert!(!contains_removed_top_level_command(&help));
    assert!(help.lines().any(|line| line == "doc"));
    assert!(help.lines().any(|line| line == "admin"));

    for removed in ["checklist", "item", "sync"] {
        let err = parse_args([removed]).unwrap_err();
        assert_eq!(err.category, "invalid-args");
        assert!(err.has_category_object_and_next_step());
    }
}
