mod background;
mod config;
mod export;
mod otlp;
mod report;
mod wal;

use std::collections::BTreeMap;
use std::path::Path;

pub use config::load_config;
pub use report::{health_summary_line, report_recent, report_run, RecentReport, RunReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordOutcome {
    Ok,
    Degraded,
}

#[derive(Debug, Clone)]
pub struct RecordResult {
    pub outcome: RecordOutcome,
    pub message: Option<String>,
    pub wal_path: Option<String>,
    /// Plain-text stdout (e.g. `action=guide`).
    pub stdout: Option<String>,
    /// JSON payload for `action=report` (REPORT_SCHEMA).
    pub report_json: Option<String>,
}

impl RecordResult {
    pub fn ok() -> Self {
        Self {
            outcome: RecordOutcome::Ok,
            message: None,
            wal_path: None,
            stdout: None,
            report_json: None,
        }
    }

    pub fn degraded(msg: impl Into<String>) -> Self {
        Self {
            outcome: RecordOutcome::Degraded,
            message: Some(msg.into()),
            wal_path: None,
            stdout: None,
            report_json: None,
        }
    }

    pub fn with_stdout(text: String) -> Self {
        Self {
            outcome: RecordOutcome::Ok,
            message: None,
            wal_path: None,
            stdout: Some(text),
            report_json: None,
        }
    }

    pub fn with_report(json: String, outcome: RecordOutcome) -> Self {
        Self {
            outcome,
            message: None,
            wal_path: None,
            stdout: None,
            report_json: Some(json),
        }
    }
}

/// Append one span to the run WAL. Never panics; IO errors become `Degraded`.
pub fn record_span(
    workspace_root: &Path,
    run_id: &str,
    span_name: &str,
    attributes: &BTreeMap<String, String>,
) -> RecordResult {
    if run_id.trim().is_empty() {
        return RecordResult::degraded("run_id required");
    }
    match wal::append_span(workspace_root, run_id, span_name, attributes) {
        Ok(rel) => {
            background::maybe_start_background_flush(workspace_root);
            RecordResult {
                outcome: RecordOutcome::Ok,
                message: None,
                wal_path: Some(rel),
                stdout: None,
                report_json: None,
            }
        }
        Err(e) => RecordResult::degraded(e),
    }
}

pub fn flush(workspace_root: &Path, run_id: Option<&str>) -> RecordResult {
    export::flush(workspace_root, run_id)
}

pub fn status(workspace_root: &Path, run_id: Option<&str>) -> RecordResult {
    export::status(workspace_root, run_id)
}

fn resolve_guide_path(workspace_root: &Path) -> Option<std::path::PathBuf> {
    [
        workspace_root.join("intent-coder/tools/telemetry/guide.md"),
        workspace_root.join(".popsicle/modules/intent-coder/tools/telemetry/guide.md"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

/// Print bundled Agent guide (`intent-coder/tools/telemetry/guide.md`).
pub fn guide(workspace_root: &Path) -> RecordResult {
    match resolve_guide_path(workspace_root) {
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(text) => RecordResult::with_stdout(text),
            Err(e) => RecordResult::degraded(e.to_string()),
        },
        None => RecordResult::degraded(
            "telemetry guide not found; run popsicle init or admin sync-intent-coder",
        ),
    }
}

/// `popsicle tool run telemetry action=guide|record|flush|status|report ...`
pub fn run_tool(args: &BTreeMap<String, String>, workspace_root: &Path) -> (RecordResult, i32) {
    let action = args.get("action").map(String::as_str).unwrap_or("record");
    let run_id = args.get("run").map(String::as_str);
    let result = match action {
        "guide" => guide(workspace_root),
        "flush" => flush(workspace_root, run_id),
        "status" => status(workspace_root, run_id),
        "report" => run_report_action(args, workspace_root, run_id),
        "record" => {
            let span = args
                .get("span")
                .map(String::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("gen_ai.chat");
            let run = match run_id.filter(|s| !s.is_empty()) {
                Some(r) => r,
                None => return (RecordResult::degraded("run= required for record"), 0),
            };
            let mut attrs = BTreeMap::new();
            for (k, v) in args {
                if matches!(k.as_str(), "action" | "span" | "run" | "format" | "tool") {
                    continue;
                }
                attrs.insert(k.clone(), v.clone());
            }
            if let Some(doc) = args.get("doc") {
                attrs
                    .entry("popsicle.doc_id".into())
                    .or_insert_with(|| doc.clone());
            }
            record_span(workspace_root, run, span, &attrs)
        }
        other => RecordResult::degraded(format!("unknown action: {other}")),
    };
    (result, 0)
}

fn run_report_action(
    args: &BTreeMap<String, String>,
    workspace_root: &Path,
    run_id: Option<&str>,
) -> RecordResult {
    if let Some(run) = run_id.filter(|s| !s.is_empty()) {
        let report = report_run(workspace_root, run);
        let outcome = if report.status == "ok" {
            RecordOutcome::Ok
        } else {
            RecordOutcome::Degraded
        };
        return match serde_json::to_string(&report) {
            Ok(json) => RecordResult::with_report(json, outcome),
            Err(e) => RecordResult::degraded(e.to_string()),
        };
    }
    let limit = args
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    let recent = report_recent(workspace_root, limit);
    let outcome = if recent.status == "ok" {
        RecordOutcome::Ok
    } else {
        RecordOutcome::Degraded
    };
    match serde_json::to_string(&recent) {
        Ok(json) => RecordResult::with_report(json, outcome),
        Err(e) => RecordResult::degraded(e.to_string()),
    }
}

pub fn result_to_json_fields(result: &RecordResult) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    fields.insert(
        "telemetry_outcome".into(),
        match result.outcome {
            RecordOutcome::Ok => "ok".into(),
            RecordOutcome::Degraded => "degraded".into(),
        },
    );
    if let Some(msg) = &result.message {
        fields.insert("message".into(), msg.clone());
    }
    if let Some(path) = &result.wal_path {
        fields.insert("wal_path".into(), path.clone());
    }
    if let Some(report) = &result.report_json {
        fields.insert("report".into(), report.clone());
    }
    if let Some(text) = &result.stdout {
        fields.insert("guide".into(), text.clone());
    }
    fields
}

#[cfg(test)]
mod integration {
    use std::collections::BTreeMap;

    use super::{flush, record_span, RecordOutcome};

    #[test]
    fn record_and_flush_fail_open_without_endpoint() {
        let tmp = tempfile_dir();
        let run_id = "test-run-001";
        let mut attrs = BTreeMap::new();
        attrs.insert("gen_ai.request.model".into(), "test-model".into());
        let r = record_span(&tmp, run_id, "gen_ai.chat", &attrs);
        assert_eq!(r.outcome, RecordOutcome::Ok);
        assert!(r.wal_path.is_some());
        let f = flush(&tmp, Some(run_id));
        assert_eq!(f.outcome, RecordOutcome::Ok);
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("popsicle-telemetry-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmpdir");
        dir
    }
}
