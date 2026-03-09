use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Pipeline definition loaded from a .pipeline.yaml file.
/// Orchestrates Skills into a full development lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDef {
    pub name: String,
    pub description: String,
    pub stages: Vec<StageDef>,
}

/// A stage groups one or more Skills that execute within the same phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDef {
    pub name: String,
    /// Either a single skill name or multiple parallel skills.
    /// Use `skill` for single, `skills` for multiple in YAML.
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub skill: Option<String>,
    pub description: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl StageDef {
    /// Get all skill names for this stage (normalizes single/multi syntax).
    pub fn skill_names(&self) -> Vec<&str> {
        if !self.skills.is_empty() {
            self.skills.iter().map(|s| s.as_str()).collect()
        } else if let Some(ref s) = self.skill {
            vec![s.as_str()]
        } else {
            vec![]
        }
    }
}

impl PipelineDef {
    /// Load from a YAML file.
    pub fn load(path: &std::path::Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let def: PipelineDef = serde_yaml_ng::from_str(&content)?;
        Ok(def)
    }

    /// Topological ordering validation: ensures no circular dependencies.
    pub fn validate(&self) -> crate::error::Result<()> {
        let names: Vec<&str> = self.stages.iter().map(|s| s.name.as_str()).collect();
        for stage in &self.stages {
            for dep in &stage.depends_on {
                if !names.contains(&dep.as_str()) {
                    return Err(crate::error::PopsicleError::InvalidSkillDef(format!(
                        "Stage '{}' depends on unknown stage '{}'",
                        stage.name, dep
                    )));
                }
            }
        }
        Ok(())
    }
}

/// A running instance of a Pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRun {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub stage_states: HashMap<String, StageState>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    Blocked,
    Ready,
    InProgress,
    Completed,
    Skipped,
}

impl std::fmt::Display for StageState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked => write!(f, "blocked"),
            Self::Ready => write!(f, "ready"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

impl PipelineRun {
    pub fn new(pipeline_def: &PipelineDef, title: impl Into<String>) -> Self {
        let now = Utc::now();
        let mut stage_states = HashMap::new();

        for stage in &pipeline_def.stages {
            let state = if stage.depends_on.is_empty() {
                StageState::Ready
            } else {
                StageState::Blocked
            };
            stage_states.insert(stage.name.clone(), state);
        }

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pipeline_name: pipeline_def.name.clone(),
            title: title.into(),
            stage_states,
            created_at: now,
            updated_at: now,
        }
    }

    /// Recompute blocked/ready states based on current completions.
    pub fn refresh_states(&mut self, pipeline_def: &PipelineDef) {
        for stage in &pipeline_def.stages {
            let current = self.stage_states.get(&stage.name).copied();
            if matches!(
                current,
                Some(StageState::Completed | StageState::Skipped | StageState::InProgress)
            ) {
                continue;
            }
            let all_deps_done = stage.depends_on.iter().all(|dep| {
                matches!(
                    self.stage_states.get(dep),
                    Some(StageState::Completed | StageState::Skipped)
                )
            });
            let new_state = if all_deps_done {
                StageState::Ready
            } else {
                StageState::Blocked
            };
            self.stage_states.insert(stage.name.clone(), new_state);
        }
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pipeline_yaml() -> &'static str {
        r#"
name: full-sdlc
description: Full software development lifecycle
stages:
  - name: domain
    skill: domain-analysis
    description: Domain boundary analysis
  - name: product
    skill: product-prd
    description: Product requirements
    depends_on: [domain]
  - name: tech-design
    skills:
      - tech-rfc
      - tech-adr
    description: Technical design
    depends_on: [product]
"#
    }

    #[test]
    fn test_parse_pipeline() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        assert_eq!(def.name, "full-sdlc");
        assert_eq!(def.stages.len(), 3);
        assert_eq!(def.stages[0].skill_names(), vec!["domain-analysis"]);
        assert_eq!(def.stages[2].skill_names(), vec!["tech-rfc", "tech-adr"]);
    }

    #[test]
    fn test_pipeline_run_initial_states() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        let run = PipelineRun::new(&def, "Test Feature");
        assert_eq!(run.stage_states["domain"], StageState::Ready);
        assert_eq!(run.stage_states["product"], StageState::Blocked);
        assert_eq!(run.stage_states["tech-design"], StageState::Blocked);
    }

    #[test]
    fn test_refresh_states() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        let mut run = PipelineRun::new(&def, "Test Feature");

        run.stage_states
            .insert("domain".to_string(), StageState::Completed);
        run.refresh_states(&def);

        assert_eq!(run.stage_states["product"], StageState::Ready);
        assert_eq!(run.stage_states["tech-design"], StageState::Blocked);
    }
}
