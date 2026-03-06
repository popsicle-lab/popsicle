mod loader;

pub use loader::{PipelineLoader, SkillLoader};

use std::collections::HashMap;

use crate::error::{PopsicleError, Result};
use crate::model::SkillDef;

/// Central registry that discovers, loads, and provides access to Skills.
#[derive(Debug, Default)]
pub struct SkillRegistry {
    skills: HashMap<String, SkillDef>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a skill definition.
    pub fn register(&mut self, skill: SkillDef) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Get a skill by name.
    pub fn get(&self, name: &str) -> Result<&SkillDef> {
        self.skills
            .get(name)
            .ok_or_else(|| PopsicleError::SkillNotFound(name.to_string()))
    }

    /// List all registered skills.
    pub fn list(&self) -> Vec<&SkillDef> {
        let mut skills: Vec<_> = self.skills.values().collect();
        skills.sort_by_key(|s| &s.name);
        skills
    }

    /// Check if a skill is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.skills.contains_key(name)
    }

    /// Number of registered skills.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}
