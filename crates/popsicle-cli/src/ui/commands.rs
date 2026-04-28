use std::path::PathBuf;

use popsicle_core::dto::*;
use popsicle_core::engine::{Advisor, count_checkboxes};
use popsicle_core::git::GitTracker;
use popsicle_core::helpers;
use popsicle_core::memory::{MemoryLayer, MemoryStore, MemoryType};
use popsicle_core::model::{
    Issue, IssueStatus, IssueType, PipelineRun, Priority, Spec, StageState, WorkItem, WorkItemKind,
};
use popsicle_core::storage::{DocumentRow, FileStorage, IndexDb, ProjectConfig, ProjectLayout};
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
pub fn get_initial_dir(state: State<AppState>) -> String {
    state.initial_dir.clone()
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
            doc_lifecycle: s.doc_lifecycle.to_string(),
            workflow_initial: s.workflow.initial.clone(),
            inputs: s
                .inputs
                .iter()
                .map(|i| SkillInputInfo {
                    from_skill: i.from_skill.clone(),
                    artifact_type: i.artifact_type.clone(),
                    required: i.required,
                    relevance: i.relevance.to_string(),
                    sections: i.sections.clone(),
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
    Ok(runs.iter().map(run_row_to_info).collect())
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
                requires_approval: stage.requires_approval,
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

    let doc_tags: Vec<String> = serde_json::from_str(&doc_row.doc_tags).unwrap_or_default();

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
        summary: doc_row.summary.clone(),
        doc_tags,
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

    // Load all docs from the same spec for cross-run visibility
    let spec_docs = db.query_spec_documents(&run.spec_id).unwrap_or_default();

    let steps = Advisor::next_steps(&pipeline_def, &run, &registry, &docs, &spec_docs);
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
            context_command: s.context_command.clone(),
            hints: s.hints.clone(),
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
                if d.status != "final" {
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

// ── Document search ──

#[tauri::command]
pub fn search_documents(
    query: String,
    status: Option<String>,
    skill: Option<String>,
    exclude_run: Option<String>,
    limit: Option<usize>,
    state: State<AppState>,
) -> Result<Vec<SearchDocResult>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let results = db
        .search_documents(
            &query,
            status.as_deref(),
            skill.as_deref(),
            exclude_run.as_deref(),
            limit.unwrap_or(20),
        )
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|(row, score)| {
            let tags: Vec<String> = serde_json::from_str(&row.doc_tags).unwrap_or_default();
            SearchDocResult {
                id: row.id,
                doc_type: row.doc_type,
                title: row.title,
                status: row.status,
                skill_name: row.skill_name,
                pipeline_run_id: row.pipeline_run_id,
                file_path: row.file_path,
                summary: row.summary,
                doc_tags: tags,
                bm25_score: score,
            }
        })
        .collect())
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
        spec_id: i.spec_id.clone(),
        pipeline: i.pipeline.clone(),
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
        .query_issues(
            issue_type.as_deref(),
            status.as_deref(),
            label.as_deref(),
            None,
        )
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
        spec_id: issue.spec_id,
        pipeline: issue.pipeline,
        labels: issue.labels,
        created_at: issue.created_at.to_rfc3339(),
        updated_at: issue.updated_at.to_rfc3339(),
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_issue(
    issue_type: String,
    title: String,
    spec_name: String,
    description: Option<String>,
    priority: Option<String>,
    pipeline: Option<String>,
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

    if let Some(ref name) = pipeline {
        helpers::find_pipeline(&dir, name)
            .map_err(|_| format!("Pipeline template not found: {}", name))?;
    }

    // Resolve or create the spec
    let spec = if let Some(t) = db
        .find_spec_by_name(&spec_name)
        .map_err(|e| e.to_string())?
    {
        t
    } else {
        let t = Spec::new(&spec_name, "", "");
        db.create_spec(&t).map_err(|e| e.to_string())?;
        t
    };

    let prefix = config.project.key_prefix_or_default();
    let seq = db.next_issue_seq(prefix).map_err(|e| e.to_string())?;
    let key = format!("{}-{}", prefix, seq);

    let mut issue = Issue::new(key, &title, it, &spec.id);
    issue.description = description.unwrap_or_default();
    issue.priority = pr;
    issue.pipeline = pipeline;
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

    let pipelines = helpers::load_pipelines(&dir).map_err(|e| e.to_string())?;
    let resolved = helpers::resolve_pipeline_for_issue(&issue, &pipelines).ok_or_else(|| {
        format!(
            "Could not determine pipeline for issue type '{}'",
            issue.issue_type
        )
    })?;

    let pipeline_def =
        helpers::find_pipeline(&dir, &resolved.pipeline_name).map_err(|e| e.to_string())?;
    pipeline_def.validate().map_err(|e| e.to_string())?;

    // Issue already has spec_id
    let spec = db
        .get_spec(&issue.spec_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Spec not found for issue: {}", issue.spec_id))?;

    let run = PipelineRun::new(&pipeline_def, &issue.title, &spec.id, &issue.id);
    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir).map_err(|e| e.to_string())?;

    db.upsert_pipeline_run(&run).map_err(|e| e.to_string())?;

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

    let issue = db
        .get_issue(&key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Issue not found: {}", key))?;

    // Get the latest pipeline run for this issue
    let runs = db
        .find_runs_by_issue(&issue.id)
        .map_err(|e| e.to_string())?;

    let run_info_list: Vec<PipelineRunInfo> = runs.iter().map(run_row_to_info).collect();

    let latest_run = match runs.first() {
        Some(r) => r,
        None => {
            return Ok(IssueProgress {
                issue_key: issue.key,
                spec_id: issue.spec_id,
                pipeline_runs: run_info_list,
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

    let run_id = &latest_run.id;
    let run = db
        .get_pipeline_run(run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def =
        helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;

    let docs = db
        .query_documents(None, None, Some(run_id))
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
                if d.status == "final" {
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
        spec_id: issue.spec_id,
        pipeline_runs: run_info_list,
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
        if let Some(ref ts) = d.updated_at
            && d.updated_at != d.created_at
        {
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
pub fn find_issue_by_run(
    run_id: String,
    state: State<AppState>,
) -> Result<Option<IssueInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let issue = db
        .find_issue_by_run_id(&run_id)
        .map_err(|e| e.to_string())?;

    Ok(issue.as_ref().map(issue_to_info))
}

// ── Project context ──

#[tauri::command]
pub fn get_project_context(state: State<AppState>) -> Result<ProjectContextInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let path = layout.project_context_path();

    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        Ok(ProjectContextInfo {
            available: true,
            content: Some(content),
            path: Some(path.display().to_string()),
        })
    } else {
        Ok(ProjectContextInfo {
            available: false,
            content: None,
            path: None,
        })
    }
}

// ── WorkItem commands (unified bug/story/test_case) ──

fn work_item_to_info(w: &WorkItem) -> WorkItemInfo {
    WorkItemInfo {
        id: w.id.clone(),
        key: w.key.clone(),
        kind: w.kind.as_str().to_string(),
        title: w.title.clone(),
        status: w.status.clone(),
        priority: w.priority.to_string(),
        labels: w.labels.clone(),
        issue_id: w.issue_id.clone(),
        pipeline_run_id: w.pipeline_run_id.clone(),
        created_at: w.created_at.to_rfc3339(),
        updated_at: w.updated_at.to_rfc3339(),
    }
}

fn work_item_to_full(w: WorkItem) -> WorkItemFull {
    WorkItemFull {
        id: w.id,
        key: w.key,
        kind: w.kind.as_str().to_string(),
        title: w.title,
        description: w.description,
        status: w.status,
        priority: w.priority.to_string(),
        labels: w.labels,
        issue_id: w.issue_id,
        pipeline_run_id: w.pipeline_run_id,
        source_doc_id: w.source_doc_id,
        fields: w.fields,
        created_at: w.created_at.to_rfc3339(),
        updated_at: w.updated_at.to_rfc3339(),
    }
}

#[tauri::command]
pub fn list_work_items(
    kind: Option<String>,
    status: Option<String>,
    issue_id: Option<String>,
    run_id: Option<String>,
    state: State<AppState>,
) -> Result<Vec<WorkItemInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let k = match kind {
        Some(s) => Some(s.parse::<WorkItemKind>().map_err(|e| e.to_string())?),
        None => None,
    };
    let items = db
        .query_work_items(k, status.as_deref(), issue_id.as_deref(), run_id.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(items.iter().map(work_item_to_info).collect())
}

#[tauri::command]
pub fn get_work_item(key: String, state: State<AppState>) -> Result<WorkItemFull, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let item = db
        .get_work_item(&key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Work item not found: {}", key))?;
    Ok(work_item_to_full(item))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_work_item(
    kind: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    issue_id: Option<String>,
    run_id: Option<String>,
    fields: Option<serde_json::Value>,
    state: State<AppState>,
) -> Result<WorkItemInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let config = ProjectConfig::load(&layout.config_path()).map_err(|e| e.to_string())?;
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let k: WorkItemKind = kind.parse().map_err(|e: String| e)?;
    let prefix = config.project.key_prefix_or_default();
    let seq = db
        .next_work_item_seq(k, prefix)
        .map_err(|e| e.to_string())?;
    let key = format!("{}-{}-{}", k.key_prefix(), prefix, seq);

    let mut wi = WorkItem::new(key, k, &title);
    if let Some(d) = description {
        wi.description = d;
    }
    if let Some(p) = priority {
        wi.priority = p.parse().map_err(|e: String| e)?;
    }
    wi.issue_id = issue_id;
    wi.pipeline_run_id = run_id;
    if let Some(f) = fields
        && f.is_object()
    {
        wi.fields = f;
    }

    db.create_work_item(&wi).map_err(|e| e.to_string())?;
    Ok(work_item_to_info(&wi))
}

#[tauri::command]
pub fn update_work_item(
    key: String,
    title: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    fields: Option<serde_json::Value>,
    state: State<AppState>,
) -> Result<WorkItemInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let mut wi = db
        .get_work_item(&key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Work item not found: {}", key))?;

    if let Some(t) = title {
        wi.title = t;
    }
    if let Some(s) = status {
        wi.status = s;
    }
    if let Some(p) = priority {
        wi.priority = p.parse().map_err(|e: String| e)?;
    }
    if let Some(f) = fields
        && let Some(obj) = f.as_object()
    {
        for (k, v) in obj {
            wi.set_field(k, v.clone());
        }
    }
    wi.updated_at = chrono::Utc::now();

    db.update_work_item(&wi).map_err(|e| e.to_string())?;
    Ok(work_item_to_info(&wi))
}

// ── Memory commands ──

fn memory_to_info(m: &popsicle_core::memory::Memory) -> MemoryInfo {
    MemoryInfo {
        id: m.id,
        memory_type: m.memory_type.to_string(),
        summary: m.summary.clone(),
        created: m.created.clone(),
        layer: m.layer.to_string(),
        refs: m.refs,
        tags: m.tags.clone(),
        files: m.files.clone(),
        run: m.run.clone(),
        stale: m.stale,
        detail: m.detail.clone(),
    }
}

#[tauri::command]
pub fn list_memories(
    layer: Option<String>,
    memory_type: Option<String>,
    state: State<AppState>,
) -> Result<Vec<MemoryInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let path = layout.memories_path();
    let memories = MemoryStore::load(&path).map_err(|e| e.to_string())?;

    let layer_filter: Option<MemoryLayer> = layer
        .as_deref()
        .map(|s| s.parse().map_err(|e: String| e))
        .transpose()?;

    let type_filter: Option<MemoryType> = memory_type
        .as_deref()
        .map(|s| s.parse().map_err(|e: String| e))
        .transpose()?;

    let filtered: Vec<MemoryInfo> = memories
        .iter()
        .filter(|m| layer_filter.is_none_or(|l| m.layer == l))
        .filter(|m| type_filter.is_none_or(|t| m.memory_type == t))
        .map(memory_to_info)
        .collect();

    Ok(filtered)
}

#[tauri::command]
pub fn get_memory_stats(state: State<AppState>) -> Result<MemoryStatsInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let path = layout.memories_path();
    let memories = MemoryStore::load(&path).map_err(|e| e.to_string())?;

    Ok(MemoryStatsInfo {
        line_count: MemoryStore::line_count(&memories),
        max_lines: popsicle_core::memory::MAX_LINES,
        total: memories.len(),
        long_term: memories
            .iter()
            .filter(|m| m.layer == MemoryLayer::LongTerm)
            .count(),
        short_term: memories
            .iter()
            .filter(|m| m.layer == MemoryLayer::ShortTerm)
            .count(),
        bugs: memories
            .iter()
            .filter(|m| m.memory_type == MemoryType::Bug)
            .count(),
        decisions: memories
            .iter()
            .filter(|m| m.memory_type == MemoryType::Decision)
            .count(),
        patterns: memories
            .iter()
            .filter(|m| m.memory_type == MemoryType::Pattern)
            .count(),
        gotchas: memories
            .iter()
            .filter(|m| m.memory_type == MemoryType::Gotcha)
            .count(),
        stale: memories.iter().filter(|m| m.stale).count(),
    })
}

#[tauri::command]
pub fn get_memory(id: u32, state: State<AppState>) -> Result<MemoryInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let path = layout.memories_path();
    let memories = MemoryStore::load(&path).map_err(|e| e.to_string())?;

    let memory = memories
        .iter()
        .find(|m| m.id == id)
        .ok_or_else(|| format!("Memory #{} not found", id))?;

    Ok(memory_to_info(memory))
}

#[tauri::command]
pub fn list_specs(state: State<AppState>) -> Result<Vec<SpecInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let specs = db.list_specs().map_err(|e| e.to_string())?;
    Ok(specs.iter().map(|t| spec_to_info(&db, t)).collect())
}

fn run_row_to_info(r: &popsicle_core::storage::PipelineRunRow) -> PipelineRunInfo {
    PipelineRunInfo {
        id: r.id.clone(),
        pipeline_name: r.pipeline_name.clone(),
        title: r.title.clone(),
        created_at: r.created_at.clone(),
        updated_at: r.updated_at.clone(),
        spec_id: r.spec_id.clone(),
        // Storage layer flattens NULL into an empty string; surface it as
        // an absent field so the frontend gets `string | undefined` semantics.
        issue_id: if r.issue_id.is_empty() {
            None
        } else {
            Some(r.issue_id.clone())
        },
        run_type: r.run_type.clone(),
    }
}

fn spec_to_info(db: &IndexDb, t: &Spec) -> SpecInfo {
    let run_count = db
        .list_spec_runs(&t.id)
        .map(|r| r.len() as u32)
        .unwrap_or(0);
    let doc_count = db
        .query_spec_documents(&t.id)
        .map(|d| d.len() as u32)
        .unwrap_or(0);
    SpecInfo {
        id: t.id.clone(),
        name: t.name.clone(),
        slug: t.slug.clone(),
        description: t.description.clone(),
        namespace_id: t.namespace_id.clone(),
        tags: t.tags.clone(),
        created_at: t.created_at.to_rfc3339(),
        run_count,
        doc_count,
        locked_by_run_id: t.locked_by_run_id.clone(),
        locked_at: t.locked_at.map(|d| d.to_rfc3339()),
    }
}

#[tauri::command]
pub fn get_spec(spec_name: String, state: State<AppState>) -> Result<SpecDetailInfo, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let spec = db
        .find_spec_by_name(&spec_name)
        .map_err(|e| e.to_string())?
        .or_else(|| db.get_spec(&spec_name).ok().flatten())
        .ok_or_else(|| format!("Spec not found: {}", spec_name))?;

    let runs = db.list_spec_runs(&spec.id).map_err(|e| e.to_string())?;
    let docs = db
        .query_spec_documents(&spec.id)
        .map_err(|e| e.to_string())?;
    let issues = db
        .query_issues(None, None, None, Some(&spec.id))
        .map_err(|e| e.to_string())?;

    Ok(SpecDetailInfo {
        id: spec.id.clone(),
        name: spec.name.clone(),
        slug: spec.slug.clone(),
        description: spec.description.clone(),
        namespace_id: spec.namespace_id.clone(),
        tags: spec.tags.clone(),
        created_at: spec.created_at.to_rfc3339(),
        locked_by_run_id: spec.locked_by_run_id.clone(),
        locked_at: spec.locked_at.map(|d| d.to_rfc3339()),
        runs: runs.iter().map(run_row_to_info).collect(),
        documents: docs.iter().map(doc_row_to_info).collect(),
        issues: issues.iter().map(issue_to_info).collect(),
    })
}

// ── Namespace entity commands ──

#[tauri::command]
pub fn list_namespace_entities(state: State<AppState>) -> Result<Vec<NamespaceEntityInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let namespaces = db.list_namespaces(None).map_err(|e| e.to_string())?;
    Ok(namespaces
        .iter()
        .map(|p| {
            let spec_count = db
                .list_specs_by_namespace(Some(&p.id))
                .map(|t| t.len() as u32)
                .unwrap_or(0);
            NamespaceEntityInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                slug: p.slug.clone(),
                description: p.description.clone(),
                status: p.status.to_string(),
                tags: p.tags.clone(),
                spec_count,
                created_at: p.created_at.to_rfc3339(),
                updated_at: p.updated_at.to_rfc3339(),
            }
        })
        .collect())
}

#[tauri::command]
pub fn get_namespace_entity(
    namespace_id: String,
    state: State<AppState>,
) -> Result<NamespaceEntityDetail, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let namespace = db
        .find_namespace_by_name(&namespace_id)
        .ok()
        .flatten()
        .or_else(|| db.get_namespace(&namespace_id).ok().flatten())
        .ok_or_else(|| format!("Namespace not found: {}", namespace_id))?;

    let specs = db
        .list_specs_by_namespace(Some(&namespace.id))
        .map_err(|e| e.to_string())?;

    Ok(NamespaceEntityDetail {
        id: namespace.id.clone(),
        name: namespace.name.clone(),
        slug: namespace.slug.clone(),
        description: namespace.description.clone(),
        status: namespace.status.to_string(),
        tags: namespace.tags.clone(),
        specs: specs.iter().map(|t| spec_to_info(&db, t)).collect(),
        created_at: namespace.created_at.to_rfc3339(),
        updated_at: namespace.updated_at.to_rfc3339(),
    })
}

#[tauri::command]
pub fn complete_stage(
    run_id: String,
    stage_name: String,
    confirm: bool,
    state: State<AppState>,
) -> Result<StageCompleteResult, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;

    let mut run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def =
        helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;

    let stage = pipeline_def
        .stages
        .iter()
        .find(|s| s.name == stage_name)
        .ok_or_else(|| {
            format!(
                "Stage '{}' not found in pipeline '{}'",
                stage_name, pipeline_def.name
            )
        })?;

    let current = run
        .stage_states
        .get(&stage.name)
        .copied()
        .unwrap_or(StageState::Blocked);

    if !matches!(current, StageState::Ready | StageState::InProgress) {
        return Err(format!(
            "Stage '{}' is '{}', can only complete from 'ready' or 'in_progress'",
            stage_name, current
        ));
    }

    if stage.requires_approval && !confirm {
        return Err(format!(
            "Stage '{}' requires human approval. Please confirm.",
            stage_name
        ));
    }

    // Verify docs exist for all skills in this stage
    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| e.to_string())?;
    let missing_skills: Vec<&str> = stage
        .skill_names()
        .into_iter()
        .filter(|sn| !docs.iter().any(|d| d.skill_name == *sn))
        .collect();
    if !missing_skills.is_empty() {
        return Err(format!(
            "Stage '{}' cannot be completed — missing documents for skills: {}",
            stage_name,
            missing_skills.join(", ")
        ));
    }

    // Mark all docs in this stage as "final"
    let stage_skills: Vec<&str> = stage.skill_names();
    for doc_row in &docs {
        if stage_skills.contains(&doc_row.skill_name.as_str()) && doc_row.status != "final" {
            let _ = db.update_document_status(&doc_row.id, "final");
            if let Ok(mut doc) =
                FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
            {
                doc.status = "final".to_string();
                doc.updated_at = Some(chrono::Utc::now());
                let _ = FileStorage::write_document(&doc, std::path::Path::new(&doc_row.file_path));
            }
        }
    }

    // Transition stage to Completed
    run.stage_states
        .insert(stage.name.clone(), StageState::Completed);
    run.refresh_states(&pipeline_def);
    run.updated_at = chrono::Utc::now();
    db.upsert_pipeline_run(&run).map_err(|e| e.to_string())?;

    // Check if all stages are done → auto-release spec lock + mark issue done
    let all_done = pipeline_def.stages.iter().all(|s| {
        matches!(
            run.stage_states.get(&s.name),
            Some(StageState::Completed | StageState::Skipped)
        )
    });

    let mut auto_released = false;
    if all_done {
        let _ = db.release_spec_lock(&run.spec_id, Some(&run.id));
        auto_released = true;

        if let Ok(Some(mut issue)) = db.find_issue_by_run_id(&run.id)
            && issue.status != IssueStatus::Done
        {
            issue.status = IssueStatus::Done;
            let _ = db.update_issue(&issue);
        }
    }

    let unblocked: Vec<String> = if !all_done {
        pipeline_def
            .stages
            .iter()
            .filter(|s| {
                run.stage_states.get(&s.name) == Some(&StageState::Ready)
                    && s.depends_on.contains(&stage_name)
            })
            .map(|s| s.name.clone())
            .collect()
    } else {
        vec![]
    };

    Ok(StageCompleteResult {
        stage: stage_name,
        state: "completed".to_string(),
        run_id: run.id,
        all_done,
        auto_released,
        unblocked,
    })
}
