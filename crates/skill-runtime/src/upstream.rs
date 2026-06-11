//! ADR-004: `UpstreamApprovalChecker` implementation in skill-runtime.

use std::collections::BTreeSet;

use artifact_system::document::Document;
use artifact_system::guard::{GuardResult, UpstreamApprovalChecker};

/// Checks upstream pipeline-stage approval using completed stage names + doc status.
///
/// Mirrors legacy `upstream_approved` guard semantics at a reduced fidelity suitable
/// for in-shadow: a document passes when its `status` is `final` **or** every
/// stage listed in `required_stages` is in `completed_stages`.
#[derive(Debug, Clone, Default)]
pub struct PipelineUpstreamChecker {
    pub completed_stages: BTreeSet<String>,
    /// When set, doc must be `final` unless all `required_stages` ⊆ completed.
    pub required_stages: BTreeSet<String>,
}

impl PipelineUpstreamChecker {
    pub fn with_completed(stages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            completed_stages: stages.into_iter().map(Into::into).collect(),
            required_stages: BTreeSet::new(),
        }
    }
}

impl UpstreamApprovalChecker for PipelineUpstreamChecker {
    fn check_upstream_approved(&self, doc: &Document) -> GuardResult {
        if doc.status == "final" {
            return GuardResult {
                passed: true,
                guard_name: "upstream_approved".into(),
                message: "Document is final.".into(),
            };
        }

        let required: BTreeSet<String> = if self.required_stages.is_empty() {
            doc.extra_frontmatter
                .get("upstream_stages")
                .map(|s| {
                    s.split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            self.required_stages.clone()
        };

        if required.is_empty() {
            return GuardResult {
                passed: false,
                guard_name: "upstream_approved".into(),
                message: "No upstream stage metadata and document is not final.".into(),
            };
        }

        let missing: Vec<_> = required
            .difference(&self.completed_stages)
            .cloned()
            .collect();

        if missing.is_empty() {
            GuardResult {
                passed: true,
                guard_name: "upstream_approved".into(),
                message: "All upstream stages completed.".into(),
            }
        } else {
            GuardResult {
                passed: false,
                guard_name: "upstream_approved".into(),
                message: format!("Upstream stages not completed: {}", missing.join(", ")),
            }
        }
    }
}
