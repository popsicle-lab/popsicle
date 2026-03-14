use std::env;

use popsicle_core::engine::{ContextInput, assemble_input_context};
use popsicle_core::helpers;
use popsicle_core::memory::{self, Memory, MemoryStore};
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

    let state = args.state.as_deref().unwrap_or(&skill.workflow.initial);

    let raw_prompt = skill.prompts.get(state).cloned().unwrap_or_default();

    let base_prompt = expand_prompt_vars(&raw_prompt, &skill.name, state, args.run.as_deref());

    let project_context = load_project_context();

    let memories = load_ranked_memories();

    let assembled = if let Some(ref run_id) = args.run {
        build_input_context(&args.skill, run_id, &registry)?
    } else {
        None
    };

    // Attention-optimized ordering:
    //   1. Project context (background, lowest relevance — front of prompt)
    //   2. Project memories (accumulated experience — low relevance)
    //   3. Input context (low→med→high from upstream skills)
    //   4. Prompt instruction (highest attention — end of prompt)
    let full_prompt = build_full_prompt(&project_context, &memories, &assembled, &base_prompt);

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
            let context_parts = assembled.as_ref().map(|a| &a.parts);
            let memory_summaries: Option<Vec<_>> = memories.as_ref().map(|mems| {
                mems.iter()
                    .map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "type": m.memory_type.to_string(),
                            "summary": m.summary,
                        })
                    })
                    .collect()
            });
            let result = serde_json::json!({
                "skill": skill.name,
                "state": state,
                "prompt": base_prompt,
                "full_prompt": full_prompt,
                "project_context": project_context,
                "memories": memory_summaries,
                "input_context": assembled.as_ref().map(|a| &a.full_text),
                "context_parts": context_parts,
                "available_states": skill.prompts.keys().collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

/// Read `.popsicle/project-context.md` if it exists.
fn load_project_context() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let layout = ProjectLayout::new(&cwd);
    let path = layout.project_context_path();
    std::fs::read_to_string(path)
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Load memories from `.popsicle/memories.md`, auto-expire stale short-term entries, and rank.
fn load_ranked_memories() -> Option<Vec<Memory>> {
    let cwd = env::current_dir().ok()?;
    let layout = ProjectLayout::new(&cwd);
    let path = layout.memories_path();
    let mut memories = MemoryStore::load(&path).ok()?;
    if memories.is_empty() {
        return None;
    }

    let expired = MemoryStore::expire_short_term(&mut memories, memory::SHORT_TERM_EXPIRY_DAYS);
    if !expired.is_empty() {
        let _ = MemoryStore::save(&path, &memories);
    }

    let pct = MemoryStore::capacity_pct(&memories);
    if pct >= 80 {
        eprintln!(
            "warning: memory capacity at {}% ({} / {} lines). Consider running `popsicle memory gc`.",
            pct,
            MemoryStore::line_count(&memories),
            memory::MAX_LINES,
        );
    }

    let ranked: Vec<Memory> =
        memory::rank_memories(&memories, &[], &[], memory::DEFAULT_INJECT_LIMIT)
            .into_iter()
            .cloned()
            .collect();
    if ranked.is_empty() {
        None
    } else {
        Some(ranked)
    }
}

/// Format memories as a prompt section.
fn format_memories_section(memories: &[Memory]) -> String {
    let mut lines = vec![
        "## Project Memories".to_string(),
        String::new(),
        "以下是项目积累的经验，请在工作中注意避免已知问题：".to_string(),
    ];
    for m in memories {
        let stale_mark = if m.stale { " [STALE]" } else { "" };
        lines.push(format!("- [{}] {}{}", m.memory_type, m.summary, stale_mark));
    }
    lines.join("\n")
}

/// Assemble the final prompt with attention-optimized ordering.
fn build_full_prompt(
    project_context: &Option<String>,
    memories: &Option<Vec<Memory>>,
    assembled: &Option<popsicle_core::engine::AssembledContext>,
    base_prompt: &str,
) -> String {
    let mut sections = Vec::new();

    if let Some(pc) = project_context {
        sections.push(format!("## Project Context (background)\n\n{}", pc.trim()));
    }

    if let Some(mems) = memories {
        sections.push(format_memories_section(mems));
    }

    if let Some(ctx) = assembled {
        sections.push(format!(
            "## Input Context (from upstream skills)\n\n{}",
            ctx.full_text
        ));
    }

    if sections.is_empty() {
        return base_prompt.to_string();
    }

    sections.push(base_prompt.trim().to_string());
    sections.join("\n\n---\n\n")
}

/// Build context from upstream skill documents using selective injection.
///
/// When a required input's `from_skill` is not present in the current pipeline,
/// injects an adaptive guidance message instead of "NOT YET CREATED".
fn build_input_context(
    skill_name: &str,
    run_id: &str,
    registry: &SkillRegistry,
) -> anyhow::Result<Option<popsicle_core::engine::AssembledContext>> {
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

    let pipeline_skills: Vec<String> = db
        .get_pipeline_run(run_id)
        .ok()
        .flatten()
        .and_then(|run| {
            helpers::find_pipeline(&cwd, &run.pipeline_name)
                .ok()
                .map(|p| {
                    p.all_skill_names()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect()
                })
        })
        .unwrap_or_default();

    let mut context_inputs = Vec::new();

    for input in &skill.inputs {
        let upstream_docs: Vec<_> = all_docs
            .iter()
            .filter(|d| d.skill_name == input.from_skill)
            .collect();

        if upstream_docs.is_empty() {
            if input.required {
                let skill_in_pipeline = pipeline_skills.is_empty()
                    || pipeline_skills.iter().any(|s| s == &input.from_skill);

                if skill_in_pipeline {
                    context_inputs.push(ContextInput {
                        artifact_type: input.artifact_type.clone(),
                        title: format!("{} (NOT YET CREATED)", input.from_skill),
                        status: "missing".into(),
                        body: "> This required input document has not been created yet.\n"
                            .to_string(),
                        relevance: input.relevance,
                        sections: input.sections.clone(),
                    });
                } else {
                    context_inputs.push(ContextInput {
                        artifact_type: input.artifact_type.clone(),
                        title: format!("{} (skipped by pipeline)", input.from_skill),
                        status: "skipped".into(),
                        body: format!(
                            "> This pipeline does not include the '{}' skill. \
                            Gather relevant {} context directly from the user, \
                            codebase, or project context instead.\n",
                            input.from_skill, input.artifact_type
                        ),
                        relevance: input.relevance,
                        sections: input.sections.clone(),
                    });
                }
            }
            continue;
        }

        for doc_row in upstream_docs {
            let body = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
                .map(|d| d.body)
                .unwrap_or_else(|_| "(unable to read document)".to_string());

            context_inputs.push(ContextInput {
                artifact_type: input.artifact_type.clone(),
                title: doc_row.title.clone(),
                status: doc_row.status.clone(),
                body,
                relevance: input.relevance,
                sections: input.sections.clone(),
            });
        }
    }

    if context_inputs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(assemble_input_context(context_inputs)))
    }
}

fn load_registry() -> anyhow::Result<popsicle_core::registry::SkillRegistry> {
    let cwd = env::current_dir()?;
    helpers::load_registry(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Replace template variables in prompt text.
/// Supported: {skill}, {state}, {run_id}, {date}, {branch}
fn expand_prompt_vars(prompt: &str, skill: &str, state: &str, run_id: Option<&str>) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use popsicle_core::engine::{AssembledContext, ContextPart};
    use popsicle_core::memory::{MemoryLayer, MemoryType};

    #[test]
    fn test_build_full_prompt_no_context() {
        let result = build_full_prompt(&None, &None, &None, "Do the thing.");
        assert_eq!(result, "Do the thing.");
    }

    #[test]
    fn test_build_full_prompt_project_context_only() {
        let pc = Some("## Tech Stack\n- Rust".to_string());
        let result = build_full_prompt(&pc, &None, &None, "Do the thing.");
        assert!(result.starts_with("## Project Context (background)"));
        assert!(result.contains("## Tech Stack\n- Rust"));
        assert!(result.ends_with("Do the thing."));
    }

    #[test]
    fn test_build_full_prompt_input_context_only() {
        let assembled = Some(AssembledContext {
            parts: vec![ContextPart {
                artifact_type: "prd".into(),
                title: "PRD".into(),
                status: "approved".into(),
                relevance: "high".into(),
                content: "PRD body".into(),
            }],
            full_text: "### [Primary] prd — PRD [approved]\n\nPRD body".into(),
        });
        let result = build_full_prompt(&None, &None, &assembled, "Do the thing.");
        assert!(!result.contains("Project Context"));
        assert!(result.contains("## Input Context (from upstream skills)"));
        assert!(result.contains("PRD body"));
        assert!(result.ends_with("Do the thing."));
    }

    #[test]
    fn test_build_full_prompt_all_sections_ordering() {
        let pc = Some("## Tech Stack\n- Rust".to_string());
        let mems = Some(vec![Memory {
            id: 1,
            memory_type: MemoryType::Bug,
            summary: "Test bug".into(),
            created: "2026-03-14".into(),
            layer: MemoryLayer::LongTerm,
            refs: 0,
            tags: vec![],
            files: vec![],
            run: None,
            stale: false,
            detail: String::new(),
        }]);
        let assembled = Some(AssembledContext {
            parts: vec![],
            full_text: "upstream docs".into(),
        });
        let result = build_full_prompt(&pc, &mems, &assembled, "instruction");

        let pc_pos = result.find("Project Context").unwrap();
        let mem_pos = result.find("Project Memories").unwrap();
        let ic_pos = result.find("Input Context").unwrap();
        let inst_pos = result.find("instruction").unwrap();
        assert!(pc_pos < mem_pos, "project context before memories");
        assert!(mem_pos < ic_pos, "memories before input context");
        assert!(ic_pos < inst_pos, "input context before instruction");
    }

    #[test]
    fn test_build_full_prompt_memories_only() {
        let mems = Some(vec![Memory {
            id: 1,
            memory_type: MemoryType::Pattern,
            summary: "Always use serde(default)".into(),
            created: "2026-03-14".into(),
            layer: MemoryLayer::LongTerm,
            refs: 3,
            tags: vec![],
            files: vec![],
            run: None,
            stale: false,
            detail: String::new(),
        }]);
        let result = build_full_prompt(&None, &mems, &None, "Do the thing.");
        assert!(result.contains("## Project Memories"));
        assert!(result.contains("[PATTERN] Always use serde(default)"));
        assert!(result.ends_with("Do the thing."));
    }
}
