use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::issue::Priority;

/// Unified work item: replaces the former Bug / UserStory / TestCase trio.
///
/// Kind-specific extras (steps, severity, acceptance criteria, …) live in
/// `fields` as a free-form JSON blob. This trades strict typing for a much
/// smaller surface — three storage tables + three CLI commands collapse to
/// one of each.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    /// Human-readable key, e.g. `BUG-PRJ-1`, `STORY-PRJ-1`, `TC-PRJ-1`.
    pub key: String,
    pub kind: WorkItemKind,
    pub title: String,
    pub description: String,
    /// Free-form status string (no enforced enum). Conventional values:
    /// - bug:   open | in_progress | fixed | wont_fix | closed
    /// - story: draft | accepted | implemented | verified
    /// - tc:    draft | ready | automated | deprecated
    pub status: String,
    pub priority: Priority,
    pub labels: Vec<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub source_doc_id: Option<String>,
    /// Kind-specific structured fields. Examples:
    /// - bug:   `{ severity, steps_to_reproduce, expected_behavior, actual_behavior, ... }`
    /// - story: `{ persona, goal, benefit, acceptance: ["..."] }`
    /// - tc:    `{ test_type, preconditions, steps, expected_result }`
    #[serde(default)]
    pub fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemKind {
    Bug,
    Story,
    TestCase,
}

impl WorkItemKind {
    /// Key prefix used when generating new keys (BUG-/STORY-/TC-).
    pub fn key_prefix(&self) -> &'static str {
        match self {
            Self::Bug => "BUG",
            Self::Story => "STORY",
            Self::TestCase => "TC",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Story => "story",
            Self::TestCase => "testcase",
        }
    }
}

impl std::fmt::Display for WorkItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for WorkItemKind {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bug" => Ok(Self::Bug),
            "story" | "user_story" | "userstory" => Ok(Self::Story),
            "tc" | "test" | "testcase" | "test_case" => Ok(Self::TestCase),
            other => Err(format!("Unknown work item kind: {other}")),
        }
    }
}

impl WorkItem {
    pub fn new(key: impl Into<String>, kind: WorkItemKind, title: impl Into<String>) -> Self {
        let now = Utc::now();
        let default_status = match kind {
            WorkItemKind::Bug => "open",
            WorkItemKind::Story => "draft",
            WorkItemKind::TestCase => "draft",
        };
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key: key.into(),
            kind,
            title: title.into(),
            description: String::new(),
            status: default_status.to_string(),
            priority: Priority::Medium,
            labels: Vec::new(),
            issue_id: None,
            pipeline_run_id: None,
            source_doc_id: None,
            fields: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
        }
    }

    /// Convenience getter for a string field inside `fields`.
    pub fn field_str(&self, name: &str) -> Option<&str> {
        self.fields.get(name).and_then(|v| v.as_str())
    }

    /// Convenience setter that ensures `fields` is an object before insertion.
    pub fn set_field(&mut self, name: &str, value: serde_json::Value) {
        if !self.fields.is_object() {
            self.fields = serde_json::Value::Object(serde_json::Map::new());
        }
        if let Some(map) = self.fields.as_object_mut() {
            map.insert(name.to_string(), value);
        }
    }
}
