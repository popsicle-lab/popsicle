//! Best-effort background OTLP flush (ADR-001 §5).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use crate::config::load_config;
use crate::export::flush_run_internal;

struct WorkerState {
    workspace_root: PathBuf,
    interval: Duration,
}

static WORKER: OnceLock<Mutex<Option<WorkerState>>> = OnceLock::new();

/// Start at most one background flush thread per workspace when endpoint + interval configured.
pub fn maybe_start_background_flush(workspace_root: &Path) {
    let config = load_config(workspace_root);
    let interval_secs = crate::config::effective_flush_interval(&config);
    if interval_secs == 0 {
        return;
    }

    let root = workspace_root.to_path_buf();
    let interval = Duration::from_secs(interval_secs);

    let lock = WORKER.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
    if guard
        .as_ref()
        .is_some_and(|s| s.workspace_root == root && s.interval == interval)
    {
        return;
    }

    if let Some(state) = guard.take() {
        drop(state);
    }

    let root_clone = root.clone();
    thread::Builder::new()
        .name("popsicle-telemetry-flush".into())
        .spawn(move || background_loop(root_clone, interval))
        .ok();

    *guard = Some(WorkerState {
        workspace_root: root,
        interval,
    });
}

fn background_loop(workspace_root: PathBuf, interval: Duration) {
    loop {
        thread::sleep(interval);
        flush_all_runs(&workspace_root);
    }
}

fn flush_all_runs(workspace_root: &Path) {
    let telemetry_root = workspace_root.join(".popsicle/telemetry");
    let Ok(entries) = std::fs::read_dir(&telemetry_root) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let run_id = entry.file_name().to_string_lossy().into_owned();
        let _ = flush_run_internal(workspace_root, &run_id, false);
    }
}
