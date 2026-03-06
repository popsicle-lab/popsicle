use std::env;

use anyhow::Context;
use popsicle_core::engine::Advisor;
use popsicle_core::model::{PipelineDef, PipelineRun, StageState};
use popsicle_core::registry::{PipelineLoader, SkillLoader, SkillRegistry};
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum PipelineCommand {
    /// List available pipeline templates
    List,
    /// Create a new custom pipeline template
    Create {
        /// Pipeline name
        name: String,
        /// Short description
        #[arg(short, long, default_value = "A custom pipeline")]
        description: String,
        /// Create in project-local .popsicle/pipelines/ instead of workspace pipelines/
        #[arg(long)]
        local: bool,
    },
    /// Start a new pipeline run
    Run {
        /// Pipeline template name
        pipeline: String,
        /// Title for this pipeline run
        #[arg(short, long)]
        title: String,
    },
    /// Show status of a pipeline run
    Status {
        /// Pipeline run ID (omit to show the latest run)
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Show next recommended steps for a pipeline run
    Next {
        /// Pipeline run ID (omit for latest run)
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Verify a pipeline run: check all stages completed and reviews passed
    Verify {
        /// Pipeline run ID (omit for latest run)
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Archive a completed pipeline run
    Archive {
        /// Pipeline run ID (omit for latest run)
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Quick path: create a single-stage pipeline for small changes
    Quick {
        /// Title for this quick change
        #[arg(short, long)]
        title: String,
        /// Skill to use (defaults to implementation)
        #[arg(short, long, default_value = "implementation")]
        skill: String,
    },
}

pub fn execute(cmd: PipelineCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        PipelineCommand::List => list_pipelines(format),
        PipelineCommand::Create {
            name,
            description,
            local,
        } => create_pipeline(&name, &description, local, format),
        PipelineCommand::Run { pipeline, title } => run_pipeline(&pipeline, &title, format),
        PipelineCommand::Status { run } => show_status(run.as_deref(), format),
        PipelineCommand::Next { run } => show_next(run.as_deref(), format),
        PipelineCommand::Verify { run } => verify_run(run.as_deref(), format),
        PipelineCommand::Archive { run } => archive_run(run.as_deref(), format),
        PipelineCommand::Quick { title, skill } => quick_run(&title, &skill, format),
    }
}

fn load_pipelines() -> anyhow::Result<Vec<PipelineDef>> {
    let cwd = env::current_dir()?;
    let mut all = Vec::new();

    let workspace_pipelines = cwd.join("pipelines");
    if workspace_pipelines.is_dir() {
        all.extend(
            PipelineLoader::load_dir(&workspace_pipelines)
                .context("Loading pipeline templates")?,
        );
    }

    let local_pipelines = cwd.join(".popsicle").join("pipelines");
    if local_pipelines.is_dir() {
        all.extend(
            PipelineLoader::load_dir(&local_pipelines).context("Loading local pipelines")?,
        );
    }

    Ok(all)
}

fn verify_run(run_id: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let run = get_run(&db, run_id)?;
    let pipeline_def = find_pipeline(&run.pipeline_name)?;
    let registry = load_registry()?;

    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut issues = Vec::new();

    for stage in &pipeline_def.stages {
        let state = run.stage_states.get(&stage.name);
        if !matches!(state, Some(StageState::Completed) | Some(StageState::Skipped)) {
            issues.push(format!("Stage '{}' is {} (not completed)", stage.name, state.map(|s| s.to_string()).unwrap_or("unknown".into())));
        }

        for skill_name in stage.skill_names() {
            let skill_docs: Vec<_> = docs.iter().filter(|d| d.skill_name == skill_name).collect();
            if skill_docs.is_empty() {
                issues.push(format!("No documents for skill '{}'", skill_name));
            }
            for d in &skill_docs {
                if let Ok(skill) = registry.get(&d.skill_name) {
                    if !skill.is_final_state(&d.status) {
                        issues.push(format!("Document '{}' is '{}', not final", d.title, d.status));
                    }
                }
            }
        }
    }

    let passed = issues.is_empty();

    match format {
        OutputFormat::Text => {
            if passed {
                println!("Pipeline run '{}' VERIFIED — all stages complete, all documents approved.", run.title);
            } else {
                println!("Pipeline run '{}' has {} issue(s):", run.title, issues.len());
                for issue in &issues {
                    println!("  - {}", issue);
                }
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "run_id": run.id,
                "title": run.title,
                "verified": passed,
                "issues": issues,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn archive_run(run_id: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut run = get_run(&db, run_id)?;
    let pipeline_def = find_pipeline(&run.pipeline_name)?;

    let all_done = pipeline_def.stages.iter().all(|s| {
        matches!(
            run.stage_states.get(&s.name),
            Some(StageState::Completed) | Some(StageState::Skipped)
        )
    });

    if !all_done {
        anyhow::bail!(
            "Cannot archive: not all stages are completed. Run `popsicle pipeline verify` to check."
        );
    }

    let archive_dir = layout.artifacts_dir().join("_archived");
    std::fs::create_dir_all(&archive_dir)?;

    let run_dir = layout.run_dir(&run.id);
    if run_dir.is_dir() {
        let dest = archive_dir.join(&run.id);
        std::fs::rename(&run_dir, &dest)?;
    }

    for stage in &pipeline_def.stages {
        run.stage_states
            .insert(stage.name.clone(), StageState::Skipped);
    }
    run.updated_at = chrono::Utc::now();
    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Archived pipeline run: {} ({})", run.title, run.id);
            println!("  Artifacts moved to: .popsicle/artifacts/_archived/{}", run.id);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "run_id": run.id,
                "title": run.title,
                "status": "archived",
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn quick_run(title: &str, skill_name: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let registry = load_registry()?;

    registry
        .get(skill_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let quick_def = PipelineDef {
        name: "quick".to_string(),
        description: "Quick single-stage change".to_string(),
        stages: vec![popsicle_core::model::StageDef {
            name: "work".to_string(),
            skills: vec![],
            skill: Some(skill_name.to_string()),
            description: format!("Quick: {}", title),
            depends_on: vec![],
        }],
    };

    let run = PipelineRun::new(&quick_def, title);
    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir)?;

    let db = IndexDb::open(&layout.db_path())?;
    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Quick pipeline run: {}", run.id);
            println!("  Title: {}", title);
            println!("  Skill: {}", skill_name);
            println!("\nCreate a document:");
            println!(
                "  $ popsicle doc create {} --title \"{}\" --run {}",
                skill_name, title, run.id
            );
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": run.id,
                "title": title,
                "pipeline": "quick",
                "skill": skill_name,
                "cli_command": format!("popsicle doc create {} --title \"{}\" --run {}", skill_name, title, run.id),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn create_pipeline(
    name: &str,
    description: &str,
    local: bool,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let base_dir = if local {
        let dir = cwd.join(".popsicle").join("pipelines");
        std::fs::create_dir_all(&dir)?;
        dir
    } else {
        let dir = cwd.join("pipelines");
        std::fs::create_dir_all(&dir)?;
        dir
    };

    let file_path = base_dir.join(format!("{}.pipeline.yaml", name));
    if file_path.exists() {
        anyhow::bail!("Pipeline template already exists: {}", file_path.display());
    }

    let registry = load_registry()?;
    let skills = registry.list();
    let skill_names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();

    let content = format!(
        r#"name: {name}
description: {description}

# Available skills: {available}
# Customize stages below. Each stage runs one or more skills.
# Use `depends_on` to define execution order.

stages:
  - name: stage-1
    skill: domain-analysis
    description: First stage
    # depends_on: []

  # - name: stage-2
  #   skill: product-prd
  #   description: Second stage
  #   depends_on: [stage-1]

  # - name: stage-3
  #   skills:
  #     - tech-rfc
  #     - tech-adr
  #   description: Parallel skills in one stage
  #   depends_on: [stage-2]
"#,
        name = name,
        description = description,
        available = skill_names.join(", "),
    );

    std::fs::write(&file_path, &content)?;

    match format {
        OutputFormat::Text => {
            println!("Created pipeline template: {}", name);
            println!("  File: {}", file_path.display());
            println!("  Available skills: {}", skill_names.join(", "));
            println!("\nEdit the file to define your pipeline stages.");
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "name": name,
                "path": file_path.display().to_string(),
                "available_skills": skill_names,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn find_pipeline(name: &str) -> anyhow::Result<PipelineDef> {
    let pipelines = load_pipelines()?;
    pipelines
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Pipeline template not found: {}", name))
}

fn load_registry() -> anyhow::Result<SkillRegistry> {
    let mut registry = SkillRegistry::new();
    let cwd = env::current_dir()?;

    let workspace_skills = cwd.join("skills");
    if workspace_skills.is_dir() {
        SkillLoader::load_dir(&workspace_skills, &mut registry)?;
    }

    let local_skills = cwd.join(".popsicle").join("skills");
    if local_skills.is_dir() {
        SkillLoader::load_dir(&local_skills, &mut registry)?;
    }

    Ok(registry)
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(layout)
}

fn get_run(db: &IndexDb, run_id: Option<&str>) -> anyhow::Result<PipelineRun> {
    if let Some(id) = run_id {
        db.get_pipeline_run(id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Pipeline run not found: {}", id))
    } else {
        let runs = db
            .list_pipeline_runs()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let latest = runs
            .first()
            .ok_or_else(|| anyhow::anyhow!("No pipeline runs found. Use `popsicle pipeline run` to start one."))?;
        db.get_pipeline_run(&latest.id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Pipeline run not found: {}", latest.id))
    }
}

fn list_pipelines(format: &OutputFormat) -> anyhow::Result<()> {
    let pipelines = load_pipelines()?;

    match format {
        OutputFormat::Text => {
            if pipelines.is_empty() {
                println!("No pipeline templates found.");
                return Ok(());
            }
            println!("{:<25} {:<6} {}", "NAME", "STAGES", "DESCRIPTION");
            println!("{}", "-".repeat(70));
            for p in &pipelines {
                println!("{:<25} {:<6} {}", p.name, p.stages.len(), p.description);
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = pipelines
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "description": p.description,
                        "stages": p.stages.iter().map(|s| {
                            serde_json::json!({
                                "name": s.name,
                                "skills": s.skill_names(),
                                "description": s.description,
                                "depends_on": s.depends_on,
                            })
                        }).collect::<Vec<_>>(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }

    Ok(())
}

fn run_pipeline(pipeline_name: &str, title: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let pipeline_def = find_pipeline(pipeline_name)?;
    pipeline_def
        .validate()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let run = PipelineRun::new(&pipeline_def, title);
    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir)?;

    let db = IndexDb::open(&layout.db_path())?;
    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Started pipeline run: {}", run.id);
            println!("  Pipeline: {}", pipeline_def.name);
            println!("  Title: {}", title);
            println!("  Stages:");
            for stage in &pipeline_def.stages {
                let state = run.stage_states.get(&stage.name).unwrap();
                println!(
                    "    {:<20} {:<12} [{}]",
                    stage.name,
                    state,
                    stage.skill_names().join(", ")
                );
            }
            println!("\nUse `popsicle pipeline next` to see what to do first.");
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": run.id,
                "pipeline": pipeline_def.name,
                "title": title,
                "stage_states": run.stage_states,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn show_status(run_id: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let run = get_run(&db, run_id)?;
    let pipeline_def = find_pipeline(&run.pipeline_name)?;

    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Pipeline Run: {} ({})", run.title, run.id);
            println!("Pipeline: {}", run.pipeline_name);
            println!("Created: {}", run.created_at.format("%Y-%m-%d %H:%M"));
            println!();
            println!("{:<20} {:<14} {}", "STAGE", "STATUS", "SKILLS / DOCUMENTS");
            println!("{}", "-".repeat(75));

            for stage in &pipeline_def.stages {
                let state = run
                    .stage_states
                    .get(&stage.name)
                    .unwrap_or(&StageState::Blocked);
                let skills_str = stage.skill_names().join(", ");
                println!("{:<20} {:<14} {}", stage.name, state, skills_str);

                let stage_docs: Vec<_> = docs
                    .iter()
                    .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
                    .collect();
                for doc in stage_docs {
                    println!(
                        "  └─ {} [{}] ({})",
                        doc.title, doc.status, doc.doc_type
                    );
                }
            }
        }
        OutputFormat::Json => {
            let stages: Vec<_> = pipeline_def
                .stages
                .iter()
                .map(|stage| {
                    let state = run.stage_states.get(&stage.name);
                    let stage_docs: Vec<_> = docs
                        .iter()
                        .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
                        .map(|d| {
                            serde_json::json!({
                                "id": d.id,
                                "title": d.title,
                                "doc_type": d.doc_type,
                                "status": d.status,
                                "skill_name": d.skill_name,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "name": stage.name,
                        "state": state,
                        "skills": stage.skill_names(),
                        "documents": stage_docs,
                    })
                })
                .collect();

            let result = serde_json::json!({
                "id": run.id,
                "pipeline": run.pipeline_name,
                "title": run.title,
                "stages": stages,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn show_next(run_id: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let run = get_run(&db, run_id)?;
    let pipeline_def = find_pipeline(&run.pipeline_name)?;
    let registry = load_registry()?;

    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let steps = Advisor::next_steps(&pipeline_def, &run, &registry, &docs);

    match format {
        OutputFormat::Text => {
            let actionable: Vec<_> = steps.iter().filter(|s| s.action != "blocked").collect();
            let blocked: Vec<_> = steps.iter().filter(|s| s.action == "blocked").collect();

            if actionable.is_empty() && blocked.is_empty() {
                println!("All stages completed!");
                return Ok(());
            }

            if !actionable.is_empty() {
                println!("=== Next Steps ===\n");
                for (i, step) in actionable.iter().enumerate() {
                    println!("{}. [{}] {} → {}", i + 1, step.stage, step.skill, step.action);
                    println!("   {}", step.description);
                    println!("   $ {}", step.cli_command);
                    if let Some(prompt) = &step.prompt {
                        let preview: String = prompt.chars().take(100).collect();
                        println!("   AI Prompt: {}...", preview.trim());
                    }
                    println!();
                }
            }

            if !blocked.is_empty() {
                println!("=== Blocked ===\n");
                for step in &blocked {
                    println!(
                        "  [{}] {} — {}",
                        step.stage, step.skill, step.description
                    );
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&steps)?);
        }
    }

    Ok(())
}
