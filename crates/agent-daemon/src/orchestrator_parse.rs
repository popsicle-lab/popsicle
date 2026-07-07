//! Parse pipeline CLI JSON and next-step commands for unattended orchestration.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextAction {
    AllStagesCompleted,
    StageComplete { stage: String, confirm: bool },
    AwaitingApproval { stage: String },
    PipelineStatus,
    Unknown(String),
}

pub fn run_status_completed(status_json: &str) -> bool {
    parse_field(status_json, "run_status").is_some_and(|s| s == "completed")
}

pub fn parse_next_action(next_text: &str) -> NextAction {
    let text = next_text.trim();
    if text.contains("all stages completed") {
        return NextAction::AllStagesCompleted;
    }
    if text.contains("approve then") {
        if let Some(stage) = extract_stage_from_complete_cmd(text) {
            return NextAction::AwaitingApproval { stage };
        }
    }
    if text.contains("pipeline stage complete") {
        if let Some(stage) = extract_stage_from_complete_cmd(text) {
            let confirm = text.contains("--confirm");
            return NextAction::StageComplete { stage, confirm };
        }
    }
    if text.contains("pipeline status") {
        return NextAction::PipelineStatus;
    }
    NextAction::Unknown(text.to_string())
}

pub fn parse_doc_id(doc_create_stdout: &str) -> Option<String> {
    serde_json::from_str::<Value>(doc_create_stdout.trim())
        .ok()
        .and_then(|v| v.get("id").and_then(|id| id.as_str()).map(str::to_string))
        .filter(|s| !s.is_empty())
}

pub fn doc_check_passed(doc_check_stdout: &str) -> bool {
    serde_json::from_str::<Value>(doc_check_stdout.trim())
        .ok()
        .and_then(|v| {
            v.get("passed")
                .and_then(|p| p.as_str())
                .map(|s| s == "true")
        })
        .unwrap_or(false)
}

fn parse_field(json: &str, key: &str) -> Option<String> {
    serde_json::from_str::<Value>(json.trim())
        .ok()
        .and_then(|v| v.get(key).and_then(|x| x.as_str()).map(str::to_string))
        .filter(|s| !s.is_empty())
}

fn extract_stage_from_complete_cmd(text: &str) -> Option<String> {
    let marker = "pipeline stage complete ";
    let start = text.find(marker)? + marker.len();
    let rest = &text[start..];
    let stage = rest.split_whitespace().next()?;
    Some(stage.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_completed_run_status() {
        let json = r#"{"run_status":"completed","current_stage":"doc-sync"}"#;
        assert!(run_status_completed(json));
    }

    #[test]
    fn parses_stage_complete_without_confirm() {
        let next = "popsicle pipeline stage complete implement --run run-1";
        assert_eq!(
            parse_next_action(next),
            NextAction::StageComplete {
                stage: "implement".into(),
                confirm: false,
            }
        );
    }

    #[test]
    fn parses_awaiting_approval() {
        let next = "approve then `popsicle pipeline stage complete cutover --run run-1 --confirm`";
        assert_eq!(
            parse_next_action(next),
            NextAction::AwaitingApproval {
                stage: "cutover".into(),
            }
        );
    }

    #[test]
    fn parses_doc_id_from_create_json() {
        let stdout = r#"{"id":"doc-211","status":"ok"}"#;
        assert_eq!(parse_doc_id(stdout), Some("doc-211".into()));
    }
}
