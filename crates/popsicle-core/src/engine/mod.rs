mod advisor;
pub mod context;
pub mod extractor;
pub mod guard;
pub mod hooks;
pub mod markdown;
mod recommender;

pub use advisor::{Advisor, NextStep};
pub use context::{AssembledContext, ContextInput, ContextPart, assemble_input_context};
pub use guard::{GuardResult, check_guard, count_checkboxes};
pub use recommender::{Alternative, PipelineRecommender, Recommendation};
