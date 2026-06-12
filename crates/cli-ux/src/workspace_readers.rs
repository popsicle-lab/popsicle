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

#[derive(Debug, Clone, Serialize)]
pub struct TaskFull {
    pub task_id: String,
    pub title: String,
    pub journey_stage: String,
    pub product: String,
    pub file_path: String,
    pub frontmatter: BTreeMap<String, serde_json::Value>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntentBlockDetail {
    pub name: String,
    pub kind: String,
    pub task_id: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntentFileFull {
    pub product: String,
    pub file: String,
    pub content: String,
    pub blocks: Vec<IntentBlockDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntentRef {
    pub reference: String,
    pub file: String,
    pub block: String,
    pub product: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposedTaskHint {
    pub title: String,
    pub journey_stage: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueGuidance {
    pub product: Option<String>,
    pub pipeline_stage: Option<String>,
    pub hint: String,
    pub linked_tasks: Vec<TaskNode>,
    pub proposed_tasks: Vec<ProposedTaskHint>,
    pub recommended_tasks: Vec<TaskNode>,
    pub related_intents: Vec<IntentRef>,
}

/// Scan `products/<product>/tasks/**/*.md` for one product.
pub fn scan_product_tasks(
    workspace_root: &Path,
    product: &str,
) -> Result<TaskGraph, WorkspaceError> {
    let tasks_root = workspace_root.join("products").join(product).join("tasks");
    let mut nodes = Vec::new();
    if tasks_root.is_dir() {
        walk_tasks(&tasks_root, product, &mut nodes)?;
    }
    nodes.sort_by(|a, b| a.task_id.cmp(&b.task_id));
    Ok(TaskGraph { nodes })
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

/// Resolve a CLI/UI product id, accepting legacy slice-style spec names.
pub fn resolve_product_id(workspace_root: &Path, input: &str) -> Result<String, WorkspaceError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceError::InvalidState(
            "product id required (products/<name>/)".into(),
        ));
    }
    let products = list_products(workspace_root)?;
    if products.iter().any(|p| p == trimmed) {
        return Ok(trimmed.to_string());
    }
    if let Some(p) = product_for_spec(trimmed, &products) {
        return Ok(p);
    }
    Err(WorkspaceError::InvalidState(format!(
        "unknown product '{trimmed}' (available: {})",
        if products.is_empty() {
            "none — create products/<name>/ first".into()
        } else {
            products.join(", ")
        }
    )))
}

/// Backfill `product_id` on legacy rows; normalize `spec_id` to the lock key.
pub fn backfill_issue_products(
    workspace_root: &Path,
    product_id: &mut String,
    spec_id: &mut String,
) {
    if product_id.is_empty() {
        let products = list_products(workspace_root).unwrap_or_default();
        *product_id = product_for_spec(spec_id, &products).unwrap_or_else(|| spec_id.clone());
    }
    *spec_id = product_id.clone();
}

/// Resolve configured default product id (accepts legacy spec-style values).
pub fn resolve_default_product(workspace_root: &Path, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let products = list_products(workspace_root).ok()?;
    if products.iter().any(|p| p == trimmed) {
        return Some(trimmed.to_string());
    }
    product_for_spec(trimmed, &products)
}

/// Map legacy issue `spec_id` to a `products/<name>/` directory.
pub fn product_for_spec(spec_id: &str, products: &[String]) -> Option<String> {
    if products.iter().any(|p| p == spec_id) {
        return Some(spec_id.to_string());
    }
    let mut best: Option<(usize, String)> = None;
    for p in products {
        if spec_id.ends_with(p) {
            let score = p.len();
            if best.as_ref().is_none_or(|(s, _)| score > *s) {
                best = Some((score, p.clone()));
            }
        } else if spec_id.contains(&format!("-{p}")) {
            let score = p.len();
            if best.as_ref().is_none_or(|(s, _)| score > *s) {
                best = Some((score, p.clone()));
            }
        }
    }
    best.map(|(_, p)| p)
}

pub fn read_task(
    workspace_root: &Path,
    task_id: &str,
    product: Option<&str>,
) -> Result<TaskFull, WorkspaceError> {
    let products = list_products(workspace_root)?;
    let candidates: Vec<String> = match product {
        Some(p) => vec![p.to_string()],
        None => products,
    };
    for p in candidates {
        let graph = scan_product_tasks(workspace_root, &p)?;
        if let Some(node) = graph.nodes.iter().find(|n| n.task_id == task_id) {
            let path = PathBuf::from(&node.file_path);
            let content = fs::read_to_string(&path).map_err(io_err)?;
            let (frontmatter, body) = split_task_content(&content).ok_or_else(|| {
                WorkspaceError::InvalidState(format!("task {task_id}: missing frontmatter"))
            })?;
            return Ok(TaskFull {
                task_id: node.task_id.clone(),
                title: node.title.clone(),
                journey_stage: node.journey_stage.clone(),
                product: node.product.clone(),
                file_path: node.file_path.clone(),
                frontmatter,
                body,
            });
        }
    }
    Err(WorkspaceError::NotFound(format!("task {task_id}")))
}

pub fn read_intent_file(
    workspace_root: &Path,
    product: &str,
    file: &str,
) -> Result<IntentFileFull, WorkspaceError> {
    let fname = if file.ends_with(".intent") {
        file.to_string()
    } else {
        format!("{file}.intent")
    };
    let path = workspace_root
        .join("products")
        .join(product)
        .join("intents")
        .join(&fname);
    let content = fs::read_to_string(&path)
        .map_err(|_| WorkspaceError::NotFound(format!("intent file {file}")))?;
    let fname = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let blocks = parse_intent_blocks_detailed(&content);
    Ok(IntentFileFull {
        product: product.to_string(),
        file: fname,
        content,
        blocks,
    })
}

pub fn resolve_intent_ref(
    workspace_root: &Path,
    reference: &str,
    product: Option<&str>,
) -> Result<IntentBlockDetail, WorkspaceError> {
    let (file, block_name) = reference
        .split_once('#')
        .ok_or_else(|| WorkspaceError::InvalidState(format!("invalid intent ref: {reference}")))?;
    let products = list_products(workspace_root)?;
    let candidates: Vec<String> = match product {
        Some(p) => vec![p.to_string()],
        None => products,
    };
    for p in candidates {
        if let Ok(intent_file) = read_intent_file(workspace_root, &p, file) {
            if let Some(block) = intent_file.blocks.iter().find(|b| b.name == block_name) {
                return Ok(block.clone());
            }
        }
    }
    Err(WorkspaceError::NotFound(format!(
        "intent block {reference}"
    )))
}

pub fn guidance_for_issue(
    workspace_root: &Path,
    product_id: &str,
    issue_type: &str,
    status: &str,
    pipeline_stage: Option<&str>,
    task_links: &[storage::IssueTaskLink],
) -> Result<IssueGuidance, WorkspaceError> {
    let products = list_products(workspace_root)?;
    let product = if product_id.is_empty() {
        None
    } else if products.iter().any(|p| p == product_id) {
        Some(product_id.to_string())
    } else {
        product_for_spec(product_id, &products)
    };

    let mut linked_tasks = Vec::new();
    let mut proposed_tasks = Vec::new();
    let mut recommended_tasks = Vec::new();
    let mut related_intents = Vec::new();

    if let Some(ref prod) = product {
        let graph = scan_product_tasks(workspace_root, prod)?;
        let node_by_id: std::collections::BTreeMap<String, TaskNode> = graph
            .nodes
            .into_iter()
            .map(|n| (n.task_id.clone(), n))
            .collect();

        for link in task_links {
            match link.role.as_str() {
                "linked" => {
                    if let Some(task_id) = link.task_id.as_ref() {
                        if let Some(node) = node_by_id.get(task_id) {
                            linked_tasks.push(node.clone());
                        }
                    }
                }
                "proposed" => {
                    if let Some(title) = link
                        .proposed_title
                        .as_ref()
                        .filter(|s: &&String| !s.is_empty())
                    {
                        proposed_tasks.push(ProposedTaskHint {
                            title: title.clone(),
                            journey_stage: link.journey_stage.clone(),
                            source: link.source.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        if linked_tasks.is_empty() && proposed_tasks.is_empty() {
            let mut scored: Vec<(i32, TaskNode)> = node_by_id
                .into_values()
                .map(|node| (score_task(&node, issue_type, status, pipeline_stage), node))
                .collect();
            scored.sort_by_key(|b| std::cmp::Reverse(b.0));
            recommended_tasks = scored.into_iter().take(3).map(|(_, n)| n).collect();
        }

        let hint = if !linked_tasks.is_empty() || !proposed_tasks.is_empty() {
            if let Some(stage) = pipeline_stage {
                format!("当前阶段「{stage}」— 已关联 task 优先；proposed 待 living-doc 晋升")
            } else {
                "已关联 task / 待创建 proposed task".to_string()
            }
        } else if let Some(stage) = pipeline_stage {
            format!("当前 pipeline 阶段「{stage}」— 启发式推荐（未关联 task）")
        } else {
            "未关联 task — 以下为启发式推荐".to_string()
        };

        let mut seen = std::collections::BTreeSet::new();
        let intent_sources: Vec<&TaskNode> = if linked_tasks.is_empty() {
            recommended_tasks.iter().collect()
        } else {
            linked_tasks.iter().collect()
        };
        for task in intent_sources {
            for ri in &task.related_intents {
                if seen.insert(ri.clone()) {
                    if let Some((file, block)) = ri.split_once('#') {
                        related_intents.push(IntentRef {
                            reference: ri.clone(),
                            file: file.to_string(),
                            block: block.to_string(),
                            product: prod.clone(),
                        });
                    }
                }
            }
        }

        return Ok(IssueGuidance {
            product,
            pipeline_stage: pipeline_stage.map(str::to_string),
            hint,
            linked_tasks,
            proposed_tasks,
            recommended_tasks,
            related_intents,
        });
    }

    Ok(IssueGuidance {
        product,
        pipeline_stage: pipeline_stage.map(str::to_string),
        hint: "无可用产品目录".to_string(),
        linked_tasks,
        proposed_tasks,
        recommended_tasks,
        related_intents,
    })
}

fn score_task(
    node: &TaskNode,
    issue_type: &str,
    status: &str,
    pipeline_stage: Option<&str>,
) -> i32 {
    let mut score = 0;
    if issue_type == "bug" && node.journey_stage == "troubleshooting" {
        score += 10;
    }
    if status == "in_progress" && node.journey_stage == "daily-ops" {
        score += 10;
    }
    if let Some(stage) = pipeline_stage {
        match stage {
            "implement" | "equivalence" => {
                if node.journey_stage == "daily-ops" {
                    score += 5;
                }
                if node.journey_stage == "lifecycle" {
                    score += 3;
                }
            }
            "cutover" | "living-docs" => {
                if node.journey_stage == "lifecycle" {
                    score += 8;
                }
            }
            _ => {
                if node.journey_stage == "daily-ops" {
                    score += 5;
                }
            }
        }
    }
    score
}

fn split_task_content(content: &str) -> Option<(BTreeMap<String, serde_json::Value>, String)> {
    let fm = content.strip_prefix("---")?;
    let rest = fm.strip_prefix('\n').or_else(|| fm.strip_prefix("\r\n"))?;
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n').to_string();
    let map: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(yaml).ok()?;
    let frontmatter = map
        .into_iter()
        .map(|(k, v)| (k, yaml_value_to_json(v)))
        .collect();
    Some((frontmatter, body))
}

fn yaml_value_to_json(v: serde_yaml::Value) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

fn parse_intent_blocks_detailed(content: &str) -> Vec<IntentBlockDetail> {
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let (kind, name) = if let Some(rest) = trimmed.strip_prefix("intent ") {
            ("intent", rest.split('(').next().unwrap_or("unknown").trim())
        } else if let Some(rest) = trimmed.strip_prefix("safety ") {
            ("safety", rest.split('(').next().unwrap_or("unknown").trim())
        } else {
            i += 1;
            continue;
        };
        let start_line = i + 1;
        let mut brace_depth = 0;
        let mut saw_open = false;
        let mut end_i = i;
        for (j, line) in lines.iter().enumerate().skip(i) {
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                    saw_open = true;
                } else if ch == '}' {
                    brace_depth -= 1;
                }
            }
            end_i = j;
            if saw_open && brace_depth == 0 {
                break;
            }
        }
        let end_line = end_i + 1;
        let snippet = lines[i..=end_i].join("\n");
        let mut task_id = None;
        if end_i + 1 < lines.len() {
            let next = lines[end_i + 1].trim();
            if let Some(t) = next.strip_prefix("// task:") {
                task_id = Some(t.trim().to_string());
            }
        }
        blocks.push(IntentBlockDetail {
            name: name.to_string(),
            kind: kind.to_string(),
            task_id,
            start_line,
            end_line,
            snippet,
        });
        i = end_i + 1;
    }
    blocks
}

pub fn list_products(workspace_root: &Path) -> Result<Vec<String>, WorkspaceError> {
    let products_dir = crate::project_config::products_dir_path(workspace_root);
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

/// Living-doc style health snapshot for the Products dashboard (read-only).
#[derive(Debug, Clone, Serialize)]
pub struct ProductHealthReport {
    pub product: String,
    pub task_count: u32,
    pub intent_block_count: u32,
    pub journey_stages: Vec<String>,
    pub unverified_tasks: u32,
    pub broken_refs: u32,
    pub orphan_intents: u32,
    pub has_product_md: bool,
    pub has_architecture_md: bool,
    pub health: String,
    pub hints: Vec<String>,
}

pub fn scan_product_health(
    workspace_root: &Path,
    product: &str,
) -> Result<ProductHealthReport, WorkspaceError> {
    let product_dir = workspace_root.join("products").join(product);
    let tasks = scan_product_tasks(workspace_root, product)?;
    let intents = scan_intents(workspace_root, product)?;
    let task_ids: std::collections::BTreeSet<String> =
        tasks.nodes.iter().map(|n| n.task_id.clone()).collect();

    let mut broken_refs = 0u32;
    let mut unverified_tasks = 0u32;
    let mut journey_stages = std::collections::BTreeSet::new();

    for node in &tasks.nodes {
        journey_stages.insert(node.journey_stage.clone());
        for next in &node.related_next_tasks {
            if !task_ids.contains(next) {
                broken_refs += 1;
            }
        }
        if task_last_verified_missing(&node.file_path)? {
            unverified_tasks += 1;
        }
    }

    let mut orphan_intents = 0u32;
    for block in &intents.blocks {
        if let Some(ref tid) = block.task_id {
            if !task_ids.contains(tid) {
                orphan_intents += 1;
            }
        }
    }

    let has_product_md = product_dir.join("PRODUCT.md").is_file();
    let has_architecture_md = product_dir.join("ARCHITECTURE.md").is_file();

    let mut hints = Vec::new();
    let mut health = "good".to_string();
    if broken_refs > 0 {
        health = "critical".into();
        hints.push(format!("{broken_refs} broken task cross-reference(s)"));
    }
    if orphan_intents > 0 {
        health = "critical".into();
        hints.push(format!(
            "{orphan_intents} intent block(s) reference missing tasks"
        ));
    }
    if unverified_tasks > 0 {
        if health == "good" {
            health = "warn".into();
        }
        hints.push(format!(
            "{unverified_tasks} task(s) missing last_verified (run intent-validate + living-doc)"
        ));
    }
    if !has_product_md {
        if health == "good" {
            health = "warn".into();
        }
        hints.push("PRODUCT.md missing".into());
    }

    Ok(ProductHealthReport {
        product: product.to_string(),
        task_count: tasks.nodes.len() as u32,
        intent_block_count: intents.blocks.len() as u32,
        journey_stages: journey_stages.into_iter().collect(),
        unverified_tasks,
        broken_refs,
        orphan_intents,
        has_product_md,
        has_architecture_md,
        health,
        hints,
    })
}

fn task_last_verified_missing(path: &str) -> Result<bool, WorkspaceError> {
    let content = fs::read_to_string(path).map_err(io_err)?;
    let Some(yaml) = extract_frontmatter_yaml(&content) else {
        return Ok(true);
    };
    let map: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(&yaml).unwrap_or_default();
    let lv = map.get("last_verified");
    Ok(match lv {
        None => true,
        Some(serde_yaml::Value::Null) => true,
        Some(serde_yaml::Value::String(s)) => {
            let t = s.trim();
            t.is_empty() || t == "~" || t == "null"
        }
        _ => false,
    })
}

fn extract_frontmatter_yaml(content: &str) -> Option<String> {
    let fm = content.strip_prefix("---")?;
    let rest = fm.strip_prefix('\n').or_else(|| fm.strip_prefix("\r\n"))?;
    let end = rest.find("\n---")?;
    Some(rest[..end].to_string())
}

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}
