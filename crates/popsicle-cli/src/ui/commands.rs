use std::path::PathBuf;

use popsicle_core::dto::*;
use popsicle_core::engine::Advisor;
use popsicle_core::git::GitTracker;
use popsicle_core::helpers;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};
use tauri::State;

use super::AppState;

fn get_dir(state: &State<AppState>) -> Result<PathBuf, String> {
    let guard = state.project_dir.lock().map_err(|e| e.to_string())?;
    guard
        .as_ref()
        .map(PathBuf::from)
        .ok_or_else(|| "No project directory set".to_string())
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
            artifact_types: s.artifacts.iter().map(|a| a.artifact_type.clone()).collect(),
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

    let pipeline_def = helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;
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
                .map(|d| DocInfo {
                    id: d.id.clone(),
                    doc_type: d.doc_type.clone(),
                    title: d.title.clone(),
                    status: d.status.clone(),
                    skill_name: d.skill_name.clone(),
                    created_at: d.created_at.clone(),
                    updated_at: d.updated_at.clone(),
                })
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
        .query_documents(
            skill.as_deref(),
            status.as_deref(),
            run_id.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    Ok(docs
        .iter()
        .map(|d| DocInfo {
            id: d.id.clone(),
            doc_type: d.doc_type.clone(),
            title: d.title.clone(),
            status: d.status.clone(),
            skill_name: d.skill_name.clone(),
            created_at: d.created_at.clone(),
            updated_at: d.updated_at.clone(),
        })
        .collect())
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

    let doc =
        FileStorage::read_document(std::path::Path::new(&doc_row.file_path)).map_err(|e| e.to_string())?;

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
pub fn get_next_steps(
    run_id: String,
    state: State<AppState>,
) -> Result<Vec<NextStepInfo>, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let registry = helpers::load_registry(&dir).map_err(|e| e.to_string())?;

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def = helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;
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
pub fn verify_pipeline_run(
    run_id: String,
    state: State<AppState>,
) -> Result<VerifyResult, String> {
    let dir = get_dir(&state)?;
    let layout = ProjectLayout::new(&dir);
    let db = IndexDb::open(&layout.db_path()).map_err(|e| e.to_string())?;
    let registry = helpers::load_registry(&dir).map_err(|e| e.to_string())?;

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def = helpers::find_pipeline(&dir, &run.pipeline_name).map_err(|e| e.to_string())?;
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
                    issues.push(format!(
                        "'{}' is '{}', not final",
                        d.title, d.status
                    ));
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
    let config =
        popsicle_core::storage::ProjectConfig::load(&layout.config_path()).map_err(|e| e.to_string())?;
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
        let p = links.iter().filter(|l| l.review_status == popsicle_core::git::ReviewStatus::Pending).count();
        let pa = links.iter().filter(|l| l.review_status == popsicle_core::git::ReviewStatus::Passed).count();
        let f = links.iter().filter(|l| l.review_status == popsicle_core::git::ReviewStatus::Failed).count();
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
                short_sha: commit.as_ref().map(|c| c.short_sha.clone()).unwrap_or_else(|| l.sha[..8.min(l.sha.len())].to_string()),
                message: commit.as_ref().map(|c| c.message.clone()).unwrap_or_default(),
                author: commit.as_ref().map(|c| c.author.clone()).unwrap_or_default(),
                timestamp: commit.as_ref().map(|c| c.timestamp.clone()).unwrap_or_default(),
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

