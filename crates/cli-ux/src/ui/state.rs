use std::path::PathBuf;
use std::sync::Mutex;

use crate::LocalWorkspace;

pub struct AppState {
    pub project_dir: Mutex<Option<PathBuf>>,
    pub initial_dir: String,
}

impl AppState {
    pub fn with_store<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut LocalWorkspace) -> Result<T, String>,
    {
        let dir = self
            .project_dir
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or_else(|| "no project selected".to_string())?;
        let mut store = LocalWorkspace::open_at(dir).map_err(|e| e.to_string())?;
        f(&mut store)
    }
}
