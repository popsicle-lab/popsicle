use std::env;
use std::io::Read as _;

use popsicle_core::engine::markdown::upsert_section;
use popsicle_core::helpers;
use popsicle_core::scanner::ProjectScanner;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum ContextCommand {
    /// Output the full context of a pipeline run (for AI agents)
    Show(ShowArgs),

    /// Scan the project and generate a technical profile at .popsicle/project-context.md
    Scan(ScanArgs),

    /// Update a section in project-context.md (for agent-driven deep analysis)
    Update(UpdateArgs),

    /// Search documents across all pipeline runs using full-text search
    Search(SearchArgs),
}

#[derive(clap::Args)]
pub struct ShowArgs {
    /// Pipeline run ID (omit for latest run)
    #[arg(short, long)]
    run: Option<String>,
    /// Filter to a specific stage
    #[arg(short, long)]
    stage: Option<String>,
}

#[derive(clap::Args)]
pub struct ScanArgs {
    /// Overwrite existing project-context.md
    #[arg(long)]
    force: bool,
}

#[derive(clap::Args)]
pub struct SearchArgs {
    /// Search query (FTS5 syntax supported)
    query: String,
    /// Filter by document status (e.g. approved, accepted)
    #[arg(long)]
    status: Option<String>,
    /// Filter by skill name
    #[arg(long)]
    skill: Option<String>,
    /// Exclude documents from this pipeline run
    #[arg(long)]
    exclude_run: Option<String>,
    /// Maximum number of results
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}

#[derive(clap::Args)]
pub struct UpdateArgs {
    /// H2 section name to update (e.g. "Architecture Patterns")
    #[arg(long)]
    section: String,
    /// Content to write (reads from stdin if omitted)
    #[arg(long)]
    content: Option<String>,
    /// Append to existing section instead of replacing
    #[arg(long)]
    append: bool,
}

pub fn execute(cmd: ContextCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        ContextCommand::Show(args) => execute_show(args, format),
        ContextCommand::Scan(args) => execute_scan(args, format),
        ContextCommand::Update(args) => execute_update(args, format),
        ContextCommand::Search(args) => execute_search(args, format),
    }
}

// ── show (original context command) ──

fn execute_show(args: ShowArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let db = IndexDb::open(&layout.db_path())?;

    let run = if let Some(ref id) = args.run {
        db.get_pipeline_run(id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Pipeline run not found: {}", id))?
    } else {
        let runs = db
            .list_pipeline_runs()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let latest = runs
            .first()
            .ok_or_else(|| anyhow::anyhow!("No pipeline runs found."))?;
        db.get_pipeline_run(&latest.id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Pipeline run not found"))?
    };

    let pipeline_def = find_pipeline(&run.pipeline_name)?;
    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("=== Context: {} ===", run.title);
            println!("Pipeline: {} | Run: {}", run.pipeline_name, run.id);
            println!();

            for stage in &pipeline_def.stages {
                if let Some(ref filter) = args.stage
                    && &stage.name != filter
                {
                    continue;
                }

                let state = run.stage_states.get(&stage.name);
                println!(
                    "--- Stage: {} ({}) ---",
                    stage.name,
                    state.map(|s| s.to_string()).unwrap_or_default()
                );

                let stage_docs: Vec<_> = docs
                    .iter()
                    .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
                    .collect();

                if stage_docs.is_empty() {
                    println!("  (no documents yet)\n");
                    continue;
                }

                for doc_row in stage_docs {
                    println!(
                        "\n  ## {} [{}] ({})",
                        doc_row.title, doc_row.status, doc_row.doc_type
                    );
                    println!("  ID: {} | Skill: {}", doc_row.id, doc_row.skill_name);

                    if let Ok(doc) =
                        FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
                    {
                        println!();
                        for line in doc.body.lines() {
                            println!("  {}", line);
                        }
                    }
                    println!();
                }
            }
        }
        OutputFormat::Json => {
            let mut stages_json = Vec::new();

            for stage in &pipeline_def.stages {
                if let Some(ref filter) = args.stage
                    && &stage.name != filter
                {
                    continue;
                }

                let state = run.stage_states.get(&stage.name);
                let stage_docs: Vec<_> = docs
                    .iter()
                    .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
                    .collect();

                let docs_json: Vec<_> = stage_docs
                    .iter()
                    .map(|d| {
                        let body = FileStorage::read_document(std::path::Path::new(&d.file_path))
                            .map(|doc| doc.body)
                            .unwrap_or_default();
                        serde_json::json!({
                            "id": d.id,
                            "doc_type": d.doc_type,
                            "title": d.title,
                            "status": d.status,
                            "skill_name": d.skill_name,
                            "body": body,
                        })
                    })
                    .collect();

                stages_json.push(serde_json::json!({
                    "stage": stage.name,
                    "state": state,
                    "documents": docs_json,
                }));
            }

            let result = serde_json::json!({
                "run_id": run.id,
                "pipeline": run.pipeline_name,
                "title": run.title,
                "stages": stages_json,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── scan ──

fn execute_scan(args: ScanArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let context_path = layout.project_context_path();

    if context_path.exists() && !args.force {
        match format {
            OutputFormat::Text => {
                println!(
                    "Project context already exists at {}",
                    context_path.display()
                );
                println!("Use --force to overwrite.");
            }
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "status": "skipped",
                    "path": context_path.display().to_string(),
                    "reason": "file already exists, use --force to overwrite",
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        return Ok(());
    }

    let scanner = ProjectScanner::new(&cwd);
    let content = scanner.scan();
    std::fs::write(&context_path, &content)?;

    match format {
        OutputFormat::Text => {
            println!("Project context written to {}", context_path.display());
            println!();
            println!("{}", content.trim());
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "path": context_path.display().to_string(),
                "content": content,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── update ──

fn execute_update(args: UpdateArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    let context_path = layout.project_context_path();

    if !context_path.exists() {
        anyhow::bail!("project-context.md does not exist. Run `popsicle context scan` first.");
    }

    let content = match args.content {
        Some(c) => c,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let doc = std::fs::read_to_string(&context_path)?;
    let updated = upsert_section(&doc, &args.section, content.trim(), args.append);
    std::fs::write(&context_path, &updated)?;

    match format {
        OutputFormat::Text => {
            let verb = if args.append {
                "appended to"
            } else {
                "updated"
            };
            println!(
                "Section \"{}\" {} in {}",
                args.section,
                verb,
                context_path.display()
            );
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "section": args.section,
                "action": if args.append { "append" } else { "replace" },
                "path": context_path.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── search ──

fn execute_search(args: SearchArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let db = IndexDb::open(&layout.db_path())?;
    let results = db
        .search_documents(
            &args.query,
            args.status.as_deref(),
            args.skill.as_deref(),
            args.exclude_run.as_deref(),
            args.limit,
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            if results.is_empty() {
                println!("No documents found matching '{}'.", args.query);
                return Ok(());
            }
            println!(
                "{:<38} {:<12} {:<15} {:<12} TITLE",
                "ID", "TYPE", "SKILL", "STATUS"
            );
            println!("{}", "-".repeat(100));
            for (doc, rank) in &results {
                println!(
                    "{:<38} {:<12} {:<15} {:<12} {} (score: {:.2})",
                    doc.id, doc.doc_type, doc.skill_name, doc.status, doc.title, rank
                );
                if !doc.summary.is_empty() {
                    let preview: String = doc
                        .summary
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(80)
                        .collect();
                    println!("  {}", preview);
                }
            }
            println!("\n{} result(s).", results.len());
        }
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|(doc, rank)| {
                    serde_json::json!({
                        "id": doc.id,
                        "doc_type": doc.doc_type,
                        "title": doc.title,
                        "status": doc.status,
                        "skill_name": doc.skill_name,
                        "pipeline_run_id": doc.pipeline_run_id,
                        "file_path": doc.file_path,
                        "summary": doc.summary,
                        "doc_tags": doc.doc_tags,
                        "bm25_score": rank,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
    }

    Ok(())
}

fn find_pipeline(name: &str) -> anyhow::Result<popsicle_core::model::PipelineDef> {
    let cwd = env::current_dir()?;
    helpers::find_pipeline(&cwd, name).map_err(|e| anyhow::anyhow!("{}", e))
}
