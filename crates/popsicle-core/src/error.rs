use thiserror::Error;

#[derive(Debug, Error)]
pub enum PopsicleError {
    #[error("Project not initialized. Run `popsicle init` first.")]
    NotInitialized,

    #[error("Project already initialized at {0}")]
    AlreadyInitialized(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid document format: {0}")]
    InvalidDocumentFormat(String),

    #[error("Invalid skill definition: {0}")]
    InvalidSkillDef(String),

    #[error("Workflow transition not allowed: {action} from state {from}")]
    InvalidTransition { from: String, action: String },

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, PopsicleError>;
