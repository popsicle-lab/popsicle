use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Discriminator for syncable entities. Matches `docs/sync-api.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Namespace,
    Spec,
    Issue,
    PipelineRun,
    Document,
    Bug,
    UserStory,
    TestCase,
}

impl EntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Namespace => "namespace",
            EntityKind::Spec => "spec",
            EntityKind::Issue => "issue",
            EntityKind::PipelineRun => "pipeline_run",
            EntityKind::Document => "document",
            EntityKind::Bug => "bug",
            EntityKind::UserStory => "user_story",
            EntityKind::TestCase => "test_case",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub access_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// One entity update from the server's change log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub kind: EntityKind,
    pub id: Uuid,
    pub version: u64,
    #[serde(default)]
    pub deleted: bool,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesPage {
    pub schema_version: u32,
    pub changes: Vec<Change>,
    pub next_since: u64,
    pub has_more: bool,
}

/// A single upsert/delete the client wants to push.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushOperation {
    pub kind: EntityKind,
    pub id: Uuid,
    /// The server `version` the client last saw. `None` for new entities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_version: Option<u64>,
    #[serde(default)]
    pub deleted: bool,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub schema_version: u32,
    pub client_id: Uuid,
    pub operations: Vec<PushOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushOutcome {
    Applied,
    Conflict,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    pub id: Uuid,
    pub status: PushOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub results: Vec<PushResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiError,
}

/// Body of a CRDT update batch for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocUpdates {
    pub schema_version: u32,
    /// Each entry is base64-encoded Yjs/yrs update bytes.
    pub updates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocUpdatesResponse {
    pub applied: Vec<String>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocState {
    pub state_vector: String,
    pub state: String,
    pub version: u64,
}
