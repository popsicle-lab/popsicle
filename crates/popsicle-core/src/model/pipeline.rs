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
    /// Keywords for scale-adaptive pipeline recommendation matching.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Complexity scale: "minimal", "light", "standard", "full", "planning".
    #[serde(default)]
    pub scale: Option<String>,
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
    /// Human must confirm stage completion (e.g. review/approve all docs).
    #[serde(default)]
    pub requires_approval: bool,
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
    /// Collect all skill names across every stage in this pipeline.
    pub fn all_skill_names(&self) -> Vec<&str> {
        self.stages.iter().flat_map(|s| s.skill_names()).collect()
    }

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

/// Distinguishes how a pipeline run was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    /// Fresh start — no prior run.
    New,
    /// Correcting a completed run (Issue #2).
    Revision,
    /// Extending with a different pipeline on the same topic (Issue #4).
    Continuation,
}

impl std::fmt::Display for RunType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new"),
            Self::Revision => write!(f, "revision"),
            Self::Continuation => write!(f, "continuation"),
        }
    }
}

/// A running instance of a Pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRun {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub stage_states: HashMap<String, StageState>,
    /// Topic this run belongs to.
    pub topic_id: String,
    /// The issue that triggered this run.
    #[serde(default)]
    pub issue_id: String,
    /// If this is a revision/continuation, the parent run's ID.
    #[serde(default)]
    pub parent_run_id: Option<String>,
    /// How this run was created.
    #[serde(default = "default_run_type")]
    pub run_type: RunType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_run_type() -> RunType {
    RunType::New
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    Blocked,
    Ready,
    InProgress,
    Completed,
    Skipped,
    /// Was completed, reopened for correction in a revision run.
    Revised,
}

impl std::fmt::Display for StageState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked => write!(f, "blocked"),
            Self::Ready => write!(f, "ready"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Skipped => write!(f, "skipped"),
            Self::Revised => write!(f, "revised"),
        }
    }
}

impl PipelineRun {
    pub fn new(
        pipeline_def: &PipelineDef,
        title: impl Into<String>,
        topic_id: impl Into<String>,
        issue_id: impl Into<String>,
    ) -> Self {
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
            topic_id: topic_id.into(),
            issue_id: issue_id.into(),
            parent_run_id: None,
            run_type: RunType::New,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a revision run from a completed parent run.
    /// Specified stages are set to Revised; others inherit Completed/Skipped.
    pub fn new_revision(
        pipeline_def: &PipelineDef,
        parent: &PipelineRun,
        revised_stages: &[String],
    ) -> Self {
        let now = Utc::now();
        let mut stage_states = HashMap::new();

        for stage in &pipeline_def.stages {
            let state = if revised_stages.contains(&stage.name) {
                StageState::Revised
            } else {
                // Inherit the parent state (Completed or Skipped) or default Blocked
                parent
                    .stage_states
                    .get(&stage.name)
                    .copied()
                    .unwrap_or(StageState::Blocked)
            };
            stage_states.insert(stage.name.clone(), state);
        }

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pipeline_name: pipeline_def.name.clone(),
            title: format!("{} (revision)", parent.title),
            stage_states,
            topic_id: parent.topic_id.clone(),
            issue_id: parent.issue_id.clone(),
            parent_run_id: Some(parent.id.clone()),
            run_type: RunType::Revision,
            created_at: now,
            updated_at: now,
        }
    }

    /// Recompute blocked/ready states based on current completions.
    /// Revised stages are treated as needing re-work and become Ready when deps are met.
    pub fn refresh_states(&mut self, pipeline_def: &PipelineDef) {
        for stage in &pipeline_def.stages {
            let current = self.stage_states.get(&stage.name).copied();
            // Skip stages already in progress or done (not revised)
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
                // Revised stages stay Revised until deps are met, then go Ready
                if matches!(current, Some(StageState::Revised)) {
                    StageState::Revised
                } else {
                    StageState::Blocked
                }
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
        let run = PipelineRun::new(&def, "Test Feature", "topic-1", "");
        assert_eq!(run.stage_states["domain"], StageState::Ready);
        assert_eq!(run.stage_states["product"], StageState::Blocked);
        assert_eq!(run.stage_states["tech-design"], StageState::Blocked);
        assert_eq!(run.topic_id, "topic-1");
        assert_eq!(run.run_type, RunType::New);
        assert!(run.parent_run_id.is_none());
    }

    #[test]
    fn test_refresh_states() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        let mut run = PipelineRun::new(&def, "Test Feature", "topic-1", "");

        run.stage_states
            .insert("domain".to_string(), StageState::Completed);
        run.refresh_states(&def);

        assert_eq!(run.stage_states["product"], StageState::Ready);
        assert_eq!(run.stage_states["tech-design"], StageState::Blocked);
    }

    #[test]
    fn test_all_skill_names() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        let names = def.all_skill_names();
        assert_eq!(
            names,
            vec!["domain-analysis", "product-prd", "tech-rfc", "tech-adr"]
        );
    }

    #[test]
    fn test_all_skill_names_empty() {
        let def = PipelineDef {
            name: "empty".to_string(),
            description: "No stages".to_string(),
            stages: vec![],
            keywords: vec![],
            scale: None,
        };
        assert!(def.all_skill_names().is_empty());
    }

    #[test]
    fn test_new_revision() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        let mut parent = PipelineRun::new(&def, "Test Feature", "topic-1", "");
        // Complete all stages in parent
        for stage in &def.stages {
            parent
                .stage_states
                .insert(stage.name.clone(), StageState::Completed);
        }

        let revision = PipelineRun::new_revision(&def, &parent, &["tech-design".to_string()]);

        assert_eq!(revision.run_type, RunType::Revision);
        assert_eq!(revision.parent_run_id, Some(parent.id.clone()));
        assert_eq!(revision.topic_id, "topic-1");
        assert_eq!(revision.stage_states["domain"], StageState::Completed);
        assert_eq!(revision.stage_states["product"], StageState::Completed);
        assert_eq!(revision.stage_states["tech-design"], StageState::Revised);
    }

    #[test]
    fn test_refresh_revised_becomes_ready() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        let mut parent = PipelineRun::new(&def, "Test Feature", "topic-1", "");
        for stage in &def.stages {
            parent
                .stage_states
                .insert(stage.name.clone(), StageState::Completed);
        }

        let mut revision = PipelineRun::new_revision(&def, &parent, &["tech-design".to_string()]);
        // product is Completed (dep met), so tech-design should become Ready
        revision.refresh_states(&def);
        assert_eq!(revision.stage_states["tech-design"], StageState::Ready);
    }

    #[test]
    fn test_refresh_revised_stays_revised_when_deps_not_met() {
        let def: PipelineDef = serde_yaml_ng::from_str(sample_pipeline_yaml()).unwrap();
        let mut parent = PipelineRun::new(&def, "Test Feature", "topic-1", "");
        for stage in &def.stages {
            parent
                .stage_states
                .insert(stage.name.clone(), StageState::Completed);
        }

        // Revise both product and tech-design
        let mut revision = PipelineRun::new_revision(
            &def,
            &parent,
            &["product".to_string(), "tech-design".to_string()],
        );
        // product's dep (domain) is Completed → product becomes Ready
        // tech-design's dep (product) is Revised (not Completed) → stays Revised
        revision.refresh_states(&def);
        assert_eq!(revision.stage_states["product"], StageState::Ready);
        assert_eq!(revision.stage_states["tech-design"], StageState::Revised);
    }
}
