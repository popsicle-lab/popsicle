//! `popsicle daemon` subcommands (ADR-001 thin shell).

use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use agent_daemon::{run_poll_loop, PopsicleInvoker, RuntimeStatus, ServerClient};

use crate::agent_runtime_config::{
    effective_runtime_id, load_agent_runtime_config, server_client_from_config,
};

const STATUS_REL: &str = "daemon/status.json";
const LOG_REL: &str = "daemon/logs.txt";
const PID_REL: &str = "daemon/pid";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonCommand {
    Start { foreground: bool, background: bool },
    Stop,
    Status,
    Logs,
}

pub fn parse_daemon_args(args: &[String]) -> Result<DaemonCommand, String> {
    match args.first().map(String::as_str) {
        None => Err("usage: popsicle daemon start|stop|status|logs".into()),
        Some("start") => Ok(DaemonCommand::Start {
            foreground: args.iter().any(|a| a == "--foreground"),
            background: args.iter().any(|a| a == "--background"),
        }),
        Some("stop") => Ok(DaemonCommand::Stop),
        Some("status") => Ok(DaemonCommand::Status),
        Some("logs") => Ok(DaemonCommand::Logs),
        Some(other) => Err(format!("unknown daemon subcommand: {other}")),
    }
}

fn daemon_home_paths() -> Result<(PathBuf, PathBuf), crate::CliError> {
    let home = crate::global_config::global_home().map_err(|e| {
        crate::CliError::actionable("io", "daemon", e.to_string(), "resolve global home")
    })?;
    Ok((home.join(STATUS_REL), home.join(LOG_REL)))
}

fn pid_path() -> Result<PathBuf, crate::CliError> {
    Ok(crate::global_config::global_home()
        .map_err(|e| {
            crate::CliError::actionable("io", "daemon", e.to_string(), "resolve global home")
        })?
        .join(PID_REL))
}

pub fn server_client_for_daemon() -> Option<ServerClient> {
    if let Some(client) = ServerClient::from_env() {
        return Some(client);
    }
    let cfg = load_agent_runtime_config().ok()?;
    server_client_from_config(&cfg).ok()
}

fn read_daemon_pid() -> Option<u32> {
    let raw = fs::read_to_string(pid_path().ok()?).ok()?;
    raw.trim().parse().ok()
}

fn write_daemon_pid(pid: u32) -> Result<(), crate::CliError> {
    let path = pid_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            crate::CliError::actionable("io", "daemon-start", e.to_string(), "create daemon dir")
        })?;
    }
    fs::write(&path, pid.to_string()).map_err(|e| {
        crate::CliError::actionable("io", "daemon-start", e.to_string(), "write daemon pid")
    })
}

fn clear_daemon_pid() -> Result<(), crate::CliError> {
    let path = pid_path()?;
    if path.is_file() {
        fs::remove_file(&path).map_err(|e| {
            crate::CliError::actionable("io", "daemon-stop", e.to_string(), "remove daemon pid")
        })?;
    }
    Ok(())
}

pub fn daemon_poll_running() -> bool {
    read_daemon_pid().map(process_alive).unwrap_or(false)
}

fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn terminate_daemon_pid(pid: u32) {
    let _ = Command::new("kill")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    for _ in 0..20 {
        if !process_alive(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    let _ = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn resolve_binary(workspace_root: &Path) -> PathBuf {
    let expected = workspace_root.join("target/debug/popsicle");
    if expected.is_file() {
        return expected;
    }
    std::env::current_exe().unwrap_or(expected)
}

pub fn daemon_status(workspace_root: &Path) -> Result<crate::CommandResponse, crate::CliError> {
    let poll_running = daemon_poll_running();
    let pid = read_daemon_pid();
    let (status_path, log_path) = daemon_home_paths()?;
    let mut fields = if status_path.is_file() {
        let status = RuntimeStatus::read(&status_path).map_err(|e| {
            crate::CliError::actionable("io", "daemon-status", e.to_string(), "daemon status")
        })?;
        let mut m = std::collections::BTreeMap::new();
        m.insert("online".into(), status.online.to_string());
        m.insert("workspace".into(), status.workspace.display().to_string());
        m.insert("detected_clis".into(), status.detected_clis.join(","));
        if let Some(err) = status.last_error {
            m.insert("last_error".into(), err);
        }
        m
    } else {
        let workspace = workspace_root.to_path_buf();
        let binary = resolve_binary(&workspace);
        let invoker = PopsicleInvoker::new(binary, &workspace);
        let status = RuntimeStatus::probe(&workspace, &invoker);
        let mut m = std::collections::BTreeMap::new();
        m.insert("online".into(), status.online.to_string());
        m.insert("workspace".into(), workspace.display().to_string());
        m.insert("detected_clis".into(), status.detected_clis.join(","));
        if !poll_running {
            m.insert("note".into(), "ephemeral probe (daemon not started)".into());
        }
        m
    };
    fields.insert("poll_running".into(), poll_running.to_string());
    if let Some(pid) = pid {
        fields.insert("pid".into(), pid.to_string());
    }
    fields.insert("log_path".into(), log_path.display().to_string());
    Ok(crate::CommandResponse {
        status: "ok",
        next_step: Some("popsicle daemon start --background".into()),
        fields,
    })
}

pub fn daemon_start(
    workspace_root: &Path,
    foreground: bool,
) -> Result<crate::CommandResponse, crate::CliError> {
    let workspace = workspace_root.to_path_buf();
    let binary = resolve_binary(&workspace);
    let invoker = PopsicleInvoker::new(&binary, &workspace);
    let status = RuntimeStatus::probe(&workspace, &invoker);
    let (status_path, log_path) = daemon_home_paths()?;
    status.write(&status_path).map_err(|e| {
        crate::CliError::actionable("io", "daemon-start", e.to_string(), "write status")
    })?;
    let server_client = server_client_for_daemon();
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("online".into(), status.online.to_string());
    fields.insert("status_path".into(), status_path.display().to_string());
    fields.insert("detected_clis".into(), status.detected_clis.join(","));
    fields.insert(
        "foreground".into(),
        if foreground { "true" } else { "false" }.into(),
    );
    if let Some(client) = &server_client {
        fields.insert("runtime_id".into(), client.runtime_id().into());
        if let Ok(url) = env::var("AGENT_RUNTIME_SERVER_URL") {
            fields.insert("server_url".into(), url);
        } else if let Ok(cfg) = load_agent_runtime_config() {
            if let Some(url) = cfg.server_url {
                fields.insert("server_url".into(), url);
            }
        }
        if let Err(e) = client.heartbeat() {
            fields.insert("heartbeat".into(), format!("failed: {e}"));
        } else {
            fields.insert("heartbeat".into(), "ok".into());
        }
    }
    if foreground {
        if let Some(client) = server_client {
            fields.insert("poll".into(), "started".into());
            run_poll_loop(&invoker, &client, Some(&log_path));
        } else {
            fields.insert(
                "poll".into(),
                "skipped (configure AGENT_RUNTIME_SERVER_URL or ~/.popsicle/agent-runtime.json)"
                    .into(),
            );
        }
    }
    Ok(crate::CommandResponse {
        status: if status.online { "ok" } else { "degraded" },
        next_step: Some("popsicle daemon status".into()),
        fields,
    })
}

pub fn daemon_start_background(
    workspace_root: &Path,
) -> Result<crate::CommandResponse, crate::CliError> {
    if daemon_poll_running() {
        let pid = read_daemon_pid().unwrap_or(0);
        return Err(crate::CliError::actionable(
            "daemon",
            "already-running",
            format!("daemon 已在运行 (pid {pid})"),
            "popsicle daemon status",
        ));
    }
    clear_daemon_pid()?;

    let cfg = load_agent_runtime_config().map_err(|e| {
        crate::CliError::actionable(
            "config",
            "daemon-start",
            e.to_string(),
            "configure ~/.popsicle/agent-runtime.json",
        )
    })?;
    let client = server_client_for_daemon().ok_or_else(|| {
        crate::CliError::actionable(
            "config",
            "daemon-start",
            "未配置 Agent Runtime Server URL".to_string(),
            "在 Settings 保存 server_url 或设置 AGENT_RUNTIME_SERVER_URL",
        )
    })?;

    let workspace = workspace_root.to_path_buf();
    let binary = resolve_binary(&workspace);
    let invoker = PopsicleInvoker::new(&binary, &workspace);
    let status = RuntimeStatus::probe(&workspace, &invoker);
    let (status_path, log_path) = daemon_home_paths()?;
    status.write(&status_path).map_err(|e| {
        crate::CliError::actionable("io", "daemon-start", e.to_string(), "write status")
    })?;

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| {
            crate::CliError::actionable("io", "daemon-start", e.to_string(), "open daemon log")
        })?;

    let runtime_id = effective_runtime_id(&cfg);
    let server_url = cfg
        .server_url
        .clone()
        .or_else(|| env::var("AGENT_RUNTIME_SERVER_URL").ok())
        .ok_or_else(|| {
            crate::CliError::actionable(
                "config",
                "daemon-start",
                "未配置 Agent Runtime Server URL".to_string(),
                "在 Settings 保存 server_url",
            )
        })?;

    let mut cmd = Command::new(&binary);
    cmd.current_dir(&workspace)
        .args(["daemon", "start", "--foreground"])
        .env("AGENT_RUNTIME_SERVER_URL", &server_url)
        .env("AGENT_RUNTIME_ID", &runtime_id)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file.try_clone().map_err(|e| {
            crate::CliError::actionable("io", "daemon-start", e.to_string(), "")
        })?))
        .stderr(Stdio::from(log_file));

    let child = cmd.spawn().map_err(|e| {
        crate::CliError::actionable(
            "io",
            "daemon-start",
            e.to_string(),
            "popsicle daemon start --foreground",
        )
    })?;
    let pid = child.id();
    write_daemon_pid(pid)?;

    let mut fields = std::collections::BTreeMap::new();
    fields.insert("pid".into(), pid.to_string());
    fields.insert("poll_running".into(), "true".into());
    fields.insert("workspace".into(), workspace.display().to_string());
    fields.insert("server_url".into(), server_url);
    fields.insert("runtime_id".into(), client.runtime_id().into());
    fields.insert("log_path".into(), log_path.display().to_string());
    fields.insert("online".into(), status.online.to_string());
    fields.insert("detected_clis".into(), status.detected_clis.join(","));

    Ok(crate::CommandResponse {
        status: "ok",
        next_step: Some("popsicle daemon status".into()),
        fields,
    })
}

pub fn daemon_stop() -> Result<crate::CommandResponse, crate::CliError> {
    let mut stopped = false;
    if let Some(pid) = read_daemon_pid() {
        if process_alive(pid) {
            terminate_daemon_pid(pid);
            stopped = true;
        }
        clear_daemon_pid()?;
    }
    let (status_path, _) = daemon_home_paths()?;
    if status_path.is_file() {
        fs::remove_file(&status_path).map_err(|e| {
            crate::CliError::actionable("io", "daemon-stop", e.to_string(), "remove status")
        })?;
    }
    Ok(crate::CommandResponse {
        status: "ok",
        next_step: Some("popsicle daemon start --background".into()),
        fields: std::collections::BTreeMap::from([
            ("stopped".into(), "true".into()),
            ("poll_running".into(), "false".into()),
            ("was_running".into(), stopped.to_string()),
        ]),
    })
}

pub fn daemon_logs() -> Result<crate::CommandResponse, crate::CliError> {
    let (_, log_path) = daemon_home_paths()?;
    let tail = if log_path.is_file() {
        fs::read_to_string(&log_path).unwrap_or_default()
    } else {
        String::new()
    };
    let lines: Vec<&str> = tail.lines().collect();
    let recent: String = lines
        .iter()
        .rev()
        .take(20)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    Ok(crate::CommandResponse {
        status: "ok",
        next_step: Some("popsicle daemon start --background".into()),
        fields: std::collections::BTreeMap::from([
            ("log_path".into(), log_path.display().to_string()),
            ("tail".into(), recent),
        ]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_start_background_flag() {
        let args = vec!["start".into(), "--background".into(), "--foreground".into()];
        assert_eq!(
            parse_daemon_args(&args).unwrap(),
            DaemonCommand::Start {
                foreground: true,
                background: true,
            }
        );
    }
}
