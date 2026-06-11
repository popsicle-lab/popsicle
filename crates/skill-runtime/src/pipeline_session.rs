//! In-memory pipeline run orchestration (T-0001 / T-0002 / T-0004).

use crate::domain::{PipelineRun, PipelineRunStatus, Stage, StageStatus};
use crate::inspect::PipelineStatusSnapshot;
use crate::loader::PipelineDef;
use crate::runs::{
    bootstrap_to_first_pause, recover_blocked_pipeline, BootstrapError, RecoverError,
};
use crate::state_machine::{advance_stage_with_approval, StageAdvanceError};

/// Errors mutating a [`PipelineSession`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    Bootstrap(BootstrapError),
    Advance(StageAdvanceError),
    Recover(RecoverError),
    NoCurrentStage,
    RunAlreadyStarted,
    RunNotActive,
}

impl From<BootstrapError> for SessionError {
    fn from(e: BootstrapError) -> Self {
        Self::Bootstrap(e)
    }
}

impl From<StageAdvanceError> for SessionError {
    fn from(e: StageAdvanceError) -> Self {
        Self::Advance(e)
    }
}

impl From<RecoverError> for SessionError {
    fn from(e: RecoverError) -> Self {
        Self::Recover(e)
    }
}

/// A pipeline run with per-stage state aligned to a [`PipelineDef`].
#[derive(Debug, Clone)]
pub struct PipelineSession {
    pub pipeline: PipelineDef,
    pub run: PipelineRun,
    pub stages: Vec<Stage>,
}

impl PipelineSession {
    /// Fresh pending run; first stage `blocked`, rest `ready`.
    pub fn new_pending(run_id: impl Into<String>, pipeline: PipelineDef) -> Self {
        let total = pipeline.stages.len() as i64;
        let stages = pipeline
            .stages
            .iter()
            .enumerate()
            .map(|(i, def)| Stage {
                name: def.name.clone(),
                status: if i == 0 {
                    StageStatus::StageBlocked
                } else {
                    StageStatus::StageReady
                },
                requires_approval: def.requires_approval,
                approved_at: 0,
            })
            .collect();

        Self {
            pipeline,
            run: PipelineRun {
                id: run_id.into(),
                status: PipelineRunStatus::RunPending,
                current_stage_index: 0,
                total_stages: total,
            },
            stages,
        }
    }

    fn current_stage_mut(&mut self) -> Option<&mut Stage> {
        let idx = self.run.current_stage_index as usize;
        self.stages.get_mut(idx)
    }

    /// `PipelineBootstrapsToFirstPause` (acceptance.intent, T-0001).
    pub fn start(&mut self) -> Result<(), SessionError> {
        if self.run.status != PipelineRunStatus::RunPending {
            return Err(SessionError::RunAlreadyStarted);
        }
        let idx = self.run.current_stage_index as usize;
        let stage = self
            .stages
            .get_mut(idx)
            .ok_or(SessionError::NoCurrentStage)?;
        let (run2, stage2) = bootstrap_to_first_pause(&self.run, stage)?;
        self.run = run2;
        *stage = stage2;
        Ok(())
    }

    /// Record human approval for the current stage (`approved_at > 0`).
    pub fn approve_current(&mut self, approved_at: i64) -> Result<(), SessionError> {
        if self.run.status != PipelineRunStatus::RunInProgress {
            return Err(SessionError::RunNotActive);
        }
        let stage = self
            .current_stage_mut()
            .ok_or(SessionError::NoCurrentStage)?;
        stage.approved_at = approved_at;
        Ok(())
    }

    /// `StageAdvanceWithApproval` + advance pipeline index when more stages remain.
    pub fn complete_current(&mut self) -> Result<(), SessionError> {
        if self.run.status != PipelineRunStatus::RunInProgress {
            return Err(SessionError::RunNotActive);
        }
        let idx = self.run.current_stage_index as usize;
        let stage = self
            .stages
            .get_mut(idx)
            .ok_or(SessionError::NoCurrentStage)?;
        let completed = advance_stage_with_approval(stage)?;
        *stage = completed;

        if (idx + 1) < self.stages.len() {
            self.run.current_stage_index += 1;
            if let Some(next) = self.stages.get_mut(idx + 1) {
                next.status = StageStatus::StageInProgress;
            }
        } else {
            self.run.status = PipelineRunStatus::RunCompleted;
        }
        Ok(())
    }

    /// Mark current stage errored and block the run (simulates a failed stage).
    pub fn fail_current(&mut self) -> Result<(), SessionError> {
        if self.run.status != PipelineRunStatus::RunInProgress {
            return Err(SessionError::RunNotActive);
        }
        let stage = self
            .current_stage_mut()
            .ok_or(SessionError::NoCurrentStage)?;
        stage.status = StageStatus::StageError;
        self.run.status = PipelineRunStatus::RunBlocked;
        Ok(())
    }

    /// `RecoveredPipelineCanAdvance` (acceptance.intent, T-0004).
    pub fn recover_current(&mut self) -> Result<(), SessionError> {
        let idx = self.run.current_stage_index as usize;
        let stage = self
            .stages
            .get_mut(idx)
            .ok_or(SessionError::NoCurrentStage)?;
        let (run2, stage2) = recover_blocked_pipeline(&self.run, stage)?;
        self.run = run2;
        *stage = stage2;
        Ok(())
    }

    /// Read-only status view (T-0003 inspect-state).
    pub fn snapshot(&self) -> PipelineStatusSnapshot {
        PipelineStatusSnapshot::from_session(self)
    }
}
