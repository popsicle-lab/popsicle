use std::env;

use popsicle_core::git::GitTracker;
use popsicle_core::memory::{MAX_LINES, Memory, MemoryLayer, MemoryStore, MemoryType};
use popsicle_core::storage::ProjectLayout;

use crate::OutputFormat;

/// Lines-changed threshold for marking a memory as stale.
const STALE_CHANGE_THRESHOLD: u64 = 50;

#[derive(clap::Subcommand)]
pub enum MemoryCommand {
    /// Save a new memory entry
    Save(SaveArgs),
    /// List memory entries
    List(ListArgs),
    /// Show a single memory entry by ID
    Show(ShowArgs),
    /// Delete a memory entry by ID
    Delete(DeleteArgs),
    /// Promote a memory from short-term to long-term
    Promote(PromoteArgs),
    /// Mark a memory as stale
    Stale(StaleArgs),
    /// Remove all stale memories
    Gc,
    /// Detect outdated memories by checking git changes in associated files
    CheckStale,
    /// Show memory usage statistics
    Stats,
}

#[derive(clap::Args)]
pub struct SaveArgs {
    /// Memory type: bug, decision, pattern, gotcha
    #[arg(short = 't', long = "type")]
    memory_type: String,
    /// One-line summary
    #[arg(short, long)]
    summary: String,
    /// Detailed description (1-5 lines)
    #[arg(short, long, default_value = "")]
    detail: String,
    /// Comma-separated tags
    #[arg(long, default_value = "")]
    tags: String,
    /// Comma-separated related files
    #[arg(long, default_value = "")]
    files: String,
    /// Associated pipeline run ID
    #[arg(long)]
    run: Option<String>,
    /// Save directly as long-term (default: short-term)
    #[arg(long)]
    long_term: bool,
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Filter by layer: short-term, long-term
    #[arg(short, long)]
    layer: Option<String>,
    /// Filter by type: bug, decision, pattern, gotcha
    #[arg(short = 't', long = "type")]
    memory_type: Option<String>,
}

#[derive(clap::Args)]
pub struct ShowArgs {
    /// Memory ID
    id: u32,
}

#[derive(clap::Args)]
pub struct DeleteArgs {
    /// Memory ID
    id: u32,
}

#[derive(clap::Args)]
pub struct PromoteArgs {
    /// Memory ID
    id: u32,
}

#[derive(clap::Args)]
pub struct StaleArgs {
    /// Memory ID
    id: u32,
}

pub fn execute(cmd: MemoryCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        MemoryCommand::Save(args) => execute_save(args, format),
        MemoryCommand::List(args) => execute_list(args, format),
        MemoryCommand::Show(args) => execute_show(args, format),
        MemoryCommand::Delete(args) => execute_delete(args, format),
        MemoryCommand::Promote(args) => execute_promote(args, format),
        MemoryCommand::Stale(args) => execute_stale(args, format),
        MemoryCommand::Gc => execute_gc(format),
        MemoryCommand::CheckStale => execute_check_stale(format),
        MemoryCommand::Stats => execute_stats(format),
    }
}

fn memories_path() -> anyhow::Result<std::path::PathBuf> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(layout.memories_path())
}

fn load_memories() -> anyhow::Result<(std::path::PathBuf, Vec<Memory>)> {
    let path = memories_path()?;
    let memories = MemoryStore::load(&path).map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok((path, memories))
}

fn save_memories(path: &std::path::Path, memories: &[Memory]) -> anyhow::Result<()> {
    MemoryStore::save(path, memories).map_err(|e| anyhow::anyhow!("{}", e))
}

fn next_id(memories: &[Memory]) -> u32 {
    memories.iter().map(|m| m.id).max().unwrap_or(0) + 1
}

fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

// ── save ──

fn execute_save(args: SaveArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let (path, mut memories) = load_memories()?;

    let memory_type: MemoryType = args
        .memory_type
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;

    let layer = if args.long_term {
        MemoryLayer::LongTerm
    } else {
        MemoryLayer::ShortTerm
    };

    let id = next_id(&memories);
    let created = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let memory = Memory {
        id,
        memory_type,
        summary: args.summary,
        created,
        layer,
        refs: 0,
        tags: parse_csv(&args.tags),
        files: parse_csv(&args.files),
        run: args.run,
        stale: false,
        detail: args.detail,
    };

    memories.push(memory.clone());

    let line_count = MemoryStore::line_count(&memories);
    if line_count > MAX_LINES {
        anyhow::bail!(
            "Adding this memory would exceed the {} line limit ({} lines). \
             Run `popsicle memory gc` to clean stale memories, \
             merge similar memories into patterns, \
             or delete low-value entries.",
            MAX_LINES,
            line_count
        );
    }

    save_memories(&path, &memories)?;

    let capacity_pct = MemoryStore::capacity_pct(&memories);
    let capacity_warning = capacity_pct >= 80;

    match format {
        OutputFormat::Text => {
            println!(
                "Memory saved: [{}] #{} — {}",
                memory.memory_type, memory.id, memory.summary
            );
            println!(
                "Layer: {} | Lines: {}/{}",
                memory.layer, line_count, MAX_LINES
            );
            if capacity_warning {
                eprintln!(
                    "warning: memory capacity at {}%. Consider running `popsicle memory gc` or deleting low-value entries.",
                    capacity_pct
                );
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "id": memory.id,
                "type": memory.memory_type.to_string(),
                "layer": memory.layer.to_string(),
                "summary": memory.summary,
                "line_count": line_count,
                "max_lines": MAX_LINES,
                "capacity_pct": capacity_pct,
                "capacity_warning": capacity_warning,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── list ──

fn execute_list(args: ListArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let (_, memories) = load_memories()?;

    let layer_filter: Option<MemoryLayer> = args
        .layer
        .as_deref()
        .map(|s| s.parse().map_err(|e: String| anyhow::anyhow!("{}", e)))
        .transpose()?;

    let type_filter: Option<MemoryType> = args
        .memory_type
        .as_deref()
        .map(|s| s.parse().map_err(|e: String| anyhow::anyhow!("{}", e)))
        .transpose()?;

    let filtered: Vec<&Memory> = memories
        .iter()
        .filter(|m| layer_filter.is_none_or(|l| m.layer == l))
        .filter(|m| type_filter.is_none_or(|t| m.memory_type == t))
        .collect();

    match format {
        OutputFormat::Text => {
            if filtered.is_empty() {
                println!("No memories found.");
                return Ok(());
            }
            println!(
                "{:<4} {:<10} {:<11} {:<5} SUMMARY",
                "ID", "TYPE", "LAYER", "REFS"
            );
            println!("{}", "-".repeat(70));
            for m in &filtered {
                let stale_mark = if m.stale { " [STALE]" } else { "" };
                println!(
                    "{:<4} {:<10} {:<11} {:<5} {}{}",
                    m.id, m.memory_type, m.layer, m.refs, m.summary, stale_mark
                );
            }
            println!("\n{} memories total.", filtered.len());
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "memories": filtered,
                "count": filtered.len(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── show ──

fn execute_show(args: ShowArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let (_, memories) = load_memories()?;

    let memory = memories
        .iter()
        .find(|m| m.id == args.id)
        .ok_or_else(|| anyhow::anyhow!("Memory #{} not found.", args.id))?;

    match format {
        OutputFormat::Text => {
            println!(
                "[{}] #{} — {}",
                memory.memory_type, memory.id, memory.summary
            );
            println!(
                "Created: {} | Layer: {} | Refs: {}",
                memory.created, memory.layer, memory.refs
            );
            if !memory.tags.is_empty() {
                println!("Tags: {}", memory.tags.join(", "));
            }
            if !memory.files.is_empty() {
                println!("Files: {}", memory.files.join(", "));
            }
            if let Some(ref run) = memory.run {
                println!("Run: {run}");
            }
            if memory.stale {
                println!("Status: STALE");
            }
            if !memory.detail.is_empty() {
                println!("\n{}", memory.detail);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(memory)?);
        }
    }

    Ok(())
}

// ── delete ──

fn execute_delete(args: DeleteArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let (path, mut memories) = load_memories()?;

    let pos = memories
        .iter()
        .position(|m| m.id == args.id)
        .ok_or_else(|| anyhow::anyhow!("Memory #{} not found.", args.id))?;

    let removed = memories.remove(pos);
    save_memories(&path, &memories)?;

    match format {
        OutputFormat::Text => {
            println!("Deleted memory #{} — {}", removed.id, removed.summary);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "deleted_id": removed.id,
                "summary": removed.summary,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── promote ──

fn execute_promote(args: PromoteArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let (path, mut memories) = load_memories()?;

    let memory = memories
        .iter_mut()
        .find(|m| m.id == args.id)
        .ok_or_else(|| anyhow::anyhow!("Memory #{} not found.", args.id))?;

    if memory.layer == MemoryLayer::LongTerm {
        anyhow::bail!("Memory #{} is already long-term.", args.id);
    }

    memory.layer = MemoryLayer::LongTerm;
    let summary = memory.summary.clone();
    save_memories(&path, &memories)?;

    match format {
        OutputFormat::Text => {
            println!("Promoted memory #{} to long-term — {}", args.id, summary);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "id": args.id,
                "layer": "long-term",
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── stale ──

fn execute_stale(args: StaleArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let (path, mut memories) = load_memories()?;

    let memory = memories
        .iter_mut()
        .find(|m| m.id == args.id)
        .ok_or_else(|| anyhow::anyhow!("Memory #{} not found.", args.id))?;

    if memory.stale {
        anyhow::bail!("Memory #{} is already marked stale.", args.id);
    }

    memory.stale = true;
    let summary = memory.summary.clone();
    save_memories(&path, &memories)?;

    match format {
        OutputFormat::Text => {
            println!("Marked memory #{} as stale — {}", args.id, summary);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "id": args.id,
                "stale": true,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── gc ──

fn execute_gc(format: &OutputFormat) -> anyhow::Result<()> {
    let (path, memories) = load_memories()?;

    let stale_count = memories.iter().filter(|m| m.stale).count();
    if stale_count == 0 {
        match format {
            OutputFormat::Text => println!("No stale memories to clean."),
            OutputFormat::Json => {
                let result = serde_json::json!({ "status": "ok", "removed": 0 });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        return Ok(());
    }

    let remaining: Vec<_> = memories.into_iter().filter(|m| !m.stale).collect();
    let lines_after = MemoryStore::line_count(&remaining);
    save_memories(&path, &remaining)?;

    match format {
        OutputFormat::Text => {
            println!(
                "Removed {} stale memories. Lines: {}/{}",
                stale_count, lines_after, MAX_LINES
            );
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "removed": stale_count,
                "remaining": remaining.len(),
                "line_count": lines_after,
                "max_lines": MAX_LINES,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── check-stale ──

fn execute_check_stale(format: &OutputFormat) -> anyhow::Result<()> {
    let (path, mut memories) = load_memories()?;
    let cwd = env::current_dir()?;

    let mut newly_stale = Vec::new();

    for m in memories.iter_mut() {
        if m.stale || m.files.is_empty() {
            continue;
        }
        for file in &m.files {
            let changes = GitTracker::file_changes_since(&cwd, file, &m.created).unwrap_or(0);
            if changes >= STALE_CHANGE_THRESHOLD {
                m.stale = true;
                newly_stale.push((m.id, m.summary.clone(), file.clone(), changes));
                break;
            }
        }
    }

    if !newly_stale.is_empty() {
        save_memories(&path, &memories)?;
    }

    match format {
        OutputFormat::Text => {
            if newly_stale.is_empty() {
                println!("No newly stale memories detected.");
            } else {
                println!("Marked {} memories as stale:\n", newly_stale.len());
                for (id, summary, file, changes) in &newly_stale {
                    println!(
                        "  #{} — {} ({}: {} lines changed)",
                        id, summary, file, changes
                    );
                }
                println!("\nRun `popsicle memory gc` to remove stale memories.");
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = newly_stale
                .iter()
                .map(|(id, summary, file, changes)| {
                    serde_json::json!({
                        "id": id,
                        "summary": summary,
                        "trigger_file": file,
                        "lines_changed": changes,
                    })
                })
                .collect();
            let result = serde_json::json!({
                "status": "ok",
                "newly_stale": items,
                "count": newly_stale.len(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ── stats ──

fn execute_stats(format: &OutputFormat) -> anyhow::Result<()> {
    let (_, memories) = load_memories()?;

    let line_count = MemoryStore::line_count(&memories);
    let total = memories.len();
    let long_term = memories
        .iter()
        .filter(|m| m.layer == MemoryLayer::LongTerm)
        .count();
    let short_term = memories
        .iter()
        .filter(|m| m.layer == MemoryLayer::ShortTerm)
        .count();
    let bugs = memories
        .iter()
        .filter(|m| m.memory_type == MemoryType::Bug)
        .count();
    let decisions = memories
        .iter()
        .filter(|m| m.memory_type == MemoryType::Decision)
        .count();
    let patterns = memories
        .iter()
        .filter(|m| m.memory_type == MemoryType::Pattern)
        .count();
    let gotchas = memories
        .iter()
        .filter(|m| m.memory_type == MemoryType::Gotcha)
        .count();
    let stale = memories.iter().filter(|m| m.stale).count();

    match format {
        OutputFormat::Text => {
            println!("=== Memory Stats ===");
            println!("Lines: {}/{}", line_count, MAX_LINES);
            println!("Total: {}", total);
            println!();
            println!("By layer:");
            println!("  long-term:  {}", long_term);
            println!("  short-term: {}", short_term);
            println!();
            println!("By type:");
            println!("  bug:      {}", bugs);
            println!("  decision: {}", decisions);
            println!("  pattern:  {}", patterns);
            println!("  gotcha:   {}", gotchas);
            println!();
            println!("Stale: {}", stale);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "line_count": line_count,
                "max_lines": MAX_LINES,
                "total": total,
                "by_layer": {
                    "long_term": long_term,
                    "short_term": short_term,
                },
                "by_type": {
                    "bug": bugs,
                    "decision": decisions,
                    "pattern": patterns,
                    "gotcha": gotchas,
                },
                "stale": stale,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}
