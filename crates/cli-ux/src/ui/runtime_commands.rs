//! Tauri IPC for agent-runtime UI (PROJ-90 / T-AR-0001, T-AR-0002).

use agent_daemon::{
    cursor_agent_display_label, detect_cursor_agent_binary, run_cursor_agent_status,
    spawn_cursor_agent_login, ServerClient,
};
use storage::WorkspaceStore;
use tauri::State;

use crate::agent_runtime_config::{
    effective_runtime_id, load_agent_runtime_config, save_agent_runtime_config, AgentRuntimeConfig,
};
use crate::daemon::{daemon_start_background, daemon_status, daemon_stop};

use super::dto::*;
use super::state::AppState;

fn workspace_root(state: &State<AppState>) -> Result<std::path::PathBuf, String> {
    state
        .project_dir
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "未选择工作区".to_string())
}

fn client_for_config(cfg: &AgentRuntimeConfig) -> Result<ServerClient, String> {
    crate::agent_runtime_config::server_client_from_config(cfg)
}

#[tauri::command]
pub fn get_agent_runtime_config() -> Result<AgentRuntimeConfigDto, String> {
    let cfg = load_agent_runtime_config().map_err(|e| e.to_string())?;
    let path = crate::agent_runtime_config::agent_runtime_config_path()
        .map_err(|e| e.to_string())?
        .display()
        .to_string();
    Ok(AgentRuntimeConfigDto {
        server_url: cfg.server_url.clone().unwrap_or_default(),
        runtime_id: effective_runtime_id(&cfg),
        config_path: path,
        cursor_agent_installed: detect_cursor_agent_binary().is_some(),
    })
}

#[derive(serde::Deserialize)]
pub struct SaveAgentRuntimeConfigInput {
    pub server_url: String,
    pub runtime_id: String,
}

#[tauri::command]
pub fn save_agent_runtime_config_cmd(
    input: SaveAgentRuntimeConfigInput,
) -> Result<AgentRuntimeConfigDto, String> {
    let server_url = input.server_url.trim();
    let runtime_id = input.runtime_id.trim();
    let cfg = AgentRuntimeConfig {
        server_url: if server_url.is_empty() {
            None
        } else {
            Some(server_url.to_string())
        },
        runtime_id: if runtime_id.is_empty() {
            None
        } else {
            Some(runtime_id.to_string())
        },
    };
    save_agent_runtime_config(&cfg).map_err(|e| e.to_string())?;
    get_agent_runtime_config()
}

#[tauri::command]
pub fn cursor_agent_status_cmd() -> Result<CursorAgentStatusDto, String> {
    let binary = cursor_agent_display_label().unwrap_or_default();
    if binary.is_empty() {
        return Ok(CursorAgentStatusDto {
            installed: false,
            binary_path: String::new(),
            logged_in: false,
            output: String::new(),
            error: Some(
                "未找到 cursor-agent 或 cursor agent。桌面 App 不继承终端 PATH；\
                 请确认 ~/.local/bin/cursor-agent 存在，或已安装 Cursor 的 shell 命令（cursor agent login）。"
                    .into(),
            ),
        });
    }
    match run_cursor_agent_status() {
        Ok((code, stdout, stderr)) => {
            let output = if stdout.is_empty() {
                stderr.clone()
            } else {
                stdout
            };
            let logged_in = code == 0
                && !output.to_lowercase().contains("not logged in")
                && !output.to_lowercase().contains("not authenticated");
            Ok(CursorAgentStatusDto {
                installed: true,
                binary_path: binary,
                logged_in,
                output,
                error: if code == 0 {
                    None
                } else {
                    Some(format!("exit {code}"))
                },
            })
        }
        Err(e) => Ok(CursorAgentStatusDto {
            installed: true,
            binary_path: binary,
            logged_in: false,
            output: String::new(),
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub fn cursor_agent_login_cmd() -> Result<String, String> {
    spawn_cursor_agent_login().map_err(|e| e.to_string())?;
    Ok("已在终端启动 cursor-agent login（将打开浏览器完成登录）".into())
}

#[tauri::command]
pub fn daemon_status_cmd(state: State<AppState>) -> Result<DaemonStatusDto, String> {
    let root = workspace_root(&state)?;
    let resp = daemon_status(&root).map_err(|e| e.to_string())?;
    let online = resp
        .fields
        .get("online")
        .map(|s| s == "true")
        .unwrap_or(false);
    let note = resp.fields.get("note").cloned();
    let last_error = resp.fields.get("last_error").cloned();
    let detected_clis = resp
        .fields
        .get("detected_clis")
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let poll_running = resp
        .fields
        .get("poll_running")
        .map(|s| s == "true")
        .unwrap_or(false);
    let pid = resp.fields.get("pid").and_then(|s| s.parse().ok());
    let log_path = resp.fields.get("log_path").cloned().unwrap_or_default();
    Ok(DaemonStatusDto {
        online,
        poll_running,
        pid,
        workspace: resp
            .fields
            .get("workspace")
            .cloned()
            .unwrap_or_else(|| root.display().to_string()),
        detected_clis,
        note,
        last_error,
        log_path,
        foreground_hint: "popsicle daemon start --background".into(),
    })
}

#[tauri::command]
pub fn daemon_start_cmd(state: State<AppState>) -> Result<DaemonControlResultDto, String> {
    let root = workspace_root(&state)?;
    let cfg = load_agent_runtime_config().map_err(|e| e.to_string())?;
    if cfg.server_url.as_deref().unwrap_or("").trim().is_empty()
        && std::env::var("AGENT_RUNTIME_SERVER_URL").is_err()
    {
        return Err("请先在 Settings 保存 Server URL".into());
    }
    let resp = daemon_start_background(&root).map_err(|e| e.to_string())?;
    let pid = resp.fields.get("pid").and_then(|s| s.parse().ok());
    Ok(DaemonControlResultDto {
        poll_running: true,
        pid,
        message: format!(
            "Daemon 已启动{}",
            pid.map(|p| format!(" (pid {p})")).unwrap_or_default()
        ),
    })
}

#[tauri::command]
pub fn daemon_stop_cmd() -> Result<DaemonControlResultDto, String> {
    let resp = daemon_stop().map_err(|e| e.to_string())?;
    let was_running = resp
        .fields
        .get("was_running")
        .map(|s| s == "true")
        .unwrap_or(false);
    Ok(DaemonControlResultDto {
        poll_running: false,
        pid: None,
        message: if was_running {
            "Daemon 已停止".into()
        } else {
            "Daemon 未在运行".into()
        },
    })
}

#[tauri::command]
pub fn agent_runtime_server_status() -> Result<AgentRuntimeServerStatusDto, String> {
    let cfg = load_agent_runtime_config().map_err(|e| e.to_string())?;
    let client = client_for_config(&cfg)?;
    let health = client
        .server_health()
        .map_err(|e| format!("无法连接 Server：{e}"))?;
    let runtime = client.runtime_state().ok();
    Ok(AgentRuntimeServerStatusDto {
        server_url: cfg.server_url.clone().unwrap_or_default(),
        runtime_id: effective_runtime_id(&cfg),
        server_ok: health.status == "ok",
        storage: health.storage,
        runtime_state: runtime.map(|r| r.state).unwrap_or_else(|| "offline".into()),
    })
}

#[tauri::command]
pub fn dispatch_issue_remote(
    issue_key: String,
    state: State<AppState>,
) -> Result<DispatchIssueResultDto, String> {
    let root = workspace_root(&state)?;
    let cfg = load_agent_runtime_config().map_err(|e| e.to_string())?;
    let client = client_for_config(&cfg)?;

    let (pipeline, has_active) = state.with_store(|store| {
        let row = store.get_issue(&issue_key).map_err(|e| e.to_string())?;
        let active = store
            .active_run_id(&issue_key)
            .map_err(|e| e.to_string())?
            .is_some();
        let pipeline = row
            .pipeline
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "Issue 未配置 pipeline，无法派活".to_string())?;
        Ok::<(String, bool), String>((pipeline, active))
    })?;

    if has_active {
        return Err("该 Issue 已有 active run，请先完成或取消后再派活".into());
    }

    let workspace_id = root.display().to_string();
    let resp = client
        .dispatch_issue(&workspace_id, &issue_key, &pipeline)
        .map_err(|e| format!("派活请求失败：{e}"))?;

    Ok(DispatchIssueResultDto {
        accepted: resp.accepted,
        state: resp.state,
        reason: resp.reason,
        task_id: resp
            .task
            .as_ref()
            .and_then(|t| t.get("id"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    })
}

#[tauri::command]
pub fn list_remote_runs() -> Result<Vec<RemoteRunMirrorDto>, String> {
    let cfg = load_agent_runtime_config().map_err(|e| e.to_string())?;
    let client = client_for_config(&cfg)?;
    let runs = client
        .list_run_mirrors()
        .map_err(|e| format!("获取远程 run 列表失败：{e}"))?;
    Ok(runs
        .into_iter()
        .map(|r| RemoteRunMirrorDto {
            run_id: r.run_id,
            issue_key: r.issue_key,
            pipeline: r.pipeline,
            run_status: r.run_status,
            current_stage: r.current_stage,
        })
        .collect())
}
