use std::env;

use popsicle_core::engine::extractor;
use popsicle_core::helpers;
use popsicle_core::model::{WorkItem, WorkItemKind};
use popsicle_core::storage::{FileStorage, IndexDb, ProjectConfig, ProjectLayout};
use serde_json::json;

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum ExtractCommand {
    /// Extract user stories from a PRD document
    UserStories {
        #[arg(long)]
        from_doc: String,
    },
    /// Extract test cases from a test-spec document
    TestCases {
        #[arg(long)]
        from_doc: String,
        #[arg(short = 't', long = "type", default_value = "unit")]
        test_type: String,
    },
    /// Extract bugs from a bug-report document
    Bugs {
        #[arg(long)]
        from_doc: String,
    },
}

pub fn execute(cmd: ExtractCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        ExtractCommand::UserStories { from_doc } => {
            run(WorkItemKind::Story, &from_doc, None, format)
        }
        ExtractCommand::TestCases {
            from_doc,
            test_type,
        } => run(WorkItemKind::TestCase, &from_doc, Some(test_type), format),
        ExtractCommand::Bugs { from_doc } => run(WorkItemKind::Bug, &from_doc, None, format),
    }
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn load_config(layout: &ProjectLayout) -> anyhow::Result<ProjectConfig> {
    ProjectConfig::load(&layout.config_path()).map_err(|e| anyhow::anyhow!("{}", e))
}

fn resolve_issue_id(db: &IndexDb, run_id: Option<&str>) -> Option<String> {
    run_id.and_then(|rid| db.find_issue_by_run_id(rid).ok().flatten().map(|i| i.id))
}

fn run(
    kind: WorkItemKind,
    doc_id: &str,
    test_type: Option<String>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let config = load_config(&layout)?;
    let db = IndexDb::open(&layout.db_path())?;

    let doc_row = db
        .query_documents(None, None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .into_iter()
        .find(|d| d.id == doc_id)
        .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

    let doc = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let prefix = config.project.key_prefix_or_default();
    let run_id_opt = if doc.pipeline_run_id.is_empty() {
        None
    } else {
        Some(doc.pipeline_run_id.as_str())
    };
    let issue_id = resolve_issue_id(&db, run_id_opt);

    let extracted: Vec<WorkItem> = match kind {
        WorkItemKind::Story => extractor::extract_user_stories(&doc),
        WorkItemKind::TestCase => {
            extractor::extract_test_cases(&doc, test_type.as_deref().unwrap_or("unit"))
        }
        WorkItemKind::Bug => {
            let mut bugs = extractor::extract_bugs(&doc);
            for b in &mut bugs {
                b.set_field("source", json!("doc_extracted"));
            }
            bugs
        }
    };

    let mut created = Vec::new();
    for mut item in extracted {
        let seq = db
            .next_work_item_seq(kind, prefix)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        item.key = format!("{}-{}-{}", kind.key_prefix(), prefix, seq);
        item.source_doc_id = Some(doc_id.to_string());
        item.pipeline_run_id = run_id_opt.map(String::from);
        item.issue_id = issue_id.clone();
        db.create_work_item(&item)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        created.push(item);
    }

    match format {
        OutputFormat::Text => {
            println!(
                "Extracted {} {} from document {}",
                created.len(),
                kind.as_str(),
                doc_id
            );
            for i in &created {
                println!("  {} — {}", i.key, i.title);
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = created
                .iter()
                .map(|i| {
                    json!({
                        "key": i.key,
                        "title": i.title,
                        "fields": i.fields,
                    })
                })
                .collect();
            let result =
                json!({ "extracted": created.len(), "kind": kind.as_str(), "items": items });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
