use std::path::PathBuf;

use popsicle_core::engine::Advisor;
use popsicle_core::git::GitTracker;
use popsicle_core::model::PipelineDef;
use popsicle_core::registry::{PipelineLoader, SkillLoader, SkillRegistry};
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};
use serde::Serialize;
use tauri::State;

use super::AppState;

fn get_dir(state: &State<AppState>) -> Result<PathBuf, String> {
    let guard = state.project_dir.lock().map_err(|e| e.to_string())?;
    guard
        .as_ref()
        .map(PathBuf::from)
        .ok_or_else(|| "No project directory set".to_string())
}

fn load_registry(project_dir: &PathBuf) -> Result<SkillRegistry, String> {
    let mut registry = SkillRegistry::new();
    let skills_dir = project_dir.join("skills");
    if skills_dir.is_dir() {
        SkillLoader::load_dir(&skills_dir, &mut registry).map_err(|e| e.to_string())?;
    }
    let local_skills = project_dir.join(".popsicle").join("skills");
    if local_skills.is_dir() {
        SkillLoader::load_dir(&local_skills, &mut registry).map_err(|e| e.to_string())?;
    }
    Ok(registry)
}

fn load_pipelines(project_dir: &PathBuf) -> Result<Vec<PipelineDef>, String> {
    let mut all = Vec::new();
    for dir in [
        project_dir.join("pipelines"),
        project_dir.join(".popsicle").join("pipelines"),
    ] {
        if dir.is_dir() {
            all.extend(PipelineLoader::load_dir(&dir).map_err(|e| e.to_string())?);
        }
    }
    Ok(all)
}

fn find_pipeline(project_dir: &PathBuf, name: &str) -> Result<PipelineDef, String> {
    load_pipelines(project_dir)?
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Pipeline not found: {}", name))
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
    let registry = load_registry(&dir)?;
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
    let pipelines = load_pipelines(&dir)?;
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

    let pipeline_def = find_pipeline(&dir, &run.pipeline_name)?;
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
    let registry = load_registry(&dir)?;

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def = find_pipeline(&dir, &run.pipeline_name)?;
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
    let registry = load_registry(&dir)?;
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
    let registry = load_registry(&dir)?;

    let run = db
        .get_pipeline_run(&run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Run not found: {}", run_id))?;

    let pipeline_def = find_pipeline(&dir, &run.pipeline_name)?;
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
                if let Ok(skill) = registry.get(&d.skill_name) {
                    if !skill.is_final_state(&d.status) {
                        issues.push(format!(
                            "'{}' is '{}', not final",
                            d.title, d.status
                        ));
                    }
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

#[derive(Serialize)]
pub struct ProjectInfo {
    pub path: String,
    pub initialized: bool,
}

#[derive(Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub artifact_types: Vec<String>,
    pub workflow_initial: String,
    pub inputs: Vec<SkillInputInfo>,
    pub workflow_states: Vec<WorkflowStateInfo>,
}

#[derive(Serialize)]
pub struct SkillInputInfo {
    pub from_skill: String,
    pub artifact_type: String,
    pub required: bool,
}

#[derive(Serialize)]
pub struct WorkflowStateInfo {
    pub name: String,
    pub is_final: bool,
    pub transitions: Vec<TransitionInfo>,
}

#[derive(Serialize)]
pub struct TransitionInfo {
    pub to: String,
    pub action: String,
}

#[derive(Serialize)]
pub struct PipelineInfo {
    pub name: String,
    pub description: String,
    pub stages: Vec<StageInfo>,
}

#[derive(Serialize)]
pub struct StageInfo {
    pub name: String,
    pub skills: Vec<String>,
    pub description: String,
    pub depends_on: Vec<String>,
}

#[derive(Serialize)]
pub struct PipelineRunInfo {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct PipelineStatusFull {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub stages: Vec<StageStatusInfo>,
}

#[derive(Serialize)]
pub struct StageStatusInfo {
    pub name: String,
    pub state: String,
    pub skills: Vec<String>,
    pub description: String,
    pub depends_on: Vec<String>,
    pub documents: Vec<DocInfo>,
}

#[derive(Clone, Serialize)]
pub struct DocInfo {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub skill_name: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
pub struct DocFull {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub skill_name: String,
    pub pipeline_run_id: String,
    pub tags: Vec<String>,
    pub body: String,
    pub file_path: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
pub struct NextStepInfo {
    pub stage: String,
    pub skill: String,
    pub action: String,
    pub description: String,
    pub cli_command: String,
    pub prompt: Option<String>,
    pub blocked_by: Vec<String>,
}

#[derive(Serialize)]
pub struct VerifyResult {
    pub run_id: String,
    pub verified: bool,
    pub issues: Vec<String>,
}

#[derive(Serialize)]
pub struct PromptInfo {
    pub skill: String,
    pub state: String,
    pub prompt: Option<String>,
    pub available_states: Vec<String>,
}

#[derive(Serialize)]
pub struct GitStatusInfo {
    pub branch: String,
    pub head: String,
    pub uncommitted_changes: bool,
    pub pipeline_run_id: Option<String>,
    pub total_commits: usize,
    pub pending_review: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Serialize)]
pub struct CommitLinkInfo {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub doc_id: Option<String>,
    pub pipeline_run_id: String,
    pub stage: Option<String>,
    pub skill: Option<String>,
    pub review_status: String,
    pub review_summary: Option<String>,
    pub linked_at: String,
}
