use std::path::Path;

use crate::error::Result;
use crate::model::{PipelineDef, SkillDef, ToolDef};
use crate::registry::{SkillRegistry, ToolRegistry};

/// Loads Skill definitions from the filesystem.
pub struct SkillLoader;

impl SkillLoader {
    /// Scan a directory for subdirectories containing skill.yaml files.
    pub fn load_dir(dir: &Path, registry: &mut SkillRegistry) -> Result<usize> {
        if !dir.is_dir() {
            return Ok(0);
        }

        let mut count = 0;
        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join("skill.yaml");
                if skill_file.exists() {
                    let skill = SkillDef::load(&skill_file)?;
                    registry.register(skill);
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

/// Loads Pipeline definitions from the filesystem.
pub struct PipelineLoader;

impl PipelineLoader {
    /// Scan a directory for *.pipeline.yaml files.
    pub fn load_dir(dir: &Path) -> Result<Vec<PipelineDef>> {
        let mut pipelines = Vec::new();
        if !dir.is_dir() {
            return Ok(pipelines);
        }

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".pipeline.yaml"))
            {
                let pipeline = PipelineDef::load(&path)?;
                pipelines.push(pipeline);
            }
        }

        pipelines.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(pipelines)
    }
}

/// Loads Tool definitions from the filesystem.
pub struct ToolLoader;

impl ToolLoader {
    /// Scan a directory for subdirectories containing tool.yaml files.
    pub fn load_dir(dir: &Path, registry: &mut ToolRegistry) -> Result<usize> {
        if !dir.is_dir() {
            return Ok(0);
        }

        let mut count = 0;
        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let tool_file = path.join("tool.yaml");
                if tool_file.exists() {
                    let tool = ToolDef::load(&tool_file)?;
                    registry.register(tool);
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}
