//! cursor-agent subprocess adapter (ADR-001 §5).
//!
//! Resolves the agent CLI even when GUI apps lack shell PATH (macOS Tauri):
//! checks `~/.local/bin/cursor-agent`, then falls back to `cursor agent …`.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::pipeline_skill::skill_from_status_json;
use crate::prompt::load_skill_prompt;
use crate::PopsicleInvoker;

const DEFAULT_TIMEOUT_SECS: u64 = 600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInvokeResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub dry_run: bool,
}

/// How to invoke the Cursor agent CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CursorAgentTool {
    /// Standalone `cursor-agent` binary.
    Standalone(PathBuf),
    /// Cursor shell shim: `cursor agent …` (Cursor IDE install).
    ViaCursorShell(PathBuf),
}

impl CursorAgentTool {
    fn detect() -> Option<Self> {
        if let Some(path) = resolve_standalone_binary() {
            return Some(Self::Standalone(path));
        }
        if let Some(path) = resolve_cursor_shell_binary() {
            return Some(Self::ViaCursorShell(path));
        }
        None
    }

    fn display_label(&self) -> String {
        match self {
            Self::Standalone(path) => path.display().to_string(),
            Self::ViaCursorShell(path) => format!("{} agent", path.display()),
        }
    }

    fn run_status(&self) -> io::Result<(i32, String, String)> {
        for sub in ["status", "whoami"] {
            let output = self.command().arg(sub).output()?;
            let code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if code == 0 || !stdout.is_empty() {
                return Ok((code, stdout, stderr));
            }
        }
        Ok((
            -1,
            String::new(),
            "cursor agent status/whoami failed".into(),
        ))
    }

    fn spawn_login(&self) -> io::Result<()> {
        // Detached spawn: GUI apps have no TTY; login opens the system browser.
        self.command().arg("login").spawn()?;
        Ok(())
    }

    fn spawn_prompt(&self, workspace: &Path, prompt: &str) -> io::Result<std::process::Child> {
        self.command()
            .arg("-p")
            .arg("--output-format")
            .arg("text")
            .arg(prompt)
            .current_dir(workspace)
            .spawn()
    }

    fn command(&self) -> Command {
        match self {
            Self::Standalone(path) => Command::new(path),
            Self::ViaCursorShell(path) => {
                let mut cmd = Command::new(path);
                cmd.arg("agent");
                cmd
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CursorAgentAdapter {
    tool: CursorAgentTool,
    workspace: PathBuf,
}

impl CursorAgentAdapter {
    pub fn detect() -> Option<Self> {
        let tool = CursorAgentTool::detect()?;
        let workspace = std::env::current_dir().ok()?;
        Some(Self { tool, workspace })
    }

    pub fn new(binary: impl Into<PathBuf>, workspace: impl Into<PathBuf>) -> Self {
        Self {
            tool: CursorAgentTool::Standalone(binary.into()),
            workspace: workspace.into(),
        }
    }

    pub fn binary(&self) -> &Path {
        match &self.tool {
            CursorAgentTool::Standalone(path) => path,
            CursorAgentTool::ViaCursorShell(path) => path,
        }
    }

    pub fn invoke_prompt(&self, prompt: &str) -> io::Result<AgentInvokeResult> {
        if dry_run_enabled() {
            return Ok(AgentInvokeResult {
                exit_code: 0,
                stdout: format!("[dry-run] cursor-agent prompt len={}", prompt.len()),
                stderr: String::new(),
                dry_run: true,
            });
        }
        let timeout = Duration::from_secs(
            std::env::var("AGENT_RUNTIME_AGENT_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_TIMEOUT_SECS),
        );
        let child = self.tool.spawn_prompt(&self.workspace, prompt)?;
        let output = wait_timeout::wait_timeout(child, timeout)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "cursor-agent timed out"))?;
        Ok(AgentInvokeResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            dry_run: false,
        })
    }

    pub fn maybe_run_for_run(
        &self,
        invoker: &PopsicleInvoker,
        run_id: &str,
    ) -> io::Result<Option<AgentInvokeResult>> {
        if !auto_agent_enabled() {
            return Ok(None);
        }
        let status = invoker.run(["pipeline", "status", "--run", run_id, "--format", "json"])?;
        if status.exit_code != 0 {
            return Err(io::Error::other(format!(
                "pipeline status exit {}",
                status.exit_code
            )));
        }
        let Some(skill) = skill_from_status_json(invoker.workspace(), &status.stdout)? else {
            return Ok(None);
        };
        let prompt = load_skill_prompt(invoker.workspace(), &skill, run_id)?;
        self.invoke_prompt(&prompt).map(Some)
    }
}

pub fn auto_agent_enabled() -> bool {
    !matches!(
        std::env::var("AGENT_RUNTIME_AUTO_AGENT").as_deref(),
        Ok("0") | Ok("false") | Ok("off")
    )
}

fn dry_run_enabled() -> bool {
    matches!(
        std::env::var("AGENT_RUNTIME_AGENT_DRY_RUN").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

/// Primary binary path for display (standalone or `cursor` shim).
pub fn detect_cursor_agent_binary() -> Option<PathBuf> {
    CursorAgentTool::detect().map(|tool| match tool {
        CursorAgentTool::Standalone(path) => path,
        CursorAgentTool::ViaCursorShell(path) => path,
    })
}

pub fn cursor_agent_display_label() -> Option<String> {
    CursorAgentTool::detect().map(|t| t.display_label())
}

/// Run agent auth status (works from GUI apps without shell PATH).
pub fn run_cursor_agent_status() -> io::Result<(i32, String, String)> {
    let tool = CursorAgentTool::detect().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "cursor-agent / cursor agent not found (check ~/.local/bin or Cursor shell command)",
        )
    })?;
    tool.run_status()
}

/// Spawn agent login (opens browser when available).
pub fn spawn_cursor_agent_login() -> io::Result<()> {
    let tool = CursorAgentTool::detect().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "cursor-agent / cursor agent not found (check ~/.local/bin or Cursor shell command)",
        )
    })?;
    tool.spawn_login()
}

fn resolve_standalone_binary() -> Option<PathBuf> {
    for candidate in standalone_candidates() {
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    which_with_augmented_path("cursor-agent")
}

fn resolve_cursor_shell_binary() -> Option<PathBuf> {
    for candidate in cursor_shell_candidates() {
        if is_executable_file(&candidate) && cursor_shell_has_agent(&candidate) {
            return Some(candidate);
        }
    }
    which_with_augmented_path("cursor").filter(|path| cursor_shell_has_agent(path))
}

fn standalone_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        out.push(PathBuf::from(home).join(".local/bin/cursor-agent"));
    }
    out.extend([
        PathBuf::from("/usr/local/bin/cursor-agent"),
        PathBuf::from("/opt/homebrew/bin/cursor-agent"),
    ]);
    out
}

fn cursor_shell_candidates() -> Vec<PathBuf> {
    let mut out = vec![PathBuf::from(
        "/Applications/Cursor.app/Contents/Resources/app/bin/cursor",
    )];
    if let Ok(home) = std::env::var("HOME") {
        out.push(PathBuf::from(home).join(".local/bin/cursor"));
    }
    out.extend([
        PathBuf::from("/usr/local/bin/cursor"),
        PathBuf::from("/opt/homebrew/bin/cursor"),
    ]);
    out
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn cursor_shell_has_agent(cursor: &Path) -> bool {
    Command::new(cursor)
        .args(["agent", "--help"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn augmented_path() -> String {
    let mut dirs = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(format!("{home}/.local/bin"));
    }
    dirs.extend([
        "/usr/local/bin".into(),
        "/opt/homebrew/bin".into(),
        "/Applications/Cursor.app/Contents/Resources/app/bin".into(),
    ]);
    if let Ok(existing) = std::env::var("PATH") {
        dirs.push(existing);
    }
    dirs.join(":")
}

fn which_with_augmented_path(name: &str) -> Option<PathBuf> {
    let output = Command::new("which")
        .arg(name)
        .env("PATH", augmented_path())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

mod wait_timeout {
    use std::io;
    use std::process::{Child, Output};
    use std::time::{Duration, Instant};

    pub fn wait_timeout(mut child: Child, limit: Duration) -> io::Result<Option<Output>> {
        let start = Instant::now();
        loop {
            match child.try_wait()? {
                Some(_) => return Ok(Some(child.wait_with_output()?)),
                None if start.elapsed() >= limit => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(None);
                }
                None => std::thread::sleep(Duration::from_millis(200)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_skips_subprocess() {
        std::env::set_var("AGENT_RUNTIME_AGENT_DRY_RUN", "1");
        let adapter = CursorAgentAdapter::new("cursor-agent", "/tmp");
        let result = adapter.invoke_prompt("hello").expect("invoke");
        assert!(result.dry_run);
        assert!(result.stdout.contains("dry-run"));
        std::env::remove_var("AGENT_RUNTIME_AGENT_DRY_RUN");
    }

    #[test]
    fn auto_agent_defaults_on() {
        std::env::remove_var("AGENT_RUNTIME_AUTO_AGENT");
        assert!(auto_agent_enabled());
    }

    #[test]
    fn augmented_path_includes_local_bin() {
        std::env::set_var("HOME", "/tmp/testhome");
        let path = augmented_path();
        assert!(path.contains("/tmp/testhome/.local/bin"));
        std::env::remove_var("HOME");
    }

    #[test]
    fn standalone_candidates_include_home_local_bin() {
        std::env::set_var("HOME", "/tmp/testhome");
        let c = standalone_candidates();
        assert!(c.iter().any(|p| p.ends_with(".local/bin/cursor-agent")));
        std::env::remove_var("HOME");
    }
}
