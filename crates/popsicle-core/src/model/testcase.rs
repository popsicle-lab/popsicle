use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub test_type: TestType,
    pub priority_level: TestPriority,
    pub status: TestCaseStatus,
    pub preconditions: Vec<String>,
    pub steps: Vec<String>,
    pub expected_result: String,
    pub source_doc_id: Option<String>,
    pub user_story_id: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResult {
    pub id: String,
    pub test_case_id: String,
    pub passed: bool,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub commit_sha: Option<String>,
    pub run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestType {
    #[default]
    Unit,
    Api,
    E2e,
    Ui,
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Api => write!(f, "api"),
            Self::E2e => write!(f, "e2e"),
            Self::Ui => write!(f, "ui"),
        }
    }
}

impl std::str::FromStr for TestType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "unit" => Ok(Self::Unit),
            "api" => Ok(Self::Api),
            "e2e" => Ok(Self::E2e),
            "ui" => Ok(Self::Ui),
            _ => Err(format!("Unknown test type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestPriority {
    P0,
    #[default]
    P1,
    P2,
}

impl std::fmt::Display for TestPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P0 => write!(f, "p0"),
            Self::P1 => write!(f, "p1"),
            Self::P2 => write!(f, "p2"),
        }
    }
}

impl std::str::FromStr for TestPriority {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "p0" | "P0" => Ok(Self::P0),
            "p1" | "P1" => Ok(Self::P1),
            "p2" | "P2" => Ok(Self::P2),
            _ => Err(format!("Unknown test priority: {s}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCaseStatus {
    #[default]
    Draft,
    Ready,
    Automated,
    Deprecated,
}

impl std::fmt::Display for TestCaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Ready => write!(f, "ready"),
            Self::Automated => write!(f, "automated"),
            Self::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl std::str::FromStr for TestCaseStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "ready" => Ok(Self::Ready),
            "automated" => Ok(Self::Automated),
            "deprecated" => Ok(Self::Deprecated),
            _ => Err(format!("Unknown test case status: {s}")),
        }
    }
}

impl TestCase {
    pub fn new(key: String, title: &str, test_type: TestType) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            title: title.to_string(),
            description: String::new(),
            test_type,
            priority_level: TestPriority::P1,
            status: TestCaseStatus::Draft,
            preconditions: Vec::new(),
            steps: Vec::new(),
            expected_result: String::new(),
            source_doc_id: None,
            user_story_id: None,
            issue_id: None,
            pipeline_run_id: None,
            labels: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl TestRunResult {
    pub fn new(test_case_id: &str, passed: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            test_case_id: test_case_id.to_string(),
            passed,
            duration_ms: None,
            error_message: None,
            commit_sha: None,
            run_at: Utc::now(),
        }
    }
}
