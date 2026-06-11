//! The `skill load` contract (ADR-002 / `contracts.intent`).

use crate::state_machine::SkillState;

/// The stable load-result contract returned by `skill load`.
///
/// **Exactly four fields** per ADR-002; adding or removing a field is a
/// schema-incompatible change and MUST bump `schema_version`
/// (see `contracts.intent` — *"始终含且仅含约定四字段"*).
///
/// `pkg_version` is the package release (semver); `schema_version` is the
/// load-result/state-machine schema version, **independent** of `pkg_version`
/// (ADR-002 dual-version). A backward-compatible package upgrade changes
/// `pkg_version` while leaving `schema_version` untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillLoadResult {
    pub name: String,
    pub pkg_version: String,
    pub schema_version: String,
    pub state_machine: StateMachine,
}

/// The `state_machine` field: the skill's execution state machine, expressed as
/// its allowed transition set. The canonical machine is the 4-state, single-forward
/// `pending → in_progress → completed | blocked` (ADR-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    pub transitions: Vec<(SkillState, SkillState)>,
}

impl StateMachine {
    /// The canonical ADR-002 machine (no bypass, no backward).
    pub fn canonical() -> Self {
        Self {
            transitions: SkillState::TRANSITIONS.to_vec(),
        }
    }
}

/// Whether `new` is a backward-compatible package upgrade of `old`
/// (ADR-002 dual-version): same `name`, `pkg_version` changed, `schema_version`
/// unchanged. When true, a consumer may treat the load-result structure as
/// unchanged (and, per `acceptance.intent`, completed runs are unaffected).
pub fn is_backward_compatible_upgrade(old: &SkillLoadResult, new: &SkillLoadResult) -> bool {
    old.name == new.name
        && old.pkg_version != new.pkg_version
        && old.schema_version == new.schema_version
}
