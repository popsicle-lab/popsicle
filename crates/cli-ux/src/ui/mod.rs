mod commands;
mod dto;
mod state;
mod watcher;

use std::sync::Mutex;

use tauri::Manager;

pub use dto::*;

use state::AppState;

pub fn run(project: Option<String>) {
    let initial_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let initial_project = project.map(std::path::PathBuf::from);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            project_dir: Mutex::new(initial_project.filter(|p| p.join(".popsicle").is_dir())),
            initial_dir,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_dir,
            commands::set_project_dir,
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
        ])
        .setup(|app| {
            app.manage(watcher::ProjectWatcher {
                last_emit: Mutex::new(None),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
