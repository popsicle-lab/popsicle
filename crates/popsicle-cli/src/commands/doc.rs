use std::env;

use anyhow::Context;
use popsicle_core::engine::guard;
use popsicle_core::engine::hooks::{self, HookContext, HookEvent};
use popsicle_core::helpers;
use popsicle_core::model::{Document, IssueStatus, PipelineDef, StageState};
use popsicle_core::registry::SkillRegistry;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum DocCommand {
    /// Create a new document from a Skill's template
    Create {
        /// Skill name to use
        skill: String,
        /// Document title
        #[arg(short, long)]
        title: String,
        /// Pipeline run ID
        #[arg(short, long, default_value = "default")]
        run: String,
    },
    /// List documents
    List {
        /// Filter by skill name
        #[arg(short, long)]
        skill: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by pipeline run ID
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Show a document's content and metadata
    Show {
        /// Document ID
        id: String,
    },
    /// Transition a document to a new state via a workflow action
    Transition {
        /// Document ID
        id: String,
        /// Action name (e.g., submit, approve, revise)
        action: String,
        /// Confirm human approval (required for transitions with requires_approval)
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },
}

pub fn execute(cmd: DocCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        DocCommand::Create { skill, title, run } => create_doc(&skill, &title, &run, format),
        DocCommand::List { skill, status, run } => {
            list_docs(skill.as_deref(), status.as_deref(), run.as_deref(), format)
        }
        DocCommand::Show { id } => show_doc(&id, format),
        DocCommand::Transition {
            id,
            action,
            confirm,
        } => transition_doc(&id, &action, confirm, format),
    }
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn load_registry() -> anyhow::Result<popsicle_core::registry::SkillRegistry> {
    let cwd = env::current_dir()?;
    helpers::load_registry(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn create_doc(
    skill_name: &str,
    title: &str,
    run_id: &str,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let registry = load_registry()?;
    let skill = registry
        .get(skill_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let artifact = skill.artifacts.first().context("Skill has no artifacts")?;

    let slug = title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string();

    let mut doc = Document::new(&artifact.artifact_type, title, skill_name, run_id);
    doc.status = skill.workflow.initial.clone();

    // Try to load template body
    let template_path = skill.source_dir.join(&artifact.template);
    if template_path.exists() {
        doc.body = FileStorage::read_template(&template_path)?;
    }

    let run_dir = layout.run_dir(run_id);
    let file_path = FileStorage::artifact_path(&run_dir, &artifact.file_pattern, &slug);
    doc.file_path = file_path.clone();

    FileStorage::write_document(&doc, &file_path)?;

    let db = IndexDb::open(&layout.db_path())?;
    db.upsert_document(&doc)?;

    let hook_ctx = HookContext::from_document(&doc, "artifact_created");
    let hook_result = hooks::run_hook(&skill.hooks, HookEvent::OnArtifactCreated, &hook_ctx)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Created document: {}", doc.id);
            println!("  Type: {}", doc.doc_type);
            println!("  Title: {}", doc.title);
            println!("  Status: {}", doc.status);
            println!("  File: {}", file_path.display());
            if let Some(ref result) = hook_result {
                println!(
                    "  Hook [{}]: {}",
                    result.event,
                    if result.success { "ok" } else { "failed" }
                );
                if !result.stdout.trim().is_empty() {
                    println!("    {}", result.stdout.trim());
                }
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": doc.id,
                "doc_type": doc.doc_type,
                "title": doc.title,
                "status": doc.status,
                "skill": skill_name,
                "file_path": file_path.display().to_string(),
                "hook": hook_result.as_ref().map(|r| serde_json::json!({
                    "event": r.event.to_string(),
                    "success": r.success,
                    "stdout": r.stdout.trim(),
                })),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn list_docs(
    skill: Option<&str>,
    status: Option<&str>,
    run: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let docs = db
        .query_documents(skill, status, run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            if docs.is_empty() {
                println!("No documents found.");
                return Ok(());
            }
            println!(
                "{:<38} {:<15} {:<15} {:<12} TITLE",
                "ID", "TYPE", "SKILL", "STATUS"
            );
            println!("{}", "-".repeat(95));
            for doc in &docs {
                println!(
                    "{:<38} {:<15} {:<15} {:<12} {}",
                    doc.id, doc.doc_type, doc.skill_name, doc.status, doc.title
                );
            }
            println!("\n{} document(s).", docs.len());
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&docs)?);
        }
    }

    Ok(())
}

fn show_doc(id: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let docs = db
        .query_documents(None, None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let doc_row = docs
        .iter()
        .find(|d| d.id == id)
        .context(format!("Document not found: {}", id))?;

    let doc = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("=== {} ===", doc.title);
            println!("ID: {}", doc.id);
            println!("Type: {}", doc.doc_type);
            println!("Skill: {}", doc.skill_name);
            println!("Status: {}", doc.status);
            println!("Pipeline Run: {}", doc.pipeline_run_id);
            if !doc.tags.is_empty() {
                println!("Tags: {}", doc.tags.join(", "));
            }
            println!("File: {}", doc_row.file_path);
            println!("{}", "-".repeat(60));
            println!("{}", doc.body);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": doc.id,
                "doc_type": doc.doc_type,
                "title": doc.title,
                "status": doc.status,
                "skill_name": doc.skill_name,
                "pipeline_run_id": doc.pipeline_run_id,
                "tags": doc.tags,
                "metadata": doc.metadata,
                "body": doc.body,
                "file_path": doc_row.file_path,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn find_pipeline(name: &str) -> anyhow::Result<Option<PipelineDef>> {
    let cwd = env::current_dir()?;
    match helpers::find_pipeline(&cwd, name) {
        Ok(p) => Ok(Some(p)),
        Err(popsicle_core::error::PopsicleError::Storage(_)) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
}

/// After a document reaches a final state, check if the Pipeline Stage
/// should be updated (Ready → InProgress, or InProgress → Completed).
fn sync_pipeline_stage(
    db: &IndexDb,
    doc: &Document,
    skill_is_final: bool,
    registry: &SkillRegistry,
) -> anyhow::Result<Option<String>> {
    let run_id = &doc.pipeline_run_id;
    let mut run = match db
        .get_pipeline_run(run_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?
    {
        Some(r) => r,
        None => return Ok(None),
    };

    let pipeline_def = match find_pipeline(&run.pipeline_name)? {
        Some(p) => p,
        None => return Ok(None),
    };

    let stage = pipeline_def
        .stages
        .iter()
        .find(|s| s.skill_names().contains(&doc.skill_name.as_str()));

    let stage = match stage {
        Some(s) => s,
        None => return Ok(None),
    };

    let current_state = run
        .stage_states
        .get(&stage.name)
        .copied()
        .unwrap_or(StageState::Blocked);

    if current_state == StageState::Ready {
        run.stage_states
            .insert(stage.name.clone(), StageState::InProgress);
    }

    if skill_is_final && current_state != StageState::Completed {
        let all_docs = db
            .query_documents(None, None, Some(run_id))
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let all_skills_done = stage.skill_names().iter().all(|skill_name| {
            let skill_docs: Vec<_> = all_docs
                .iter()
                .filter(|d| &d.skill_name == skill_name)
                .collect();

            if skill_docs.is_empty() {
                return false;
            }

            skill_docs.iter().all(|d| {
                registry
                    .get(&d.skill_name)
                    .map(|s| s.is_final_state(&d.status))
                    .unwrap_or(false)
            })
        });

        if all_skills_done {
            run.stage_states
                .insert(stage.name.clone(), StageState::Completed);
            run.refresh_states(&pipeline_def);

            let all_pipeline_done = pipeline_def.stages.iter().all(|s| {
                matches!(
                    run.stage_states.get(&s.name),
                    Some(StageState::Completed | StageState::Skipped)
                )
            });
            if all_pipeline_done {
                if let Ok(Some(mut issue)) = db.find_issue_by_run_id(&run.id) {
                    if issue.status != IssueStatus::Done {
                        issue.status = IssueStatus::Done;
                        let _ = db.update_issue(&issue);
                    }
                }
            }
        }
    }

    run.updated_at = chrono::Utc::now();
    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let new_state = run.stage_states.get(&stage.name).copied();
    Ok(new_state.map(|s| format!("Stage '{}' → {}", stage.name, s)))
}

fn transition_doc(
    id: &str,
    action: &str,
    confirmed: bool,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let registry = load_registry()?;
    let db = IndexDb::open(&layout.db_path())?;

    let docs = db
        .query_documents(None, None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let doc_row = docs
        .iter()
        .find(|d| d.id == id)
        .context(format!("Document not found: {}", id))?;

    let mut doc = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let skill = registry
        .get(&doc.skill_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let transition = skill
        .available_actions(&doc.status)
        .into_iter()
        .find(|t| t.action == action)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Action '{}' not available from state '{}'",
                action,
                doc.status
            )
        })?;

    if transition.requires_approval && !confirmed {
        anyhow::bail!(
            "Action '{}' on '{}' requires human approval. Review the document and re-run with --confirm:\n  popsicle doc transition {} {} --confirm",
            action,
            doc.title,
            id,
            action
        );
    }

    if let Some(ref guard_expr) = transition.guard {
        let all_docs = db
            .query_documents(None, None, Some(&doc.pipeline_run_id))
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let guard_result = guard::check_guard(guard_expr, &doc, &all_docs, &registry)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if !guard_result.passed {
            anyhow::bail!(
                "Guard '{}' failed: {}",
                guard_result.guard_name,
                guard_result.message
            );
        }
    }

    let new_state = skill
        .try_transition(&doc.status, action)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let old_status = doc.status.clone();
    let is_final = skill.is_final_state(&new_state);
    doc.status = new_state.clone();
    doc.updated_at = Some(chrono::Utc::now());

    FileStorage::write_document(&doc, std::path::Path::new(&doc_row.file_path))?;
    db.upsert_document(&doc)?;

    let stage_update = sync_pipeline_stage(&db, &doc, is_final, &registry)?;

    let hook_ctx = HookContext::from_document(&doc, if is_final { "complete" } else { "enter" });
    let hook_event = if is_final {
        HookEvent::OnComplete
    } else {
        HookEvent::OnEnter
    };
    let hook_result = hooks::run_hook(&skill.hooks, hook_event, &hook_ctx)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!(
                "Transitioned document '{}': {} --{}--> {}",
                doc.title, old_status, action, new_state
            );
            if let Some(stage_msg) = &stage_update {
                println!("  {}", stage_msg);
            }
            if let Some(ref result) = hook_result {
                println!(
                    "  Hook [{}]: {}",
                    result.event,
                    if result.success { "ok" } else { "failed" }
                );
                if !result.stdout.trim().is_empty() {
                    println!("    {}", result.stdout.trim());
                }
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": doc.id,
                "action": action,
                "from": old_status,
                "to": new_state,
                "title": doc.title,
                "is_final": is_final,
                "stage_update": stage_update,
                "hook": hook_result.as_ref().map(|r| serde_json::json!({
                    "event": r.event.to_string(),
                    "success": r.success,
                    "stdout": r.stdout.trim(),
                })),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}
