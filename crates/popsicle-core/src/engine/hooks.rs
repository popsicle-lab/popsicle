use std::collections::HashMap;
use std::process::Command;

use crate::error::Result;
use crate::model::Document;
use crate::model::skill::HooksDef;

/// Hook execution context passed as environment variables to hook commands.
#[derive(Debug)]
pub struct HookContext {
    pub env: HashMap<String, String>,
}

impl HookContext {
    pub fn from_document(doc: &Document, event: &str) -> Self {
        let mut env = HashMap::new();
        env.insert("POPSICLE_EVENT".to_string(), event.to_string());
        env.insert("POPSICLE_DOC_ID".to_string(), doc.id.clone());
        env.insert("POPSICLE_DOC_TYPE".to_string(), doc.doc_type.clone());
        env.insert("POPSICLE_DOC_TITLE".to_string(), doc.title.clone());
        env.insert("POPSICLE_DOC_STATUS".to_string(), doc.status.clone());
        env.insert("POPSICLE_SKILL".to_string(), doc.skill_name.clone());
        env.insert("POPSICLE_RUN_ID".to_string(), doc.pipeline_run_id.clone());
        env.insert(
            "POPSICLE_FILE_PATH".to_string(),
            doc.file_path.display().to_string(),
        );
        Self { env }
    }
}

/// Execute a hook command if defined.
/// Returns the command output or None if no hook was defined.
pub fn run_hook(
    hooks: &HooksDef,
    event: HookEvent,
    ctx: &HookContext,
) -> Result<Option<HookResult>> {
    let cmd_str = match event {
        HookEvent::OnEnter => hooks.on_enter.as_deref(),
        HookEvent::OnArtifactCreated => hooks.on_artifact_created.as_deref(),
        HookEvent::OnComplete => hooks.on_complete.as_deref(),
    };

    let cmd_str = match cmd_str {
        Some(c) if !c.is_empty() => c,
        _ => return Ok(None),
    };

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .envs(&ctx.env)
        .output()
        .map_err(|e| {
            crate::error::PopsicleError::Storage(format!("Hook execution failed: {}", e))
        })?;

    Ok(Some(HookResult {
        event,
        command: cmd_str.to_string(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }))
}

#[derive(Debug, Clone, Copy)]
pub enum HookEvent {
    OnEnter,
    OnArtifactCreated,
    OnComplete,
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnEnter => write!(f, "on_enter"),
            Self::OnArtifactCreated => write!(f, "on_artifact_created"),
            Self::OnComplete => write!(f, "on_complete"),
        }
    }
}

#[derive(Debug)]
pub struct HookResult {
    pub event: HookEvent,
    pub command: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}
