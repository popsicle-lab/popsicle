//! Merged-product goal traceability checks (`realized_by` on L4 goals).

use std::fs;
use std::path::{Path, PathBuf};

use intent_lang_syntax::ast::{Declaration, Program};
use intent_lang_syntax::parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalTraceFinding {
    pub product: String,
    pub goal: String,
    pub code: String,
    pub message: String,
}

/// When `validate_path` is under `products/`, merge each product's `intents/*.intent`
/// and require every `goal` to have non-empty `realized_by` pointing at declared
/// safety/intent/theorem names. Products with `contracts.intent` must declare at least
/// one goal (`E_PRODUCT_MISSING_GOALS` when absent).
pub fn check_products_goal_trace(
    workspace_root: &Path,
    validate_path: &str,
) -> Result<Vec<GoalTraceFinding>, String> {
    let target = resolve_validate_path(workspace_root, validate_path);
    let mut findings = Vec::new();
    for (product, intents_dir) in discover_product_intent_dirs(&target)? {
        findings.extend(check_one_product(&product, &intents_dir)?);
    }
    Ok(findings)
}

/// Product `intents/` dirs implied by `validate_path` (empty for a single `.intent` file).
/// Reused by the opt-in merged whole-program check (`intent-validate merge=true`, feedback #14).
pub fn product_intent_dirs(
    workspace_root: &Path,
    validate_path: &str,
) -> Result<Vec<(String, PathBuf)>, String> {
    let target = resolve_validate_path(workspace_root, validate_path);
    discover_product_intent_dirs(&target)
}

/// Concatenate a product's `intents/*.intent` into one whole-program source
/// (sorted, deterministic) so a single `intent check` sees cross-file `realized_by`
/// without the per-file `W0010` noise (feedback #14).
pub fn merge_product_intents(intents_dir: &Path) -> Result<String, String> {
    merge_intent_sources(intents_dir)
}

pub fn print_goal_trace_json(findings: &[GoalTraceFinding]) {
    for f in findings {
        let line = serde_json::json!({
            "product": f.product,
            "goal": f.goal,
            "diagnostics": [{
                "level": "error",
                "code": f.code,
                "message": f.message,
            }],
            "ok": false,
        });
        println!("{line}");
    }
}

pub fn print_goal_trace_text(findings: &[GoalTraceFinding]) {
    for f in findings {
        eprintln!("{} [{}] {} — {}", f.product, f.code, f.goal, f.message);
    }
}

fn resolve_validate_path(workspace_root: &Path, validate_path: &str) -> PathBuf {
    let p = PathBuf::from(validate_path);
    if p.is_absolute() {
        p
    } else {
        workspace_root.join(p)
    }
}

fn discover_product_intent_dirs(target: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    if target.extension().and_then(|e| e.to_str()) == Some("intent") {
        return Ok(Vec::new());
    }

    if target.file_name().and_then(|n| n.to_str()) == Some("intents") && target.is_dir() {
        let product = target
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("cannot infer product from {}", target.display()))?;
        return Ok(vec![(product.to_string(), target.to_path_buf())]);
    }

    if target.join("intents").is_dir() {
        let product = target
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("cannot infer product from {}", target.display()))?;
        return Ok(vec![(product.to_string(), target.join("intents"))]);
    }

    if target.file_name().and_then(|n| n.to_str()) == Some("products") && target.is_dir() {
        let mut out = Vec::new();
        for entry in fs::read_dir(target).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let intents = entry.path().join("intents");
            if intents.is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                out.push((name, intents));
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        return Ok(out);
    }

    Ok(Vec::new())
}

fn check_one_product(product: &str, intents_dir: &Path) -> Result<Vec<GoalTraceFinding>, String> {
    let merged = merge_intent_sources(intents_dir)?;
    if merged.trim().is_empty() {
        return Ok(Vec::new());
    }
    let program = parse(&merged)
        .map_err(|e| format!("{product}: merged intents parse error: {}", e.message))?;
    let requires_goals = intents_dir.join("contracts.intent").is_file();
    Ok(audit_goal_trace(product, &program, requires_goals))
}

fn merge_intent_sources(intents_dir: &Path) -> Result<String, String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(intents_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("intent"))
        .collect();
    paths.sort();
    let mut merged = String::new();
    for path in paths {
        merged.push_str(&fs::read_to_string(&path).map_err(|e| e.to_string())?);
        merged.push('\n');
    }
    Ok(merged)
}

fn audit_goal_trace(
    product: &str,
    program: &Program,
    requires_goals: bool,
) -> Vec<GoalTraceFinding> {
    let mut decls = std::collections::HashSet::new();
    for sp in &program.declarations {
        match &sp.node {
            Declaration::Intent(i) => {
                decls.insert(i.name.clone());
            }
            Declaration::Safety(s) => {
                decls.insert(s.name.clone());
            }
            Declaration::Theorem(t) => {
                decls.insert(t.name.clone());
            }
            _ => {}
        }
    }

    let goal_count = program
        .declarations
        .iter()
        .filter(|sp| matches!(sp.node, Declaration::Goal(_)))
        .count();

    let mut findings = Vec::new();
    if requires_goals && goal_count == 0 {
        findings.push(GoalTraceFinding {
            product: product.to_string(),
            goal: "(contracts)".into(),
            code: "E_PRODUCT_MISSING_GOALS".into(),
            message: "contracts.intent exists but merged program has no goal blocks — add goals with realized_by (intent-spec-writer Step 4)".into(),
        });
        return findings;
    }

    for sp in &program.declarations {
        let Declaration::Goal(g) = &sp.node else {
            continue;
        };
        if g.realized_by.is_empty() {
            findings.push(GoalTraceFinding {
                product: product.to_string(),
                goal: g.name.clone(),
                code: "E_GOAL_UNLINKED".into(),
                message: format!(
                    "goal `{}` has empty realized_by — link to acceptance/invariants safety or intent after merge",
                    g.name
                ),
            });
            continue;
        }
        for ref_name in &g.realized_by {
            if !decls.contains(ref_name) {
                findings.push(GoalTraceFinding {
                    product: product.to_string(),
                    goal: g.name.clone(),
                    code: "E_GOAL_UNKNOWN_REF".into(),
                    message: format!(
                        "goal `{}` realized_by references unknown declaration `{ref_name}` in merged program",
                        g.name
                    ),
                });
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_products_pass_with_realized_by() {
        let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let findings = check_products_goal_trace(&ws, "products").expect("check");
        assert!(
            findings.is_empty(),
            "expected no goal trace findings, got: {findings:?}"
        );
    }

    #[test]
    fn orphan_goal_is_reported() {
        let src = r#"
goal "orphan" {
  rationale: "x"
  stakeholder: ["a"]
  measure: "y"
}
intent Foo(x: Int) {
  require true
  ensure x' == x
}
"#;
        let program = parse(src).unwrap();
        let findings = audit_goal_trace("test", &program, false);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "E_GOAL_UNLINKED");
    }

    #[test]
    fn missing_goals_when_contracts_file_expected() {
        let src = r#"
@tobe
intent Foo(x: Int) {
  require true
  ensure x' == x
}
"#;
        let program = parse(src).unwrap();
        let findings = audit_goal_trace("telemetry", &program, true);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "E_PRODUCT_MISSING_GOALS");
    }
}
