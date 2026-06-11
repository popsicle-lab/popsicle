//! Debounced filesystem watcher → `popsicle://refresh` event (legacy parity).

#![allow(dead_code)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

pub struct ProjectWatcher {
    pub last_emit: Mutex<Option<Instant>>,
}

impl ProjectWatcher {
    pub fn notify_refresh(app: &AppHandle) {
        if let Some(state) = app.try_state::<ProjectWatcher>() {
            if let Ok(mut guard) = state.last_emit.lock() {
                let now = Instant::now();
                if guard
                    .as_ref()
                    .is_some_and(|t| now.duration_since(*t) < Duration::from_millis(500))
                {
                    return;
                }
                *guard = Some(now);
            }
        }
        let _ = app.emit("popsicle://refresh", ());
    }
}
