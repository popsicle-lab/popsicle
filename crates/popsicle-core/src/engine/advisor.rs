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
}

impl Advisor {
    /// Compute all available next steps for a pipeline run.
    /// Takes existing documents into account for InProgress stages.
    pub fn next_steps(
        pipeline_def: &PipelineDef,
        run: &PipelineRun,
        registry: &SkillRegistry,
        docs: &[DocumentRow],
    ) -> Vec<NextStep> {
        let mut steps = Vec::new();

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
                            description: format!(
                                "Blocked: waiting for {}",
                                missing.join(", ")
                            ),
                            cli_command: String::new(),
                            prompt: None,
                            blocked_by: missing.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        steps
    }

    fn add_doc_steps(
        steps: &mut Vec<NextStep>,
        stage_name: &str,
        skill_name: &str,
        registry: &SkillRegistry,
        docs: &[DocumentRow],
        run: &PipelineRun,
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
                    });
                }
            }
        }
    }
}
