use std::env;

use popsicle_core::helpers;
use popsicle_core::registry::SkillRegistry;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Args)]
pub struct PromptArgs {
    /// Skill name
    skill: String,
    /// Workflow state to get prompt for (defaults to skill's initial state)
    #[arg(short, long)]
    state: Option<String>,
    /// Pipeline run ID — if provided, injects upstream documents as context
    #[arg(short, long)]
    run: Option<String>,
}

pub fn execute(args: PromptArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let registry = load_registry()?;
    let skill = registry
        .get(&args.skill)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let state = args
        .state
        .as_deref()
        .unwrap_or(&skill.workflow.initial);

    let raw_prompt = skill.prompts.get(state).cloned().unwrap_or_default();

    let base_prompt = expand_prompt_vars(&raw_prompt, &skill.name, state, args.run.as_deref());

    let input_context = if let Some(ref run_id) = args.run {
        build_input_context(&args.skill, run_id, &registry)?
    } else {
        None
    };

    let full_prompt = if let Some(ref ctx) = input_context {
        format!("{}\n\n---\n\n## Input Context (from upstream skills)\n\n{}", base_prompt.trim(), ctx)
    } else {
        base_prompt.clone()
    };

    match format {
        OutputFormat::Text => {
            if base_prompt.is_empty() {
                println!(
                    "No prompt defined for skill '{}' at state '{}'.",
                    skill.name, state
                );
                let available: Vec<_> = skill.prompts.keys().collect();
                if !available.is_empty() {
                    println!(
                        "Available states with prompts: {}",
                        available
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            } else {
                println!("=== Prompt: {} @ {} ===\n", skill.name, state);
                println!("{}", full_prompt.trim());
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "skill": skill.name,
                "state": state,
                "prompt": base_prompt,
                "full_prompt": full_prompt,
                "input_context": input_context,
                "available_states": skill.prompts.keys().collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

/// Build context from upstream skill documents for this skill's inputs.
fn build_input_context(
    skill_name: &str,
    run_id: &str,
    registry: &SkillRegistry,
) -> anyhow::Result<Option<String>> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    if !layout.is_initialized() {
        return Ok(None);
    }

    let skill = registry
        .get(skill_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if skill.inputs.is_empty() {
        return Ok(None);
    }

    let db = IndexDb::open(&layout.db_path())?;
    let all_docs = db
        .query_documents(None, None, Some(run_id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut context_parts = Vec::new();

    for input in &skill.inputs {
        let upstream_docs: Vec<_> = all_docs
            .iter()
            .filter(|d| d.skill_name == input.from_skill)
            .collect();

        if upstream_docs.is_empty() {
            if input.required {
                context_parts.push(format!(
                    "### {} (from: {}) — NOT YET CREATED\n\n> This required input document has not been created yet.\n",
                    input.artifact_type, input.from_skill
                ));
            }
            continue;
        }

        for doc_row in upstream_docs {
            let body = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
                .map(|d| d.body)
                .unwrap_or_else(|_| "(unable to read document)".to_string());

            context_parts.push(format!(
                "### {} — {} [{}]\n\n{}\n",
                input.artifact_type, doc_row.title, doc_row.status, body.trim()
            ));
        }
    }

    if context_parts.is_empty() {
        Ok(None)
    } else {
        Ok(Some(context_parts.join("\n---\n\n")))
    }
}

fn load_registry() -> anyhow::Result<popsicle_core::registry::SkillRegistry> {
    let cwd = env::current_dir()?;
    helpers::load_registry(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Replace template variables in prompt text.
/// Supported: {skill}, {state}, {run_id}, {date}, {branch}
fn expand_prompt_vars(
    prompt: &str,
    skill: &str,
    state: &str,
    run_id: Option<&str>,
) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    prompt
        .replace("{skill}", skill)
        .replace("{state}", state)
        .replace("{run_id}", run_id.unwrap_or("none"))
        .replace("{date}", &date)
        .replace("{branch}", &branch)
}
