use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A topic groups related pipeline runs and documents under a single
/// feature or initiative.  Documents and runs always belong to a topic,
/// enabling cross-pipeline reuse and revision tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    /// URL-safe slug used as the `{slug}` prefix in artifact file patterns.
    pub slug: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Topic {
    /// Create a new topic. The slug is auto-generated from `name` if not
    /// supplied (lowercased, spaces/underscores → hyphens, non-alphanumeric stripped).
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        let slug = slugify(&name);
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            slug,
            description: description.into(),
            tags: Vec::new(),
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
    fn test_new_topic_generates_slug() {
        let topic = Topic::new("Add User Auth", "Implement JWT auth");
        assert_eq!(topic.slug, "add-user-auth");
        assert_eq!(topic.name, "Add User Auth");
        assert_eq!(topic.description, "Implement JWT auth");
        assert!(!topic.id.is_empty());
    }

    #[test]
    fn test_new_topic_timestamps() {
        let before = Utc::now();
        let topic = Topic::new("Test", "");
        let after = Utc::now();
        assert!(topic.created_at >= before && topic.created_at <= after);
        assert!(topic.updated_at >= before && topic.updated_at <= after);
    }
}
