use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use popsicle_core::helpers::load_tools;
use popsicle_core::model::ToolDef;
use popsicle_core::registry::ToolRegistry;
use popsicle_core::storage::ProjectLayout;

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum ToolCommand {
    /// List all available tools
    List,
    /// Show details of a tool (arguments, command/prompt, guide)
    Show {
        /// Tool name
        name: String,
    },
    /// Install a tool from a local path or Git repository
    ///
    /// The tool is installed into `.popsicle/tools/<name>/` and is immediately
    /// available via `popsicle tool run <name>`.
    ///
    /// Source formats:
    ///   - Local directory:   path/to/tool-dir
    ///   - GitHub:            github:user/repo[#ref][//subdir]
    Install {
        /// Source: local path or github:user/repo[#ref][//subdir]
        source: String,
        /// Override the tool name (defaults to the name in tool.yaml)
        #[arg(long)]
        name: Option<String>,
    },
    /// Run a tool with the given arguments
    ///
    /// Arguments are passed as KEY=VALUE pairs.
    /// When the tool defines a `command`, it is executed as a shell command.
    /// When the tool defines a `prompt`, the rendered prompt is printed to stdout
    /// for use by an AI agent.
    Run {
        /// Tool name
        name: String,
        /// Arguments as key=value pairs (e.g. input=diagram.md type=sequence)
        #[arg(value_name = "KEY=VALUE")]
        args: Vec<String>,
    },
}

pub fn execute(cmd: ToolCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        ToolCommand::List => list_tools(format),
        ToolCommand::Show { name } => show_tool(&name, format),
        ToolCommand::Install { source, name } => install_tool(&source, name.as_deref(), format),
        ToolCommand::Run { name, args } => run_tool(&name, &args, format),
    }
}

fn project_dir() -> anyhow::Result<PathBuf> {
    env::current_dir().context("Cannot determine current directory")
}

fn load_registry() -> anyhow::Result<ToolRegistry> {
    let project_dir = project_dir()?;
    load_tools(&project_dir).map_err(|e| anyhow::anyhow!("{}", e))
}

// ── list ──────────────────────────────────────────────────────────────────────

fn list_tools(format: &OutputFormat) -> anyhow::Result<()> {
    let registry = load_registry()?;
    let tools = registry.list();

    match format {
        OutputFormat::Text => {
            if tools.is_empty() {
                println!("No tools installed.");
                println!();
                println!(
                    "Install a tool with: popsicle tool install <source>"
                );
                return Ok(());
            }
            println!("{:<20} {:<10} {}", "NAME", "VERSION", "DESCRIPTION");
            println!("{}", "-".repeat(70));
            for t in &tools {
                println!(
                    "{:<20} {:<10} {}",
                    t.name,
                    t.version,
                    t.description
                );
            }
            println!("\n{} tool(s) available.", tools.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "version": t.version,
                        "description": t.description,
                        "source": t.source,
                        "kind": if t.command.is_some() { "command" } else { "prompt" },
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

// ── show ──────────────────────────────────────────────────────────────────────

fn show_tool(name: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let registry = load_registry()?;
    let tool = registry.get(name).map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => print_tool_detail(tool),
        OutputFormat::Json => {
            let args: Vec<_> = tool
                .args
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "name": a.name,
                        "description": a.description,
                        "required": a.required,
                        "default": a.default,
                    })
                })
                .collect();
            let obj = serde_json::json!({
                "name": tool.name,
                "version": tool.version,
                "description": tool.description,
                "source": tool.source,
                "kind": if tool.command.is_some() { "command" } else { "prompt" },
                "command": tool.command,
                "prompt": tool.prompt,
                "args": args,
                "guide": tool.guide,
            });
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
    }
    Ok(())
}

fn print_tool_detail(tool: &ToolDef) {
    println!("Tool: {}", tool.name);
    println!("  Version:     {}", tool.version);
    println!("  Description: {}", tool.description);
    if let Some(src) = &tool.source {
        println!("  Source:      {}", src);
    }
    let kind = if tool.command.is_some() { "command" } else { "prompt" };
    println!("  Kind:        {}", kind);

    if !tool.args.is_empty() {
        println!();
        println!("Arguments:");
        for arg in &tool.args {
            let req = if arg.required { "required" } else { "optional" };
            let default = arg
                .default
                .as_deref()
                .map(|d| format!(" (default: {})", d))
                .unwrap_or_default();
            println!("  {:.<20} {} [{}]{}", arg.name, arg.description, req, default);
        }
    }

    if let Some(cmd) = &tool.command {
        println!();
        println!("Command:");
        println!("  {}", cmd);
    }

    if let Some(prompt) = &tool.prompt {
        println!();
        println!("Prompt template:");
        for line in prompt.lines() {
            println!("  {}", line);
        }
    }

    if let Some(guide) = &tool.guide {
        println!();
        println!("Guide:");
        println!("{}", guide);
    }
}

// ── install ───────────────────────────────────────────────────────────────────

fn install_tool(source: &str, name_override: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let project_dir = project_dir()?;
    let layout = ProjectLayout::new(&project_dir);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // If the source looks like a bare registry name, resolve via index first.
    let actual_source = if popsicle_core::registry::is_registry_name(source) {
        let index = popsicle_core::registry::RegistryIndex::open(None)
            .map_err(|e| anyhow::anyhow!("Failed to open registry: {}", e))?;
        let resolved = index
            .resolve(source)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        if resolved.pkg_type != popsicle_core::registry::PackageType::Tool {
            anyhow::bail!(
                "'{}' is a {} in the registry, not a tool. Use `popsicle module install {}` instead.",
                resolved.name,
                resolved.pkg_type,
                source
            );
        }
        eprintln!(
            "Resolved '{}' → {} v{} ({})",
            source, resolved.name, resolved.version, resolved.source
        );
        resolved.source
    } else {
        source.to_string()
    };

    let resolved = resolve_source(&actual_source)?;
    let source_path = resolved.path();

    // Validate tool.yaml exists
    let tool_yaml = source_path.join("tool.yaml");
    if !tool_yaml.exists() {
        bail!("tool.yaml not found in source: {}", source_path.display());
    }

    let mut tool_def = ToolDef::load(&tool_yaml).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Apply name override if provided
    let install_name = name_override.unwrap_or(&tool_def.name).to_string();
    tool_def.name = install_name.clone();

    // Install into .popsicle/tools/<name>/
    let tools_dir = layout.tools_dir();
    let dest = tools_dir.join(&install_name);
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    copy_dir_recursive(source_path, &dest)?;

    // Record source in tool.yaml if it came from a remote
    if source.starts_with("github:") {
        patch_tool_source(&dest.join("tool.yaml"), source)?;
    }

    match format {
        OutputFormat::Text => {
            println!("Installed tool '{}' (v{}) from {}", tool_def.name, tool_def.version, source);
            println!("  Location: {}", dest.display());
            println!();
            println!("Run with: popsicle tool run {}", install_name);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "installed",
                    "name": install_name,
                    "version": tool_def.version,
                    "source": source,
                    "path": dest.display().to_string(),
                }))?
            );
        }
    }
    Ok(())
}

/// Inject/overwrite the `source:` field in a tool.yaml to record its origin.
fn patch_tool_source(tool_yaml: &Path, source: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(tool_yaml)?;
    let updated = if content.contains("\nsource:") || content.starts_with("source:") {
        // Replace existing source line
        content
            .lines()
            .map(|line| {
                if line.starts_with("source:") {
                    format!("source: \"{}\"", source)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Append source field after name line
        content
            .lines()
            .flat_map(|line| {
                let mut out = vec![line.to_string()];
                if line.starts_with("name:") {
                    out.push(format!("source: \"{}\"", source));
                }
                out
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    std::fs::write(tool_yaml, updated)?;
    Ok(())
}

// ── run ───────────────────────────────────────────────────────────────────────

fn run_tool(name: &str, raw_args: &[String], format: &OutputFormat) -> anyhow::Result<()> {
    let registry = load_registry()?;
    let tool = registry.get(name).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Parse KEY=VALUE args
    let provided = parse_args(raw_args)?;

    // Resolve args (apply defaults, check required)
    let args = tool
        .resolve_args(&provided)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Some(cmd) = tool.render_command(&args) {
        // Execute the rendered shell command
        run_shell_command(&cmd, &tool.source_dir, format)
    } else if let Some(prompt) = tool.render_prompt(&args) {
        // Print the rendered prompt for the AI agent to consume
        match format {
            OutputFormat::Text => println!("{}", prompt),
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "tool": name,
                        "prompt": prompt,
                    }))?
                );
            }
        }
        Ok(())
    } else {
        bail!("Tool '{}' has neither a command nor a prompt defined", name);
    }
}

fn parse_args(raw: &[String]) -> anyhow::Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for item in raw {
        let (key, value) = item
            .split_once('=')
            .with_context(|| format!("Invalid argument '{}': expected KEY=VALUE format", item))?;
        map.insert(key.to_string(), value.to_string());
    }
    Ok(map)
}

fn run_shell_command(cmd: &str, working_dir: &Path, format: &OutputFormat) -> anyhow::Result<()> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(working_dir)
        .output()
        .context("Failed to execute shell command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    match format {
        OutputFormat::Text => {
            if !stdout.is_empty() {
                print!("{}", stdout);
            }
            if !stderr.is_empty() {
                eprint!("{}", stderr);
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "exit_code": output.status.code(),
                    "stdout": stdout.trim(),
                    "stderr": stderr.trim(),
                }))?
            );
        }
    }

    if !output.status.success() {
        bail!("Command exited with status: {}", output.status);
    }
    Ok(())
}

// ── git source helpers (shared pattern from module.rs) ─────────────────────

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
            "Invalid github source '{}'. Expected: github:user/repo[#ref][//subdir]",
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
        .context("Failed to run git. Ensure git is installed and in PATH.")?;

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

enum ResolvedSource {
    Local(PathBuf),
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
