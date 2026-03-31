use std::env;

use popsicle_core::engine::extractor;
use popsicle_core::engine::guard::{check_guard, count_checkboxes};
use popsicle_core::engine::hooks::{self, HookContext, HookEvent};
use popsicle_core::engine::{Advisor, PipelineRecommender};
use popsicle_core::helpers;
use popsicle_core::model::bug::BugSource;
use popsicle_core::model::skill::ExtractionSpec;
use popsicle_core::model::testcase::TestType;
use popsicle_core::model::{IssueStatus, PipelineDef, PipelineRun, StageState};
use popsicle_core::storage::{FileStorage, IndexDb, ProjectConfig, ProjectLayout};

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
        /// Strict mode: treat unchecked checkboxes as verification failures
        #[arg(long, default_value_t = false)]
        strict: bool,
    },
    /// Review a pipeline run: evaluate all guard conditions across completed stages
    Review {
        /// Pipeline run ID (omit for latest run)
        #[arg(short, long)]
        run: Option<String>,
        /// Show all unchecked checkboxes across all documents for agent-assisted review
        #[arg(long, default_value_t = false)]
        checklist: bool,
    },
    /// Archive a completed pipeline run
    Archive {
        /// Pipeline run ID (omit for latest run)
        #[arg(short, long)]
        run: Option<String>,
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
    /// Manage pipeline stage state (start, complete)
    Stage {
        #[command(subcommand)]
        action: StageAction,
    },
    /// Force-release the spec lock held by a pipeline run
    Unlock {
        /// Spec ID to unlock (omit to unlock the spec of the latest run)
        #[arg(long)]
        spec: Option<String>,
    },
}

#[derive(clap::Subcommand)]
pub enum StageAction {
    /// Mark a stage as started (Ready → InProgress)
    Start {
        /// Stage name
        stage: String,
        /// Pipeline run ID (omit for latest run)
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Mark a stage as completed (InProgress → Completed)
    Complete {
        /// Stage name
        stage: String,
        /// Pipeline run ID (omit for latest run)
        #[arg(short, long)]
        run: Option<String>,
        /// Confirm human approval (required for stages with requires_approval)
        #[arg(long, default_value_t = false)]
        confirm: bool,
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
        PipelineCommand::Status { run } => show_status(run.as_deref(), format),
        PipelineCommand::Next { run } => show_next(run.as_deref(), format),
        PipelineCommand::Verify { run, strict } => verify_run(run.as_deref(), strict, format),
        PipelineCommand::Review { run, checklist } => {
            review_run(run.as_deref(), checklist, format)
        }
        PipelineCommand::Archive { run } => archive_run(run.as_deref(), format),
        PipelineCommand::Recommend { task } => recommend_pipeline(&task, format),
        PipelineCommand::Revise { run, stages } => {
            let stage_list: Vec<String> = stages.split(',').map(|s| s.trim().to_string()).collect();
            revise_pipeline(&run, &stage_list, format)
        }
        PipelineCommand::Stage { action } => execute_stage_action(action, format),
        PipelineCommand::Unlock { spec } => unlock_spec(spec.as_deref(), format),
    }
}

fn load_pipelines() -> anyhow::Result<Vec<PipelineDef>> {
    let cwd = env::current_dir()?;
    helpers::load_pipelines(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn verify_run(run_id: Option<&str>, strict: bool, format: &OutputFormat) -> anyhow::Result<()> {
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
                if d.status != "final" {
                    issues.push(format!(
                        "Document '{}' is '{}', not final",
                        d.title, d.status
                    ));
                }
            }
        }
    }

    let state_passed = issues.is_empty();

    // Guard evaluation (post-pipeline review)
    let guard_reports = evaluate_all_guards(&run, &pipeline_def, &db, &registry)?;
    let failed_guards: usize = guard_reports
        .iter()
        .map(|r| r.guards.iter().filter(|g| !g.passed).count())
        .sum();

    // Checklist audit: scan ALL documents for unchecked checkboxes
    let checklist_audit = audit_all_checklists(&docs);
    let total_unchecked: usize = checklist_audit.iter().map(|a| a.unchecked).sum();
    let checklist_failed = strict && total_unchecked > 0;

    match format {
        OutputFormat::Text => {
            if state_passed && failed_guards == 0 && !checklist_failed {
                println!(
                    "Pipeline run '{}' VERIFIED — all stages complete, all documents approved, all guards passed.",
                    run.title
                );
            } else {
                if !state_passed {
                    println!(
                        "Pipeline run '{}' has {} state issue(s):",
                        run.title,
                        issues.len()
                    );
                    for issue in &issues {
                        println!("  - {}", issue);
                    }
                }
                if failed_guards > 0 {
                    println!(
                        "Pipeline run '{}' has {} guard failure(s). Run `popsicle pipeline review` for details.",
                        run.title, failed_guards
                    );
                }
                if checklist_failed {
                    println!(
                        "Pipeline run '{}' has {} unchecked checklist item(s) (--strict mode).",
                        run.title, total_unchecked
                    );
                }
            }
            // Always show checklist audit if there are unchecked items
            if total_unchecked > 0 {
                println!();
                println!("Checklist audit:");
                for audit in &checklist_audit {
                    if audit.unchecked > 0 {
                        println!(
                            "  ⚠ {} — {}/{} checked ({} unchecked)",
                            audit.title,
                            audit.checked,
                            audit.checked + audit.unchecked,
                            audit.unchecked
                        );
                    }
                }
                println!();
                println!(
                    "  Run `popsicle pipeline review --checklist` to see all unchecked items."
                );
            }
        }
        OutputFormat::Json => {
            let audit_json: Vec<_> = checklist_audit
                .iter()
                .filter(|a| a.checked + a.unchecked > 0)
                .map(|a| {
                    serde_json::json!({
                        "doc_id": a.doc_id,
                        "title": a.title,
                        "skill": a.skill,
                        "checked": a.checked,
                        "unchecked": a.unchecked,
                    })
                })
                .collect();
            let result = serde_json::json!({
                "run_id": run.id,
                "title": run.title,
                "verified": state_passed && failed_guards == 0 && !checklist_failed,
                "state_passed": state_passed,
                "issues": issues,
                "failed_guards": failed_guards,
                "checklist_audit": {
                    "total_checked": checklist_audit.iter().map(|a| a.checked).sum::<usize>(),
                    "total_unchecked": total_unchecked,
                    "strict": strict,
                    "documents": audit_json,
                },
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

/// Audit all documents for checkbox completion status.
fn audit_all_checklists(
    docs: &[popsicle_core::storage::DocumentRow],
) -> Vec<ChecklistAudit> {
    let mut audits = Vec::new();
    for doc_row in docs {
        let doc = match FileStorage::read_document(std::path::Path::new(&doc_row.file_path)) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let (checked, unchecked) = count_checkboxes(&doc.body);
        if checked + unchecked > 0 {
            audits.push(ChecklistAudit {
                doc_id: doc_row.id.clone(),
                title: doc_row.title.clone(),
                skill: doc_row.skill_name.clone(),
                checked,
                unchecked,
            });
        }
    }
    audits
}

struct ChecklistAudit {
    doc_id: String,
    title: String,
    skill: String,
    checked: usize,
    unchecked: usize,
}

/// Shared guard evaluation logic used by `review` and `verify`.
///
/// Evaluates all guards on forward transitions for each completed stage's skills.
/// Returns a list of per-stage guard results.
fn evaluate_all_guards(
    run: &PipelineRun,
    pipeline_def: &PipelineDef,
    db: &IndexDb,
    registry: &popsicle_core::registry::SkillRegistry,
) -> anyhow::Result<Vec<StageGuardReport>> {
    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut reports = Vec::new();

    for stage in &pipeline_def.stages {
        let state = run.stage_states.get(&stage.name).copied();
        if !matches!(state, Some(StageState::Completed)) {
            continue;
        }

        for skill_name in stage.skill_names() {
            let skill = match registry.get(skill_name) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Find forward transitions that have guards (transitions leading to final states
            // or any transition with a guard condition).
            let guarded_transitions: Vec<_> = skill
                .workflow
                .states
                .values()
                .flat_map(|s| s.transitions.iter())
                .filter(|t| t.guard.is_some())
                .collect();

            if guarded_transitions.is_empty() {
                continue;
            }

            // Find the document for this skill in this run
            let skill_doc_row = docs.iter().find(|d| d.skill_name == skill_name);
            let doc = match skill_doc_row {
                Some(row) => {
                    match FileStorage::read_document(std::path::Path::new(&row.file_path)) {
                        Ok(d) => d,
                        Err(_) => continue,
                    }
                }
                None => continue,
            };

            let mut guard_results = Vec::new();
            for transition in &guarded_transitions {
                if let Some(ref guard_str) = transition.guard {
                    let result = check_guard(
                        guard_str,
                        &doc,
                        &docs,
                        registry,
                        Some(pipeline_def),
                        Some(run),
                    )
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                    guard_results.push(result);
                }
            }

            if !guard_results.is_empty() {
                reports.push(StageGuardReport {
                    stage_name: stage.name.clone(),
                    skill_name: skill_name.to_string(),
                    guards: guard_results,
                });
            }
        }
    }

    Ok(reports)
}

struct StageGuardReport {
    stage_name: String,
    skill_name: String,
    guards: Vec<popsicle_core::engine::GuardResult>,
}

fn review_run(run_id: Option<&str>, checklist: bool, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let run = get_run(&db, run_id)?;
    let pipeline_def = find_pipeline(&run.pipeline_name)?;
    let registry = load_registry()?;

    // --checklist mode: output all unchecked checkboxes for agent review
    if checklist {
        return review_checklist(&run, &db);
    }

    let reports = evaluate_all_guards(&run, &pipeline_def, &db, &registry)?;

    let total_guards: usize = reports.iter().map(|r| r.guards.len()).sum();
    let failed_guards: usize = reports
        .iter()
        .map(|r| r.guards.iter().filter(|g| !g.passed).count())
        .sum();
    let failed_stages: usize = reports
        .iter()
        .filter(|r| r.guards.iter().any(|g| !g.passed))
        .count();

    match format {
        OutputFormat::Text => {
            println!("Pipeline Review: {} ({})\n", run.title, run.id);
            if reports.is_empty() {
                println!("  No guards configured on any completed stage.");
            } else {
                for report in &reports {
                    println!("Stage: {} ({})", report.stage_name, report.skill_name);
                    for guard in &report.guards {
                        let icon = if guard.passed { "✅" } else { "❌" };
                        println!("  {} {}  {}", icon, guard.guard_name, guard.message);
                    }
                    println!();
                }
                if failed_guards == 0 {
                    println!(
                        "Result: All {} guards passed across {} stages.",
                        total_guards,
                        reports.len()
                    );
                } else {
                    println!(
                        "Result: {} guard(s) failed in {} stage(s). Review the documents above.",
                        failed_guards, failed_stages
                    );
                }
            }
        }
        OutputFormat::Json => {
            let stages: Vec<_> = reports
                .iter()
                .map(|r| {
                    let guards: Vec<_> = r
                        .guards
                        .iter()
                        .map(|g| {
                            serde_json::json!({
                                "guard": g.guard_name,
                                "passed": g.passed,
                                "message": g.message,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "stage": r.stage_name,
                        "skill": r.skill_name,
                        "guards": guards,
                    })
                })
                .collect();
            let result = serde_json::json!({
                "run_id": run.id,
                "title": run.title,
                "total_guards": total_guards,
                "failed_guards": failed_guards,
                "failed_stages": failed_stages,
                "stages": stages,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

/// `pipeline review --checklist` — output all unchecked checkboxes for agent-assisted review.
fn review_checklist(
    run: &PipelineRun,
    db: &IndexDb,
) -> anyhow::Result<()> {
    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut doc_checklists: Vec<serde_json::Value> = Vec::new();
    let mut total_checked = 0usize;
    let mut total_unchecked = 0usize;

    for doc_row in &docs {
        let doc = match FileStorage::read_document(std::path::Path::new(&doc_row.file_path)) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let mut unchecked_items = Vec::new();
        let mut checked_count = 0usize;
        for (line_no, line) in doc.body.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- [ ] ") {
                unchecked_items.push(serde_json::json!({
                    "line": line_no + 1,
                    "text": trimmed.trim_start_matches("- [ ] ").to_string(),
                }));
            } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
                checked_count += 1;
            }
        }

        if unchecked_items.is_empty() && checked_count == 0 {
            continue;
        }

        total_checked += checked_count;
        total_unchecked += unchecked_items.len();

        doc_checklists.push(serde_json::json!({
            "doc_id": doc_row.id,
            "title": doc_row.title,
            "skill": doc_row.skill_name,
            "file_path": doc_row.file_path,
            "checked": checked_count,
            "unchecked": unchecked_items.len(),
            "unchecked_items": unchecked_items,
        }));
    }

    let result = serde_json::json!({
        "run_id": run.id,
        "title": run.title,
        "total_checked": total_checked,
        "total_unchecked": total_unchecked,
        "documents": doc_checklists,
    });
    println!("{}", serde_json::to_string_pretty(&result)?);

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
            anyhow::anyhow!(
                "No pipeline runs found. Use `popsicle issue start <KEY>` to create one."
            )
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

    // Load all docs from the same spec for cross-run visibility
    let spec_docs = db.query_spec_documents(&run.spec_id).unwrap_or_default();

    let steps = Advisor::next_steps(&pipeline_def, &run, &registry, &docs, &spec_docs);
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

fn execute_stage_action(action: StageAction, format: &OutputFormat) -> anyhow::Result<()> {
    match action {
        StageAction::Start { stage, run } => stage_start(run.as_deref(), &stage, format),
        StageAction::Complete {
            stage,
            run,
            confirm,
        } => stage_complete(run.as_deref(), &stage, confirm, format),
    }
}

fn stage_start(
    run_id: Option<&str>,
    stage_name: &str,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut run = get_run(&db, run_id)?;
    let pipeline_def = find_pipeline(&run.pipeline_name)?;

    let _stage = pipeline_def
        .stages
        .iter()
        .find(|s| s.name == stage_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Stage '{}' not found in pipeline '{}'",
                stage_name,
                pipeline_def.name
            )
        })?;

    let current = run
        .stage_states
        .get(stage_name)
        .copied()
        .unwrap_or(StageState::Blocked);

    if current != StageState::Ready {
        anyhow::bail!(
            "Stage '{}' is '{}', can only start from 'ready'",
            stage_name,
            current
        );
    }

    run.stage_states
        .insert(stage_name.to_string(), StageState::InProgress);
    run.updated_at = chrono::Utc::now();
    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Stage '{}' → in_progress", stage_name);
            println!("  Run: {}", run.id);
        }
        OutputFormat::Json => {
            let out = serde_json::json!({
                "stage": stage_name,
                "state": "in_progress",
                "run_id": run.id,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }
    Ok(())
}

fn stage_complete(
    run_id: Option<&str>,
    stage_name: &str,
    confirmed: bool,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let config =
        ProjectConfig::load(&layout.config_path()).map_err(|e| anyhow::anyhow!("{}", e))?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut run = get_run(&db, run_id)?;
    let pipeline_def = find_pipeline(&run.pipeline_name)?;
    let registry = load_registry()?;

    let stage = pipeline_def
        .stages
        .iter()
        .find(|s| s.name == stage_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Stage '{}' not found in pipeline '{}'",
                stage_name,
                pipeline_def.name
            )
        })?;

    let current = run
        .stage_states
        .get(&stage.name)
        .copied()
        .unwrap_or(StageState::Blocked);

    if !matches!(current, StageState::Ready | StageState::InProgress) {
        anyhow::bail!(
            "Stage '{}' is '{}', can only complete from 'ready' or 'in_progress'",
            stage_name,
            current
        );
    }

    // Check requires_approval
    if stage.requires_approval && !confirmed {
        anyhow::bail!(
            "Stage '{}' requires human approval. Review all documents and re-run with --confirm:\n  popsicle pipeline stage complete {} --confirm",
            stage_name,
            stage_name
        );
    }

    // Verify docs exist for all skills in this stage
    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let missing_skills: Vec<&str> = stage
        .skill_names()
        .into_iter()
        .filter(|sn| !docs.iter().any(|d| d.skill_name == *sn))
        .collect();
    if !missing_skills.is_empty() {
        anyhow::bail!(
            "Stage '{}' cannot be completed — missing documents for skills: {}",
            stage_name,
            missing_skills.join(", ")
        );
    }

    // Warn about unchecked checkboxes before finalizing
    let stage_skills: Vec<&str> = stage.skill_names();
    for doc_row in &docs {
        if stage_skills.contains(&doc_row.skill_name.as_str())
            && let Ok(doc) =
                FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
        {
            let (checked, unchecked) = count_checkboxes(&doc.body);
            if unchecked > 0 {
                eprintln!(
                    "  ⚠ '{}' has {}/{} checklist items unchecked",
                    doc_row.title,
                    unchecked,
                    checked + unchecked
                );
            }
        }
    }

    // Mark all docs in this stage as "final"
    for doc_row in &docs {
        if stage_skills.contains(&doc_row.skill_name.as_str()) && doc_row.status != "final" {
            let _ = db.update_document_status(&doc_row.id, "final");
            // Update file on disk
            if let Ok(mut doc) =
                FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
            {
                doc.status = "final".to_string();
                doc.updated_at = Some(chrono::Utc::now());
                let _ = FileStorage::write_document(&doc, std::path::Path::new(&doc_row.file_path));
            }
        }
    }

    // Run on_complete hooks and declarative extractions for each skill
    let mut extraction_summary: Vec<String> = Vec::new();
    for skill_name in &stage_skills {
        if let Ok(skill) = registry.get(skill_name) {
            // Run on_complete hook
            for doc_row in docs.iter().filter(|d| d.skill_name == *skill_name) {
                if let Ok(doc) =
                    FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
                {
                    let ctx = HookContext::from_document(&doc, "on_complete");
                    if let Ok(Some(result)) =
                        hooks::run_hook(&skill.hooks, HookEvent::OnComplete, &ctx)
                        && !result.success
                    {
                        eprintln!(
                            "Warning: on_complete hook for '{}' failed: {}",
                            skill_name, result.stderr
                        );
                    }

                    // Declarative extractions
                    run_declarative_extractions(
                        &skill.artifacts,
                        &doc,
                        &doc_row.id,
                        &db,
                        &config,
                        &mut extraction_summary,
                    );
                }
            }
        }
    }

    // Transition stage to Completed
    run.stage_states
        .insert(stage.name.clone(), StageState::Completed);
    run.refresh_states(&pipeline_def);
    run.updated_at = chrono::Utc::now();
    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Check if all stages are done → auto-release spec lock + mark issue done
    let all_done = pipeline_def.stages.iter().all(|s| {
        matches!(
            run.stage_states.get(&s.name),
            Some(StageState::Completed | StageState::Skipped)
        )
    });

    let mut auto_released = false;
    if all_done {
        let _ = db.release_spec_lock(&run.spec_id, Some(&run.id));
        auto_released = true;

        if let Ok(Some(mut issue)) = db.find_issue_by_run_id(&run.id)
            && issue.status != IssueStatus::Done
        {
            issue.status = IssueStatus::Done;
            let _ = db.update_issue(&issue);
        }
    }

    match format {
        OutputFormat::Text => {
            println!("Stage '{}' → completed", stage_name);
            println!("  Run: {}", run.id);
            for line in &extraction_summary {
                println!("  {}", line);
            }
            if all_done {
                println!("  All stages completed!");
                if auto_released {
                    println!("  Spec lock released.");
                }
            } else {
                for s in &pipeline_def.stages {
                    if run.stage_states.get(&s.name) == Some(&StageState::Ready)
                        && s.depends_on.contains(&stage_name.to_string())
                    {
                        println!("  Unblocked: {}", s.name);
                    }
                }
            }
        }
        OutputFormat::Json => {
            let out = serde_json::json!({
                "stage": stage_name,
                "state": "completed",
                "run_id": run.id,
                "all_done": all_done,
                "auto_released": auto_released,
                "extractions": extraction_summary,
                "stage_states": run.stage_states,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }
    Ok(())
}

/// Run declarative extractions for artifacts that have `extractions` configured.
fn run_declarative_extractions(
    artifacts: &[popsicle_core::model::skill::ArtifactDef],
    doc: &popsicle_core::model::Document,
    doc_id: &str,
    db: &IndexDb,
    config: &ProjectConfig,
    summary: &mut Vec<String>,
) {
    let prefix = config.project.key_prefix_or_default();
    let run_id_opt = if doc.pipeline_run_id.is_empty() {
        None
    } else {
        Some(doc.pipeline_run_id.as_str())
    };
    let issue_id: Option<String> =
        run_id_opt.and_then(|rid| db.find_issue_by_run_id(rid).ok().flatten().map(|i| i.id));

    for artifact in artifacts {
        if artifact.artifact_type != doc.doc_type {
            continue;
        }
        for spec in &artifact.extractions {
            match spec {
                ExtractionSpec::UserStories => {
                    let extracted = extractor::extract_user_stories(doc);
                    let mut count = 0;
                    for mut story in extracted {
                        if let Ok(seq) = db.next_story_seq(prefix) {
                            story.key = format!("US-{}-{}", prefix, seq);
                            story.source_doc_id = Some(doc_id.to_string());
                            story.pipeline_run_id = run_id_opt.map(String::from);
                            story.issue_id = issue_id.clone();
                            if db.create_user_story(&story).is_ok() {
                                count += 1;
                            }
                        }
                    }
                    if count > 0 {
                        summary.push(format!("Extracted {} user stories", count));
                    }
                }
                ExtractionSpec::TestCases { test_type } => {
                    let tt: TestType = test_type.parse().unwrap_or(TestType::Unit);
                    let extracted = extractor::extract_test_cases(doc, tt);
                    let mut count = 0;
                    for mut tc in extracted {
                        if let Ok(seq) = db.next_testcase_seq(prefix) {
                            tc.key = format!("TC-{}-{}", prefix, seq);
                            tc.source_doc_id = Some(doc_id.to_string());
                            tc.pipeline_run_id = run_id_opt.map(String::from);
                            tc.issue_id = issue_id.clone();
                            if db.create_test_case(&tc).is_ok() {
                                count += 1;
                            }
                        }
                    }
                    if count > 0 {
                        summary.push(format!("Extracted {} test cases ({})", count, test_type));
                    }
                }
                ExtractionSpec::Bugs => {
                    let extracted = extractor::extract_bugs(doc);
                    let mut count = 0;
                    for mut bug in extracted {
                        if let Ok(seq) = db.next_bug_seq(prefix) {
                            bug.key = format!("BUG-{}-{}", prefix, seq);
                            bug.source = BugSource::DocExtracted;
                            bug.pipeline_run_id = run_id_opt.map(String::from);
                            bug.issue_id = issue_id.clone();
                            if db.create_bug(&bug).is_ok() {
                                count += 1;
                            }
                        }
                    }
                    if count > 0 {
                        summary.push(format!("Extracted {} bugs", count));
                    }
                }
            }
        }
    }
}

fn unlock_spec(spec_id: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let sid = match spec_id {
        Some(t) => t.to_string(),
        None => {
            let run = get_run(&db, None)?;
            run.spec_id.clone()
        }
    };

    let spec = db
        .get_spec(&sid)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Spec not found: {}", sid))?;

    match &spec.locked_by_run_id {
        Some(run_id) => {
            db.release_spec_lock(&sid, None)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            match format {
                OutputFormat::Text => {
                    println!(
                        "Unlocked spec '{}' (was locked by run '{}')",
                        spec.name, run_id
                    );
                }
                OutputFormat::Json => {
                    let out = serde_json::json!({
                        "spec_id": sid,
                        "spec_name": spec.name,
                        "released_from": run_id,
                    });
                    println!("{}", serde_json::to_string_pretty(&out)?);
                }
            }
        }
        None => match format {
            OutputFormat::Text => println!("Spec '{}' is not locked.", spec.name),
            OutputFormat::Json => {
                let out = serde_json::json!({
                    "spec_id": sid,
                    "locked": false,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            }
        },
    }
    Ok(())
}
