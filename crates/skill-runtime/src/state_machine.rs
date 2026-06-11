//! The skill execution state machine (ADR-002) and the HC-2 approval gate.
//!
//! The canonical machine is **4-state, single-direction-forward**:
//! `pending → in_progress → completed | blocked`, with **no bypass and no
//! backward** transitions. This mirrors the `contracts.intent` measure
//! *"state_machine 仅含 {pending→in_progress→completed/blocked} 转移，无绕过路径"*.

use crate::domain::{Stage, StageStatus};

/// The four states of the skill/stage execution machine (ADR-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillState {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

impl SkillState {
    /// The full set of allowed transitions — single-forward, no bypass.
    /// This is the source of truth used to build
    /// [`StateMachine::canonical`](crate::skill_load::StateMachine::canonical).
    pub const TRANSITIONS: [(SkillState, SkillState); 3] = [
        (SkillState::Pending, SkillState::InProgress),
        (SkillState::InProgress, SkillState::Completed),
        (SkillState::InProgress, SkillState::Blocked),
    ];

    /// Whether a direct transition `self → to` is permitted.
    pub fn can_transition_to(self, to: SkillState) -> bool {
        Self::TRANSITIONS.contains(&(self, to))
    }

    /// Apply a transition, or report the illegal pair.
    pub fn transition(self, to: SkillState) -> Result<SkillState, IllegalTransition> {
        if self.can_transition_to(to) {
            Ok(to)
        } else {
            Err(IllegalTransition { from: self, to })
        }
    }

    /// `completed` and `blocked` are terminal (no outgoing transitions).
    pub fn is_terminal(self) -> bool {
        matches!(self, SkillState::Completed | SkillState::Blocked)
    }
}

/// An attempted transition that the machine does not permit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllegalTransition {
    pub from: SkillState,
    pub to: SkillState,
}

/// Why a [`advance_stage_with_approval`] call was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageAdvanceError {
    /// Precondition `s.status == StageInProgress` failed.
    NotInProgress,
    /// HC-2 gate: a `requires_approval` stage had no approval (`approved_at == 0`).
    ApprovalRequired,
}

/// `StageAdvanceWithApproval` (invariants.intent + acceptance.intent).
///
/// Advances an in-progress stage to `Completed`, enforcing the HC-2 approval gate
/// so that the safety [`approved_before_completed`] is preserved. `requires_approval`
/// and `approved_at` are **framed** (never silently mutated).
///
/// - `require s.status == StageInProgress`
/// - `require !s.requires_approval || s.approved_at > 0`
/// - `ensure s.status' == StageCompleted`
/// - frame: `requires_approval`, `approved_at`
pub fn advance_stage_with_approval(s: &Stage) -> Result<Stage, StageAdvanceError> {
    if s.status != StageStatus::StageInProgress {
        return Err(StageAdvanceError::NotInProgress);
    }
    if s.requires_approval && s.approved_at == 0 {
        return Err(StageAdvanceError::ApprovalRequired);
    }
    Ok(Stage {
        name: s.name.clone(),
        status: StageStatus::StageCompleted,
        requires_approval: s.requires_approval, // frame
        approved_at: s.approved_at,             // frame
    })
}

/// The safety `ApprovedBeforeCompleted` (invariants.intent, HC-2) as a runtime
/// predicate over a single state:
///
/// `(status == Completed && requires_approval) ==> approved_at > 0`.
pub fn approved_before_completed(s: &Stage) -> bool {
    !(s.status == StageStatus::StageCompleted && s.requires_approval) || s.approved_at > 0
}
