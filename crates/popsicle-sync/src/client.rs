use async_trait::async_trait;
use uuid::Uuid;

use crate::error::Result;
use crate::types::*;

/// Transport-agnostic interface for any popsicle-cloud-compatible server.
///
/// Implementations are responsible for: schema version checking, attaching
/// auth tokens, and translating HTTP errors into [`crate::SyncError`].
#[async_trait]
pub trait SyncClient: Send + Sync {
    // ---- auth ----
    async fn register(&self, req: RegisterRequest) -> Result<AuthTokens>;
    async fn login(&self, req: LoginRequest) -> Result<AuthTokens>;
    async fn refresh(&self, req: RefreshRequest) -> Result<AccessToken>;
    async fn logout(&self, refresh_token: &str) -> Result<()>;
    async fn me(&self) -> Result<User>;

    // ---- sync ----
    async fn pull_changes(&self, since: u64, limit: usize) -> Result<ChangesPage>;
    async fn push(&self, req: PushRequest) -> Result<PushResponse>;

    // ---- document CRDT ----
    async fn doc_state(&self, doc_id: Uuid) -> Result<DocState>;
    async fn doc_apply_updates(
        &self,
        doc_id: Uuid,
        updates: DocUpdates,
    ) -> Result<DocUpdatesResponse>;
}
