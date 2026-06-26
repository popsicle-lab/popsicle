mod commands;
mod dto;
mod state;
mod watcher;

use std::sync::Mutex;

use tauri::Manager;

pub use dto::*;

use crate::global_config::resolve_ui_startup_root;

use state::AppState;

pub fn run(project: Option<String>) {
    let initial_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let cli_project = project.map(std::path::PathBuf::from);
    let initial_project = resolve_ui_startup_root(cli_project.as_deref())
        .ok()
        .flatten();
    let startup_project = initial_project.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            project_dir: Mutex::new(initial_project),
            initial_dir,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_dir,
            commands::set_project_dir,
            commands::open_project_cmd,
            commands::workspace_needs_bootstrap_cmd,
            commands::list_registered_projects,
            commands::remove_registered_project,
            commands::pick_project_directory,
            commands::resolve_startup_project,
            commands::get_active_project,
            commands::get_workspace_info,
            commands::list_issues,
            commands::get_issue,
            commands::create_issue,
            commands::start_issue,
            commands::list_docs_for_run,
            commands::read_doc,
            commands::get_pipeline_status,
            commands::complete_stage,
            commands::scan_task_graph,
            commands::task_graph_mermaid_cmd,
            commands::list_product_names,
            commands::scan_intent_graph,
            commands::intent_graph_mermaid,
            commands::scan_product_task_graph,
            commands::read_task_content,
            commands::read_intent_file_cmd,
            commands::resolve_intent_ref_cmd,
            commands::get_issue_guidance,
            commands::get_create_issue_form_options,
            commands::get_product_health,
            commands::get_project_config,
            commands::save_project_config_cmd,
            commands::get_project_context_md,
            commands::save_project_context_md,
            commands::get_workflow_catalog,
            commands::get_telemetry_run_detail,
        ])
        .setup(move |app| {
            app.manage(watcher::ProjectWatcher::new());
            if let Some(root) = startup_project.as_ref() {
                watcher::ProjectWatcher::restart(app.handle(), Some(root));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
