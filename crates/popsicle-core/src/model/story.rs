use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::issue::Priority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub persona: String,
    pub goal: String,
    pub benefit: String,
    pub priority: Priority,
    pub status: UserStoryStatus,
    pub source_doc_id: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
    pub verified: bool,
    pub test_case_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStoryStatus {
    #[default]
    Draft,
    Accepted,
    Implemented,
    Verified,
}

impl std::fmt::Display for UserStoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Accepted => write!(f, "accepted"),
            Self::Implemented => write!(f, "implemented"),
            Self::Verified => write!(f, "verified"),
        }
    }
}

impl std::str::FromStr for UserStoryStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "accepted" => Ok(Self::Accepted),
            "implemented" => Ok(Self::Implemented),
            "verified" => Ok(Self::Verified),
            _ => Err(format!("Unknown user story status: {s}")),
        }
    }
}

impl UserStory {
    pub fn new(key: String, title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            title: title.to_string(),
            description: String::new(),
            persona: String::new(),
            goal: String::new(),
            benefit: String::new(),
            priority: Priority::Medium,
            status: UserStoryStatus::Draft,
            source_doc_id: None,
            issue_id: None,
            pipeline_run_id: None,
            acceptance_criteria: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl AcceptanceCriterion {
    pub fn new(description: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            verified: false,
            test_case_ids: Vec::new(),
        }
    }
}
