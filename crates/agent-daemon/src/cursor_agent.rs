//! cursor-agent subprocess adapter (ADR-001 §5).
//!
//! Resolves the agent CLI even when GUI apps lack shell PATH (macOS Tauri):
//! checks `~/.local/bin/cursor-agent`, then falls back to `cursor agent …`.
//!
//! Default output: `--output-format stream-json` with per-event run-log lines.
//! Set `AGENT_RUNTIME_AGENT_OUTPUT_FORMAT=text` to restore legacy batch text mode.

use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::pipeline_skill::skill_from_status_json;
use crate::prompt::load_skill_prompt;
use crate::stream_json::{format_stream_event, stream_partial_enabled};
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
        let mut cmd = self.command();
        cmd.arg("-p").arg("--trust");
        if agent_output_format() == AgentOutputFormat::Text {
            cmd.arg("--output-format").arg("text");
        } else {
            cmd.arg("--output-format").arg("stream-json");
            if stream_partial_enabled() {
                cmd.arg("--stream-partial-output");
            }
        }
        cmd.arg(prompt)
            .current_dir(workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentOutputFormat {
    StreamJson,
    Text,
}

fn agent_output_format() -> AgentOutputFormat {
    match std::env::var("AGENT_RUNTIME_AGENT_OUTPUT_FORMAT")
        .ok()
        .as_deref()
    {
        Some("text") => AgentOutputFormat::Text,
        _ => AgentOutputFormat::StreamJson,
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

    pub fn for_workspace(workspace: impl Into<PathBuf>) -> Option<Self> {
        let tool = CursorAgentTool::detect()?;
        Some(Self {
            tool,
            workspace: workspace.into(),
        })
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
        self.invoke_prompt_with_log(prompt, |_| {})
    }

    pub fn invoke_prompt_with_log(
        &self,
        prompt: &str,
        mut log: impl FnMut(&str),
    ) -> io::Result<AgentInvokeResult> {
        if dry_run_enabled() {
            log("[dry-run] cursor-agent prompt skipped");
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
        if agent_output_format() == AgentOutputFormat::Text {
            return invoke_text_batch(child, timeout, &mut log);
        }
        invoke_stream_json(child, timeout, &mut log)
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

fn invoke_text_batch(
    child: std::process::Child,
    timeout: Duration,
    log: &mut impl FnMut(&str),
) -> io::Result<AgentInvokeResult> {
    let output = wait_timeout::wait_timeout(child, timeout)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "cursor-agent timed out"))?;
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    log(&format!("cursor-agent: exit={exit_code} dry_run=false"));
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
    Ok(AgentInvokeResult {
        exit_code,
        stdout,
        stderr,
        dry_run: false,
    })
}

fn invoke_stream_json(
    mut child: std::process::Child,
    timeout: Duration,
    log: &mut impl FnMut(&str),
) -> io::Result<AgentInvokeResult> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("cursor-agent stdout pipe missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("cursor-agent stderr pipe missing"))?;

    let (stdout_tx, stdout_rx) = mpsc::channel::<io::Result<String>>();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if stdout_tx.send(line).is_err() {
                break;
            }
        }
    });

    let (stderr_tx, stderr_rx) = mpsc::channel::<io::Result<String>>();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if stderr_tx.send(line).is_err() {
                break;
            }
        }
    });

    let mut stdout_acc = String::new();
    let mut stderr_acc = String::new();
    let mut partial_buf = String::new();
    let start = std::time::Instant::now();

    log("cursor-agent: stream-json started");

    loop {
        while let Ok(Ok(line)) = stdout_rx.try_recv() {
            stdout_acc.push_str(&line);
            stdout_acc.push('\n');
            if let Some(msg) = format_stream_event(&line) {
                if stream_partial_enabled() && msg.starts_with('›') {
                    partial_buf.push_str(msg.trim_start_matches('›').trim_start());
                    if partial_buf.contains('\n') || partial_buf.chars().count() >= 240 {
                        flush_partial_buffer(log, &mut partial_buf);
                    }
                } else {
                    flush_partial_buffer(log, &mut partial_buf);
                    log(&msg);
                }
            }
        }
        while let Ok(Ok(line)) = stderr_rx.try_recv() {
            stderr_acc.push_str(&line);
            stderr_acc.push('\n');
            let t = line.trim();
            if !t.is_empty() {
                log(&format!("✗ {t}"));
            }
        }

        match child.try_wait()? {
            Some(status) => {
                flush_partial_buffer(log, &mut partial_buf);
                // Drain remaining pipe lines.
                while let Ok(Ok(line)) = stdout_rx.try_recv() {
                    stdout_acc.push_str(&line);
                    stdout_acc.push('\n');
                    if let Some(msg) = format_stream_event(&line) {
                        log(&msg);
                    }
                }
                while let Ok(Ok(line)) = stderr_rx.try_recv() {
                    stderr_acc.push_str(&line);
                    stderr_acc.push('\n');
                    let t = line.trim();
                    if !t.is_empty() {
                        log(&format!("✗ {t}"));
                    }
                }
                let exit_code = status.code().unwrap_or(-1);
                log(&format!("cursor-agent: exit={exit_code} dry_run=false"));
                return Ok(AgentInvokeResult {
                    exit_code,
                    stdout: stdout_acc,
                    stderr: stderr_acc,
                    dry_run: false,
                });
            }
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "cursor-agent timed out",
                ));
            }
            None => thread::sleep(Duration::from_millis(50)),
        }
    }
}

fn flush_partial_buffer(log: &mut impl FnMut(&str), buf: &mut String) {
    let t = buf.trim();
    if t.is_empty() {
        buf.clear();
        return;
    }
    let line = if t.chars().count() > 500 {
        format!("› {}…", t.chars().take(500).collect::<String>())
    } else {
        format!("› {t}")
    };
    log(&line);
    buf.clear();
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
        let mut lines = Vec::new();
        let result = adapter
            .invoke_prompt_with_log("hello", |l| lines.push(l.to_string()))
            .expect("invoke");
        assert!(result.dry_run);
        assert!(result.stdout.contains("dry-run"));
        assert!(lines.iter().any(|l| l.contains("dry-run")));
        std::env::remove_var("AGENT_RUNTIME_AGENT_DRY_RUN");
    }

    #[test]
    fn auto_agent_defaults_on() {
        std::env::remove_var("AGENT_RUNTIME_AUTO_AGENT");
        assert!(auto_agent_enabled());
    }

    #[test]
    fn default_output_format_is_stream_json() {
        std::env::remove_var("AGENT_RUNTIME_AGENT_OUTPUT_FORMAT");
        assert_eq!(agent_output_format(), AgentOutputFormat::StreamJson);
    }

    #[test]
    fn text_output_format_env() {
        std::env::set_var("AGENT_RUNTIME_AGENT_OUTPUT_FORMAT", "text");
        assert_eq!(agent_output_format(), AgentOutputFormat::Text);
        std::env::remove_var("AGENT_RUNTIME_AGENT_OUTPUT_FORMAT");
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
