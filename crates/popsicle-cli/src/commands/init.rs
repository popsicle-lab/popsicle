use std::env;
use std::path::PathBuf;

use anyhow::Context;
use popsicle_core::agent::AgentInstaller;
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Args)]
pub struct InitArgs {
    /// Project directory (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Skip installing agent instruction files (AGENTS.md, CLAUDE.md, .cursor/rules/)
    #[arg(long)]
    no_agent_files: bool,
}

pub fn execute(args: InitArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let project_root = args
        .path
        .unwrap_or_else(|| env::current_dir().expect("Cannot determine current directory"));

    let layout = ProjectLayout::new(&project_root);
    layout
        .initialize()
        .context("Failed to initialize project")?;

    IndexDb::open(&layout.db_path()).context("Failed to create database")?;

    let default_config = format!(
        r#"[project]
# default_pipeline = "full-sdlc"

[git]
auto_track = true

[agent]
install_instructions = true
"#
    );
    let config_path = layout.config_path();
    if !config_path.exists() {
        std::fs::write(&config_path, default_config)?;
    }

    let agent_files = if !args.no_agent_files {
        AgentInstaller::install(&project_root)
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
                "agent_files": agent_files,
                "config": config_path.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}
