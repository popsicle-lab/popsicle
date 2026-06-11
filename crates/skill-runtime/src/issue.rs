//! Minimal `issue` entity (PDR-001: PipelineRun 启动入口归 skill-runtime).

/// Issue type — maps to default pipeline templates in legacy popsicle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueType {
    Product,
    Technical,
    Bug,
    Idea,
}

impl IssueType {
    pub fn default_pipeline(&self) -> Option<&'static str> {
        match self {
            Self::Product => Some("full-sdlc"),
            Self::Technical => Some("tech-sdlc"),
            Self::Bug => Some("test-only"),
            Self::Idea => Some("design-only"),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Product => "product",
            Self::Technical => "technical",
            Self::Bug => "bug",
            Self::Idea => "idea",
        }
    }
}

/// A work item that may spawn a [`crate::pipeline_session::PipelineSession`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub key: String,
    pub title: String,
    pub description: String,
    pub issue_type: IssueType,
    /// Explicit pipeline override (e.g. `slice-delivery`).
    pub pipeline: Option<String>,
    pub spec_id: String,
}

impl Issue {
    /// Resolve the pipeline name: explicit override or type default.
    pub fn resolved_pipeline(&self) -> Option<&str> {
        self.pipeline
            .as_deref()
            .or_else(|| self.issue_type.default_pipeline())
    }
}
