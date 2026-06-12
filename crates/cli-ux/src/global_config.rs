//! Global multi-project registry at `~/.popsicle/global.json` (or `POPSICLE_HOME`).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use storage::WorkspaceError;

pub const GLOBAL_CONFIG_FILE: &str = "global.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GlobalConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_project: Option<String>,
    #[serde(default)]
    pub projects: Vec<ProjectEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectEntry {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_opened_at: Option<u64>,
}

const MAX_RECENT: usize = 12;

/// How the active workspace root was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSource {
    CliFlag,
    EnvVar,
    GlobalDefault,
    CwdWalk,
    /// `popsicle init` without an explicit project targets cwd.
    InitBootstrap,
    /// UI open or CLI lazy-init bootstrapped a directory without `.popsicle/`.
    LazyBootstrap,
}

impl WorkspaceSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CliFlag => "cli_flag",
            Self::EnvVar => "env_var",
            Self::GlobalDefault => "global_default",
            Self::CwdWalk => "cwd_walk",
            Self::InitBootstrap => "init_bootstrap",
            Self::LazyBootstrap => "lazy_bootstrap",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedWorkspace {
    pub root: PathBuf,
    pub source: WorkspaceSource,
}

pub fn global_home() -> Result<PathBuf, WorkspaceError> {
    if let Ok(home) = std::env::var("POPSICLE_HOME") {
        return Ok(PathBuf::from(home));
    }
    let home =
        std::env::var("HOME").map_err(|_| WorkspaceError::InvalidState("HOME not set".into()))?;
    Ok(PathBuf::from(home).join(".popsicle"))
}

pub fn global_config_path() -> Result<PathBuf, WorkspaceError> {
    Ok(global_home()?.join(GLOBAL_CONFIG_FILE))
}

pub fn load_global_config() -> Result<GlobalConfig, WorkspaceError> {
    let path = global_config_path()?;
    if !path.is_file() {
        return Ok(GlobalConfig::default());
    }
    let content = fs::read_to_string(&path).map_err(io_err)?;
    serde_json::from_str(&content)
        .map_err(|e| WorkspaceError::InvalidState(format!("invalid global config: {e}")))
}

pub fn save_global_config(config: &GlobalConfig) -> Result<(), WorkspaceError> {
    let dir = global_home()?;
    fs::create_dir_all(&dir).map_err(io_err)?;
    let path = dir.join(GLOBAL_CONFIG_FILE);
    let content = serde_json::to_string_pretty(config).map_err(|e| io_err(e.to_string()))?;
    fs::write(&path, content).map_err(io_err)
}

pub fn canonical_project_path(path: &str) -> Result<PathBuf, WorkspaceError> {
    let p = PathBuf::from(path);
    let canon = fs::canonicalize(&p).unwrap_or(p);
    Ok(canon)
}

pub fn validate_workspace_root(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !canon.join(".popsicle").is_dir() {
        return Err(WorkspaceError::InvalidState(format!(
            "{} is not a popsicle workspace (missing .popsicle/)",
            canon.display()
        )));
    }
    Ok(canon)
}

fn try_find_workspace_root_from_cwd() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join(".popsicle").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Resolve workspace root for normal commands (not init bootstrap).
pub fn resolve_workspace_root(
    cli_project: Option<&str>,
) -> Result<ResolvedWorkspace, WorkspaceError> {
    if let Some(p) = cli_project.filter(|s| !s.is_empty()) {
        let root = validate_workspace_root(Path::new(p))?;
        return Ok(ResolvedWorkspace {
            root,
            source: WorkspaceSource::CliFlag,
        });
    }
    if let Ok(p) = std::env::var("POPSICLE_PROJECT") {
        if !p.is_empty() {
            let root = validate_workspace_root(Path::new(&p))?;
            return Ok(ResolvedWorkspace {
                root,
                source: WorkspaceSource::EnvVar,
            });
        }
    }
    // Prefer the workspace enclosing cwd over global default so nested/temp
    // workspaces (smoke tests, `cd other-repo`) are not hijacked by default_project.
    if let Some(root) = try_find_workspace_root_from_cwd() {
        return Ok(ResolvedWorkspace {
            root,
            source: WorkspaceSource::CwdWalk,
        });
    }
    if let Ok(cfg) = load_global_config() {
        if let Some(p) = cfg.default_project {
            if !p.is_empty() {
                if let Ok(root) = validate_workspace_root(Path::new(&p)) {
                    return Ok(ResolvedWorkspace {
                        root,
                        source: WorkspaceSource::GlobalDefault,
                    });
                }
            }
        }
    }
    Err(WorkspaceError::InvalidState(
        "no .popsicle workspace root found".into(),
    ))
}

/// Resolve for `popsicle init`: explicit project or cwd bootstrap.
pub fn resolve_init_root(cli_project: Option<&str>) -> Result<ResolvedWorkspace, WorkspaceError> {
    if let Some(p) = cli_project.filter(|s| !s.is_empty()) {
        let path = PathBuf::from(p);
        let root = fs::canonicalize(&path).unwrap_or(path);
        return Ok(ResolvedWorkspace {
            root,
            source: WorkspaceSource::CliFlag,
        });
    }
    let cwd = std::env::current_dir().map_err(|e| io_err(e.to_string()))?;
    Ok(ResolvedWorkspace {
        root: cwd,
        source: WorkspaceSource::InitBootstrap,
    })
}

pub fn derive_project_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "project".to_string())
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn is_valid_workspace_path(path: &str) -> bool {
    Path::new(path).join(".popsicle").is_dir()
}

/// Whether opening this directory would require runtime workspace bootstrap.
pub fn workspace_needs_bootstrap(path: &str) -> Result<bool, WorkspaceError> {
    let canon = canonicalize_project_path(path);
    Ok(!is_valid_workspace_path(&canon.display().to_string()))
}

pub fn upsert_project_entry(
    cfg: &mut GlobalConfig,
    canon: &Path,
    name: Option<&str>,
    touch_opened: bool,
) -> ProjectEntry {
    let entry_name = name
        .map(str::to_string)
        .unwrap_or_else(|| derive_project_name(canon));
    let path = canon.display().to_string();
    let opened_at = touch_opened.then_some(now_unix_secs());
    if let Some(idx) = cfg
        .projects
        .iter()
        .position(|p| p.path == path || p.name == entry_name)
    {
        let existing = &mut cfg.projects[idx];
        existing.name = entry_name;
        existing.path = path.clone();
        if touch_opened {
            existing.last_opened_at = opened_at;
        }
        return existing.clone();
    }
    let entry = ProjectEntry {
        name: entry_name,
        path,
        last_opened_at: opened_at,
    };
    cfg.projects.push(entry.clone());
    entry
}

fn canonicalize_project_path(path: &str) -> PathBuf {
    let p = Path::new(path);
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

fn ensure_workspace_bootstrapped(canon: &Path) -> Result<bool, WorkspaceError> {
    if is_valid_workspace_path(&canon.display().to_string()) {
        return Ok(false);
    }
    crate::self_host::bootstrap_workspace_at(canon)?;
    Ok(true)
}

pub fn add_project(path: &str, name: Option<&str>) -> Result<ProjectEntry, WorkspaceError> {
    let canon = validate_workspace_root(Path::new(path))?;
    let mut cfg = load_global_config()?;
    let entry = upsert_project_entry(&mut cfg, &canon, name, false);
    cfg.projects.sort_by(|a, b| a.name.cmp(&b.name));
    save_global_config(&cfg)?;
    Ok(entry)
}

/// Register a project, auto-bootstrapping runtime workspace files when needed.
pub fn add_project_or_bootstrap(
    path: &str,
    name: Option<&str>,
) -> Result<ProjectEntry, WorkspaceError> {
    let canon = canonicalize_project_path(path);
    ensure_workspace_bootstrapped(&canon)?;
    add_project(&canon.display().to_string(), name)
}

/// Register, mark as recently opened, and set as the global default workspace.
pub fn open_project(path: &str, name: Option<&str>) -> Result<ProjectEntry, WorkspaceError> {
    let canon = validate_workspace_root(Path::new(path))?;
    let mut cfg = load_global_config()?;
    let entry = upsert_project_entry(&mut cfg, &canon, name, true);
    cfg.default_project = Some(entry.path.clone());
    save_global_config(&cfg)?;
    Ok(entry)
}

/// Open (register + default) a directory, auto-bootstrapping when `.popsicle/` is missing.
pub fn open_project_or_bootstrap(
    path: &str,
    name: Option<&str>,
) -> Result<ProjectEntry, WorkspaceError> {
    let canon = canonicalize_project_path(path);
    ensure_workspace_bootstrapped(&canon)?;
    open_project(&canon.display().to_string(), name)
}

pub fn list_recent_projects(limit: usize) -> Result<Vec<ProjectEntry>, WorkspaceError> {
    let cfg = load_global_config()?;
    let mut recent: Vec<ProjectEntry> = cfg
        .projects
        .into_iter()
        .filter(|p| p.last_opened_at.is_some() && is_valid_workspace_path(&p.path))
        .collect();
    recent.sort_by_key(|b| std::cmp::Reverse(b.last_opened_at));
    recent.truncate(limit.min(MAX_RECENT));
    Ok(recent)
}

/// Best workspace to open on UI startup: explicit CLI path, global default, then MRU.
pub fn resolve_ui_startup_root(
    cli_project: Option<&Path>,
) -> Result<Option<PathBuf>, WorkspaceError> {
    if let Some(p) = cli_project {
        if p.join(".popsicle").is_dir() {
            return Ok(Some(
                fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()),
            ));
        }
    }
    let cfg = load_global_config()?;
    if let Some(p) = cfg.default_project {
        if is_valid_workspace_path(&p) {
            return Ok(Some(PathBuf::from(p)));
        }
    }
    if let Some(entry) = list_recent_projects(1)?.into_iter().next() {
        return Ok(Some(PathBuf::from(entry.path)));
    }
    let cwd = std::env::current_dir().map_err(|e| io_err(e.to_string()))?;
    if cwd.join(".popsicle").is_dir() {
        return Ok(Some(cwd));
    }
    Ok(None)
}

pub fn remove_project(name: &str) -> Result<(), WorkspaceError> {
    let mut cfg = load_global_config()?;
    let removed = cfg.projects.iter().find(|p| p.name == name).cloned();
    let Some(removed) = removed else {
        return Err(WorkspaceError::NotFound(format!("project {name}")));
    };
    cfg.projects.retain(|p| p.name != name);
    if cfg.default_project.as_deref() == Some(removed.path.as_str()) {
        cfg.default_project = None;
    }
    save_global_config(&cfg)
}

pub fn set_default_project(target: &str) -> Result<ProjectEntry, WorkspaceError> {
    let mut cfg = load_global_config()?;
    if let Some(entry) = cfg.projects.iter().find(|p| p.name == target).cloned() {
        cfg.default_project = Some(entry.path.clone());
        save_global_config(&cfg)?;
        return Ok(entry);
    }
    let root = validate_workspace_root(Path::new(target))?;
    let entry = upsert_project_entry(&mut cfg, &root, None, false);
    cfg.projects.sort_by(|a, b| a.name.cmp(&b.name));
    cfg.default_project = Some(entry.path.clone());
    save_global_config(&cfg)?;
    Ok(entry)
}

pub fn list_projects() -> Result<GlobalConfig, WorkspaceError> {
    load_global_config()
}

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}
