use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use reqwest::{Client, Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;
use uuid::Uuid;

use crate::SCHEMA_VERSION;
use crate::client::SyncClient;
use crate::error::{Result, SyncError};
use crate::types::*;

/// Authentication credentials persisted by the CLI.
#[derive(Debug, Clone, Default)]
pub struct Credentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

/// HTTP/JSON [`SyncClient`] talking to a popsicle-cloud-compatible server.
pub struct HttpSyncClient {
    base: Url,
    http: Client,
    creds: Arc<RwLock<Credentials>>,
}

impl HttpSyncClient {
    pub fn new(base_url: &str, creds: Credentials) -> Result<Self> {
        let base = Url::parse(base_url)?;
        if base.scheme() != "https"
            && !matches!(base.host_str(), Some("localhost") | Some("127.0.0.1"))
        {
            return Err(SyncError::Other(format!(
                "sync endpoint must be https (got {})",
                base
            )));
        }
        let http = Client::builder()
            .user_agent(concat!("popsicle-sync/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            base,
            http,
            creds: Arc::new(RwLock::new(creds)),
        })
    }

    pub fn credentials(&self) -> Credentials {
        self.creds.read().expect("creds poisoned").clone()
    }

    pub fn set_credentials(&self, c: Credentials) {
        *self.creds.write().expect("creds poisoned") = c;
    }

    fn url(&self, path: &str) -> Result<Url> {
        Ok(self.base.join(path)?)
    }

    async fn send_json<B: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        auth: bool,
    ) -> Result<R> {
        let url = self.url(path)?;
        let mut req = self.http.request(method, url);
        if auth {
            let creds = self.creds.read().expect("creds poisoned");
            let token = creds
                .access_token
                .as_ref()
                .ok_or(SyncError::Unauthenticated)?;
            req = req.bearer_auth(token);
        }
        if let Some(b) = body {
            req = req.json(b);
        }
        let resp = req.send().await?;
        Self::parse_response(resp).await
    }

    async fn send_no_response<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        auth: bool,
    ) -> Result<()> {
        let url = self.url(path)?;
        let mut req = self.http.request(method, url);
        if auth {
            let creds = self.creds.read().expect("creds poisoned");
            let token = creds
                .access_token
                .as_ref()
                .ok_or(SyncError::Unauthenticated)?;
            req = req.bearer_auth(token);
        }
        if let Some(b) = body {
            req = req.json(b);
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        Err(Self::error_from(
            status,
            resp.text().await.unwrap_or_default(),
        ))
    }

    async fn parse_response<R: DeserializeOwned>(resp: reqwest::Response) -> Result<R> {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if status.is_success() {
            return Ok(serde_json::from_str(&text)?);
        }
        Err(Self::error_from(status, text))
    }

    fn error_from(status: StatusCode, body: String) -> SyncError {
        if let Ok(env) = serde_json::from_str::<ApiErrorEnvelope>(&body) {
            if env.error.code == "schema_version_mismatch" {
                let server = env
                    .error
                    .details
                    .as_ref()
                    .and_then(|d| d.get("server"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                return SyncError::SchemaMismatch {
                    server,
                    client: SCHEMA_VERSION,
                };
            }
            if status == StatusCode::UNAUTHORIZED {
                return SyncError::Unauthenticated;
            }
            return SyncError::Server {
                code: env.error.code,
                message: env.error.message,
            };
        }
        SyncError::Server {
            code: status.as_u16().to_string(),
            message: body,
        }
    }
}

#[async_trait]
impl SyncClient for HttpSyncClient {
    async fn register(&self, req: RegisterRequest) -> Result<AuthTokens> {
        self.send_json(Method::POST, "v1/auth/register", Some(&req), false)
            .await
    }

    async fn login(&self, req: LoginRequest) -> Result<AuthTokens> {
        self.send_json(Method::POST, "v1/auth/login", Some(&req), false)
            .await
    }

    async fn refresh(&self, req: RefreshRequest) -> Result<AccessToken> {
        self.send_json(Method::POST, "v1/auth/refresh", Some(&req), false)
            .await
    }

    async fn logout(&self, refresh_token: &str) -> Result<()> {
        let body = RefreshRequest {
            refresh_token: refresh_token.to_string(),
        };
        self.send_no_response(Method::POST, "v1/auth/logout", Some(&body), true)
            .await
    }

    async fn me(&self) -> Result<User> {
        self.send_json::<(), User>(Method::GET, "v1/me", None, true)
            .await
    }

    async fn pull_changes(&self, since: u64, limit: usize) -> Result<ChangesPage> {
        let path = format!("v1/sync/changes?since={}&limit={}", since, limit);
        self.send_json::<(), ChangesPage>(Method::GET, &path, None, true)
            .await
    }

    async fn push(&self, req: PushRequest) -> Result<PushResponse> {
        self.send_json(Method::POST, "v1/sync/push", Some(&req), true)
            .await
    }

    async fn doc_state(&self, doc_id: Uuid) -> Result<DocState> {
        let path = format!("v1/sync/documents/{}/state", doc_id);
        self.send_json::<(), DocState>(Method::GET, &path, None, true)
            .await
    }

    async fn doc_apply_updates(
        &self,
        doc_id: Uuid,
        updates: DocUpdates,
    ) -> Result<DocUpdatesResponse> {
        let path = format!("v1/sync/documents/{}/updates", doc_id);
        self.send_json(Method::POST, &path, Some(&updates), true)
            .await
    }
}
