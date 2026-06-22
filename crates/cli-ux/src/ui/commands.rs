use std::fs;
use std::path::PathBuf;

use storage::WorkspaceStore;
use tauri::State;

use crate::global_config::{
    global_config_path, is_valid_workspace_path, list_projects, open_project,
    open_project_or_bootstrap, remove_project, workspace_needs_bootstrap, WorkspaceSource,
};
use crate::project_config::{
    default_pipelines_by_type, ensure_project_config, load_project_config, project_config_path,
    save_project_config, sync_agents_md, AgentLanguage, ProjectConfig, WorkflowProfile,
};
use crate::self_host::{
    binary_provenance_for, list_installed_pipeline_names, load_pipeline_def, Workspace,
};
use crate::workspace_readers::{
    guidance_for_issue, intent_graph_mermaid as render_intent_graph_mermaid, list_products,
    read_intent_file, read_task, resolve_intent_ref, scan_intents, scan_product_health,
    scan_product_tasks, scan_tasks, task_graph_mermaid, IntentBlockDetail, IntentFileFull,
    IntentGraph, IssueGuidance, ProductHealthReport, TaskFull, TaskGraph,
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
pub fn get_active_project(state: State<AppState>) -> Result<Option<ProjectInfo>, String> {
    let dir = state.project_dir.lock().map_err(|e| e.to_string())?.clone();
    let Some(dir) = dir else {
        return Ok(None);
    };
    let path = dir.display().to_string();
    let cfg = list_projects().map_err(|e| e.to_string())?;
    let entry = cfg.projects.into_iter().find(|p| p.path == path);
    Ok(entry.map(|e| project_info_from_entry(e, cfg.default_project.as_deref())))
}

fn project_info_from_entry(
    entry: crate::global_config::ProjectEntry,
    default_path: Option<&str>,
) -> ProjectInfo {
    let is_default = default_path == Some(entry.path.as_str());
    let is_valid = is_valid_workspace_path(&entry.path);
    ProjectInfo {
        name: entry.name,
        path: entry.path,
        last_opened_at: entry.last_opened_at,
        is_default,
        is_valid,
    }
}

fn apply_project_dir(
    state: &State<AppState>,
    path: &str,
    confirm_bootstrap: bool,
) -> Result<PathBuf, String> {
    let needs = workspace_needs_bootstrap(path).map_err(|e| e.to_string())?;
    if needs && !confirm_bootstrap {
        return Err(
            "workspace not initialized; confirm bootstrap before opening this folder".into(),
        );
    }
    let entry = if needs {
        open_project_or_bootstrap(path, None).map_err(|e| e.to_string())?
    } else {
        open_project(path, None).map_err(|e| e.to_string())?
    };
    let root = PathBuf::from(&entry.path);
    *state.project_dir.lock().map_err(|e| e.to_string())? = Some(root.clone());
    Ok(root)
}

#[tauri::command]
pub fn workspace_needs_bootstrap_cmd(path: String) -> Result<bool, String> {
    workspace_needs_bootstrap(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_project_dir(
    path: String,
    confirm_bootstrap: bool,
    state: State<AppState>,
) -> Result<ProjectInfo, String> {
    let root = apply_project_dir(&state, &path, confirm_bootstrap)?;
    let cfg = list_projects().map_err(|e| e.to_string())?;
    let entry = cfg
        .projects
        .into_iter()
        .find(|p| p.path == root.display().to_string())
        .ok_or_else(|| "project not registered".to_string())?;
    Ok(project_info_from_entry(
        entry,
        cfg.default_project.as_deref(),
    ))
}

#[tauri::command]
pub fn open_project_cmd(
    path: String,
    confirm_bootstrap: bool,
    state: State<AppState>,
) -> Result<ProjectInfo, String> {
    let root = apply_project_dir(&state, &path, confirm_bootstrap)?;
    let cfg = list_projects().map_err(|e| e.to_string())?;
    let entry = cfg
        .projects
        .into_iter()
        .find(|p| p.path == root.display().to_string())
        .ok_or_else(|| "project not registered".to_string())?;
    Ok(project_info_from_entry(
        entry,
        cfg.default_project.as_deref(),
    ))
}

#[tauri::command]
pub fn list_registered_projects() -> Result<ProjectsList, String> {
    let cfg = list_projects().map_err(|e| e.to_string())?;
    let default = cfg.default_project.clone();
    let mut projects: Vec<ProjectInfo> = cfg
        .projects
        .into_iter()
        .map(|e| project_info_from_entry(e, default.as_deref()))
        .collect();
    projects.sort_by(|a, b| {
        b.last_opened_at
            .cmp(&a.last_opened_at)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(ProjectsList {
        projects,
        default_path: default,
        global_config_path: global_config_path()
            .map(|p| p.display().to_string())
            .map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub fn remove_registered_project(name: String) -> Result<(), String> {
    remove_project(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pick_project_directory() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Open Popsicle Project")
        .pick_folder()
        .map(|p| p.display().to_string())
}

#[tauri::command]
pub fn resolve_startup_project(cli_project: Option<String>) -> Result<Option<String>, String> {
    let path = cli_project.map(PathBuf::from);
    crate::global_config::resolve_ui_startup_root(path.as_deref())
        .map(|opt| opt.map(|p| p.display().to_string()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_workspace_info(state: State<AppState>) -> Result<WorkspaceInfo, String> {
    let dir = get_dir(&state)?;
    let store = LocalWorkspace::open_at(dir.clone()).map_err(|e| e.to_string())?;
    let prov = binary_provenance_for(&Workspace::at(dir.clone()), WorkspaceSource::CliFlag)
        .map_err(|e| e.to_string())?;
    let cfg = list_projects().unwrap_or_default();
    let project_name = cfg
        .projects
        .iter()
        .find(|p| p.path == dir.display().to_string())
        .map(|p| p.name.clone())
        .unwrap_or_else(|| {
            dir.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "project".to_string())
        });
    Ok(WorkspaceInfo {
        root: dir.display().to_string(),
        project_name,
        storage_backend: store.backend().describe(store.workspace()),
        binary_match: prov.current_workspace_binary_match,
        executable_path: prov.executable_path,
    })
}

fn issue_task_links(
    store: &crate::self_host::LocalWorkspace,
    issue_key: &str,
) -> Result<Vec<crate::ui::dto::IssueTaskLinkDto>, String> {
    store
        .list_issue_tasks(issue_key)
        .map(|links| {
            links
                .into_iter()
                .map(|l| crate::ui::dto::IssueTaskLinkDto {
                    role: l.role,
                    task_id: l.task_id,
                    proposed_title: l.proposed_title,
                    journey_stage: l.journey_stage,
                    source: l.source,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

fn issue_info_from_row(
    store: &crate::self_host::LocalWorkspace,
    row: storage::IssueRow,
) -> Result<IssueInfo, String> {
    let active = store.active_run_id(&row.key).ok().flatten();
    let run_ids = store.run_ids_for_issue(&row.key);
    let task_links = issue_task_links(store, &row.key)?;
    Ok(IssueInfo {
        key: row.key,
        title: row.title,
        issue_type: row.issue_type,
        priority: row.priority,
        status: row.status,
        product_id: row.product_id,
        pipeline: row.pipeline,
        description: row.description,
        epic_task_id: row.epic_task_id,
        task_links,
        active_run_id: active,
        run_ids,
    })
}

#[tauri::command]
pub fn list_issues(state: State<AppState>) -> Result<Vec<IssueInfo>, String> {
    state.with_store(|store| {
        let rows = store.list_issues().map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(issue_info_from_row(store, row)?);
        }
        Ok(out)
    })
}

#[tauri::command]
pub fn get_issue(key: String, state: State<AppState>) -> Result<IssueInfo, String> {
    state.with_store(|store| {
        let row = store.get_issue(&key).map_err(|e| e.to_string())?;
        issue_info_from_row(store, row)
    })
}

#[tauri::command]
pub fn get_product_health(
    product: String,
    state: State<AppState>,
) -> Result<ProductHealthReport, String> {
    let dir = get_dir(&state)?;
    scan_product_health(&dir, &product).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_create_issue_form_options(
    state: State<AppState>,
) -> Result<CreateIssueFormOptions, String> {
    state.with_store(|store| {
        let root = store.workspace().root.clone();
        let cfg = load_project_config(&root).unwrap_or_default();
        let mut product_options = list_products(&root).map_err(|e| e.to_string())?;
        for issue in store.list_issues().map_err(|e| e.to_string())? {
            if !issue.product_id.is_empty()
                && !product_options.iter().any(|p| p == &issue.product_id)
            {
                product_options.push(issue.product_id);
            }
        }
        product_options.sort();
        product_options.dedup();
        let default_product =
            crate::workspace_readers::resolve_default_product(&root, &cfg.paths.default_product)
                .or_else(|| product_options.first().cloned())
                .unwrap_or_else(|| "cli-ux".into());
        let pipeline_options = list_installed_pipeline_names(store.workspace());
        let profile = cfg.workflow.profile;
        let default_pipeline_by_type = default_pipelines_by_type(profile);
        let task_options = scan_product_tasks(&root, &default_product)
            .map(|g| {
                g.nodes
                    .into_iter()
                    .map(|n| crate::ui::dto::TaskOptionDto {
                        task_id: n.task_id,
                        title: n.title,
                        journey_stage: n.journey_stage,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(CreateIssueFormOptions {
            default_product,
            product_options,
            pipeline_options,
            default_pipeline_by_type,
            workflow_profile: profile.as_str().to_string(),
            task_options,
        })
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_issue(
    issue_type: String,
    title: String,
    product_id: String,
    pipeline: Option<String>,
    priority: Option<String>,
    description: Option<String>,
    epic_task_id: Option<String>,
    linked_task_ids: Option<Vec<String>>,
    proposed_tasks: Option<Vec<(String, Option<String>)>>,
    state: State<AppState>,
) -> Result<IssueInfo, String> {
    state.with_store(|store| {
        let linked_ids = linked_task_ids.unwrap_or_default();
        let linked: Vec<&str> = linked_ids.iter().map(String::as_str).collect();
        let proposed = proposed_tasks.unwrap_or_default();
        let row = store
            .create_issue(
                &issue_type,
                &title,
                &product_id,
                pipeline.as_deref(),
                priority.as_deref().unwrap_or("medium"),
                description.as_deref().unwrap_or(""),
                epic_task_id.as_deref(),
                &linked,
                &proposed,
            )
            .map_err(|e| e.to_string())?;
        issue_info_from_row(store, row)
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
    Ok(render_intent_graph_mermaid(&graph))
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
        let task_links = store
            .list_issue_tasks(&issue.key)
            .map_err(|e| e.to_string())?;
        guidance_for_issue(
            &dir,
            &issue.product_id,
            &issue.issue_type,
            &issue.status,
            pipeline_stage.as_deref(),
            &task_links,
        )
        .map_err(|e| e.to_string())
    })
}

fn config_to_dto(root: &std::path::Path, cfg: &ProjectConfig) -> ProjectConfigDto {
    let product_options = list_products(root).unwrap_or_default();
    let default_product =
        crate::workspace_readers::resolve_default_product(root, &cfg.paths.default_product)
            .unwrap_or_default();
    ProjectConfigDto {
        language: cfg.agent.language.as_str().to_string(),
        products_dir: cfg.paths.products_dir.clone(),
        default_product,
        product_options,
        workflow_profile: cfg.workflow.profile.as_str().to_string(),
        sync_agents_md: cfg.workflow.sync_agents_md,
        inject_on_run: cfg.workflow.inject_on_run,
        approval_mode: cfg.workflow.approval_mode.as_str().to_string(),
        config_path: project_config_path(root).display().to_string(),
    }
}

#[tauri::command]
pub fn get_project_config(state: State<AppState>) -> Result<ProjectConfigDto, String> {
    let dir = get_dir(&state)?;
    let cfg = if project_config_path(&dir).is_file() {
        load_project_config(&dir).map_err(|e| e.to_string())?
    } else {
        ensure_project_config(&dir).map_err(|e| e.to_string())?
    };
    Ok(config_to_dto(&dir, &cfg))
}

#[derive(serde::Deserialize)]
pub struct SaveProjectConfigInput {
    pub language: String,
    pub products_dir: String,
    pub default_product: String,
    pub workflow_profile: String,
    pub sync_agents_md: bool,
    pub inject_on_run: bool,
    pub approval_mode: String,
}

#[tauri::command]
pub fn save_project_config_cmd(
    input: SaveProjectConfigInput,
    state: State<AppState>,
) -> Result<ProjectConfigDto, String> {
    let dir = get_dir(&state)?;
    let products_dir = input.products_dir.trim();
    if products_dir.is_empty() || products_dir.contains("..") {
        return Err("products_dir must be a non-empty relative path".into());
    }
    let default_product = input.default_product.trim();
    if !default_product.is_empty() {
        crate::workspace_readers::resolve_product_id(&dir, default_product)
            .map_err(|e| e.to_string())?;
    }
    let cfg = ProjectConfig {
        version: 1,
        agent: crate::project_config::AgentConfig {
            language: AgentLanguage::parse(&input.language),
        },
        paths: crate::project_config::PathConfig {
            products_dir: products_dir.to_string(),
            default_product: default_product.to_string(),
        },
        workflow: crate::project_config::WorkflowConfig {
            profile: WorkflowProfile::parse(&input.workflow_profile),
            sync_agents_md: input.sync_agents_md,
            inject_on_run: input.inject_on_run,
            approval_mode: crate::project_config::ApprovalMode::parse(&input.approval_mode),
        },
    };
    save_project_config(&dir, &cfg).map_err(|e| e.to_string())?;
    if cfg.workflow.sync_agents_md {
        sync_agents_md(&dir, &cfg).map_err(|e| e.to_string())?;
    }
    Ok(config_to_dto(&dir, &cfg))
}
