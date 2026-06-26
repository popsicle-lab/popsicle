//! Debounced filesystem watcher → `popsicle://refresh` (ADR-015 O-501).

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

const DEBOUNCE: Duration = Duration::from_millis(500);

struct WatchThread {
    stop: mpsc::Sender<()>,
    join: JoinHandle<()>,
}

pub struct ProjectWatcher {
    pub last_emit: Mutex<Option<Instant>>,
    watch: Mutex<Option<WatchThread>>,
}

impl ProjectWatcher {
    pub fn new() -> Self {
        Self {
            last_emit: Mutex::new(None),
            watch: Mutex::new(None),
        }
    }

    pub fn notify_refresh(app: &AppHandle) {
        if let Some(state) = app.try_state::<ProjectWatcher>() {
            if let Ok(mut guard) = state.last_emit.lock() {
                let now = Instant::now();
                if guard
                    .as_ref()
                    .is_some_and(|t| now.duration_since(*t) < DEBOUNCE)
                {
                    return;
                }
                *guard = Some(now);
            }
        }
        let _ = app.emit("popsicle://refresh", ());
    }

    pub fn restart(app: &AppHandle, project_root: Option<&Path>) {
        if let Some(state) = app.try_state::<ProjectWatcher>() {
            Self::stop(&state);
        }
        let Some(root) = project_root else {
            return;
        };
        let popsicle = root.join(".popsicle");
        if !popsicle.is_dir() {
            return;
        }

        let (stop_tx, stop_rx) = mpsc::channel();
        let app_handle = app.clone();
        let join = thread::spawn(move || watch_loop(app_handle, popsicle, stop_rx));

        if let Some(state) = app.try_state::<ProjectWatcher>() {
            if let Ok(mut guard) = state.watch.lock() {
                *guard = Some(WatchThread {
                    stop: stop_tx,
                    join,
                });
            }
        }
    }

    fn stop(state: &ProjectWatcher) {
        let thread = state.watch.lock().ok().and_then(|mut g| g.take());
        if let Some(t) = thread {
            let _ = t.stop.send(());
            let _ = t.join.join();
        }
    }
}

fn watch_loop(app: AppHandle, popsicle: PathBuf, stop_rx: mpsc::Receiver<()>) {
    let (event_tx, event_rx) = mpsc::channel();
    let mut watcher = match RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                if is_relevant_event(&event) {
                    let _ = event_tx.send(());
                }
            }
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(_) => return,
    };

    if watcher
        .watch(&popsicle, RecursiveMode::NonRecursive)
        .is_err()
    {
        return;
    }
    let runs = popsicle.join("runs");
    if runs.is_dir() {
        let _ = watcher.watch(&runs, RecursiveMode::Recursive);
    }

    let mut pending = false;
    let mut last_signal = Instant::now();

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }
        match event_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(()) => {
                pending = true;
                last_signal = Instant::now();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if pending && last_signal.elapsed() >= DEBOUNCE {
                    ProjectWatcher::notify_refresh(&app);
                    pending = false;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn is_relevant_event(event: &Event) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
        _ => return false,
    }
    event.paths.iter().any(|p| is_relevant_path(p.as_path()))
}

fn is_relevant_path(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if file_name.starts_with("state.db") {
        return true;
    }
    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        return path.components().any(|c| c.as_os_str() == "runs");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn relevant_paths_include_state_db_and_run_json() {
        assert!(is_relevant_path(Path::new("/proj/.popsicle/state.db-wal")));
        assert!(is_relevant_path(Path::new(
            "/proj/.popsicle/runs/run_abc/session.json"
        )));
        assert!(!is_relevant_path(Path::new("/proj/.popsicle/project.yaml")));
    }
}
