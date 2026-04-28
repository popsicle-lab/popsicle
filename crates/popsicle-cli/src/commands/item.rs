use anyhow::Result;
use clap::Subcommand;
use popsicle_core::model::{Spec, WorkItem, WorkItemKind};
use popsicle_core::storage::IndexDb;
use serde_json::json;

use crate::OutputFormat;

#[derive(Subcommand)]
pub enum ItemCommand {
    /// Create a work item (bug | story | tc)
    Add {
        /// Item kind: bug | story | tc
        #[arg(long)]
        kind: String,
        /// Title (required)
        #[arg(long)]
        title: String,
        /// Description
        #[arg(long, default_value = "")]
        description: String,
        /// Priority: low | medium | high | critical
        #[arg(long, default_value = "medium")]
        priority: String,
        /// Issue ID to link
        #[arg(long)]
        issue: Option<String>,
        /// Pipeline run ID to link
        #[arg(long)]
        run: Option<String>,
        /// Source document ID
        #[arg(long)]
        doc: Option<String>,
        /// Free-form labels (repeatable)
        #[arg(long)]
        label: Vec<String>,
        /// Custom field as `key=value` (repeatable). Values are stored as strings.
        #[arg(long = "field")]
        fields: Vec<String>,
        /// Spec name to use for key prefix
        #[arg(long)]
        spec: Option<String>,
    },
    /// List work items with optional filters
    List {
        /// Filter by kind: bug | story | tc
        #[arg(long)]
        kind: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by issue ID
        #[arg(long)]
        issue: Option<String>,
        /// Filter by pipeline run ID
        #[arg(long)]
        run: Option<String>,
    },
    /// Show a work item by ID or key
    Show {
        /// ID or key (e.g. BUG-PRJ-1)
        key: String,
    },
    /// Update fields on a work item
    Update {
        /// ID or key
        key: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New priority
        #[arg(long)]
        priority: Option<String>,
        /// Custom field as `key=value` (repeatable)
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    /// Delete a work item
    Delete {
        /// ID or key
        key: String,
    },
}

pub fn execute(cmd: ItemCommand, format: &OutputFormat) -> Result<()> {
    let db = IndexDb::open(&std::path::PathBuf::from(".popsicle/popsicle.db"))?;
    match cmd {
        ItemCommand::Add {
            kind,
            title,
            description,
            priority,
            issue,
            run,
            doc,
            label,
            fields,
            spec,
        } => add(
            &db,
            &kind,
            title,
            description,
            priority,
            issue,
            run,
            doc,
            label,
            fields,
            spec,
            format,
        ),
        ItemCommand::List {
            kind,
            status,
            issue,
            run,
        } => list(&db, kind, status, issue, run, format),
        ItemCommand::Show { key } => show(&db, &key, format),
        ItemCommand::Update {
            key,
            title,
            status,
            priority,
            fields,
        } => update(&db, &key, title, status, priority, fields, format),
        ItemCommand::Delete { key } => delete(&db, &key, format),
    }
}

fn parse_kind(s: &str) -> Result<WorkItemKind> {
    s.parse::<WorkItemKind>()
        .map_err(|e| anyhow::anyhow!("invalid kind: {e}"))
}

fn parse_priority(s: &str) -> Result<popsicle_core::model::Priority> {
    s.parse::<popsicle_core::model::Priority>()
        .map_err(|e| anyhow::anyhow!("invalid priority: {e}"))
}

fn apply_fields(item: &mut WorkItem, raw: &[String]) -> Result<()> {
    for entry in raw {
        let (k, v) = entry
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("--field expects key=value, got `{entry}`"))?;
        // Try parse as JSON literal first; fall back to string.
        let value = serde_json::from_str(v.trim()).unwrap_or_else(|_| json!(v.trim()));
        item.set_field(k.trim(), value);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn add(
    db: &IndexDb,
    kind: &str,
    title: String,
    description: String,
    priority: String,
    issue: Option<String>,
    run: Option<String>,
    doc: Option<String>,
    labels: Vec<String>,
    fields: Vec<String>,
    spec: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    let kind = parse_kind(kind)?;
    let priority = parse_priority(&priority)?;

    // Determine prefix from spec name (or fallback to "GEN")
    let prefix = match spec.as_deref() {
        Some(name) => name.to_uppercase(),
        None => match db.list_specs()?.first() {
            Some(s) => spec_prefix(s),
            None => "GEN".to_string(),
        },
    };

    let seq = db.next_work_item_seq(kind, &prefix)?;
    let key = format!("{}-{}-{}", kind.key_prefix(), prefix, seq);

    let mut item = WorkItem::new(&key, kind, &title);
    item.description = description;
    item.priority = priority;
    item.labels = labels;
    item.issue_id = issue;
    item.pipeline_run_id = run;
    item.source_doc_id = doc;
    apply_fields(&mut item, &fields)?;

    db.create_work_item(&item)?;
    emit_item(&item, format)
}

fn spec_prefix(spec: &Spec) -> String {
    spec.name.to_uppercase().replace([' ', '-', '_'], "")
}

fn list(
    db: &IndexDb,
    kind: Option<String>,
    status: Option<String>,
    issue: Option<String>,
    run: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    let k = match kind.as_deref() {
        Some(s) => Some(parse_kind(s)?),
        None => None,
    };
    let items = db.query_work_items(k, status.as_deref(), issue.as_deref(), run.as_deref())?;
    match format {
        OutputFormat::Json => {
            let arr: Vec<_> = items.iter().map(item_to_json).collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        OutputFormat::Text => {
            if items.is_empty() {
                println!("(no work items)");
            } else {
                for item in &items {
                    println!(
                        "{:8} {:6} {:10} {}",
                        item.key,
                        item.kind.as_str(),
                        item.status,
                        item.title
                    );
                }
            }
        }
    }
    Ok(())
}

fn show(db: &IndexDb, key: &str, format: &OutputFormat) -> Result<()> {
    let item = db
        .get_work_item(key)?
        .ok_or_else(|| anyhow::anyhow!("work item not found: {key}"))?;
    emit_item(&item, format)
}

fn update(
    db: &IndexDb,
    key: &str,
    title: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    fields: Vec<String>,
    format: &OutputFormat,
) -> Result<()> {
    let mut item = db
        .get_work_item(key)?
        .ok_or_else(|| anyhow::anyhow!("work item not found: {key}"))?;
    if let Some(t) = title {
        item.title = t;
    }
    if let Some(s) = status {
        item.status = s;
    }
    if let Some(p) = priority {
        item.priority = parse_priority(&p)?;
    }
    apply_fields(&mut item, &fields)?;
    item.updated_at = chrono::Utc::now();
    db.update_work_item(&item)?;
    emit_item(&item, format)
}

fn delete(db: &IndexDb, key: &str, format: &OutputFormat) -> Result<()> {
    let item = db
        .get_work_item(key)?
        .ok_or_else(|| anyhow::anyhow!("work item not found: {key}"))?;
    db.delete_work_item(&item.id)?;
    match format {
        OutputFormat::Json => println!("{}", json!({ "deleted": item.key })),
        OutputFormat::Text => println!("deleted {}", item.key),
    }
    Ok(())
}

fn emit_item(item: &WorkItem, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&item_to_json(item))?);
        }
        OutputFormat::Text => {
            println!("Key:         {}", item.key);
            println!("Kind:        {}", item.kind);
            println!("Title:       {}", item.title);
            println!("Status:      {}", item.status);
            println!("Priority:    {}", item.priority);
            if !item.labels.is_empty() {
                println!("Labels:      {}", item.labels.join(", "));
            }
            if let Some(i) = &item.issue_id {
                println!("Issue:       {i}");
            }
            if let Some(r) = &item.pipeline_run_id {
                println!("Run:         {r}");
            }
            if !item.description.is_empty() {
                println!("\n{}", item.description);
            }
            if !item
                .fields
                .as_object()
                .map(|m| m.is_empty())
                .unwrap_or(true)
            {
                println!("\nFields:");
                println!("{}", serde_json::to_string_pretty(&item.fields)?);
            }
        }
    }
    Ok(())
}

fn item_to_json(item: &WorkItem) -> serde_json::Value {
    json!({
        "id": item.id,
        "key": item.key,
        "kind": item.kind.as_str(),
        "title": item.title,
        "description": item.description,
        "status": item.status,
        "priority": item.priority.to_string(),
        "labels": item.labels,
        "issue_id": item.issue_id,
        "pipeline_run_id": item.pipeline_run_id,
        "source_doc_id": item.source_doc_id,
        "fields": item.fields,
        "created_at": item.created_at.to_rfc3339(),
        "updated_at": item.updated_at.to_rfc3339(),
    })
}
