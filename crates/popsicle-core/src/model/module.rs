use std::path::Path;

use serde::{Deserialize, Serialize};

/// A Module definition loaded from module.yaml.
/// Modules are the distribution unit: a self-contained package of Skills + Pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDef {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

impl ModuleDef {
    /// Load a Module definition from a module.yaml file.
    pub fn load(path: &Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let def: ModuleDef = serde_yaml_ng::from_str(&content).map_err(|e| {
            crate::error::PopsicleError::InvalidSkillDef(format!(
                "Invalid module.yaml {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(def)
    }
}
