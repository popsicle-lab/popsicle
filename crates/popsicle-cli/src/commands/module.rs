use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use popsicle_core::model::ModuleDef;
use popsicle_core::registry::{PipelineLoader, SkillLoader, SkillRegistry};
use popsicle_core::scaffold;
use popsicle_core::storage::{ProjectConfig, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum ModuleCommand {
    /// List installed modules (marks the active one)
    List,
    /// Show details of a module (skills, pipelines, metadata)
    Show {
        /// Module name (defaults to active module)
        name: Option<String>,
    },
    /// Install a module from a local path or Git repository (replaces the active module)
    Install {
        /// Source: local path, or github:user/repo[#ref][//subdir]
        source: String,
    },
    /// Upgrade the active module to the latest version
    Upgrade {
        /// Force upgrade even if versions match
        #[arg(long)]
        force: bool,
    },
}

pub fn execute(cmd: ModuleCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        ModuleCommand::List => list_modules(format),
        ModuleCommand::Show { name } => show_module(name, format),
        ModuleCommand::Install { source } => install_module(&source, format),
        ModuleCommand::Upgrade { force } => upgrade_module(force, format),
    }
}

fn project_dir() -> anyhow::Result<PathBuf> {
    env::current_dir().context("Cannot determine current directory")
}

fn list_modules(format: &OutputFormat) -> anyhow::Result<()> {
    let project_dir = project_dir()?;
    let layout = ProjectLayout::new(&project_dir);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let config = ProjectConfig::load(&layout.config_path()).unwrap_or_default();
    let active = config.module.name_or_default().to_string();

    let modules_dir = layout.modules_dir();
    let mut modules = Vec::new();

    if modules_dir.is_dir() {
        for entry in std::fs::read_dir(&modules_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest = path.join("module.yaml");
            if manifest.exists() {
                match ModuleDef::load(&manifest) {
                    Ok(def) => modules.push(def),
                    Err(_) => continue,
                }
            }
        }
    }

    modules.sort_by(|a, b| a.name.cmp(&b.name));

    match format {
        OutputFormat::Text => {
            if modules.is_empty() {
                println!("No modules installed.");
                return Ok(());
            }
            println!("{:<5} {:<20} {:<10} DESCRIPTION", "", "NAME", "VERSION");
            println!("{}", "-".repeat(65));
            for m in &modules {
                let marker = if m.name == active { " *" } else { "" };
                println!(
                    "{:<5} {:<20} {:<10} {}",
                    marker,
                    m.name,
                    m.version,
                    m.description.as_deref().unwrap_or("")
                );
            }
            println!("\n{} module(s) installed. (* = active)", modules.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = modules
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "name": m.name,
                        "version": m.version,
                        "description": m.description,
                        "active": m.name == active,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }

    Ok(())
}

fn show_module(name: Option<String>, format: &OutputFormat) -> anyhow::Result<()> {
    let project_dir = project_dir()?;
    let layout = ProjectLayout::new(&project_dir);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let config = ProjectConfig::load(&layout.config_path()).unwrap_or_default();
    let module_name = name
        .as_deref()
        .unwrap_or_else(|| config.module.name_or_default());

    let module_dir = layout.module_dir(module_name);
    let manifest = module_dir.join("module.yaml");
    if !manifest.exists() {
        bail!("Module '{}' not found", module_name);
    }

    let def = ModuleDef::load(&manifest).map_err(|e| anyhow::anyhow!("{}", e))?;

    let skills_dir = module_dir.join("skills");
    let mut skill_names = Vec::new();
    if skills_dir.is_dir() {
        let mut registry = SkillRegistry::new();
        let _ = SkillLoader::load_dir(&skills_dir, &mut registry);
        skill_names = registry.list().iter().map(|s| s.name.clone()).collect();
        skill_names.sort();
    }

    let pipelines_dir = module_dir.join("pipelines");
    let mut pipeline_names = Vec::new();
    if pipelines_dir.is_dir()
        && let Ok(pipelines) = PipelineLoader::load_dir(&pipelines_dir)
    {
        pipeline_names = pipelines.iter().map(|p| p.name.clone()).collect();
        pipeline_names.sort();
    }

    let is_active = config.module.name_or_default() == module_name;

    match format {
        OutputFormat::Text => {
            println!(
                "Module: {}{}",
                def.name,
                if is_active { " (active)" } else { "" }
            );
            println!("Version: {}", def.version);
            if let Some(desc) = &def.description {
                println!("Description: {}", desc);
            }
            if let Some(author) = &def.author {
                println!("Author: {}", author);
            }
            if let Some(source) = &config.module.source {
                println!("Source: {}", source);
            }
            println!("\nSkills ({}):", skill_names.len());
            for name in &skill_names {
                println!("  - {}", name);
            }
            println!("\nPipelines ({}):", pipeline_names.len());
            for name in &pipeline_names {
                println!("  - {}", name);
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "name": def.name,
                "version": def.version,
                "description": def.description,
                "author": def.author,
                "active": is_active,
                "source": config.module.source,
                "skills": skill_names,
                "pipelines": pipeline_names,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn install_module(source: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let project_dir = project_dir()?;
    let layout = ProjectLayout::new(&project_dir);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let resolved = resolve_source(source)?;
    let source_path = resolved.path();

    let manifest = source_path.join("module.yaml");
    if !manifest.exists() {
        bail!(
            "No module.yaml found in '{}'. A valid module directory must contain module.yaml.",
            source_path.display()
        );
    }

    let def = ModuleDef::load(&manifest).map_err(|e| anyhow::anyhow!("{}", e))?;

    let skills_dir = source_path.join("skills");
    if !skills_dir.is_dir() {
        bail!("Module '{}' has no skills/ directory", def.name);
    }

    let target_dir = layout.module_dir(&def.name);
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir)
            .context("Failed to remove existing module directory")?;
    }
    copy_dir_recursive(source_path, &target_dir)?;

    update_config_module(&layout, &def, source)?;

    match format {
        OutputFormat::Text => {
            println!("Installed module '{}'", def.name);
            println!("  Version: {}", def.version);
            println!("  Source: {}", source);
            println!("  Path: {}", target_dir.display());
            if let Some(desc) = &def.description {
                println!("  Description: {}", desc);
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "name": def.name,
                "version": def.version,
                "source": source,
                "path": target_dir.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn upgrade_module(force: bool, format: &OutputFormat) -> anyhow::Result<()> {
    let project_dir = project_dir()?;
    let layout = ProjectLayout::new(&project_dir);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let config = ProjectConfig::load(&layout.config_path()).unwrap_or_default();
    let source = config.module.source.as_deref().unwrap_or("builtin");

    if source == "builtin" {
        let embedded = scaffold::embedded_module_version();

        if embedded.is_none() {
            bail!(
                "No built-in module embedded in this binary.\n\
                 Install a module with: popsicle module install github:<user>/<repo>"
            );
        }

        let installed = config.module.version.as_deref();

        if !force
            && let (Some(emb), Some(inst)) = (embedded.as_deref(), installed)
            && emb == inst
        {
            match format {
                OutputFormat::Text => {
                    println!(
                        "Module 'official' is already at version {}. Use --force to reinstall.",
                        inst
                    );
                }
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "status": "up_to_date",
                        "version": inst,
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
            }
            return Ok(());
        }

        let result = scaffold::upgrade_builtins(&project_dir)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .expect("embedded module verified above");

        // Update config.toml with new version
        let mut config = ProjectConfig::load(&layout.config_path()).unwrap_or_default();
        config.module.version = Some(result.new_version.clone());
        let _ = config.save(&layout.config_path());

        match format {
            OutputFormat::Text => {
                let old = result.old_version.as_deref().unwrap_or("unknown");
                if old == result.new_version {
                    println!(
                        "Reinstalled module 'official' (version {}), {} files written",
                        result.new_version, result.files_written
                    );
                } else {
                    println!(
                        "Upgraded module 'official' ({} -> {}), {} files written",
                        old, result.new_version, result.files_written
                    );
                }
            }
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "status": "upgraded",
                    "old_version": result.old_version,
                    "new_version": result.new_version,
                    "files_written": result.files_written,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
        }
    } else {
        // Remote source: re-install from the recorded source
        install_module(source, format)?;
    }

    Ok(())
}

/// Parsed remote source: `github:user/repo[#ref][//subdir]`
struct GitSource {
    url: String,
    git_ref: Option<String>,
    subdir: Option<String>,
}

fn parse_git_source(source: &str) -> anyhow::Result<GitSource> {
    let rest = source
        .strip_prefix("github:")
        .context("Expected github: prefix")?;

    let (repo_and_ref, subdir) = match rest.split_once("//") {
        Some((before, sub)) => (before, Some(sub.to_string())),
        None => (rest, None),
    };

    let (repo, git_ref) = match repo_and_ref.split_once('#') {
        Some((r, refspec)) => (r, Some(refspec.to_string())),
        None => (repo_and_ref, None),
    };

    if !repo.contains('/') || repo.split('/').count() != 2 {
        bail!(
            "Invalid github source '{}'. Expected format: github:user/repo",
            source
        );
    }

    Ok(GitSource {
        url: format!("https://github.com/{}.git", repo),
        git_ref,
        subdir,
    })
}

fn clone_git_source(gs: &GitSource) -> anyhow::Result<(tempfile::TempDir, PathBuf)> {
    let tmp = tempfile::tempdir().context("Failed to create temp directory")?;

    let mut cmd = std::process::Command::new("git");
    cmd.args(["clone", "--depth", "1"]);

    if let Some(r) = &gs.git_ref {
        cmd.args(["--branch", r]);
    }

    cmd.arg(&gs.url).arg(tmp.path());

    let output = cmd
        .output()
        .context("Failed to run git. Make sure git is installed and in PATH.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git clone failed:\n{}", stderr.trim());
    }

    let root = match &gs.subdir {
        Some(sub) => {
            let p = tmp.path().join(sub);
            if !p.is_dir() {
                bail!("Subdirectory '{}' not found in cloned repository", sub);
            }
            p
        }
        None => tmp.path().to_path_buf(),
    };

    Ok((tmp, root))
}

fn resolve_source(source: &str) -> anyhow::Result<ResolvedSource> {
    if source.starts_with("github:") {
        let gs = parse_git_source(source)?;
        let (tmp, root) = clone_git_source(&gs)?;
        Ok(ResolvedSource::Git { _tmp: tmp, root })
    } else {
        let path = PathBuf::from(source);
        if !path.is_dir() {
            bail!("Source path '{}' is not a directory", source);
        }
        Ok(ResolvedSource::Local(path))
    }
}

enum ResolvedSource {
    Local(PathBuf),
    /// _tmp keeps the TempDir alive until install completes; root is the path to use.
    Git {
        _tmp: tempfile::TempDir,
        root: PathBuf,
    },
}

impl ResolvedSource {
    fn path(&self) -> &Path {
        match self {
            Self::Local(p) => p,
            Self::Git { root, .. } => root,
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn update_config_module(
    layout: &ProjectLayout,
    def: &ModuleDef,
    source: &str,
) -> anyhow::Result<()> {
    let config_path = layout.config_path();
    let mut config = ProjectConfig::load(&config_path).unwrap_or_default();
    config.module.name = Some(def.name.clone());
    config.module.source = Some(source.to_string());
    config.module.version = Some(def.version.clone());

    config
        .save(&config_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}
