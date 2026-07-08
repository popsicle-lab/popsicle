//! Parse cursor-agent `--output-format stream-json` NDJSON for run log lines.

use serde_json::Value;

/// Whether to emit character-level assistant deltas (noisier run logs).
pub fn stream_partial_enabled() -> bool {
    matches!(
        std::env::var("AGENT_RUNTIME_AGENT_STREAM_PARTIAL").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

/// Map one NDJSON line to a human-readable run-log message, if any.
pub fn format_stream_event(line: &str) -> Option<String> {
    let v: Value = serde_json::from_str(line.trim()).ok()?;
    let typ = v.get("type")?.as_str()?;
    match typ {
        "system" if v.get("subtype").and_then(|s| s.as_str()) == Some("init") => {
            let model = v.get("model").and_then(|m| m.as_str()).unwrap_or("unknown");
            let session = v.get("session_id").and_then(|s| s.as_str()).unwrap_or("?");
            Some(format!("agent: start model={model} session={session}"))
        }
        "assistant" => format_assistant_event(&v),
        "tool_call" => format_tool_call_event(&v),
        "result" => {
            let dur = v.get("duration_ms").and_then(|d| d.as_u64()).unwrap_or(0);
            let err = v.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
            let subtype = v
                .get("subtype")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            Some(format!(
                "agent: finished subtype={subtype} duration_ms={dur} is_error={err}"
            ))
        }
        _ => None,
    }
}

fn format_assistant_event(v: &Value) -> Option<String> {
    if stream_partial_enabled() {
        // Skip buffered duplicates before tool calls and final flush (Cursor CLI docs).
        if v.get("model_call_id").is_some() {
            return None;
        }
        v.get("timestamp_ms")?;
    }
    assistant_text(v).map(|text| truncate_line(&format!("› {text}"), 500))
}

fn format_tool_call_event(v: &Value) -> Option<String> {
    let subtype = v.get("subtype").and_then(|s| s.as_str()).unwrap_or("event");
    let tool_call = v.get("tool_call")?;
    let (kind, detail) = summarize_tool_call(tool_call, subtype == "completed");
    let prefix = if subtype == "started" {
        "agent: tool ▶"
    } else {
        "agent: tool ✓"
    };
    Some(format!("{prefix} {kind} {detail}"))
}

/// Prose for chat bubbles — same source as run-log `›` lines (assistant events only).
/// Ignores `result` payloads that often include tool noise and overly long dumps.
pub fn collect_assistant_prose(stdout: &str) -> String {
    let mut chunks: Vec<String> = Vec::new();
    let mut saw_stream = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        saw_stream = true;
        if v.get("type").and_then(|t| t.as_str()) != Some("assistant") {
            continue;
        }
        if stream_partial_enabled() && v.get("model_call_id").is_some() {
            continue;
        }
        if let Some(text) = assistant_text(&v) {
            chunks.push(text);
        }
    }

    if !saw_stream {
        // Chat must never surface raw NDJSON when stream events fail to parse.
        return String::new();
    }
    merge_assistant_snapshots(&chunks)
}

/// Final `result` payload from stream-json (fallback when assistant events are empty).
pub fn collect_result_prose(stdout: &str) -> String {
    let mut last = String::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if v.get("type").and_then(|t| t.as_str()) != Some("result") {
            continue;
        }
        if v.get("is_error").and_then(|e| e.as_bool()) == Some(true) {
            continue;
        }
        if let Some(text) = v.get("result").and_then(|r| r.as_str()) {
            let t = text.trim();
            if !t.is_empty() {
                last = t.to_string();
            }
        }
    }
    last
}

/// Strip the `›` prefix from a formatted run-log line.
pub fn strip_assistant_log_marker(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    trimmed
        .strip_prefix('›')
        .map(|rest| rest.trim_start())
        .filter(|rest| !rest.is_empty())
}

/// Merge streamed `›` log lines (handles cumulative snapshots + partial fragments).
pub fn merge_assistant_log_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    if stream_partial_enabled() {
        return lines.join("");
    }
    let mut kept: Vec<String> = Vec::new();
    for line in lines {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        kept.retain(|prev| !t.starts_with(prev.as_str()));
        if kept.last().is_some_and(|prev| prev.starts_with(t)) {
            continue;
        }
        kept.push(t.to_string());
    }
    kept.into_iter().last().unwrap_or_default()
}

fn merge_assistant_snapshots(chunks: &[String]) -> String {
    if chunks.is_empty() {
        return String::new();
    }
    if stream_partial_enabled() {
        return chunks.join("");
    }
    chunks
        .iter()
        .max_by_key(|s| s.len())
        .cloned()
        .unwrap_or_default()
}

pub fn assistant_text(v: &Value) -> Option<String> {
    let content = v.get("message")?.get("content")?.as_array()?;
    let mut out = String::new();
    for item in content {
        if item.get("type").and_then(|t| t.as_str()) == Some("text") {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                out.push_str(text);
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn summarize_tool_call(tool_call: &Value, completed: bool) -> (String, String) {
    if let Some(read) = tool_call.get("readToolCall") {
        let path = read
            .get("args")
            .and_then(|a| a.get("path"))
            .and_then(|p| p.as_str())
            .unwrap_or("?");
        if completed {
            let lines = read
                .get("result")
                .and_then(|r| r.get("success"))
                .and_then(|s| s.get("totalLines"))
                .and_then(|l| l.as_u64())
                .map(|n| format!("lines={n}"))
                .unwrap_or_default();
            return ("read".into(), format!("path={path} {lines}"));
        }
        return ("read".into(), format!("path={path}"));
    }
    if let Some(write) = tool_call.get("writeToolCall") {
        let path = write
            .get("args")
            .and_then(|a| a.get("path"))
            .and_then(|p| p.as_str())
            .unwrap_or("?");
        if completed {
            let lines = write
                .get("result")
                .and_then(|r| r.get("success"))
                .and_then(|s| s.get("linesCreated"))
                .and_then(|l| l.as_u64())
                .map(|n| format!("lines={n}"))
                .unwrap_or_default();
            return ("write".into(), format!("path={path} {lines}"));
        }
        return ("write".into(), format!("path={path}"));
    }
    if let Some(shell) = tool_call.get("shellToolCall") {
        let cmd = shell
            .get("args")
            .and_then(|a| a.get("command"))
            .and_then(|c| c.as_str())
            .unwrap_or("?");
        return ("shell".into(), truncate_line(&format!("cmd={cmd}"), 200));
    }
    if let Some(func) = tool_call.get("function") {
        let name = func
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("function");
        return (name.into(), String::new());
    }
    // Unknown nested tool keys — list first key name.
    if let Some(obj) = tool_call.as_object() {
        if let Some((key, _)) = obj.iter().next() {
            return (key.clone(), String::new());
        }
    }
    ("tool".into(), String::new())
}

fn truncate_line(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_system_init() {
        let line = r#"{"type":"system","subtype":"init","model":"Claude 4","session_id":"abc"}"#;
        let msg = format_stream_event(line).expect("msg");
        assert!(msg.contains("model=Claude 4"));
        assert!(msg.contains("session=abc"));
    }

    #[test]
    fn formats_tool_call_started() {
        let line = r#"{"type":"tool_call","subtype":"started","tool_call":{"readToolCall":{"args":{"path":"README.md"}}}}"#;
        let msg = format_stream_event(line).expect("msg");
        assert!(msg.contains("tool ▶"));
        assert!(msg.contains("read"));
        assert!(msg.contains("README.md"));
    }

    #[test]
    fn skips_assistant_duplicate_with_model_call_id() {
        std::env::set_var("AGENT_RUNTIME_AGENT_STREAM_PARTIAL", "1");
        let line = r#"{"type":"assistant","model_call_id":"x","message":{"role":"assistant","content":[{"type":"text","text":"dup"}]},"timestamp_ms":1}"#;
        assert!(format_stream_event(line).is_none());
        std::env::remove_var("AGENT_RUNTIME_AGENT_STREAM_PARTIAL");
    }

    #[test]
    fn formats_result_event() {
        let line = r#"{"type":"result","subtype":"success","duration_ms":1234,"is_error":false,"result":"ok"}"#;
        let msg = format_stream_event(line).expect("msg");
        assert!(msg.contains("duration_ms=1234"));
        assert!(msg.contains("is_error=false"));
    }

    #[test]
    fn collect_assistant_prose_ignores_result_blob() {
        let stdout = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"你好，欢迎。"}]}}
{"type":"tool_call","subtype":"started","tool_call":{"readToolCall":{"args":{"path":"README.md"}}}}
{"type":"result","subtype":"success","duration_ms":10,"is_error":false,"result":"你好，欢迎。\n\n[tool output and 5000 chars of junk]"}
"#;
        assert_eq!(collect_assistant_prose(stdout), "你好，欢迎。");
    }

    #[test]
    fn merge_assistant_log_lines_keeps_latest_snapshot() {
        let lines = vec![
            "你好".into(),
            "你好，欢迎。".into(),
            "你好，欢迎。\n\n这里是澄清阶段。".into(),
        ];
        assert_eq!(
            merge_assistant_log_lines(&lines),
            "你好，欢迎。\n\n这里是澄清阶段。"
        );
    }

    #[test]
    fn collect_result_prose_reads_last_success_result() {
        let stdout = r#"{"type":"result","subtype":"success","duration_ms":10,"is_error":false,"result":"partial"}
{"type":"result","subtype":"success","duration_ms":20,"is_error":false,"result":"完整回复"}
"#;
        assert_eq!(collect_result_prose(stdout), "完整回复");
    }

    #[test]
    fn strip_assistant_log_marker_parses_prefix() {
        assert_eq!(strip_assistant_log_marker("› 你好"), Some("你好"));
        assert_eq!(strip_assistant_log_marker("agent: start"), None);
    }
}
