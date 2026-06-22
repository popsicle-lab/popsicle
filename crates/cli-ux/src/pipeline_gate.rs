//! Pre-flight gates for pipeline routing (`issue-author` guide § slice-delivery / bugfix 硬门禁).

use std::path::Path;

use storage::{IssueTaskLink, WorkspaceError};

/// Reject `slice-delivery` + `--proposed-task` at issue create time.
pub fn validate_slice_delivery_create(
    pipeline: Option<&str>,
    proposed_tasks: &[(String, Option<String>)],
) -> Result<(), WorkspaceError> {
    if pipeline != Some("slice-delivery") {
        return Ok(());
    }
    let has_proposed = proposed_tasks
        .iter()
        .any(|(title, _)| !title.trim().is_empty());
    if has_proposed {
        return Err(WorkspaceError::InvalidState(
            "slice-delivery-gate:create-proposed:--pipeline slice-delivery 不可与 --proposed-task 同用；新能力用 slice-spec（见 intent-coder/skills/issue-author/guide.md）".into(),
        ));
    }
    Ok(())
}

/// Reject obvious `bugfix` pipeline misuse at issue create time (PROJ-53).
pub fn validate_bugfix_create(
    issue_type: &str,
    pipeline: Option<&str>,
    title: &str,
    description: &str,
) -> Result<(), WorkspaceError> {
    if pipeline != Some("bugfix") {
        return Ok(());
    }

    let haystack = format!("{title}\n{description}").to_lowercase();

    // Allow issues that implement or document this gate itself.
    if haystack.contains("bugfix-gate")
        || haystack.contains("pipeline_gate")
        || haystack.contains("pipeline gate")
    {
        return Ok(());
    }

    if issue_type == "product" {
        return Err(WorkspaceError::InvalidState(
            "bugfix-gate:product-type:--type product 不可与 --pipeline bugfix 同用；新产品用 greenfield-product-spec，已交付能力补 spec 见 issue-author/guide.md § retro".into(),
        ));
    }

    if touches_intent_spec_content(&haystack) {
        return Err(WorkspaceError::InvalidState(
            "bugfix-gate:intent-content:description 触达 products/*/intents 或 realized_by；请用 slice-spec、retro spec（issue-author § retro），或 spec 已定后 slice-delivery + --tasks".into(),
        ));
    }

    if touches_intent_coder_skill_chain(&haystack) {
        return Err(WorkspaceError::InvalidState(
            "bugfix-gate:skill-chain:description 触达 intent-coder 技能链；请用 --type technical --pipeline tech-decision 或 slice-spec，见 issue-author/guide.md".into(),
        ));
    }

    if touches_major_ui_capability(&haystack) {
        return Err(WorkspaceError::InvalidState(
            "bugfix-gate:ui-capability:新 UI/可视化能力不宜 bugfix；spec 已覆盖用 slice-delivery + --tasks，未覆盖用 slice-spec 或 retro spec".into(),
        ));
    }

    Ok(())
}

fn touches_intent_spec_content(haystack: &str) -> bool {
    if haystack.contains("realized_by") {
        return true;
    }
    if haystack.contains("acceptance.intent")
        || haystack.contains("contracts.intent")
        || haystack.contains("invariants.intent")
    {
        return true;
    }
    if haystack.contains("/intents/") || haystack.contains("intents/*.intent") {
        return true;
    }
    if haystack.contains("products/") && haystack.contains(".intent") {
        return true;
    }
    false
}

fn touches_intent_coder_skill_chain(haystack: &str) -> bool {
    if haystack.contains("intent-coder/skills") || haystack.contains("intent-coder/skills/") {
        return true;
    }
    if haystack.contains("intent-coder") {
        let skill_marker = [
            "intent-spec-writer",
            "rfc-writer",
            "adr-writer",
            "prd-writer",
            "issue-author",
            "skill.yaml",
            "guide.md",
        ];
        if skill_marker.iter().any(|m| haystack.contains(m)) {
            return true;
        }
    }
    false
}

fn touches_major_ui_capability(haystack: &str) -> bool {
    if haystack.contains("intent-lang-visualizer") {
        return true;
    }
    (haystack.contains("visualizer") || haystack.contains("多图") || haystack.contains("关系图"))
        && (haystack.contains("接入") || haystack.contains("完善") || haystack.contains("新"))
        && !haystack.contains("修复")
        && !haystack.contains("fix")
}

/// Hard gate before starting a `slice-delivery` run (`issue start`).
pub fn validate_slice_delivery_start(
    workspace_root: &Path,
    product_id: &str,
    description: &str,
    task_links: &[IssueTaskLink],
) -> Result<(), WorkspaceError> {
    let proposed: Vec<_> = task_links.iter().filter(|l| l.role == "proposed").collect();
    if !proposed.is_empty() {
        return Err(WorkspaceError::InvalidState(format!(
            "slice-delivery-gate:proposed-task:{}:Issue 含 proposed task，须改用 --pipeline slice-spec 或先晋升 task；见 issue-author/guide.md",
            proposed.len()
        )));
    }

    let linked_ids: Vec<String> = task_links
        .iter()
        .filter(|l| l.role == "linked")
        .filter_map(|l| l.task_id.clone())
        .collect();

    if linked_ids.is_empty() {
        return Err(WorkspaceError::InvalidState(
            "slice-delivery-gate:no-tasks:须先执行 issue-author，create 时传 --tasks <id>；description 须引用每个 task_id".into(),
        ));
    }

    for task_id in &linked_ids {
        if !description.contains(task_id) {
            return Err(WorkspaceError::InvalidState(format!(
                "slice-delivery-gate:description-missing-task:{task_id}:--description 须写明本 run 实现的 task_id；新能力用 --proposed-task + slice-spec"
            )));
        }

        let task = crate::workspace_readers::read_task(workspace_root, task_id, Some(product_id))?;

        let related: Vec<String> = task
            .frontmatter
            .get("related_intents")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        if related.is_empty() {
            return Err(WorkspaceError::InvalidState(format!(
                "slice-delivery-gate:no-related-intents:{task_id}:task frontmatter 缺少 related_intents"
            )));
        }

        let intent_ok = related.iter().any(|reference| {
            crate::workspace_readers::resolve_intent_ref(
                workspace_root,
                reference,
                Some(product_id),
            )
            .is_ok()
        });
        if !intent_ok {
            return Err(WorkspaceError::InvalidState(format!(
                "slice-delivery-gate:intent-missing:{task_id}:related_intents 须在 products/{product_id}/intents/ 可解析"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn gate_fixture_root() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static SEQ: AtomicU32 = AtomicU32::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("popsicle-gate-test-{nanos}-{seq}"));
        let task_dir = root.join("products/cli-ux/tasks/daily-ops");
        fs::create_dir_all(&task_dir).expect("task dir");
        fs::create_dir_all(root.join("products/cli-ux/intents")).expect("intents dir");
        fs::write(
            task_dir.join("T-CU-0099-gate-test.md"),
            r#"---
task_id: T-CU-0099
title: "gate test task"
journey_stage: daily-ops
related_intents:
  - acceptance.intent#GateTestIntent
---
# gate test
"#,
        )
        .expect("task file");
        fs::write(
            root.join("products/cli-ux/intents/acceptance.intent"),
            r#"type GateTestResult { ok: Bool }
intent GateTestIntent(r: GateTestResult) {
  require true
  ensure r.ok' == true
}
"#,
        )
        .expect("intent file");
        root
    }

    #[test]
    fn create_rejects_slice_delivery_with_proposed() {
        let err = validate_slice_delivery_create(
            Some("slice-delivery"),
            &[("new capability".into(), Some("daily-ops".into()))],
        )
        .expect_err("expected reject");
        assert!(err.to_string().contains("create-proposed"));
    }

    #[test]
    fn bugfix_rejects_product_type() {
        let err = validate_bugfix_create(
            "product",
            Some("bugfix"),
            "补 realized_by",
            "改 products/foo/intents",
        )
        .expect_err("expected reject");
        assert!(err.to_string().contains("bugfix-gate:product-type"));
    }

    #[test]
    fn bugfix_rejects_intent_content() {
        let err = validate_bugfix_create(
            "bug",
            Some("bugfix"),
            "补链",
            "为 contracts.intent 补 realized_by",
        )
        .expect_err("expected reject");
        assert!(err.to_string().contains("bugfix-gate:intent-content"));
    }

    #[test]
    fn bugfix_rejects_skill_chain() {
        let err = validate_bugfix_create(
            "technical",
            Some("bugfix"),
            "技能",
            "更新 intent-coder/skills/intent-spec-writer/guide.md",
        )
        .expect_err("expected reject");
        assert!(err.to_string().contains("bugfix-gate:skill-chain"));
    }

    #[test]
    fn bugfix_rejects_visualizer_capability() {
        let err = validate_bugfix_create(
            "bug",
            Some("bugfix"),
            "接入 visualizer",
            "接入 intent-lang-visualizer 完善 UI",
        )
        .expect_err("expected reject");
        assert!(err.to_string().contains("bugfix-gate:ui-capability"));
    }

    #[test]
    fn bugfix_allows_ui_regression_fix() {
        validate_bugfix_create(
            "bug",
            Some("bugfix"),
            "修复关系图缩放",
            "修复 Products 关系图 Mermaid 缩放与对比度",
        )
        .expect("true bugfix should pass");
    }

    #[test]
    fn bugfix_allows_gate_meta_issue() {
        validate_bugfix_create(
            "technical",
            Some("bugfix"),
            "bugfix-gate",
            "实现 pipeline_gate bugfix-gate；触达 products/intents 关键词仅用于门禁本身",
        )
        .expect("meta gate issue should pass");
    }

    #[test]
    fn start_rejects_missing_task_in_description() {
        let root = gate_fixture_root();
        let links = vec![IssueTaskLink {
            issue_key: "PROJ-X".into(),
            role: "linked".into(),
            task_id: Some("T-CU-0099".into()),
            proposed_title: None,
            journey_stage: None,
            source: "test".into(),
            sort_order: 0,
        }];
        let err = validate_slice_delivery_start(&root, "cli-ux", "no task id here", &links)
            .expect_err("expected reject");
        assert!(err.to_string().contains("description-missing-task"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn start_passes_when_description_cites_linked_task() {
        let root = gate_fixture_root();
        let links = vec![IssueTaskLink {
            issue_key: "PROJ-X".into(),
            role: "linked".into(),
            task_id: Some("T-CU-0099".into()),
            proposed_title: None,
            journey_stage: None,
            source: "test".into(),
            sort_order: 0,
        }];
        validate_slice_delivery_start(&root, "cli-ux", "实现 T-CU-0099 门禁", &links)
            .expect("gate should pass");
        let _ = fs::remove_dir_all(&root);
    }
}
