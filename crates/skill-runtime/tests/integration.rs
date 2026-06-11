//! Integration tests for loader, session, ADR-004 ports.

use std::path::PathBuf;

use artifact_system::document::Document;
use artifact_system::guard::check_guard;
use skill_runtime::context::ContextRegistry;
use skill_runtime::domain::StageStatus;
use skill_runtime::loader::{load_skill, PipelineDef, SKILL_LOAD_SCHEMA_VERSION};
use skill_runtime::memory_layer::{MemoriesLayer, Memory};
use skill_runtime::pipeline_session::PipelineSession;
use skill_runtime::registry::{PipelineRegistry, SkillRegistry};
use skill_runtime::upstream::PipelineUpstreamChecker;
use skill_runtime::SkillLoadResult;
use skill_runtime::StateMachine;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn load_skill_yaml_produces_adr002_result() {
    let path = fixtures_dir().join("demo-skill/skill.yaml");
    let loaded = load_skill(&path).expect("load fixture skill");
    assert_eq!(loaded.load_result.name, "demo-skill");
    assert_eq!(loaded.load_result.pkg_version, "1.2.3");
    assert_eq!(loaded.load_result.schema_version, SKILL_LOAD_SCHEMA_VERSION);
    assert_eq!(loaded.load_result.state_machine, StateMachine::canonical());
    assert_eq!(loaded.workflow_initial, "scoping");
}

#[test]
fn load_pipeline_yaml_and_run_session_happy_path() {
    let pipeline_path = fixtures_dir().join("demo.pipeline.yaml");
    let pipeline = PipelineDef::load(&pipeline_path).expect("load pipeline");
    pipeline.validate().expect("valid deps");

    let mut session = PipelineSession::new_pending("run-1", pipeline);
    session.start().expect("bootstrap");
    assert_eq!(
        session.run.status,
        skill_runtime::PipelineRunStatus::RunInProgress
    );
    assert_eq!(session.stages[0].status, StageStatus::StageInProgress);

    session.approve_current(42).expect("approve");
    session.complete_current().expect("complete stage 0");
    assert_eq!(session.run.current_stage_index, 1);
    assert_eq!(session.stages[1].status, StageStatus::StageInProgress);

    let snap = session.snapshot();
    assert_eq!(snap.pipeline_name, "demo-pipeline");
    assert_eq!(snap.current_stage_name(), Some("facts"));
}

#[test]
fn recover_blocked_pipeline_session() {
    let pipeline_path = fixtures_dir().join("demo.pipeline.yaml");
    let pipeline = PipelineDef::load(&pipeline_path).unwrap();
    let mut session = PipelineSession::new_pending("run-2", pipeline);
    session.start().unwrap();
    session.fail_current().unwrap();
    session.recover_current().unwrap();
    assert_ne!(session.stages[0].status, StageStatus::StageError);
}

#[test]
fn registry_loads_fixture_dirs() {
    let mut skills = SkillRegistry::new();
    let n = skills.load_dir(&fixtures_dir()).expect("load skills dir");
    assert_eq!(n, 1);
    assert!(skills.get("demo-skill").is_some());

    let mut pipes = PipelineRegistry::new();
    let m = pipes.load_dir(&fixtures_dir()).expect("load pipelines");
    assert_eq!(m, 1);
    assert!(pipes.get("demo-pipeline").is_some());
}

#[test]
fn upstream_checker_and_guard_integration() {
    let mut doc = Document::new("d1", "report", "t");
    doc.extra_frontmatter
        .insert("upstream_stages".into(), "init,facts".into());

    let checker = PipelineUpstreamChecker::with_completed(["init"]);
    let r = check_guard("upstream_approved", &doc, Some(&checker)).unwrap();
    assert!(!r.passed, "facts not completed yet");

    let checker2 = PipelineUpstreamChecker::with_completed(["init", "facts"]);
    let r2 = check_guard("upstream_approved", &doc, Some(&checker2)).unwrap();
    assert!(r2.passed);

    doc.status = "final".into();
    let checker_empty = PipelineUpstreamChecker::default();
    let r3 = check_guard("upstream_approved", &doc, Some(&checker_empty)).unwrap();
    assert!(r3.passed);
}

#[test]
fn memories_layer_assembles_into_prompt() {
    let mut reg = ContextRegistry::new();
    reg.register_memories(MemoriesLayer::new(vec![Memory {
        memory_type: "tip".into(),
        summary: "run intent check before merge".into(),
        stale: false,
    }]));
    let prompt = reg.assemble_borrowed("Do the task.");
    assert!(prompt.contains("Project Memories"));
    assert!(prompt.contains("intent check"));
    assert!(prompt.ends_with("Do the task."));
}

#[test]
fn skill_load_result_four_fields_unchanged() {
    let r = SkillLoadResult {
        name: "x".into(),
        pkg_version: "1".into(),
        schema_version: "1".into(),
        state_machine: StateMachine::canonical(),
    };
    let SkillLoadResult {
        name,
        pkg_version,
        schema_version,
        state_machine,
    } = r;
    assert_eq!(name, "x");
    assert_eq!(pkg_version, "1");
    assert_eq!(schema_version, "1");
    assert_eq!(state_machine.transitions.len(), 3);
}
