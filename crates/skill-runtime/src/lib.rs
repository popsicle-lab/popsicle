//! # skill-runtime
//!
//! In-shadow implementation of the `skill-runtime` product slice
//! (`migration/progress.md`). Mirrors Z3-verified intents and ADR-002/003/004.
//!
//! - [`domain`] / [`state_machine`] / [`runs`] — pure domain + acceptance intents
//! - [`loader`] / [`registry`] — load `skill.yaml` + `.pipeline.yaml` from disk
//! - [`pipeline_session`] — orchestrate pipeline runs (T-0001/0002/0004)
//! - [`inspect`] — read-only status (T-0003)
//! - [`upstream`] — ADR-004 `UpstreamApprovalChecker` port
//! - [`memory_layer`] / [`context`] — ADR-004 `MemoriesLayer` + registry
//! - [`issue`] — minimal issue entity (PDR-001)
//!
//! Persistence rows live in [`storage`] crate (`DocumentRow`, ADR-004).

pub mod context;
pub mod domain;
pub mod inspect;
pub mod issue;
pub mod loader;
pub mod memory_layer;
pub mod pipeline_session;
pub mod registry;
pub mod runs;
pub mod skill_load;
pub mod state_machine;
pub mod upstream;

pub use context::ContextRegistry;
pub use domain::{
    PipelineRun, PipelineRunStatus, Skill, SkillStatus, Stage, StageStatus,
};
pub use inspect::{PipelineStatusSnapshot, StageSnapshot};
pub use issue::{Issue, IssueType};
pub use loader::{
    load_pipelines_dir, load_skill, load_skills_dir, LoadedSkill, LoadError, PipelineDef,
    PipelineStageDef, SKILL_LOAD_SCHEMA_VERSION,
};
pub use memory_layer::{MemoriesLayer, Memory};
pub use pipeline_session::{PipelineSession, SessionError};
pub use registry::{PipelineRegistry, SkillRegistry};
pub use runs::{
    apply_skill_upgrade, bootstrap_to_first_pause, recover_blocked_pipeline, BootstrapError,
    RecoverError,
};
pub use skill_load::{is_backward_compatible_upgrade, SkillLoadResult, StateMachine};
pub use state_machine::{
    advance_stage_with_approval, approved_before_completed, SkillState, StageAdvanceError,
};
pub use upstream::PipelineUpstreamChecker;
