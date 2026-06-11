//! In-memory skill / pipeline registries.

use std::collections::BTreeMap;
use std::path::Path;

use crate::loader::{load_pipelines_dir, load_skills_dir, LoadError, LoadedSkill, PipelineDef};

/// In-memory skill catalog keyed by skill name.
#[derive(Debug, Default)]
pub struct SkillRegistry {
    skills: BTreeMap<String, LoadedSkill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, skill: LoadedSkill) {
        let name = skill.load_result.name.clone();
        self.skills.insert(name, skill);
    }

    pub fn get(&self, name: &str) -> Option<&LoadedSkill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&LoadedSkill> {
        self.skills.values().collect()
    }

    /// Load every `skill.yaml` under `dir`.
    pub fn load_dir(&mut self, dir: &Path) -> Result<usize, LoadError> {
        let mut loaded = Vec::new();
        let count = load_skills_dir(dir, &mut loaded)?;
        for skill in loaded {
            self.register(skill);
        }
        Ok(count)
    }
}

/// In-memory pipeline catalog keyed by pipeline name.
#[derive(Debug, Default)]
pub struct PipelineRegistry {
    pipelines: BTreeMap<String, PipelineDef>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, pipeline: PipelineDef) {
        let name = pipeline.name.clone();
        self.pipelines.insert(name, pipeline);
    }

    pub fn get(&self, name: &str) -> Option<&PipelineDef> {
        self.pipelines.get(name)
    }

    pub fn load_dir(&mut self, dir: &Path) -> Result<usize, LoadError> {
        let pipelines = load_pipelines_dir(dir)?;
        let count = pipelines.len();
        for p in pipelines {
            p.validate()?;
            self.register(p);
        }
        Ok(count)
    }
}
