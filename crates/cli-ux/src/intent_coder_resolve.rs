//! Unified intent-coder path resolution for skills and pipelines (ADR-017).
//!
//! Dogfood workspaces with live `intent-coder/module.yaml` at the repo root read
//! skills and pipelines from that tree first; otherwise fall back to the installed
//! module under `.popsicle/modules/intent-coder/`. Legacy `.popsicle/pipelines/`
//! remains as a workspace install/override layer between live root and module.

use std::fs;
use std::path::{Path, PathBuf};

use storage::WorkspaceError;

use crate::intent_coder_bundle::{embedded_pipeline_content, embedded_pipeline_names};
use crate::pipeline_taxonomy::{canonical_pipeline_name, deprecated_aliases_for};

const LEGACY_PIPELINES_REL: &str = ".popsicle/pipelines";
const MODULE_REL: &str = ".popsicle/modules/intent-coder";
const LIVE_REL: &str = "intent-coder";

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}

/// Live intent-coder checkout at workspace root (dogfood monorepo).
pub fn has_live_intent_coder_root(root: &Path) -> bool {
    root.join(LIVE_REL).join("module.yaml").is_file()
}

pub fn legacy_pipelines_dir(root: &Path) -> PathBuf {
    root.join(LEGACY_PIPELINES_REL)
}

pub fn intent_coder_module_dir(root: &Path) -> PathBuf {
    root.join(MODULE_REL)
}

/// Skills directory: live root when present, else installed module.
pub fn intent_coder_skills_dir(root: &Path) -> PathBuf {
    if has_live_intent_coder_root(root) {
        let live = root.join(LIVE_REL).join("skills");
        if live.is_dir() {
            return live;
        }
    }
    intent_coder_module_dir(root).join("skills")
}

/// Pipeline YAML search order (first existing file wins).
pub fn pipeline_search_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if has_live_intent_coder_root(root) {
        let live = root.join(LIVE_REL).join("pipelines");
        if live.is_dir() {
            dirs.push(live);
        }
    }
    let legacy = legacy_pipelines_dir(root);
    if legacy.is_dir() {
        dirs.push(legacy);
    }
    let module = intent_coder_module_dir(root).join("pipelines");
    if module.is_dir() {
        dirs.push(module);
    }
    dirs
}

fn pipeline_stem_from_entry(name: &str) -> Option<&str> {
    name.strip_suffix(".pipeline.yaml")
}

fn collect_pipeline_names_in_dir(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(stem) = file_name.to_str().and_then(pipeline_stem_from_entry) else {
            continue;
        };
        out.push(stem.to_string());
    }
}

/// Names of pipelines embedded in the binary (compile-time intent-coder snapshot).
pub fn bundled_pipeline_names() -> Vec<String> {
    embedded_pipeline_names()
}

/// All pipeline template names visible in this workspace.
pub fn list_pipeline_template_names(root: &Path) -> Vec<String> {
    let mut names = bundled_pipeline_names();
    for dir in pipeline_search_dirs(root) {
        collect_pipeline_names_in_dir(&dir, &mut names);
    }
    names = names
        .into_iter()
        .map(|n| canonical_pipeline_name(&n).to_string())
        .collect();
    names.sort();
    names.dedup();
    names
}

pub fn find_pipeline_path(root: &Path, name: &str) -> Option<PathBuf> {
    let canonical = canonical_pipeline_name(name);
    let mut candidates = vec![canonical, name];
    for alias in deprecated_aliases_for(canonical) {
        if !candidates.contains(&alias) {
            candidates.push(alias);
        }
    }
    for candidate in candidates {
        for dir in pipeline_search_dirs(root) {
            let path = dir.join(format!("{candidate}.pipeline.yaml"));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

/// Write a missing bundled pipeline into `.popsicle/pipelines/` (legacy install path).
pub fn self_heal_pipeline(root: &Path, name: &str) -> Result<PathBuf, WorkspaceError> {
    let canonical = canonical_pipeline_name(name);
    let content = embedded_pipeline_content(canonical).ok_or_else(|| {
        WorkspaceError::NotFound(format!(
            "pipeline {name} (available: {})",
            bundled_pipeline_names().join(", ")
        ))
    })?;
    let dir = legacy_pipelines_dir(root);
    fs::create_dir_all(&dir).map_err(io_err)?;
    let path = dir.join(format!("{canonical}.pipeline.yaml"));
    fs::write(&path, content).map_err(io_err)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "popsicle-ic-resolve-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn live_root_precedes_legacy_pipelines_for_load() {
        let root = temp_root("live-first");
        fs::create_dir_all(root.join("intent-coder/pipelines")).unwrap();
        fs::create_dir_all(root.join(".popsicle/pipelines")).unwrap();
        fs::write(
            root.join("intent-coder/module.yaml"),
            "name: intent-coder\nversion: \"0.0.0\"\n",
        )
        .unwrap();
        fs::write(
            root.join("intent-coder/pipelines/demo.pipeline.yaml"),
            "name: demo\nstages: []\n",
        )
        .unwrap();
        fs::write(
            root.join(".popsicle/pipelines/demo.pipeline.yaml"),
            "name: stale\nstages: []\n",
        )
        .unwrap();

        let path = find_pipeline_path(&root, "demo").unwrap();
        assert!(path.starts_with(root.join("intent-coder/pipelines")));
    }

    #[test]
    fn skills_dir_uses_live_root_in_dogfood() {
        let root = temp_root("skills-live");
        fs::create_dir_all(root.join("intent-coder/skills/foo")).unwrap();
        fs::create_dir_all(root.join(".popsicle/modules/intent-coder/skills/bar")).unwrap();
        fs::write(
            root.join("intent-coder/module.yaml"),
            "name: intent-coder\nversion: \"0.0.0\"\n",
        )
        .unwrap();

        assert_eq!(
            intent_coder_skills_dir(&root),
            root.join("intent-coder/skills")
        );
    }
}
