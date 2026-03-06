use std::env;
use std::path::PathBuf;

use anyhow::Context;
use popsicle_core::registry::{SkillLoader, SkillRegistry};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum SkillCommand {
    /// List all registered Skills
    List {
        /// Additional skills directory to scan
        #[arg(short, long)]
        skills_dir: Option<PathBuf>,
    },
    /// Show details of a specific Skill
    Show {
        /// Skill name
        name: String,
        /// Additional skills directory to scan
        #[arg(short, long)]
        skills_dir: Option<PathBuf>,
    },
    /// Create a new custom Skill scaffold
    Create {
        /// Skill name (lowercase, hyphens allowed)
        name: String,
        /// Short description
        #[arg(short, long, default_value = "A custom skill")]
        description: String,
        /// Artifact type this skill produces
        #[arg(short, long)]
        artifact_type: Option<String>,
        /// Create in project-local .popsicle/skills/ instead of workspace skills/
        #[arg(long)]
        local: bool,
    },
}

pub fn execute(cmd: SkillCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        SkillCommand::List { skills_dir } => list_skills(skills_dir, format),
        SkillCommand::Show { name, skills_dir } => show_skill(&name, skills_dir, format),
        SkillCommand::Create {
            name,
            description,
            artifact_type,
            local,
        } => create_skill(&name, &description, artifact_type.as_deref(), local, format),
    }
}

fn load_registry(extra_dir: Option<PathBuf>) -> anyhow::Result<SkillRegistry> {
    let mut registry = SkillRegistry::new();

    // Load built-in skills from the popsicle binary's sibling `skills/` directory.
    if let Ok(exe) = env::current_exe() {
        // Try: <exe_dir>/../skills/ (for development layout)
        if let Some(exe_dir) = exe.parent() {
            let dev_skills = exe_dir
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .map(|p| p.join("skills"));
            if let Some(dir) = dev_skills {
                if dir.is_dir() {
                    let _ = SkillLoader::load_dir(&dir, &mut registry);
                }
            }
        }
    }

    // Load from workspace root skills/ (for development)
    let cwd = env::current_dir()?;
    let workspace_skills = cwd.join("skills");
    if workspace_skills.is_dir() {
        SkillLoader::load_dir(&workspace_skills, &mut registry)
            .context("Loading workspace skills")?;
    }

    // Load project-local custom skills from .popsicle/skills/
    let local_skills = cwd.join(".popsicle").join("skills");
    if local_skills.is_dir() {
        SkillLoader::load_dir(&local_skills, &mut registry)
            .context("Loading project skills")?;
    }

    if let Some(dir) = extra_dir {
        SkillLoader::load_dir(&dir, &mut registry)
            .context("Loading extra skills directory")?;
    }

    Ok(registry)
}

fn list_skills(skills_dir: Option<PathBuf>, format: &OutputFormat) -> anyhow::Result<()> {
    let registry = load_registry(skills_dir)?;
    let skills = registry.list();

    match format {
        OutputFormat::Text => {
            if skills.is_empty() {
                println!("No skills found.");
                return Ok(());
            }
            println!("{:<25} {:<12} {}", "NAME", "VERSION", "DESCRIPTION");
            println!("{}", "-".repeat(70));
            for skill in &skills {
                println!(
                    "{:<25} {:<12} {}",
                    skill.name, skill.version, skill.description
                );
            }
            println!("\n{} skill(s) registered.", skills.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = skills
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "name": s.name,
                        "version": s.version,
                        "description": s.description,
                        "artifacts": s.artifacts.iter().map(|a| &a.artifact_type).collect::<Vec<_>>(),
                        "workflow_initial": s.workflow.initial,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }

    Ok(())
}

fn show_skill(name: &str, skills_dir: Option<PathBuf>, format: &OutputFormat) -> anyhow::Result<()> {
    let registry = load_registry(skills_dir)?;
    let skill = registry.get(name).map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Skill: {}", skill.name);
            println!("Version: {}", skill.version);
            println!("Description: {}", skill.description);
            println!();

            if !skill.inputs.is_empty() {
                println!("Inputs:");
                for input in &skill.inputs {
                    println!(
                        "  - {} from '{}' ({})",
                        input.artifact_type,
                        input.from_skill,
                        if input.required {
                            "required"
                        } else {
                            "optional"
                        }
                    );
                }
                println!();
            }

            println!("Artifacts:");
            for artifact in &skill.artifacts {
                println!(
                    "  - type: {}, pattern: {}",
                    artifact.artifact_type, artifact.file_pattern
                );
            }
            println!();

            println!("Workflow (initial: {}):", skill.workflow.initial);
            for (state_name, state_def) in &skill.workflow.states {
                let marker = if state_def.r#final { " [final]" } else { "" };
                println!("  {}{}:", state_name, marker);
                for t in &state_def.transitions {
                    let guard = t
                        .guard
                        .as_ref()
                        .map(|g| format!(" [guard: {}]", g))
                        .unwrap_or_default();
                    println!("    --{}--> {}{}", t.action, t.to, guard);
                }
            }

            if !skill.prompts.is_empty() {
                println!();
                println!("Prompts:");
                for (state, prompt) in &skill.prompts {
                    let preview: String = prompt.chars().take(80).collect();
                    println!("  {}: {}...", state, preview.trim());
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(skill)?);
        }
    }

    Ok(())
}

fn create_skill(
    name: &str,
    description: &str,
    artifact_type: Option<&str>,
    local: bool,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let base_dir = if local {
        cwd.join(".popsicle").join("skills")
    } else {
        cwd.join("skills")
    };

    let skill_dir = base_dir.join(name);
    if skill_dir.exists() {
        anyhow::bail!("Skill directory already exists: {}", skill_dir.display());
    }

    let artifact = artifact_type.unwrap_or(name);
    let template_dir = skill_dir.join("templates");
    std::fs::create_dir_all(&template_dir)?;

    let skill_yaml = format!(
        r#"name: {name}
description: {description}
version: "0.1.0"

inputs: []

artifacts:
  - type: {artifact}
    template: templates/{artifact}.md
    file_pattern: "{{slug}}.{artifact}.md"

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
  draft: |
    Create a {artifact} document.
    Follow the template structure provided.
  review: |
    Review this {artifact} document for completeness and correctness.

hooks:
  on_enter: null
  on_artifact_created: null
  on_complete: null
"#,
        name = name,
        description = description,
        artifact = artifact,
    );

    let template_md = format!(
        r#"## Overview

Describe the purpose and scope of this {artifact}.

## Details

Add detailed content here.

## Open Questions

- [ ] Question 1
"#,
        artifact = artifact,
    );

    std::fs::write(skill_dir.join("skill.yaml"), &skill_yaml)?;
    std::fs::write(template_dir.join(format!("{}.md", artifact)), &template_md)?;

    match format {
        OutputFormat::Text => {
            println!("Created skill scaffold: {}", name);
            println!("  Directory: {}", skill_dir.display());
            println!("  Files:");
            println!("    skill.yaml");
            println!("    templates/{}.md", artifact);
            println!("\nEdit skill.yaml to customize the workflow, inputs, and prompts.");
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "name": name,
                "path": skill_dir.display().to_string(),
                "artifact_type": artifact,
                "files": [
                    "skill.yaml",
                    format!("templates/{}.md", artifact),
                ],
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}
