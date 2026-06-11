//! Domain types for skill-runtime.
//!
//! These mirror the shared domain block declared in
//! `products/skill-runtime/intents/acceptance.intent` and
//! `products/skill-runtime/intents/invariants.intent` (kept in sync — a change
//! here that alters the field/variant set must trace to a product ADR/PDR).

/// Stage lifecycle status. Mirrors `enum StageStatus` in the intents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    StageBlocked,
    StageReady,
    StageInProgress,
    StageCompleted,
    StageError,
}

/// Pipeline-run status. Mirrors `enum PipelineRunStatus` in the intents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineRunStatus {
    RunPending,
    RunInProgress,
    RunCompleted,
    RunBlocked,
}

/// Skill status. Mirrors `enum SkillStatus` in the intents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillStatus {
    Loaded,
    Active,
    Deprecated,
}

/// A loaded skill. Mirrors `type Skill` in the intents.
///
/// Per ADR-002 the legacy single `version` is split into `pkg_version`
/// (package release, semver) and `schema_version` (load-result/state-machine
/// schema, independent of `pkg_version`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub pkg_version: String,
    pub schema_version: String,
    pub status: SkillStatus,
}

impl Skill {
    /// Whether `self` is a **backward-compatible package upgrade** of `old`
    /// (ADR-002 dual-version): same name, `pkg_version` changed, `schema_version`
    /// unchanged. Consumers may treat the load-result structure as compatible.
    pub fn is_backward_compatible_upgrade_of(&self, old: &Skill) -> bool {
        self.name == old.name
            && self.pkg_version != old.pkg_version
            && self.schema_version == old.schema_version
    }
}

/// A pipeline stage. Mirrors `type Stage` in the intents.
///
/// `approved_at`: `0` = not approved; `> 0` = approval timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub name: String,
    pub status: StageStatus,
    pub requires_approval: bool,
    pub approved_at: i64,
}

/// A pipeline run. Mirrors `type PipelineRun` in the intents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineRun {
    pub id: String,
    pub status: PipelineRunStatus,
    pub current_stage_index: i64,
    pub total_stages: i64,
}
