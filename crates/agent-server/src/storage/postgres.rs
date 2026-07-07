//! PostgreSQL storage backend (`AGENT_RUNTIME_DATABASE_URL`).

use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::approval::ConfirmTask;
use crate::chat::{
    BootstrapTask, ChatMessage, ChatSession, ChatSessionView, ChatStore, ChatTurnTask,
    CompleteBootstrapRequest, CompleteChatTurnRequest, CreateChatSessionRequest,
};
use crate::run_log::{RunLogEntry, MAX_LINES_PER_RUN};
use crate::run_mirror::{RunMirror, RunMirrorUpsert, StageMirror};
use crate::runtime::RuntimeState;
use crate::{DispatchRequest, DispatchTask, TaskPhase};

const SCHEMA_SQL: &str = include_str!("../../../../deploy/agent-runtime/schema.sql");
const DEFAULT_HEARTBEAT_TTL_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct PostgresStorage {
    pool: PgPool,
    heartbeat_ttl: Duration,
    /// Chat intake (PDR-002): process-local until PG chat tables are wired.
    chat: std::sync::Arc<std::sync::Mutex<ChatStore>>,
}

impl PostgresStorage {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        for statement in SCHEMA_SQL
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            sqlx::query(statement).execute(&pool).await?;
        }
        Ok(Self {
            pool,
            heartbeat_ttl: Duration::from_secs(
                std::env::var("AGENT_RUNTIME_HEARTBEAT_TTL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_HEARTBEAT_TTL_SECS),
            ),
            chat: std::sync::Arc::new(std::sync::Mutex::new(ChatStore::default())),
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn dispatch(&self, req: DispatchRequest) -> Result<DispatchTask, sqlx::Error> {
        let id = Uuid::new_v4();
        let phase = TaskPhase::Queued.as_str();
        sqlx::query(
            r#"
            INSERT INTO dispatch_tasks (id, workspace_id, runtime_id, issue_key, pipeline, phase)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(&req.workspace_id)
        .bind(&req.runtime_id)
        .bind(&req.issue_key)
        .bind(&req.pipeline)
        .bind(phase)
        .execute(&self.pool)
        .await?;
        Ok(DispatchTask {
            id,
            workspace_id: req.workspace_id,
            runtime_id: req.runtime_id,
            issue_key: req.issue_key,
            pipeline: req.pipeline,
            phase: TaskPhase::Queued,
            run_id: None,
        })
    }

    pub async fn resume_dispatch(
        &self,
        workspace_id: String,
        runtime_id: String,
        issue_key: String,
        pipeline: String,
        run_id: String,
    ) -> Result<DispatchTask, sqlx::Error> {
        let id = Uuid::new_v4();
        let phase = TaskPhase::Queued.as_str();
        sqlx::query(
            r#"
            INSERT INTO dispatch_tasks (id, workspace_id, runtime_id, issue_key, pipeline, phase, run_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(&workspace_id)
        .bind(&runtime_id)
        .bind(&issue_key)
        .bind(&pipeline)
        .bind(phase)
        .bind(&run_id)
        .execute(&self.pool)
        .await?;
        Ok(DispatchTask {
            id,
            workspace_id,
            runtime_id,
            issue_key,
            pipeline,
            phase: TaskPhase::Queued,
            run_id: Some(run_id),
        })
    }

    pub async fn has_queued_resume(
        &self,
        runtime_id: &str,
        run_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM dispatch_tasks
            WHERE runtime_id = $1 AND phase = 'queued' AND run_id = $2
            "#,
        )
        .bind(runtime_id)
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row > 0)
    }

    pub async fn claim_dispatch(
        &self,
        runtime_id: &str,
    ) -> Result<Option<DispatchTask>, sqlx::Error> {
        let row = sqlx::query_as::<_, DispatchTaskRow>(
            r#"
            UPDATE dispatch_tasks
            SET phase = 'dispatched'
            WHERE id = (
                SELECT id FROM dispatch_tasks
                WHERE runtime_id = $1 AND phase = 'queued'
                ORDER BY created_at
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, workspace_id, runtime_id, issue_key, pipeline, phase, run_id
            "#,
        )
        .bind(runtime_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    pub async fn get_dispatch(&self, id: Uuid) -> Result<Option<DispatchTask>, sqlx::Error> {
        let row = sqlx::query_as::<_, DispatchTaskRow>(
            r#"
            SELECT id, workspace_id, runtime_id, issue_key, pipeline, phase, run_id
            FROM dispatch_tasks WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    pub async fn heartbeat(&self, runtime_id: &str) -> Result<RuntimeState, sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO runtime_heartbeats (runtime_id, last_heartbeat_at)
            VALUES ($1, NOW())
            ON CONFLICT (runtime_id) DO UPDATE
            SET last_heartbeat_at = EXCLUDED.last_heartbeat_at
            "#,
        )
        .bind(runtime_id)
        .execute(&self.pool)
        .await?;
        Ok(RuntimeState::Online)
    }

    pub async fn runtime_state(&self, runtime_id: &str) -> Result<RuntimeState, sqlx::Error> {
        let row = sqlx::query_as::<_, HeartbeatRow>(
            "SELECT last_heartbeat_at FROM runtime_heartbeats WHERE runtime_id = $1",
        )
        .bind(runtime_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .filter(|r| {
                Utc::now()
                    .signed_duration_since(r.last_heartbeat_at)
                    .to_std()
                    .map(|d| d <= self.heartbeat_ttl)
                    .unwrap_or(false)
            })
            .map(|_| RuntimeState::Online)
            .unwrap_or(RuntimeState::Offline))
    }

    pub async fn is_online(&self, runtime_id: &str) -> Result<bool, sqlx::Error> {
        Ok(matches!(
            self.runtime_state(runtime_id).await?,
            RuntimeState::Online
        ))
    }

    pub async fn queue_confirm(
        &self,
        runtime_id: &str,
        run_id: &str,
        stage: &str,
    ) -> Result<ConfirmTask, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO confirm_tasks (id, runtime_id, run_id, stage, phase)
            VALUES ($1, $2, $3, $4, 'queued')
            "#,
        )
        .bind(id)
        .bind(runtime_id)
        .bind(run_id)
        .bind(stage)
        .execute(&self.pool)
        .await?;
        Ok(ConfirmTask {
            id,
            runtime_id: runtime_id.to_string(),
            run_id: run_id.to_string(),
            stage: stage.to_string(),
            phase: TaskPhase::Queued,
        })
    }

    pub async fn claim_confirm(
        &self,
        runtime_id: &str,
    ) -> Result<Option<ConfirmTask>, sqlx::Error> {
        let row = sqlx::query_as::<_, ConfirmTaskRow>(
            r#"
            UPDATE confirm_tasks
            SET phase = 'dispatched'
            WHERE id = (
                SELECT id FROM confirm_tasks
                WHERE runtime_id = $1 AND phase = 'queued'
                ORDER BY created_at
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, runtime_id, run_id, stage, phase
            "#,
        )
        .bind(runtime_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    pub async fn upsert_mirror(
        &self,
        run_id: &str,
        upsert: RunMirrorUpsert,
    ) -> Result<RunMirror, sqlx::Error> {
        let mirror = crate::run_mirror::build_mirror(run_id, upsert);
        let stages_json =
            serde_json::to_value(&mirror.stages).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        sqlx::query(
            r#"
            INSERT INTO run_mirrors (run_id, issue_key, pipeline, run_status, current_stage, stages, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (run_id) DO UPDATE SET
                issue_key = COALESCE(EXCLUDED.issue_key, run_mirrors.issue_key),
                pipeline = EXCLUDED.pipeline,
                run_status = EXCLUDED.run_status,
                current_stage = EXCLUDED.current_stage,
                stages = EXCLUDED.stages,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&mirror.run_id)
        .bind(&mirror.issue_key)
        .bind(&mirror.pipeline)
        .bind(&mirror.run_status)
        .bind(&mirror.current_stage)
        .bind(stages_json)
        .bind(mirror.updated_at as i64)
        .execute(&self.pool)
        .await?;
        Ok(mirror)
    }

    pub async fn get_mirror(&self, run_id: &str) -> Result<Option<RunMirror>, sqlx::Error> {
        let row = sqlx::query_as::<_, RunMirrorRow>(
            r#"
            SELECT run_id, issue_key, pipeline, run_status, current_stage, stages, updated_at
            FROM run_mirrors WHERE run_id = $1
            "#,
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    pub async fn list_mirrors(&self) -> Result<Vec<RunMirror>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RunMirrorRow>(
            r#"
            SELECT run_id, issue_key, pipeline, run_status, current_stage, stages, updated_at
            FROM run_mirrors ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn append_run_log(
        &self,
        run_id: &str,
        level: &str,
        message: &str,
    ) -> Result<RunLogEntry, sqlx::Error> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let entry = sqlx::query_as::<_, RunLogRow>(
            r#"
            INSERT INTO run_logs (run_id, ts, level, message)
            VALUES ($1, $2, $3, $4)
            RETURNING ts, level, message
            "#,
        )
        .bind(run_id)
        .bind(ts)
        .bind(level)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM run_logs
            WHERE run_id = $1
              AND id NOT IN (
                SELECT id FROM run_logs
                WHERE run_id = $1
                ORDER BY ts DESC
                LIMIT $2
              )
            "#,
        )
        .bind(run_id)
        .bind(MAX_LINES_PER_RUN as i64)
        .execute(&self.pool)
        .await?;
        Ok(entry.into())
    }

    pub async fn list_run_logs(
        &self,
        run_id: &str,
        limit: usize,
    ) -> Result<Vec<RunLogEntry>, sqlx::Error> {
        let mut rows = sqlx::query_as::<_, RunLogRow>(
            r#"
            SELECT ts, level, message FROM run_logs
            WHERE run_id = $1
            ORDER BY ts DESC
            LIMIT $2
            "#,
        )
        .bind(run_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        rows.reverse();
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn create_chat_session(
        &self,
        req: CreateChatSessionRequest,
    ) -> Result<ChatSession, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .create_session(req))
    }

    pub async fn get_chat_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<ChatSessionView>, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .get_session_view(session_id))
    }

    pub async fn post_chat_user_message(
        &self,
        session_id: Uuid,
        content: &str,
    ) -> Result<Option<(ChatMessage, ChatTurnTask)>, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .post_user_message(session_id, content))
    }

    pub async fn claim_chat_turn(
        &self,
        runtime_id: &str,
    ) -> Result<Option<ChatTurnTask>, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .claim_chat_turn(runtime_id))
    }

    pub async fn complete_chat_turn(
        &self,
        session_id: Uuid,
        req: CompleteChatTurnRequest,
    ) -> Result<Option<ChatSessionView>, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .complete_chat_turn(req)
            .filter(|v| v.session.id == session_id))
    }

    pub async fn queue_bootstrap(
        &self,
        session_id: Uuid,
    ) -> Result<Option<BootstrapTask>, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .queue_bootstrap(session_id))
    }

    pub async fn claim_bootstrap(
        &self,
        runtime_id: &str,
    ) -> Result<Option<BootstrapTask>, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .claim_bootstrap(runtime_id))
    }

    pub async fn complete_bootstrap(
        &self,
        session_id: Uuid,
        req: CompleteBootstrapRequest,
    ) -> Result<Option<ChatSessionView>, sqlx::Error> {
        Ok(self
            .chat
            .lock()
            .map_err(|_| sqlx::Error::Protocol("chat lock poisoned".into()))?
            .complete_bootstrap(req)
            .filter(|v| v.session.id == session_id))
    }
}

#[derive(sqlx::FromRow)]
struct DispatchTaskRow {
    id: Uuid,
    workspace_id: String,
    runtime_id: String,
    issue_key: String,
    pipeline: String,
    phase: String,
    run_id: Option<String>,
}

impl From<DispatchTaskRow> for DispatchTask {
    fn from(row: DispatchTaskRow) -> Self {
        Self {
            id: row.id,
            workspace_id: row.workspace_id,
            runtime_id: row.runtime_id,
            issue_key: row.issue_key,
            pipeline: row.pipeline,
            phase: TaskPhase::parse_phase(&row.phase).unwrap_or(TaskPhase::Queued),
            run_id: row.run_id,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ConfirmTaskRow {
    id: Uuid,
    runtime_id: String,
    run_id: String,
    stage: String,
    phase: String,
}

impl From<ConfirmTaskRow> for ConfirmTask {
    fn from(row: ConfirmTaskRow) -> Self {
        Self {
            id: row.id,
            runtime_id: row.runtime_id,
            run_id: row.run_id,
            stage: row.stage,
            phase: TaskPhase::parse_phase(&row.phase).unwrap_or(TaskPhase::Queued),
        }
    }
}

#[derive(sqlx::FromRow)]
struct HeartbeatRow {
    last_heartbeat_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct RunMirrorRow {
    run_id: String,
    issue_key: Option<String>,
    pipeline: String,
    run_status: String,
    current_stage: String,
    stages: serde_json::Value,
    updated_at: i64,
}

#[derive(sqlx::FromRow)]
struct RunLogRow {
    ts: i64,
    level: String,
    message: String,
}

impl From<RunLogRow> for RunLogEntry {
    fn from(row: RunLogRow) -> Self {
        Self {
            ts: row.ts as u64,
            level: row.level,
            message: row.message,
        }
    }
}

impl From<RunMirrorRow> for RunMirror {
    fn from(row: RunMirrorRow) -> Self {
        let stages: Vec<StageMirror> = serde_json::from_value(row.stages).unwrap_or_default();
        let issue_key = row.issue_key.filter(|k| !k.is_empty() && k != "UNKNOWN");
        Self {
            run_id: row.run_id,
            issue_key,
            pipeline: row.pipeline,
            run_status: row.run_status,
            current_stage: row.current_stage,
            stages,
            updated_at: row.updated_at as u64,
        }
    }
}
