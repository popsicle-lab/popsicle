use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{PopsicleError, Result};

/// A Tool definition loaded from tool.yaml.
///
/// Tools are action-oriented skills that execute a command or AI prompt with
/// named arguments. Unlike pipeline skills (which produce documents through a
/// workflow), tools perform discrete operations such as drawing diagrams,
/// converting files, or generating code snippets.
///
/// Tools can be sourced from external repositories and installed independently
/// of a module's pipeline skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,

    /// Original source reference (e.g. "github:user/repo//tool-dir#v1.0").
    /// Recorded for upgrade purposes; not required for local tools.
    #[serde(default)]
    pub source: Option<String>,

    /// Arguments accepted by this tool.
    #[serde(default)]
    pub args: Vec<ToolArg>,

    /// Shell command template. Supports `{{arg_name}}` substitution.
    /// Mutually exclusive with `prompt`.
    #[serde(default)]
    pub command: Option<String>,

    /// AI prompt template. Supports `{{arg_name}}` substitution.
    /// Used when `command` is absent.
    #[serde(default)]
    pub prompt: Option<String>,

    /// Optional usage guide loaded from guide.md (not part of tool.yaml).
    #[serde(skip)]
    pub guide: Option<String>,

    /// Directory the tool was loaded from (not part of tool.yaml).
    #[serde(skip)]
    pub source_dir: PathBuf,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Defines a named argument accepted by a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArg {
    pub name: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub required: bool,
    /// Default value used when the argument is not provided.
    #[serde(default)]
    pub default: Option<String>,
}

fn default_true() -> bool {
    true
}

impl ToolDef {
    /// Load a ToolDef from a `tool.yaml` file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut def: ToolDef = serde_yaml_ng::from_str(&content).map_err(|e| {
            PopsicleError::InvalidSkillDef(format!(
                "Invalid tool.yaml {}: {}",
                path.display(),
                e
            ))
        })?;

        def.source_dir = path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();

        // Load optional guide.md
        let guide_path = def.source_dir.join("guide.md");
        if guide_path.exists() {
            def.guide = std::fs::read_to_string(&guide_path).ok();
        }

        if def.command.is_none() && def.prompt.is_none() {
            return Err(PopsicleError::InvalidSkillDef(format!(
                "tool.yaml {} must define either 'command' or 'prompt'",
                path.display()
            )));
        }

        Ok(def)
    }

    /// Resolve argument values, applying defaults for optional args.
    /// Returns an error if a required argument is missing.
    pub fn resolve_args(&self, provided: &HashMap<String, String>) -> Result<HashMap<String, String>> {
        let mut resolved = HashMap::new();
        for arg in &self.args {
            if let Some(val) = provided.get(&arg.name) {
                resolved.insert(arg.name.clone(), val.clone());
            } else if let Some(default) = &arg.default {
                resolved.insert(arg.name.clone(), default.clone());
            } else if arg.required {
                return Err(PopsicleError::InvalidSkillDef(format!(
                    "Required argument '{}' not provided for tool '{}'",
                    arg.name, self.name
                )));
            }
        }
        Ok(resolved)
    }

    /// Render the command template with resolved argument values.
    pub fn render_command(&self, args: &HashMap<String, String>) -> Option<String> {
        self.command.as_ref().map(|cmd| render_template(cmd, args))
    }

    /// Render the prompt template with resolved argument values.
    pub fn render_prompt(&self, args: &HashMap<String, String>) -> Option<String> {
        self.prompt.as_ref().map(|p| render_template(p, args))
    }
}

/// Replace `{{key}}` placeholders in a template string with values from `args`.
pub fn render_template(template: &str, args: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in args {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tool_yaml() -> &'static str {
        r#"
name: draw-diagram
description: Generate a Mermaid diagram
version: "1.0.0"
args:
  - name: type
    description: Diagram type
    required: false
    default: flowchart
  - name: input
    description: Input description
    required: true
prompt: |
  Draw a {{type}} diagram for: {{input}}
"#
    }

    #[test]
    fn test_parse_tool_yaml() {
        let def: ToolDef = serde_yaml_ng::from_str(sample_tool_yaml()).unwrap();
        assert_eq!(def.name, "draw-diagram");
        assert_eq!(def.args.len(), 2);
        assert!(def.prompt.is_some());
        assert!(def.command.is_none());
    }

    #[test]
    fn test_resolve_args_with_default() {
        let def: ToolDef = serde_yaml_ng::from_str(sample_tool_yaml()).unwrap();
        let mut provided = HashMap::new();
        provided.insert("input".to_string(), "user login flow".to_string());
        let resolved = def.resolve_args(&provided).unwrap();
        assert_eq!(resolved["type"], "flowchart");
        assert_eq!(resolved["input"], "user login flow");
    }

    #[test]
    fn test_resolve_args_missing_required() {
        let def: ToolDef = serde_yaml_ng::from_str(sample_tool_yaml()).unwrap();
        let provided = HashMap::new(); // missing "input"
        assert!(def.resolve_args(&provided).is_err());
    }

    #[test]
    fn test_render_prompt() {
        let def: ToolDef = serde_yaml_ng::from_str(sample_tool_yaml()).unwrap();
        let mut args = HashMap::new();
        args.insert("type".to_string(), "sequence".to_string());
        args.insert("input".to_string(), "user login".to_string());
        let rendered = def.render_prompt(&args).unwrap();
        assert!(rendered.contains("sequence"));
        assert!(rendered.contains("user login"));
    }

    #[test]
    fn test_render_template() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), "world".to_string());
        assert_eq!(render_template("Hello {{name}}!", &args), "Hello world!");
    }
}
