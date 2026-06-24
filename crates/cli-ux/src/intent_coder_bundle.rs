//! intent-coder module embedded at compile time (ADR-017).
//!
//! Workspace-root `intent-coder/` overrides the bundle when present (dogfood).
//! Release/DMG installs extract this tree into `.popsicle/modules/intent-coder/`.

use std::path::Path;

use include_dir::{include_dir, Dir, DirEntry};

use storage::WorkspaceError;

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}

/// Compiled from `intent-coder/` at build time (`crates/cli-ux/../../intent-coder`).
pub static EMBEDDED_INTENT_CODER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../intent-coder");

/// Pipeline template names from the compile-time `intent-coder/pipelines/` tree.
pub fn embedded_pipeline_names() -> Vec<String> {
    let Some(pipelines) = EMBEDDED_INTENT_CODER.get_dir("pipelines") else {
        return Vec::new();
    };
    pipelines
        .entries()
        .iter()
        .filter_map(|entry| {
            let DirEntry::File(file) = entry else {
                return None;
            };
            file.path()
                .file_name()?
                .to_str()?
                .strip_suffix(".pipeline.yaml")
                .map(str::to_string)
        })
        .collect()
}

/// Raw YAML bytes for a bundled pipeline template.
pub fn embedded_pipeline_content(name: &str) -> Option<&'static [u8]> {
    let rel = format!("pipelines/{name}.pipeline.yaml");
    EMBEDDED_INTENT_CODER
        .get_file(rel.as_str())
        .map(|file| file.contents())
}

pub fn embedded_module_version() -> Option<String> {
    let content = EMBEDDED_INTENT_CODER
        .get_file("module.yaml")?
        .contents_utf8()?;
    content
        .lines()
        .find(|line| line.starts_with("version:"))
        .and_then(|line| line.split('"').nth(1).map(str::to_string))
}

pub fn extract_embedded_intent_coder(dest: &Path) -> Result<(), WorkspaceError> {
    if dest.exists() {
        fs_remove_dir_all(dest)?;
    }
    std::fs::create_dir_all(dest).map_err(io_err)?;
    for entry in EMBEDDED_INTENT_CODER.entries() {
        extract_entry(entry, dest)?;
    }
    Ok(())
}

/// `include_dir` paths are relative to the bundle root — always join against `base`.
fn extract_entry(entry: &DirEntry<'_>, base: &Path) -> Result<(), WorkspaceError> {
    match entry {
        DirEntry::Dir(dir) => {
            std::fs::create_dir_all(base.join(dir.path())).map_err(io_err)?;
            for child in dir.entries() {
                extract_entry(child, base)?;
            }
        }
        DirEntry::File(file) => {
            let path = base.join(file.path());
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(io_err)?;
            }
            std::fs::write(path, file.contents()).map_err(io_err)?;
        }
    }
    Ok(())
}

fn fs_remove_dir_all(path: &Path) -> Result<(), WorkspaceError> {
    std::fs::remove_dir_all(path).map_err(io_err)
}
