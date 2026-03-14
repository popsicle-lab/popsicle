use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::issue::Priority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bug {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub severity: BugSeverity,
    pub priority: Priority,
    pub status: BugStatus,
    pub steps_to_reproduce: Vec<String>,
    pub expected_behavior: String,
    pub actual_behavior: String,
    pub environment: Option<String>,
    pub stack_trace: Option<String>,
    pub source: BugSource,
    pub related_test_case_id: Option<String>,
    pub related_commit_sha: Option<String>,
    pub fix_commit_sha: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BugSeverity {
    Blocker,
    Critical,
    #[default]
    Major,
    Minor,
    Trivial,
}

impl std::fmt::Display for BugSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocker => write!(f, "blocker"),
            Self::Critical => write!(f, "critical"),
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Trivial => write!(f, "trivial"),
        }
    }
}

impl std::str::FromStr for BugSeverity {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "blocker" => Ok(Self::Blocker),
            "critical" => Ok(Self::Critical),
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "trivial" => Ok(Self::Trivial),
            _ => Err(format!("Unknown bug severity: {s}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BugStatus {
    #[default]
    Open,
    Confirmed,
    InProgress,
    Fixed,
    Verified,
    Closed,
    WontFix,
}

impl std::fmt::Display for BugStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Fixed => write!(f, "fixed"),
            Self::Verified => write!(f, "verified"),
            Self::Closed => write!(f, "closed"),
            Self::WontFix => write!(f, "wont_fix"),
        }
    }
}

impl std::str::FromStr for BugStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "confirmed" => Ok(Self::Confirmed),
            "in_progress" => Ok(Self::InProgress),
            "fixed" => Ok(Self::Fixed),
            "verified" => Ok(Self::Verified),
            "closed" => Ok(Self::Closed),
            "wont_fix" => Ok(Self::WontFix),
            _ => Err(format!("Unknown bug status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BugSource {
    #[default]
    Manual,
    TestFailure,
    DocExtracted,
}

impl std::fmt::Display for BugSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::TestFailure => write!(f, "test_failure"),
            Self::DocExtracted => write!(f, "doc_extracted"),
        }
    }
}

impl std::str::FromStr for BugSource {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "manual" => Ok(Self::Manual),
            "test_failure" => Ok(Self::TestFailure),
            "doc_extracted" => Ok(Self::DocExtracted),
            _ => Err(format!("Unknown bug source: {s}")),
        }
    }
}

impl Bug {
    pub fn new(key: String, title: &str, severity: BugSeverity) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            title: title.to_string(),
            description: String::new(),
            severity,
            priority: Priority::Medium,
            status: BugStatus::Open,
            steps_to_reproduce: Vec::new(),
            expected_behavior: String::new(),
            actual_behavior: String::new(),
            environment: None,
            stack_trace: None,
            source: BugSource::Manual,
            related_test_case_id: None,
            related_commit_sha: None,
            fix_commit_sha: None,
            issue_id: None,
            pipeline_run_id: None,
            labels: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
