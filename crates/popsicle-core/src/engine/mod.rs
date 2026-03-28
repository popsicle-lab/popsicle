mod advisor;
pub mod bootstrap;
pub mod context;
pub mod extractor;
pub mod guard;
pub mod hooks;
pub mod markdown;
mod recommender;

pub use advisor::{Advisor, NextStep};
pub use bootstrap::{
    BootstrapDoc, BootstrapPlan, BootstrapResult, build_bootstrap_prompt, execute_bootstrap_plan,
};
pub use context::{AssembledContext, ContextInput, ContextPart, assemble_input_context};
pub use guard::{GuardResult, check_guard, count_checkboxes};
pub use recommender::{Alternative, PipelineRecommender, Recommendation};
