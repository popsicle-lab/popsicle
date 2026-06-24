//! Read-only catalog of bundled/installed pipelines and intent-coder skills (UI help center).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use skill_runtime::PipelineDef;
use storage::WorkspaceError;

use crate::intent_coder_resolve::intent_coder_skills_dir;
use crate::pipeline_taxonomy::pipeline_domain;
use crate::project_config::{default_pipelines_by_type, load_project_config, WorkflowProfile};
use crate::workspace::{list_installed_pipeline_names, load_pipeline_def, Workspace};

const STANDALONE_SKILLS: &[&str] = &["issue-author"];

#[derive(Debug, Clone, Serialize)]
pub struct StageCatalogEntry {
    pub name: String,
    pub skills: Vec<String>,
    pub description: String,
    pub depends_on: Vec<String>,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineCatalogEntry {
    pub name: String,
    pub description: String,
    pub scale: String,
    pub keywords: Vec<String>,
    pub category: String,
    pub stage_count: usize,
    pub approval_count: usize,
    pub stages: Vec<StageCatalogEntry>,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillArtifactEntry {
    pub artifact_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillStateEntry {
    pub name: String,
    pub requires_approval: bool,
    pub is_initial: bool,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillCatalogEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub artifacts: Vec<SkillArtifactEntry>,
    pub workflow_states: Vec<SkillStateEntry>,
    pub used_in_pipelines: Vec<String>,
    pub standalone: bool,
    pub guide_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowCatalog {
    pub workflow_profile: String,
    pub workflow_profile_label: String,
    pub default_pipeline_by_type: BTreeMap<String, String>,
    pub pipelines: Vec<PipelineCatalogEntry>,
    pub skills: Vec<SkillCatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct SkillCatalogYaml {
    name: String,
    #[serde(default = "default_version")]
    version: String,
    description: String,
    #[serde(default)]
    artifacts: Vec<SkillArtifactYaml>,
    workflow: SkillWorkflowYaml,
}

#[derive(Debug, Deserialize)]
struct SkillArtifactYaml {
    #[serde(rename = "type")]
    artifact_type: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct SkillWorkflowYaml {
    initial: String,
    #[serde(default)]
    states: HashMap<String, SkillStateYaml>,
}

#[derive(Debug, Deserialize, Default)]
struct SkillStateYaml {
    #[serde(default)]
    requires_approval: bool,
    #[serde(default)]
    final_state: bool,
}

fn default_version() -> String {
    "0.1.0".into()
}

fn pipeline_category(name: &str, _keywords: &[String]) -> &'static str {
    pipeline_domain(name)
}

fn skills_dir(workspace: &Workspace) -> PathBuf {
    intent_coder_skills_dir(&workspace.root)
}

fn load_skill_entries(skills_dir: &Path) -> Result<Vec<SkillCatalogEntry>, WorkspaceError> {
    if !skills_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(skills_dir)
        .map_err(|e| WorkspaceError::Io(format!("read {}: {e}", skills_dir.display())))?
    {
        let entry = entry.map_err(|e| WorkspaceError::Io(e.to_string()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_file = path.join("skill.yaml");
        if !skill_file.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&skill_file)
            .map_err(|e| WorkspaceError::Io(format!("read {}: {e}", skill_file.display())))?;
        let yaml: SkillCatalogYaml = serde_yaml::from_str(&content)
            .map_err(|e| WorkspaceError::Io(format!("parse {}: {e}", skill_file.display())))?;
        let mut workflow_states: Vec<SkillStateEntry> = yaml
            .workflow
            .states
            .iter()
            .map(|(name, st)| SkillStateEntry {
                name: name.clone(),
                requires_approval: st.requires_approval,
                is_initial: name == &yaml.workflow.initial,
                is_final: st.final_state,
            })
            .collect();
        workflow_states.sort_by(|a, b| a.name.cmp(&b.name));
        let guide = path.join("guide.md");
        let guide_path = if guide.is_file() {
            guide.display().to_string()
        } else {
            String::new()
        };
        let skill_name = yaml.name.clone();
        out.push(SkillCatalogEntry {
            name: yaml.name,
            version: yaml.version,
            description: yaml.description,
            artifacts: yaml
                .artifacts
                .into_iter()
                .map(|a| SkillArtifactEntry {
                    artifact_type: a.artifact_type,
                    description: a.description,
                })
                .collect(),
            workflow_states,
            used_in_pipelines: Vec::new(),
            standalone: STANDALONE_SKILLS.contains(&skill_name.as_str()),
            guide_path,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn pipeline_to_entry(
    def: &PipelineDef,
    recommended_names: &BTreeSet<String>,
) -> PipelineCatalogEntry {
    let approval_count = def.stages.iter().filter(|s| s.requires_approval).count();
    PipelineCatalogEntry {
        name: def.name.clone(),
        description: def.description.clone(),
        scale: def.scale.clone().unwrap_or_else(|| "full".into()),
        keywords: def.keywords.clone(),
        category: pipeline_category(&def.name, &def.keywords).to_string(),
        stage_count: def.stages.len(),
        approval_count,
        stages: def
            .stages
            .iter()
            .map(|s| StageCatalogEntry {
                name: s.name.clone(),
                skills: s.skill_names().into_iter().map(str::to_string).collect(),
                description: s.description.clone(),
                depends_on: s.depends_on.clone(),
                requires_approval: s.requires_approval,
            })
            .collect(),
        recommended: recommended_names.contains(&def.name),
    }
}

fn recommended_pipeline_names(profile: WorkflowProfile) -> BTreeSet<String> {
    default_pipelines_by_type(profile)
        .into_values()
        .chain(std::iter::once("doc-sync-weekly".into()))
        .collect()
}

pub fn build_workflow_catalog(workspace: &Workspace) -> Result<WorkflowCatalog, WorkspaceError> {
    let cfg = load_project_config(&workspace.root).unwrap_or_default();
    let profile = cfg.workflow.profile;
    let recommended = recommended_pipeline_names(profile);
    let default_pipeline_by_type = default_pipelines_by_type(profile);

    let mut pipelines = Vec::new();
    for name in list_installed_pipeline_names(workspace) {
        let def = load_pipeline_def(workspace, &name)?;
        pipelines.push(pipeline_to_entry(&def, &recommended));
    }

    let mut skills = load_skill_entries(&skills_dir(workspace))?;
    let mut skill_to_pipelines: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for p in &pipelines {
        for stage in &p.stages {
            for skill in &stage.skills {
                skill_to_pipelines
                    .entry(skill.clone())
                    .or_default()
                    .insert(p.name.clone());
            }
        }
    }
    for skill in &mut skills {
        if let Some(names) = skill_to_pipelines.get(&skill.name) {
            skill.used_in_pipelines = names.iter().cloned().collect();
        }
    }

    Ok(WorkflowCatalog {
        workflow_profile: profile.as_str().to_string(),
        workflow_profile_label: profile.label(cfg.agent.language).to_string(),
        default_pipeline_by_type,
        pipelines,
        skills,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_ws() -> Workspace {
        let root = std::env::temp_dir().join(format!(
            "popsicle-wf-catalog-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join(".popsicle/pipelines")).unwrap();
        fs::create_dir_all(root.join(".popsicle")).unwrap();
        Workspace::at(root)
    }

    #[test]
    fn catalog_lists_bundled_pipelines_and_skills() {
        let ws = temp_ws();
        ws.install_bundled_pipelines().unwrap();
        crate::install_intent_coder_module(&ws, false).unwrap();
        let cat = build_workflow_catalog(&ws).unwrap();
        assert!(cat.pipelines.iter().any(|p| p.name == "feature-delivery"));
        assert!(cat.skills.iter().any(|s| s.name == "shadow-implementer"));
    }
}
