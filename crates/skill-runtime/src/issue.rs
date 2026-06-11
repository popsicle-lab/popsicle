//! Minimal `issue` entity (PDR-001: PipelineRun 启动入口归 skill-runtime).

/// Issue type — maps to bundled default pipeline templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueType {
    Product,
    Technical,
    Bug,
    Idea,
}

impl IssueType {
    /// Default pipeline per issue type. Every name must exist in the bundled
    /// templates (ADR-012 closes D-101: the legacy `full-sdlc`/`tech-sdlc`/
    /// `test-only`/`design-only` names were never shipped).
    pub fn default_pipeline(&self) -> Option<&'static str> {
        match self {
            Self::Product => Some("greenfield-product-spec"),
            Self::Technical => Some("tech-decision"),
            Self::Bug => Some("bugfix"),
            Self::Idea => Some("tech-decision"),
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
