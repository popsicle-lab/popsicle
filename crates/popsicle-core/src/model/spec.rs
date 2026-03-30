use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A spec groups related pipeline runs and documents under a single
/// feature or initiative.  Documents and runs always belong to a spec,
/// enabling cross-pipeline reuse and revision tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub id: String,
    pub name: String,
    /// URL-safe slug used as the `{slug}` prefix in artifact file patterns.
    pub slug: String,
    #[serde(default)]
    pub description: String,
    /// Parent namespace this spec belongs to (required).
    #[serde(default)]
    pub namespace_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Exclusive lock: the pipeline run currently operating on this spec.
    #[serde(default)]
    pub locked_by_run_id: Option<String>,
    #[serde(default)]
    pub locked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Spec {
    /// Create a new spec. The slug is auto-generated from `name` if not
    /// supplied (lowercased, spaces/underscores → hyphens, non-alphanumeric stripped).
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        namespace_id: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let slug = slugify(&name);
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            slug,
            description: description.into(),
            namespace_id: namespace_id.into(),
            tags: Vec::new(),
            locked_by_run_id: None,
            locked_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Produce a URL-safe slug from an arbitrary string.
pub fn slugify(s: &str) -> String {
    let mut slug: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse consecutive hyphens
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("JWT Migration"), "jwt-migration");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Login: Cookie → JWT"), "login-cookie-jwt");
    }

    #[test]
    fn test_slugify_underscores_and_spaces() {
        assert_eq!(slugify("user_auth feature"), "user-auth-feature");
    }

    #[test]
    fn test_slugify_leading_trailing() {
        assert_eq!(slugify("  --hello-- "), "hello");
    }

    #[test]
    fn test_new_spec_generates_slug() {
        let spec = Spec::new("Add User Auth", "Implement JWT auth", "proj-1");
        assert_eq!(spec.slug, "add-user-auth");
        assert_eq!(spec.name, "Add User Auth");
        assert_eq!(spec.description, "Implement JWT auth");
        assert!(!spec.id.is_empty());
    }

    #[test]
    fn test_new_spec_timestamps() {
        let before = Utc::now();
        let spec = Spec::new("Test", "", "proj-1");
        let after = Utc::now();
        assert!(spec.created_at >= before && spec.created_at <= after);
        assert!(spec.updated_at >= before && spec.updated_at <= after);
    }
}
