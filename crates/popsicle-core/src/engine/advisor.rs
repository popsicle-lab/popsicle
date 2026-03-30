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
    /// CLI command to retrieve enriched prompt with historical references.
    /// Present when action is "create" — the agent should run this BEFORE creating the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_command: Option<String>,
    /// Contextual hints for the user/agent (e.g., skipped upstream skills).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
}

impl Advisor {
    /// Compute all available next steps for a pipeline run.
    ///
    /// With the pipeline-as-state-source model, the advisor only looks at stage states:
    /// - Ready/InProgress: suggest doc create for missing skills, stage complete when all docs exist
    /// - Blocked: show which upstream stages are blocking
    /// - Completed/Skipped: no action needed
    pub fn next_steps(
        pipeline_def: &PipelineDef,
        run: &PipelineRun,
        registry: &SkillRegistry,
        docs: &[DocumentRow],
        topic_docs: &[DocumentRow],
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
                StageState::Ready | StageState::InProgress => {
                    let mut all_skills_have_docs = true;

                    for skill_name in stage.skill_names() {
                        let has_doc_in_run = docs.iter().any(|d| d.skill_name == skill_name);

                        if has_doc_in_run {
                            // Doc exists — no action needed for this skill
                            continue;
                        }

                        all_skills_have_docs = false;

                        // Check topic-level docs from other runs
                        let topic_skill_docs: Vec<&DocumentRow> = topic_docs
                            .iter()
                            .filter(|d| {
                                d.skill_name == skill_name && d.pipeline_run_id != run.id
                            })
                            .collect();

                        let is_cumulative = registry
                            .get(skill_name)
                            .map(|s| s.is_cumulative())
                            .unwrap_or(false);

                        if !is_cumulative && !topic_skill_docs.is_empty() {
                            let latest = &topic_skill_docs[0];
                            if latest.status == "final" {
                                // Already final in another run — suggest skip
                                steps.push(NextStep {
                                    stage: stage.name.clone(),
                                    skill: skill_name.to_string(),
                                    action: "skip".to_string(),
                                    description: format!(
                                        "Existing finalized '{}' (v{}) from topic — stage can be skipped",
                                        latest.title, latest.version
                                    ),
                                    cli_command: String::new(),
                                    prompt: None,
                                    blocked_by: vec![],
                                    requires_approval: false,
                                    context_command: None,
                                    hints: vec![format!(
                                        "Document '{}' already finalized in a previous run",
                                        latest.title
                                    )],
                                });
                                // Count as "has doc" for stage completion check
                                continue;
                            } else {
                                // Exists but not final — suggest update
                                let hints =
                                    Self::build_skill_hints(skill_name, registry, &pipeline_skills);
                                steps.push(NextStep {
                                    stage: stage.name.clone(),
                                    skill: skill_name.to_string(),
                                    action: "update".to_string(),
                                    description: format!(
                                        "Update existing document '{}' (v{}, status: {})",
                                        latest.title, latest.version, latest.status
                                    ),
                                    cli_command: format!(
                                        "popsicle doc create {} --title \"{}\" --run {}",
                                        skill_name, latest.title, run.id
                                    ),
                                    prompt: registry
                                        .get(skill_name)
                                        .ok()
                                        .and_then(|s| s.prompts.get("active").cloned()),
                                    blocked_by: vec![],
                                    requires_approval: false,
                                    context_command: Some(format!(
                                        "popsicle prompt {} --run {} --related --format json",
                                        skill_name, run.id
                                    )),
                                    hints,
                                });
                                continue;
                            }
                        }

                        // No existing docs or cumulative — suggest create
                        let description = registry
                            .get(skill_name)
                            .map(|s| s.description.clone())
                            .unwrap_or_else(|_| format!("Execute skill: {}", skill_name));

                        let prompt = registry
                            .get(skill_name)
                            .ok()
                            .and_then(|s| s.prompts.get("active").cloned().or_else(|| {
                                s.prompts.values().next().cloned()
                            }));

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
                            context_command: Some(format!(
                                "popsicle prompt {} --run {} --related --format json",
                                skill_name, run.id
                            )),
                            hints,
                        });
                    }

                    // If all skills have docs, suggest stage complete
                    if all_skills_have_docs {
                        steps.push(NextStep {
                            stage: stage.name.clone(),
                            skill: String::new(),
                            action: "complete_stage".to_string(),
                            description: format!(
                                "All documents created for '{}' — ready to complete",
                                stage.name
                            ),
                            cli_command: format!(
                                "popsicle pipeline stage complete {}{}",
                                stage.name,
                                if stage.requires_approval {
                                    " --confirm"
                                } else {
                                    ""
                                }
                            ),
                            prompt: None,
                            blocked_by: vec![],
                            requires_approval: stage.requires_approval,
                            context_command: None,
                            hints: if stage.requires_approval {
                                vec!["Requires human review and approval".to_string()]
                            } else {
                                vec![]
                            },
                        });
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
                            context_command: None,
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
        topic_docs: &[DocumentRow],
    ) -> bool {
        Self::next_steps(pipeline_def, run, registry, docs, topic_docs)
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
                    requires_approval: false,
                },
                StageDef {
                    name: "design".to_string(),
                    skills: vec![],
                    skill: Some("domain-analysis".to_string()),
                    description: "Design".to_string(),
                    depends_on: vec!["domain".to_string()],
                    requires_approval: false,
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
        let run = PipelineRun::new(&pipeline, "Test", "test-topic".to_string(), "");
        let docs: Vec<DocumentRow> = vec![];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs, &[]);

        let actionable: Vec<_> = steps.iter().filter(|s| s.action != "blocked").collect();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].action, "create");
        assert_eq!(actionable[0].skill, "domain-analysis");

        let blocked: Vec<_> = steps.iter().filter(|s| s.action == "blocked").collect();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].stage, "design");
    }

    #[test]
    fn test_next_steps_with_doc_suggests_stage_complete() {
        let registry = make_registry();
        let pipeline = make_pipeline();
        let mut run = PipelineRun::new(&pipeline, "Test", "test-topic".to_string(), "");
        run.stage_states
            .insert("domain".to_string(), StageState::InProgress);
        let docs = vec![DocumentRow {
            id: "d1".to_string(),
            doc_type: "domain-model".to_string(),
            title: "Domain Doc".to_string(),
            status: "active".to_string(),
            skill_name: "domain-analysis".to_string(),
            pipeline_run_id: run.id.clone(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
            summary: String::new(),
            doc_tags: "[]".to_string(),
            topic_id: "test-topic".to_string(),
            version: 1,
            parent_doc_id: None,
        }];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs, &[]);
        let actionable: Vec<_> = steps.iter().filter(|s| s.action != "blocked").collect();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].action, "complete_stage");
        assert!(actionable[0].cli_command.contains("pipeline stage complete"));
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
                requires_approval: false,
            }],
            keywords: vec![],
            scale: None,
        };
        let mut run = PipelineRun::new(&pipeline, "Test", "test-topic".to_string(), "");
        run.stage_states
            .insert("domain".to_string(), StageState::Completed);

        let docs = vec![DocumentRow {
            id: "d1".to_string(),
            doc_type: "domain-model".to_string(),
            title: "Done".to_string(),
            status: "final".to_string(),
            skill_name: "domain-analysis".to_string(),
            pipeline_run_id: run.id.clone(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
            summary: String::new(),
            doc_tags: "[]".to_string(),
            topic_id: "test-topic".to_string(),
            version: 1,
            parent_doc_id: None,
        }];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs, &[]);
        assert!(steps.is_empty());
    }

    #[test]
    fn test_has_actionable_steps() {
        let registry = make_registry();
        let pipeline = make_pipeline();
        let run = PipelineRun::new(&pipeline, "Test", "test-topic".to_string(), "");
        let docs: Vec<DocumentRow> = vec![];

        assert!(Advisor::has_actionable_steps(
            &pipeline, &run, &registry, &docs, &[]
        ));
    }

    #[test]
    fn test_stage_requires_approval_propagated() {
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
                requires_approval: true, // Stage requires approval
            }],
            keywords: vec![],
            scale: None,
        };
        let mut run = PipelineRun::new(&pipeline, "Test", "test-topic".to_string(), "");
        run.stage_states
            .insert("domain".to_string(), StageState::InProgress);
        let docs = vec![DocumentRow {
            id: "d1".to_string(),
            doc_type: "domain-model".to_string(),
            title: "Domain Doc".to_string(),
            status: "active".to_string(),
            skill_name: "domain-analysis".to_string(),
            pipeline_run_id: run.id.clone(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
            summary: String::new(),
            doc_tags: "[]".to_string(),
            topic_id: "test-topic".to_string(),
            version: 1,
            parent_doc_id: None,
        }];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs, &[]);
        let complete_step = steps
            .iter()
            .find(|s| s.action == "complete_stage")
            .unwrap();
        assert!(complete_step.requires_approval);
        assert!(complete_step.cli_command.contains("--confirm"));
    }

    #[test]
    fn test_next_steps_cross_run_singleton_skip() {
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
                requires_approval: false,
            }],
            keywords: vec![],
            scale: None,
        };
        let run = PipelineRun::new(&pipeline, "Test", "test-topic", "");
        let docs: Vec<DocumentRow> = vec![]; // No docs in current run

        // Finalized doc from another run in same topic
        let topic_docs = vec![DocumentRow {
            id: "d-prev".to_string(),
            doc_type: "domain-model".to_string(),
            title: "Previous Domain Doc".to_string(),
            status: "final".to_string(),
            skill_name: "domain-analysis".to_string(),
            pipeline_run_id: "other-run-id".to_string(),
            topic_id: "test-topic".to_string(),
            version: 1,
            parent_doc_id: None,
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
            summary: String::new(),
            doc_tags: "[]".to_string(),
        }];

        let steps = Advisor::next_steps(&pipeline, &run, &registry, &docs, &topic_docs);
        let actionable: Vec<_> = steps.iter().filter(|s| s.action != "blocked").collect();
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].action, "skip");
    }
}
