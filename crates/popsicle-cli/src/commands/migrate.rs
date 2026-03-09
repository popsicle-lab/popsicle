use std::env;

use popsicle_core::model::Document;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Args)]
pub struct MigrateArgs {
    /// Pipeline run ID to import documents into (defaults to latest)
    #[arg(short, long)]
    run: Option<String>,

    /// Skill name to assign to discovered documents
    #[arg(short, long, default_value = "domain-analysis")]
    skill: String,

    /// Directories to scan for existing Markdown documents
    #[arg(required = true)]
    paths: Vec<String>,
}

pub fn execute(args: MigrateArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let db = IndexDb::open(&layout.db_path())?;

    let run_id = match args.run {
        Some(r) => r,
        None => {
            let runs = db
                .list_pipeline_runs()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            runs.first().map(|r| r.id.clone()).ok_or_else(|| {
                anyhow::anyhow!("No pipeline runs found. Start one with `popsicle pipeline run`.")
            })?
        }
    };

    let mut imported = Vec::new();

    for path_str in &args.paths {
        let scan_path = cwd.join(path_str);
        if scan_path.is_file() && scan_path.extension().is_some_and(|e| e == "md") {
            if let Some(name) = import_file(&scan_path, &args.skill, &run_id, &layout, &db)? {
                imported.push(name);
            }
        } else if scan_path.is_dir() {
            let files =
                FileStorage::list_documents(&scan_path).map_err(|e| anyhow::anyhow!("{}", e))?;
            for file in files {
                if let Some(name) = import_file(&file, &args.skill, &run_id, &layout, &db)? {
                    imported.push(name);
                }
            }
        }
    }

    match format {
        OutputFormat::Text => {
            if imported.is_empty() {
                println!("No Markdown documents found to import.");
            } else {
                println!("Imported {} document(s):", imported.len());
                for name in &imported {
                    println!("  {}", name);
                }
                println!(
                    "\nDocuments added to pipeline run {} as '{}' skill artifacts.",
                    &run_id[..8],
                    args.skill
                );
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "imported": imported,
                "count": imported.len(),
                "pipeline_run_id": run_id,
                "skill": args.skill,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn import_file(
    file_path: &std::path::Path,
    skill: &str,
    run_id: &str,
    layout: &ProjectLayout,
    db: &IndexDb,
) -> anyhow::Result<Option<String>> {
    let content = std::fs::read_to_string(file_path)?;

    let title = extract_title(&content).unwrap_or_else(|| {
        file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string()
    });

    let slug = title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string();

    let mut doc = Document::new("imported", &title, skill, run_id);
    doc.status = "approved".to_string();
    doc.body = content;

    let dest = layout.run_dir(run_id).join(format!("{}.imported.md", slug));
    doc.file_path = dest.clone();

    FileStorage::write_document(&doc, &dest).map_err(|e| anyhow::anyhow!("{}", e))?;
    db.upsert_document(&doc)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(Some(format!("{} ← {}", title, file_path.display())))
}

fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}
