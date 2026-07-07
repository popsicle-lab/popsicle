//! Per-run log ring buffer for mobile / WS subscribers (T-AR-0003).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

pub const MAX_LINES_PER_RUN: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunLogEntry {
    pub ts: u64,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunLogAppend {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct RunLogStore {
    lines: HashMap<String, VecDeque<RunLogEntry>>,
}

impl RunLogStore {
    pub fn append(&mut self, run_id: &str, level: &str, message: &str) -> RunLogEntry {
        let entry = RunLogEntry {
            ts: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            level: level.to_string(),
            message: message.to_string(),
        };
        let buf = self.lines.entry(run_id.to_string()).or_default();
        buf.push_back(entry.clone());
        while buf.len() > MAX_LINES_PER_RUN {
            buf.pop_front();
        }
        entry
    }

    pub fn list(&self, run_id: &str, limit: usize) -> Vec<RunLogEntry> {
        self.lines
            .get(run_id)
            .map(|buf| {
                let n = limit.min(buf.len());
                buf.iter()
                    .skip(buf.len().saturating_sub(n))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}
