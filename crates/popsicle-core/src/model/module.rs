use std::path::Path;

use serde::{Deserialize, Serialize};

/// A Module definition loaded from module.yaml.
/// Modules are the distribution unit: a self-contained package of Skills + Pipelines.
/// Modules may also declare external tool dependencies that are fetched and installed
/// from other repositories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDef {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    /// External tool dependencies to install alongside this module.
    #[serde(default)]
    pub tool_dependencies: Vec<ToolDependency>,
}

/// Declares an external tool that should be fetched and installed with the module.
///
/// Example in module.yaml:
/// ```yaml
/// tool_dependencies:
///   - name: draw-diagram
///     source: github:curtiseng/popsicle-tools//draw-diagram#v1.0
///     description: Generate Mermaid diagrams
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependency {
    /// Local name for this tool (used to invoke it via `popsicle tool run <name>`).
    pub name: String,
    /// Source reference: local path or `github:user/repo[#ref][//subdir]`.
    pub source: String,
    #[serde(default)]
    pub description: Option<String>,
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
