pub mod config;
mod file;
mod index;

pub use config::ProjectConfig;
pub use file::FileStorage;
pub use index::{DocumentRow, IndexDb, PipelineRunRow};

use std::path::{Path, PathBuf};

use crate::error::{PopsicleError, Result};

/// The `.popsicle/` project data directory layout.
pub struct ProjectLayout {
    root: PathBuf,
}

impl ProjectLayout {
    pub fn new(project_root: &Path) -> Self {
        Self {
            root: project_root.join(".popsicle"),
        }
    }

    pub fn dot_dir(&self) -> &Path {
        &self.root
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        self.root.join("artifacts")
    }

    pub fn skills_dir(&self) -> PathBuf {
        self.root.join("skills")
    }

    pub fn db_path(&self) -> PathBuf {
        self.root.join("popsicle.db")
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn project_context_path(&self) -> PathBuf {
        self.root.join("project-context.md")
    }

    pub fn memories_path(&self) -> PathBuf {
        self.root.join("memories.md")
    }

    /// The artifacts directory for a specific pipeline run.
    pub fn run_dir(&self, run_slug: &str) -> PathBuf {
        self.artifacts_dir().join(run_slug)
    }

    /// Check if the project is initialized.
    pub fn is_initialized(&self) -> bool {
        self.root.is_dir()
    }

    pub fn ensure_initialized(&self) -> Result<()> {
        if !self.is_initialized() {
            return Err(PopsicleError::NotInitialized);
        }
        Ok(())
    }

    /// Initialize the project directory structure.
    pub fn initialize(&self) -> Result<()> {
        if self.is_initialized() {
            return Err(PopsicleError::AlreadyInitialized(
                self.root.display().to_string(),
            ));
        }
        std::fs::create_dir_all(self.artifacts_dir())?;
        std::fs::create_dir_all(self.skills_dir())?;
        Ok(())
    }
}
