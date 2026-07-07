//! Local execution plane for agent-runtime (ADR-001).
//!
//! Daemon subprocess-invokes the workspace `popsicle` binary; it does not embed
//! skill-runtime pipeline logic.

mod chat_intake;
mod cursor_agent;
mod orchestrator;
mod orchestrator_parse;
mod pipeline_skill;
mod prompt;
mod server_client;
mod stream_json;

pub use cursor_agent::{
    auto_agent_enabled, cursor_agent_display_label, detect_cursor_agent_binary,
    run_cursor_agent_status, spawn_cursor_agent_login, AgentInvokeResult, CursorAgentAdapter,
};
pub use orchestrator::{
    orchestrator_enabled, run_unattended, OrchestratorConfig, OrchestratorOutcome,
};
pub use server_client::{
    execute_doc_check, execute_doc_create, execute_issue_close, execute_issue_start,
    execute_pipeline_next, execute_pipeline_status, execute_stage_complete,
    parse_run_id_from_issue_start, run_poll_loop, sync_run_mirror, ClaimedConfirm, ClaimedTask,
    DispatchResponse, RunMirrorSummary, RuntimeStateResponse, ServerClient, ServerHealthResponse,
};

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

use serde::{Deserialize, Serialize};

/// Result of invoking `popsicle` with JSON output expectations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopsicleInvokeResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Invokes `popsicle` in a workspace (contracts: DaemonSubprocessInvokesPopsicle).
#[derive(Debug, Clone)]
pub struct PopsicleInvoker {
    binary: PathBuf,
    workspace: PathBuf,
}

impl PopsicleInvoker {
    pub fn new(binary: impl Into<PathBuf>, workspace: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            workspace: workspace.into(),
        }
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    pub fn binary(&self) -> &Path {
        &self.binary
    }

    pub fn run<I, S>(&self, args: I) -> io::Result<PopsicleInvokeResult>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new(&self.binary)
            .args(args)
            .current_dir(&self.workspace)
            .output()?;
        Ok(PopsicleInvokeResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    pub fn doctor_status_json(&self) -> io::Result<PopsicleInvokeResult> {
        self.run(["doctor", "--format", "json"])
    }

    /// Subprocess started with exit 0 (contracts#DaemonSubprocessInvokesPopsicle).
    pub fn invokes_popsicle_successfully(&self) -> io::Result<bool> {
        Ok(self.doctor_status_json()?.exit_code == 0)
    }
}

/// Persisted daemon heartbeat + runtime registration (local only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub online: bool,
    pub workspace: PathBuf,
    pub detected_clis: Vec<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

impl RuntimeStatus {
    pub fn detect_clis() -> Vec<String> {
        let mut out = Vec::new();
        if crate::detect_cursor_agent_binary().is_some() {
            out.push("cursor-agent".into());
        }
        for (label, name) in [("claude", "claude"), ("codex", "codex")] {
            if which::which(name).is_ok() {
                out.push(label.into());
            }
        }
        out
    }

    pub fn probe(workspace: &Path, invoker: &PopsicleInvoker) -> Self {
        let detected_clis = Self::detect_clis();
        let doctor = invoker.doctor_status_json().ok();
        let online = doctor.as_ref().is_some_and(|r| r.exit_code == 0);
        let last_error = if online {
            None
        } else {
            Some(
                doctor
                    .map(|r| {
                        if r.stderr.is_empty() {
                            r.stdout
                        } else {
                            r.stderr
                        }
                    })
                    .unwrap_or_else(|| "doctor failed".into()),
            )
        };
        Self {
            online,
            workspace: workspace.to_path_buf(),
            detected_clis,
            last_error,
        }
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)
    }

    pub fn read(path: &Path) -> io::Result<Self> {
        let raw = fs::read_to_string(path)?;
        serde_json::from_str(&raw).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

mod which {
    use std::process::Command;

    pub fn which(name: &str) -> Result<(), ()> {
        Command::new("which")
            .arg(name)
            .status()
            .ok()
            .filter(|s| s.success())
            .map(|_| ())
            .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn daemon_subprocess_invokes_popsicle() {
        let workspace = env::current_dir().expect("cwd");
        let binary = workspace.join("target/debug/popsicle");
        if !binary.is_file() {
            return;
        }
        let invoker = PopsicleInvoker::new(&binary, &workspace);
        assert!(invoker.invokes_popsicle_successfully().unwrap());
    }

    #[test]
    fn invoker_runs_doctor_when_binary_exists() {
        let workspace = env::current_dir().expect("cwd");
        let binary = workspace.join("target/debug/popsicle");
        if !binary.is_file() {
            return;
        }
        let invoker = PopsicleInvoker::new(&binary, &workspace);
        let result = invoker.doctor_status_json().expect("doctor");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("\"status\""));
    }

    #[test]
    fn runtime_status_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("daemon.json");
        let status = RuntimeStatus {
            online: true,
            workspace: dir.path().to_path_buf(),
            detected_clis: vec!["cursor-agent".into()],
            last_error: None,
        };
        status.write(&path).unwrap();
        let read = RuntimeStatus::read(&path).unwrap();
        assert_eq!(status, read);
    }
}
