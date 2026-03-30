use anyhow::{Context, Result};
use clap::Subcommand;

use popsicle_core::model::Project;
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

fn project_layout() -> Result<ProjectLayout> {
    let cwd = std::env::current_dir()?;
    popsicle_core::helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

#[derive(Subcommand)]
pub enum ProjectCommand {
    /// Create a new project
    Create {
        /// Project name (human-readable)
        name: String,
        /// Short description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Comma-separated tags
        #[arg(short, long, default_value = "")]
        tags: String,
    },
    /// List all projects
    List {
        /// Filter by status (active, completed, archived)
        #[arg(long)]
        status: Option<String>,
    },
    /// Show project details with associated topics
    Show {
        /// Project name or ID
        name: String,
    },
    /// Update a project
    Update {
        /// Project name or ID
        name: String,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// New status (active, completed, archived)
        #[arg(short, long)]
        status: Option<String>,
        /// New comma-separated tags
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// Delete a project
    Delete {
        /// Project name or ID
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

pub fn execute(cmd: ProjectCommand, format: &OutputFormat) -> Result<()> {
    match cmd {
        ProjectCommand::Create {
            name,
            description,
            tags,
        } => create_project(&name, &description, &tags, format),
        ProjectCommand::List { status } => list_projects(status.as_deref(), format),
        ProjectCommand::Show { name } => show_project(&name, format),
        ProjectCommand::Update {
            name,
            description,
            status,
            tags,
        } => update_project(&name, description, status, tags, format),
        ProjectCommand::Delete { name, force } => delete_project(&name, force, format),
    }
}

fn resolve_project(db: &IndexDb, name_or_id: &str) -> Result<Project> {
    db.find_project_by_name(name_or_id)
        .ok()
        .flatten()
        .or_else(|| db.get_project(name_or_id).ok().flatten())
        .ok_or_else(|| anyhow::anyhow!("Project not found: {}", name_or_id))
}

fn create_project(name: &str, description: &str, tags: &str, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let tag_list: Vec<String> = if tags.is_empty() {
        vec![]
    } else {
        tags.split(',').map(|t| t.trim().to_string()).collect()
    };

    let mut project = Project::new(name.to_string(), description.to_string());
    project.tags = tag_list;
    db.create_project(&project)
        .context("Failed to create project")?;

    match format {
        OutputFormat::Text => {
            println!("✅ Project created: {} ({})", project.name, project.slug);
            println!("   ID: {}", project.id);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": project.id,
                    "name": project.name,
                    "slug": project.slug,
                    "description": project.description,
                    "status": project.status.to_string(),
                    "tags": project.tags,
                }))?,
            );
        }
    }
    Ok(())
}

fn list_projects(status_filter: Option<&str>, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let projects = db
        .list_projects(status_filter)
        .context("Failed to list projects")?;

    match format {
        OutputFormat::Text => {
            if projects.is_empty() {
                println!("No projects found.");
                return Ok(());
            }
            println!(
                "{:<36}  {:<20}  {:<10}  DESCRIPTION",
                "ID", "NAME", "STATUS"
            );
            println!("{}", "-".repeat(100));
            for p in &projects {
                let desc = if p.description.len() > 40 {
                    format!("{}...", &p.description[..37])
                } else {
                    p.description.clone()
                };
                println!(
                    "{:<36}  {:<20}  {:<10}  {}",
                    p.id, p.name, p.status, desc
                );
            }
            println!("\n{} project(s)", projects.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = projects
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "slug": p.slug,
                        "description": p.description,
                        "status": p.status.to_string(),
                        "tags": p.tags,
                        "created_at": p.created_at.to_rfc3339(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

fn show_project(name: &str, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let project = resolve_project(&db, name)?;

    let topics = db
        .list_topics_by_project(Some(&project.id))
        .or_else(|_| db.list_topics())
        .unwrap_or_default()
        .into_iter()
        .filter(|t| t.project_id == project.id)
        .collect::<Vec<_>>();

    match format {
        OutputFormat::Text => {
            println!("Project: {}", project.name);
            println!("  ID:          {}", project.id);
            println!("  Slug:        {}", project.slug);
            println!("  Status:      {}", project.status);
            println!("  Description: {}", project.description);
            if !project.tags.is_empty() {
                println!("  Tags:        {}", project.tags.join(", "));
            }
            println!(
                "  Created:     {}",
                project.created_at.format("%Y-%m-%d %H:%M")
            );
            println!();

            if topics.is_empty() {
                println!("  No topics.");
            } else {
                println!("  Topics ({}):", topics.len());
                for t in &topics {
                    let tag_str = if t.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", t.tags.join(", "))
                    };
                    println!("    • {} — {}{}", t.name, t.description, tag_str);
                }
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": project.id,
                    "name": project.name,
                    "slug": project.slug,
                    "description": project.description,
                    "status": project.status.to_string(),
                    "tags": project.tags,
                    "created_at": project.created_at.to_rfc3339(),
                    "topics": topics.iter().map(|t| serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "slug": t.slug,
                        "description": t.description,
                    })).collect::<Vec<_>>(),
                }))?,
            );
        }
    }
    Ok(())
}

fn update_project(
    name: &str,
    description: Option<String>,
    status: Option<String>,
    tags: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut project = resolve_project(&db, name)?;

    if let Some(desc) = description {
        project.description = desc;
    }
    if let Some(s) = status {
        project.status = s
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid status: {}. Use active/completed/archived", s))?;
    }
    if let Some(t) = tags {
        project.tags = if t.is_empty() {
            vec![]
        } else {
            t.split(',').map(|s| s.trim().to_string()).collect()
        };
    }
    project.updated_at = chrono::Utc::now();

    db.update_project(&project)
        .context("Failed to update project")?;

    match format {
        OutputFormat::Text => println!("✅ Project updated: {}", project.name),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "updated": project.id,
                    "name": project.name,
                    "status": project.status.to_string(),
                })
            );
        }
    }
    Ok(())
}

fn delete_project(name: &str, force: bool, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let project = resolve_project(&db, name)?;

    if !force {
        let topics = db
            .list_topics_by_project(Some(&project.id))
            .unwrap_or_default();
        if !topics.is_empty() {
            anyhow::bail!(
                "Project '{}' has {} topic(s). Use --force to delete anyway.",
                project.name,
                topics.len()
            );
        }
    }

    db.delete_project(&project.id)
        .context("Failed to delete project")?;

    match format {
        OutputFormat::Text => println!("🗑  Project deleted: {}", project.name),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"deleted": project.id, "name": project.name})
            );
        }
    }
    Ok(())
}
