//! Intake Chat sessions + turn/bootstrap queues (PDR-002 / PROJ-95).

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, ClaimRequest, DispatchState, TaskPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatSessionStatus {
    Active,
    Ready,
    Bootstrapped,
    Abandoned,
}

impl ChatSessionStatus {
    #[allow(dead_code)]
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Ready => "ready",
            Self::Bootstrapped => "bootstrapped",
            Self::Abandoned => "abandoned",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: Uuid,
    pub workspace_id: String,
    pub runtime_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    pub status: ChatSessionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_pipeline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_issue_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_run_id: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    pub ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionView {
    #[serde(flatten)]
    pub session: ChatSession,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatSessionRequest {
    pub workspace_id: String,
    pub runtime_id: String,
    #[serde(default)]
    pub product_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostChatMessageRequest {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurnResult {
    pub accepted: bool,
    pub state: DispatchState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResult {
    pub accepted: bool,
    pub state: DispatchState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurnTask {
    pub id: Uuid,
    pub session_id: Uuid,
    pub runtime_id: String,
    pub user_message_id: Uuid,
    pub user_content: String,
    pub phase: TaskPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapTask {
    pub id: Uuid,
    pub session_id: Uuid,
    pub runtime_id: String,
    pub workspace_id: String,
    pub product_id: String,
    pub draft_title: String,
    pub draft_pipeline: String,
    pub draft_description: String,
    pub phase: TaskPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteChatTurnRequest {
    pub runtime_id: String,
    pub turn_id: Uuid,
    pub assistant_content: String,
    #[serde(default)]
    pub draft_title: Option<String>,
    #[serde(default)]
    pub draft_pipeline: Option<String>,
    #[serde(default)]
    pub draft_description: Option<String>,
    #[serde(default)]
    pub mark_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteBootstrapRequest {
    pub runtime_id: String,
    pub task_id: Uuid,
    pub issue_key: String,
    pub run_id: String,
}

#[derive(Debug, Default)]
pub struct ChatStore {
    sessions: HashMap<Uuid, ChatSession>,
    messages: Vec<ChatMessage>,
    turns: HashMap<Uuid, ChatTurnTask>,
    bootstraps: HashMap<Uuid, BootstrapTask>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl ChatStore {
    pub fn create_session(&mut self, req: CreateChatSessionRequest) -> ChatSession {
        let id = Uuid::new_v4();
        let session = ChatSession {
            id,
            workspace_id: req.workspace_id,
            runtime_id: req.runtime_id,
            product_id: req.product_id,
            status: ChatSessionStatus::Active,
            draft_title: None,
            draft_pipeline: None,
            draft_description: None,
            linked_issue_key: None,
            linked_run_id: None,
            updated_at: now_secs(),
        };
        self.sessions.insert(id, session.clone());
        session
    }

    pub fn get_session_view(&self, id: Uuid) -> Option<ChatSessionView> {
        let session = self.sessions.get(&id)?.clone();
        let messages: Vec<_> = self
            .messages
            .iter()
            .filter(|m| m.session_id == id)
            .cloned()
            .collect();
        Some(ChatSessionView { session, messages })
    }

    pub fn post_user_message(
        &mut self,
        session_id: Uuid,
        content: &str,
    ) -> Option<(ChatMessage, ChatTurnTask)> {
        let session = self.sessions.get_mut(&session_id)?;
        if session.status == ChatSessionStatus::Bootstrapped {
            return None;
        }
        let msg = ChatMessage {
            id: Uuid::new_v4(),
            session_id,
            role: "user".into(),
            content: content.to_string(),
            ts: now_secs(),
        };
        self.messages.push(msg.clone());
        session.updated_at = now_secs();
        let turn = ChatTurnTask {
            id: Uuid::new_v4(),
            session_id,
            runtime_id: session.runtime_id.clone(),
            user_message_id: msg.id,
            user_content: content.to_string(),
            phase: TaskPhase::Queued,
        };
        self.turns.insert(turn.id, turn.clone());
        Some((msg, turn))
    }

    pub fn claim_chat_turn(&mut self, runtime_id: &str) -> Option<ChatTurnTask> {
        let id = self
            .turns
            .values()
            .find(|t| t.runtime_id == runtime_id && t.phase == TaskPhase::Queued)
            .map(|t| t.id)?;
        let turn = self.turns.get_mut(&id)?;
        turn.phase = TaskPhase::Dispatched;
        Some(turn.clone())
    }

    pub fn complete_chat_turn(&mut self, req: CompleteChatTurnRequest) -> Option<ChatSessionView> {
        let session_id = {
            let turn = self.turns.get_mut(&req.turn_id)?;
            if turn.runtime_id != req.runtime_id {
                return None;
            }
            turn.phase = TaskPhase::Completed;
            turn.session_id
        };

        let assistant = ChatMessage {
            id: Uuid::new_v4(),
            session_id,
            role: "assistant".into(),
            content: req.assistant_content,
            ts: now_secs(),
        };
        self.messages.push(assistant);

        let session = self.sessions.get_mut(&session_id)?;
        if let Some(t) = req.draft_title.filter(|s| !s.is_empty()) {
            session.draft_title = Some(t);
        }
        if let Some(p) = req.draft_pipeline.filter(|s| !s.is_empty()) {
            session.draft_pipeline = Some(p);
        }
        if let Some(d) = req.draft_description.filter(|s| !s.is_empty()) {
            session.draft_description = Some(d);
        }
        if req.mark_ready {
            session.status = ChatSessionStatus::Ready;
        }
        session.updated_at = now_secs();
        self.get_session_view(session_id)
    }

    pub fn queue_bootstrap(&mut self, session_id: Uuid) -> Option<BootstrapTask> {
        let session = self.sessions.get(&session_id)?;
        if session.status != ChatSessionStatus::Ready {
            return None;
        }
        let title = session.draft_title.clone()?;
        let pipeline = session.draft_pipeline.clone()?;
        let description = session.draft_description.clone().unwrap_or_default();
        let product_id = session
            .product_id
            .clone()
            .unwrap_or_else(|| "agent-runtime".into());
        let task = BootstrapTask {
            id: Uuid::new_v4(),
            session_id,
            runtime_id: session.runtime_id.clone(),
            workspace_id: session.workspace_id.clone(),
            product_id,
            draft_title: title,
            draft_pipeline: pipeline,
            draft_description: description,
            phase: TaskPhase::Queued,
        };
        self.bootstraps.insert(task.id, task.clone());
        Some(task)
    }

    pub fn claim_bootstrap(&mut self, runtime_id: &str) -> Option<BootstrapTask> {
        let id = self
            .bootstraps
            .values()
            .find(|t| t.runtime_id == runtime_id && t.phase == TaskPhase::Queued)
            .map(|t| t.id)?;
        let task = self.bootstraps.get_mut(&id)?;
        task.phase = TaskPhase::Dispatched;
        Some(task.clone())
    }

    pub fn complete_bootstrap(&mut self, req: CompleteBootstrapRequest) -> Option<ChatSessionView> {
        let session_id = {
            let task = self.bootstraps.get_mut(&req.task_id)?;
            if task.runtime_id != req.runtime_id {
                return None;
            }
            task.phase = TaskPhase::Completed;
            task.session_id
        };

        let session = self.sessions.get_mut(&session_id)?;
        session.status = ChatSessionStatus::Bootstrapped;
        session.linked_issue_key = Some(req.issue_key);
        session.linked_run_id = Some(req.run_id);
        session.updated_at = now_secs();
        self.get_session_view(session_id)
    }
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateChatSessionRequest>,
) -> Result<Json<ChatSession>, StatusCode> {
    if req.workspace_id.is_empty() || req.runtime_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let session = state
        .backend
        .create_chat_session(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(session))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ChatSessionView>, StatusCode> {
    state
        .backend
        .get_chat_session(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json)
}

pub async fn post_message(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<PostChatMessageRequest>,
) -> Result<Json<ChatTurnResult>, StatusCode> {
    if req.content.trim().is_empty() || req.role != "user" {
        return Err(StatusCode::BAD_REQUEST);
    }
    let view = state
        .backend
        .get_chat_session(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let online = state
        .backend
        .is_online(&view.session.runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !online {
        return Ok(Json(ChatTurnResult {
            accepted: false,
            state: DispatchState::Rejected,
            reason: Some("runtime_offline".into()),
            message: None,
        }));
    }
    let (message, turn) = state
        .backend
        .post_chat_user_message(session_id, &req.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::CONFLICT)?;
    state.publish_event(serde_json::json!({
        "type": "chat_turn_queued",
        "session_id": session_id,
        "turn_id": turn.id,
        "runtime_id": view.session.runtime_id,
    }));
    Ok(Json(ChatTurnResult {
        accepted: true,
        state: DispatchState::Queued,
        reason: None,
        message: Some(message),
    }))
}

pub async fn bootstrap_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<BootstrapResult>, StatusCode> {
    let view = state
        .backend
        .get_chat_session(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let online = state
        .backend
        .is_online(&view.session.runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !online {
        return Ok(Json(BootstrapResult {
            accepted: false,
            state: DispatchState::Rejected,
            reason: Some("runtime_offline".into()),
        }));
    }
    let task = state
        .backend
        .queue_bootstrap(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::CONFLICT)?;
    state.publish_event(serde_json::json!({
        "type": "bootstrap_queued",
        "session_id": session_id,
        "task_id": task.id,
        "runtime_id": view.session.runtime_id,
    }));
    Ok(Json(BootstrapResult {
        accepted: true,
        state: DispatchState::Queued,
        reason: None,
    }))
}

pub async fn claim_chat_turn(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
    Json(_body): Json<ClaimRequest>,
) -> Result<Json<ChatTurnTask>, StatusCode> {
    let task = state
        .backend
        .claim_chat_turn(&runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(task))
}

pub async fn claim_bootstrap(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
    Json(_body): Json<ClaimRequest>,
) -> Result<Json<BootstrapTask>, StatusCode> {
    let task = state
        .backend
        .claim_bootstrap(&runtime_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(task))
}

pub async fn complete_chat_turn(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<CompleteChatTurnRequest>,
) -> Result<Json<ChatSessionView>, StatusCode> {
    let draft_updated = req.draft_title.is_some()
        || req.draft_pipeline.is_some()
        || req.draft_description.is_some();
    let view = state
        .backend
        .complete_chat_turn(session_id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    state.publish_event(serde_json::json!({
        "type": "chat_message",
        "session_id": session_id,
        "session": view.session,
    }));
    if draft_updated {
        state.publish_event(serde_json::json!({
            "type": "chat_draft_updated",
            "session_id": session_id,
            "session": view.session,
        }));
    }
    Ok(Json(view))
}

pub async fn complete_bootstrap(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<CompleteBootstrapRequest>,
) -> Result<Json<ChatSessionView>, StatusCode> {
    let view = state
        .backend
        .complete_bootstrap(session_id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    state.publish_event(serde_json::json!({
        "type": "session_bootstrapped",
        "session_id": session_id,
        "issue_key": view.session.linked_issue_key,
        "run_id": view.session.linked_run_id,
    }));
    Ok(Json(view))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_turn_flow_memory_store() {
        let mut store = ChatStore::default();
        let session = store.create_session(CreateChatSessionRequest {
            workspace_id: "ws".into(),
            runtime_id: "rt".into(),
            product_id: Some("agent-runtime".into()),
        });
        let (_msg, turn) = store
            .post_user_message(session.id, "修复 mobile unknown")
            .expect("message");
        let claimed = store.claim_chat_turn("rt").expect("claim");
        assert_eq!(claimed.id, turn.id);
        let view = store
            .complete_chat_turn(CompleteChatTurnRequest {
                runtime_id: "rt".into(),
                turn_id: turn.id,
                assistant_content: "好的，我来澄清需求。".into(),
                draft_title: Some("Mobile 修复 UNKNOWN".into()),
                draft_pipeline: Some("fix-regression".into()),
                draft_description: Some("修复 mobile UI issue_key 显示".into()),
                mark_ready: true,
            })
            .expect("complete");
        assert_eq!(view.session.status, ChatSessionStatus::Ready);
        assert_eq!(view.messages.len(), 2);
    }
}
