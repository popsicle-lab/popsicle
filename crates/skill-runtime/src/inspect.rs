//! Read-only pipeline / stage views (T-0003 — inspect-state).

use crate::domain::{PipelineRunStatus, StageStatus};
use crate::pipeline_session::PipelineSession;

/// Read-only pipeline status for agents (`popsicle pipeline status` semantics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStatusSnapshot {
    pub run_id: String,
    pub pipeline_name: String,
    pub run_status: PipelineRunStatus,
    pub current_stage_index: i64,
    pub total_stages: i64,
    pub stages: Vec<StageSnapshot>,
}

/// One stage row in a status snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageSnapshot {
    pub name: String,
    pub status: StageStatus,
    pub requires_approval: bool,
    pub approved_at: i64,
    pub skill_names: Vec<String>,
}

impl PipelineStatusSnapshot {
    pub fn from_session(session: &PipelineSession) -> Self {
        let stages = session
            .pipeline
            .stages
            .iter()
            .zip(session.stages.iter())
            .map(|(def, st)| StageSnapshot {
                name: st.name.clone(),
                status: st.status,
                requires_approval: st.requires_approval,
                approved_at: st.approved_at,
                skill_names: def.skill_names().into_iter().map(str::to_string).collect(),
            })
            .collect();

        Self {
            run_id: session.run.id.clone(),
            pipeline_name: session.pipeline.name.clone(),
            run_status: session.run.status,
            current_stage_index: session.run.current_stage_index,
            total_stages: session.run.total_stages,
            stages,
        }
    }

    /// Name of the stage the run is currently positioned on.
    pub fn current_stage_name(&self) -> Option<&str> {
        self.stages
            .get(self.current_stage_index as usize)
            .map(|s| s.name.as_str())
    }
}
