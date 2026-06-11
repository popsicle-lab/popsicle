//! Read-only scanners for IDD task chunks and intent metadata (PROJ-27 UI).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use storage::WorkspaceError;

#[derive(Debug, Clone, Serialize)]
pub struct TaskNode {
    pub task_id: String,
    pub title: String,
    pub journey_stage: String,
    pub product: String,
    pub related_next_tasks: Vec<String>,
    pub related_intents: Vec<String>,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskGraph {
    pub nodes: Vec<TaskNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntentBlockNode {
    pub name: String,
    pub kind: String,
    pub task_id: Option<String>,
    pub product: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntentGraph {
    pub blocks: Vec<IntentBlockNode>,
    pub mermaid: Option<String>,
    pub source: String,
}

/// Scan `products/*/tasks/**/*.md` for task frontmatter.
pub fn scan_tasks(products_dir: &Path) -> Result<TaskGraph, WorkspaceError> {
    let mut nodes = Vec::new();
    if !products_dir.is_dir() {
        return Ok(TaskGraph { nodes });
    }
    for product_entry in fs::read_dir(products_dir).map_err(io_err)? {
        let product_entry = product_entry.map_err(io_err)?;
        let product_path = product_entry.path();
        if !product_path.is_dir() {
            continue;
        }
        let product = product_entry.file_name().to_string_lossy().into_owned();
        let tasks_root = product_path.join("tasks");
        if !tasks_root.is_dir() {
            continue;
        }
        walk_tasks(&tasks_root, &product, &mut nodes)?;
    }
    nodes.sort_by(|a, b| a.task_id.cmp(&b.task_id));
    Ok(TaskGraph { nodes })
}

fn walk_tasks(dir: &Path, product: &str, out: &mut Vec<TaskNode>) -> Result<(), WorkspaceError> {
    for entry in fs::read_dir(dir).map_err(io_err)? {
        let entry = entry.map_err(io_err)?;
        let path = entry.path();
        if path.is_dir() {
            walk_tasks(&path, product, out)?;
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(io_err)?;
        if let Some(node) = parse_task_frontmatter(&content, product, &path) {
            out.push(node);
        }
    }
    Ok(())
}

fn parse_task_frontmatter(content: &str, product: &str, path: &Path) -> Option<TaskNode> {
    let fm = content.strip_prefix("---")?;
    let rest = fm.strip_prefix('\n').or_else(|| fm.strip_prefix("\r\n"))?;
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];
    let mut map: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(yaml).ok()?;
    let task_id = map.remove("task_id")?.as_str()?.to_string();
    let title = map
        .remove("title")
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| task_id.clone());
    let journey_stage = map
        .remove("journey_stage")
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "daily-ops".into());
    let related_next_tasks = map.remove("related_next_tasks").and_then(parse_str_list);
    let related_intents = map.remove("related_intents").and_then(parse_str_list);
    Some(TaskNode {
        task_id,
        title,
        journey_stage,
        product: product.to_string(),
        related_next_tasks: related_next_tasks.unwrap_or_default(),
        related_intents: related_intents.unwrap_or_default(),
        file_path: path.display().to_string(),
    })
}

fn parse_str_list(v: serde_yaml::Value) -> Option<Vec<String>> {
    match v {
        serde_yaml::Value::Sequence(seq) => Some(
            seq.into_iter()
                .filter_map(|i| i.as_str().map(str::to_string))
                .collect(),
        ),
        serde_yaml::Value::String(s) => Some(vec![s]),
        _ => None,
    }
}

/// Build an intent graph: try `intent goals --diagram`, else parse `.intent` files.
pub fn scan_intents(workspace_root: &Path, product: &str) -> Result<IntentGraph, WorkspaceError> {
    let products_dir = workspace_root.join("products").join(product);
    let mermaid = try_intent_cli_diagram(&products_dir);
    let mut blocks = Vec::new();
    let intents_dir = products_dir.join("intents");
    if intents_dir.is_dir() {
        for entry in fs::read_dir(&intents_dir).map_err(io_err)? {
            let entry = entry.map_err(io_err)?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("intent") {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(io_err)?;
            blocks.extend(parse_intent_blocks(&content, product, &path));
        }
    }
    let source = if mermaid.is_some() {
        "intent-cli".into()
    } else {
        "parsed".into()
    };
    Ok(IntentGraph {
        blocks,
        mermaid,
        source,
    })
}

pub fn list_products(workspace_root: &Path) -> Result<Vec<String>, WorkspaceError> {
    let products_dir = workspace_root.join("products");
    let mut names = Vec::new();
    if products_dir.is_dir() {
        for entry in fs::read_dir(&products_dir).map_err(io_err)? {
            let entry = entry.map_err(io_err)?;
            if entry.path().is_dir() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }
    names.sort();
    Ok(names)
}

fn try_intent_cli_diagram(products_product_dir: &Path) -> Option<String> {
    if !products_product_dir.is_dir() {
        return None;
    }
    let intent_cli = which_intent_cli()?;
    let output = Command::new(intent_cli)
        .args([
            "goals",
            "--diagram",
            &format!("path={}", products_product_dir.display()),
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.contains("graph ") || text.contains("flowchart ") {
        Some(text)
    } else {
        None
    }
}

fn which_intent_cli() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("./target/debug/intent"),
        PathBuf::from("intent"),
    ];
    for c in candidates {
        if c.is_file() {
            return Some(c);
        }
    }
    Command::new("which")
        .arg("intent")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
}

fn parse_intent_blocks(content: &str, product: &str, path: &Path) -> Vec<IntentBlockNode> {
    let mut blocks = Vec::new();
    let file = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("intent ") {
            let name = trimmed
                .strip_prefix("intent ")
                .and_then(|s| s.split('(').next())
                .unwrap_or("unknown")
                .trim()
                .to_string();
            blocks.push(IntentBlockNode {
                name,
                kind: "intent".into(),
                task_id: None,
                product: product.to_string(),
                file: file.clone(),
            });
        } else if trimmed.starts_with("safety ") {
            let name = trimmed
                .strip_prefix("safety ")
                .and_then(|s| s.split('(').next())
                .unwrap_or("unknown")
                .trim()
                .to_string();
            blocks.push(IntentBlockNode {
                name,
                kind: "safety".into(),
                task_id: None,
                product: product.to_string(),
                file: file.clone(),
            });
        } else if trimmed.starts_with("// task:") {
            let task_id = trimmed.trim_start_matches("// task:").trim().to_string();
            if let Some(last) = blocks.last_mut() {
                last.task_id = Some(task_id);
            }
        }
    }
    blocks
}

/// Fallback mermaid when intent-cli is unavailable.
pub fn task_graph_mermaid(graph: &TaskGraph) -> String {
    let mut lines = vec!["flowchart LR".to_string()];
    for node in &graph.nodes {
        let id = sanitize_id(&node.task_id);
        let label = format!("{}\\n{}", node.task_id, node.title.replace('"', "'"));
        lines.push(format!("  {id}[\"{label}\"]"));
    }
    for node in &graph.nodes {
        let from = sanitize_id(&node.task_id);
        for next in &node.related_next_tasks {
            lines.push(format!("  {from} --> {}", sanitize_id(next)));
        }
    }
    lines.join("\n")
}

pub fn intent_fallback_mermaid(graph: &IntentGraph) -> String {
    let mut lines = vec!["flowchart TB".to_string()];
    for (i, block) in graph.blocks.iter().enumerate() {
        let id = format!("n{i}");
        let label = format!("{}:{}\\n{}", block.kind, block.name, block.file);
        lines.push(format!("  {id}[\"{label}\"]"));
        if let Some(ref task) = block.task_id {
            lines.push(format!("  {id} -.-> {}", sanitize_id(task)));
        }
    }
    lines.join("\n")
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}
