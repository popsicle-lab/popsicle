use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::load_config;
use crate::otlp::{
    build_span_exporter, export_span_batch, parse_wal_json_lines, wal_lines_to_span_data,
};
use crate::wal::{telemetry_run_dir, wal_path};
use crate::{RecordOutcome, RecordResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportState {
    pub last_flush_ts: String,
    pub lines_exported: u64,
    pub last_error: Option<String>,
    pub endpoint: String,
    pub protocol: String,
}

pub fn status(workspace_root: &Path, run_id: Option<&str>) -> RecordResult {
    let Some(run_id) = run_id.filter(|s| !s.is_empty()) else {
        return RecordResult::degraded("run= required for status");
    };
    let state_path = telemetry_run_dir(workspace_root, run_id).join("export.state.json");
    if !state_path.is_file() {
        return RecordResult {
            outcome: RecordOutcome::Ok,
            message: Some("no export.state.json yet".into()),
            wal_path: Some(wal_path(workspace_root, run_id).display().to_string()),
            stdout: None,
            report_json: None,
        };
    }
    let content = fs::read_to_string(&state_path).map_err_display();
    match content {
        Ok(c) => RecordResult {
            outcome: RecordOutcome::Ok,
            message: Some(c),
            wal_path: None,
            stdout: None,
            report_json: None,
        },
        Err(e) => RecordResult::degraded(e),
    }
}

pub fn flush(workspace_root: &Path, run_id: Option<&str>) -> RecordResult {
    let Some(run_id) = run_id.filter(|s| !s.is_empty()) else {
        return RecordResult::degraded("run= required for flush");
    };
    flush_run_internal(workspace_root, run_id, true)
}

/// Export new WAL lines via OTLP HTTP/protobuf. `force` writes state even when no endpoint.
pub fn flush_run_internal(workspace_root: &Path, run_id: &str, force: bool) -> RecordResult {
    let config = load_config(workspace_root);
    let path = wal_path(workspace_root, run_id);
    if !path.is_file() {
        return write_state(
            workspace_root,
            run_id,
            ExportState {
                last_flush_ts: now_stub(),
                lines_exported: 0,
                last_error: Some("no wal file".into()),
                endpoint: config.exporter.endpoint.clone(),
                protocol: protocol_label(&config),
            },
            RecordResult::degraded("no spans.wal.jsonl"),
        );
    }

    let all_lines = match read_lines(&path) {
        Ok(l) => l,
        Err(e) => return RecordResult::degraded(e),
    };

    let state_path = telemetry_run_dir(workspace_root, run_id).join("export.state.json");
    let prev_exported = read_exported_count(&state_path);

    if config.exporter.endpoint.trim().is_empty() {
        return write_state(
            workspace_root,
            run_id,
            ExportState {
                last_flush_ts: now_stub(),
                lines_exported: all_lines.len() as u64,
                last_error: None,
                endpoint: String::new(),
                protocol: protocol_label(&config),
            },
            RecordResult {
                outcome: RecordOutcome::Ok,
                message: Some("wal ready; no exporter.endpoint configured".into()),
                wal_path: None,
                stdout: None,
                report_json: None,
            },
        );
    }

    if (all_lines.len() as u64) <= prev_exported && !force {
        return RecordResult {
            outcome: RecordOutcome::Ok,
            message: Some("no new wal lines to export".into()),
            wal_path: None,
            stdout: None,
            report_json: None,
        };
    }

    let wal_lines = parse_wal_json_lines(&all_lines);
    let batch = wal_lines_to_span_data(run_id, &wal_lines);
    let to_export: Vec<_> = batch.into_iter().skip(prev_exported as usize).collect();

    let mut exporter = match build_span_exporter(&config.exporter) {
        Ok(e) => e,
        Err(e) => {
            return write_state(
                workspace_root,
                run_id,
                ExportState {
                    last_flush_ts: now_stub(),
                    lines_exported: prev_exported,
                    last_error: Some(e.clone()),
                    endpoint: config.exporter.endpoint.clone(),
                    protocol: protocol_label(&config),
                },
                RecordResult::degraded(e),
            );
        }
    };

    match export_span_batch(&mut exporter, to_export) {
        Ok(()) => write_state(
            workspace_root,
            run_id,
            ExportState {
                last_flush_ts: now_stub(),
                lines_exported: all_lines.len() as u64,
                last_error: None,
                endpoint: config.exporter.endpoint.clone(),
                protocol: protocol_label(&config),
            },
            RecordResult {
                outcome: RecordOutcome::Ok,
                message: Some(format!(
                    "otlp export ok ({} lines, http/protobuf)",
                    all_lines.len()
                )),
                wal_path: None,
                stdout: None,
                report_json: None,
            },
        ),
        Err(e) => write_state(
            workspace_root,
            run_id,
            ExportState {
                last_flush_ts: now_stub(),
                lines_exported: prev_exported,
                last_error: Some(e.clone()),
                endpoint: config.exporter.endpoint.clone(),
                protocol: protocol_label(&config),
            },
            RecordResult::degraded(e),
        ),
    }
}

fn protocol_label(config: &crate::config::OtelConfig) -> String {
    let p = config.exporter.protocol.trim();
    if p.is_empty() {
        "http/protobuf".into()
    } else {
        p.to_string()
    }
}

fn read_exported_count(state_path: &Path) -> u64 {
    fs::read_to_string(state_path)
        .ok()
        .and_then(|c| serde_json::from_str::<ExportState>(&c).ok())
        .map(|s| s.lines_exported)
        .unwrap_or(0)
}

fn write_state(
    workspace_root: &Path,
    run_id: &str,
    state: ExportState,
    mut result: RecordResult,
) -> RecordResult {
    let dir = telemetry_run_dir(workspace_root, run_id);
    if fs::create_dir_all(&dir).is_err() {
        return RecordResult::degraded("cannot create telemetry dir");
    }
    let state_path = dir.join("export.state.json");
    match serde_json::to_string_pretty(&state) {
        Ok(json) => {
            if fs::write(&state_path, json).is_err() {
                result.outcome = RecordOutcome::Degraded;
                result.message = Some("failed to write export.state.json".into());
            }
        }
        Err(_) => {
            result.outcome = RecordOutcome::Degraded;
            result.message = Some("failed to serialize export state".into());
        }
    }
    result
}

fn read_lines(path: &Path) -> Result<Vec<String>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| l.map_err(|e| e.to_string()))
        .collect()
}

fn now_stub() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
}

trait MapErrDisplay<T> {
    fn map_err_display(self) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> MapErrDisplay<T> for Result<T, E> {
    fn map_err_display(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}
