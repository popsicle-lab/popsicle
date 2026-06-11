//! Run lifecycle operations (`acceptance.intent` run-level intents).

use crate::domain::{PipelineRun, PipelineRunStatus, Skill, Stage, StageStatus};

/// Why a [`bootstrap_to_first_pause`] call was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapError {
    /// Precondition `r.status == RunPending` failed.
    RunNotPending,
    /// Precondition `s.status == StageBlocked` failed.
    StageNotBlocked,
}

/// Why a [`recover_blocked_pipeline`] call was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverError {
    /// Precondition `r.status == RunBlocked` failed.
    RunNotBlocked,
    /// Precondition `s.status == StageError` failed.
    StageNotError,
}

/// `PipelineBootstrapsToFirstPause` (`acceptance.intent`, T-0001).
///
/// Starts a pending run: the run goes `RunInProgress` and its first (blocked)
/// stage enters `StageInProgress`, where it parks at the first
/// `requires_approval` pause point. Run progress counters are **framed**.
///
/// - `require r.status == RunPending`
/// - `require s.status == StageBlocked`
/// - `ensure r.status' == RunInProgress`
/// - `ensure s.status' == StageInProgress`
/// - frame: `current_stage_index`, `total_stages`
pub fn bootstrap_to_first_pause(
    r: &PipelineRun,
    s: &Stage,
) -> Result<(PipelineRun, Stage), BootstrapError> {
    if r.status != PipelineRunStatus::RunPending {
        return Err(BootstrapError::RunNotPending);
    }
    if s.status != StageStatus::StageBlocked {
        return Err(BootstrapError::StageNotBlocked);
    }
    let r2 = PipelineRun {
        status: PipelineRunStatus::RunInProgress,
        current_stage_index: r.current_stage_index, // frame
        total_stages: r.total_stages,               // frame
        ..r.clone()
    };
    let s2 = Stage {
        status: StageStatus::StageInProgress,
        ..s.clone()
    };
    Ok((r2, s2))
}

/// `RecoveredPipelineCanAdvance` (`acceptance.intent`, T-0004).
///
/// Recovers a blocked run whose current stage errored (e.g. after
/// `popsicle pipeline unlock`): the run resumes `RunInProgress` and the stage
/// leaves the `StageError` state — here to `StageReady`, so it can be re-run
/// without losing prior output. Run progress counters are **framed** (recovery
/// does not roll back completed work).
///
/// - `require r.status == RunBlocked`
/// - `require s.status == StageError`
/// - `ensure r.status' == RunInProgress`
/// - `ensure s.status' != StageError`
/// - frame: `current_stage_index`, `total_stages`
pub fn recover_blocked_pipeline(
    r: &PipelineRun,
    s: &Stage,
) -> Result<(PipelineRun, Stage), RecoverError> {
    if r.status != PipelineRunStatus::RunBlocked {
        return Err(RecoverError::RunNotBlocked);
    }
    if s.status != StageStatus::StageError {
        return Err(RecoverError::StageNotError);
    }
    let r2 = PipelineRun {
        status: PipelineRunStatus::RunInProgress,
        current_stage_index: r.current_stage_index, // frame: 恢复不丢前面产出
        total_stages: r.total_stages,               // frame
        ..r.clone()
    };
    let s2 = Stage {
        status: StageStatus::StageReady,
        ..s.clone()
    };
    Ok((r2, s2))
}

/// `UpgradeDoesNotAffectCompletedRuns` (`acceptance.intent`).
///
/// A skill **package** upgrade (same name, `pkg_version` changed; the schema is
/// compatible) never rewrites run state. In particular a `RunCompleted` run is
/// returned frozen — same `status`, `current_stage_index`, `total_stages`.
///
/// - `require r.status == RunCompleted`
/// - `require old.name == new.name && old.pkg_version != new.pkg_version`
/// - `ensure` run `status` / `current_stage_index` / `total_stages` unchanged
///
/// Modelled as a pure function: applying a skill upgrade is orthogonal to run
/// state, so runs are left untouched.
pub fn apply_skill_upgrade(r: &PipelineRun, old: &Skill, new: &Skill) -> PipelineRun {
    debug_assert_eq!(old.name, new.name, "upgrade must keep the skill name");
    debug_assert!(
        new.is_backward_compatible_upgrade_of(old) || old == new,
        "apply_skill_upgrade expects a backward-compatible package upgrade",
    );
    // A package upgrade does not mutate run progress; completed runs stay frozen.
    r.clone()
}
