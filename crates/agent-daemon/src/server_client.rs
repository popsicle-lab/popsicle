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

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimedChatTurn {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub session_id: String,
    pub runtime_id: String,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub user_message_id: String,
    pub user_content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimedBootstrap {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub session_id: String,
    pub runtime_id: String,
    pub workspace_id: String,
    pub product_id: String,
    pub draft_title: String,
    pub draft_pipeline: String,
    pub draft_description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatSessionView {
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(default)]
    pub draft_title: Option<String>,
    #[serde(default)]
    pub draft_pipeline: Option<String>,
    #[serde(default)]
    pub draft_description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub linked_issue_key: Option<String>,
    #[serde(default)]
    pub linked_run_id: Option<String>,
    #[serde(default)]
    pub messages: Vec<ChatMessageRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageRow {
    pub role: String,
    pub content: String,
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

    pub fn claim_chat_turn(&self) -> io::Result<Option<ClaimedChatTurn>> {
        let url = format!(
            "{}/v1/runtimes/{}/chat-turns/claim",
            self.base_url, self.runtime_id
        );
        ureq_empty_on_not_found(
            ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_string(&format!(r#"{{"runtime_id":"{}"}}"#, self.runtime_id)),
            |resp| {
                if !(200..300).contains(&resp.status()) {
                    return Err(io::Error::other(format!(
                        "chat turn claim failed: HTTP {}",
                        resp.status()
                    )));
                }
                resp.into_json()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            },
        )
    }

    pub fn claim_bootstrap(&self) -> io::Result<Option<ClaimedBootstrap>> {
        let url = format!(
            "{}/v1/runtimes/{}/bootstraps/claim",
            self.base_url, self.runtime_id
        );
        ureq_empty_on_not_found(
            ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_string(&format!(r#"{{"runtime_id":"{}"}}"#, self.runtime_id)),
            |resp| {
                if !(200..300).contains(&resp.status()) {
                    return Err(io::Error::other(format!(
                        "bootstrap claim failed: HTTP {}",
                        resp.status()
                    )));
                }
                resp.into_json()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            },
        )
    }

    pub fn get_chat_session(&self, session_id: &str) -> io::Result<Option<ChatSessionView>> {
        let url = format!("{}/v1/chat/sessions/{session_id}", self.base_url);
        ureq_empty_on_not_found(ureq::get(&url).call(), |resp| {
            if !(200..300).contains(&resp.status()) {
                return Err(io::Error::other(format!(
                    "get chat session failed: HTTP {}",
                    resp.status()
                )));
            }
            resp.into_json()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        })
    }

    pub fn complete_chat_turn(&self, params: CompleteChatTurnParams<'_>) -> io::Result<()> {
        let url = format!(
            "{}/v1/chat/sessions/{}/turn-complete",
            self.base_url, params.session_id
        );
        let body = serde_json::json!({
            "runtime_id": self.runtime_id,
            "turn_id": params.turn_id,
            "assistant_content": params.assistant_content,
            "draft_title": params.draft_title,
            "draft_pipeline": params.draft_pipeline,
            "draft_description": params.draft_description,
            "mark_ready": params.mark_ready,
        });
        let resp = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body.to_string())
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "complete chat turn failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    pub fn complete_bootstrap(
        &self,
        session_id: &str,
        task_id: &str,
        issue_key: &str,
        run_id: &str,
    ) -> io::Result<()> {
        let url = format!(
            "{}/v1/chat/sessions/{session_id}/bootstrap-complete",
            self.base_url,
        );
        let body = serde_json::json!({
            "runtime_id": self.runtime_id,
            "task_id": task_id,
            "issue_key": issue_key,
            "run_id": run_id,
        });
        let resp = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body.to_string())
            .map_err(|e| io::Error::other(e.to_string()))?;
        if !(200..300).contains(&resp.status()) {
            return Err(io::Error::other(format!(
                "complete bootstrap failed: HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CompleteChatTurnParams<'a> {
    pub session_id: &'a str,
    pub turn_id: &'a str,
    pub assistant_content: &'a str,
    pub draft_title: Option<&'a str>,
    pub draft_pipeline: Option<&'a str>,
    pub draft_description: Option<&'a str>,
    pub mark_ready: bool,
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

pub fn execute_issue_list(invoker: &PopsicleInvoker) -> io::Result<PopsicleInvokeResult> {
    invoker.run(["issue", "list", "--format", "json"])
}

pub fn execute_issue_show(
    invoker: &PopsicleInvoker,
    issue_key: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run(["issue", "show", issue_key, "--format", "json"])
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

pub fn execute_issue_create(
    invoker: &PopsicleInvoker,
    issue_type: &str,
    title: &str,
    product: &str,
    pipeline: &str,
    description: &str,
) -> io::Result<PopsicleInvokeResult> {
    invoker.run([
        "issue",
        "create",
        "--type",
        issue_type,
        "--title",
        title,
        "--product",
        product,
        "--pipeline",
        pipeline,
        "--description",
        description,
        "--format",
        "json",
    ])
}

pub fn parse_issue_key_from_create(stdout: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .and_then(|v| v.get("key").and_then(|k| k.as_str()).map(str::to_string))
        .filter(|s| !s.is_empty())
}

impl ServerClient {
    pub fn run_issue_key(&self, run_id: &str) -> io::Result<Option<String>> {
        Ok(self
            .get_run_mirror(run_id)?
            .and_then(|m| m.issue_key)
            .filter(|s| !s.is_empty()))
    }

    pub fn get_run_mirror(&self, run_id: &str) -> io::Result<Option<RunMirrorSummary>> {
        let url = format!("{}/v1/runs/{run_id}", self.base_url);
        ureq_empty_on_not_found(ureq::get(&url).call(), |resp| {
            if !(200..300).contains(&resp.status()) {
                return Err(io::Error::other(format!(
                    "get run failed: HTTP {}",
                    resp.status()
                )));
            }
            resp.into_json()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        })
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
        if let Some(key) = issue_key.filter(|k| !k.is_empty() && *k != "UNKNOWN") {
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
    let key = resolve_issue_key(issue_key, &status.stdout, client, run_id);
    client.post_run_mirror(run_id, key.as_deref(), &status.stdout)
}

fn issue_key_from_status_json(status_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(status_json.trim())
        .ok()
        .and_then(|v| {
            v.get("issue_key")
                .and_then(|x| x.as_str())
                .map(str::to_string)
        })
        .filter(|s| !s.is_empty())
}

fn resolve_issue_key(
    explicit: Option<&str>,
    status_json: &str,
    client: &ServerClient,
    run_id: &str,
) -> Option<String> {
    if let Some(key) = explicit.filter(|s| !s.is_empty() && *s != "UNKNOWN") {
        return Some(key.to_string());
    }
    if let Some(key) = issue_key_from_status_json(status_json) {
        return Some(key);
    }
    client
        .run_issue_key(run_id)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty() && s != "UNKNOWN")
}

fn resolve_issue_key_for_run(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    run_id: &str,
) -> String {
    if let Ok(status) = execute_pipeline_status(invoker, run_id) {
        if status.exit_code == 0 {
            if let Some(key) = resolve_issue_key(None, &status.stdout, client, run_id) {
                return key;
            }
        }
    }
    client
        .run_issue_key(run_id)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty() && s != "UNKNOWN")
        .unwrap_or_else(|| "UNKNOWN".into())
}

fn reconcile_run_mirrors(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    log_path: Option<&Path>,
) {
    discover_active_run_mirrors(invoker, client, log_path);
    let Ok(runs) = client.list_run_mirrors() else {
        return;
    };
    for run in runs {
        if let Err(e) = sync_run_mirror(client, invoker, &run.run_id, run.issue_key.as_deref()) {
            append_log(log_path, &format!("mirror reconcile {}: {e}", run.run_id));
        }
    }
}

/// Push mirrors for in-progress issues that have an active run but are not yet on the server.
fn discover_active_run_mirrors(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    log_path: Option<&Path>,
) {
    let Ok(list) = execute_issue_list(invoker) else {
        return;
    };
    if list.exit_code != 0 {
        return;
    }
    let known: std::collections::HashSet<String> = client
        .list_run_mirrors()
        .ok()
        .into_iter()
        .flatten()
        .map(|r| r.run_id)
        .collect();
    for key in parse_in_progress_issue_keys(&list.stdout) {
        let Ok(show) = execute_issue_show(invoker, &key) else {
            continue;
        };
        if show.exit_code != 0 {
            continue;
        }
        let Some(run_id) = parse_active_run_id(&show.stdout) else {
            continue;
        };
        if known.contains(&run_id) {
            continue;
        }
        if let Err(e) = sync_run_mirror(client, invoker, &run_id, Some(key.as_str())) {
            append_log(
                log_path,
                &format!("mirror discover {key} run={run_id}: {e}"),
            );
        } else {
            append_log(log_path, &format!("mirror discovered {key} run={run_id}"));
        }
    }
}

pub fn parse_in_progress_issue_keys(stdout: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(stdout.trim()) else {
        return Vec::new();
    };
    let count = v
        .get("count")
        .and_then(|c| {
            c.as_str()
                .and_then(|s| s.parse::<usize>().ok())
                .or_else(|| c.as_u64().map(|n| n as usize))
        })
        .unwrap_or(0);
    let mut keys = Vec::new();
    for i in 0..count {
        let status = v
            .get(format!("issue_{i}_status"))
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if status != "in_progress" {
            continue;
        }
        if let Some(key) = v
            .get(format!("issue_{i}_key"))
            .and_then(|k| k.as_str())
            .filter(|k| !k.is_empty())
        {
            keys.push(key.to_string());
        }
    }
    keys
}

pub fn parse_active_run_id(stdout: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .and_then(|v| {
            v.get("active_run_id")
                .and_then(|x| x.as_str())
                .map(str::to_string)
        })
        .filter(|s| !s.is_empty())
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
                Ok(()) => {
                    append_log(log_path, "heartbeat ok");
                    reconcile_run_mirrors(invoker, client, log_path);
                }
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
            Ok(None) => match client.claim_bootstrap() {
                Ok(Some(task)) => {
                    append_log(
                        log_path,
                        &format!("claimed bootstrap {} session={}", task.id, task.session_id),
                    );
                    if let Err(e) =
                        crate::chat_intake::handle_bootstrap(invoker, client, &task, log_path)
                    {
                        append_log(log_path, &format!("bootstrap error: {e}"));
                    }
                }
                Ok(None) => match client.claim_chat_turn() {
                    Ok(Some(turn)) => {
                        append_log(
                            log_path,
                            &format!("claimed chat turn {} session={}", turn.id, turn.session_id),
                        );
                        let log = |msg: &str| append_log(log_path, msg);
                        if let Err(e) =
                            crate::chat_intake::handle_chat_turn(invoker, client, &turn, log)
                        {
                            append_log(log_path, &format!("chat turn error: {e}"));
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
                                if let Err(e) = sync_run_mirror(
                                    client,
                                    invoker,
                                    run_id,
                                    Some(task.issue_key.as_str()),
                                ) {
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
                                                        invoker, client, &run_id, issue_key,
                                                        log_path,
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
                                    Err(e) => {
                                        append_log(log_path, &format!("issue start error: {e}"))
                                    }
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(e) => append_log(log_path, &format!("claim error: {e}")),
                    },
                    Err(e) => append_log(log_path, &format!("chat turn claim error: {e}")),
                },
                Err(e) => append_log(log_path, &format!("bootstrap claim error: {e}")),
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
    let issue_key = resolve_issue_key_for_run(invoker, client, run_id);
    if issue_key == "UNKNOWN" {
        append_log(
            log_path,
            "orchestrator: issue_key missing on run mirror for resume",
        );
    }
    run_orchestrator_for_task(invoker, client, run_id, &issue_key, log_path);
}

pub(crate) fn append_log(path: Option<&Path>, line: &str) {
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

    #[test]
    fn issue_key_from_pipeline_status_json() {
        let stdout = r#"{"run_id":"run-1","issue_key":"PROJ-93","run_status":"completed"}"#;
        assert_eq!(
            issue_key_from_status_json(stdout).as_deref(),
            Some("PROJ-93")
        );
    }

    #[test]
    fn parse_in_progress_issue_keys_reads_flat_json() {
        let stdout = r#"{"count":"3","issue_0_key":"PROJ-1","issue_0_status":"done","issue_1_key":"PROJ-80","issue_1_status":"in_progress","issue_2_key":"PROJ-96","issue_2_status":"in_progress"}"#;
        assert_eq!(
            parse_in_progress_issue_keys(stdout),
            vec!["PROJ-80".to_string(), "PROJ-96".to_string()]
        );
    }

    #[test]
    fn parse_active_run_id_reads_issue_show_json() {
        let stdout =
            r#"{"key":"PROJ-80","active_run_id":"0000005b-0000-405b-8005-5b00000000005b"}"#;
        assert_eq!(
            parse_active_run_id(stdout).as_deref(),
            Some("0000005b-0000-405b-8005-5b00000000005b")
        );
    }
}
