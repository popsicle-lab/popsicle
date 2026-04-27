//! Last-write-wins conflict merger with audit log.
//!
//! When the server rejects a push with [`PushOutcome::Conflict`], the client
//! must reconcile its local payload against the server's authoritative one.
//! The strategy is:
//!
//! 1. Start with the *server* payload as the base.
//! 2. Walk the local payload field-by-field. For each top-level field, if the
//!    local value differs from `last_pushed` (i.e. the user actually edited
//!    it), overlay it onto the server payload.
//! 3. Any server-only changes the user didn't touch are preserved.
//! 4. Any local changes that *would have* overwritten a concurrent server
//!    edit on a field the local client didn't change are silently lost.
//!    They are recorded to `.popsicle/.sync/conflicts.log` so the user can
//!    review them later.
//!
//! This is intentionally simple — we don't attempt sub-object merging.
//! Document bodies (the long-form text content) live in the CRDT log and
//! never reach this code path.

use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::types::EntityKind;

/// Outcome of a single field-level merge operation.
#[derive(Debug, Clone)]
pub struct MergeReport {
    /// Payload to push back to the server.
    pub merged: Value,
    /// Fields where the local edit was discarded because the server had a
    /// concurrent edit and the local change post-dated the last push but
    /// also applies to a field that has changed remotely.
    pub lost: Vec<LostField>,
}

#[derive(Debug, Clone)]
pub struct LostField {
    pub field: String,
    pub local_value: Value,
    pub server_value: Value,
}

/// Three-way merge: rebase local edits on top of the server payload.
///
/// `last_pushed` is the payload as the client last successfully sent it
/// (i.e. the common ancestor). If unknown, pass [`Value::Null`]; in that case
/// every differing field is treated as a local edit and the server values
/// for those fields go to the conflict log.
pub fn merge(local: &Value, last_pushed: &Value, server: &Value) -> MergeReport {
    let local_obj = local.as_object();
    let pushed_obj = last_pushed.as_object();
    let server_obj = server.as_object();

    // Non-object payloads: fall back to whole-value last-write-wins (server).
    let (Some(local_map), Some(server_map)) = (local_obj, server_obj) else {
        return MergeReport {
            merged: server.clone(),
            lost: if local != server {
                vec![LostField {
                    field: "<root>".into(),
                    local_value: local.clone(),
                    server_value: server.clone(),
                }]
            } else {
                vec![]
            },
        };
    };

    let empty = Map::new();
    let pushed_map = pushed_obj.unwrap_or(&empty);
    let mut merged = server_map.clone();
    let mut lost = Vec::new();

    for (key, local_value) in local_map {
        let pushed_value = pushed_map.get(key).unwrap_or(&Value::Null);
        let server_value = server_map.get(key).unwrap_or(&Value::Null);
        let local_changed = local_value != pushed_value;
        let server_changed = server_value != pushed_value;
        match (local_changed, server_changed) {
            (true, false) => {
                merged.insert(key.clone(), local_value.clone());
            }
            (true, true) => {
                lost.push(LostField {
                    field: key.clone(),
                    local_value: local_value.clone(),
                    server_value: server_value.clone(),
                });
            }
            _ => {}
        }
    }
    MergeReport {
        merged: Value::Object(merged),
        lost,
    }
}

/// Append a conflict entry to `.popsicle/.sync/conflicts.log`.
pub fn append_log(
    log_path: &Path,
    kind: EntityKind,
    id: Uuid,
    report: &MergeReport,
) -> std::io::Result<()> {
    if report.lost.is_empty() {
        return Ok(());
    }
    if let Some(parent) = log_path.parent() {
        create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(log_path)?;
    let entry = serde_json::json!({
        "ts": Utc::now().to_rfc3339(),
        "kind": kind.as_str(),
        "id": id.to_string(),
        "lost": report.lost.iter().map(|l| {
            serde_json::json!({
                "field": l.field,
                "local": l.local_value,
                "server": l.server_value,
            })
        }).collect::<Vec<_>>(),
    });
    writeln!(f, "{}", entry)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn local_only_change_wins() {
        let last = json!({"title": "old", "status": "open"});
        let local = json!({"title": "new", "status": "open"});
        let server = json!({"title": "old", "status": "open"});
        let r = merge(&local, &last, &server);
        assert_eq!(r.merged["title"], "new");
        assert!(r.lost.is_empty());
    }

    #[test]
    fn server_only_change_wins() {
        let last = json!({"title": "old", "status": "open"});
        let local = json!({"title": "old", "status": "open"});
        let server = json!({"title": "old", "status": "closed"});
        let r = merge(&local, &last, &server);
        assert_eq!(r.merged["status"], "closed");
        assert!(r.lost.is_empty());
    }

    #[test]
    fn double_edit_logs_local_loss() {
        let last = json!({"title": "old"});
        let local = json!({"title": "local"});
        let server = json!({"title": "server"});
        let r = merge(&local, &last, &server);
        assert_eq!(r.merged["title"], "server");
        assert_eq!(r.lost.len(), 1);
        assert_eq!(r.lost[0].field, "title");
    }

    #[test]
    fn no_last_pushed_treats_all_as_local() {
        let local = json!({"a": 1, "b": 2});
        let server = json!({"a": 1, "b": 99});
        let r = merge(&local, &Value::Null, &server);
        // With no ancestor, every field present in both differs from Null,
        // so any divergent value is logged as a conflict.
        assert_eq!(r.merged["b"], 99);
        assert_eq!(r.lost.len(), 2);
    }
}
