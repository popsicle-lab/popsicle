//! Machine-enforced pipeline stage gates (feedback #18/#19/P3).
//!
//! Gates are the "machine" verification axis, evaluated at `stage complete`
//! *before* the human approval axis. They are **fail-closed** and
//! `approval_mode: auto` cannot bypass them (P4/H6): the whole point is that a
//! run cannot advance past e.g. `cutover` unless `cargo test` really exits 0 and
//! the golden numbers in `baseline.yaml` actually recompute.

use std::path::{Path, PathBuf};
use std::process::Command;

use skill_runtime::{AssertGate, GateSpec, ManifestRecomputesGate, RefResolvableGate};

/// One gate's verdict. `detail` carries recomputed evidence for the failure.
#[derive(Debug, Clone)]
pub struct GateReport {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Evaluate every gate on a stage. Stops reporting nothing early — returns all
/// verdicts so callers can log the full picture; the caller fails on the first
/// `!passed`.
pub fn evaluate_stage_gates(
    workspace_root: &Path,
    product: &str,
    run_id: &str,
    gates: &[GateSpec],
) -> Vec<GateReport> {
    gates
        .iter()
        .map(|g| evaluate_one(workspace_root, product, run_id, g))
        .collect()
}

fn evaluate_one(workspace_root: &Path, product: &str, run_id: &str, gate: &GateSpec) -> GateReport {
    match gate {
        GateSpec::CommandExitZero(cmd) => eval_command_exit_zero(workspace_root, cmd),
        GateSpec::Assert(a) => eval_assert(workspace_root, product, run_id, a),
        GateSpec::ManifestRecomputes(m) => {
            eval_manifest_recomputes(workspace_root, product, run_id, m)
        }
        GateSpec::RefResolvable(r) => eval_ref_resolvable(workspace_root, product, r),
    }
}

fn eval_command_exit_zero(workspace_root: &Path, cmd: &str) -> GateReport {
    let name = format!("command_exit_zero:{cmd}");
    let result = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(workspace_root)
        .status();
    match result {
        Ok(status) if status.success() => GateReport {
            name,
            passed: true,
            detail: "exit 0".into(),
        },
        Ok(status) => GateReport {
            name,
            passed: false,
            detail: format!("exit {}", status.code().unwrap_or(-1)),
        },
        Err(e) => GateReport {
            name,
            passed: false,
            detail: format!("failed to run: {e}"),
        },
    }
}

fn eval_assert(workspace_root: &Path, product: &str, run_id: &str, a: &AssertGate) -> GateReport {
    let name = format!("assert:{}#{} {} {:?}", a.file, a.field, a.op, a.value);
    let file = match resolve_file(workspace_root, product, run_id, &a.file) {
        Ok(p) => p,
        Err(e) => return fail(name, e),
    };
    let doc = match load_yaml(&file) {
        Ok(v) => v,
        Err(e) => return fail(name, e),
    };
    let Some(field) = lookup_path(&doc, &a.field) else {
        return fail(
            name,
            format!("field `{}` not found in {}", a.field, file.display()),
        );
    };
    match compare(field, &a.op, &a.value) {
        Ok(true) => GateReport {
            name,
            passed: true,
            detail: format!("{:?} {} {:?}", field, a.op, a.value),
        },
        Ok(false) => fail(
            name,
            format!(
                "{}={:?} does not satisfy {} {:?}",
                a.field, field, a.op, a.value
            ),
        ),
        Err(e) => fail(name, e),
    }
}

fn eval_manifest_recomputes(
    workspace_root: &Path,
    product: &str,
    run_id: &str,
    m: &ManifestRecomputesGate,
) -> GateReport {
    let name = format!(
        "manifest_recomputes:{}#{}=count({}{})",
        m.file,
        m.field,
        m.equals_count_of,
        m.where_clause
            .as_ref()
            .map(|w| format!(" where {w}"))
            .unwrap_or_default()
    );
    let file = match resolve_file(workspace_root, product, run_id, &m.file) {
        Ok(p) => p,
        Err(e) => return fail(name, e),
    };
    let doc = match load_yaml(&file) {
        Ok(v) => v,
        Err(e) => return fail(name, e),
    };
    let Some(field_val) = lookup_path(&doc, &m.field).and_then(as_i64) else {
        return fail(name, format!("field `{}` missing or non-integer", m.field));
    };
    let Some(list) = lookup_path(&doc, &m.equals_count_of).and_then(|v| v.as_sequence()) else {
        return fail(
            name,
            format!("list `{}` missing or not a sequence", m.equals_count_of),
        );
    };
    let where_pair = m.where_clause.as_ref().and_then(|w| w.split_once('='));
    let count = list
        .iter()
        .filter(|item| match where_pair {
            None => true,
            Some((k, v)) => item
                .get(k)
                .map(|iv| scalar_to_string(iv) == v.trim())
                .unwrap_or(false),
        })
        .count() as i64;
    if field_val == count {
        GateReport {
            name,
            passed: true,
            detail: format!("{}={count} matches recomputed count", m.field),
        }
    } else {
        fail(
            name,
            format!(
                "{}={field_val} but recomputed count={count} — summary number does not match itemized list",
                m.field
            ),
        )
    }
}

fn eval_ref_resolvable(workspace_root: &Path, product: &str, r: &RefResolvableGate) -> GateReport {
    let name = format!("ref_resolvable:{:?}", r.fields);
    if !r.product_intents {
        return GateReport {
            name,
            passed: true,
            detail: "no product_intents check requested".into(),
        };
    }
    let path = if product.is_empty() {
        "products".to_string()
    } else {
        format!("products/{product}/intents")
    };
    match crate::intent_goal_trace::check_products_goal_trace(workspace_root, &path) {
        Ok(findings) if findings.is_empty() => GateReport {
            name,
            passed: true,
            detail: "all goal realized_by references resolve".into(),
        },
        Ok(findings) => {
            let first = &findings[0];
            fail(
                name,
                format!(
                    "{} unresolved reference(s); e.g. {} [{}] {}",
                    findings.len(),
                    first.product,
                    first.code,
                    first.message
                ),
            )
        }
        Err(e) => fail(name, format!("goal-trace error: {e}")),
    }
}

fn fail(name: String, detail: impl Into<String>) -> GateReport {
    GateReport {
        name,
        passed: false,
        detail: detail.into(),
    }
}

/// Resolve a gate `file` spec: interpolate `{run_id}`/`{product}`, then treat a
/// `*`/`**` pattern as "newest matching file". Non-glob paths resolve directly.
fn resolve_file(
    workspace_root: &Path,
    product: &str,
    run_id: &str,
    spec: &str,
) -> Result<PathBuf, String> {
    let interpolated = spec
        .replace("{run_id}", run_id)
        .replace("{product}", product);
    if interpolated.contains('*') {
        newest_glob_match(workspace_root, &interpolated).ok_or_else(|| {
            format!(
                "no file matches `{interpolated}` under {}",
                workspace_root.display()
            )
        })
    } else {
        let p = workspace_root.join(&interpolated);
        if p.is_file() {
            Ok(p)
        } else {
            Err(format!("file not found: {interpolated}"))
        }
    }
}

/// Minimal glob: `<fixed>/**/<filename>` or `<fixed>/*/<filename>` — walks the
/// fixed prefix recursively for files named `<filename>` and returns the newest.
fn newest_glob_match(workspace_root: &Path, pattern: &str) -> Option<PathBuf> {
    let (prefix, filename) = match pattern.rsplit_once('/') {
        Some((p, f)) => (p.trim_end_matches("/**").trim_end_matches("/*"), f),
        None => ("", pattern),
    };
    // Strip any remaining glob segments from the prefix, keeping the fixed head.
    let fixed: PathBuf = prefix.split('/').take_while(|seg| !seg.contains('*')).fold(
        workspace_root.to_path_buf(),
        |acc, seg| {
            if seg.is_empty() {
                acc
            } else {
                acc.join(seg)
            }
        },
    );
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    collect_named(&fixed, filename, &mut best);
    best.map(|(_, p)| p)
}

fn collect_named(dir: &Path, filename: &str, best: &mut Option<(std::time::SystemTime, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_named(&path, filename, best);
        } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            if best.as_ref().map(|(t, _)| mtime > *t).unwrap_or(true) {
                *best = Some((mtime, path));
            }
        }
    }
}

fn load_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_yaml::from_str(&content).map_err(|e| format!("parse {}: {e}", path.display()))
}

/// Navigate a dotted path (`meta.legacy_pin`) into a YAML mapping.
fn lookup_path<'a>(root: &'a serde_yaml::Value, path: &str) -> Option<&'a serde_yaml::Value> {
    let mut cur = root;
    for key in path.split('.') {
        cur = cur.get(key)?;
    }
    Some(cur)
}

fn as_i64(v: &serde_yaml::Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))
}

fn scalar_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// Compare a manifest field against `op value`. Numeric ops when `value` is a
/// number; string `==`/`!=` otherwise.
fn compare(field: &serde_yaml::Value, op: &str, value: &serde_yaml::Value) -> Result<bool, String> {
    if let Some(expected) = value.as_f64() {
        let actual = field
            .as_f64()
            .or_else(|| field.as_i64().map(|i| i as f64))
            .ok_or_else(|| format!("field value {field:?} is not numeric for op `{op}`"))?;
        return Ok(match op {
            ">=" => actual >= expected,
            ">" => actual > expected,
            "<=" => actual <= expected,
            "<" => actual < expected,
            "==" => (actual - expected).abs() < f64::EPSILON,
            "!=" => (actual - expected).abs() >= f64::EPSILON,
            _ => return Err(format!("unknown numeric op `{op}`")),
        });
    }
    let expected = scalar_to_string(value);
    let actual = scalar_to_string(field);
    match op {
        "==" => Ok(actual == expected),
        "!=" => Ok(actual != expected),
        _ => Err(format!("op `{op}` needs a numeric value; got {value:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "popsicle-gate-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn command_exit_zero_pass_and_fail() {
        let root = tmp();
        let ok = eval_command_exit_zero(&root, "true");
        assert!(ok.passed, "{}", ok.detail);
        let bad = eval_command_exit_zero(&root, "exit 3");
        assert!(!bad.passed);
        assert!(bad.detail.contains('3'));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn assert_numeric_and_manifest_recompute() {
        let root = tmp();
        let dir = root.join("docs/baseline/2026-07-13");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("baseline.yaml"),
            "golden_pass: 2\ngoldens:\n  - {id: g1, status: pass}\n  - {id: g2, status: pass}\n  - {id: g3, status: fail}\n",
        )
        .unwrap();

        let a = AssertGate {
            file: "docs/baseline/**/baseline.yaml".into(),
            field: "golden_pass".into(),
            op: ">=".into(),
            value: serde_yaml::Value::Number(2.into()),
        };
        let r = eval_assert(&root, "store", "run1", &a);
        assert!(r.passed, "{}", r.detail);

        // golden_pass=2 but only 2 of 3 goldens are status=pass → recompute matches.
        let m = ManifestRecomputesGate {
            file: "docs/baseline/**/baseline.yaml".into(),
            field: "golden_pass".into(),
            equals_count_of: "goldens".into(),
            where_clause: Some("status=pass".into()),
        };
        let ok = eval_manifest_recomputes(&root, "store", "run1", &m);
        assert!(ok.passed, "{}", ok.detail);

        // Fabricate: bump golden_pass to a lie → recompute catches it.
        std::fs::write(
            dir.join("baseline.yaml"),
            "golden_pass: 9\ngoldens:\n  - {id: g1, status: pass}\n  - {id: g2, status: pass}\n  - {id: g3, status: fail}\n",
        )
        .unwrap();
        let caught = eval_manifest_recomputes(&root, "store", "run1", &m);
        assert!(!caught.passed);
        assert!(caught.detail.contains("count=2"), "{}", caught.detail);
        let _ = std::fs::remove_dir_all(root);
    }
}
