use std::path::Path;

use crate::engine::PipelineRecommender;
use crate::model::{Issue, PipelineDef};
use crate::registry::{PipelineLoader, SkillLoader, SkillRegistry};
use crate::storage::{ProjectConfig, ProjectLayout};

/// Resolve the active module name from config, falling back to "official".
fn active_module_name(project_dir: &Path) -> String {
    let config_path = project_dir.join(".popsicle").join("config.toml");
    ProjectConfig::load(&config_path)
        .map(|c| c.module.name_or_default().to_string())
        .unwrap_or_else(|_| "official".to_string())
}

/// Load a SkillRegistry from standard project directories.
///
/// Loading order (later overwrites earlier — HashMap insert semantics):
/// 1. Active module skills (lowest priority)
/// 2. Project-local `.popsicle/skills/` (user overrides)
/// 3. Workspace `skills/` (highest priority, development)
pub fn load_registry(project_dir: &Path) -> crate::error::Result<SkillRegistry> {
    let mut registry = SkillRegistry::new();

    let module_name = active_module_name(project_dir);
    let module_skills = project_dir
        .join(".popsicle/modules")
        .join(&module_name)
        .join("skills");
    if module_skills.is_dir() {
        SkillLoader::load_dir(&module_skills, &mut registry)?;
    }

    let local_skills = project_dir.join(".popsicle").join("skills");
    if local_skills.is_dir() {
        SkillLoader::load_dir(&local_skills, &mut registry)?;
    }

    let workspace_skills = project_dir.join("skills");
    if workspace_skills.is_dir() {
        SkillLoader::load_dir(&workspace_skills, &mut registry)?;
    }

    Ok(registry)
}

/// Load all pipeline definitions from standard directories.
///
/// Loading order (later entries with same name overwrite earlier):
/// 1. Active module pipelines (lowest priority)
/// 2. Project-local `.popsicle/pipelines/` (user overrides)
/// 3. Workspace `pipelines/` (highest priority, development)
pub fn load_pipelines(project_dir: &Path) -> crate::error::Result<Vec<PipelineDef>> {
    let module_name = active_module_name(project_dir);

    let dirs = [
        project_dir
            .join(".popsicle/modules")
            .join(&module_name)
            .join("pipelines"),
        project_dir.join(".popsicle").join("pipelines"),
        project_dir.join("pipelines"),
    ];

    let mut all = Vec::new();
    for dir in dirs {
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

/// Resolve the best pipeline for an issue.
///
/// Priority order:
/// 1. Explicit `issue.pipeline` field (set via `--pipeline` at creation) — highest priority.
/// 2. PipelineRecommender keyword match on title + description.
/// 3. `IssueType::default_pipeline()` fallback.
pub fn resolve_pipeline_for_issue(
    issue: &Issue,
    pipelines: &[PipelineDef],
) -> Option<ResolvedPipeline> {
    if let Some(ref name) = issue.pipeline {
        return Some(ResolvedPipeline {
            pipeline_name: name.clone(),
            reason: format!("Explicitly bound pipeline '{}'", name),
            source: PipelineSource::Explicit,
        });
    }

    let task_text = if issue.description.is_empty() {
        issue.title.clone()
    } else {
        format!("{} {}", issue.title, issue.description)
    };

    let rec = PipelineRecommender::recommend(&task_text, pipelines);

    if !rec.reason.contains("No keyword match") {
        return Some(ResolvedPipeline {
            pipeline_name: rec.pipeline_name,
            reason: rec.reason,
            source: PipelineSource::Recommender,
        });
    }

    issue
        .issue_type
        .default_pipeline()
        .map(|name| ResolvedPipeline {
            pipeline_name: name.to_string(),
            reason: format!("Default pipeline for issue type '{}'", issue.issue_type),
            source: PipelineSource::IssueTypeDefault,
        })
}

/// Result of pipeline resolution for an issue.
#[derive(Debug, Clone)]
pub struct ResolvedPipeline {
    pub pipeline_name: String,
    pub reason: String,
    pub source: PipelineSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineSource {
    /// User explicitly specified `--pipeline` at issue creation.
    Explicit,
    Recommender,
    IssueTypeDefault,
}
