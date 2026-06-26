//! Pipeline session span events (ADR-001 P1 — skill-runtime inject port).

use std::sync::Arc;

use crate::domain::PipelineRunStatus;

/// Read-only session fields passed to span sinks (avoids circular refs with `PipelineSession`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSpanContext {
    pub run_id: String,
    pub pipeline_name: String,
    pub run_status: PipelineRunStatus,
    pub current_stage_index: i64,
}

/// Lifecycle point where a session span may be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionSpanEvent {
    /// `PipelineSession::start` — run entered first stage.
    PipelineStarted {
        stage_name: String,
        stage_index: i64,
        stage_skill: String,
    },
    /// `PipelineSession::complete_current` — a stage finished.
    StageCompleted {
        stage_name: String,
        stage_index: i64,
        stage_skill: String,
        run_status: PipelineRunStatus,
    },
}

/// Fail-open sink for session spans. Implementations must not panic or block the caller.
pub trait SessionSpanSink: Send + Sync {
    fn emit(&self, ctx: &SessionSpanContext, event: SessionSpanEvent);
}

/// Shared handle stored on [`crate::pipeline_session::PipelineSession`].
pub type SessionSpanSinkHandle = Arc<dyn SessionSpanSink>;

#[derive(Debug, Clone, Default)]
pub struct NoopSessionSpanSink;

impl SessionSpanSink for NoopSessionSpanSink {
    fn emit(&self, _ctx: &SessionSpanContext, _event: SessionSpanEvent) {}
}

/// In-process test double counting emissions.
#[derive(Debug, Default)]
pub struct RecordingSessionSpanSink {
    pub events: std::sync::Mutex<Vec<(SessionSpanContext, SessionSpanEvent)>>,
}

impl RecordingSessionSpanSink {
    pub fn into_handle(self) -> SessionSpanSinkHandle {
        Arc::new(self)
    }
}

impl SessionSpanSink for RecordingSessionSpanSink {
    fn emit(&self, ctx: &SessionSpanContext, event: SessionSpanEvent) {
        if let Ok(mut g) = self.events.lock() {
            g.push((ctx.clone(), event));
        }
    }
}
