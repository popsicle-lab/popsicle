mod commands;
mod watcher;

use std::sync::Mutex;
use watcher::ProjectWatcher;

pub struct AppState {
    pub project_dir: Mutex<Option<String>>,
    pub initial_dir: String,
}

pub fn run() {
    let initial_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            project_dir: Mutex::new(None),
            initial_dir,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_dir,
            commands::set_project_dir,
            commands::get_project_status,
            commands::list_skills,
            commands::list_pipelines,
            commands::list_pipeline_runs,
            commands::get_pipeline_status,
            commands::list_documents,
            commands::get_document,
            commands::search_documents,
            commands::get_next_steps,
            commands::get_prompt,
            commands::verify_pipeline_run,
            commands::get_project_config,
            commands::get_git_status,
            commands::get_commit_links,
            commands::list_issues,
            commands::get_issue,
            commands::create_issue,
            commands::start_issue,
            commands::update_issue,
            commands::get_issue_progress,
            commands::get_activity,
            commands::find_issue_by_run,
            commands::get_project_context,
            commands::list_work_items,
            commands::get_work_item,
            commands::create_work_item,
            commands::update_work_item,
            commands::list_memories,
            commands::get_memory_stats,
            commands::get_memory,
            commands::list_specs,
            commands::get_spec,
            commands::list_namespace_entities,
            commands::get_namespace_entity,
            commands::complete_stage,
        ])
        .setup(|app| {
            ProjectWatcher::setup(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
