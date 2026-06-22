//! Sanitize intent-lang-visualizer Mermaid for Mermaid 10+/11.
//!
//! The upstream visualizer only replaces spaces/hyphens in node IDs; goal names with
//! CJK or punctuation (e.g. `、`) break `mermaid.render()` in the browser.

use std::collections::BTreeMap;

/// Rewrite node IDs to ASCII `n0`, `n1`, … while preserving quoted labels.
pub fn sanitize_mermaid_for_render(input: &str) -> String {
    let mut id_map: BTreeMap<String, String> = BTreeMap::new();
    let mut next = 0u32;

    let mut out = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("graph ") || trimmed.starts_with("classDef") {
            out.push(line.to_string());
            continue;
        }

        if let Some(rewritten) = rewrite_node_line(line, &mut id_map, &mut next) {
            out.push(rewritten);
        } else if let Some(rewritten) = rewrite_edge_line(line, &id_map) {
            out.push(rewritten);
        } else {
            out.push(line.to_string());
        }
    }
    out.join("\n")
}

fn rewrite_node_line(
    line: &str,
    map: &mut BTreeMap<String, String>,
    next: &mut u32,
) -> Option<String> {
    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let content = line.trim_start();
    let shape_idx = content.find(['[', '('])?;
    if shape_idx == 0 {
        return None;
    }
    let old_id = content[..shape_idx].trim();
    if old_id.is_empty() {
        return None;
    }
    let new_id = if is_safe_mermaid_id(old_id) {
        old_id.to_string()
    } else {
        allocate_id(old_id, map, next)
    };
    let suffix = &content[shape_idx..];
    Some(format!("{indent}{new_id}{suffix}"))
}

fn rewrite_edge_line(line: &str, map: &BTreeMap<String, String>) -> Option<String> {
    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let content = line.trim_start();

    for arrow in ["-.->", "==>", "-->", "---"] {
        let Some(idx) = content.find(arrow) else {
            continue;
        };
        let from = content[..idx].trim();
        let rest = content[idx + arrow.len()..].trim();
        let (label, to) = parse_edge_tail(rest)?;
        let from = map.get(from).cloned().unwrap_or_else(|| from.to_string());
        let to = map.get(&to).cloned().unwrap_or(to);
        let mid = if let Some(l) = label {
            format!(" {arrow}|{l}| ")
        } else {
            format!(" {arrow} ")
        };
        return Some(format!("{indent}{from}{mid}{to}"));
    }
    None
}

fn parse_edge_tail(rest: &str) -> Option<(Option<String>, String)> {
    if let Some(after) = rest.strip_prefix('|') {
        let end = after.find('|')?;
        let label = after[..end].to_string();
        let to = after[end + 1..].trim();
        Some((Some(label), to.to_string()))
    } else {
        Some((None, rest.to_string()))
    }
}

fn allocate_id(raw: &str, map: &mut BTreeMap<String, String>, next: &mut u32) -> String {
    if let Some(existing) = map.get(raw) {
        return existing.clone();
    }
    let id = format!("n{next}");
    *next += 1;
    map.insert(raw.to_string(), id.clone());
    id
}

fn is_safe_mermaid_id(id: &str) -> bool {
    let Some(first) = id.chars().next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_cjk_goal_ids() {
        let raw = r#"graph TD
    guard_与_extractor_对任意输入全函数、不_panic["guard 与 extractor 对任意输入全函数、不 panic"]:::goalNode
    UnknownGuardIsInvalid("UnknownGuardIsInvalid"):::safetyNode
    guard_与_extractor_对任意输入全函数、不_panic -->|realized_by| UnknownGuardIsInvalid
    classDef goalNode fill:#e1f5ff"#;
        let out = sanitize_mermaid_for_render(raw);
        assert!(out.contains("n0[\"guard 与 extractor"));
        assert!(!out.contains("guard_与_extractor_对任意输入全函数、不_panic"));
        assert!(out.contains("n0 -->|realized_by| UnknownGuardIsInvalid"));
        assert!(is_safe_mermaid_id("UnknownGuardIsInvalid"));
    }

    #[test]
    fn preserves_ascii_intent_ids() {
        let raw = r#"graph TD
    IssueStartCreatesRun(("IssueStartCreatesRun")):::intentNode"#;
        let out = sanitize_mermaid_for_render(raw);
        assert!(out.contains("IssueStartCreatesRun"));
    }
}
