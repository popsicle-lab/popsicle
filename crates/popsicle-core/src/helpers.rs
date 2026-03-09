use std::path::Path;

use crate::model::PipelineDef;
use crate::registry::{PipelineLoader, SkillLoader, SkillRegistry};
use crate::storage::ProjectLayout;

/// Load a SkillRegistry from standard project directories.
pub fn load_registry(project_dir: &Path) -> crate::error::Result<SkillRegistry> {
    let mut registry = SkillRegistry::new();

    let workspace_skills = project_dir.join("skills");
    if workspace_skills.is_dir() {
        SkillLoader::load_dir(&workspace_skills, &mut registry)?;
    }

    let local_skills = project_dir.join(".popsicle").join("skills");
    if local_skills.is_dir() {
        SkillLoader::load_dir(&local_skills, &mut registry)?;
    }

    Ok(registry)
}

/// Load all pipeline definitions from standard directories.
pub fn load_pipelines(project_dir: &Path) -> crate::error::Result<Vec<PipelineDef>> {
    let mut all = Vec::new();
    for dir in [
        project_dir.join("pipelines"),
        project_dir.join(".popsicle").join("pipelines"),
    ] {
        if dir.is_dir() {
            all.extend(PipelineLoader::load_dir(&dir)?);
        }
    }
    Ok(all)
}

/// Find a pipeline definition by name.
pub fn find_pipeline(project_dir: &Path, name: &str) -> crate::error::Result<PipelineDef> {
    load_pipelines(project_dir)?
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| {
            crate::error::PopsicleError::Storage(format!("Pipeline template not found: {}", name))
        })
}

/// Convert a title to a URL-friendly slug.
pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

/// Ensure project is initialized and return layout.
pub fn project_layout(project_dir: &Path) -> crate::error::Result<ProjectLayout> {
    let layout = ProjectLayout::new(project_dir);
    layout.ensure_initialized()?;
    Ok(layout)
}
