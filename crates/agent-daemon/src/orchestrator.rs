//! Unattended pipeline loop: doc create → agent → doc check → stage complete → issue close.

use std::io;

use crate::cursor_agent::{auto_agent_enabled, CursorAgentAdapter};
use crate::orchestrator_parse::{
    doc_check_passed, parse_doc_id, parse_next_action, run_status_completed, NextAction,
};
use crate::pipeline_skill::skill_from_status_json;
use crate::prompt::load_skill_prompt;
use crate::server_client::{
    execute_doc_check, execute_doc_create, execute_issue_close, execute_pipeline_next,
    execute_pipeline_status, execute_stage_complete, sync_run_mirror, ServerClient,
};
use crate::PopsicleInvoker;

const DEFAULT_MAX_STEPS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestratorOutcome {
    RunCompleted,
    PausedForApproval,
    Failed,
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub max_steps: usize,
    pub invoke_agent: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_steps: DEFAULT_MAX_STEPS,
            invoke_agent: auto_agent_enabled(),
        }
    }
}

impl OrchestratorConfig {
    pub fn from_env() -> Self {
        let max_steps = std::env::var("AGENT_RUNTIME_ORCHESTRATOR_MAX_STEPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_STEPS);
        let invoke_agent = orchestrator_enabled() && auto_agent_enabled();
        Self {
            max_steps,
            invoke_agent,
        }
    }
}

pub fn orchestrator_enabled() -> bool {
    !matches!(
        std::env::var("AGENT_RUNTIME_ORCHESTRATOR").as_deref(),
        Ok("0") | Ok("false") | Ok("off")
    )
}

pub fn run_unattended(
    invoker: &PopsicleInvoker,
    client: Option<&ServerClient>,
    run_id: &str,
    issue_key: &str,
    log: impl Fn(&str),
    config: &OrchestratorConfig,
) -> io::Result<OrchestratorOutcome> {
    for step in 0..config.max_steps {
        let status = execute_pipeline_status(invoker, run_id)?;
        if status.exit_code != 0 {
            log(&format!(
                "orchestrator: pipeline status exit={}",
                status.exit_code
            ));
            return Ok(OrchestratorOutcome::Failed);
        }
        if run_status_completed(&status.stdout) {
            return finish_run(invoker, client, run_id, issue_key, &log);
        }
        if let Some(client) = client {
            if let Err(e) = sync_run_mirror(client, invoker, run_id, Some(issue_key)) {
                log(&format!("orchestrator: mirror sync error: {e}"));
            }
        }

        let next = execute_pipeline_next(invoker, run_id)?;
        if next.exit_code != 0 {
            log(&format!(
                "orchestrator: pipeline next exit={}",
                next.exit_code
            ));
            return Ok(OrchestratorOutcome::Failed);
        }
        let next_text = extract_next_field(&next.stdout).unwrap_or_default();
        log(&format!("orchestrator step={step} next={next_text}"));

        match parse_next_action(&next_text) {
            NextAction::AllStagesCompleted => {
                return finish_run(invoker, client, run_id, issue_key, &log);
            }
            NextAction::AwaitingApproval { stage } => {
                log(&format!(
                    "orchestrator: paused awaiting approval for stage={stage}"
                ));
                return Ok(OrchestratorOutcome::PausedForApproval);
            }
            NextAction::PipelineStatus | NextAction::Unknown(_) => {
                continue;
            }
            NextAction::StageComplete { stage, confirm } => {
                if !run_stage_work(
                    invoker,
                    run_id,
                    issue_key,
                    &stage,
                    &status.stdout,
                    config,
                    &log,
                )? {
                    return Ok(OrchestratorOutcome::Failed);
                }
                let complete = execute_stage_complete(invoker, &stage, run_id, confirm)?;
                if complete.exit_code != 0 {
                    log(&format!(
                        "orchestrator: stage complete exit={} stage={stage}",
                        complete.exit_code
                    ));
                    return Ok(OrchestratorOutcome::Failed);
                }
                if let Some(client) = client {
                    if let Err(e) = sync_run_mirror(client, invoker, run_id, Some(issue_key)) {
                        log(&format!("orchestrator: mirror sync error: {e}"));
                    }
                }
            }
        }
    }
    log("orchestrator: max steps exceeded");
    Ok(OrchestratorOutcome::Failed)
}

fn run_stage_work(
    invoker: &PopsicleInvoker,
    run_id: &str,
    issue_key: &str,
    stage: &str,
    status_json: &str,
    config: &OrchestratorConfig,
    log: &impl Fn(&str),
) -> io::Result<bool> {
    let Some(skill) = skill_from_status_json(invoker.workspace(), status_json)? else {
        log(&format!(
            "orchestrator: no skill for stage={stage}, skipping doc chain"
        ));
        return Ok(true);
    };
    let title = format!("{issue_key} {stage} 自动执行");
    let created = execute_doc_create(invoker, &skill, &title, run_id)?;
    if created.exit_code != 0 {
        log(&format!(
            "orchestrator: doc create exit={} skill={skill}",
            created.exit_code
        ));
        return Ok(false);
    }
    let Some(doc_id) = parse_doc_id(&created.stdout) else {
        log("orchestrator: doc create missing doc id");
        return Ok(false);
    };
    log(&format!("orchestrator: created {doc_id} skill={skill}"));

    if config.invoke_agent {
        if let Some(adapter) = CursorAgentAdapter::detect() {
            let mut prompt = load_skill_prompt(invoker.workspace(), &skill, run_id)?;
            prompt.push_str(&format!(
                "\n\n## Orchestrator instructions\n\
                 - Artifact doc id: `{doc_id}`\n\
                 - Run `popsicle doc show {doc_id}` then fill the document body per the skill guide.\n\
                 - Run `popsicle doc check {doc_id} --format json` until passed=true.\n\
                 - Do not run `pipeline stage complete` — the daemon completes the stage.\n"
            ));
            match adapter.invoke_prompt(&prompt) {
                Ok(result) => log_agent_output(
                    log,
                    &result.stdout,
                    &result.stderr,
                    result.exit_code,
                    result.dry_run,
                ),
                Err(e) => log(&format!("orchestrator: cursor-agent error: {e}")),
            }
        }
    }

    let checked = execute_doc_check(invoker, &doc_id)?;
    if checked.exit_code != 0 || !doc_check_passed(&checked.stdout) {
        log(&format!(
            "orchestrator: doc check failed exit={} doc={doc_id}",
            checked.exit_code
        ));
        return Ok(false);
    }
    log(&format!("orchestrator: doc check passed {doc_id}"));
    Ok(true)
}

fn finish_run(
    invoker: &PopsicleInvoker,
    client: Option<&ServerClient>,
    run_id: &str,
    issue_key: &str,
    log: &impl Fn(&str),
) -> io::Result<OrchestratorOutcome> {
    if let Some(client) = client {
        if let Err(e) = sync_run_mirror(client, invoker, run_id, Some(issue_key)) {
            log(&format!("orchestrator: final mirror sync error: {e}"));
        }
    }
    let closed = execute_issue_close(invoker, issue_key)?;
    if closed.exit_code != 0 {
        log(&format!(
            "orchestrator: issue close exit={} key={issue_key}",
            closed.exit_code
        ));
        return Ok(OrchestratorOutcome::Failed);
    }
    log(&format!(
        "orchestrator: run completed issue={issue_key} run={run_id}"
    ));
    Ok(OrchestratorOutcome::RunCompleted)
}

fn extract_next_field(stdout: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .and_then(|v| v.get("next").and_then(|n| n.as_str()).map(str::to_string))
}

fn log_agent_output(
    log: &impl Fn(&str),
    stdout: &str,
    stderr: &str,
    exit_code: i32,
    dry_run: bool,
) {
    log(&format!("cursor-agent: exit={exit_code} dry_run={dry_run}"));
    for line in stdout.lines().take(120) {
        let t = line.trim();
        if !t.is_empty() {
            log(&format!("› {t}"));
        }
    }
    for line in stderr.lines().take(60) {
        let t = line.trim();
        if !t.is_empty() {
            log(&format!("✗ {t}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orchestrator_enabled_by_default() {
        std::env::remove_var("AGENT_RUNTIME_ORCHESTRATOR");
        assert!(orchestrator_enabled());
    }
}
