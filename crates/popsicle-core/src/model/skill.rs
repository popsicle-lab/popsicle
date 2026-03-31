use std::collections::HashMap;
use std::fmt;
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

    /// How documents produced by this skill accumulate within a Spec.
    #[serde(default)]
    pub doc_lifecycle: DocLifecycle,

    #[serde(skip)]
    pub source_dir: PathBuf,
    /// Writing guide loaded from guide.md (not part of skill.yaml).
    #[serde(skip)]
    pub guide: Option<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// How documents produced by this skill accumulate within a Spec.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocLifecycle {
    /// One document per spec — new pipeline runs update the existing doc (default).
    #[default]
    Singleton,
    /// Each pipeline run creates a new document — history accumulates (e.g. ADRs).
    Cumulative,
}

impl std::fmt::Display for DocLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Singleton => write!(f, "singleton"),
            Self::Cumulative => write!(f, "cumulative"),
        }
    }
}

/// How important an upstream input is to the current Skill.
/// Controls injection position (attention-aware ordering) and content extraction.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Relevance {
    Low,
    Medium,
    #[default]
    High,
}

impl std::fmt::Display for Relevance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// Declares a dependency on another Skill's artifact output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub from_skill: String,
    pub artifact_type: String,
    #[serde(default = "default_true")]
    pub required: bool,
    /// Importance level — controls injection position and content extraction.
    #[serde(default)]
    pub relevance: Relevance,
    /// When set, only these H2 sections are injected (used with `medium` relevance).
    #[serde(default)]
    pub sections: Option<Vec<String>>,
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
    /// Entities to auto-extract when the owning stage completes.
    #[serde(default)]
    pub extractions: Vec<ExtractionSpec>,
}

/// Declarative extraction specification.
///
/// YAML shorthand format:
/// - `user-stories`
/// - `test-cases:unit`   (with test_type parameter)
/// - `bugs`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractionSpec {
    UserStories,
    TestCases { test_type: String },
    Bugs,
}

impl ExtractionSpec {
    /// Parse from a shorthand string like "user-stories", "test-cases:unit", "bugs".
    pub fn parse(s: &str) -> Result<Self, String> {
        if let Some(rest) = s.strip_prefix("test-cases:") {
            let tt = rest.trim();
            if tt.is_empty() {
                return Err("test-cases requires a type (e.g. test-cases:unit)".to_string());
            }
            Ok(Self::TestCases {
                test_type: tt.to_string(),
            })
        } else {
            match s {
                "user-stories" => Ok(Self::UserStories),
                "test-cases" => {
                    Err("test-cases requires a type parameter (e.g. test-cases:unit)".to_string())
                }
                "bugs" => Ok(Self::Bugs),
                other => Err(format!("unknown extraction type: {other}")),
            }
        }
    }
}

impl fmt::Display for ExtractionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserStories => write!(f, "user-stories"),
            Self::TestCases { test_type } => write!(f, "test-cases:{test_type}"),
            Self::Bugs => write!(f, "bugs"),
        }
    }
}

impl Serialize for ExtractionSpec {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ExtractionSpec {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
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
    #[serde(default)]
    pub requires_approval: bool,
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

    /// Returns true if this skill produces cumulative documents (like ADRs).
    pub fn is_cumulative(&self) -> bool {
        self.doc_lifecycle == DocLifecycle::Cumulative
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
        self.workflow.states.get(state).is_some_and(|s| s.r#final)
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
        assert_eq!(
            skill.try_transition("draft", "submit").unwrap(),
            "discussion"
        );
        assert_eq!(
            skill.try_transition("discussion", "approve").unwrap(),
            "approved"
        );
        assert_eq!(
            skill.try_transition("discussion", "revise").unwrap(),
            "draft"
        );
        assert!(skill.try_transition("draft", "approve").is_err());
    }

    #[test]
    fn test_available_actions() {
        let skill: SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        let actions = skill.available_actions("discussion");
        assert_eq!(actions.len(), 2);
        assert!(skill.available_actions("approved").is_empty());
    }

    #[test]
    fn test_requires_approval_defaults_to_false() {
        let skill: SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        let actions = skill.available_actions("discussion");
        for t in &actions {
            assert!(!t.requires_approval);
        }
    }

    #[test]
    fn test_requires_approval_parsed_from_yaml() {
        let yaml = r#"
name: test-skill
description: Test
version: "0.1.0"
artifacts:
  - type: doc
    template: templates/doc.md
    file_pattern: "{slug}.doc.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: review
          action: submit
    review:
      transitions:
        - to: approved
          action: approve
          requires_approval: true
        - to: draft
          action: revise
    approved:
      final: true
"#;
        let skill: SkillDef = serde_yaml_ng::from_str(yaml).unwrap();
        let actions = skill.available_actions("review");
        let approve = actions.iter().find(|t| t.action == "approve").unwrap();
        assert!(approve.requires_approval);
        let revise = actions.iter().find(|t| t.action == "revise").unwrap();
        assert!(!revise.requires_approval);
    }

    #[test]
    fn test_relevance_defaults_to_high() {
        let skill: SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        assert_eq!(skill.inputs[0].relevance, Relevance::High);
        assert!(skill.inputs[0].sections.is_none());
    }

    #[test]
    fn test_relevance_and_sections_parsed() {
        let yaml = r#"
name: test-skill
description: Test
version: "0.1.0"
inputs:
  - from_skill: upstream-a
    artifact_type: doc-a
    required: true
    relevance: low
  - from_skill: upstream-b
    artifact_type: doc-b
    required: true
    relevance: medium
    sections:
      - Problem Statement
      - Goals
  - from_skill: upstream-c
    artifact_type: doc-c
    required: false
artifacts:
  - type: doc
    template: templates/doc.md
    file_pattern: "{slug}.doc.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: done
          action: finish
    done:
      final: true
"#;
        let skill: SkillDef = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(skill.inputs.len(), 3);

        assert_eq!(skill.inputs[0].relevance, Relevance::Low);
        assert!(skill.inputs[0].sections.is_none());

        assert_eq!(skill.inputs[1].relevance, Relevance::Medium);
        let sections = skill.inputs[1].sections.as_ref().unwrap();
        assert_eq!(sections, &["Problem Statement", "Goals"]);

        assert_eq!(skill.inputs[2].relevance, Relevance::High);
        assert!(skill.inputs[2].sections.is_none());
    }

    #[test]
    fn test_relevance_ordering() {
        assert!(Relevance::Low < Relevance::Medium);
        assert!(Relevance::Medium < Relevance::High);
    }

    #[test]
    fn test_relevance_display() {
        assert_eq!(Relevance::Low.to_string(), "low");
        assert_eq!(Relevance::Medium.to_string(), "medium");
        assert_eq!(Relevance::High.to_string(), "high");
    }

    #[test]
    fn test_doc_lifecycle_default_is_singleton() {
        let yaml = r#"
name: test-skill
description: Test
version: "0.1.0"
artifacts:
  - type: test-doc
    template: t.md
    file_pattern: "{slug}.test.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: done
          action: finish
    done:
      final: true
"#;
        let skill: SkillDef = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(skill.doc_lifecycle, DocLifecycle::Singleton);
        assert!(!skill.is_cumulative());
    }

    #[test]
    fn test_doc_lifecycle_cumulative() {
        let yaml = r#"
name: adr-writer
description: Architecture decisions
version: "0.1.0"
doc_lifecycle: cumulative
artifacts:
  - type: adr
    template: t.md
    file_pattern: "{slug}.adr.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: done
          action: finish
    done:
      final: true
"#;
        let skill: SkillDef = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(skill.doc_lifecycle, DocLifecycle::Cumulative);
        assert!(skill.is_cumulative());
    }

    #[test]
    fn test_extraction_spec_parse() {
        assert_eq!(
            ExtractionSpec::parse("user-stories").unwrap(),
            ExtractionSpec::UserStories
        );
        assert_eq!(ExtractionSpec::parse("bugs").unwrap(), ExtractionSpec::Bugs);
        assert_eq!(
            ExtractionSpec::parse("test-cases:unit").unwrap(),
            ExtractionSpec::TestCases {
                test_type: "unit".to_string()
            }
        );
        assert_eq!(
            ExtractionSpec::parse("test-cases:e2e").unwrap(),
            ExtractionSpec::TestCases {
                test_type: "e2e".to_string()
            }
        );
        assert!(ExtractionSpec::parse("test-cases").is_err());
        assert!(ExtractionSpec::parse("test-cases:").is_err());
        assert!(ExtractionSpec::parse("unknown").is_err());
    }

    #[test]
    fn test_extraction_spec_display() {
        assert_eq!(ExtractionSpec::UserStories.to_string(), "user-stories");
        assert_eq!(ExtractionSpec::Bugs.to_string(), "bugs");
        assert_eq!(
            ExtractionSpec::TestCases {
                test_type: "unit".to_string()
            }
            .to_string(),
            "test-cases:unit"
        );
    }

    #[test]
    fn test_extraction_spec_serde_roundtrip() {
        let specs = vec![
            ExtractionSpec::UserStories,
            ExtractionSpec::TestCases {
                test_type: "api".to_string(),
            },
            ExtractionSpec::Bugs,
        ];
        let json = serde_json::to_string(&specs).unwrap();
        let parsed: Vec<ExtractionSpec> = serde_json::from_str(&json).unwrap();
        assert_eq!(specs, parsed);
    }

    #[test]
    fn test_artifact_with_extractions_yaml() {
        let yaml = r#"
name: prd-writer
description: Product requirements
version: "0.1.0"
artifacts:
  - type: prd
    template: templates/prd.md
    file_pattern: "{slug}.prd.md"
    extractions:
      - user-stories
      - test-cases:unit
      - bugs
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: done
          action: finish
    done:
      final: true
"#;
        let skill: SkillDef = serde_yaml_ng::from_str(yaml).unwrap();
        let artifact = &skill.artifacts[0];
        assert_eq!(artifact.extractions.len(), 3);
        assert_eq!(artifact.extractions[0], ExtractionSpec::UserStories);
        assert_eq!(
            artifact.extractions[1],
            ExtractionSpec::TestCases {
                test_type: "unit".to_string()
            }
        );
        assert_eq!(artifact.extractions[2], ExtractionSpec::Bugs);
    }

    #[test]
    fn test_artifact_without_extractions_defaults_empty() {
        let skill: SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        assert!(skill.artifacts[0].extractions.is_empty());
    }
}
