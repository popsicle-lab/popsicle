use anyhow::{Context, Result};
use clap::Subcommand;

use popsicle_core::model::Topic;
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

fn project_layout() -> Result<ProjectLayout> {
    let cwd = std::env::current_dir()?;
    popsicle_core::helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

#[derive(Subcommand)]
pub enum TopicCommand {
    /// Create a new topic
    Create {
        /// Topic name (e.g. "jwt-migration")
        name: String,
        /// Short description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Comma-separated tags
        #[arg(short, long, default_value = "")]
        tags: String,
        /// Project this topic belongs to (name or ID) — required
        #[arg(long)]
        project: String,
    },
    /// List all topics
    List {
        /// Filter by project (name or ID)
        #[arg(long)]
        project: Option<String>,
    },
    /// Show details of a topic
    Show {
        /// Topic name or ID
        name: String,
    },
    /// Delete a topic
    Delete {
        /// Topic name or ID
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

pub fn execute(cmd: TopicCommand, format: &OutputFormat) -> Result<()> {
    match cmd {
        TopicCommand::Create {
            name,
            description,
            tags,
            project,
        } => create_topic(&name, &description, &tags, &project, format),
        TopicCommand::List { project } => list_topics(project.as_deref(), format),
        TopicCommand::Show { name } => show_topic(&name, format),
        TopicCommand::Delete { name, force } => delete_topic(&name, force, format),
    }
}

fn resolve_project_id(db: &IndexDb, name_or_id: &str) -> Result<String> {
    if let Some(p) = db.find_project_by_name(name_or_id).ok().flatten() {
        return Ok(p.id);
    }
    if let Some(p) = db.get_project(name_or_id).ok().flatten() {
        return Ok(p.id);
    }
    anyhow::bail!("Project not found: {}", name_or_id)
}

fn create_topic(
    name: &str,
    description: &str,
    tags: &str,
    project: &str,
    format: &OutputFormat,
) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let tag_list: Vec<String> = if tags.is_empty() {
        vec![]
    } else {
        tags.split(',').map(|t| t.trim().to_string()).collect()
    };

    let project_id = resolve_project_id(&db, project)?;

    let mut topic = Topic::new(name.to_string(), description.to_string(), project_id.clone());
    topic.tags = tag_list;
    db.create_topic(&topic).context("Failed to create topic")?;

    match format {
        OutputFormat::Text => {
            println!("✅ Topic created: {} ({})", topic.name, topic.slug);
            println!("   ID: {}", topic.id);
            println!("   Project: {}", project_id);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": topic.id,
                    "name": topic.name,
                    "slug": topic.slug,
                    "description": topic.description,
                    "tags": topic.tags,
                    "project_id": topic.project_id,
                }))?,
            );
        }
    }
    Ok(())
}

fn list_topics(project: Option<&str>, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let project_id = match project {
        Some(p) => Some(resolve_project_id(&db, p)?),
        None => None,
    };

    let topics = db
        .list_topics_by_project(project_id.as_deref())
        .or_else(|_| db.list_topics())
        .context("Failed to list topics")?;

    match format {
        OutputFormat::Text => {
            if topics.is_empty() {
                println!("No topics found.");
                return Ok(());
            }
            println!("{:<36}  {:<24}  {:<20}  TAGS", "ID", "NAME", "SLUG");
            println!("{}", "-".repeat(100));
            for t in &topics {
                let tags = if t.tags.is_empty() {
                    String::new()
                } else {
                    t.tags.join(", ")
                };
                println!("{:<36}  {:<24}  {:<20}  {}", t.id, t.name, t.slug, tags);
            }
            println!("\n{} topic(s)", topics.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = topics
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "slug": t.slug,
                        "description": t.description,
                        "tags": t.tags,
                        "created_at": t.created_at.to_rfc3339(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

fn show_topic(name: &str, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let topic = db
        .find_topic_by_name(name)
        .context("Failed to look up topic")?
        .or_else(|| db.get_topic(name).ok().flatten())
        .ok_or_else(|| anyhow::anyhow!("Topic not found: {}", name))?;

    let runs = db.list_topic_runs(&topic.id).unwrap_or_default();
    let docs = db.query_topic_documents(&topic.id).unwrap_or_default();
    let issues = db.query_issues(None, None, None, Some(&topic.id)).unwrap_or_default();

    // Resolve project name if present
    let project_name = if topic.project_id.is_empty() {
        None
    } else {
        db.get_project(&topic.project_id).ok().flatten().map(|p| p.name)
    };

    match format {
        OutputFormat::Text => {
            println!("Topic: {}", topic.name);
            println!("  ID:          {}", topic.id);
            println!("  Slug:        {}", topic.slug);
            println!("  Description: {}", topic.description);
            if let Some(ref pname) = project_name {
                println!("  Project:     {}", pname);
            }
            if !topic.tags.is_empty() {
                println!("  Tags:        {}", topic.tags.join(", "));
            }
            println!(
                "  Created:     {}",
                topic.created_at.format("%Y-%m-%d %H:%M")
            );
            println!();

            if issues.is_empty() {
                println!("  No issues.");
            } else {
                println!("  Issues ({}):", issues.len());
                for i in &issues {
                    println!(
                        "    • {} [{}] {} ({})",
                        i.key, i.issue_type, i.title, i.status
                    );
                }
            }
            println!();

            if runs.is_empty() {
                println!("  No pipeline runs.");
            } else {
                println!("  Pipeline Runs ({}):", runs.len());
                for r in &runs {
                    println!(
                        "    • {} [{}] {} ({})",
                        &r.id[..8],
                        r.run_type,
                        r.title,
                        r.pipeline_name
                    );
                }
            }
            println!();

            if docs.is_empty() {
                println!("  No documents.");
            } else {
                println!("  Documents ({}):", docs.len());
                for d in &docs {
                    println!(
                        "    • {} [v{}] {} ({})",
                        d.doc_type, d.version, d.title, d.status
                    );
                }
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": topic.id,
                    "name": topic.name,
                    "slug": topic.slug,
                    "description": topic.description,
                    "project_id": topic.project_id,
                    "project_name": project_name,
                    "tags": topic.tags,
                    "created_at": topic.created_at.to_rfc3339(),
                    "issues": issues.iter().map(|i| serde_json::json!({
                        "key": i.key,
                        "title": i.title,
                        "type": i.issue_type.to_string(),
                        "status": i.status.to_string(),
                    })).collect::<Vec<_>>(),
                    "runs": runs.iter().map(|r| serde_json::json!({
                        "id": r.id,
                        "title": r.title,
                        "pipeline": r.pipeline_name,
                        "run_type": r.run_type.to_string(),
                    })).collect::<Vec<_>>(),
                    "documents": docs.iter().map(|d| serde_json::json!({
                        "id": d.id,
                        "type": d.doc_type,
                        "title": d.title,
                        "version": d.version,
                        "status": d.status,
                    })).collect::<Vec<_>>(),
                }))?,
            );
        }
    }
    Ok(())
}

fn delete_topic(name: &str, force: bool, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let topic = db
        .find_topic_by_name(name)
        .context("Failed to look up topic")?
        .or_else(|| db.get_topic(name).ok().flatten())
        .ok_or_else(|| anyhow::anyhow!("Topic not found: {}", name))?;

    if !force {
        let runs = db.list_topic_runs(&topic.id).unwrap_or_default();
        if !runs.is_empty() {
            anyhow::bail!(
                "Topic '{}' has {} pipeline run(s). Use --force to delete anyway.",
                topic.name,
                runs.len()
            );
        }
    }

    db.delete_topic(&topic.id)
        .context("Failed to delete topic")?;

    match format {
        OutputFormat::Text => println!("🗑  Topic deleted: {}", topic.name),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"deleted": topic.id, "name": topic.name})
            );
        }
    }
    Ok(())
}
