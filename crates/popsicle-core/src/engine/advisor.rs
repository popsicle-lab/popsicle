use serde::Serialize;

use crate::model::{PipelineDef, PipelineRun, StageState};
use crate::registry::SkillRegistry;
use crate::storage::DocumentRow;

/// Generates "next step" suggestions based on the current pipeline state.
#[derive(Debug)]
pub struct Advisor;

/// A recommended next action for the user or AI agent.
#[derive(Debug, Clone, Serialize)]
pub struct NextStep {
    pub stage: String,
    pub skill: String,
    pub action: String,
    pub description: String,
    pub cli_command: String,
    pub prompt: Option<String>,
    pub blocked_by: Vec<String>,
    pub requires_approval: bool,
    /// Contextual hints for the user/agent (e.g., skipped upstream skills).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
}

impl Advisor {
    /// Compute all available next steps for a pipeline run.
    /// Takes existing documents into account for InProgress stages.
    /// Generates hints when skills have upstream dependencies skipped by the pipeline.
    pub fn next_steps(
        pipeline_def: &PipelineDef,
        run: &PipelineRun,
        registry: &SkillRegistry,
        docs: &[DocumentRow],
    ) -> Vec<NextStep> {
        let mut steps = Vec::new();
        let pipeline_skills = pipeline_def.all_skill_names();

        for stage in &pipeline_def.stages {
            let state = run
                .stage_states
                .get(&stage.name)
                .copied()
                .unwrap_or(StageState::Blocked);

            match state {
                StageState::Ready => {
                    for skill_name in stage.skill_names() {
                        let has_doc = docs.iter().any(|d| d.skill_name == skill_name);

                        if has_doc {
                            Self::add_doc_steps(
                                &mut steps,
                                &stage.name,
                                skill_name,
                                registry,
                                docs,
                                run,
                                pipeline_def,
                                &pipeline_skills,
                            );
                        } else {
                            let description = registry
                                .get(skill_name)
                                .map(|s| s.description.clone())
                                .unwrap_or_else(|_| format!("Execute skill: {}", skill_name));

                            let prompt = registry
                                .get(skill_name)
                                .ok()
                                .and_then(|s| s.prompts.get(&s.workflow.initial).cloned());

                            let hints =
                                Self::build_skill_hints(skill_name, registry, &pipeline_skills);

                            steps.push(NextStep {
                                stage: stage.name.clone(),
                                skill: skill_name.to_string(),
                                action: "create".to_string(),
                                description,
                                cli_command: format!(
                                    "popsicle doc create {} --title \"<title>\" --run {}",
                                    skill_name, run.id
                                ),
                                prompt,
                                blocked_by: vec![],
                                requires_approval: false,
                                hints,
                            });
                        }
                    }
                }
                StageState::InProgress => {
                    for skill_name in stage.skill_names() {
                        Self::add_doc_steps(
                            &mut steps,
                            &stage.name,
                            skill_name,
                            registry,
                            docs,
                            run,
                            pipeline_def,
                            &pipeline_skills,
                        );
                    }
                }
                StageState::Blocked => {
                    let missing: Vec<String> = stage
                        .depends_on
                        .iter()
                        .filter(|dep| {
                            !matches!(
                                run.stage_states.get(*dep),
                                Some(StageState::Completed | StageState::Skipped)
                            )
                        })
                        .cloned()
                        .collect();

                    for skill_name in stage.skill_names() {
                        steps.push(NextStep {
                            stage: stage.name.clone(),
                            skill: skill_name.to_string(),
                            action: "blocked".to_string(),
                            description: format!("Blocked: waiting for {}", missing.join(", ")),
                            cli_command: String::new(),
                            prompt: None,
                            blocked_by: missing.clone(),
                            requires_approval: false,
                            hints: vec![],
                        });
                    }
                }
                _ => {}
            }
        }

        steps
    }

    /// Check if a step is actionable (not blocked).
    pub fn has_actionable_steps(
        pipeline_def: &PipelineDef,
        run: &PipelineRun,
        registry: &SkillRegistry,
        docs: &[DocumentRow],
    ) -> bool {
        Self::next_steps(pipeline_def, run, registry, docs)
            .iter()
            .any(|s| s.action != "blocked")
    }

    /// Build hints for a skill whose required upstream skills are not in the pipeline.
    fn build_skill_hints(
        skill_name: &str,
        registry: &SkillRegistry,
        pipeline_skills: &[&str],
    ) -> Vec<String> {
        let mut hints = Vec::new();
        if let Ok(skill) = registry.get(skill_name) {
            let skipped: Vec<&str> = skill
                .inputs
                .iter()
                .filter(|i| i.required && !pipeline_skills.contains(&i.from_skill.as_str()))
                .map(|i| i.from_skill.as_str())
                .collect();
            if !skipped.is_empty() {
                hints.push(format!(
                    "Pipeline skips [{}] — gather relevant context from the user or codebase directly",
                    skipped.join(", ")
                ));
            }
        }
        hints
    }

    #[allow(clippy::too_many_arguments)]
    fn add_doc_steps(
        steps: &mut Vec<NextStep>,
        stage_name: &str,
        skill_name: &str,
        registry: &SkillRegistry,
        docs: &[DocumentRow],
        run: &PipelineRun,
        pipeline_def: &PipelineDef,
        pipeline_skills: &[&str],
    ) {
        let skill_docs: Vec<&DocumentRow> = docs
            .iter()
            .filter(|d| d.skill_name == skill_name && d.pipeline_run_id == run.id)
            .collect();

        for doc in skill_docs {
            if let Ok(skill) = registry.get(skill_name) {
                if skill.is_final_state(&doc.status) {
                    continue;
                }
                let actions = skill.available_actions(&doc.status);
                for transition in actions {
                    let prompt = skill.prompts.get(&doc.status).cloned();
                    let mut hints = Self::build_skill_hints(skill_name, registry, pipeline_skills);

                    // Pre-check guard: if the transition has a guard with upstream_approved,
                    // verify it would pass with pipeline-aware logic
                    if let Some(ref guard_expr) = transition.guard
                        && guard_expr.contains("upstream_approved")
                    {
                        let full_doc = crate::model::Document {
                            id: doc.id.clone(),
                            doc_type: doc.doc_type.clone(),
                            title: doc.title.clone(),
                            status: doc.status.clone(),
                            skill_name: doc.skill_name.clone(),
                            pipeline_run_id: doc.pipeline_run_id.clone(),
                            tags: vec![],
                            metadata: serde_yaml_ng::Value::Null,
                            created_at: None,
                            updated_at: None,
                            body: String::new(),
                            file_path: std::path::PathBuf::new(),
                        };
                        if let Ok(result) = super::guard::check_guard(
                            "upstream_approved",
                            &full_doc,
                            docs,
                            registry,
                            Some(pipeline_def),
                        ) && !result.passed
                        {
                            hints.push(format!("Guard will block: {}", result.message));
                        }
                    }

                    steps.push(NextStep {
                        stage: stage_name.to_string(),
                        skill: skill_name.to_string(),
                        action: transition.action.clone(),
                        description: format!(
                            "{}: {} ({})",
                            doc.title, transition.action, doc.status
                        ),
                        cli_command: format!(
                            "popsicle doc transition {} {}",
                            doc.id, transition.action
                        ),
                        prompt,
                        blocked_by: vec![],
                        requires_approval: transition.requires_approval,
                        hints,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{StageDef, StageState};
    use crate::registry::SkillRegistry;

    fn sample_skill_yaml() -> &'static str {
        r#"
name: domain-analysis
description: Domain boundary analysis
version: "0.1.0"
artifacts:
  - type: domain-model
    template: templates/domain.md
    file_pattern: "{slug}.domain.md"
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
        - to: draft
          action: revise
    approved:
      final: true
prompts:
  draft: "Analyze the domain..."
"#
    }

    fn make_registry() -> SkillRegistry {
        let mut registry = SkillRegistry::new();
        let skill: crate::model::SkillDef = serde_yaml_ng::from_str(sample_skill_yaml()).unwrap();
        registry.register(skill);
        registry
    }

    fn make_pipeline() -> PipelineDef {
        PipelineDef {
            name: "test".to_string(),
            description: "Test pipeline".to_string(),
            stages: vec![
                StageDef {
                    name: "domain".to_string(),
                    skills: vec![],
                    skill: Some("domain-analysis".to_string()),
                    description: "Domain".to_string(),
                    depends_on: vec![],
                },
                StageDef {
                    name: "design".to_string(),
                    skills: vec![],
                    skill: Some("domain-analysis".to_string()),
                    description: "Design".to_string(),
                    depends_on: vec!["domain".to_string()],
                },
            ],
            keywords: vec![],
            scale: None,
        }
    }

    #[test]
    fn test_next_steps_ready_no_docs() {
        let registry = make_registry();
        let pipeline = make_pipeline();
        let run = PipelineRun::new(&pipeline, "Test");
        let docs: Vec<DocumentRow> = vec![];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs);

        let actionable: Vec<_> = steps.iter().filter(|s| s.action != "blocked").collect();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].action, "create");
        assert_eq!(actionable[0].skill, "domain-analysis");

        let blocked: Vec<_> = steps.iter().filter(|s| s.action == "blocked").collect();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].stage, "design");
    }

    #[test]
    fn test_next_steps_with_draft_doc() {
        let registry = make_registry();
        let pipeline = make_pipeline();
        let run = PipelineRun::new(&pipeline, "Test");
        let docs = vec![DocumentRow {
            id: "d1".to_string(),
            doc_type: "domain-model".to_string(),
            title: "Domain Doc".to_string(),
            status: "draft".to_string(),
            skill_name: "domain-analysis".to_string(),
            pipeline_run_id: run.id.clone(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
        }];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs);
        let actionable: Vec<_> = steps.iter().filter(|s| s.action != "blocked").collect();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].action, "submit");
    }

    #[test]
    fn test_next_steps_all_complete() {
        let registry = make_registry();
        let pipeline = PipelineDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            stages: vec![StageDef {
                name: "domain".to_string(),
                skills: vec![],
                skill: Some("domain-analysis".to_string()),
                description: "Domain".to_string(),
                depends_on: vec![],
            }],
            keywords: vec![],
            scale: None,
        };
        let mut run = PipelineRun::new(&pipeline, "Test");
        run.stage_states
            .insert("domain".to_string(), StageState::Completed);

        let docs = vec![DocumentRow {
            id: "d1".to_string(),
            doc_type: "domain-model".to_string(),
            title: "Done".to_string(),
            status: "approved".to_string(),
            skill_name: "domain-analysis".to_string(),
            pipeline_run_id: run.id.clone(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
        }];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs);
        assert!(steps.is_empty());
    }

    #[test]
    fn test_has_actionable_steps() {
        let registry = make_registry();
        let pipeline = make_pipeline();
        let run = PipelineRun::new(&pipeline, "Test");
        let docs: Vec<DocumentRow> = vec![];

        assert!(Advisor::has_actionable_steps(
            &pipeline, &run, &registry, &docs
        ));
    }

    fn sample_skill_with_approval() -> &'static str {
        r#"
name: domain-analysis
description: Domain boundary analysis
version: "0.1.0"
artifacts:
  - type: domain-model
    template: templates/domain.md
    file_pattern: "{slug}.domain.md"
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
prompts:
  draft: "Analyze the domain..."
"#
    }

    #[test]
    fn test_requires_approval_propagated_to_next_step() {
        let mut registry = SkillRegistry::new();
        let skill: crate::model::SkillDef =
            serde_yaml_ng::from_str(sample_skill_with_approval()).unwrap();
        registry.register(skill);

        let pipeline = PipelineDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            stages: vec![StageDef {
                name: "domain".to_string(),
                skills: vec![],
                skill: Some("domain-analysis".to_string()),
                description: "Domain".to_string(),
                depends_on: vec![],
            }],
            keywords: vec![],
            scale: None,
        };
        let run = PipelineRun::new(&pipeline, "Test");
        let docs = vec![DocumentRow {
            id: "d1".to_string(),
            doc_type: "domain-model".to_string(),
            title: "Domain Doc".to_string(),
            status: "review".to_string(),
            skill_name: "domain-analysis".to_string(),
            pipeline_run_id: run.id.clone(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
        }];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs);
        let approve_step = steps.iter().find(|s| s.action == "approve").unwrap();
        assert!(approve_step.requires_approval);

        let revise_step = steps.iter().find(|s| s.action == "revise").unwrap();
        assert!(!revise_step.requires_approval);
    }
}
