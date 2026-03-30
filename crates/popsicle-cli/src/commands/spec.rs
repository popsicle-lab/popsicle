use anyhow::{Context, Result};
use clap::Subcommand;

use popsicle_core::model::Spec;
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

fn project_layout() -> Result<ProjectLayout> {
    let cwd = std::env::current_dir()?;
    popsicle_core::helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

#[derive(Subcommand)]
pub enum SpecCommand {
    /// Create a new spec
    Create {
        /// Spec name (e.g. "jwt-migration")
        name: String,
        /// Short description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Comma-separated tags
        #[arg(short, long, default_value = "")]
        tags: String,
        /// Namespace this spec belongs to (name or ID) — required
        #[arg(long)]
        namespace: String,
    },
    /// List all specs
    List {
        /// Filter by namespace (name or ID)
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Show details of a spec
    Show {
        /// Spec name or ID
        name: String,
    },
    /// Delete a spec
    Delete {
        /// Spec name or ID
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

pub fn execute(cmd: SpecCommand, format: &OutputFormat) -> Result<()> {
    match cmd {
        SpecCommand::Create {
            name,
            description,
            tags,
            namespace,
        } => create_spec(&name, &description, &tags, &namespace, format),
        SpecCommand::List { namespace } => list_specs(namespace.as_deref(), format),
        SpecCommand::Show { name } => show_spec(&name, format),
        SpecCommand::Delete { name, force } => delete_spec(&name, force, format),
    }
}

fn resolve_namespace_id(db: &IndexDb, name_or_id: &str) -> Result<String> {
    if let Some(p) = db.find_namespace_by_name(name_or_id).ok().flatten() {
        return Ok(p.id);
    }
    if let Some(p) = db.get_namespace(name_or_id).ok().flatten() {
        return Ok(p.id);
    }
    anyhow::bail!("Namespace not found: {}", name_or_id)
}

fn create_spec(
    name: &str,
    description: &str,
    tags: &str,
    namespace: &str,
    format: &OutputFormat,
) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let tag_list: Vec<String> = if tags.is_empty() {
        vec![]
    } else {
        tags.split(',').map(|t| t.trim().to_string()).collect()
    };

    let namespace_id = resolve_namespace_id(&db, namespace)?;

    let mut spec = Spec::new(
        name.to_string(),
        description.to_string(),
        namespace_id.clone(),
    );
    spec.tags = tag_list;
    db.create_spec(&spec).context("Failed to create spec")?;

    match format {
        OutputFormat::Text => {
            println!("✅ Spec created: {} ({})", spec.name, spec.slug);
            println!("   ID: {}", spec.id);
            println!("   Namespace: {}", namespace_id);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": spec.id,
                    "name": spec.name,
                    "slug": spec.slug,
                    "description": spec.description,
                    "tags": spec.tags,
                    "namespace_id": spec.namespace_id,
                }))?,
            );
        }
    }
    Ok(())
}

fn list_specs(namespace: Option<&str>, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let namespace_id = match namespace {
        Some(p) => Some(resolve_namespace_id(&db, p)?),
        None => None,
    };

    let specs = db
        .list_specs_by_namespace(namespace_id.as_deref())
        .or_else(|_| db.list_specs())
        .context("Failed to list specs")?;

    match format {
        OutputFormat::Text => {
            if specs.is_empty() {
                println!("No specs found.");
                return Ok(());
            }
            println!("{:<36}  {:<24}  {:<20}  TAGS", "ID", "NAME", "SLUG");
            println!("{}", "-".repeat(100));
            for t in &specs {
                let tags = if t.tags.is_empty() {
                    String::new()
                } else {
                    t.tags.join(", ")
                };
                println!("{:<36}  {:<24}  {:<20}  {}", t.id, t.name, t.slug, tags);
            }
            println!("\n{} spec(s)", specs.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = specs
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

fn show_spec(name: &str, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let spec = db
        .find_spec_by_name(name)
        .context("Failed to look up spec")?
        .or_else(|| db.get_spec(name).ok().flatten())
        .ok_or_else(|| anyhow::anyhow!("Spec not found: {}", name))?;

    let runs = db.list_spec_runs(&spec.id).unwrap_or_default();
    let docs = db.query_spec_documents(&spec.id).unwrap_or_default();
    let issues = db
        .query_issues(None, None, None, Some(&spec.id))
        .unwrap_or_default();

    // Resolve namespace name if present
    let namespace_name = if spec.namespace_id.is_empty() {
        None
    } else {
        db.get_namespace(&spec.namespace_id)
            .ok()
            .flatten()
            .map(|p| p.name)
    };

    match format {
        OutputFormat::Text => {
            println!("Spec: {}", spec.name);
            println!("  ID:          {}", spec.id);
            println!("  Slug:        {}", spec.slug);
            println!("  Description: {}", spec.description);
            if let Some(ref pname) = namespace_name {
                println!("  Namespace:     {}", pname);
            }
            if !spec.tags.is_empty() {
                println!("  Tags:        {}", spec.tags.join(", "));
            }
            println!(
                "  Created:     {}",
                spec.created_at.format("%Y-%m-%d %H:%M")
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
                    "id": spec.id,
                    "name": spec.name,
                    "slug": spec.slug,
                    "description": spec.description,
                    "namespace_id": spec.namespace_id,
                    "namespace_name": namespace_name,
                    "tags": spec.tags,
                    "created_at": spec.created_at.to_rfc3339(),
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

fn delete_spec(name: &str, force: bool, format: &OutputFormat) -> Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let spec = db
        .find_spec_by_name(name)
        .context("Failed to look up spec")?
        .or_else(|| db.get_spec(name).ok().flatten())
        .ok_or_else(|| anyhow::anyhow!("Spec not found: {}", name))?;

    if !force {
        let runs = db.list_spec_runs(&spec.id).unwrap_or_default();
        if !runs.is_empty() {
            anyhow::bail!(
                "Spec '{}' has {} pipeline run(s). Use --force to delete anyway.",
                spec.name,
                runs.len()
            );
        }
    }

    db.delete_spec(&spec.id).context("Failed to delete spec")?;

    match format {
        OutputFormat::Text => println!("🗑  Spec deleted: {}", spec.name),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"deleted": spec.id, "name": spec.name})
            );
        }
    }
    Ok(())
}
