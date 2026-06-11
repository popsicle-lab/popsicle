//! Property tests that mirror the Z3-verified intents in
//! `products/skill-runtime/intents/`. Each test name traces to a named intent
//! contract / acceptance proof / invariant so a reviewer can line them up 1:1.

use skill_runtime::domain::{
    PipelineRun, PipelineRunStatus, Skill, SkillStatus, Stage, StageStatus,
};
use skill_runtime::runs::{
    apply_skill_upgrade, bootstrap_to_first_pause, recover_blocked_pipeline, BootstrapError,
    RecoverError,
};
use skill_runtime::skill_load::{is_backward_compatible_upgrade, SkillLoadResult, StateMachine};
use skill_runtime::state_machine::{
    advance_stage_with_approval, approved_before_completed, SkillState, StageAdvanceError,
};

const ALL_STATES: [SkillState; 4] = [
    SkillState::Pending,
    SkillState::InProgress,
    SkillState::Completed,
    SkillState::Blocked,
];

fn stage(status: StageStatus, requires_approval: bool, approved_at: i64) -> Stage {
    Stage {
        name: "s".into(),
        status,
        requires_approval,
        approved_at,
    }
}

fn run(status: PipelineRunStatus, idx: i64, total: i64) -> PipelineRun {
    PipelineRun {
        id: "run-1".into(),
        status,
        current_stage_index: idx,
        total_stages: total,
    }
}

// --- contracts.intent: SkillLoadResult shape (ADR-002) ----------------------

/// `SkillLoadResult` contains *and only contains* the four agreed fields.
/// Exhaustive destructuring makes adding/removing a field fail to compile,
/// which is exactly the "始终含且仅含约定四字段" guarantee at the type level.
#[test]
fn skill_load_result_has_exactly_four_fields() {
    let r = SkillLoadResult {
        name: "demo".into(),
        pkg_version: "1.0.0".into(),
        schema_version: "1".into(),
        state_machine: StateMachine::canonical(),
    };
    let SkillLoadResult {
        name,
        pkg_version,
        schema_version,
        state_machine,
    } = &r;
    assert_eq!(name, "demo");
    assert_eq!(pkg_version, "1.0.0");
    assert_eq!(schema_version, "1");
    assert_eq!(state_machine.transitions.len(), 3);
}

/// pkg_version vs schema_version are decoupled: a package bump that keeps the
/// schema is a backward-compatible upgrade (ADR-002 dual-version).
#[test]
fn backward_compatible_upgrade_changes_pkg_not_schema() {
    let old = SkillLoadResult {
        name: "demo".into(),
        pkg_version: "1.0.0".into(),
        schema_version: "1".into(),
        state_machine: StateMachine::canonical(),
    };
    let new = SkillLoadResult {
        pkg_version: "1.1.0".into(),
        ..old.clone()
    };
    assert!(is_backward_compatible_upgrade(&old, &new));

    // A schema bump is NOT a backward-compatible package upgrade.
    let schema_bump = SkillLoadResult {
        schema_version: "2".into(),
        ..old.clone()
    };
    assert!(!is_backward_compatible_upgrade(&old, &schema_bump));
}

// --- contracts.intent: canonical 4-state machine, no bypass -----------------

/// The machine permits *only* the three canonical single-forward transitions;
/// every other ordered pair (incl. bypass pending→completed and any backward
/// edge) is rejected. Mirrors "无绕过路径".
#[test]
fn state_machine_allows_only_canonical_forward_transitions() {
    let allowed = [
        (SkillState::Pending, SkillState::InProgress),
        (SkillState::InProgress, SkillState::Completed),
        (SkillState::InProgress, SkillState::Blocked),
    ];
    for &from in &ALL_STATES {
        for &to in &ALL_STATES {
            let expected = allowed.contains(&(from, to));
            assert_eq!(
                from.can_transition_to(to),
                expected,
                "transition {from:?} -> {to:?} should be {expected}",
            );
        }
    }
    // No bypass: pending cannot jump straight to completed.
    assert!(SkillState::Pending.transition(SkillState::Completed).is_err());
}

#[test]
fn completed_and_blocked_are_terminal() {
    assert!(SkillState::Completed.is_terminal());
    assert!(SkillState::Blocked.is_terminal());
    assert!(!SkillState::Pending.is_terminal());
    assert!(!SkillState::InProgress.is_terminal());
    for &to in &ALL_STATES {
        assert!(!SkillState::Completed.can_transition_to(to));
        assert!(!SkillState::Blocked.can_transition_to(to));
    }
}

// --- invariants.intent: StageAdvanceWithApproval + ApprovedBeforeCompleted --

/// HC-2 gate: a `requires_approval` stage with no approval cannot complete.
#[test]
fn unapproved_required_stage_cannot_complete() {
    let s = stage(StageStatus::StageInProgress, true, 0);
    assert_eq!(
        advance_stage_with_approval(&s),
        Err(StageAdvanceError::ApprovalRequired)
    );
}

/// An approved required stage completes, frames its fields, and the safety
/// `ApprovedBeforeCompleted` holds on the post-state.
#[test]
fn approved_required_stage_completes_and_preserves_invariant() {
    let s = stage(StageStatus::StageInProgress, true, 42);
    let out = advance_stage_with_approval(&s).expect("approved stage advances");
    assert_eq!(out.status, StageStatus::StageCompleted);
    assert_eq!(out.requires_approval, true); // framed
    assert_eq!(out.approved_at, 42); // framed
    assert!(approved_before_completed(&out));
}

/// A stage that does not require approval completes without any approval.
#[test]
fn non_required_stage_completes_without_approval() {
    let s = stage(StageStatus::StageInProgress, false, 0);
    let out = advance_stage_with_approval(&s).expect("non-required stage advances");
    assert_eq!(out.status, StageStatus::StageCompleted);
    assert!(approved_before_completed(&out));
}

/// Precondition: only an in-progress stage may advance.
#[test]
fn advance_requires_in_progress() {
    for status in [
        StageStatus::StageBlocked,
        StageStatus::StageReady,
        StageStatus::StageCompleted,
        StageStatus::StageError,
    ] {
        let s = stage(status, false, 0);
        assert_eq!(
            advance_stage_with_approval(&s),
            Err(StageAdvanceError::NotInProgress),
            "status {status:?} must not advance",
        );
    }
}

/// `ApprovedBeforeCompleted` holds for every reachable post-state of the
/// advance operation, across the full cross-product of inputs that satisfy the
/// preconditions. This is the runtime echo of the Z3 safety proof.
#[test]
fn approved_before_completed_holds_after_any_advance() {
    for requires_approval in [false, true] {
        for approved_at in [0_i64, 7] {
            let s = stage(StageStatus::StageInProgress, requires_approval, approved_at);
            if let Ok(out) = advance_stage_with_approval(&s) {
                assert!(
                    approved_before_completed(&out),
                    "invariant violated for requires_approval={requires_approval} approved_at={approved_at}",
                );
            }
        }
    }
}

// --- acceptance.intent: UpgradeDoesNotAffectCompletedRuns -------------------

#[test]
fn upgrade_does_not_affect_completed_runs() {
    let run = PipelineRun {
        id: "run-1".into(),
        status: PipelineRunStatus::RunCompleted,
        current_stage_index: 3,
        total_stages: 3,
    };
    let old = Skill {
        name: "demo".into(),
        pkg_version: "1.0.0".into(),
        schema_version: "1".into(),
        status: SkillStatus::Active,
    };
    let new = Skill {
        pkg_version: "2.0.0".into(),
        status: SkillStatus::Deprecated,
        ..old.clone()
    };
    assert!(new.is_backward_compatible_upgrade_of(&old));

    let after = apply_skill_upgrade(&run, &old, &new);
    assert_eq!(after.status, PipelineRunStatus::RunCompleted);
    assert_eq!(after.current_stage_index, 3);
    assert_eq!(after.total_stages, 3);
    assert_eq!(after, run);
}

// --- acceptance.intent: PipelineBootstrapsToFirstPause (T-0001) -------------

/// A pending run with a blocked first stage bootstraps: run -> in_progress,
/// stage -> in_progress, progress counters framed.
#[test]
fn pipeline_bootstraps_to_first_pause() {
    let r = run(PipelineRunStatus::RunPending, 0, 5);
    let s = stage(StageStatus::StageBlocked, true, 0);
    let (r2, s2) = bootstrap_to_first_pause(&r, &s).expect("pending+blocked bootstraps");
    assert_eq!(r2.status, PipelineRunStatus::RunInProgress);
    assert_eq!(s2.status, StageStatus::StageInProgress);
    assert_eq!(r2.current_stage_index, 0); // frame
    assert_eq!(r2.total_stages, 5); // frame
}

/// Bootstrap is rejected unless both preconditions hold.
#[test]
fn bootstrap_requires_pending_run_and_blocked_stage() {
    let blocked = stage(StageStatus::StageBlocked, false, 0);
    let in_progress_stage = stage(StageStatus::StageInProgress, false, 0);

    // Wrong run status.
    for status in [
        PipelineRunStatus::RunInProgress,
        PipelineRunStatus::RunCompleted,
        PipelineRunStatus::RunBlocked,
    ] {
        assert_eq!(
            bootstrap_to_first_pause(&run(status, 0, 5), &blocked),
            Err(BootstrapError::RunNotPending),
        );
    }
    // Right run, wrong stage status.
    assert_eq!(
        bootstrap_to_first_pause(&run(PipelineRunStatus::RunPending, 0, 5), &in_progress_stage),
        Err(BootstrapError::StageNotBlocked),
    );
}

// --- acceptance.intent: RecoveredPipelineCanAdvance (T-0004) ----------------

/// A blocked run whose stage errored recovers: run -> in_progress, stage leaves
/// the error state, and progress counters are preserved (no lost output).
#[test]
fn recovered_pipeline_can_advance() {
    let r = run(PipelineRunStatus::RunBlocked, 2, 5);
    let s = stage(StageStatus::StageError, false, 0);
    let (r2, s2) = recover_blocked_pipeline(&r, &s).expect("blocked+error recovers");
    assert_eq!(r2.status, PipelineRunStatus::RunInProgress);
    assert_ne!(s2.status, StageStatus::StageError);
    assert_eq!(r2.current_stage_index, 2); // frame: 恢复不丢前面产出
    assert_eq!(r2.total_stages, 5); // frame
}

/// Recovery is rejected unless both preconditions hold.
#[test]
fn recover_requires_blocked_run_and_errored_stage() {
    let errored = stage(StageStatus::StageError, false, 0);
    let ready_stage = stage(StageStatus::StageReady, false, 0);

    for status in [
        PipelineRunStatus::RunPending,
        PipelineRunStatus::RunInProgress,
        PipelineRunStatus::RunCompleted,
    ] {
        assert_eq!(
            recover_blocked_pipeline(&run(status, 2, 5), &errored),
            Err(RecoverError::RunNotBlocked),
        );
    }
    assert_eq!(
        recover_blocked_pipeline(&run(PipelineRunStatus::RunBlocked, 2, 5), &ready_stage),
        Err(RecoverError::StageNotError),
    );
}
