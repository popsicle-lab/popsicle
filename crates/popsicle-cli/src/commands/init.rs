use std::env;
use std::path::PathBuf;

use anyhow::Context;
use popsicle_core::agent::{AgentInstaller, AgentTarget};
use popsicle_core::registry::{SkillLoader, SkillRegistry};
use popsicle_core::scaffold;
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Args)]
pub struct InitArgs {
    /// Project directory (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Target AI agents to install instructions for (comma-separated)
    /// Options: claude, cursor. Default: claude
    #[arg(short, long, value_delimiter = ',', default_value = "claude")]
    agent: Vec<String>,

    /// Skip installing agent instruction files
    #[arg(long)]
    no_agent_files: bool,

    /// Skip installing built-in skills and pipeline templates
    #[arg(long)]
    no_builtins: bool,
}

pub fn execute(args: InitArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let project_root = match args.path {
        Some(p) => p,
        None => env::current_dir().context("Cannot determine current directory")?,
    };

    let layout = ProjectLayout::new(&project_root);
    layout
        .initialize()
        .context("Failed to initialize project")?;

    IndexDb::open(&layout.db_path()).context("Failed to create database")?;

    ensure_gitignore(&project_root)?;
    ensure_cursorignore(&project_root)?;

    let targets: Vec<AgentTarget> = args
        .agent
        .iter()
        .filter_map(|s| {
            AgentTarget::parse(s).or_else(|| {
                eprintln!(
                    "Warning: unknown agent '{}', skipping. Available: claude, cursor",
                    s
                );
                None
            })
        })
        .collect();

    let default_config = format!(
        r#"[project]
# default_pipeline = "full-sdlc"
key_prefix = "PROJ"

[git]
auto_track = true

[agent]
targets = [{}]
"#,
        targets
            .iter()
            .map(|t| format!("\"{}\"", t.name()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let config_path = layout.config_path();
    if !config_path.exists() {
        std::fs::write(&config_path, &default_config)?;
    }

    // Install built-in skills first so agent instructions can reference them
    let builtin_files = if !args.no_builtins {
        scaffold::install_builtins(&project_root)
            .context("Failed to install built-in skills and pipelines")?
    } else {
        vec![]
    };

    // Load all skills (built-in + any pre-existing) to generate agent instructions
    let mut registry = SkillRegistry::new();
    let skills_dir = project_root.join(".popsicle").join("skills");
    if skills_dir.is_dir() {
        let _ = SkillLoader::load_dir(&skills_dir, &mut registry);
    }
    let workspace_skills = project_root.join("skills");
    if workspace_skills.is_dir() {
        let _ = SkillLoader::load_dir(&workspace_skills, &mut registry);
    }
    let skill_list = registry.list();
    let skill_refs: Vec<&popsicle_core::model::SkillDef> = skill_list.into_iter().collect();

    let agent_files = if !args.no_agent_files {
        AgentInstaller::install(&project_root, &targets, &skill_refs)
            .context("Failed to install agent instruction files")?
    } else {
        vec![]
    };

    match format {
        OutputFormat::Text => {
            println!(
                "Initialized Popsicle project at {}",
                layout.dot_dir().display()
            );
            println!(
                "  Agent targets: {}",
                targets
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if !builtin_files.is_empty() {
                println!(
                    "  Built-in skills & pipelines: {} files",
                    builtin_files.len()
                );
            }
            println!("  Skills registered: {}", skill_refs.len());
            if !agent_files.is_empty() {
                println!("  Agent instructions:");
                for f in &agent_files {
                    println!("    {}", f);
                }
            }
            println!("  Config: {}", config_path.display());
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "path": layout.dot_dir().display().to_string(),
                "agent_targets": targets.iter().map(|t| t.name()).collect::<Vec<_>>(),
                "skills_count": skill_refs.len(),
                "builtin_files": builtin_files,
                "agent_files": agent_files,
                "config": config_path.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

/// Append `.popsicle/` to `.gitignore` if not already present.
/// Claude Code respects `.gitignore`, so this prevents it from indexing popsicle internals.
fn ensure_gitignore(project_root: &std::path::Path) -> anyhow::Result<()> {
    let gitignore = project_root.join(".gitignore");
    let entry = ".popsicle/";

    if gitignore.exists() {
        let content = std::fs::read_to_string(&gitignore)?;
        if content.lines().any(|l| l.trim() == entry) {
            return Ok(());
        }
        let mut appended = content;
        if !appended.ends_with('\n') {
            appended.push('\n');
        }
        appended.push_str(&format!("\n# Popsicle project data\n{}\n", entry));
        std::fs::write(&gitignore, appended)?;
    } else {
        std::fs::write(&gitignore, format!("# Popsicle project data\n{}\n", entry))?;
    }

    Ok(())
}

/// Create `.cursorignore` with `.popsicle/` if not already present.
/// Prevents Cursor from indexing popsicle internals into AI context.
fn ensure_cursorignore(project_root: &std::path::Path) -> anyhow::Result<()> {
    let cursorignore = project_root.join(".cursorignore");
    let entry = ".popsicle/";

    if cursorignore.exists() {
        let content = std::fs::read_to_string(&cursorignore)?;
        if content.lines().any(|l| l.trim() == entry) {
            return Ok(());
        }
        let mut appended = content;
        if !appended.ends_with('\n') {
            appended.push('\n');
        }
        appended.push_str(&format!("{}\n", entry));
        std::fs::write(&cursorignore, appended)?;
    } else {
        std::fs::write(
            &cursorignore,
            format!("# Popsicle internals (read by CLI, not by AI)\n{}\n", entry),
        )?;
    }

    Ok(())
}
