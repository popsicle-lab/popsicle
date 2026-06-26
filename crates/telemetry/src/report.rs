//! WAL → RunReport aggregation (ADR-002 / T-TEL-0004).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::wal::{read_wal_lines, telemetry_root, WalLine};

#[derive(Debug, Clone, Serialize)]
pub struct StageReport {
    pub name: String,
    pub skill: Option<String>,
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillCheckCounts {
    pub passed: u32,
    pub failed: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocChecksReport {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub by_skill: BTreeMap<String, SkillCheckCounts>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentCoverageReport {
    pub gen_ai_chat: bool,
    pub run_score: bool,
    pub decision: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunReport {
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<String>,
    pub span_count: u32,
    pub stages: Vec<StageReport>,
    pub doc_checks: DocChecksReport,
    pub agent_coverage: AgentCoverageReport,
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentReport {
    pub runs: Vec<RunReport>,
    pub run_count: u32,
    pub doc_check_failures: u32,
    pub runs_with_score: u32,
    pub status: &'static str,
}

pub fn report_run(workspace_root: &Path, run_id: &str) -> RunReport {
    let lines = read_wal_lines(workspace_root, run_id);
    if lines.is_empty() && !wal_exists(workspace_root, run_id) {
        return empty_run_report(run_id, "degraded");
    }
    build_run_report(run_id, lines, "ok")
}

pub fn report_recent(workspace_root: &Path, limit: usize) -> RecentReport {
    let ids = list_run_ids(workspace_root, limit);
    if ids.is_empty() {
        return RecentReport {
            runs: vec![],
            run_count: 0,
            doc_check_failures: 0,
            runs_with_score: 0,
            status: "degraded",
        };
    }
    let mut runs = Vec::new();
    let mut doc_check_failures = 0u32;
    let mut runs_with_score = 0u32;
    for id in &ids {
        let report = report_run(workspace_root, id);
        doc_check_failures += report.doc_checks.failed;
        if report.agent_coverage.run_score {
            runs_with_score += 1;
        }
        runs.push(report);
    }
    RecentReport {
        run_count: runs.len() as u32,
        doc_check_failures,
        runs_with_score,
        runs,
        status: "ok",
    }
}

/// One-line summary for `docs/PROJECT_CONTEXT.md` §现在状态 (doc-sync-weekly).
pub fn health_summary_line(workspace_root: &Path, limit: usize) -> String {
    let recent = report_recent(workspace_root, limit);
    if recent.run_count == 0 {
        return "最近无 WAL run（telemetry 旁路未写入或未配置）".into();
    }
    format!(
        "最近 {} 个 run；doc_check 失败 {} 次；{} 个 run 含 Agent score",
        recent.run_count, recent.doc_check_failures, recent.runs_with_score
    )
}

fn wal_exists(workspace_root: &Path, run_id: &str) -> bool {
    crate::wal::wal_path(workspace_root, run_id).is_file()
}

fn empty_run_report(run_id: &str, status: &'static str) -> RunReport {
    RunReport {
        run_id: run_id.to_string(),
        issue_key: None,
        pipeline: None,
        span_count: 0,
        stages: vec![],
        doc_checks: empty_doc_checks(),
        agent_coverage: AgentCoverageReport {
            gen_ai_chat: false,
            run_score: false,
            decision: false,
        },
        status,
    }
}

fn build_run_report(run_id: &str, lines: Vec<WalLine>, status: &'static str) -> RunReport {
    let mut issue_key = None;
    let mut pipeline = None;
    let mut stages: BTreeMap<String, StageReport> = BTreeMap::new();
    let mut doc_checks = empty_doc_checks();
    let mut agent = AgentCoverageReport {
        gen_ai_chat: false,
        run_score: false,
        decision: false,
    };

    for line in &lines {
        if issue_key.is_none() {
            issue_key = attr(line, "popsicle.issue_key");
        }
        if pipeline.is_none() {
            pipeline = attr(line, "popsicle.pipeline");
        }
        match line.span.as_str() {
            "gen_ai.chat" => agent.gen_ai_chat = true,
            "popsicle.run.score" => agent.run_score = true,
            "popsicle.decision" => agent.decision = true,
            "popsicle.doc.check" => {
                doc_checks.total += 1;
                let skill = attr(line, "popsicle.skill").unwrap_or_else(|| "unknown".into());
                let passed = attr(line, "popsicle.doc_check.passed")
                    .map(|v| v == "true")
                    .unwrap_or(false);
                let entry = doc_checks
                    .by_skill
                    .entry(skill)
                    .or_insert(SkillCheckCounts {
                        passed: 0,
                        failed: 0,
                    });
                if passed {
                    doc_checks.passed += 1;
                    entry.passed += 1;
                } else {
                    doc_checks.failed += 1;
                    entry.failed += 1;
                }
            }
            "popsicle.stage.complete" => {
                let name = attr(line, "popsicle.stage").unwrap_or_else(|| "unknown".into());
                let skill = attr(line, "popsicle.skill");
                let duration_ms = attr(line, "popsicle.duration_ms").and_then(|s| s.parse().ok());
                stages.insert(
                    name.clone(),
                    StageReport {
                        name,
                        skill,
                        completed: true,
                        duration_ms,
                    },
                );
            }
            "popsicle.run.start" => {
                let name = attr(line, "popsicle.stage").unwrap_or_else(|| "start".into());
                let skill = attr(line, "popsicle.skill");
                stages.entry(name.clone()).or_insert(StageReport {
                    name,
                    skill,
                    completed: false,
                    duration_ms: None,
                });
            }
            _ => {}
        }
    }

    RunReport {
        run_id: run_id.to_string(),
        issue_key,
        pipeline,
        span_count: lines.len() as u32,
        stages: stages.into_values().collect(),
        doc_checks,
        agent_coverage: agent,
        status,
    }
}

fn empty_doc_checks() -> DocChecksReport {
    DocChecksReport {
        total: 0,
        passed: 0,
        failed: 0,
        by_skill: BTreeMap::new(),
    }
}

fn attr(line: &WalLine, key: &str) -> Option<String> {
    line.attributes.get(key).cloned()
}

fn list_run_ids(workspace_root: &Path, limit: usize) -> Vec<String> {
    let root = telemetry_root(workspace_root);
    let Ok(entries) = fs::read_dir(&root) else {
        return vec![];
    };
    let mut dirs: Vec<(std::time::SystemTime, String)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            let name = e.file_name().to_string_lossy().into_owned();
            Some((modified, name))
        })
        .collect();
    dirs.sort_by(|a, b| b.0.cmp(&a.0));
    dirs.into_iter()
        .take(limit.max(1))
        .map(|(_, id)| id)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::wal::append_span;

    fn tmp() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("telemetry-report-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn report_aggregates_stage_and_doc_check() {
        let root = tmp();
        let run = "run-rpt";
        let mut base = BTreeMap::new();
        base.insert("popsicle.issue_key".into(), "PROJ-1".into());
        base.insert("popsicle.pipeline".into(), "feature-delivery".into());
        base.insert("popsicle.stage".into(), "implement".into());
        base.insert("popsicle.skill".into(), "shadow-implementer".into());
        append_span(&root, run, "popsicle.run.start", &base).unwrap();
        append_span(&root, run, "popsicle.stage.complete", &base).unwrap();
        let mut check = base.clone();
        check.insert("popsicle.doc_check.passed".into(), "true".into());
        append_span(&root, run, "popsicle.doc.check", &check).unwrap();
        append_span(&root, run, "gen_ai.chat", &base).unwrap();
        append_span(&root, run, "popsicle.run.score", &base).unwrap();

        let r = report_run(&root, run);
        assert_eq!(r.status, "ok");
        assert_eq!(r.span_count, 5);
        assert_eq!(r.issue_key.as_deref(), Some("PROJ-1"));
        assert_eq!(r.doc_checks.total, 1);
        assert_eq!(r.doc_checks.passed, 1);
        assert!(r.agent_coverage.gen_ai_chat);
        assert!(r.agent_coverage.run_score);
        assert!(r
            .stages
            .iter()
            .any(|s| s.name == "implement" && s.completed));
    }

    #[test]
    fn missing_wal_is_degraded_fail_open() {
        let root = tmp();
        let r = report_run(&root, "no-such-run");
        assert_eq!(r.status, "degraded");
        assert_eq!(r.span_count, 0);
    }

    #[test]
    fn health_summary_empty_runs() {
        let root = tmp();
        let line = health_summary_line(&root, 5);
        assert!(line.contains("无 WAL"));
    }
}
