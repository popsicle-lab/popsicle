//! HTTP client for agent-server poll/claim/heartbeat (P0/P1).

use std::io;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::orchestrator::{
    orchestrator_enabled, run_unattended, OrchestratorConfig, OrchestratorOutcome,
};
use crate::{CursorAgentAdapter, PopsicleInvokeResult, PopsicleInvoker};

const DEFAULT_POLL_SECS: u64 = 3;
const DEFAULT_HEARTBEAT_SECS: u64 = 15;

fn ureq_empty_on_not_found<T, F>(
    result: Result<ureq::Response, ureq::Error>,
    f: F,
) -> io::Result<Option<T>>
where
    F: FnOnce(ureq::Response) -> io::Result<T>,
{
    match result {
        Ok(resp) => Ok(Some(f(resp)?)),
        Err(ureq::Error::Status(404, _)) => Ok(None),
        Err(e) => Err(io::Error::other(e.to_string())),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimedTask {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub id: String,
    pub issue_key: String,
    pub pipeline: String,
    #[serde(default)]
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DispatchResponse {
    pub accepted: bool,
    pub state: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub task: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeStateResponse {
    pub runtime_id: String,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerHealthResponse {
    pub status: String,
    pub storage: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunMirrorSummary {
    pub run_id: String,
    #[serde(default)]
    pub issue_key: Option<String>,
    pub pipeline: String,
    pub run_status: String,
    pub current_stage: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimedConfirm {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub id: String,
    pub run_id: String,
    pub stage: String,
}

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        other => Err(D::Error::custom(format!(
            "expected uuid string, got {other}"
        ))),
    }
}

#[derive(Debug, Clone)]
pub struct ServerClient {
    base_url: String,
    runtime_id: String,
}

impl ServerClient {
    pub fn new(base_url: impl Into<String>, runtime_id: impl Into<String>) -> Self {
        let mut base = base_url.into();
        while base.ends_with('/') {
            base.pop();
        }
        Self {
            base_url: base,
            runtime_id: runtime_id.into(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let base = std::env::var("AGENT_RUNTIME_SERVER_URL").ok()?;
        if base.is_empty() {
            return None;
        }
        let runtime_id = std::env::var("AGENT_RUNTIME_ID").unwrap_or_else(|_| "default".into());
        Some(Self::new(base, runtime_id))
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn heartbeat(&self) -> io::Result<()> {
        let url = format!(
            "{}/v1/runtimes/{}/heartbeat",
            self.base_url, self.runtime_id
        );
        let resp = ureq::post(&url)
            .set("Content-Length", "0")
            .call()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "heartbeat failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    pub fn claim(&self) -> io::Result<Option<ClaimedTask>> {
        let url = format!(
            "{}/v1/runtimes/{}/tasks/claim",
            self.base_url, self.runtime_id
        );
        ureq_empty_on_not_found(
            ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_string(&format!(r#"{{"runtime_id":"{}"}}"#, self.runtime_id)),
            |resp| {
                if !(200..300).contains(&resp.status()) {
                    return Err(io::Error::other(format!(
                        "claim failed: HTTP {}",
                        resp.status()
                    )));
                }
                resp.into_json()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            },
        )
    }

    pub fn server_health(&self) -> io::Result<ServerHealthResponse> {
        let url = format!("{}/health", self.base_url);
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "health failed: HTTP {}",
                resp.status()
            )));
        }
        resp.into_json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn runtime_state(&self) -> io::Result<RuntimeStateResponse> {
        let url = format!("{}/v1/runtimes/{}", self.base_url, self.runtime_id);
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "runtime state failed: HTTP {}",
                resp.status()
            )));
        }
        resp.into_json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn dispatch_issue(
        &self,
        workspace_id: &str,
        issue_key: &str,
        pipeline: &str,
    ) -> io::Result<DispatchResponse> {
        let url = format!("{}/v1/dispatch", self.base_url);
        let body = serde_json::json!({
            "workspace_id": workspace_id,
            "runtime_id": self.runtime_id,
            "issue_key": issue_key,
            "pipeline": pipeline,
        });
        let resp = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body.to_string())
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "dispatch failed: HTTP {}",
                resp.status()
            )));
        }
        resp.into_json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn list_run_mirrors(&self) -> io::Result<Vec<RunMirrorSummary>> {
        let url = format!("{}/v1/runs", self.base_url);
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "list runs failed: HTTP {}",
                resp.status()
            )));
        }
        resp.into_json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn claim_confirm(&self) -> io::Result<Option<ClaimedConfirm>> {
        let url = format!(
            "{}/v1/runtimes/{}/confirms/claim",
            self.base_url, self.runtime_id
        );
        ureq_empty_on_not_found(
            ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_string(&format!(r#"{{"runtime_id":"{}"}}"#, self.runtime_id)),
            |resp| {
                if !(200..300).contains(&resp.status()) {
                    return Err(io::Error::other(format!(
                        "confirm claim failed: HTTP {}",
                        resp.status()
                    )));
                }
                resp.into_json()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            },
        )
    }
}

pub fn parse_run_id_from_issue_start(stdout: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .and_then(|v| v.get("run_id").and_then(|r| r.as_str()).map(str::to_string))
        .filter(|s| !s.is_empty())
}

pub fn execute_issue_start(
    invoker: &PopsicleInvoker,
    issue_key: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run(["issue", "start", issue_key, "--format", "json"])
}

pub fn execute_pipeline_next(
    invoker: &PopsicleInvoker,
    run_id: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run(["pipeline", "next", "--run", run_id, "--format", "json"])
}

pub fn execute_pipeline_status(
    invoker: &PopsicleInvoker,
    run_id: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run(["pipeline", "status", "--run", run_id, "--format", "json"])
}

pub fn execute_stage_complete(
    invoker: &PopsicleInvoker,
    stage: &str,
    run_id: &str,
    confirm: bool,
) -> io::Result<PopsicleInvokeResult> {
    let mut args = vec![
        "pipeline", "stage", "complete", stage, "--run", run_id, "--format", "json",
    ];
    if confirm {
        args.push("--confirm");
    }
    invoker.run(args)
}

pub fn execute_doc_create(
    invoker: &PopsicleInvoker,
    skill: &str,
    title: &str,
    run_id: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run([
        "doc", "create", skill, "--title", title, "--run", run_id, "--format", "json",
    ])
}

pub fn execute_doc_check(
    invoker: &PopsicleInvoker,
    doc_id: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run(["doc", "check", doc_id, "--format", "json"])
}

pub fn execute_issue_close(
    invoker: &PopsicleInvoker,
    issue_key: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run(["issue", "close", issue_key, "--format", "json"])
}

impl ServerClient {
    pub fn run_issue_key(&self, run_id: &str) -> io::Result<Option<String>> {
        let url = format!("{}/v1/runs/{run_id}", self.base_url);
        match ureq::get(&url).call() {
            Ok(resp) => {
                if !(200..300).contains(&resp.status()) {
                    return Err(io::Error::other(format!(
                        "get run failed: HTTP {}",
                        resp.status()
                    )));
                }
                let mirror: serde_json::Value = resp
                    .into_json()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                Ok(mirror
                    .get("issue_key")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .filter(|s| !s.is_empty()))
            }
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(e) => Err(io::Error::other(e.to_string())),
        }
    }

    pub fn post_run_mirror(
        &self,
        run_id: &str,
        issue_key: Option<&str>,
        status_json: &str,
    ) -> io::Result<()> {
        let mut body: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(status_json.trim())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if let Some(key) = issue_key {
            body.insert("issue_key".into(), serde_json::Value::String(key.into()));
        }
        let url = format!("{}/v1/runs/{run_id}/mirror", self.base_url);
        let resp = ureq::put(&url)
            .set("Content-Type", "application/json")
            .send_string(&serde_json::Value::Object(body).to_string())
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "mirror upsert failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    pub fn post_run_log(&self, run_id: &str, level: &str, message: &str) -> io::Result<()> {
        let url = format!("{}/v1/runs/{run_id}/logs", self.base_url);
        let body = serde_json::json!({ "level": level, "message": message });
        let resp = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body.to_string())
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "run log failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }
}

pub fn sync_run_mirror(
    client: &ServerClient,
    invoker: &PopsicleInvoker,
    run_id: &str,
    issue_key: Option<&str>,
) -> io::Result<()> {
    let status = execute_pipeline_status(invoker, run_id)?;
    if status.exit_code != 0 {
        return Err(io::Error::other(format!(
            "pipeline status exit {}",
            status.exit_code
        )));
    }
    client.post_run_mirror(run_id, issue_key, &status.stdout)
}

/// Blocking poll loop: heartbeat, claim tasks, subprocess issue start + pipeline next.
pub fn run_poll_loop(invoker: &PopsicleInvoker, client: &ServerClient, log_path: Option<&Path>) {
    let poll_interval = Duration::from_secs(
        std::env::var("AGENT_RUNTIME_POLL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_POLL_SECS),
    );
    let heartbeat_interval = Duration::from_secs(
        std::env::var("AGENT_RUNTIME_HEARTBEAT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_HEARTBEAT_SECS),
    );
    let mut last_heartbeat = Instant::now() - heartbeat_interval;
    loop {
        if last_heartbeat.elapsed() >= heartbeat_interval {
            match client.heartbeat() {
                Ok(()) => append_log(log_path, "heartbeat ok"),
                Err(e) => append_log(log_path, &format!("heartbeat error: {e}")),
            }
            last_heartbeat = Instant::now();
        }
        match client.claim_confirm() {
            Ok(Some(confirm)) => {
                let msg = format!(
                    "claimed confirm {} run={} stage={}",
                    confirm.id, confirm.run_id, confirm.stage
                );
                append_log(log_path, &msg);
                match execute_stage_complete(invoker, &confirm.stage, &confirm.run_id, true) {
                    Ok(result) => {
                        append_log(
                            log_path,
                            &format!(
                                "stage complete exit={} run={} stage={}",
                                result.exit_code, confirm.run_id, confirm.stage
                            ),
                        );
                        if result.exit_code == 0 {
                            if let Err(e) = sync_run_mirror(client, invoker, &confirm.run_id, None)
                            {
                                append_log(log_path, &format!("mirror sync error: {e}"));
                            }
                            if orchestrator_enabled() {
                                resume_orchestrator_after_confirm(
                                    invoker,
                                    client,
                                    &confirm.run_id,
                                    log_path,
                                );
                            }
                        }
                    }
                    Err(e) => append_log(log_path, &format!("stage complete error: {e}")),
                }
            }
            Ok(None) => match client.claim() {
                Ok(Some(task)) => {
                    if let Some(ref run_id) = task.run_id {
                        let msg = format!(
                            "claimed resume {} issue={} run={}",
                            task.id, task.issue_key, run_id
                        );
                        append_log(log_path, &msg);
                        if let Err(e) =
                            sync_run_mirror(client, invoker, run_id, Some(task.issue_key.as_str()))
                        {
                            append_log(log_path, &format!("mirror sync error: {e}"));
                        }
                        if orchestrator_enabled() {
                            run_orchestrator_for_task(
                                invoker,
                                client,
                                run_id,
                                task.issue_key.as_str(),
                                log_path,
                            );
                        } else {
                            run_legacy_bootstrap(
                                invoker,
                                client,
                                run_id,
                                Some(task.issue_key.as_str()),
                                log_path,
                            );
                        }
                    } else {
                        let msg = format!(
                            "claimed task {} issue={} pipeline={}",
                            task.id, task.issue_key, task.pipeline
                        );
                        append_log(log_path, &msg);
                        match execute_issue_start(invoker, &task.issue_key) {
                            Ok(result) => {
                                append_log(
                                    log_path,
                                    &format!(
                                        "issue start exit={} stdout_len={}",
                                        result.exit_code,
                                        result.stdout.len()
                                    ),
                                );
                                if result.exit_code == 0 {
                                    if let Some(run_id) =
                                        parse_run_id_from_issue_start(&result.stdout)
                                    {
                                        let issue_key = task.issue_key.as_str();
                                        if let Err(e) = sync_run_mirror(
                                            client,
                                            invoker,
                                            &run_id,
                                            Some(issue_key),
                                        ) {
                                            append_log(
                                                log_path,
                                                &format!("mirror sync error: {e}"),
                                            );
                                        }
                                        if orchestrator_enabled() {
                                            run_orchestrator_for_task(
                                                invoker, client, &run_id, issue_key, log_path,
                                            );
                                        } else {
                                            run_legacy_bootstrap(
                                                invoker,
                                                client,
                                                &run_id,
                                                Some(issue_key),
                                                log_path,
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => append_log(log_path, &format!("issue start error: {e}")),
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => append_log(log_path, &format!("claim error: {e}")),
            },
            Err(e) => append_log(log_path, &format!("confirm claim error: {e}")),
        }
        thread::sleep(poll_interval);
    }
}

fn run_legacy_bootstrap(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    run_id: &str,
    issue_key: Option<&str>,
    log_path: Option<&Path>,
) {
    match execute_pipeline_next(invoker, run_id) {
        Ok(next) => {
            append_log(
                log_path,
                &format!("pipeline next exit={} run={run_id}", next.exit_code),
            );
            if next.exit_code == 0 {
                if let Err(e) = sync_run_mirror(client, invoker, run_id, issue_key) {
                    append_log(log_path, &format!("mirror sync error: {e}"));
                }
                if let Some(adapter) = CursorAgentAdapter::detect() {
                    match adapter.maybe_run_for_run(invoker, run_id) {
                        Ok(Some(result)) => append_log(
                            log_path,
                            &format!(
                                "cursor-agent exit={} dry_run={} stdout_len={}",
                                result.exit_code,
                                result.dry_run,
                                result.stdout.len()
                            ),
                        ),
                        Ok(None) => {}
                        Err(e) => append_log(log_path, &format!("cursor-agent error: {e}")),
                    }
                }
            }
        }
        Err(e) => append_log(log_path, &format!("pipeline next error: {e}")),
    }
}

fn run_orchestrator_for_task(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    run_id: &str,
    issue_key: &str,
    log_path: Option<&Path>,
) {
    let config = OrchestratorConfig::from_env();
    let log = |line: &str| emit_run_log(log_path, Some(client), run_id, "info", line);
    match run_unattended(invoker, Some(client), run_id, issue_key, log, &config) {
        Ok(OrchestratorOutcome::RunCompleted) => {
            emit_run_log(
                log_path,
                Some(client),
                run_id,
                "info",
                "orchestrator: run completed",
            );
        }
        Ok(OrchestratorOutcome::PausedForApproval) => {
            emit_run_log(
                log_path,
                Some(client),
                run_id,
                "info",
                "orchestrator: paused for approval",
            );
        }
        Ok(OrchestratorOutcome::Failed) => {
            emit_run_log(
                log_path,
                Some(client),
                run_id,
                "error",
                "orchestrator: failed",
            );
        }
        Err(e) => emit_run_log(
            log_path,
            Some(client),
            run_id,
            "error",
            &format!("orchestrator error: {e}"),
        ),
    }
}

fn resume_orchestrator_after_confirm(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    run_id: &str,
    log_path: Option<&Path>,
) {
    let issue_key = client
        .run_issue_key(run_id)
        .ok()
        .flatten()
        .unwrap_or_else(|| "UNKNOWN".into());
    if issue_key == "UNKNOWN" {
        append_log(
            log_path,
            "orchestrator: issue_key missing on run mirror for resume",
        );
    }
    run_orchestrator_for_task(invoker, client, run_id, &issue_key, log_path);
}

fn append_log(path: Option<&Path>, line: &str) {
    if let Some(path) = path {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "{line}");
        }
    }
}

fn emit_run_log(
    path: Option<&Path>,
    client: Option<&ServerClient>,
    run_id: &str,
    level: &str,
    line: &str,
) {
    append_log(path, line);
    if let Some(client) = client {
        let _ = client.post_run_log(run_id, level, line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_run_id_from_json_stdout() {
        let stdout = r#"{"run_id":"00000051-0000-4051-8005-51000000000051","status":"ok"}"#;
        assert_eq!(
            parse_run_id_from_issue_start(stdout),
            Some("00000051-0000-4051-8005-51000000000051".into())
        );
    }
}
