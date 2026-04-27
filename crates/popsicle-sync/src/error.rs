use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Sync is disabled in config.toml")]
    Disabled,

    #[error("Not authenticated. Run `popsicle login`.")]
    Unauthenticated,

    #[error("Server schema version {server} is incompatible with client {client}")]
    SchemaMismatch { server: u32, client: u32 },

    #[error("Server returned error {code}: {message}")]
    Server { code: String, message: String },

    #[error("Conflict on entity {id}: {message}")]
    Conflict { id: String, message: String },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, SyncError>;
