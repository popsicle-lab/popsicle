use anyhow::{Context, Result};
use clap::Subcommand;

use popsicle_core::model::Namespace;
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

fn project_layout() -> Result<ProjectLayout> {
    let cwd = std::env::current_dir()?;
    popsicle_core::helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

#[derive(Subcommand)]
pub enum NamespaceCommand {
    /// Create a new namespace
    Create {
        /// Namespace name (human-readable)
        name: String,
        /// Short description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Comma-separated tags
        #[arg(short, long, default_value = "")]
        tags: String,
    },
    /// List all namespaces
    List {
        /// Filter by status (active, completed, archived)
        #[arg(long)]
        status: Option<String>,
    },
    /// Show namespace details with associated specs
    Show {
        /// Namespace name or ID
        name: String,
    },
    /// Update a namespace
    Update {
        /// Namespace name or ID
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
    /// Delete a namespace
    Delete {
        /// Namespace name or ID
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

pub fn execute(cmd: NamespaceCommand, format: &OutputFormat) -> Result<()> {
    match cmd {
        NamespaceCommand::Create {
            name,
            description,
            tags,
        } => create_namespace(&name, &description, &tags, format),
        NamespaceCommand::List { status } => list_namespaces(status.as_deref(), format),
        NamespaceCommand::Show { name } => show_namespace(&name, format),
        NamespaceCommand::Update {
            name,
            description,
            status,
            tags,
        } => update_namespace(&name, description, status, tags, format),
        NamespaceCommand::Delete { name, force } => delete_namespace(&name, force, format),
    }
}

fn resolve_namespace(db: &IndexDb, name_or_id: &str) -> Result<Namespace> {
    db.find_namespace_by_name(name_or_id)
        .ok()
        .flatten()
        .or_else(|| db.get_namespace(name_or_id).ok().flatten())
        .ok_or_else(|| anyhow::anyhow!("Namespace not found: {}", name_or_id))
}

fn create_namespace(name: &str, description: &str, tags: &str, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let tag_list: Vec<String> = if tags.is_empty() {
        vec![]
    } else {
        tags.split(',').map(|t| t.trim().to_string()).collect()
    };

    let mut namespace = Namespace::new(name.to_string(), description.to_string());
    namespace.tags = tag_list;
    db.create_namespace(&namespace)
        .context("Failed to create namespace")?;

    match format {
        OutputFormat::Text => {
            println!("✅ Namespace created: {} ({})", namespace.name, namespace.slug);
            println!("   ID: {}", namespace.id);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": namespace.id,
                    "name": namespace.name,
                    "slug": namespace.slug,
                    "description": namespace.description,
                    "status": namespace.status.to_string(),
                    "tags": namespace.tags,
                }))?,
            );
        }
    }
    Ok(())
}

fn list_namespaces(status_filter: Option<&str>, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let namespaces = db
        .list_namespaces(status_filter)
        .context("Failed to list namespaces")?;

    match format {
        OutputFormat::Text => {
            if namespaces.is_empty() {
                println!("No namespaces found.");
                return Ok(());
            }
            println!(
                "{:<36}  {:<20}  {:<10}  DESCRIPTION",
                "ID", "NAME", "STATUS"
            );
            println!("{}", "-".repeat(100));
            for p in &namespaces {
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
            println!("\n{} namespace(s)", namespaces.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = namespaces
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

fn show_namespace(name: &str, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let namespace = resolve_namespace(&db, name)?;

    let specs = db
        .list_specs_by_namespace(Some(&namespace.id))
        .or_else(|_| db.list_specs())
        .unwrap_or_default()
        .into_iter()
        .filter(|t| t.namespace_id == namespace.id)
        .collect::<Vec<_>>();

    match format {
        OutputFormat::Text => {
            println!("Namespace: {}", namespace.name);
            println!("  ID:          {}", namespace.id);
            println!("  Slug:        {}", namespace.slug);
            println!("  Status:      {}", namespace.status);
            println!("  Description: {}", namespace.description);
            if !namespace.tags.is_empty() {
                println!("  Tags:        {}", namespace.tags.join(", "));
            }
            println!(
                "  Created:     {}",
                namespace.created_at.format("%Y-%m-%d %H:%M")
            );
            println!();

            if specs.is_empty() {
                println!("  No specs.");
            } else {
                println!("  Specs ({}):", specs.len());
                for t in &specs {
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
                    "id": namespace.id,
                    "name": namespace.name,
                    "slug": namespace.slug,
                    "description": namespace.description,
                    "status": namespace.status.to_string(),
                    "tags": namespace.tags,
                    "created_at": namespace.created_at.to_rfc3339(),
                    "specs": specs.iter().map(|t| serde_json::json!({
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

fn update_namespace(
    name: &str,
    description: Option<String>,
    status: Option<String>,
    tags: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut namespace = resolve_namespace(&db, name)?;

    if let Some(desc) = description {
        namespace.description = desc;
    }
    if let Some(s) = status {
        namespace.status = s
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid status: {}. Use active/completed/archived", s))?;
    }
    if let Some(t) = tags {
        namespace.tags = if t.is_empty() {
            vec![]
        } else {
            t.split(',').map(|s| s.trim().to_string()).collect()
        };
    }
    namespace.updated_at = chrono::Utc::now();

    db.update_namespace(&namespace)
        .context("Failed to update namespace")?;

    match format {
        OutputFormat::Text => println!("✅ Namespace updated: {}", namespace.name),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "updated": namespace.id,
                    "name": namespace.name,
                    "status": namespace.status.to_string(),
                })
            );
        }
    }
    Ok(())
}

fn delete_namespace(name: &str, force: bool, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let namespace = resolve_namespace(&db, name)?;

    if !force {
        let specs = db
            .list_specs_by_namespace(Some(&namespace.id))
            .unwrap_or_default();
        if !specs.is_empty() {
            anyhow::bail!(
                "Namespace '{}' has {} spec(s). Use --force to delete anyway.",
                namespace.name,
                specs.len()
            );
        }
    }

    db.delete_namespace(&namespace.id)
        .context("Failed to delete namespace")?;

    match format {
        OutputFormat::Text => println!("🗑  Namespace deleted: {}", namespace.name),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"deleted": namespace.id, "name": namespace.name})
            );
        }
    }
    Ok(())
}
