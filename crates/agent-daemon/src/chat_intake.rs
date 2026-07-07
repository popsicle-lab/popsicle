//! Intake Chat handlers: chat turn + bootstrap (PDR-002 / PROJ-95).

use std::io;

use crate::cursor_agent::{auto_agent_enabled, CursorAgentAdapter};
use crate::orchestrator::{orchestrator_enabled, run_unattended, OrchestratorConfig};
use crate::prompt::load_skill_prompt;
use crate::server_client::{
    append_log, execute_issue_create, execute_issue_start, parse_issue_key_from_create,
    parse_run_id_from_issue_start, sync_run_mirror, ClaimedBootstrap, ClaimedChatTurn,
    ServerClient,
};
use crate::stream_json::{
    collect_assistant_prose, merge_assistant_log_lines, strip_assistant_log_marker,
};
use crate::PopsicleInvoker;

type ChatDraftReply = (String, Option<String>, Option<String>, Option<String>);

pub fn chat_dry_run_enabled() -> bool {
    matches!(
        std::env::var("AGENT_RUNTIME_CHAT_DRY_RUN").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

pub fn handle_chat_turn(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    turn: &ClaimedChatTurn,
    log: impl Fn(&str),
) -> io::Result<()> {
    let product = client
        .get_chat_session(&turn.session_id)?
        .and_then(|v| v.product_id)
        .unwrap_or_else(|| "agent-runtime".into());

    let (assistant, title, pipeline, description) = if chat_dry_run_enabled() {
        dry_run_reply(&turn.user_content, Some(product.as_str()))
    } else if auto_agent_enabled() {
        match agent_reply(invoker, client, turn, &product, &log) {
            Ok(v) => v,
            Err(e) => {
                log(&format!("chat agent fallback: {e}"));
                dry_run_reply(&turn.user_content, Some(product.as_str()))
            }
        }
    } else {
        (
            format!(
                "已收到：「{}」。请配置 cursor-agent 或设置 AGENT_RUNTIME_CHAT_DRY_RUN=1。",
                turn.user_content
            ),
            Some(truncate_title(&turn.user_content)),
            Some(default_pipeline()),
            Some(turn.user_content.clone()),
        )
    };

    let mark_ready = title.is_some() && pipeline.is_some();
    client.complete_chat_turn(crate::server_client::CompleteChatTurnParams {
        session_id: &turn.session_id,
        turn_id: &turn.id,
        assistant_content: &assistant,
        draft_title: title.as_deref(),
        draft_pipeline: pipeline.as_deref(),
        draft_description: description.as_deref(),
        mark_ready,
    })?;
    log(&format!("chat turn completed session={}", turn.session_id));
    Ok(())
}

pub fn handle_bootstrap(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    task: &ClaimedBootstrap,
    log_path: Option<&std::path::Path>,
) -> io::Result<()> {
    let log = |msg: &str| append_log(log_path, msg);
    log(&format!(
        "bootstrap: create issue title={} pipeline={}",
        task.draft_title, task.draft_pipeline
    ));
    let create = execute_issue_create(
        invoker,
        "product",
        &task.draft_title,
        &task.product_id,
        &task.draft_pipeline,
        &task.draft_description,
    )?;
    if create.exit_code != 0 {
        log(&format!(
            "bootstrap: issue create failed exit={}",
            create.exit_code
        ));
        return Err(io::Error::other(format!(
            "issue create exit {}",
            create.exit_code
        )));
    }
    let issue_key = parse_issue_key_from_create(&create.stdout).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "issue create missing key in stdout",
        )
    })?;
    log(&format!("bootstrap: created {issue_key}"));

    let start = execute_issue_start(invoker, &issue_key)?;
    if start.exit_code != 0 {
        return Err(io::Error::other(format!(
            "issue start exit {}",
            start.exit_code
        )));
    }
    let run_id = parse_run_id_from_issue_start(&start.stdout)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "issue start missing run_id"))?;
    sync_run_mirror(client, invoker, &run_id, Some(&issue_key))?;

    if orchestrator_enabled() {
        let cfg = OrchestratorConfig::from_env();
        let _ = run_unattended(
            invoker,
            Some(client),
            &run_id,
            &issue_key,
            |msg| log(msg),
            &cfg,
        );
    }

    client.complete_bootstrap(&task.session_id, &task.id, &issue_key, &run_id)?;
    log(&format!("bootstrap: run {run_id} started for {issue_key}"));
    Ok(())
}

fn dry_run_reply(user: &str, product: Option<&str>) -> ChatDraftReply {
    let title = truncate_title(user);
    let pipeline = default_pipeline();
    let desc = format!(
        "## 需求摘要\n{user}\n\n## 产品\n{}\n",
        product.unwrap_or("agent-runtime")
    );
    (
        format!(
            "我理解你的需求是：{user}\n\n建议 pipeline：`{pipeline}`\n标题：{title}\n\n请在下方确认后开始。"
        ),
        Some(title),
        Some(pipeline),
        Some(desc),
    )
}

fn agent_reply(
    invoker: &PopsicleInvoker,
    client: &ServerClient,
    turn: &ClaimedChatTurn,
    product: &str,
    log: &dyn Fn(&str),
) -> io::Result<ChatDraftReply> {
    let guide = load_skill_prompt(invoker.workspace(), "issue-author", &turn.session_id)
        .unwrap_or_else(|_| "Follow issue-author workflow for pipeline selection.".into());
    let history = client
        .get_chat_session(&turn.session_id)?
        .map(|v| {
            v.messages
                .iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let prompt = format!(
        "{guide}\n\n\
         你在 Mobile Intake Chat 中做需求澄清，尚未创建 Issue。\n\
         产品：{product}\n\
         对话历史：\n{history}\n\n\
         用户最新消息：{msg}\n\n\
         用简体中文回复。\n\
         - 信息仍不足：只追问，不要输出 draft JSON。\n\
         - 信息已足够：在回复末尾单独一行输出（不要用 markdown 代码块）：\
         {{\"draft_title\":\"...\",\"draft_pipeline\":\"...\",\"draft_description\":\"...\"}}\n\
         pipeline 必须按 issue-author 决策树选择（bug 用 fix-regression，大特性用 feature-arch-spec 等），不要默认 feature-spec。",
        msg = turn.user_content
    );
    let adapter = CursorAgentAdapter::for_workspace(invoker.workspace())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cursor-agent not found"))?;
    let session_id = turn.session_id.clone();
    let mut log_prose: Vec<String> = Vec::new();
    let result = adapter.invoke_prompt_with_log(&prompt, |line| {
        let _ = client.post_run_log(&session_id, "agent", line);
        log(&format!("chat-agent: {line}"));
        if let Some(text) = strip_assistant_log_marker(line) {
            log_prose.push(text.to_string());
        }
    })?;
    parse_agent_reply(&result.stdout, &log_prose, &turn.user_content)
}

fn parse_agent_reply(
    stdout: &str,
    log_prose: &[String],
    fallback_user: &str,
) -> io::Result<ChatDraftReply> {
    let _ = fallback_user;
    let mut draft_title = None;
    let mut draft_pipeline = None;
    let mut draft_description = None;

    for line in stdout.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
            continue;
        };
        if v.get("draft_title").is_some() {
            draft_title = v
                .get("draft_title")
                .and_then(|x| x.as_str())
                .map(str::to_string);
            draft_pipeline = v
                .get("draft_pipeline")
                .and_then(|x| x.as_str())
                .map(str::to_string);
            draft_description = v
                .get("draft_description")
                .and_then(|x| x.as_str())
                .map(str::to_string);
        }
    }

    let from_log = merge_assistant_log_lines(log_prose);
    let from_stream = collect_assistant_prose(stdout);
    let mut raw_assistant = if !from_log.is_empty() {
        from_log.clone()
    } else {
        from_stream
    };
    if looks_like_stream_json_dump(&raw_assistant) && !from_log.is_empty() {
        raw_assistant = from_log;
    }
    let (assistant, inline_draft) = extract_draft_from_assistant(&raw_assistant);
    if draft_title.is_none() {
        if let Some((title, pipeline, description)) = inline_draft {
            draft_title = Some(title);
            draft_pipeline = Some(pipeline);
            draft_description = Some(description);
        }
    }
    let assistant = assistant.trim().to_string();
    Ok((assistant, draft_title, draft_pipeline, draft_description))
}

fn parse_draft_json_value(json: &str) -> Option<(String, String, String)> {
    let v = serde_json::from_str::<serde_json::Value>(json.trim()).ok()?;
    let title = v
        .get("draft_title")
        .and_then(|x| x.as_str())
        .map(str::to_string)?;
    let pipeline = v
        .get("draft_pipeline")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .unwrap_or_else(default_pipeline);
    let description = v
        .get("draft_description")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| title.clone());
    Some((title, pipeline, description))
}

fn markdown_json_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_fence = false;
    let mut buf = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_fence {
                let block = buf.trim();
                if !block.is_empty() {
                    blocks.push(block.to_string());
                }
                buf.clear();
                in_fence = false;
            } else {
                in_fence = true;
            }
            continue;
        }
        if in_fence {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    blocks
}

/// Strip trailing draft JSON (standalone line or embedded block) from assistant prose.
fn extract_draft_from_assistant(text: &str) -> (String, Option<(String, String, String)>) {
    let (body, draft) = split_draft_json_line(text);
    if draft.is_some() {
        return (body, draft);
    }
    for block in markdown_json_blocks(text) {
        if let Some(draft) = parse_draft_json_value(&block) {
            return (text.trim().to_string(), Some(draft));
        }
    }
    let Some(start) = text.rfind("{\"draft_title\"") else {
        return (text.trim().to_string(), None);
    };
    let json_slice = text[start..].trim();
    let Some((title, pipeline, description)) = parse_draft_json_value(json_slice) else {
        return (text.trim().to_string(), None);
    };
    let body = text[..start]
        .trim_end()
        .trim_end_matches("---")
        .trim_end()
        .to_string();
    (body, Some((title, pipeline, description)))
}

/// Strip trailing `{"draft_title":...}` line from assistant prose when present.
fn split_draft_json_line(text: &str) -> (String, Option<(String, String, String)>) {
    let lines: Vec<&str> = text.lines().collect();
    for (idx, line) in lines.iter().enumerate().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            break;
        };
        let _ = v;
        let Some((title, pipeline, description)) = parse_draft_json_value(trimmed) else {
            break;
        };
        let body = lines[..idx]
            .join("\n")
            .trim_end()
            .trim_end_matches("---")
            .trim_end()
            .to_string();
        return (body, Some((title, pipeline, description)));
    }
    (text.trim().to_string(), None)
}

fn looks_like_stream_json_dump(text: &str) -> bool {
    text.trim_start().starts_with("{\"type\"")
}

fn truncate_title(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() <= 80 {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(77).collect::<String>())
    }
}

fn default_pipeline() -> String {
    std::env::var("AGENT_RUNTIME_CHAT_DEFAULT_PIPELINE").unwrap_or_else(|_| "feature-spec".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_produces_draft() {
        let (reply, title, pipe, _) = dry_run_reply("给 mobile 加 chat", Some("agent-runtime"));
        assert!(reply.contains("chat"));
        assert!(title.is_some());
        assert_eq!(pipe.as_deref(), Some("feature-spec"));
    }

    #[test]
    fn parse_agent_reply_extracts_assistant_from_stream_json() {
        let stdout = r#"{"type":"system","subtype":"init","model":"auto","session_id":"s1"}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"你好，欢迎。"}]}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"你好，欢迎。\n\n这里是需求澄清阶段。"}]}}
{"type":"result","subtype":"success","duration_ms":1200,"is_error":false,"result":"你好，欢迎。\n\n这里是需求澄清阶段。"}
"#;
        let (assistant, title, pipeline, _) =
            parse_agent_reply(stdout, &[], "fallback").expect("parse");
        assert!(assistant.contains("你好，欢迎"));
        assert!(!assistant.contains(r#""type":"system""#));
        assert_eq!(title.as_deref(), None);
        assert_eq!(pipeline.as_deref(), None);
    }

    #[test]
    fn parse_agent_reply_prefers_log_prose_over_stdout() {
        let stdout = r#"{"type":"system","subtype":"init"}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ignored"}]}}
"#;
        let log_prose = vec!["短".into(), "你好，我是 Issue 立项助手。".into()];
        let (assistant, _, _, _) =
            parse_agent_reply(stdout, &log_prose, "fallback").expect("parse");
        assert_eq!(assistant, "你好，我是 Issue 立项助手。");
    }

    #[test]
    fn parse_agent_reply_strips_trailing_draft_json() {
        let stdout = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"草案如下。\n\n{\"draft_title\":\"Mobile Chat\",\"draft_pipeline\":\"feature-spec\",\"draft_description\":\"desc\"}"}]}}
"#;
        let (assistant, title, pipeline, desc) =
            parse_agent_reply(stdout, &[], "fallback").expect("parse");
        assert_eq!(assistant, "草案如下。");
        assert_eq!(title.as_deref(), Some("Mobile Chat"));
        assert_eq!(pipeline.as_deref(), Some("feature-spec"));
        assert_eq!(desc.as_deref(), Some("desc"));
    }

    #[test]
    fn split_draft_json_line_handles_trailing_separator() {
        let (body, draft) = extract_draft_from_assistant(
            "正文\n\n---\n{\"draft_title\":\"T\",\"draft_pipeline\":\"feature-spec\",\"draft_description\":\"D\"}",
        );
        assert_eq!(body, "正文");
        assert_eq!(draft, Some(("T".into(), "feature-spec".into(), "D".into())));
    }

    #[test]
    fn clarifying_turn_without_draft_json_does_not_invent_pipeline() {
        let stdout = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"请补充一下复现步骤。"}]}}"#;
        let (assistant, title, pipeline, desc) =
            parse_agent_reply(stdout, &[], "用户消息").expect("parse");
        assert!(assistant.contains("复现"));
        assert!(title.is_none());
        assert!(pipeline.is_none());
        assert!(desc.is_none());
    }

    #[test]
    fn parse_agent_reply_extracts_draft_from_markdown_fence() {
        let stdout = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"草案如下。\n\n```json\n{\"draft_title\":\"修复 Chat\",\"draft_pipeline\":\"fix-regression\",\"draft_description\":\"desc\"}\n```"}]}}"#;
        let (assistant, title, pipeline, _) =
            parse_agent_reply(stdout, &[], "fallback").expect("parse");
        assert!(assistant.contains("草案如下"));
        assert_eq!(title.as_deref(), Some("修复 Chat"));
        assert_eq!(pipeline.as_deref(), Some("fix-regression"));
    }

    #[test]
    fn extract_draft_from_embedded_json_block() {
        let (body, draft) = extract_draft_from_assistant(
            "草案如下。\n\n{\"draft_title\":\"Mobile Chat\",\"draft_pipeline\":\"feature-spec\",\"draft_description\":\"desc\"}",
        );
        assert_eq!(body, "草案如下。");
        assert_eq!(draft.map(|d| d.0), Some("Mobile Chat".into()));
    }
}
