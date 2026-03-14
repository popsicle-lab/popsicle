use std::env;

use popsicle_core::engine::extractor;
use popsicle_core::helpers;
use popsicle_core::model::bug::BugSource;
use popsicle_core::model::testcase::TestType;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectConfig, ProjectLayout};

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
        ExtractCommand::UserStories { from_doc } => extract_user_stories(&from_doc, format),
        ExtractCommand::TestCases {
            from_doc,
            test_type,
        } => extract_test_cases(&from_doc, &test_type, format),
        ExtractCommand::Bugs { from_doc } => extract_bugs(&from_doc, format),
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

fn extract_user_stories(doc_id: &str, format: &OutputFormat) -> anyhow::Result<()> {
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

    let extracted = extractor::extract_user_stories(&doc);
    let prefix = config.project.key_prefix_or_default();
    let run_id_opt = if doc.pipeline_run_id.is_empty() {
        None
    } else {
        Some(doc.pipeline_run_id.as_str())
    };
    let issue_id = resolve_issue_id(&db, run_id_opt);

    let mut created = Vec::new();
    for mut story in extracted {
        let seq = db
            .next_story_seq(prefix)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        story.key = format!("US-{}-{}", prefix, seq);
        story.source_doc_id = Some(doc_id.to_string());
        story.pipeline_run_id = run_id_opt.map(String::from);
        story.issue_id = issue_id.clone();
        db.create_user_story(&story)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        created.push(story);
    }

    match format {
        OutputFormat::Text => {
            println!(
                "Extracted {} user stories from document {}",
                created.len(),
                doc_id
            );
            for s in &created {
                println!("  {} — {}", s.key, s.title);
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = created
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "key": s.key, "title": s.title, "ac_count": s.acceptance_criteria.len(),
                    })
                })
                .collect();
            let result = serde_json::json!({ "extracted": created.len(), "user_stories": items });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn extract_test_cases(doc_id: &str, type_str: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let test_type: TestType = type_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;
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

    let extracted = extractor::extract_test_cases(&doc, test_type);
    let prefix = config.project.key_prefix_or_default();
    let run_id_opt = if doc.pipeline_run_id.is_empty() {
        None
    } else {
        Some(doc.pipeline_run_id.as_str())
    };
    let issue_id = resolve_issue_id(&db, run_id_opt);

    let mut created = Vec::new();
    for mut tc in extracted {
        let seq = db
            .next_testcase_seq(prefix)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        tc.key = format!("TC-{}-{}", prefix, seq);
        tc.source_doc_id = Some(doc_id.to_string());
        tc.pipeline_run_id = run_id_opt.map(String::from);
        tc.issue_id = issue_id.clone();
        db.create_test_case(&tc)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        created.push(tc);
    }

    match format {
        OutputFormat::Text => {
            println!(
                "Extracted {} test cases from document {}",
                created.len(),
                doc_id
            );
            for tc in &created {
                println!("  {} — {}", tc.key, tc.title);
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = created.iter().map(|tc| serde_json::json!({
                "key": tc.key, "title": tc.title,
                "test_type": tc.test_type.to_string(), "priority_level": tc.priority_level.to_string(),
            })).collect();
            let result = serde_json::json!({ "extracted": created.len(), "test_cases": items });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn extract_bugs(doc_id: &str, format: &OutputFormat) -> anyhow::Result<()> {
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

    let extracted = extractor::extract_bugs(&doc);
    let prefix = config.project.key_prefix_or_default();
    let run_id_opt = if doc.pipeline_run_id.is_empty() {
        None
    } else {
        Some(doc.pipeline_run_id.as_str())
    };
    let issue_id = resolve_issue_id(&db, run_id_opt);

    let mut created = Vec::new();
    for mut bug in extracted {
        let seq = db
            .next_bug_seq(prefix)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        bug.key = format!("BUG-{}-{}", prefix, seq);
        bug.source = BugSource::DocExtracted;
        bug.pipeline_run_id = run_id_opt.map(String::from);
        bug.issue_id = issue_id.clone();
        db.create_bug(&bug).map_err(|e| anyhow::anyhow!("{}", e))?;
        created.push(bug);
    }

    match format {
        OutputFormat::Text => {
            println!("Extracted {} bugs from document {}", created.len(), doc_id);
            for b in &created {
                println!("  {} — {}", b.key, b.title);
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = created
                .iter()
                .map(|b| {
                    serde_json::json!({
                        "key": b.key, "title": b.title, "severity": b.severity.to_string(),
                    })
                })
                .collect();
            let result = serde_json::json!({ "extracted": created.len(), "bugs": items });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
