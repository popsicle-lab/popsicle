use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A Skill definition loaded from skill.yaml.
/// Skills are first-class citizens: each carries its own sub-workflow,
/// document templates, input/output definitions, and AI prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub inputs: Vec<SkillInput>,
    pub artifacts: Vec<ArtifactDef>,
    pub workflow: WorkflowDef,
    #[serde(default)]
    pub prompts: HashMap<String, String>,
    #[serde(default)]
    pub hooks: HooksDef,

    #[serde(skip)]
    pub source_dir: PathBuf,
    /// Writing guide loaded from guide.md (not part of skill.yaml).
    #[serde(skip)]
    pub guide: Option<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Declares a dependency on another Skill's artifact output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub from_skill: String,
    pub artifact_type: String,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// Defines what type of document this Skill produces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDef {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub template: PathBuf,
    pub file_pattern: String,
}

/// State-machine workflow definition within a Skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub initial: String,
    pub states: HashMap<String, StateDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDef {
    #[serde(default)]
    pub transitions: Vec<TransitionDef>,
    #[serde(default)]
    pub r#final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDef {
    pub to: String,
    pub action: String,
    pub guard: Option<String>,
}

/// Extension points: hooks triggered at various lifecycle events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksDef {
    pub on_enter: Option<String>,
    pub on_artifact_created: Option<String>,
    pub on_complete: Option<String>,
}

impl SkillDef {
    /// Load a Skill definition from a skill.yaml file.
    pub fn load(path: &std::path::Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut skill: SkillDef = serde_yaml_ng::from_str(&content).map_err(|e| {
            crate::error::PopsicleError::InvalidSkillDef(format!("{}: {}", path.display(), e))
        })?;
        let dir = path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        let guide_path = dir.join("guide.md");
        if guide_path.exists() {
            skill.guide = std::fs::read_to_string(&guide_path).ok();
        }

        skill.source_dir = dir;
        Ok(skill)
    }

    /// Get the template file path (resolved relative to skill source directory).
    pub fn template_path(&self, artifact_type: &str) -> Option<PathBuf> {
        self.artifacts
            .iter()
            .find(|a| a.artifact_type == artifact_type)
            .map(|a| self.source_dir.join(&a.template))
    }

    /// Get available actions from a given state.
    pub fn available_actions(&self, state: &str) -> Vec<&TransitionDef> {
        self.workflow
            .states
            .get(state)
            .map(|s| s.transitions.iter().collect())
            .unwrap_or_default()
    }

    /// Check if a state is final.
    pub fn is_final_state(&self, state: &str) -> bool {
        self.workflow
            .states
            .get(state)
            .is_some_and(|s| s.r#final)
    }

    /// Attempt a transition: returns the target state if valid.
    pub fn try_transition(&self, from: &str, action: &str) -> crate::error::Result<String> {
        let state = self.workflow.states.get(from).ok_or_else(|| {
            crate::error::PopsicleError::InvalidTransition {
                from: from.to_string(),
                action: action.to_string(),
            }
        })?;

        state
            .transitions
            .iter()
            .find(|t| t.action == action)
            .map(|t| t.to.clone())
            .ok_or_else(|| crate::error::PopsicleError::InvalidTransition {
                from: from.to_string(),
                action: action.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skill_yaml() -> &'static str {
        r#"
name: product-prd
description: Product requirements document
version: "0.1.0"
inputs:
  - from_skill: domain-analysis
    artifact_type: domain-model
    required: true
artifacts:
  - type: prd
    template: templates/prd.md
    file_pattern: "{slug}.prd.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: discussion
          action: submit
    discussion:
      transitions:
        - to: approved
          action: approve
        - to: draft
          action: revise
    approved:
      final: true
prompts:
  draft: "Write a PRD based on the domain model..."
"#
    }

    #[test]
    fn test_parse_skill_yaml() {
        let skill: SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        assert_eq!(skill.name, "product-prd");
        assert_eq!(skill.inputs.len(), 1);
        assert_eq!(skill.artifacts[0].artifact_type, "prd");
        assert_eq!(skill.workflow.initial, "draft");
        assert!(skill.is_final_state("approved"));
        assert!(!skill.is_final_state("draft"));
    }

    #[test]
    fn test_transitions() {
        let skill: SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        assert_eq!(skill.try_transition("draft", "submit").unwrap(), "discussion");
        assert_eq!(skill.try_transition("discussion", "approve").unwrap(), "approved");
        assert_eq!(skill.try_transition("discussion", "revise").unwrap(), "draft");
        assert!(skill.try_transition("draft", "approve").is_err());
    }

    #[test]
    fn test_available_actions() {
        let skill: SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        let actions = skill.available_actions("discussion");
        assert_eq!(actions.len(), 2);
        assert!(skill.available_actions("approved").is_empty());
    }
}
