pub mod config;
mod file;
mod index;

pub use config::{ModuleSection, ProjectConfig};
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

    pub fn modules_dir(&self) -> PathBuf {
        self.root.join("modules")
    }

    pub fn module_dir(&self, name: &str) -> PathBuf {
        self.modules_dir().join(name)
    }

    /// The project-local tools directory: `.popsicle/tools/`.
    pub fn tools_dir(&self) -> PathBuf {
        self.root.join("tools")
    }

    /// The tools directory bundled inside a specific module.
    pub fn module_tools_dir(&self, module_name: &str) -> PathBuf {
        self.module_dir(module_name).join("tools")
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
    /// Returns `true` if this is a fresh initialization, `false` if already initialized.
    pub fn initialize(&self) -> Result<bool> {
        let first_time = !self.is_initialized();
        std::fs::create_dir_all(self.artifacts_dir())?;
        std::fs::create_dir_all(self.skills_dir())?;
        Ok(first_time)
    }
}
