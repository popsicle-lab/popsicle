//! Stage approval confirm task queue (P3 / T-AR-0004).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::TaskPhase;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfirmTask {
    pub id: Uuid,
    pub runtime_id: String,
    pub run_id: String,
    pub stage: String,
    pub phase: TaskPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveRequest {
    pub runtime_id: String,
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveResult {
    pub confirm_task_created: bool,
    pub task: ConfirmTask,
}

#[derive(Debug, Default)]
pub struct ConfirmTaskStore {
    tasks: HashMap<Uuid, ConfirmTask>,
}

impl ConfirmTaskStore {
    pub fn queue(&mut self, runtime_id: &str, run_id: &str, stage: &str) -> ConfirmTask {
        let task = ConfirmTask {
            id: Uuid::new_v4(),
            runtime_id: runtime_id.to_string(),
            run_id: run_id.to_string(),
            stage: stage.to_string(),
            phase: TaskPhase::Queued,
        };
        self.tasks.insert(task.id, task.clone());
        task
    }

    pub fn claim_next(&mut self, runtime_id: &str) -> Option<ConfirmTask> {
        let id = self
            .tasks
            .values()
            .find(|t| t.runtime_id == runtime_id && t.phase == TaskPhase::Queued)
            .map(|t| t.id)?;
        let task = self.tasks.get_mut(&id)?;
        task.phase = TaskPhase::Dispatched;
        Some(task.clone())
    }

    pub fn get(&self, id: Uuid) -> Option<ConfirmTask> {
        self.tasks.get(&id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_creates_confirm_task() {
        let mut store = ConfirmTaskStore::default();
        let task = store.queue("rt1", "run-1", "implement");
        assert_eq!(task.phase, TaskPhase::Queued);
        assert_eq!(task.stage, "implement");
        let claimed = store.claim_next("rt1").expect("claim");
        assert_eq!(claimed.id, task.id);
        assert_eq!(claimed.phase, TaskPhase::Dispatched);
    }
}
