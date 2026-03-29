use std::env;

use anyhow::bail;
use chrono::Utc;
use popsicle_core::model::{ModuleDef, ToolDef};
use popsicle_core::registry::{
    PackageType, PackageVersion, PipelineLoader, RegistryIndex, SkillLoader, SkillRegistry,
    ToolLoader, ToolRegistry,
};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum RegistryCommand {
    /// Search the registry for modules and tools
    Search {
        /// Search query
        query: String,
        /// Filter by package type
        #[arg(long, value_enum)]
        r#type: Option<TypeFilter>,
    },
    /// Show details of a registry package
    Info {
        /// Package name (optionally with @version)
        name: String,
    },
    /// Publish the current module or tool to the registry
    ///
    /// Must be run from a directory containing module.yaml or tool.yaml.
    /// The package source must be a public GitHub repository so others can
    /// install it. The command adds a version entry to the registry index
    /// and pushes the change.
    Publish {
        /// Source reference for users to install from
        /// (e.g. github:org/repo#v1.0.0)
        #[arg(long)]
        source: String,
        /// Repository URL for display (defaults to source)
        #[arg(long)]
        repository: Option<String>,
        /// Additional keywords for discovery
        #[arg(long, value_delimiter = ',')]
        keywords: Vec<String>,
    },
    /// Yank a published version (hide from default search)
    Yank {
        /// Package name
        name: String,
        /// Version to yank
        #[arg(long)]
        version: String,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum TypeFilter {
    Module,
    Tool,
}

impl From<TypeFilter> for PackageType {
    fn from(f: TypeFilter) -> Self {
        match f {
            TypeFilter::Module => PackageType::Module,
            TypeFilter::Tool => PackageType::Tool,
        }
    }
}

pub fn execute(cmd: RegistryCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        RegistryCommand::Search { query, r#type } => search(&query, r#type, format),
        RegistryCommand::Info { name } => info(&name, format),
        RegistryCommand::Publish {
            source,
            repository,
            keywords,
        } => publish(&source, repository.as_deref(), &keywords, format),
        RegistryCommand::Yank { name, version } => yank(&name, &version, format),
    }
}

// ── search ────────────────────────────────────────────────────────────────

fn search(
    query: &str,
    type_filter: Option<TypeFilter>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let index = open_index()?;
    let pkg_type = type_filter.map(PackageType::from);
    let results = index
        .search(query, pkg_type)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            if results.is_empty() {
                println!("No packages found for '{}'.", query);
                return Ok(());
            }
            println!("{:<25} {:<10} {:<8} DESCRIPTION", "NAME", "VERSION", "TYPE");
            println!("{}", "-".repeat(80));
            for r in &results {
                println!(
                    "{:<25} {:<10} {:<8} {}",
                    r.name,
                    r.version,
                    r.pkg_type,
                    r.description.as_deref().unwrap_or("")
                );
            }
            println!("\n{} result(s).", results.len());
        }
        OutputFormat::Json => {
            let items: Vec<_> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "version": r.version,
                        "type": r.pkg_type,
                        "description": r.description,
                        "keywords": r.keywords,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

// ── info ──────────────────────────────────────────────────────────────────

fn info(name: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let index = open_index()?;
    let entry = index.get(name).map_err(|e| anyhow::anyhow!("{}", e))?;
    let latest = entry
        .latest()
        .ok_or_else(|| anyhow::anyhow!("No available versions"))?;

    match format {
        OutputFormat::Text => {
            println!("Package: {}", latest.name);
            println!("  Type:        {}", latest.pkg_type);
            println!(
                "  Latest:      {}{}",
                latest.vers,
                if latest.yanked { " (yanked)" } else { "" }
            );
            if let Some(desc) = &latest.description {
                println!("  Description: {}", desc);
            }
            if let Some(author) = &latest.author {
                println!("  Author:      {}", author);
            }
            if let Some(repo) = &latest.repository {
                println!("  Repository:  {}", repo);
            }
            println!("  Source:      {}", latest.source);

            if !latest.skills.is_empty() {
                println!("  Skills:      {}", latest.skills.join(", "));
            }
            if !latest.pipelines.is_empty() {
                println!("  Pipelines:   {}", latest.pipelines.join(", "));
            }
            if !latest.tools.is_empty() {
                println!("  Tools:       {}", latest.tools.join(", "));
            }
            if !latest.keywords.is_empty() {
                println!("  Keywords:    {}", latest.keywords.join(", "));
            }
            if !latest.deps.is_empty() {
                println!("  Dependencies:");
                for dep in &latest.deps {
                    println!(
                        "    - {} ({}) {}",
                        dep.name,
                        dep.kind,
                        dep.req.as_deref().unwrap_or("")
                    );
                }
            }

            println!("\nVersions ({}):", entry.versions.len());
            for v in entry.versions.iter().rev() {
                let yanked = if v.yanked { " (yanked)" } else { "" };
                let date = v
                    .published_at
                    .map(|d| d.format("  %Y-%m-%d").to_string())
                    .unwrap_or_default();
                println!("  {:<12}{}{}", v.vers, date, yanked);
            }
        }
        OutputFormat::Json => {
            let versions: Vec<_> = entry
                .versions
                .iter()
                .map(|v| {
                    serde_json::json!({
                        "version": v.vers,
                        "source": v.source,
                        "yanked": v.yanked,
                        "published_at": v.published_at,
                        "skills": v.skills,
                        "pipelines": v.pipelines,
                        "tools": v.tools,
                    })
                })
                .collect();
            let obj = serde_json::json!({
                "name": latest.name,
                "type": latest.pkg_type,
                "description": latest.description,
                "author": latest.author,
                "repository": latest.repository,
                "keywords": latest.keywords,
                "deps": latest.deps,
                "versions": versions,
            });
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
    }
    Ok(())
}

// ── publish ───────────────────────────────────────────────────────────────

fn publish(
    source: &str,
    repository: Option<&str>,
    keywords: &[String],
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;

    // Detect whether this is a module or a tool
    let module_yaml = cwd.join("module.yaml");
    let tool_yaml = cwd.join("tool.yaml");

    let version = if module_yaml.exists() {
        build_module_version(&cwd, &module_yaml, source, repository, keywords)?
    } else if tool_yaml.exists() {
        build_tool_version(&cwd, &tool_yaml, source, repository, keywords)?
    } else {
        bail!(
            "No module.yaml or tool.yaml found in current directory.\n\
             Run this command from a module or tool directory."
        );
    };

    let index = open_index()?;
    let file_path = index
        .add_version(&version)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let commit_msg = format!(
        "Publish {} {} v{}",
        version.pkg_type, version.name, version.vers
    );
    index
        .commit_and_push(&commit_msg)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!(
                "Published {} '{}' v{} to registry.",
                version.pkg_type, version.name, version.vers
            );
            println!("  Source: {}", version.source);
            println!("  Index:  {}", file_path.display());
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "published",
                    "name": version.name,
                    "version": version.vers,
                    "type": version.pkg_type,
                    "source": version.source,
                }))?
            );
        }
    }
    Ok(())
}

fn build_module_version(
    dir: &std::path::Path,
    manifest: &std::path::Path,
    source: &str,
    repository: Option<&str>,
    keywords: &[String],
) -> anyhow::Result<PackageVersion> {
    let def = ModuleDef::load(manifest).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Collect skills
    let skills_dir = dir.join("skills");
    let mut skill_names = Vec::new();
    if skills_dir.is_dir() {
        let mut registry = SkillRegistry::new();
        let _ = SkillLoader::load_dir(&skills_dir, &mut registry);
        skill_names = registry.list().iter().map(|s| s.name.clone()).collect();
        skill_names.sort();
    }

    // Collect pipelines
    let pipelines_dir = dir.join("pipelines");
    let mut pipeline_names = Vec::new();
    if pipelines_dir.is_dir()
        && let Ok(pipelines) = PipelineLoader::load_dir(&pipelines_dir)
    {
        pipeline_names = pipelines.iter().map(|p| p.name.clone()).collect();
        pipeline_names.sort();
    }

    // Collect tools
    let tools_dir = dir.join("tools");
    let mut tool_names = Vec::new();
    if tools_dir.is_dir() {
        let mut registry = ToolRegistry::new();
        let _ = ToolLoader::load_dir(&tools_dir, &mut registry);
        tool_names = registry.list().iter().map(|t| t.name.clone()).collect();
        tool_names.sort();
    }

    // Build deps from tool_dependencies
    let deps: Vec<_> = def
        .tool_dependencies
        .iter()
        .map(|td| popsicle_core::registry::PackageDep {
            name: td.name.clone(),
            req: None,
            kind: PackageType::Tool,
        })
        .collect();

    Ok(PackageVersion {
        name: def.name,
        vers: def.version,
        pkg_type: PackageType::Module,
        description: def.description,
        author: def.author,
        repository: repository.map(String::from),
        source: source.to_string(),
        skills: skill_names,
        pipelines: pipeline_names,
        tools: tool_names,
        deps,
        keywords: keywords.to_vec(),
        yanked: false,
        published_at: Some(Utc::now()),
    })
}

fn build_tool_version(
    _dir: &std::path::Path,
    manifest: &std::path::Path,
    source: &str,
    repository: Option<&str>,
    keywords: &[String],
) -> anyhow::Result<PackageVersion> {
    let def = ToolDef::load(manifest).map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(PackageVersion {
        name: def.name,
        vers: def.version,
        pkg_type: PackageType::Tool,
        description: Some(def.description),
        author: None,
        repository: repository.map(String::from),
        source: source.to_string(),
        skills: vec![],
        pipelines: vec![],
        tools: vec![],
        deps: vec![],
        keywords: keywords.to_vec(),
        yanked: false,
        published_at: Some(Utc::now()),
    })
}

// ── yank ──────────────────────────────────────────────────────────────────

fn yank(name: &str, version: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let index = open_index()?;
    let entry = index.get(name).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Re-read the file, set yanked=true on the matching version, rewrite
    let rel_path = popsicle_core::registry::index_path(name);
    let file_path = index.path().join(&rel_path);

    let mut versions: Vec<PackageVersion> = entry.versions;
    let found = versions.iter_mut().find(|v| v.vers == version);
    match found {
        Some(v) => v.yanked = true,
        None => bail!("Version '{}' not found for package '{}'", version, name),
    }

    // Rewrite file
    let content: String = versions
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&file_path, format!("{}\n", content))?;

    let msg = format!("Yank {} v{}", name, version);
    index
        .commit_and_push(&msg)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => println!("Yanked {} v{}.", name, version),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "yanked",
                    "name": name,
                    "version": version,
                }))?
            );
        }
    }
    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────

fn open_index() -> anyhow::Result<RegistryIndex> {
    RegistryIndex::open(None).map_err(|e| {
        anyhow::anyhow!(
            "Failed to open registry index: {}\n\
             Make sure git is installed and you have network access.",
            e
        )
    })
}
