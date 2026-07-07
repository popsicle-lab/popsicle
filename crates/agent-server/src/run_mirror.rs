//! Pipeline run mirror (P2 / T-AR-0003).

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageMirror {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunMirror {
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_key: Option<String>,
    pub pipeline: String,
    pub run_status: String,
    pub current_stage: String,
    pub stages: Vec<StageMirror>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunMirrorUpsert {
    pub issue_key: Option<String>,
    #[serde(flatten)]
    pub status: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Default)]
pub struct RunMirrorStore {
    runs: HashMap<String, RunMirror>,
}

impl RunMirrorStore {
    pub fn upsert_from_status(&mut self, run_id: &str, upsert: RunMirrorUpsert) -> RunMirror {
        let mirror = build_mirror(run_id, upsert);
        self.runs.insert(run_id.to_string(), mirror.clone());
        mirror
    }

    pub fn get(&self, run_id: &str) -> Option<RunMirror> {
        self.runs.get(run_id).cloned()
    }

    pub fn list(&self) -> Vec<RunMirror> {
        let mut runs: Vec<_> = self.runs.values().cloned().collect();
        runs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        runs
    }
}

pub fn build_mirror(run_id: &str, upsert: RunMirrorUpsert) -> RunMirror {
    let pipeline = upsert
        .status
        .get("pipeline")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let run_status = upsert
        .status
        .get("run_status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let current_stage = upsert
        .status
        .get("current_stage")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let total_stages = upsert
        .status
        .get("total_stages")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let mut stages = Vec::new();
    for idx in 0..total_stages {
        let name_key = format!("stage_{idx}_name");
        let status_key = format!("stage_{idx}_status");
        if let (Some(name), Some(status)) = (
            upsert.status.get(&name_key).and_then(|v| v.as_str()),
            upsert.status.get(&status_key).and_then(|v| v.as_str()),
        ) {
            stages.push(StageMirror {
                name: name.to_string(),
                status: status.to_string(),
            });
        }
    }
    let updated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    RunMirror {
        run_id: run_id.to_string(),
        issue_key: upsert.issue_key,
        pipeline,
        run_status,
        current_stage,
        stages,
        updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_from_pipeline_status_fields() {
        let mut status = serde_json::Map::new();
        status.insert("pipeline".into(), "feature-delivery".into());
        status.insert("run_status".into(), "in_progress".into());
        status.insert("current_stage".into(), "implement".into());
        status.insert("total_stages".into(), "3".into());
        status.insert("stage_0_name".into(), "implement".into());
        status.insert("stage_0_status".into(), "in_progress".into());
        status.insert("stage_1_name".into(), "verify".into());
        status.insert("stage_1_status".into(), "pending".into());
        let mirror = build_mirror(
            "run-1",
            RunMirrorUpsert {
                issue_key: Some("PROJ-84".into()),
                status,
            },
        );
        assert_eq!(mirror.stages.len(), 2);
        assert_eq!(mirror.current_stage, "implement");
        assert_eq!(mirror.issue_key.as_deref(), Some("PROJ-84"));
    }

    #[test]
    fn store_lists_mirrors() {
        let mut store = RunMirrorStore::default();
        let mut status = serde_json::Map::new();
        status.insert("pipeline".into(), "feature-delivery".into());
        status.insert("run_status".into(), "completed".into());
        status.insert("current_stage".into(), "doc-sync".into());
        status.insert("total_stages".into(), "1".into());
        status.insert("stage_0_name".into(), "doc-sync".into());
        status.insert("stage_0_status".into(), "completed".into());
        store.upsert_from_status(
            "run-1",
            RunMirrorUpsert {
                issue_key: None,
                status,
            },
        );
        assert_eq!(store.list().len(), 1);
    }
}
