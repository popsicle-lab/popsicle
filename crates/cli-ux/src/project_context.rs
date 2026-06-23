//! Repository-level engineering profile at `docs/PROJECT_CONTEXT.md` (ADR-026).

use std::fs;
use std::path::{Path, PathBuf};

use storage::WorkspaceError;

pub const PROJECT_CONTEXT_REL: &str = "docs/PROJECT_CONTEXT.md";
pub const DEFAULT_INJECTION_MAX_BYTES: usize = 4096;

pub fn project_context_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(PROJECT_CONTEXT_REL)
}

pub fn load_project_context(workspace_root: &Path) -> Result<String, WorkspaceError> {
    let path = project_context_path(workspace_root);
    if !path.is_file() {
        return Ok(String::new());
    }
    fs::read_to_string(&path)
        .map_err(|e| WorkspaceError::Io(format!("read {}: {e}", path.display())))
}

pub fn save_project_context(workspace_root: &Path, content: &str) -> Result<(), WorkspaceError> {
    let path = project_context_path(workspace_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| WorkspaceError::Io(format!("mkdir {}: {e}", parent.display())))?;
    }
    fs::write(&path, content)
        .map_err(|e| WorkspaceError::Io(format!("write {}: {e}", path.display())))
}

/// Truncate for agent injection; prefer content before `## 现在状态` when present.
pub fn project_context_for_injection(workspace_root: &Path, max_bytes: usize) -> String {
    let raw = load_project_context(workspace_root).unwrap_or_default();
    if raw.trim().is_empty() {
        return String::new();
    }
    let section = extract_engineering_profile(&raw);
    truncate_utf8(&section, max_bytes)
}

fn extract_engineering_profile(content: &str) -> String {
    let marker = "## 现在状态";
    if let Some(idx) = content.find(marker) {
        content[..idx].trim().to_string()
    } else {
        content.trim().to_string()
    }
}

fn truncate_utf8(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n\n[…truncated for agent context…]", &s[..end])
}

pub fn project_context_injection_block(workspace_root: &Path, max_bytes: usize) -> String {
    let body = project_context_for_injection(workspace_root, max_bytes);
    if body.is_empty() {
        return String::new();
    }
    format!("\n\n[Project context]\n{body}")
}
