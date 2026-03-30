use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A project groups related topics under a single initiative or epic.
/// Projects are optional — topics can exist without a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    /// URL-safe slug (auto-generated from name).
    pub slug: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_project_status")]
    pub status: ProjectStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_project_status() -> ProjectStatus {
    ProjectStatus::Active
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Active,
    Completed,
    Archived,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for ProjectStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown project status: {s}")),
        }
    }
}

impl Project {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        let slug = super::topic::slugify(&name);
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            slug,
            description: description.into(),
            status: ProjectStatus::Active,
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
    fn test_new_project() {
        let proj = Project::new("User System", "Auth and authorization");
        assert_eq!(proj.slug, "user-system");
        assert_eq!(proj.status, ProjectStatus::Active);
        assert!(!proj.id.is_empty());
    }

    #[test]
    fn test_project_status_roundtrip() {
        for status in &["active", "completed", "archived"] {
            let parsed: ProjectStatus = status.parse().unwrap();
            assert_eq!(&parsed.to_string(), status);
        }
    }
}
