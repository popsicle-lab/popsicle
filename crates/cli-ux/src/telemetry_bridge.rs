//! Best-effort orchestration telemetry inject (ADR-001 fail-open).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use skill_runtime::{
    PipelineRunStatus, SessionSpanContext, SessionSpanEvent, SessionSpanSink, SessionSpanSinkHandle,
};
use storage::{DocCheckRow, RunRow};

pub fn orchestration_span(
    workspace_root: &Path,
    run_id: &str,
    run_row: Option<&RunRow>,
    span_name: &str,
    extra: &[(&str, &str)],
) {
    let mut attrs = base_attrs(run_id, run_row);
    for (k, v) in extra {
        attrs.insert(k.to_string(), (*v).to_string());
    }
    let _ = telemetry::record_span(workspace_root, run_id, span_name, &attrs);
}

/// After `doc check`, record pass/fail and checklist stats (Phase B).
pub fn doc_check_span(workspace_root: &Path, run_row: Option<&RunRow>, check: &DocCheckRow) {
    let Some(run_id) = run_id_from_artifact_path(&check.file_path) else {
        return;
    };
    let mut attrs = base_attrs(&run_id, run_row);
    attrs.insert("popsicle.doc_id".into(), check.doc_id.clone());
    attrs.insert("popsicle.span.kind".into(), "popsicle.doc".into());
    attrs.insert("popsicle.doc_check.passed".into(), check.passed.to_string());
    attrs.insert(
        "popsicle.doc_check.placeholder_count".into(),
        check.placeholder_count.to_string(),
    );
    attrs.insert(
        "popsicle.doc_check.checkboxes_checked".into(),
        check.checkboxes_checked.to_string(),
    );
    attrs.insert(
        "popsicle.doc_check.checkboxes_total".into(),
        check.checkboxes_total.to_string(),
    );
    attrs.insert(
        "popsicle.doc_check.body_filled".into(),
        check.body_filled.to_string(),
    );
    if let Some(skill) = skill_from_doc_path(&check.file_path) {
        attrs.insert("popsicle.skill".into(), skill);
    }
    let _ = telemetry::record_span(workspace_root, &run_id, "popsicle.doc.check", &attrs);
}

/// Sink wired into [`skill_runtime::PipelineSession`] (session.start / complete_current).
pub fn session_span_sink(workspace_root: &Path, issue_key: &str) -> SessionSpanSinkHandle {
    Arc::new(TelemetrySessionSpanSink {
        workspace_root: workspace_root.to_path_buf(),
        issue_key: issue_key.to_string(),
    })
}

struct TelemetrySessionSpanSink {
    workspace_root: PathBuf,
    issue_key: String,
}

impl SessionSpanSink for TelemetrySessionSpanSink {
    fn emit(&self, ctx: &SessionSpanContext, event: SessionSpanEvent) {
        let (span_name, stage_name, stage_skill, span_kind) = match &event {
            SessionSpanEvent::PipelineStarted {
                stage_name,
                stage_skill,
                ..
            } => (
                "popsicle.run.start",
                stage_name.as_str(),
                stage_skill.as_str(),
                "popsicle.run",
            ),
            SessionSpanEvent::StageCompleted {
                stage_name,
                stage_skill,
                ..
            } => (
                "popsicle.stage.complete",
                stage_name.as_str(),
                stage_skill.as_str(),
                "popsicle.stage",
            ),
        };
        let mut attrs = session_attrs(ctx, &self.issue_key, stage_name, stage_skill, span_kind);
        if let SessionSpanEvent::StageCompleted {
            stage_index,
            run_status,
            ..
        } = event
        {
            attrs.insert("popsicle.stage_index".into(), stage_index.to_string());
            attrs.insert(
                "popsicle.run_completed".into(),
                matches!(run_status, PipelineRunStatus::RunCompleted).to_string(),
            );
        }
        let _ = telemetry::record_span(&self.workspace_root, &ctx.run_id, span_name, &attrs);
    }
}

fn session_attrs(
    ctx: &SessionSpanContext,
    issue_key: &str,
    stage_name: &str,
    stage_skill: &str,
    span_kind: &str,
) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    attrs.insert("popsicle.run_id".into(), ctx.run_id.clone());
    attrs.insert("popsicle.trace_id".into(), ctx.run_id.clone());
    attrs.insert("popsicle.pipeline".into(), ctx.pipeline_name.clone());
    attrs.insert("popsicle.issue_key".into(), issue_key.to_string());
    attrs.insert("popsicle.stage".into(), stage_name.to_string());
    attrs.insert("popsicle.span.kind".into(), span_kind.into());
    attrs.insert(
        "popsicle.run_status".into(),
        run_status_label(ctx.run_status),
    );
    if !stage_skill.is_empty() {
        attrs.insert("popsicle.skill".into(), stage_skill.to_string());
    }
    attrs
}

fn base_attrs(run_id: &str, run_row: Option<&RunRow>) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    attrs.insert("popsicle.run_id".into(), run_id.to_string());
    attrs.insert("popsicle.trace_id".into(), run_id.to_string());
    if let Some(row) = run_row {
        attrs.insert("popsicle.issue_key".into(), row.issue_key.clone());
        attrs.insert("popsicle.pipeline".into(), row.pipeline_name.clone());
    }
    attrs
}

pub fn run_id_from_artifact_path(file_path: &str) -> Option<String> {
    let prefix = ".popsicle/artifacts/";
    let rest = file_path.strip_prefix(prefix)?;
    rest.split('/').next().map(str::to_string)
}

/// Suggested `tool run telemetry` for stage self-score after doc check passes.
pub fn score_hint(run_id: &str, doc_id: &str) -> String {
    format!(
        "popsicle tool run telemetry action=record span=popsicle.run.score run={run_id} doc={doc_id} score=4 rubric=stage-quality format=json --format json"
    )
}

/// `.popsicle/artifacts/{run}/{doc_id}.{skill}.md` → skill segment.
fn skill_from_doc_path(file_path: &str) -> Option<String> {
    let name = file_path.rsplit('/').next()?;
    let without_md = name.strip_suffix(".md")?;
    without_md
        .rsplit_once('.')
        .map(|(_, skill)| skill.to_string())
}

fn run_status_label(status: PipelineRunStatus) -> String {
    match status {
        PipelineRunStatus::RunPending => "pending".into(),
        PipelineRunStatus::RunInProgress => "in_progress".into(),
        PipelineRunStatus::RunCompleted => "completed".into(),
        PipelineRunStatus::RunBlocked => "blocked".into(),
    }
}

pub fn run_telemetry_tool(
    workspace_root: &Path,
    args: &BTreeMap<String, String>,
    json_format: bool,
) -> (i32, BTreeMap<String, String>) {
    let (result, code) = telemetry::run_tool(args, workspace_root);
    let fields = telemetry::result_to_json_fields(&result);
    if !json_format {
        if let Some(text) = &result.stdout {
            print!("{text}");
        } else if let Some(report) = &result.report_json {
            println!("{report}");
        } else if let Some(msg) = &result.message {
            eprintln!("telemetry: {msg}");
        } else if result.outcome == telemetry::RecordOutcome::Ok {
            if let Some(path) = &result.wal_path {
                eprintln!("telemetry: ok ({path})");
            }
        }
    }
    (code, fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_run_and_skill_from_artifact_path() {
        let p = ".popsicle/artifacts/run-abc/doc-1.shadow-implementer.md";
        assert_eq!(run_id_from_artifact_path(p).as_deref(), Some("run-abc"));
        assert_eq!(
            skill_from_doc_path(p).as_deref(),
            Some("shadow-implementer")
        );
    }

    #[test]
    fn score_hint_includes_run_and_doc() {
        let h = score_hint("run-1", "doc-99");
        assert!(h.contains("run=run-1"));
        assert!(h.contains("doc=doc-99"));
        assert!(h.contains("popsicle.run.score"));
    }
}
