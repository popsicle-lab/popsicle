//! Pluggable storage: in-memory (dev/tests) or PostgreSQL (self-host).

mod memory;
mod postgres;

pub use memory::MemoryStorage;
pub use postgres::PostgresStorage;

use std::sync::Arc;

use crate::approval::ConfirmTask;
use crate::run_log::RunLogEntry;
use crate::run_mirror::{RunMirror, RunMirrorUpsert};
use crate::runtime::RuntimeState;
use crate::{DispatchRequest, DispatchTask};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageKind {
    Memory,
    Postgres,
}

#[derive(Clone)]
pub enum Backend {
    Memory(Arc<MemoryStorage>),
    Postgres(Arc<PostgresStorage>),
}

impl Backend {
    pub fn memory() -> Self {
        Self::Memory(Arc::new(MemoryStorage::new()))
    }

    pub async fn from_env() -> Result<Self, sqlx::Error> {
        match std::env::var("AGENT_RUNTIME_DATABASE_URL") {
            Ok(url) if !url.trim().is_empty() => {
                let pg = PostgresStorage::connect(&url).await?;
                Ok(Self::Postgres(Arc::new(pg)))
            }
            _ => Ok(Self::memory()),
        }
    }

    pub fn kind(&self) -> StorageKind {
        match self {
            Self::Memory(_) => StorageKind::Memory,
            Self::Postgres(_) => StorageKind::Postgres,
        }
    }

    pub async fn dispatch(&self, req: DispatchRequest) -> Result<DispatchTask, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.tasks.lock().unwrap().dispatch(req)),
            Self::Postgres(pg) => Ok(pg.dispatch(req).await?),
        }
    }

    pub async fn resume_dispatch(
        &self,
        workspace_id: String,
        runtime_id: String,
        issue_key: String,
        pipeline: String,
        run_id: String,
    ) -> Result<DispatchTask, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.tasks.lock().unwrap().resume(
                workspace_id,
                runtime_id,
                issue_key,
                pipeline,
                run_id,
            )),
            Self::Postgres(pg) => Ok(pg
                .resume_dispatch(workspace_id, runtime_id, issue_key, pipeline, run_id)
                .await?),
        }
    }

    pub async fn has_queued_resume(
        &self,
        runtime_id: &str,
        run_id: &str,
    ) -> Result<bool, StorageError> {
        match self {
            Self::Memory(m) => Ok(m
                .tasks
                .lock()
                .unwrap()
                .has_queued_resume(runtime_id, run_id)),
            Self::Postgres(pg) => Ok(pg.has_queued_resume(runtime_id, run_id).await?),
        }
    }

    pub async fn claim_dispatch(
        &self,
        runtime_id: &str,
    ) -> Result<Option<DispatchTask>, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.tasks.lock().unwrap().claim_next(runtime_id)),
            Self::Postgres(pg) => Ok(pg.claim_dispatch(runtime_id).await?),
        }
    }

    pub async fn get_dispatch(&self, id: Uuid) -> Result<Option<DispatchTask>, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.tasks.lock().unwrap().get(id)),
            Self::Postgres(pg) => Ok(pg.get_dispatch(id).await?),
        }
    }

    pub async fn heartbeat(&self, runtime_id: &str) -> Result<RuntimeState, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.runtimes.lock().unwrap().heartbeat(runtime_id)),
            Self::Postgres(pg) => Ok(pg.heartbeat(runtime_id).await?),
        }
    }

    pub async fn runtime_state(&self, runtime_id: &str) -> Result<RuntimeState, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.runtimes.lock().unwrap().state(runtime_id)),
            Self::Postgres(pg) => Ok(pg.runtime_state(runtime_id).await?),
        }
    }

    pub async fn is_online(&self, runtime_id: &str) -> Result<bool, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.runtimes.lock().unwrap().is_online(runtime_id)),
            Self::Postgres(pg) => Ok(pg.is_online(runtime_id).await?),
        }
    }

    pub async fn queue_confirm(
        &self,
        runtime_id: &str,
        run_id: &str,
        stage: &str,
    ) -> Result<ConfirmTask, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.confirms.lock().unwrap().queue(runtime_id, run_id, stage)),
            Self::Postgres(pg) => Ok(pg.queue_confirm(runtime_id, run_id, stage).await?),
        }
    }

    pub async fn claim_confirm(
        &self,
        runtime_id: &str,
    ) -> Result<Option<ConfirmTask>, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.confirms.lock().unwrap().claim_next(runtime_id)),
            Self::Postgres(pg) => Ok(pg.claim_confirm(runtime_id).await?),
        }
    }

    pub async fn upsert_mirror(
        &self,
        run_id: &str,
        upsert: RunMirrorUpsert,
    ) -> Result<RunMirror, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.mirrors.lock().unwrap().upsert_from_status(run_id, upsert)),
            Self::Postgres(pg) => Ok(pg.upsert_mirror(run_id, upsert).await?),
        }
    }

    pub async fn get_mirror(&self, run_id: &str) -> Result<Option<RunMirror>, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.mirrors.lock().unwrap().get(run_id)),
            Self::Postgres(pg) => Ok(pg.get_mirror(run_id).await?),
        }
    }

    pub async fn list_mirrors(&self) -> Result<Vec<RunMirror>, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.mirrors.lock().unwrap().list()),
            Self::Postgres(pg) => Ok(pg.list_mirrors().await?),
        }
    }

    pub async fn append_run_log(
        &self,
        run_id: &str,
        level: &str,
        message: &str,
    ) -> Result<RunLogEntry, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.logs.lock().unwrap().append(run_id, level, message)),
            Self::Postgres(pg) => Ok(pg.append_run_log(run_id, level, message).await?),
        }
    }

    pub async fn list_run_logs(
        &self,
        run_id: &str,
        limit: usize,
    ) -> Result<Vec<RunLogEntry>, StorageError> {
        match self {
            Self::Memory(m) => Ok(m.logs.lock().unwrap().list(run_id, limit)),
            Self::Postgres(pg) => Ok(pg.list_run_logs(run_id, limit).await?),
        }
    }
}

#[derive(Debug)]
pub enum StorageError {
    Sql(sqlx::Error),
    Poison,
}

impl From<sqlx::Error> for StorageError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sql(value)
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(e) => write!(f, "{e}"),
            Self::Poison => write!(f, "lock poisoned"),
        }
    }
}

impl std::error::Error for StorageError {}
