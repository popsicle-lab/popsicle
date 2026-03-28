use std::env;
use std::path::PathBuf;

use anyhow::Context;
use popsicle_core::agent::{AgentInstaller, AgentTarget};
use popsicle_core::registry::SkillRegistry;
use popsicle_core::scaffold;
use popsicle_core::scanner::ProjectScanner;
use popsicle_core::storage::{IndexDb, ProjectConfig, ProjectLayout};

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

    /// Install a module during init (local path or github:user/repo[#ref][//subdir])
    #[arg(short, long)]
    module: Option<String>,

    /// Print bootstrap instructions after init (use with --module)
    #[arg(long)]
    bootstrap: bool,

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
    let first_time = layout
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

    let config_path = layout.config_path();
    if !config_path.exists() {
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
        std::fs::write(&config_path, &default_config)?;
    } else {
        // Re-init: update agent targets in existing config
        let mut config = ProjectConfig::load(&config_path).unwrap_or_default();
        config.agent.targets = targets.iter().map(|t| t.name().to_string()).collect();
        let _ = config.save(&config_path);
    }

    // For re-init (upgrade): upgrade builtins in-place.
    // For fresh init: install builtins (skip existing).
    let mut upgrade_info: Option<scaffold::UpgradeResult> = None;
    let mut installed_count = 0;

    if !args.no_builtins {
        if first_time {
            let installed = scaffold::install_builtins(&project_root)
                .context("Failed to install built-in skills and pipelines")?;
            installed_count = installed.len();
        } else if let Some(result) = scaffold::upgrade_builtins(&project_root)
            .context("Failed to upgrade built-in module")?
        {
            installed_count = result.files_written;
            upgrade_info = Some(result);
        }
    }

    // Install external module if --module is specified
    let mut module_installed = None;
    if let Some(ref module_source) = args.module {
        let info = super::module::install_module_from_source(&layout, module_source)
            .context("Failed to install module")?;
        module_installed = Some(info);
    }

    // Auto-scan project context if not yet generated
    let context_path = layout.project_context_path();
    let context_scanned = if !context_path.exists() {
        let scanner = ProjectScanner::new(&project_root);
        let content = scanner.scan();
        std::fs::write(&context_path, &content)?;
        true
    } else {
        false
    };

    // Load all skills (active module + project-local + workspace) for agent instructions.
    let registry = popsicle_core::helpers::load_registry(&project_root)
        .unwrap_or_else(|_| SkillRegistry::new());
    let skill_list = registry.list();
    let skill_refs: Vec<&popsicle_core::model::SkillDef> = skill_list.into_iter().collect();

    let agent_files = if !args.no_agent_files {
        AgentInstaller::install(&project_root, &targets, &skill_refs)
            .context("Failed to install agent instruction files")?
    } else {
        vec![]
    };

    // If we upgraded, update [module] version in config.toml
    if let Some(ref info) = upgrade_info
        && let Ok(mut config) = ProjectConfig::load(&config_path)
    {
        config.module.version = Some(info.new_version.clone());
        let _ = config.save(&config_path);
    }

    match format {
        OutputFormat::Text => {
            if first_time {
                println!(
                    "Initialized Popsicle project at {}",
                    layout.dot_dir().display()
                );
            } else if let Some(ref info) = upgrade_info {
                let old = info.old_version.as_deref().unwrap_or("unknown");
                if old == info.new_version {
                    println!(
                        "Re-initialized Popsicle project at {} (module already at {})",
                        layout.dot_dir().display(),
                        info.new_version
                    );
                } else {
                    println!(
                        "Upgraded Popsicle project at {} (module {} -> {})",
                        layout.dot_dir().display(),
                        old,
                        info.new_version
                    );
                }
            } else {
                println!(
                    "Re-initialized Popsicle project at {}",
                    layout.dot_dir().display()
                );
            }
            println!(
                "  Agent targets: {}",
                targets
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if installed_count > 0 {
                println!("  Built-in skills & pipelines: {} files", installed_count);
            }
            if let Some(ref mi) = module_installed {
                println!(
                    "  Module installed: {} v{} (from {})",
                    mi.name,
                    mi.version,
                    args.module.as_deref().unwrap_or("?")
                );
            }
            println!("  Skills registered: {}", skill_refs.len());
            if context_scanned {
                println!(
                    "  Project context: {} (auto-generated)",
                    context_path.display()
                );
            }
            if !agent_files.is_empty() {
                println!("  Agent instructions:");
                for f in &agent_files {
                    println!("    {}", f);
                }
            }
            println!("  Config: {}", config_path.display());

            if args.bootstrap && module_installed.is_some() {
                println!();
                println!("  Bootstrap: run the following to set up your project workflow:");
                println!("    popsicle context bootstrap --generate-prompt --format json");
                println!("    # → send the 'prompt' field to your LLM, get a JSON plan back");
                println!("    popsicle context bootstrap --apply '<JSON plan>' --start");
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "ok",
                "first_time": first_time,
                "path": layout.dot_dir().display().to_string(),
                "agent_targets": targets.iter().map(|t| t.name()).collect::<Vec<_>>(),
                "skills_count": skill_refs.len(),
                "builtin_files_count": installed_count,
                "agent_files": agent_files,
                "config": config_path.display().to_string(),
                "project_context_scanned": context_scanned,
                "upgrade": upgrade_info.as_ref().map(|i| serde_json::json!({
                    "old_version": i.old_version,
                    "new_version": i.new_version,
                })),
                "module_installed": module_installed.as_ref().map(|mi| serde_json::json!({
                    "name": mi.name,
                    "version": mi.version,
                    "source": args.module,
                })),
                "bootstrap_hint": if args.bootstrap && module_installed.is_some() {
                    Some("Run `popsicle context bootstrap --generate-prompt` to start bootstrapping")
                } else {
                    None
                },
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
