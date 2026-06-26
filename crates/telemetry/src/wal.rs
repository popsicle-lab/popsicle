use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalLine {
    pub ts: String,
    pub span: String,
    #[serde(flatten)]
    pub attributes: BTreeMap<String, String>,
}

pub fn telemetry_run_dir(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(".popsicle/telemetry").join(run_id)
}

pub fn telemetry_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".popsicle/telemetry")
}

pub fn wal_path(workspace_root: &Path, run_id: &str) -> PathBuf {
    telemetry_run_dir(workspace_root, run_id).join("spans.wal.jsonl")
}

pub fn append_span(
    workspace_root: &Path,
    run_id: &str,
    span_name: &str,
    attributes: &BTreeMap<String, String>,
) -> Result<String, String> {
    let dir = telemetry_run_dir(workspace_root, run_id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = wal_path(workspace_root, run_id);
    let now = SystemTime::now();
    let mut attrs = redact_attributes(attributes);
    if let Some(ms) = duration_since_last_span(&path, now) {
        attrs.insert("popsicle.duration_ms".into(), ms.to_string());
    }
    let line = WalLine {
        ts: format_system_time(now),
        span: span_name.to_string(),
        attributes: attrs,
    };
    let json = serde_json::to_string(&line).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{json}").map_err(|e| e.to_string())?;
    Ok(path
        .strip_prefix(workspace_root)
        .unwrap_or(&path)
        .display()
        .to_string())
}

/// Read all WAL lines for a run; missing file → empty vec (fail-open).
pub fn read_wal_lines(workspace_root: &Path, run_id: &str) -> Vec<WalLine> {
    let path = wal_path(workspace_root, run_id);
    if !path.is_file() {
        return vec![];
    }
    let Ok(file) = fs::File::open(&path) else {
        return vec![];
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(&l).ok())
        .collect()
}

fn duration_since_last_span(path: &Path, now: SystemTime) -> Option<u64> {
    let last = read_last_line(path)?;
    let prev_line: WalLine = serde_json::from_str(&last).ok()?;
    let prev = parse_wal_ts(&prev_line.ts)?;
    now.duration_since(prev).ok().map(|d| d.as_millis() as u64)
}

fn read_last_line(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    reader.lines().map_while(Result::ok).last()
}

fn parse_wal_ts(ts: &str) -> Option<SystemTime> {
    let ts = ts.strip_suffix('Z')?;
    let (secs, millis) = ts.split_once('.')?;
    let secs: u64 = secs.parse().ok()?;
    let millis: u32 = millis.parse().ok()?;
    Some(UNIX_EPOCH + Duration::from_secs(secs) + Duration::from_millis(millis as u64))
}

fn format_system_time(time: SystemTime) -> String {
    let dur = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
}

fn redact_attributes(attrs: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    attrs
        .iter()
        .map(|(k, v)| {
            if is_sensitive_attribute_key(k) {
                (k.clone(), "[REDACTED]".into())
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

/// Redact secrets but preserve OTel token *counts* (`input_tokens`, `output_tokens`, …).
fn is_sensitive_attribute_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "input_tokens" | "output_tokens" | "total_tokens" | "prompt_tokens" | "completion_tokens"
    ) {
        return false;
    }
    const SENSITIVE: &[&str] = &["authorization", "password", "api_key", "secret", "bearer"];
    if SENSITIVE.iter().any(|s| lower.contains(s)) {
        return true;
    }
    lower == "token" || lower.ends_with("_token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_counts_are_not_redacted() {
        let tmp = std::env::temp_dir().join(format!("wal-redact-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let run = "run-redact";
        let mut attrs = BTreeMap::new();
        attrs.insert("input_tokens".into(), "1200".into());
        attrs.insert("output_tokens".into(), "400".into());
        attrs.insert("access_token".into(), "secret-value".into());
        append_span(&tmp, run, "gen_ai.chat", &attrs).unwrap();
        let content = fs::read_to_string(wal_path(&tmp, run)).unwrap();
        let line: WalLine = serde_json::from_str(content.lines().next().unwrap()).unwrap();
        assert_eq!(
            line.attributes.get("input_tokens").map(String::as_str),
            Some("1200")
        );
        assert_eq!(
            line.attributes.get("output_tokens").map(String::as_str),
            Some("400")
        );
        assert_eq!(
            line.attributes.get("access_token").map(String::as_str),
            Some("[REDACTED]")
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn duration_ms_on_second_append() {
        let tmp = std::env::temp_dir().join(format!("wal-dur-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let run = "run-dur";
        let mut a = BTreeMap::new();
        a.insert("k".into(), "v".into());
        append_span(&tmp, run, "first", &a).unwrap();
        append_span(&tmp, run, "second", &a).unwrap();
        let content = fs::read_to_string(wal_path(&tmp, run)).unwrap();
        assert!(content.lines().count() >= 2);
        let last: WalLine = serde_json::from_str(content.lines().last().unwrap()).unwrap();
        assert!(last.attributes.contains_key("popsicle.duration_ms"));
    }
}
