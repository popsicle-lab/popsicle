use std::fs;
use std::path::PathBuf;

use storage::WorkspaceStore;
use tauri::State;

use crate::global_config::WorkspaceSource;
use crate::self_host::{binary_provenance_for, load_pipeline_def, Workspace};
use crate::workspace_readers::{
    guidance_for_issue, intent_fallback_mermaid, list_products, read_intent_file, read_task,
    resolve_intent_ref, scan_intents, scan_product_tasks, scan_tasks, task_graph_mermaid,
    IntentBlockDetail, IntentFileFull, IntentGraph, IssueGuidance, TaskFull, TaskGraph,
};
use crate::LocalWorkspace;

use super::dto::*;
use super::state::AppState;

fn get_dir(state: &State<AppState>) -> Result<PathBuf, String> {
    state
        .project_dir
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "no project selected".to_string())
}

#[tauri::command]
pub fn get_initial_dir(state: State<AppState>) -> Result<String, String> {
    Ok(state.initial_dir.clone())
}

#[tauri::command]
pub fn set_project_dir(path: String, state: State<AppState>) -> Result<(), String> {
    let root = PathBuf::from(&path);
    if !root.join(".popsicle").is_dir() {
        return Err(format!(
            "{path} is not a popsicle workspace (missing .popsicle/)"
        ));
    }
    *state.project_dir.lock().map_err(|e| e.to_string())? = Some(root);
    Ok(())
}

#[tauri::command]
pub fn get_workspace_info(state: State<AppState>) -> Result<WorkspaceInfo, String> {
    let dir = get_dir(&state)?;
    let store = LocalWorkspace::open_at(dir.clone()).map_err(|e| e.to_string())?;
    let prov = binary_provenance_for(&Workspace::at(dir.clone()), WorkspaceSource::CliFlag)
        .map_err(|e| e.to_string())?;
    Ok(WorkspaceInfo {
        root: dir.display().to_string(),
        storage_backend: store.backend().describe(store.workspace()),
        binary_match: prov.current_workspace_binary_match,
        executable_path: prov.executable_path,
    })
}

#[tauri::command]
pub fn list_issues(state: State<AppState>) -> Result<Vec<IssueInfo>, String> {
    state.with_store(|store| {
        let rows = store.list_issues().map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let active = store.active_run_id(&row.key).ok().flatten();
            let run_ids = store.run_ids_for_issue(&row.key);
            out.push(IssueInfo {
                key: row.key,
                title: row.title,
                issue_type: row.issue_type,
                priority: row.priority,
                status: row.status,
                spec_id: row.spec_id,
                pipeline: row.pipeline,
                description: row.description,
                active_run_id: active,
                run_ids,
            });
        }
        Ok(out)
    })
}

#[tauri::command]
pub fn get_issue(key: String, state: State<AppState>) -> Result<IssueInfo, String> {
    state.with_store(|store| {
        let row = store.get_issue(&key).map_err(|e| e.to_string())?;
        let active = store.active_run_id(&key).ok().flatten();
        let run_ids = store.run_ids_for_issue(&key);
        Ok(IssueInfo {
            key: row.key,
            title: row.title,
            issue_type: row.issue_type,
            priority: row.priority,
            status: row.status,
            spec_id: row.spec_id,
            pipeline: row.pipeline,
            description: row.description,
            active_run_id: active,
            run_ids,
        })
    })
}

#[tauri::command]
pub fn create_issue(
    issue_type: String,
    title: String,
    spec_id: String,
    pipeline: Option<String>,
    priority: Option<String>,
    description: Option<String>,
    state: State<AppState>,
) -> Result<IssueInfo, String> {
    state.with_store(|store| {
        let row = store
            .create_issue(
                &issue_type,
                &title,
                &spec_id,
                pipeline.as_deref(),
                priority.as_deref().unwrap_or("medium"),
                description.as_deref().unwrap_or(""),
            )
            .map_err(|e| e.to_string())?;
        Ok(IssueInfo {
            key: row.key.clone(),
            title: row.title,
            issue_type: row.issue_type,
            priority: row.priority,
            status: row.status,
            spec_id: row.spec_id,
            pipeline: row.pipeline,
            description: row.description,
            active_run_id: None,
            run_ids: vec![],
        })
    })
}

#[tauri::command]
pub fn start_issue(key: String, state: State<AppState>) -> Result<String, String> {
    state.with_store(|store| {
        let row = store.start_issue(&key, "", "").map_err(|e| e.to_string())?;
        Ok(row.run_id)
    })
}

#[tauri::command]
pub fn list_docs_for_run(run_id: String, state: State<AppState>) -> Result<Vec<DocInfo>, String> {
    state.with_store(|store| {
        let docs = store.list_docs(Some(&run_id)).map_err(|e| e.to_string())?;
        Ok(docs
            .into_iter()
            .map(|d| DocInfo {
                id: d.id,
                doc_type: d.doc_type,
                title: d.title,
                status: d.status,
                file_path: d.file_path,
            })
            .collect())
    })
}

#[tauri::command]
pub fn read_doc(doc_id: String, state: State<AppState>) -> Result<DocFull, String> {
    state.with_store(|store| {
        let row = store.get_doc(&doc_id).map_err(|e| e.to_string())?;
        let abs = store.workspace().root.join(&row.file_path);
        let body = fs::read_to_string(&abs).unwrap_or_default();
        let check = store.check_doc(&doc_id).map_err(|e| e.to_string())?;
        Ok(DocFull {
            id: row.id,
            doc_type: row.doc_type,
            title: row.title,
            status: row.status,
            file_path: row.file_path,
            body,
            check_passed: check.passed,
        })
    })
}

#[tauri::command]
pub fn get_pipeline_status(
    run_id: String,
    state: State<AppState>,
) -> Result<PipelineStatusFull, String> {
    state.with_store(|store| {
        let snap = store.pipeline_status(&run_id).map_err(|e| e.to_string())?;
        let issue_key = store.issue_key_for_run(&run_id).unwrap_or_default();
        let pipeline_name = store
            .pipeline_name_for_run(&run_id)
            .map_err(|e| e.to_string())?;
        let pipeline_def =
            load_pipeline_def(store.workspace(), &pipeline_name).map_err(|e| e.to_string())?;
        let session = store.load_run_session(&run_id).map_err(|e| e.to_string())?;
        let docs = store.list_docs(Some(&run_id)).map_err(|e| e.to_string())?;

        let stages: Vec<StageStatusInfo> = pipeline_def
            .stages
            .iter()
            .enumerate()
            .map(|(i, def)| {
                let st = session.stages.get(i);
                let state_str = st
                    .map(|s| match s.status {
                        skill_runtime::domain::StageStatus::StageBlocked => "blocked",
                        skill_runtime::domain::StageStatus::StageReady => "ready",
                        skill_runtime::domain::StageStatus::StageInProgress => "in_progress",
                        skill_runtime::domain::StageStatus::StageCompleted => "completed",
                        skill_runtime::domain::StageStatus::StageError => "error",
                    })
                    .unwrap_or("blocked");
                let skills: Vec<String> =
                    def.skill_names().into_iter().map(str::to_string).collect();
                let stage_docs: Vec<DocInfo> = docs
                    .iter()
                    .filter(|d| skills.contains(&d.doc_type))
                    .map(|d| DocInfo {
                        id: d.id.clone(),
                        doc_type: d.doc_type.clone(),
                        title: d.title.clone(),
                        status: d.status.clone(),
                        file_path: d.file_path.clone(),
                    })
                    .collect();
                StageStatusInfo {
                    name: def.name.clone(),
                    state: state_str.to_string(),
                    skills,
                    description: def.description.clone(),
                    depends_on: def.depends_on.clone(),
                    documents: stage_docs,
                    requires_approval: def.requires_approval,
                }
            })
            .collect();

        Ok(PipelineStatusFull {
            id: run_id,
            pipeline_name: snap.pipeline_name,
            issue_key,
            run_status: snap.run_status,
            current_stage: snap.current_stage,
            stages,
        })
    })
}

#[tauri::command]
pub fn complete_stage(
    run_id: String,
    stage_name: String,
    confirm: bool,
    state: State<AppState>,
) -> Result<StageCompleteResult, String> {
    state.with_store(|store| {
        let result = store
            .complete_stage(&stage_name, &run_id, confirm)
            .map_err(|e| e.to_string())?;
        Ok(StageCompleteResult {
            current_stage: result.current_stage,
            downstream_ready: result.downstream_ready,
        })
    })
}

#[tauri::command]
pub fn scan_task_graph(state: State<AppState>) -> Result<TaskGraph, String> {
    let dir = get_dir(&state)?;
    scan_tasks(&dir.join("products")).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn task_graph_mermaid_cmd(state: State<AppState>) -> Result<String, String> {
    let dir = get_dir(&state)?;
    let graph = scan_tasks(&dir.join("products")).map_err(|e| e.to_string())?;
    Ok(task_graph_mermaid(&graph))
}

#[tauri::command]
pub fn list_product_names(state: State<AppState>) -> Result<Vec<String>, String> {
    let dir = get_dir(&state)?;
    list_products(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_intent_graph(product: String, state: State<AppState>) -> Result<IntentGraph, String> {
    let dir = get_dir(&state)?;
    scan_intents(&dir, &product).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn intent_graph_mermaid(product: String, state: State<AppState>) -> Result<String, String> {
    let dir = get_dir(&state)?;
    let graph = scan_intents(&dir, &product).map_err(|e| e.to_string())?;
    if let Some(m) = graph.mermaid {
        return Ok(m);
    }
    Ok(intent_fallback_mermaid(&graph))
}

#[tauri::command]
pub fn scan_product_task_graph(
    product: String,
    state: State<AppState>,
) -> Result<TaskGraph, String> {
    let dir = get_dir(&state)?;
    scan_product_tasks(&dir, &product).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_task_content(
    task_id: String,
    product: Option<String>,
    state: State<AppState>,
) -> Result<TaskFull, String> {
    let dir = get_dir(&state)?;
    read_task(&dir, &task_id, product.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_intent_file_cmd(
    product: String,
    file: String,
    state: State<AppState>,
) -> Result<IntentFileFull, String> {
    let dir = get_dir(&state)?;
    read_intent_file(&dir, &product, &file).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_intent_ref_cmd(
    reference: String,
    product: Option<String>,
    state: State<AppState>,
) -> Result<IntentBlockDetail, String> {
    let dir = get_dir(&state)?;
    resolve_intent_ref(&dir, &reference, product.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_issue_guidance(
    issue_key: String,
    state: State<AppState>,
) -> Result<IssueGuidance, String> {
    state.with_store(|store| {
        let issue = store.get_issue(&issue_key).map_err(|e| e.to_string())?;
        let pipeline_stage = if let Some(run_id) = store.active_run_id(&issue.key).ok().flatten() {
            store.pipeline_status(&run_id).ok().map(|s| s.current_stage)
        } else {
            None
        };
        let dir = store.workspace().root.clone();
        guidance_for_issue(
            &dir,
            &issue.spec_id,
            &issue.issue_type,
            &issue.status,
            pipeline_stage.as_deref(),
        )
        .map_err(|e| e.to_string())
    })
}
