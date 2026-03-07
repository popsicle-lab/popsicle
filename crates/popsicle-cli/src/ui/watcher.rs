use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

use super::AppState;

pub struct ProjectWatcher;

impl ProjectWatcher {
    pub fn setup(app: AppHandle) {
        std::thread::spawn(move || {
            let mut current_dir: Option<PathBuf> = None;
            let mut _watcher: Option<RecommendedWatcher> = None;
            let last_emit = std::sync::Arc::new(Mutex::new(Instant::now()));

            loop {
                let dir = app
                    .state::<AppState>()
                    .project_dir
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(PathBuf::from));

                if dir != current_dir {
                    _watcher = None;
                    current_dir = dir.clone();

                    if let Some(ref project_dir) = current_dir {
                        let popsicle_dir = project_dir.join(".popsicle");
                        if popsicle_dir.is_dir() {
                            let app_clone = app.clone();
                            let last_clone = last_emit.clone();

                            let w = notify::recommended_watcher(
                                move |res: std::result::Result<Event, notify::Error>| {
                                    if let Ok(_event) = res {
                                        let mut last = last_clone.lock().unwrap();
                                        if last.elapsed() > Duration::from_millis(500) {
                                            *last = Instant::now();
                                            let _ = app_clone.emit("popsicle://refresh", ());
                                        }
                                    }
                                },
                            );

                            if let Ok(mut w) = w {
                                let _ = w.watch(&popsicle_dir, RecursiveMode::Recursive);
                                _watcher = Some(w);
                            }
                        }
                    }
                }

                std::thread::sleep(Duration::from_secs(2));
            }
        });
    }
}
