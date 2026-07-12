//! Golden equivalence baselines for `slice-delivery` / `equivalence-baseline` skill.
//!
//! Each `golden_*` test is referenced from
//! `docs/baseline/2026-06-09/skill-runtime/run-all.sh`.

use std::path::PathBuf;

use skill_runtime::loader::{load_skill, PipelineDef, SKILL_LOAD_SCHEMA_VERSION};
use skill_runtime::pipeline_session::PipelineSession;
use skill_runtime::registry::SkillRegistry;
use skill_runtime::state_machine::SkillState;
use skill_runtime::StateMachine;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn intent_coder_skills_dir() -> PathBuf {
    repo_root().join(".popsicle/modules/intent-coder/skills")
}

fn intent_coder_pipelines_dir() -> PathBuf {
    repo_root().join(".popsicle/modules/intent-coder/pipelines")
}

/// G-001: `project-init` skill loads with ADR-002 four-field result.
#[test]
fn golden_001_load_project_init_skill() {
    let path = intent_coder_skills_dir().join("project-init/skill.yaml");
    if !path.is_file() {
        eprintln!("skip: {path:?} missing (module not installed)");
        return;
    }
    let loaded = load_skill(&path).expect("load project-init");
    assert_eq!(loaded.load_result.name, "project-init");
    assert_eq!(loaded.load_result.schema_version, SKILL_LOAD_SCHEMA_VERSION);
    assert_eq!(loaded.load_result.state_machine, StateMachine::canonical());
    assert_eq!(loaded.workflow_initial, "surveying");
}

/// G-002: `migration-bootstrap` pipeline validates (10 stages).
#[test]
fn golden_002_load_migration_bootstrap_pipeline() {
    let path = intent_coder_pipelines_dir().join("migration-bootstrap.pipeline.yaml");
    if !path.is_file() {
        eprintln!("skip: {path:?} missing");
        return;
    }
    let p = PipelineDef::load(&path).expect("load pipeline");
    p.validate().expect("valid deps");
    assert_eq!(p.name, "migration-bootstrap");
    assert_eq!(p.stages.len(), 10);
    assert_eq!(p.stages[0].name, "init");
    assert_eq!(p.stages.last().unwrap().name, "living-docs");
}

/// G-003: `migration-slice-delivery` pipeline has 4 delivery stages.
#[test]
fn golden_003_load_slice_delivery_pipeline() {
    let path = intent_coder_pipelines_dir().join("migration-slice-delivery.pipeline.yaml");
    if !path.is_file() {
        eprintln!("skip: {path:?} missing");
        return;
    }
    let p = PipelineDef::load(&path).expect("load slice-delivery");
    assert_eq!(p.stages.len(), 4);
    let names: Vec<_> = p.stages.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        ["implement", "equivalence", "cutover", "living-docs"]
    );
}

/// G-003b: `migration-preserve` fast lane validates (7 stages, no debate/arch/rfc/adr).
#[test]
fn golden_003b_load_migration_preserve_pipeline() {
    let path = intent_coder_pipelines_dir().join("migration-preserve.pipeline.yaml");
    if !path.is_file() {
        eprintln!("skip: {path:?} missing");
        return;
    }
    let p = PipelineDef::load(&path).expect("load migration-preserve");
    p.validate().expect("valid deps");
    assert_eq!(p.name, "migration-preserve");
    let names: Vec<_> = p.stages.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "facts",
            "intent-spec",
            "intent-check",
            "implement",
            "equivalence",
            "cutover",
            "living-docs"
        ]
    );
    // The whole point of the fast lane: it skips the redesign chain.
    for skipped in ["product-debate", "prd-writer", "arch-debate", "rfc-writer"] {
        assert!(
            !p.stages.iter().any(|s| s.skill.as_deref() == Some(skipped)),
            "migration-preserve must skip {skipped}"
        );
    }
}

/// G-004: intent-coder module exposes 13 skills via registry scan.
#[test]
fn golden_004_skill_registry_count() {
    let dir = intent_coder_skills_dir();
    if !dir.is_dir() {
        eprintln!("skip: intent-coder skills dir missing");
        return;
    }
    let mut reg = SkillRegistry::new();
    let n = reg.load_dir(&dir).expect("scan skills");
    assert_eq!(n, 14, "intent-coder expects 14 skills (incl. issue-author)");
    assert!(reg.get("issue-author").is_some());
    assert!(reg.get("shadow-implementer").is_some());
    assert!(reg.get("equivalence-baseline").is_some());
    assert!(reg.get("cutover-author").is_some());
}

/// G-005: canonical state machine — exactly 3 forward transitions, no bypass.
#[test]
fn golden_005_canonical_state_machine() {
    let sm = StateMachine::canonical();
    assert_eq!(sm.transitions.len(), 3);
    assert!(!SkillState::Pending.can_transition_to(SkillState::Completed));
    assert!(SkillState::Pending.can_transition_to(SkillState::InProgress));
}

/// G-006: two-stage pipeline session — bootstrap → approve → complete advances index.
#[test]
fn golden_006_pipeline_session_stage_advance() {
    let pipeline = PipelineDef {
        name: "g6".into(),
        description: "golden".into(),
        stages: vec![
            skill_runtime::PipelineStageDef {
                name: "a".into(),
                skill: Some("s1".into()),
                skills: vec![],
                description: "first".into(),
                depends_on: vec![],
                requires_approval: true,
                gate: vec![],
            },
            skill_runtime::PipelineStageDef {
                name: "b".into(),
                skill: Some("s2".into()),
                skills: vec![],
                description: "second".into(),
                depends_on: vec!["a".into()],
                requires_approval: false,
                gate: vec![],
            },
        ],
        keywords: vec![],
        scale: None,
    };
    let mut session = PipelineSession::new_pending("g6-run", pipeline);
    session.start().unwrap();
    session.approve_current(1).unwrap();
    session.complete_current().unwrap();
    assert_eq!(session.run.current_stage_index, 1);
    let snap = session.snapshot();
    assert_eq!(snap.current_stage_name(), Some("b"));
}
