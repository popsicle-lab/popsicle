mod commands;
mod watcher;

use std::sync::Mutex;
use watcher::ProjectWatcher;

pub struct AppState {
    pub project_dir: Mutex<Option<String>>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            project_dir: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_project_dir,
            commands::get_project_status,
            commands::list_skills,
            commands::list_pipelines,
            commands::list_pipeline_runs,
            commands::get_pipeline_status,
            commands::list_documents,
            commands::get_document,
            commands::get_next_steps,
            commands::get_prompt,
            commands::verify_pipeline_run,
            commands::get_project_config,
            commands::get_git_status,
            commands::get_commit_links,
            commands::list_discussions,
            commands::get_discussion,
        ])
        .setup(|app| {
            ProjectWatcher::setup(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
