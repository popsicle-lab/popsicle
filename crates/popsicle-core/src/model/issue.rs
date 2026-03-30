use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub issue_type: IssueType,
    pub priority: Priority,
    pub status: IssueStatus,
    /// The spec this issue belongs to.
    pub spec_id: String,
    /// Explicitly chosen pipeline template name (bypasses recommender).
    pub pipeline: Option<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Product,
    Technical,
    Bug,
    Idea,
}

impl IssueType {
    /// Returns the default pipeline name for this issue type.
    pub fn default_pipeline(&self) -> Option<&'static str> {
        match self {
            Self::Product => Some("full-sdlc"),
            Self::Technical => Some("tech-sdlc"),
            Self::Bug => Some("test-only"),
            Self::Idea => Some("design-only"),
        }
    }
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Product => write!(f, "product"),
            Self::Technical => write!(f, "technical"),
            Self::Bug => write!(f, "bug"),
            Self::Idea => write!(f, "idea"),
        }
    }
}

impl std::str::FromStr for IssueType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "product" => Ok(Self::Product),
            "technical" => Ok(Self::Technical),
            "bug" => Ok(Self::Bug),
            "idea" => Ok(Self::Idea),
            _ => Err(format!("Unknown issue type: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            _ => Err(format!("Unknown priority: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Backlog,
    Ready,
    InProgress,
    Done,
    Cancelled,
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Backlog => write!(f, "backlog"),
            Self::Ready => write!(f, "ready"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Done => write!(f, "done"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for IssueStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "backlog" => Ok(Self::Backlog),
            "ready" => Ok(Self::Ready),
            "in_progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown issue status: {s}")),
        }
    }
}

impl Issue {
    pub fn new(
        key: String,
        title: &str,
        issue_type: IssueType,
        spec_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            title: title.to_string(),
            description: String::new(),
            issue_type,
            priority: Priority::Medium,
            status: IssueStatus::Backlog,
            spec_id: spec_id.into(),
            pipeline: None,
            labels: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
