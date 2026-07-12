//! Load `skill.yaml` and `.pipeline.yaml` from disk (ADR-002 skill load contract).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::skill_load::{SkillLoadResult, StateMachine};

/// Current `SkillLoadResult` / `state_machine` schema generation (ADR-002).
pub const SKILL_LOAD_SCHEMA_VERSION: &str = "1";

/// Errors loading skill or pipeline definitions from the filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io(String),
    Parse(String),
    MissingField(&'static str),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(m) => write!(f, "io: {m}"),
            Self::Parse(m) => write!(f, "parse: {m}"),
            Self::MissingField(fld) => write!(f, "missing field: {fld}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Minimal `skill.yaml` surface needed to produce [`SkillLoadResult`].
#[derive(Debug, Clone, Deserialize)]
struct SkillYaml {
    name: String,
    #[serde(default = "default_pkg_version")]
    version: String,
    workflow: WorkflowYaml,
}

fn default_pkg_version() -> String {
    "0.1.0".into()
}

#[derive(Debug, Clone, Deserialize)]
struct WorkflowYaml {
    initial: String,
    #[serde(default)]
    _states: HashMap<String, serde_yaml::Value>,
}

/// Machine-enforced gate predicate on a pipeline stage (feedback #18/#19/P3).
///
/// Gates are the "machine" axis: evaluated by the engine at `stage complete`
/// *before* the human `requires_approval` axis, and **`approval_mode: auto`
/// cannot bypass them** (P4/H6). Authored in YAML as a list of single-key maps,
/// e.g. `- command_exit_zero: "cargo test"` (see the custom `Deserialize`).
#[derive(Debug, Clone)]
pub enum GateSpec {
    /// Run a shell command in the workspace root; pass iff exit code == 0.
    CommandExitZero(String),
    /// A numeric/string field in a manifest satisfies `op value`.
    Assert(AssertGate),
    /// A summary field equals a count re-derived from an itemized list
    /// (catches hand-edited/fabricated gate numbers).
    ManifestRecomputes(ManifestRecomputesGate),
    /// References in artifacts/intents resolve (reuses goal-trace).
    RefResolvable(RefResolvableGate),
}

// serde_yaml 0.9 encodes externally-tagged enums as YAML `!tags`, but pipeline
// authors want plain single-key maps (`- assert: {...}`). Deserialize via a
// flattened raw struct and require exactly one variant key.
impl<'de> Deserialize<'de> for GateSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            #[serde(default)]
            command_exit_zero: Option<String>,
            #[serde(default)]
            assert: Option<AssertGate>,
            #[serde(default)]
            manifest_recomputes: Option<ManifestRecomputesGate>,
            #[serde(default)]
            ref_resolvable: Option<RefResolvableGate>,
        }
        let raw = Raw::deserialize(deserializer)?;
        match (
            raw.command_exit_zero,
            raw.assert,
            raw.manifest_recomputes,
            raw.ref_resolvable,
        ) {
            (Some(c), None, None, None) => Ok(GateSpec::CommandExitZero(c)),
            (None, Some(a), None, None) => Ok(GateSpec::Assert(a)),
            (None, None, Some(m), None) => Ok(GateSpec::ManifestRecomputes(m)),
            (None, None, None, Some(r)) => Ok(GateSpec::RefResolvable(r)),
            _ => Err(serde::de::Error::custom(
                "gate must have exactly one of: command_exit_zero | assert | manifest_recomputes | ref_resolvable",
            )),
        }
    }
}

/// `assert: {file, field, op, value}` — `file` may glob (newest match wins),
/// `field` is a dotted path into a YAML/JSON manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct AssertGate {
    pub file: String,
    pub field: String,
    pub op: String,
    pub value: serde_yaml::Value,
}

/// `manifest_recomputes: {file, field, equals_count_of, where}` — `field` must
/// equal the number of items in list `equals_count_of` matching `where` (`k=v`).
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestRecomputesGate {
    pub file: String,
    pub field: String,
    pub equals_count_of: String,
    #[serde(default, rename = "where")]
    pub where_clause: Option<String>,
}

/// `ref_resolvable: {product_intents, fields}` — validate cross-references.
#[derive(Debug, Clone, Deserialize)]
pub struct RefResolvableGate {
    #[serde(default)]
    pub product_intents: bool,
    #[serde(default)]
    pub fields: Vec<String>,
}

/// A pipeline stage from `.pipeline.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct PipelineStageDef {
    pub name: String,
    #[serde(default)]
    pub skill: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub requires_approval: bool,
    /// Machine-enforced gates (feedback #18/#19/P3). Empty = no gate.
    #[serde(default)]
    pub gate: Vec<GateSpec>,
}

impl PipelineStageDef {
    /// Skill names for this stage (normalizes `skill` vs `skills`).
    pub fn skill_names(&self) -> Vec<&str> {
        if !self.skills.is_empty() {
            self.skills.iter().map(|s| s.as_str()).collect()
        } else if let Some(ref s) = self.skill {
            vec![s.as_str()]
        } else {
            vec![]
        }
    }
}

/// Pipeline definition loaded from `*.pipeline.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct PipelineDef {
    pub name: String,
    pub description: String,
    pub stages: Vec<PipelineStageDef>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub scale: Option<String>,
}

impl PipelineDef {
    /// Load from a `.pipeline.yaml` file path.
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
        serde_yaml::from_str(&content)
            .map_err(|e| LoadError::Parse(format!("{}: {e}", path.display())))
    }

    /// Topological sanity: every `depends_on` names an existing stage.
    pub fn validate(&self) -> Result<(), LoadError> {
        let names: Vec<&str> = self.stages.iter().map(|s| s.name.as_str()).collect();
        for stage in &self.stages {
            for dep in &stage.depends_on {
                if !names.contains(&dep.as_str()) {
                    return Err(LoadError::Parse(format!(
                        "stage '{}' depends on unknown stage '{dep}'",
                        stage.name
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Scan `dir` for subdirectories containing `skill.yaml`; load each into `out`.
pub fn load_skills_dir(dir: &Path, out: &mut Vec<LoadedSkill>) -> Result<usize, LoadError> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in std::fs::read_dir(dir).map_err(|e| LoadError::Io(e.to_string()))? {
        let entry = entry.map_err(|e| LoadError::Io(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            let skill_file = path.join("skill.yaml");
            if skill_file.is_file() {
                out.push(load_skill(&skill_file)?);
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Scan `dir` for `*.pipeline.yaml` files.
pub fn load_pipelines_dir(dir: &Path) -> Result<Vec<PipelineDef>, LoadError> {
    let mut pipelines = Vec::new();
    if !dir.is_dir() {
        return Ok(pipelines);
    }
    for entry in std::fs::read_dir(dir).map_err(|e| LoadError::Io(e.to_string()))? {
        let entry = entry.map_err(|e| LoadError::Io(e.to_string()))?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".pipeline.yaml"))
        {
            pipelines.push(PipelineDef::load(&path)?);
        }
    }
    pipelines.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(pipelines)
}

/// A skill loaded from disk: stable [`SkillLoadResult`] + source location.
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    pub load_result: SkillLoadResult,
    pub source_dir: PathBuf,
    pub workflow_initial: String,
}

/// Load `skill.yaml` at `path` into ADR-002 [`SkillLoadResult`].
///
/// The returned `state_machine` is always the canonical 4-state machine
/// (pipeline-stage semantics live in [`PipelineSession`], not per-skill YAML).
pub fn load_skill(path: &Path) -> Result<LoadedSkill, LoadError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    let yaml: SkillYaml = serde_yaml::from_str(&content)
        .map_err(|e| LoadError::Parse(format!("{}: {e}", path.display())))?;

    if yaml.name.trim().is_empty() {
        return Err(LoadError::MissingField("name"));
    }
    if yaml.workflow.initial.trim().is_empty() {
        return Err(LoadError::MissingField("workflow.initial"));
    }

    let source_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let load_result = SkillLoadResult {
        name: yaml.name,
        pkg_version: yaml.version,
        schema_version: SKILL_LOAD_SCHEMA_VERSION.to_string(),
        state_machine: StateMachine::canonical(),
    };

    Ok(LoadedSkill {
        workflow_initial: yaml.workflow.initial,
        load_result,
        source_dir,
    })
}
