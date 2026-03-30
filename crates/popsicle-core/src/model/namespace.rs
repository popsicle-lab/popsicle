use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A namespace groups related topics under a single initiative or epic.
/// Namespaces are optional — topics can exist without a namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub id: String,
    pub name: String,
    /// URL-safe slug (auto-generated from name).
    pub slug: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_namespace_status")]
    pub status: NamespaceStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_namespace_status() -> NamespaceStatus {
    NamespaceStatus::Active
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamespaceStatus {
    Active,
    Completed,
    Archived,
}

impl std::fmt::Display for NamespaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for NamespaceStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown namespace status: {s}")),
        }
    }
}

impl Namespace {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        let slug = super::topic::slugify(&name);
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            slug,
            description: description.into(),
            status: NamespaceStatus::Active,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_namespace() {
        let ns = Namespace::new("User System", "Auth and authorization");
        assert_eq!(ns.slug, "user-system");
        assert_eq!(ns.status, NamespaceStatus::Active);
        assert!(!ns.id.is_empty());
    }

    #[test]
    fn test_namespace_status_roundtrip() {
        for status in &["active", "completed", "archived"] {
            let parsed: NamespaceStatus = status.parse().unwrap();
            assert_eq!(&parsed.to_string(), status);
        }
    }
}
