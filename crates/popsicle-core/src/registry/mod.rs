pub mod index;
mod loader;
pub mod package;

pub use index::{RegistryIndex, ResolvedPackage, SearchResult, is_registry_name};
pub use loader::{PipelineLoader, SkillLoader, ToolLoader};
pub use package::{
    PackageDep, PackageEntry, PackageType, PackageVersion, RegistryConfig, index_path,
};

use std::collections::HashMap;

use crate::error::{PopsicleError, Result};
use crate::model::{SkillDef, ToolDef};

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

/// Central registry that discovers, loads, and provides access to Tools.
///
/// Tools are action-oriented skills (command/prompt executors) that can be
/// sourced from external repositories and installed independently of a module.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolDef>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool definition. Later registrations overwrite earlier ones
    /// (matching the 3-layer priority loading pattern).
    pub fn register(&mut self, tool: ToolDef) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Result<&ToolDef> {
        self.tools
            .get(name)
            .ok_or_else(|| PopsicleError::SkillNotFound(format!("Tool '{}' not found", name)))
    }

    /// List all registered tools, sorted by name.
    pub fn list(&self) -> Vec<&ToolDef> {
        let mut tools: Vec<_> = self.tools.values().collect();
        tools.sort_by_key(|t| &t.name);
        tools
    }

    /// Check if a tool is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}
