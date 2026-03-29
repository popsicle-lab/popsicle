use std::env;

use popsicle_core::engine::{Advisor, PipelineRecommender};
use popsicle_core::helpers;
use popsicle_core::model::{PipelineDef, PipelineRun, StageState, Topic};
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
        /// Topic name (groups related runs; auto-created if not exists)
        #[arg(long)]
        topic: Option<String>,
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
    /// Recommend the best pipeline for a task based on its description
    Recommend {
        /// Task description (e.g. "add user authentication feature")
        task: String,
    },
    /// Create a revision run from a completed pipeline run
    Revise {
        /// Pipeline run ID to revise
        #[arg(short, long)]
        run: String,
        /// Comma-separated list of stage names to revise
        #[arg(short, long)]
        stages: String,
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
        PipelineCommand::Run {
            pipeline,
            title,
            topic,
        } => run_pipeline(&pipeline, &title, topic.as_deref(), format),
        PipelineCommand::Status { run } => show_status(run.as_deref(), format),
        PipelineCommand::Next { run } => show_next(run.as_deref(), format),
        PipelineCommand::Verify { run } => verify_run(run.as_deref(), format),
        PipelineCommand::Archive { run } => archive_run(run.as_deref(), format),
        PipelineCommand::Quick { title, skill } => quick_run(&title, &skill, format),
        PipelineCommand::Recommend { task } => recommend_pipeline(&task, format),
        PipelineCommand::Revise { run, stages } => {
            let stage_list: Vec<String> = stages.split(',').map(|s| s.trim().to_string()).collect();
            revise_pipeline(&run, &stage_list, format)
        }
    }
}

fn load_pipelines() -> anyhow::Result<Vec<PipelineDef>> {
    let cwd = env::current_dir()?;
    helpers::load_pipelines(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
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
        if !matches!(
            state,
            Some(StageState::Completed) | Some(StageState::Skipped)
        ) {
            issues.push(format!(
                "Stage '{}' is {} (not completed)",
                stage.name,
                state.map(|s| s.to_string()).unwrap_or("unknown".into())
            ));
        }

        for skill_name in stage.skill_names() {
            let skill_docs: Vec<_> = docs.iter().filter(|d| d.skill_name == skill_name).collect();
            if skill_docs.is_empty() {
                issues.push(format!("No documents for skill '{}'", skill_name));
            }
            for d in &skill_docs {
                if let Ok(skill) = registry.get(&d.skill_name)
                    && !skill.is_final_state(&d.status)
                {
                    issues.push(format!(
                        "Document '{}' is '{}', not final",
                        d.title, d.status
                    ));
                }
            }
        }
    }

    let passed = issues.is_empty();

    match format {
        OutputFormat::Text => {
            if passed {
                println!(
                    "Pipeline run '{}' VERIFIED — all stages complete, all documents approved.",
                    run.title
                );
            } else {
                println!(
                    "Pipeline run '{}' has {} issue(s):",
                    run.title,
                    issues.len()
                );
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
            println!(
                "  Artifacts moved to: .popsicle/artifacts/_archived/{}",
                run.id
            );
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
        keywords: vec![],
        scale: None,
    };

    let db = IndexDb::open(&layout.db_path())?;
    let topic = resolve_or_create_topic(&db, title)?;
    let run = PipelineRun::new(&quick_def, title, &topic.id);
    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir)?;

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

fn recommend_pipeline(task: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let pipelines = load_pipelines()?;

    if pipelines.is_empty() {
        anyhow::bail!("No pipeline templates found. Run `popsicle init` first.");
    }

    let rec = PipelineRecommender::recommend(task, &pipelines);

    match format {
        OutputFormat::Text => {
            println!("=== Pipeline Recommendation ===\n");
            println!("  Task:      {}", task);
            println!("  Pipeline:  {} (scale: {})", rec.pipeline_name, rec.scale);
            println!("  Reason:    {}", rec.reason);
            println!("\n  Start with:");
            println!("  $ {}", rec.cli_command);

            if !rec.alternatives.is_empty() {
                println!("\n  Alternatives:");
                for alt in &rec.alternatives {
                    println!(
                        "    - {} (scale: {}) — {}",
                        alt.pipeline_name, alt.scale, alt.reason
                    );
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&rec)?);
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
    let cwd = env::current_dir()?;
    helpers::find_pipeline(&cwd, name).map_err(|e| anyhow::anyhow!("{}", e))
}

fn load_registry() -> anyhow::Result<popsicle_core::registry::SkillRegistry> {
    let cwd = env::current_dir()?;
    helpers::load_registry(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
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
        let latest = runs.first().ok_or_else(|| {
            anyhow::anyhow!("No pipeline runs found. Use `popsicle pipeline run` to start one.")
        })?;
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
            println!("{:<25} {:<6} DESCRIPTION", "NAME", "STAGES");
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

fn run_pipeline(
    pipeline_name: &str,
    title: &str,
    topic_name: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let pipeline_def = find_pipeline(pipeline_name)?;
    pipeline_def
        .validate()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let db = IndexDb::open(&layout.db_path())?;
    let topic = resolve_or_create_topic(&db, topic_name.unwrap_or(title))?;
    let run = PipelineRun::new(&pipeline_def, title, &topic.id);
    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir)?;

    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Started pipeline run: {}", run.id);
            println!("  Pipeline: {}", pipeline_def.name);
            println!("  Title: {}", title);
            println!("  Stages:");
            for stage in &pipeline_def.stages {
                let state = run
                    .stage_states
                    .get(&stage.name)
                    .ok_or_else(|| anyhow::anyhow!("Missing state for stage '{}'", stage.name))?;
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
            println!("{:<20} {:<14} SKILLS / DOCUMENTS", "STAGE", "STATUS");
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
                    println!("  └─ {} [{}] ({})", doc.title, doc.status, doc.doc_type);
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
    let hints = collect_context_hints(&layout);

    match format {
        OutputFormat::Text => {
            for hint in &hints {
                println!("hint: {}", hint);
            }
            if !hints.is_empty() {
                println!();
            }

            let actionable: Vec<_> = steps.iter().filter(|s| s.action != "blocked").collect();
            let blocked: Vec<_> = steps.iter().filter(|s| s.action == "blocked").collect();

            if actionable.is_empty() && blocked.is_empty() {
                println!("All stages completed!");
                return Ok(());
            }

            if !actionable.is_empty() {
                println!("=== Next Steps ===\n");
                for (i, step) in actionable.iter().enumerate() {
                    let approval_tag = if step.requires_approval {
                        " ⚠ REQUIRES APPROVAL"
                    } else {
                        ""
                    };
                    println!(
                        "{}. [{}] {} → {}{}",
                        i + 1,
                        step.stage,
                        step.skill,
                        step.action,
                        approval_tag
                    );
                    println!("   {}", step.description);
                    if let Some(ctx_cmd) = &step.context_command {
                        println!("   Step 1 — get enriched prompt (with historical references):");
                        println!("   $ {}", ctx_cmd);
                        println!("   Step 2 — create document:");
                        println!("   $ {}", step.cli_command);
                    } else if step.requires_approval {
                        println!(
                            "   → 此步骤需您本人审批，请先审阅文档/参与讨论，勿由 AI 代执行。"
                        );
                        println!("   → 确认后由您本人在终端执行：");
                        println!("   $ {} --confirm", step.cli_command);
                    } else {
                        println!("   $ {}", step.cli_command);
                    }
                    for hint in &step.hints {
                        println!("   💡 {}", hint);
                    }
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
                    println!("  [{}] {} — {}", step.stage, step.skill, step.description);
                }
            }
        }
        OutputFormat::Json => {
            let mut result = serde_json::to_value(&steps)?;
            if !hints.is_empty() {
                let wrapper = serde_json::json!({
                    "hints": hints,
                    "steps": result,
                });
                result = wrapper;
            }
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn collect_context_hints(layout: &ProjectLayout) -> Vec<String> {
    let mut hints = Vec::new();
    let context_path = layout.project_context_path();

    if !context_path.exists() {
        hints.push("Run 'popsicle context scan' to generate project technical profile".to_string());
    } else if let Ok(content) = std::fs::read_to_string(&context_path) {
        let deep_sections = [
            "## Architecture Patterns",
            "## Coding Conventions",
            "## Testing Patterns",
        ];
        let has_deep = deep_sections.iter().any(|s| content.contains(s));
        if !has_deep {
            hints.push(
                "Project context lacks deep analysis. Consider running the popsicle-context-scan skill".to_string(),
            );
        }
    }

    hints
}

/// Resolve an existing topic by name, or create a new one.
fn resolve_or_create_topic(db: &IndexDb, name: &str) -> anyhow::Result<Topic> {
    if let Some(topic) = db
        .find_topic_by_name(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?
    {
        return Ok(topic);
    }
    let topic = Topic::new(name, "");
    db.create_topic(&topic)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(topic)
}

fn revise_pipeline(
    run_id: &str,
    revised_stages: &[String],
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let parent_run = db
        .get_pipeline_run(run_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Pipeline run '{}' not found", run_id))?;

    let pipeline_def = find_pipeline(&parent_run.pipeline_name)?;

    // Validate that all specified stages exist in the pipeline
    for stage_name in revised_stages {
        if !pipeline_def.stages.iter().any(|s| &s.name == stage_name) {
            anyhow::bail!(
                "Stage '{}' not found in pipeline '{}'",
                stage_name,
                pipeline_def.name
            );
        }
    }

    let mut revision = PipelineRun::new_revision(&pipeline_def, &parent_run, revised_stages);
    revision.refresh_states(&pipeline_def);

    let run_dir = layout.run_dir(&revision.id);
    std::fs::create_dir_all(&run_dir)?;

    db.upsert_pipeline_run(&revision)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create revision documents for skills in revised stages
    let docs = db
        .query_documents(None, None, Some(run_id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let revised_skills: Vec<String> = pipeline_def
        .stages
        .iter()
        .filter(|s| revised_stages.contains(&s.name))
        .flat_map(|s| s.skill_names())
        .map(|s| s.to_string())
        .collect();

    for doc_row in &docs {
        if revised_skills.contains(&doc_row.skill_name) {
            // Load the full document and create a revision
            let full_doc = popsicle_core::model::Document::from_file_content(
                &std::fs::read_to_string(&doc_row.file_path)?,
                std::path::PathBuf::from(&doc_row.file_path),
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            let rev_doc = full_doc.new_revision(&revision.id);
            let dest = run_dir.join(
                std::path::Path::new(&doc_row.file_path)
                    .file_name()
                    .unwrap_or_default(),
            );
            popsicle_core::storage::FileStorage::write_document(&rev_doc, &dest)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            db.upsert_document(&rev_doc)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }

    match format {
        OutputFormat::Text => {
            println!("Created revision run: {}", revision.id);
            println!("  Parent: {}", run_id);
            println!("  Revised stages: {}", revised_stages.join(", "));
            for stage in &pipeline_def.stages {
                let state = revision
                    .stage_states
                    .get(&stage.name)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                println!("  [{}] {}", state, stage.name);
            }
        }
        OutputFormat::Json => {
            let out = serde_json::json!({
                "id": revision.id,
                "parent_run_id": run_id,
                "run_type": "revision",
                "revised_stages": revised_stages,
                "stage_states": revision.stage_states,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }

    Ok(())
}
