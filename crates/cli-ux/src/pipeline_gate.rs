//! Pre-flight gates for pipeline routing (`issue-author` guide § slice-delivery 硬门禁).

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
