use std::env;
use std::path::Path;

use popsicle_core::helpers;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum ChecklistCommand {
    /// Show checklist status for a document or all documents in a pipeline run
    Status {
        /// Document ID
        #[arg(long, group = "target")]
        doc: Option<String>,
        /// Pipeline run ID (show all docs in this run)
        #[arg(long, group = "target")]
        run: Option<String>,
    },
    /// Check off (mark done) checklist items in a document
    Check {
        /// Document ID
        #[arg(long)]
        doc: String,
        /// Comma-separated line numbers to check (from `pipeline review --checklist`)
        #[arg(long, conflicts_with_all = ["all", "match_text"])]
        lines: Option<String>,
        /// Check all unchecked items
        #[arg(long, conflicts_with_all = ["lines", "match_text"])]
        all: bool,
        /// Check items whose text contains this substring (case-insensitive)
        #[arg(long = "match", conflicts_with_all = ["lines", "all"])]
        match_text: Option<String>,
    },
    /// Uncheck checklist items in a document
    Uncheck {
        /// Document ID
        #[arg(long)]
        doc: String,
        /// Comma-separated line numbers to uncheck
        #[arg(long, conflicts_with = "all")]
        lines: Option<String>,
        /// Uncheck all checked items
        #[arg(long, conflicts_with = "lines")]
        all: bool,
    },
}

pub fn execute(cmd: ChecklistCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        ChecklistCommand::Status { doc, run } => checklist_status(doc.as_deref(), run.as_deref(), format),
        ChecklistCommand::Check {
            doc,
            lines,
            all,
            match_text,
        } => checklist_check(&doc, lines.as_deref(), all, match_text.as_deref(), format),
        ChecklistCommand::Uncheck { doc, lines, all } => {
            checklist_uncheck(&doc, lines.as_deref(), all, format)
        }
    }
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn find_doc_row(
    db: &IndexDb,
    doc_id: &str,
) -> anyhow::Result<popsicle_core::storage::DocumentRow> {
    let all = db
        .query_documents(None, None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    all.into_iter()
        .find(|d| d.id == doc_id)
        .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))
}

/// Parse a comma-separated line-number string into a sorted set.
fn parse_line_numbers(s: &str) -> anyhow::Result<Vec<usize>> {
    let mut nums = Vec::new();
    for part in s.split(',') {
        let n: usize = part
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid line number: '{}'", part.trim()))?;
        if n == 0 {
            anyhow::bail!("Line numbers are 1-based; got 0");
        }
        nums.push(n);
    }
    nums.sort_unstable();
    nums.dedup();
    Ok(nums)
}

// ─── Status ─────────────────────────────────────────────────────────────────

fn checklist_status(
    doc_id: Option<&str>,
    run_id: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let docs: Vec<popsicle_core::storage::DocumentRow> = if let Some(did) = doc_id {
        vec![find_doc_row(&db, did)?]
    } else {
        db.query_documents(None, None, run_id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
    };

    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut grand_checked = 0usize;
    let mut grand_unchecked = 0usize;

    for doc_row in &docs {
        let path = Path::new(&doc_row.file_path);
        if !path.exists() {
            continue;
        }
        let doc = FileStorage::read_document(path).map_err(|e| anyhow::anyhow!("{}", e))?;

        let mut checked_items: Vec<serde_json::Value> = Vec::new();
        let mut unchecked_items: Vec<serde_json::Value> = Vec::new();

        for (idx, line) in doc.body.lines().enumerate() {
            let trimmed = line.trim_start();
            let line_no = idx + 1;
            if trimmed.starts_with("- [ ] ") {
                unchecked_items.push(serde_json::json!({
                    "line": line_no,
                    "text": trimmed.trim_start_matches("- [ ] ").to_string(),
                }));
            } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
                let text = trimmed
                    .trim_start_matches("- [x] ")
                    .trim_start_matches("- [X] ")
                    .to_string();
                checked_items.push(serde_json::json!({
                    "line": line_no,
                    "text": text,
                }));
            }
        }

        if checked_items.is_empty() && unchecked_items.is_empty() {
            continue;
        }

        grand_checked += checked_items.len();
        grand_unchecked += unchecked_items.len();

        results.push(serde_json::json!({
            "doc_id": doc_row.id,
            "title": doc_row.title,
            "skill": doc_row.skill_name,
            "file_path": doc_row.file_path,
            "checked": checked_items.len(),
            "unchecked": unchecked_items.len(),
            "checked_items": checked_items,
            "unchecked_items": unchecked_items,
        }));
    }

    match format {
        OutputFormat::Text => {
            if results.is_empty() {
                println!("No checklists found.");
                return Ok(());
            }
            for r in &results {
                let title = r["title"].as_str().unwrap_or("?");
                let doc_id_str = r["doc_id"].as_str().unwrap_or("?");
                let checked = r["checked"].as_u64().unwrap_or(0);
                let unchecked = r["unchecked"].as_u64().unwrap_or(0);
                let total = checked + unchecked;
                println!(
                    "📋 {} ({}) — {}/{} checked",
                    title, doc_id_str, checked, total
                );
                if let Some(items) = r["unchecked_items"].as_array() {
                    for item in items {
                        let line = item["line"].as_u64().unwrap_or(0);
                        let text = item["text"].as_str().unwrap_or("?");
                        println!("   ☐ L{}: {}", line, text);
                    }
                }
            }
            println!(
                "\nTotal: {}/{} checked",
                grand_checked,
                grand_checked + grand_unchecked
            );
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "total_checked": grand_checked,
                "total_unchecked": grand_unchecked,
                "documents": results,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

// ─── Check ──────────────────────────────────────────────────────────────────

fn checklist_check(
    doc_id: &str,
    lines_str: Option<&str>,
    all: bool,
    match_text: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    if !all && lines_str.is_none() && match_text.is_none() {
        anyhow::bail!("Specify --lines, --all, or --match to select items to check");
    }

    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let doc_row = find_doc_row(&db, doc_id)?;
    let path = std::path::PathBuf::from(&doc_row.file_path);
    let mut doc = FileStorage::read_document(&path).map_err(|e| anyhow::anyhow!("{}", e))?;

    let target_lines = lines_str.map(parse_line_numbers).transpose()?;
    let match_lower = match_text.map(|s| s.to_lowercase());

    let mut updated_count = 0usize;
    let mut updated_items: Vec<serde_json::Value> = Vec::new();

    let new_body: String = doc
        .body
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let line_no = idx + 1;
            let trimmed = line.trim_start();
            if !trimmed.starts_with("- [ ] ") {
                return line.to_string();
            }

            let should_check = if all {
                true
            } else if let Some(ref nums) = target_lines {
                nums.contains(&line_no)
            } else if let Some(ref pattern) = match_lower {
                trimmed.to_lowercase().contains(pattern.as_str())
            } else {
                false
            };

            if should_check {
                let indent = &line[..line.len() - trimmed.len()];
                let rest = trimmed.trim_start_matches("- [ ] ");
                updated_count += 1;
                updated_items.push(serde_json::json!({
                    "line": line_no,
                    "text": rest.to_string(),
                }));
                format!("{}- [x] {}", indent, rest)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Preserve trailing newline if original had one
    doc.body = if doc.body.ends_with('\n') && !new_body.ends_with('\n') {
        format!("{}\n", new_body)
    } else {
        new_body
    };

    if updated_count > 0 {
        FileStorage::write_document(&doc, &path)?;
    }

    match format {
        OutputFormat::Text => {
            if updated_count == 0 {
                println!("No items matched. Nothing changed.");
            } else {
                println!("✅ Checked {} item(s) in '{}'", updated_count, doc_row.title);
                for item in &updated_items {
                    let line = item["line"].as_u64().unwrap_or(0);
                    let text = item["text"].as_str().unwrap_or("?");
                    println!("   ✓ L{}: {}", line, text);
                }
                println!("  File: {}", doc_row.file_path);
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "doc_id": doc_id,
                "title": doc_row.title,
                "updated": updated_count,
                "items": updated_items,
                "file_path": doc_row.file_path,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

// ─── Uncheck ────────────────────────────────────────────────────────────────

fn checklist_uncheck(
    doc_id: &str,
    lines_str: Option<&str>,
    all: bool,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    if !all && lines_str.is_none() {
        anyhow::bail!("Specify --lines or --all to select items to uncheck");
    }

    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let doc_row = find_doc_row(&db, doc_id)?;
    let path = std::path::PathBuf::from(&doc_row.file_path);
    let mut doc = FileStorage::read_document(&path).map_err(|e| anyhow::anyhow!("{}", e))?;

    let target_lines = lines_str.map(parse_line_numbers).transpose()?;

    let mut updated_count = 0usize;
    let mut updated_items: Vec<serde_json::Value> = Vec::new();

    let new_body: String = doc
        .body
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let line_no = idx + 1;
            let trimmed = line.trim_start();
            let is_checked =
                trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ");
            if !is_checked {
                return line.to_string();
            }

            let should_uncheck = if all {
                true
            } else if let Some(ref nums) = target_lines {
                nums.contains(&line_no)
            } else {
                false
            };

            if should_uncheck {
                let indent = &line[..line.len() - trimmed.len()];
                let rest = trimmed
                    .trim_start_matches("- [x] ")
                    .trim_start_matches("- [X] ");
                updated_count += 1;
                updated_items.push(serde_json::json!({
                    "line": line_no,
                    "text": rest.to_string(),
                }));
                format!("{}- [ ] {}", indent, rest)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    doc.body = if doc.body.ends_with('\n') && !new_body.ends_with('\n') {
        format!("{}\n", new_body)
    } else {
        new_body
    };

    if updated_count > 0 {
        FileStorage::write_document(&doc, &path)?;
    }

    match format {
        OutputFormat::Text => {
            if updated_count == 0 {
                println!("No items matched. Nothing changed.");
            } else {
                println!(
                    "☐ Unchecked {} item(s) in '{}'",
                    updated_count, doc_row.title
                );
                for item in &updated_items {
                    let line = item["line"].as_u64().unwrap_or(0);
                    let text = item["text"].as_str().unwrap_or("?");
                    println!("   ○ L{}: {}", line, text);
                }
                println!("  File: {}", doc_row.file_path);
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "doc_id": doc_id,
                "title": doc_row.title,
                "updated": updated_count,
                "items": updated_items,
                "file_path": doc_row.file_path,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_numbers() {
        let nums = parse_line_numbers("5,12,23").unwrap();
        assert_eq!(nums, vec![5, 12, 23]);
    }

    #[test]
    fn test_parse_line_numbers_dedup_and_sort() {
        let nums = parse_line_numbers("23,5,5,12").unwrap();
        assert_eq!(nums, vec![5, 12, 23]);
    }

    #[test]
    fn test_parse_line_numbers_single() {
        let nums = parse_line_numbers("7").unwrap();
        assert_eq!(nums, vec![7]);
    }

    #[test]
    fn test_parse_line_numbers_zero_rejected() {
        assert!(parse_line_numbers("0").is_err());
    }

    #[test]
    fn test_parse_line_numbers_invalid() {
        assert!(parse_line_numbers("abc").is_err());
    }
}
