//! Canonical filesystem path derivation for syncable entities.
//!
//! The server is path-agnostic: it stores entity payloads as JSONB and never
//! sees the local filesystem layout. The client renders entities to disk
//! using deterministic rules that read slug-style identifiers from the
//! payload. This module is the single source of truth for those rules.
//!
//! Layout (rooted at `.popsicle/`):
//!
//! ```text
//! specs/<spec_slug>/spec.md
//! specs/<spec_slug>/issues/<issue_slug>/issue.md
//! specs/<spec_slug>/issues/<issue_slug>/bugs/<bug_slug>.md
//! specs/<spec_slug>/issues/<issue_slug>/stories/<story_slug>.md
//! specs/<spec_slug>/issues/<issue_slug>/tests/<tc_slug>.md
//! artifacts/<run_slug>/<doc_type>.md
//! skills/<skill_name>/skill.yaml
//! pipelines/<pipeline_name>.yaml
//! ```
//!
//! All slugs are filename-safe (`[a-z0-9_-]+`). The renderer falls back to a
//! UUID-based filename if a slug field is missing or invalid.

use std::path::PathBuf;

use serde_json::Value;
use uuid::Uuid;

use crate::types::EntityKind;

/// Compute the on-disk path (relative to the `.popsicle/` root) for an entity.
///
/// `id` is used as a fallback identifier when the payload is missing the
/// preferred slug. Returns `None` for `EntityKind::Namespace`, which has no
/// dedicated file.
pub fn canonical_path(kind: EntityKind, id: Uuid, payload: &Value) -> Option<PathBuf> {
    match kind {
        EntityKind::Namespace => None,
        EntityKind::Spec => {
            let slug = string_field(payload, "slug")
                .or_else(|| string_field(payload, "id"))
                .unwrap_or_else(|| id.to_string());
            Some(PathBuf::from("specs").join(safe_slug(&slug)).join("spec.md"))
        }
        EntityKind::Issue => {
            let spec = string_field(payload, "spec_slug")
                .or_else(|| string_field(payload, "spec_id"))
                .unwrap_or_else(|| "_unknown".into());
            let slug = string_field(payload, "slug")
                .or_else(|| string_field(payload, "key"))
                .or_else(|| string_field(payload, "id"))
                .unwrap_or_else(|| id.to_string());
            Some(
                PathBuf::from("specs")
                    .join(safe_slug(&spec))
                    .join("issues")
                    .join(safe_slug(&slug))
                    .join("issue.md"),
            )
        }
        EntityKind::PipelineRun => {
            let slug = string_field(payload, "run_slug")
                .or_else(|| string_field(payload, "slug"))
                .unwrap_or_else(|| id.to_string());
            // Runs are directories, not single files; we represent them with a
            // sentinel `run.md` file containing the run metadata.
            Some(
                PathBuf::from("artifacts")
                    .join(safe_slug(&slug))
                    .join("run.md"),
            )
        }
        EntityKind::Document => {
            let run = string_field(payload, "run_slug")
                .or_else(|| string_field(payload, "pipeline_run_id"))
                .unwrap_or_else(|| "_orphan".into());
            let kind = string_field(payload, "doc_type")
                .or_else(|| string_field(payload, "skill_name"))
                .unwrap_or_else(|| "document".into());
            // Use document id as filename suffix to disambiguate retries/forks.
            let short = id.simple().to_string();
            let filename = format!("{}-{}.md", safe_slug(&kind), &short[..8]);
            Some(PathBuf::from("artifacts").join(safe_slug(&run)).join(filename))
        }
        EntityKind::Bug | EntityKind::UserStory | EntityKind::TestCase => {
            let spec = string_field(payload, "spec_slug")
                .or_else(|| string_field(payload, "spec_id"))
                .unwrap_or_else(|| "_unknown".into());
            let issue = string_field(payload, "issue_slug")
                .or_else(|| string_field(payload, "issue_id"))
                .unwrap_or_else(|| "_unknown".into());
            let slug = string_field(payload, "slug")
                .or_else(|| string_field(payload, "key"))
                .unwrap_or_else(|| id.to_string());
            let dir = match kind {
                EntityKind::Bug => "bugs",
                EntityKind::UserStory => "stories",
                EntityKind::TestCase => "tests",
                _ => unreachable!(),
            };
            Some(
                PathBuf::from("specs")
                    .join(safe_slug(&spec))
                    .join("issues")
                    .join(safe_slug(&issue))
                    .join(dir)
                    .join(format!("{}.md", safe_slug(&slug))),
            )
        }
        EntityKind::Skill => {
            let name = string_field(payload, "name")
                .or_else(|| string_field(payload, "slug"))
                .unwrap_or_else(|| id.to_string());
            Some(
                PathBuf::from("skills")
                    .join(safe_slug(&name))
                    .join("skill.yaml"),
            )
        }
        EntityKind::Pipeline => {
            let name = string_field(payload, "name")
                .or_else(|| string_field(payload, "slug"))
                .unwrap_or_else(|| id.to_string());
            Some(PathBuf::from("pipelines").join(format!("{}.yaml", safe_slug(&name))))
        }
    }
}

fn string_field(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Coerce an arbitrary string into a filename-safe slug. Conservative: keeps
/// alphanumerics, dash, underscore; replaces everything else with `_` and
/// collapses runs of `_`. Empty input becomes `_`.
fn safe_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c);
            prev_underscore = c == '_';
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn spec_path_uses_slug() {
        let p = canonical_path(
            EntityKind::Spec,
            Uuid::nil(),
            &json!({ "slug": "auth-redesign" }),
        );
        assert_eq!(p.unwrap().to_string_lossy(), "specs/auth-redesign/spec.md");
    }

    #[test]
    fn document_path_combines_run_and_kind() {
        let id = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
        let p = canonical_path(
            EntityKind::Document,
            id,
            &json!({ "run_slug": "run-2024-01", "doc_type": "prd" }),
        )
        .unwrap();
        assert!(p.starts_with("artifacts/run-2024-01"));
        assert!(p.to_string_lossy().contains("prd-aaaaaaaa.md"));
    }

    #[test]
    fn skill_path() {
        let p = canonical_path(
            EntityKind::Skill,
            Uuid::nil(),
            &json!({ "name": "rfc-author" }),
        )
        .unwrap();
        assert_eq!(p.to_string_lossy(), "skills/rfc-author/skill.yaml");
    }

    #[test]
    fn unsafe_slug_is_sanitised() {
        assert_eq!(safe_slug("../escape"), "_escape");
    }

    #[test]
    fn fallback_to_uuid_when_slug_missing() {
        let id = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
        let p = canonical_path(EntityKind::Spec, id, &json!({})).unwrap();
        assert!(p.to_string_lossy().contains(&id.to_string()));
    }

    #[test]
    fn namespace_has_no_path() {
        assert!(canonical_path(EntityKind::Namespace, Uuid::nil(), &json!({})).is_none());
    }
}
