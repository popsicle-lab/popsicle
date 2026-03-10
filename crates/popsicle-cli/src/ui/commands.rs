use std::path::PathBuf;

use popsicle_core::dto::*;
use popsicle_core::engine::{Advisor, count_checkboxes};
use popsicle_core::git::GitTracker;
use popsicle_core::helpers;
use popsicle_core::model::{Issue, IssueStatus, IssueType, PipelineRun, Priority, StageState};
use popsicle_core::storage::{FileStorage, IndexDb, ProjectConfig, ProjectLayout, DocumentRow};
use tauri::State;

use super::AppState;

fn get_dir(state: &State<AppState>) -> Result<PathBuf, String> {
    let guard = state.project_dir.lock().map_err(|e| e.to_string())?;
    guard
        .as_ref()
        .map(PathBuf::from)
        .ok_or_else(|| "No project directory set".to_string())
}

fn doc_row_to_info(d: &DocumentRow) -> DocInfo {
    let (checked, unchecked) = std::fs::read_to_string(&d.file_path)
        .ok()
        .map(|body| count_checkboxes(&body))
        .unwrap_or((0, 0));
    DocInfo {
        id: d.id.clone(),
        doc_type: d.doc_type.clone(),
        title: d.title.clone(),
        status: d.status.clone(),
        skill_name: d.skill_name.clone(),
        created_at: d.created_at.clone(),
        updated_at: d.updated_at.clone(),
        checklist_total: (checked + unchecked) as u32,
        checklist_checked: checked as u32,
    }
}

#[tauri::command]
pub fn set_project_dir(path: String, state: State<AppState>) -> Result<ProjectInfo, String> {
    let project_dir = PathBuf::from(&path);
    let layout = ProjectLayout::new(&project_dir);

    if !layout.is_initialized() {
        return Err(format!("Not a Popsicle project: {}", path));
    }

    *state.project_dir.lock().map_err(|e| e.to_string())? = Some(path.clone());

    Ok(ProjectInfo {
        path,
        initialized: true,
    })
}

#[tauri::command]
pub fn get_project_status(state: State<AppState>) -> Result<ProjectInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    Ok(ProjectInfo {
        path: dir.display().to_string(),
        initialized: layout.is_initialized(),
    })
}

#[tauri::command]
pub fn list_skills(state: State<AppState>) -> Result<Vec<SkillInfo>, String> {
    let dir = get_dir(&state)?;
    let registry = helpers::load_registry(&dir).map_err(|e| e.to_string())?;
    Ok(registry
        .list()
        .iter()
        .map(|s| SkillInfo {
            name: s.name.clone(),
            description: s.description.clone(),
            version: s.version.clone(),
            artifact_types: s
                .artifacts
                .iter()
                .map(|a| a.artifact_type.clone())
                .collect(),
            workflow_initial: s.workflow.initial.clone(),
            inputs: s
                .inputs
                .iter()
                .map(|i| SkillInputInfo {
                    from_skill: i.from_skill.clone(),
                    artifact_type: i.artifact_type.clone(),
                    required: i.required,
                })
                .collect(),
            workflow_states: s
                .workflow
                .states
                .iter()
                .map(|(name, sd)| WorkflowStateInfo {
                    name: name.clone(),
                    is_final: sd.r#final,
                    transitions: sd
                        .transitions
                        .iter()
                        .map(|t| TransitionInfo {
                            to: t.to.clone(),
                            action: t.action.clone(),
                            requires_approval: t.requires_approval,
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub fn list_pipelines(state: State<AppState>) -> Result<Vec<PipelineInfo>, String> {
    let dir = get_dir(&state)?;
    let pipelines = helpers::load_pipelines(&dir).map_err(|e| e.to_string())?;
    Ok(pipelines
        .iter()
        .map(|p| PipelineInfo {
            name: p.name.clone(),
            description: p.description.clone(),
            stages: p
                .stages
                .iter()
                .map(|s| StageInfo {
                    name: s.name.clone(),
                    skills: s.skill_names().iter().map(|n| n.to_string()).collect(),
                    description: s.description.clone(),
                    depends_on: s.depends_on.clone(),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub fn list_pipeline_runs(state: State<AppState>) -> Result<Vec<PipelineRunInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let runs = db.list_pipeline_runs().map_err(|e| e.to_string())?;
    Ok(runs
        .iter()
        .map(|r| PipelineRunInfo {
            id: r.id.clone(),
            pipeline_name: r.pipeline_name.clone(),
            title: r.title.clone(),
            created_at: r.created_at.clone(),
            updated_at: r.updated_at.clone(),
        })
        .collect())
}

#[tauri::command]
pub fn get_pipeline_status(
    run_id: String,
    state: State<AppState>,
) -> Result<PipelineStatusFull, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def =
        helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;
    let docs = db
        .query_documents(None, None, Some(&run_id))
        .map_err(|e| e.to_string())?;

    let stages: Vec<StageStatusInfo> = pipeline_def
        .stages
        .iter()
        .map(|stage| {
            let state_val = run
                .stage_states
                .get(&stage.name)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "blocked".to_string());
            let stage_docs: Vec<DocInfo> = docs
                .iter()
                .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
                .map(doc_row_to_info)
                .collect();
            StageStatusInfo {
                name: stage.name.clone(),
                state: state_val,
                skills: stage.skill_names().iter().map(|n| n.to_string()).collect(),
                description: stage.description.clone(),
                depends_on: stage.depends_on.clone(),
                documents: stage_docs,
            }
        })
        .collect();

    Ok(PipelineStatusFull {
        id: run.id,
        pipeline_name: run.pipeline_name,
        title: run.title,
        stages,
    })
}

#[tauri::command]
pub fn list_documents(
    skill: Option<String>,
    status: Option<String>,
    run_id: Option<String>,
    state: State<AppState>,
) -> Result<Vec<DocInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let docs = db
        .query_documents(skill.as_deref(), status.as_deref(), run_id.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(docs.iter().map(doc_row_to_info).collect())
}

#[tauri::command]
pub fn get_document(doc_id: String, state: State<AppState>) -> Result<DocFull, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let docs = db
        .query_documents(None, None, None)
        .map_err(|e| e.to_string())?;
    let doc_row = docs
        .iter()
        .find(|d| d.id == doc_id)
        .ok_or_else(|| format!("Document not found: {}", doc_id))?;

    let doc = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
        .map_err(|e| e.to_string())?;

    Ok(DocFull {
        id: doc.id,
        doc_type: doc.doc_type,
        title: doc.title,
        status: doc.status,
        skill_name: doc.skill_name,
        pipeline_run_id: doc.pipeline_run_id,
        tags: doc.tags,
        body: doc.body,
        file_path: doc_row.file_path.clone(),
        created_at: doc_row.created_at.clone(),
        updated_at: doc_row.updated_at.clone(),
    })
}

#[tauri::command]
pub fn get_next_steps(run_id: String, state: State<AppState>) -> Result<Vec<NextStepInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let registry = helpers::load_registry(&dir).map_err(|e| e.to_string())?;

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def =
        helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;
    let docs = db
        .query_documents(None, None, Some(&run_id))
        .map_err(|e| e.to_string())?;

    let steps = Advisor::next_steps(&pipeline_def, &run, &registry, &docs);
    Ok(steps
        .iter()
        .map(|s| NextStepInfo {
            stage: s.stage.clone(),
            skill: s.skill.clone(),
            action: s.action.clone(),
            description: s.description.clone(),
            cli_command: s.cli_command.clone(),
            prompt: s.prompt.clone(),
            blocked_by: s.blocked_by.clone(),
            requires_approval: s.requires_approval,
        })
        .collect())
}

#[tauri::command]
pub fn get_prompt(
    skill_name: String,
    workflow_state: Option<String>,
    state: State<AppState>,
) -> Result<PromptInfo, String> {
    let dir = get_dir(&state)?;
    let registry = helpers::load_registry(&dir).map_err(|e| e.to_string())?;
    let skill = registry.get(&skill_name).map_err(|e| e.to_string())?;
    let ws = workflow_state.as_deref().unwrap_or(&skill.workflow.initial);
    let prompt = skill.prompts.get(ws).cloned();

    Ok(PromptInfo {
        skill: skill_name,
        state: ws.to_string(),
        prompt,
        available_states: skill.prompts.keys().cloned().collect(),
    })
}

#[tauri::command]
pub fn verify_pipeline_run(run_id: String, state: State<AppState>) -> Result<VerifyResult, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let registry = helpers::load_registry(&dir).map_err(|e| e.to_string())?;

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def =
        helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;
    let docs = db
        .query_documents(None, None, Some(&run_id))
        .map_err(|e| e.to_string())?;

    let mut issues = Vec::new();

    for stage in &pipeline_def.stages {
        let stage_state = run.stage_states.get(&stage.name);
        if !matches!(
            stage_state,
            Some(popsicle_core::model::StageState::Completed)
                | Some(popsicle_core::model::StageState::Skipped)
        ) {
            issues.push(format!(
                "Stage '{}' is {}",
                stage.name,
                stage_state
                    .map(|s| s.to_string())
                    .unwrap_or("unknown".into())
            ));
        }

        for skill_name in stage.skill_names() {
            let skill_docs: Vec<_> = docs.iter().filter(|d| d.skill_name == skill_name).collect();
            if skill_docs.is_empty() {
                issues.push(format!("No documents for skill '{}'", skill_name));
            }
            for d in &skill_docs {
                if let Ok(skill) = registry.get(&d.skill_name)
                    && !skill.is_final_state(&d.status)
                {
                    issues.push(format!("'{}' is '{}', not final", d.title, d.status));
                }
            }
        }
    }

    Ok(VerifyResult {
        run_id,
        verified: issues.is_empty(),
        issues,
    })
}

#[tauri::command]
pub fn get_project_config(state: State<AppState>) -> Result<serde_json::Value, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let config = popsicle_core::storage::ProjectConfig::load(&layout.config_path())
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_git_status(state: State<AppState>) -> Result<GitStatusInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let branch = GitTracker::current_branch(&dir).unwrap_or_else(|_| "unknown".into());
    let has_changes = GitTracker::has_uncommitted_changes(&dir).unwrap_or(false);
    let head = GitTracker::head_sha(&dir)
        .map(|s| s[..8.min(s.len())].to_string())
        .unwrap_or_else(|_| "unknown".into());

    let run_id = db
        .list_pipeline_runs()
        .ok()
        .and_then(|runs| runs.first().map(|r| r.id.clone()));

    let (total, pending, passed, failed) = if let Some(ref rid) = run_id {
        let links = db
            .query_commit_links(Some(rid), None, None)
            .unwrap_or_default();
        let t = links.len();
        let p = links
            .iter()
            .filter(|l| l.review_status == popsicle_core::git::ReviewStatus::Pending)
            .count();
        let pa = links
            .iter()
            .filter(|l| l.review_status == popsicle_core::git::ReviewStatus::Passed)
            .count();
        let f = links
            .iter()
            .filter(|l| l.review_status == popsicle_core::git::ReviewStatus::Failed)
            .count();
        (t, p, pa, f)
    } else {
        (0, 0, 0, 0)
    };

    Ok(GitStatusInfo {
        branch,
        head,
        uncommitted_changes: has_changes,
        pipeline_run_id: run_id,
        total_commits: total,
        pending_review: pending,
        passed,
        failed,
    })
}

#[tauri::command]
pub fn get_commit_links(
    run_id: Option<String>,
    doc_id: Option<String>,
    state: State<AppState>,
) -> Result<Vec<CommitLinkInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let rid = match run_id {
        Some(r) => Some(r),
        None => db
            .list_pipeline_runs()
            .ok()
            .and_then(|runs| runs.first().map(|r| r.id.clone())),
    };

    let links = db
        .query_commit_links(rid.as_deref(), doc_id.as_deref(), None)
        .map_err(|e| e.to_string())?;

    let result: Vec<CommitLinkInfo> = links
        .iter()
        .map(|l| {
            let commit = GitTracker::commit_info(&dir, &l.sha).ok();
            CommitLinkInfo {
                sha: l.sha.clone(),
                short_sha: commit
                    .as_ref()
                    .map(|c| c.short_sha.clone())
                    .unwrap_or_else(|| l.sha[..8.min(l.sha.len())].to_string()),
                message: commit
                    .as_ref()
                    .map(|c| c.message.clone())
                    .unwrap_or_default(),
                author: commit
                    .as_ref()
                    .map(|c| c.author.clone())
                    .unwrap_or_default(),
                timestamp: commit
                    .as_ref()
                    .map(|c| c.timestamp.clone())
                    .unwrap_or_default(),
                doc_id: l.doc_id.clone(),
                pipeline_run_id: l.pipeline_run_id.clone(),
                stage: l.stage.clone(),
                skill: l.skill.clone(),
                review_status: l.review_status.to_string(),
                review_summary: l.review_summary.clone(),
                linked_at: l.linked_at.clone(),
            }
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub fn list_discussions(
    run_id: Option<String>,
    skill: Option<String>,
    status: Option<String>,
    state: State<AppState>,
) -> Result<Vec<DiscussionInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let discussions = db
        .query_discussions(run_id.as_deref(), skill.as_deref(), status.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(discussions
        .iter()
        .map(|d| {
            let msg_count = db
                .get_discussion_messages(&d.id)
                .map(|m| m.len())
                .unwrap_or(0);
            DiscussionInfo {
                id: d.id.clone(),
                document_id: d.document_id.clone(),
                skill: d.skill.clone(),
                pipeline_run_id: d.pipeline_run_id.clone(),
                topic: d.topic.clone(),
                status: d.status.to_string(),
                user_confidence: d.user_confidence,
                message_count: msg_count,
                created_at: d.created_at.to_rfc3339(),
                concluded_at: d.concluded_at.map(|t| t.to_rfc3339()),
            }
        })
        .collect())
}

#[tauri::command]
pub fn get_discussion(
    discussion_id: String,
    state: State<AppState>,
) -> Result<DiscussionFull, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let disc = db
        .get_discussion(&discussion_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Discussion not found: {}", discussion_id))?;
    let roles = db
        .get_discussion_roles(&discussion_id)
        .map_err(|e| e.to_string())?;
    let messages = db
        .get_discussion_messages(&discussion_id)
        .map_err(|e| e.to_string())?;

    Ok(DiscussionFull {
        id: disc.id,
        document_id: disc.document_id,
        skill: disc.skill,
        pipeline_run_id: disc.pipeline_run_id,
        topic: disc.topic,
        status: disc.status.to_string(),
        user_confidence: disc.user_confidence,
        roles: roles
            .iter()
            .map(|r| DiscussionRoleInfo {
                role_id: r.role_id.clone(),
                role_name: r.role_name.clone(),
                perspective: r.perspective.clone(),
                source: r.source.to_string(),
            })
            .collect(),
        messages: messages
            .iter()
            .map(|m| DiscussionMessageInfo {
                id: m.id.clone(),
                phase: m.phase.clone(),
                role_id: m.role_id.clone(),
                role_name: m.role_name.clone(),
                content: m.content.clone(),
                message_type: m.message_type.to_string(),
                reply_to: m.reply_to.clone(),
                timestamp: m.timestamp.to_rfc3339(),
            })
            .collect(),
        created_at: disc.created_at.to_rfc3339(),
        concluded_at: disc.concluded_at.map(|t| t.to_rfc3339()),
    })
}

// ── Issue commands ──

fn issue_to_info(i: &Issue) -> IssueInfo {
    IssueInfo {
        id: i.id.clone(),
        key: i.key.clone(),
        title: i.title.clone(),
        issue_type: i.issue_type.to_string(),
        priority: i.priority.to_string(),
        status: i.status.to_string(),
        pipeline_run_id: i.pipeline_run_id.clone(),
        labels: i.labels.clone(),
        created_at: i.created_at.to_rfc3339(),
        updated_at: i.updated_at.to_rfc3339(),
    }
}

#[tauri::command]
pub fn list_issues(
    issue_type: Option<String>,
    status: Option<String>,
    label: Option<String>,
    state: State<AppState>,
) -> Result<Vec<IssueInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let issues = db
        .query_issues(issue_type.as_deref(), status.as_deref(), label.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(issues.iter().map(issue_to_info).collect())
}

#[tauri::command]
pub fn get_issue(key: String, state: State<AppState>) -> Result<IssueFull, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let issue = db
        .get_issue(&key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Issue not found: {}", key))?;

    Ok(IssueFull {
        id: issue.id,
        key: issue.key,
        title: issue.title,
        description: issue.description,
        issue_type: issue.issue_type.to_string(),
        priority: issue.priority.to_string(),
        status: issue.status.to_string(),
        pipeline_run_id: issue.pipeline_run_id,
        labels: issue.labels,
        created_at: issue.created_at.to_rfc3339(),
        updated_at: issue.updated_at.to_rfc3339(),
    })
}

#[tauri::command]
pub fn create_issue(
    issue_type: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    labels: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<IssueInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let config = ProjectConfig::load(&layout.config_path()).map_err(|e| e.to_string())?;
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let it: IssueType = issue_type.parse().map_err(|e: String| e)?;
    let pr: Priority = priority
        .as_deref()
        .unwrap_or("medium")
        .parse()
        .map_err(|e: String| e)?;

    let prefix = config.project.key_prefix_or_default();
    let seq = db.next_issue_seq(prefix).map_err(|e| e.to_string())?;
    let key = format!("{}-{}", prefix, seq);

    let mut issue = Issue::new(key, &title, it);
    issue.description = description.unwrap_or_default();
    issue.priority = pr;
    issue.labels = labels.unwrap_or_default();

    db.create_issue(&issue).map_err(|e| e.to_string())?;

    Ok(issue_to_info(&issue))
}

#[tauri::command]
pub fn start_issue(key: String, state: State<AppState>) -> Result<IssueInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let mut issue = db
        .get_issue(&key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Issue not found: {}", key))?;

    if issue.pipeline_run_id.is_some() {
        return Err(format!("Issue {} already has a pipeline run", key));
    }

    let pipeline_name = issue
        .issue_type
        .default_pipeline()
        .ok_or_else(|| format!("Issue type '{}' has no default pipeline", issue.issue_type))?;

    let pipeline_def = helpers::find_pipeline(&dir, pipeline_name).map_err(|e| e.to_string())?;
    pipeline_def.validate().map_err(|e| e.to_string())?;

    let run = PipelineRun::new(&pipeline_def, &issue.title);
    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir).map_err(|e| e.to_string())?;

    db.upsert_pipeline_run(&run).map_err(|e| e.to_string())?;

    issue.pipeline_run_id = Some(run.id);
    issue.status = IssueStatus::InProgress;
    db.update_issue(&issue).map_err(|e| e.to_string())?;

    Ok(issue_to_info(&issue))
}

#[tauri::command]
pub fn update_issue(
    key: String,
    status: Option<String>,
    priority: Option<String>,
    title: Option<String>,
    labels: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<IssueInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let mut issue = db
        .get_issue(&key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Issue not found: {}", key))?;

    if let Some(s) = status {
        issue.status = s.parse().map_err(|e: String| e)?;
    }
    if let Some(p) = priority {
        issue.priority = p.parse().map_err(|e: String| e)?;
    }
    if let Some(t) = title {
        issue.title = t;
    }
    if let Some(ls) = labels {
        for l in ls {
            if !issue.labels.contains(&l) {
                issue.labels.push(l);
            }
        }
    }

    db.update_issue(&issue).map_err(|e| e.to_string())?;

    Ok(issue_to_info(&issue))
}

// ── Issue progress aggregation ──

#[tauri::command]
pub fn get_issue_progress(key: String, state: State<AppState>) -> Result<IssueProgress, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let registry = helpers::load_registry(&dir).map_err(|e| e.to_string())?;

    let issue = db
        .get_issue(&key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Issue not found: {}", key))?;

    let run_id = match &issue.pipeline_run_id {
        Some(id) => id.clone(),
        None => {
            return Ok(IssueProgress {
                issue_key: issue.key,
                pipeline_run_id: None,
                pipeline_name: None,
                stages_total: 0,
                stages_completed: 0,
                docs_total: 0,
                docs_final: 0,
                checklist_checked: 0,
                checklist_total: 0,
                current_stage: None,
                stage_summaries: vec![],
            });
        }
    };

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def =
        helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;

    let docs = db
        .query_documents(None, None, Some(&run_id))
        .map_err(|e| e.to_string())?;

    let stages_total = pipeline_def.stages.len() as u32;
    let mut stages_completed: u32 = 0;
    let mut current_stage: Option<String> = None;
    let mut total_cl_checked: u32 = 0;
    let mut total_cl_total: u32 = 0;
    let mut docs_total: u32 = 0;
    let mut docs_final: u32 = 0;
    let mut stage_summaries = Vec::new();

    for stage in &pipeline_def.stages {
        let state_val = run
            .stage_states
            .get(&stage.name)
            .copied()
            .unwrap_or(StageState::Blocked);

        if state_val == StageState::Completed || state_val == StageState::Skipped {
            stages_completed += 1;
        }

        if current_stage.is_none()
            && (state_val == StageState::InProgress || state_val == StageState::Ready)
        {
            current_stage = Some(stage.name.clone());
        }

        let stage_docs: Vec<DocInfo> = docs
            .iter()
            .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
            .map(|d| {
                let info = doc_row_to_info(d);
                total_cl_checked += info.checklist_checked;
                total_cl_total += info.checklist_total;
                docs_total += 1;
                if registry
                    .get(&d.skill_name)
                    .map(|s| s.is_final_state(&d.status))
                    .unwrap_or(false)
                {
                    docs_final += 1;
                }
                info
            })
            .collect();

        stage_summaries.push(StageSummary {
            name: stage.name.clone(),
            state: state_val.to_string(),
            docs: stage_docs,
        });
    }

    Ok(IssueProgress {
        issue_key: issue.key,
        pipeline_run_id: Some(run_id),
        pipeline_name: Some(run.pipeline_name),
        stages_total,
        stages_completed,
        docs_total,
        docs_final,
        checklist_checked: total_cl_checked,
        checklist_total: total_cl_total,
        current_stage,
        stage_summaries,
    })
}

// ── Activity timeline ──

#[tauri::command]
pub fn get_activity(run_id: String, state: State<AppState>) -> Result<Vec<ActivityEvent>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let docs = db
        .query_documents(None, None, Some(&run_id))
        .map_err(|e| e.to_string())?;

    let commits = db
        .query_commit_links(Some(&run_id), None, None)
        .map_err(|e| e.to_string())?;

    let mut events: Vec<ActivityEvent> = Vec::new();

    for d in &docs {
        if let Some(ref ts) = d.created_at {
            events.push(ActivityEvent {
                timestamp: ts.clone(),
                event_type: "doc_created".to_string(),
                title: format!("{} created", d.title),
                detail: Some(d.doc_type.clone()),
                doc_id: Some(d.id.clone()),
                stage: None,
            });
        }
        if let Some(ref ts) = d.updated_at {
            if d.updated_at != d.created_at {
                events.push(ActivityEvent {
                    timestamp: ts.clone(),
                    event_type: "doc_updated".to_string(),
                    title: format!("{} → {}", d.title, d.status),
                    detail: None,
                    doc_id: Some(d.id.clone()),
                    stage: None,
                });
            }
        }
    }

    for c in &commits {
        let commit = GitTracker::commit_info(&dir, &c.sha).ok();
        events.push(ActivityEvent {
            timestamp: c.linked_at.clone(),
            event_type: "commit_linked".to_string(),
            title: commit
                .as_ref()
                .map(|ci| ci.message.clone())
                .unwrap_or_else(|| c.sha[..8.min(c.sha.len())].to_string()),
            detail: c.stage.clone(),
            doc_id: c.doc_id.clone(),
            stage: c.stage.clone(),
        });
    }

    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    events.truncate(50);

    Ok(events)
}

// ── Find issue by pipeline run ──

#[tauri::command]
pub fn find_issue_by_run(run_id: String, state: State<AppState>) -> Result<Option<IssueInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let issue = db
        .find_issue_by_run_id(&run_id)
        .map_err(|e| e.to_string())?;

    Ok(issue.as_ref().map(issue_to_info))
}
