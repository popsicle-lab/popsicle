//! Runtime registration and online/offline detection (P1).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const DEFAULT_HEARTBEAT_TTL_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeState {
    Online,
    Offline,
}

#[derive(Debug, Clone)]
pub struct RuntimeRecord {
    pub last_heartbeat: Instant,
}

#[derive(Debug, Default)]
pub struct RuntimeRegistry {
    runtimes: HashMap<String, RuntimeRecord>,
    ttl: Duration,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            runtimes: HashMap::new(),
            ttl: Duration::from_secs(
                std::env::var("AGENT_RUNTIME_HEARTBEAT_TTL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_HEARTBEAT_TTL_SECS),
            ),
        }
    }

    pub fn heartbeat(&mut self, runtime_id: &str) -> RuntimeState {
        self.runtimes.insert(
            runtime_id.to_string(),
            RuntimeRecord {
                last_heartbeat: Instant::now(),
            },
        );
        RuntimeState::Online
    }

    pub fn state(&self, runtime_id: &str) -> RuntimeState {
        self.runtimes
            .get(runtime_id)
            .filter(|r| r.last_heartbeat.elapsed() <= self.ttl)
            .map(|_| RuntimeState::Online)
            .unwrap_or(RuntimeState::Offline)
    }

    pub fn is_online(&self, runtime_id: &str) -> bool {
        matches!(self.state(runtime_id), RuntimeState::Online)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_until_heartbeat() {
        let registry = RuntimeRegistry::new();
        assert!(!registry.is_online("dev1"));
        assert_eq!(registry.state("dev1"), RuntimeState::Offline);
    }

    #[test]
    fn online_after_heartbeat() {
        let mut registry = RuntimeRegistry::new();
        assert_eq!(registry.heartbeat("dev1"), RuntimeState::Online);
        assert!(registry.is_online("dev1"));
    }
}
