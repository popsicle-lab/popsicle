use std::env;

use anyhow::Context;
use popsicle_core::engine::hooks::{self, HookContext, HookEvent};
use popsicle_core::helpers;
use popsicle_core::model::{Document, PipelineDef, StageState};
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
    /// Generate summary and tags for a document (for document index)
    Summarize {
        /// Document ID (if omitted, processes all unsummarized documents in the run)
        id: Option<String>,
        /// Pipeline run ID (used when no doc ID is given)
        #[arg(short, long)]
        run: Option<String>,
        /// Directly provide an LLM-generated summary (skips rule-based extraction)
        #[arg(long)]
        summary: Option<String>,
        /// Directly provide LLM-generated tags, comma-separated (skips rule-based extraction)
        #[arg(long)]
        tags: Option<String>,
        /// Output a prompt for LLM-based summarization instead of generating summary
        #[arg(long, default_value_t = false)]
        generate_prompt: bool,
    },
}

pub fn execute(cmd: DocCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        DocCommand::Create { skill, title, run } => create_doc(&skill, &title, &run, format),
        DocCommand::List { skill, status, run } => {
            list_docs(skill.as_deref(), status.as_deref(), run.as_deref(), format)
        }
        DocCommand::Show { id } => show_doc(&id, format),
        DocCommand::Summarize {
            id,
            run,
            summary,
            tags,
            generate_prompt,
        } => summarize_doc(
            id.as_deref(),
            run.as_deref(),
            summary.as_deref(),
            tags.as_deref(),
            generate_prompt,
            format,
        ),
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

    let db = IndexDb::open(&layout.db_path())?;

    // Get the topic_id from the pipeline run
    let mut pipeline_run = db
        .get_pipeline_run(run_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Pipeline run '{}' not found", run_id))?;

    // Reject document creation if the stage is Blocked
    if let Some(pipeline_def) = find_pipeline(&pipeline_run.pipeline_name)? {
        for stage in &pipeline_def.stages {
            if stage.skill_names().contains(&skill_name) {
                if let Some(StageState::Blocked) = pipeline_run.stage_states.get(&stage.name) {
                    anyhow::bail!(
                        "Stage '{}' is blocked. Complete upstream dependencies first.\n  Run `popsicle pipeline next --run {}` to see what to do.",
                        stage.name,
                        run_id
                    );
                }
                break;
            }
        }
    }

    // Verify this run holds the topic lock
    let topic_lock = db
        .get_topic_lock(&pipeline_run.topic_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    match topic_lock {
        Some(ref lock_run_id) if lock_run_id != run_id => {
            anyhow::bail!(
                "Topic is locked by a different pipeline run '{}'. This run ('{}') cannot create documents.",
                lock_run_id,
                run_id
            );
        }
        None => {
            anyhow::bail!(
                "Topic is not locked by any pipeline run. Start an issue first with `popsicle issue start`."
            );
        }
        _ => {} // lock held by this run — OK
    }

    let mut doc = Document::new(
        &artifact.artifact_type,
        title,
        skill_name,
        run_id,
        &pipeline_run.topic_id,
    );
    doc.status = "active".to_string();

    // Try to load template body
    let template_path = skill.source_dir.join(&artifact.template);
    if template_path.exists() {
        doc.body = FileStorage::read_template(&template_path)?;
    }

    let run_dir = layout.run_dir(run_id);
    let file_path = FileStorage::artifact_path(&run_dir, &artifact.file_pattern, &slug);
    doc.file_path = file_path.clone();

    FileStorage::write_document(&doc, &file_path)?;

    db.upsert_document(&doc)?;

    // Auto-transition stage: Ready → InProgress when first doc created
    if let Some(ref pipeline_def) = find_pipeline(&pipeline_run.pipeline_name)? {
        for stage in &pipeline_def.stages {
            if stage.skill_names().contains(&skill_name) {
                if let Some(StageState::Ready) = pipeline_run.stage_states.get(&stage.name) {
                    pipeline_run.stage_states
                        .insert(stage.name.clone(), StageState::InProgress);
                    pipeline_run.updated_at = chrono::Utc::now();
                    db.upsert_pipeline_run(&pipeline_run)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                }
                break;
            }
        }
    }

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

    let mut doc = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if doc.skill_name.is_empty() {
        doc.skill_name = doc_row.skill_name.clone();
    }

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

/// Build a prompt that an LLM can use to generate a high-quality summary and tags.
fn build_summarize_prompt(doc_row: &popsicle_core::storage::DocumentRow, body: &str) -> String {
    format!(
        r#"You are analyzing a technical document for indexing purposes.

Document metadata:
- Title: {}
- Type: {}
- Skill: {}

Document content:
---
{}
---

Please provide:
1. A concise summary (3-5 sentences) that captures the key decisions, requirements, or design choices in this document.
2. A list of semantic tags (5-15 keywords) that would help find this document when searching for related topics.

Respond in JSON format:
{{"summary": "...", "tags": ["tag1", "tag2", ...]}}"#,
        doc_row.title, doc_row.doc_type, doc_row.skill_name, body
    )
}

/// Find a single document row by ID.
fn find_doc_row(db: &IndexDb, doc_id: &str) -> anyhow::Result<popsicle_core::storage::DocumentRow> {
    let all = db
        .query_documents(None, None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    all.into_iter()
        .find(|d| d.id == doc_id)
        .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))
}

fn summarize_doc(
    id: Option<&str>,
    run_id: Option<&str>,
    direct_summary: Option<&str>,
    direct_tags: Option<&str>,
    generate_prompt: bool,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    // Mode 1: --generate-prompt — output a prompt for LLM-based summarization
    if generate_prompt {
        let doc_id =
            id.ok_or_else(|| anyhow::anyhow!("--generate-prompt requires a document ID"))?;
        let doc_row = find_doc_row(&db, doc_id)?;
        let doc = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let prompt = build_summarize_prompt(&doc_row, &doc.body);

        match format {
            OutputFormat::Text => println!("{}", prompt),
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "doc_id": doc_id,
                    "title": doc_row.title,
                    "prompt": prompt,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        return Ok(());
    }

    // Mode 2: --summary/--tags — directly write LLM-generated results
    if direct_summary.is_some() || direct_tags.is_some() {
        let doc_id =
            id.ok_or_else(|| anyhow::anyhow!("--summary/--tags requires a document ID"))?;
        let doc_row = find_doc_row(&db, doc_id)?;

        let summary = direct_summary.unwrap_or("").to_string();
        let tags: Vec<String> = direct_tags
            .unwrap_or("")
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        db.update_document_summary(doc_id, &summary, &tags)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        match format {
            OutputFormat::Text => {
                println!(
                    "Updated '{}' ({}): summary={} chars, tags=[{}]",
                    doc_row.title,
                    doc_id,
                    summary.len(),
                    tags.join(", ")
                );
            }
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "id": doc_id,
                    "title": doc_row.title,
                    "summary_length": summary.len(),
                    "tags": tags,
                    "status": "ok",
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        return Ok(());
    }

    // Mode 3: default (no flags) — batch generate prompts for unsummarized docs
    let target_docs: Vec<popsicle_core::storage::DocumentRow> = if let Some(doc_id) = id {
        let all = db
            .query_documents(None, None, None)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        all.into_iter().filter(|d| d.id == doc_id).collect()
    } else {
        let run = run_id.unwrap_or("default");
        db.query_documents(None, None, Some(run))
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .into_iter()
            .filter(|d| d.summary.is_empty())
            .collect()
    };

    if target_docs.is_empty() {
        match format {
            OutputFormat::Text => println!("No documents to summarize."),
            OutputFormat::Json => println!("[]"),
        }
        return Ok(());
    }

    let mut prompts = Vec::new();
    for doc_row in &target_docs {
        match FileStorage::read_document(std::path::Path::new(&doc_row.file_path)) {
            Ok(doc) => {
                let prompt = build_summarize_prompt(doc_row, &doc.body);
                prompts.push(serde_json::json!({
                    "doc_id": doc_row.id,
                    "title": doc_row.title,
                    "prompt": prompt,
                }));
            }
            Err(e) => {
                prompts.push(serde_json::json!({
                    "doc_id": doc_row.id,
                    "title": doc_row.title,
                    "error": e.to_string(),
                }));
            }
        }
    }

    match format {
        OutputFormat::Text => {
            println!("{} document(s) need LLM summarization.\n", prompts.len());
            for p in &prompts {
                let title = p["title"].as_str().unwrap_or("?");
                let doc_id = p["doc_id"].as_str().unwrap_or("?");
                if p.get("error").is_some() {
                    println!("  [ERROR] '{}' ({}): {}", title, doc_id, p["error"]);
                } else {
                    println!("  '{}' ({}):", title, doc_id);
                    println!(
                        "    popsicle doc summarize {} --generate-prompt --format json",
                        doc_id
                    );
                    println!(
                        "    popsicle doc summarize {} --summary \"...\" --tags \"...\"",
                        doc_id
                    );
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&prompts)?);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_summarize_prompt_contains_metadata() {
        let doc_row = popsicle_core::storage::DocumentRow {
            id: "doc-123".to_string(),
            doc_type: "rfc".to_string(),
            title: "JWT Authentication".to_string(),
            status: "final".to_string(),
            skill_name: "rfc-writer".to_string(),
            pipeline_run_id: "run-1".to_string(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
            summary: String::new(),
            doc_tags: "[]".to_string(),
            topic_id: "test-topic".to_string(),
            version: 1,
            parent_doc_id: None,
        };
        let body = "## Summary\nThis is a test document about JWT authentication.";

        let prompt = build_summarize_prompt(&doc_row, body);

        assert!(prompt.contains("JWT Authentication"));
        assert!(prompt.contains("rfc"));
        assert!(prompt.contains("rfc-writer"));
        assert!(prompt.contains("JWT authentication"));
        assert!(prompt.contains("\"summary\""));
        assert!(prompt.contains("\"tags\""));
    }

    #[test]
    fn test_build_summarize_prompt_json_format_instruction() {
        let doc_row = popsicle_core::storage::DocumentRow {
            id: "doc-456".to_string(),
            doc_type: "adr".to_string(),
            title: "Choose Redis for Session".to_string(),
            status: "final".to_string(),
            skill_name: "adr-writer".to_string(),
            pipeline_run_id: "run-2".to_string(),
            file_path: "test.md".to_string(),
            created_at: None,
            updated_at: None,
            summary: String::new(),
            doc_tags: "[]".to_string(),
            topic_id: "test-topic".to_string(),
            version: 1,
            parent_doc_id: None,
        };
        let body = "## Decision\nWe chose Redis for session storage.";

        let prompt = build_summarize_prompt(&doc_row, body);

        assert!(prompt.contains("Respond in JSON format"));
        assert!(prompt.contains("3-5 sentences"));
        assert!(prompt.contains("5-15 keywords"));
    }
}
