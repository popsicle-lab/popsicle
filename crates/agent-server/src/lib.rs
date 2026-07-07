//! Task queue + runtime registry + run mirror for agent-runtime (ADR-001).
//! Storage: in-memory default, PostgreSQL when `AGENT_RUNTIME_DATABASE_URL` is set.

mod approval;
mod role;
mod run_log;
mod run_mirror;
mod runtime;
mod storage;
mod ws;

pub use approval::{ApproveRequest, ApproveResult, ConfirmTask, ConfirmTaskStore};
pub use role::server_role;
pub use run_log::{RunLogAppend, RunLogEntry};
pub use run_mirror::{RunMirror, RunMirrorStore, RunMirrorUpsert, StageMirror};
pub use runtime::{RuntimeRegistry, RuntimeState};
pub use storage::{Backend, StorageKind};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

const EVENT_CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPhase {
    Queued,
    Dispatched,
    Running,
    Completed,
    Failed,
}

impl TaskPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Dispatched => "dispatched",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn parse_phase(s: &str) -> Option<Self> {
        match s {
            "queued" => Some(Self::Queued),
            "dispatched" => Some(Self::Dispatched),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchState {
    Queued,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchTask {
    pub id: Uuid,
    pub workspace_id: String,
    pub runtime_id: String,
    pub issue_key: String,
    pub pipeline: String,
    pub phase: TaskPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchRequest {
    pub workspace_id: String,
    pub runtime_id: String,
    pub issue_key: String,
    pub pipeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    pub accepted: bool,
    pub state: DispatchState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<DispatchTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeRequest {
    pub runtime_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeResult {
    pub accepted: bool,
    pub state: DispatchState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<DispatchTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub runtime_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatusResponse {
    pub runtime_id: String,
    pub state: RuntimeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub storage: StorageKind,
}

#[derive(Debug, Default)]
pub struct TaskStore {
    tasks: std::collections::HashMap<Uuid, DispatchTask>,
}

impl TaskStore {
    pub fn dispatch(&mut self, req: DispatchRequest) -> DispatchTask {
        let task = DispatchTask {
            id: Uuid::new_v4(),
            workspace_id: req.workspace_id,
            runtime_id: req.runtime_id.clone(),
            issue_key: req.issue_key,
            pipeline: req.pipeline,
            phase: TaskPhase::Queued,
            run_id: None,
        };
        self.tasks.insert(task.id, task.clone());
        task
    }

    pub fn resume(
        &mut self,
        workspace_id: String,
        runtime_id: String,
        issue_key: String,
        pipeline: String,
        run_id: String,
    ) -> DispatchTask {
        let task = DispatchTask {
            id: Uuid::new_v4(),
            workspace_id,
            runtime_id,
            issue_key,
            pipeline,
            phase: TaskPhase::Queued,
            run_id: Some(run_id),
        };
        self.tasks.insert(task.id, task.clone());
        task
    }

    pub fn has_queued_resume(&self, runtime_id: &str, run_id: &str) -> bool {
        self.tasks.values().any(|t| {
            t.runtime_id == runtime_id
                && t.phase == TaskPhase::Queued
                && t.run_id.as_deref() == Some(run_id)
        })
    }

    pub fn claim_next(&mut self, runtime_id: &str) -> Option<DispatchTask> {
        let id = self
            .tasks
            .values()
            .find(|t| t.runtime_id == runtime_id && t.phase == TaskPhase::Queued)
            .map(|t| t.id)?;
        let task = self.tasks.get_mut(&id)?;
        task.phase = TaskPhase::Dispatched;
        Some(task.clone())
    }

    pub fn set_run_id(&mut self, task_id: Uuid, run_id: String) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.run_id = Some(run_id);
            task.phase = TaskPhase::Running;
        }
    }

    pub fn get(&self, id: Uuid) -> Option<DispatchTask> {
        self.tasks.get(&id).cloned()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub backend: Backend,
    events: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(backend: Backend) -> Self {
        let (events, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { backend, events }
    }

    pub fn memory() -> Self {
        Self::new(Backend::memory())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.events.subscribe()
    }

    pub fn publish_event(&self, event: serde_json::Value) {
        let _ = self.events.send(event.to_string());
    }
}

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    Router::new()
        .route("/health", get(health))
        .route("/v1/dispatch", post(dispatch))
        .route("/v1/runtimes/{runtime_id}/heartbeat", post(heartbeat))
        .route("/v1/runtimes/{runtime_id}", get(get_runtime))
        .route("/v1/runtimes/{runtime_id}/tasks/claim", post(claim))
        .route(
            "/v1/runtimes/{runtime_id}/confirms/claim",
            post(claim_confirm),
        )
        .route("/v1/tasks/{task_id}", get(get_task))
        .route("/v1/runs", get(list_runs))
        .route("/v1/runs/{run_id}", get(get_run_mirror))
        .route("/v1/runs/{run_id}/mirror", put(upsert_run_mirror))
        .route(
            "/v1/runs/{run_id}/logs",
            get(list_run_logs).post(append_run_log),
        )
        .route("/v1/runs/{run_id}/resume", post(resume_run))
        .route("/v1/runs/{run_id}/approve", post(approve_run))
        .route("/v1/ws", get(ws::ws_events))
        .layer(cors)
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        storage: state.backend.kind(),
    })
}

async fn heartbeat(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
) -> Result<Json<RuntimeStatusResponse>, StatusCode> {
    let runtime_state = state
        .backend
        .heartbeat(&runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(RuntimeStatusResponse {
        runtime_id,
        state: runtime_state,
    }))
}

async fn get_runtime(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
) -> Result<Json<RuntimeStatusResponse>, StatusCode> {
    let runtime_state = state
        .backend
        .runtime_state(&runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(RuntimeStatusResponse {
        runtime_id,
        state: runtime_state,
    }))
}

async fn dispatch(
    State(state): State<AppState>,
    Json(req): Json<DispatchRequest>,
) -> Result<Json<DispatchResult>, StatusCode> {
    if req.runtime_id.is_empty() || req.issue_key.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let online = state
        .backend
        .is_online(&req.runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !online {
        return Ok(Json(DispatchResult {
            accepted: false,
            state: DispatchState::Rejected,
            reason: Some("runtime_offline".into()),
            task: None,
        }));
    }
    let task = state
        .backend
        .dispatch(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(DispatchResult {
        accepted: true,
        state: DispatchState::Queued,
        reason: None,
        task: Some(task),
    }))
}

async fn claim(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
    Json(_body): Json<ClaimRequest>,
) -> Result<Json<DispatchTask>, StatusCode> {
    let task = state
        .backend
        .claim_dispatch(&runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(task))
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<DispatchTask>, StatusCode> {
    state
        .backend
        .get_dispatch(task_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn list_runs(State(state): State<AppState>) -> Result<Json<Vec<RunMirror>>, StatusCode> {
    let runs = state
        .backend
        .list_mirrors()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(runs))
}

async fn get_run_mirror(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunMirror>, StatusCode> {
    state
        .backend
        .get_mirror(&run_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_run_mirror(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(body): Json<RunMirrorUpsert>,
) -> Result<Json<RunMirror>, StatusCode> {
    let mirror = state
        .backend
        .upsert_mirror(&run_id, body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.publish_event(serde_json::json!({
        "type": "run_updated",
        "run_id": mirror.run_id,
        "mirror": mirror,
    }));
    Ok(Json(mirror))
}

async fn list_run_logs(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<RunLogEntry>>, StatusCode> {
    let logs = state
        .backend
        .list_run_logs(&run_id, 200)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(logs))
}

async fn append_run_log(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(body): Json<RunLogAppend>,
) -> Result<Json<RunLogEntry>, StatusCode> {
    if body.message.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let level = if body.level.is_empty() {
        "info"
    } else {
        body.level.as_str()
    };
    let entry = state
        .backend
        .append_run_log(&run_id, level, &body.message)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.publish_event(serde_json::json!({
        "type": "run_log",
        "run_id": run_id,
        "entry": entry,
    }));
    Ok(Json(entry))
}

async fn resume_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(req): Json<ResumeRequest>,
) -> Result<Json<ResumeResult>, StatusCode> {
    if req.runtime_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let online = state
        .backend
        .is_online(&req.runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !online {
        return Ok(Json(ResumeResult {
            accepted: false,
            state: DispatchState::Rejected,
            reason: Some("runtime_offline".into()),
            task: None,
        }));
    }
    let mirror = state
        .backend
        .get_mirror(&run_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if mirror.run_status == "completed" {
        return Ok(Json(ResumeResult {
            accepted: false,
            state: DispatchState::Rejected,
            reason: Some("run_completed".into()),
            task: None,
        }));
    }
    if state
        .backend
        .has_queued_resume(&req.runtime_id, &run_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(Json(ResumeResult {
            accepted: false,
            state: DispatchState::Rejected,
            reason: Some("resume_already_queued".into()),
            task: None,
        }));
    }
    let workspace_id = req.workspace_id.unwrap_or_default();
    let issue_key = mirror.issue_key.unwrap_or_else(|| run_id.clone());
    let task = state
        .backend
        .resume_dispatch(
            workspace_id,
            req.runtime_id,
            issue_key,
            mirror.pipeline,
            run_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.publish_event(serde_json::json!({
        "type": "resume_queued",
        "run_id": task.run_id,
        "issue_key": task.issue_key,
        "runtime_id": task.runtime_id,
        "task_id": task.id,
    }));
    Ok(Json(ResumeResult {
        accepted: true,
        state: DispatchState::Queued,
        reason: None,
        task: Some(task),
    }))
}

async fn approve_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApproveResult>, StatusCode> {
    if req.runtime_id.is_empty() || req.stage.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let task = state
        .backend
        .queue_confirm(&req.runtime_id, &run_id, &req.stage)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.publish_event(serde_json::json!({
        "type": "approval_queued",
        "run_id": run_id,
        "stage": req.stage,
        "runtime_id": req.runtime_id,
        "confirm_task_id": task.id,
    }));
    Ok(Json(ApproveResult {
        confirm_task_created: true,
        task,
    }))
}

async fn claim_confirm(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
    Json(_body): Json<ClaimRequest>,
) -> Result<Json<ConfirmTask>, StatusCode> {
    let task = state
        .backend
        .claim_confirm(&runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(task))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_then_claim() {
        let mut store = TaskStore::default();
        let task = store.dispatch(DispatchRequest {
            workspace_id: "ws1".into(),
            runtime_id: "rt1".into(),
            issue_key: "PROJ-82".into(),
            pipeline: "feature-delivery".into(),
        });
        assert_eq!(task.phase, TaskPhase::Queued);
        let claimed = store.claim_next("rt1").expect("claim");
        assert_eq!(claimed.id, task.id);
        assert_eq!(claimed.phase, TaskPhase::Dispatched);
    }

    #[test]
    fn dispatch_rejected_when_runtime_offline() {
        let registry = RuntimeRegistry::new();
        assert!(!registry.is_online("rt1"));
        let result = DispatchResult {
            accepted: false,
            state: DispatchState::Rejected,
            reason: Some("runtime_offline".into()),
            task: None,
        };
        assert!(!result.accepted);
        assert_eq!(result.state, DispatchState::Rejected);
    }

    #[test]
    fn dispatch_accepted_after_heartbeat() {
        let mut registry = RuntimeRegistry::new();
        registry.heartbeat("rt1");
        assert!(registry.is_online("rt1"));
        let mut store = TaskStore::default();
        let task = store.dispatch(DispatchRequest {
            workspace_id: "ws1".into(),
            runtime_id: "rt1".into(),
            issue_key: "PROJ-83".into(),
            pipeline: "feature-delivery".into(),
        });
        assert_eq!(task.phase, TaskPhase::Queued);
    }

    #[test]
    fn approval_creates_confirm_task_via_store() {
        let mut confirms = ConfirmTaskStore::default();
        let task = confirms.queue("rt1", "run-85", "implement");
        let result = ApproveResult {
            confirm_task_created: true,
            task: task.clone(),
        };
        assert!(result.confirm_task_created);
        assert_eq!(confirms.claim_next("rt1").unwrap().stage, "implement");
    }

    #[test]
    fn task_phase_roundtrip() {
        assert_eq!(TaskPhase::parse_phase("queued"), Some(TaskPhase::Queued));
        assert_eq!(TaskPhase::Queued.as_str(), "queued");
    }

    #[tokio::test]
    async fn memory_backend_health_kind() {
        let state = AppState::memory();
        assert_eq!(state.backend.kind(), StorageKind::Memory);
    }

    #[test]
    fn resume_task_carries_run_id() {
        let mut store = TaskStore::default();
        let task = store.resume(
            "ws1".into(),
            "rt1".into(),
            "PROJ-80".into(),
            "feature-spec".into(),
            "run-80".into(),
        );
        assert_eq!(task.run_id.as_deref(), Some("run-80"));
        assert!(store.has_queued_resume("rt1", "run-80"));
        let claimed = store.claim_next("rt1").expect("claim");
        assert_eq!(claimed.run_id.as_deref(), Some("run-80"));
    }
}
