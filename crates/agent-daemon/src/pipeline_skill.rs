//! Resolve skill name from pipeline YAML + pipeline status JSON.

use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PipelineYaml {
    stages: Vec<StageYaml>,
}

#[derive(Debug, Deserialize)]
struct StageYaml {
    name: String,
    skill: Option<String>,
}

fn has_live_intent_coder_root(workspace: &Path) -> bool {
    workspace.join("intent-coder/module.yaml").is_file()
}

fn pipeline_yaml_path(workspace: &Path, pipeline: &str) -> Option<PathBuf> {
    let file = format!("{pipeline}.pipeline.yaml");
    let mut candidates = Vec::new();
    if has_live_intent_coder_root(workspace) {
        candidates.push(workspace.join("intent-coder/pipelines").join(&file));
    }
    candidates.push(workspace.join(".popsicle/pipelines").join(&file));
    candidates.push(
        workspace
            .join(".popsicle/modules/intent-coder/pipelines")
            .join(&file),
    );
    candidates.into_iter().find(|p| p.is_file())
}

pub fn skill_for_stage(
    workspace: &Path,
    pipeline: &str,
    stage: &str,
) -> io::Result<Option<String>> {
    let Some(path) = pipeline_yaml_path(workspace, pipeline) else {
        return Ok(None);
    };
    let raw = std::fs::read_to_string(&path)?;
    let doc: PipelineYaml = serde_yaml::from_str(&raw)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(doc
        .stages
        .into_iter()
        .find(|s| s.name == stage)
        .and_then(|s| s.skill))
}

pub fn skill_from_status_json(workspace: &Path, status_json: &str) -> io::Result<Option<String>> {
    let v: serde_json::Value = serde_json::from_str(status_json.trim())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let pipeline = v
        .get("pipeline")
        .and_then(|p| p.as_str())
        .unwrap_or_default();
    let stage = v
        .get("current_stage")
        .and_then(|s| s.as_str())
        .unwrap_or_default();
    if pipeline.is_empty() || stage.is_empty() {
        return Ok(None);
    }
    skill_for_stage(workspace, pipeline, stage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn resolves_skill_from_feature_delivery_pipeline() {
        let workspace = env::current_dir().expect("cwd");
        let skill = skill_for_stage(&workspace, "feature-delivery", "implement")
            .expect("read")
            .unwrap_or_default();
        if pipeline_yaml_path(&workspace, "feature-delivery").is_some() {
            assert_eq!(skill, "shadow-implementer");
        }
    }
}
