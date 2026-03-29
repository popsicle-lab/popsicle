use std::env;

use popsicle_core::helpers;
use popsicle_core::model::{Issue, IssueStatus, IssueType, PipelineRun, Priority, Topic};
use popsicle_core::storage::{IndexDb, ProjectConfig, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum IssueCommand {
    /// Create a new issue
    Create {
        /// Issue type: product, technical, bug, idea
        #[arg(short = 't', long = "type")]
        issue_type: String,
        /// Short title
        #[arg(long)]
        title: String,
        /// Detailed description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Priority: critical, high, medium, low
        #[arg(short, long, default_value = "medium")]
        priority: String,
        /// Bind a specific pipeline template (skips recommender on start)
        #[arg(long)]
        pipeline: Option<String>,
        /// Labels (can repeat)
        #[arg(short, long)]
        label: Vec<String>,
    },
    /// List issues
    List {
        /// Filter by type
        #[arg(short = 't', long = "type")]
        issue_type: Option<String>,
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
        /// Filter by label
        #[arg(short, long)]
        label: Option<String>,
    },
    /// Show issue details
    Show {
        /// Issue key (e.g. PROJ-1) or ID
        key: String,
    },
    /// Start working on an issue (creates a pipeline run)
    Start {
        /// Issue key (e.g. PROJ-1) or ID
        key: String,
    },
    /// Update an issue
    Update {
        /// Issue key (e.g. PROJ-1) or ID
        key: String,
        /// New status
        #[arg(short, long)]
        status: Option<String>,
        /// New priority
        #[arg(short, long)]
        priority: Option<String>,
        /// Add label
        #[arg(short, long)]
        label: Vec<String>,
        /// New title
        #[arg(long)]
        title: Option<String>,
    },
}

pub fn execute(cmd: IssueCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        IssueCommand::Create {
            issue_type,
            title,
            description,
            priority,
            pipeline,
            label,
        } => create_issue(
            &issue_type,
            &title,
            &description,
            &priority,
            pipeline.as_deref(),
            &label,
            format,
        ),
        IssueCommand::List {
            issue_type,
            status,
            label,
        } => list_issues(
            issue_type.as_deref(),
            status.as_deref(),
            label.as_deref(),
            format,
        ),
        IssueCommand::Show { key } => show_issue(&key, format),
        IssueCommand::Start { key } => start_issue(&key, format),
        IssueCommand::Update {
            key,
            status,
            priority,
            label,
            title,
        } => update_issue(
            &key,
            status.as_deref(),
            priority.as_deref(),
            &label,
            title.as_deref(),
            format,
        ),
    }
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn load_config(layout: &ProjectLayout) -> anyhow::Result<ProjectConfig> {
    ProjectConfig::load(&layout.config_path()).map_err(|e| anyhow::anyhow!("{}", e))
}

fn create_issue(
    type_str: &str,
    title: &str,
    description: &str,
    priority_str: &str,
    pipeline: Option<&str>,
    labels: &[String],
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let issue_type: IssueType = type_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;
    let priority: Priority = priority_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;

    let layout = project_layout()?;
    let config = load_config(&layout)?;
    let db = IndexDb::open(&layout.db_path())?;

    if let Some(name) = pipeline {
        let cwd = env::current_dir()?;
        helpers::find_pipeline(&cwd, name)
            .map_err(|_| anyhow::anyhow!("Pipeline template not found: {}", name))?;
    }

    let prefix = config.project.key_prefix_or_default();
    let seq = db
        .next_issue_seq(prefix)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let key = format!("{}-{}", prefix, seq);

    let mut issue = Issue::new(key.clone(), title, issue_type);
    issue.description = description.to_string();
    issue.priority = priority;
    issue.pipeline = pipeline.map(|s| s.to_string());
    issue.labels = labels.to_vec();

    db.create_issue(&issue)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let effective_pipeline = issue
        .pipeline
        .as_deref()
        .or_else(|| issue.issue_type.default_pipeline());

    match format {
        OutputFormat::Text => {
            println!("Created issue: {}", key);
            println!("  Title: {}", title);
            println!("  Type: {}", issue.issue_type);
            println!("  Priority: {}", issue.priority);
            if let Some(ref p) = issue.pipeline {
                println!("  Pipeline: {} (explicit)", p);
            } else if let Some(p) = effective_pipeline {
                println!("  Pipeline: {} (default for {})", p, issue.issue_type);
            }
            if !labels.is_empty() {
                println!("  Labels: {}", labels.join(", "));
            }
            println!("\nStart with: popsicle issue start {}", key);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "key": key,
                "id": issue.id,
                "title": title,
                "issue_type": issue.issue_type.to_string(),
                "priority": issue.priority.to_string(),
                "status": issue.status.to_string(),
                "pipeline": issue.pipeline,
                "effective_pipeline": effective_pipeline,
                "labels": labels,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn list_issues(
    issue_type: Option<&str>,
    status: Option<&str>,
    label: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let issues = db
        .query_issues(issue_type, status, label)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            if issues.is_empty() {
                println!("No issues found.");
                return Ok(());
            }
            println!(
                "{:<12} {:<12} {:<10} {:<10} TITLE",
                "KEY", "TYPE", "PRIORITY", "STATUS"
            );
            println!("{}", "-".repeat(70));
            for issue in &issues {
                println!(
                    "{:<12} {:<12} {:<10} {:<10} {}",
                    issue.key, issue.issue_type, issue.priority, issue.status, issue.title
                );
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = issues
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "id": i.id,
                        "key": i.key,
                        "title": i.title,
                        "issue_type": i.issue_type.to_string(),
                        "priority": i.priority.to_string(),
                        "status": i.status.to_string(),
                        "pipeline": i.pipeline,
                        "pipeline_run_id": i.pipeline_run_id,
                        "labels": i.labels,
                        "created_at": i.created_at.to_rfc3339(),
                        "updated_at": i.updated_at.to_rfc3339(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

fn show_issue(key: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let issue = db
        .get_issue(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Issue not found: {}", key))?;

    let run_info = if let Some(ref run_id) = issue.pipeline_run_id {
        db.get_pipeline_run(run_id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
    } else {
        None
    };

    match format {
        OutputFormat::Text => {
            println!("{} — {}", issue.key, issue.title);
            println!("  Type:     {}", issue.issue_type);
            println!("  Priority: {}", issue.priority);
            println!("  Status:   {}", issue.status);
            if let Some(ref p) = issue.pipeline {
                println!("  Pipeline: {} (bound)", p);
            }
            if !issue.labels.is_empty() {
                println!("  Labels:   {}", issue.labels.join(", "));
            }
            if !issue.description.is_empty() {
                println!("  Description:\n    {}", issue.description);
            }
            if let Some(run) = &run_info {
                println!("  Run:      {} ({})", run.pipeline_name, run.id);
            }
            println!("  Created:  {}", issue.created_at.format("%Y-%m-%d %H:%M"));
            println!("  Updated:  {}", issue.updated_at.format("%Y-%m-%d %H:%M"));
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": issue.id,
                "key": issue.key,
                "title": issue.title,
                "description": issue.description,
                "issue_type": issue.issue_type.to_string(),
                "priority": issue.priority.to_string(),
                "status": issue.status.to_string(),
                "pipeline": issue.pipeline,
                "pipeline_run_id": issue.pipeline_run_id,
                "labels": issue.labels,
                "pipeline_run": run_info.as_ref().map(|r| serde_json::json!({
                    "id": r.id,
                    "pipeline_name": r.pipeline_name,
                    "title": r.title,
                })),
                "created_at": issue.created_at.to_rfc3339(),
                "updated_at": issue.updated_at.to_rfc3339(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn start_issue(key: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let mut issue = db
        .get_issue(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Issue not found: {}", key))?;

    if issue.pipeline_run_id.is_some() {
        anyhow::bail!(
            "Issue {} already has a pipeline run. Use `popsicle pipeline status` to check progress.",
            key
        );
    }

    let pipelines = helpers::load_pipelines(&cwd).map_err(|e| anyhow::anyhow!("{}", e))?;
    let resolved = helpers::resolve_pipeline_for_issue(&issue, &pipelines).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not determine pipeline for issue type '{}'. Use `popsicle pipeline recommend` to pick one manually.",
            issue.issue_type
        )
    })?;

    let pipeline_def = helpers::find_pipeline(&cwd, &resolved.pipeline_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    pipeline_def
        .validate()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let topic = {
        let name = &issue.title;
        if let Some(t) = db
            .find_topic_by_name(name)
            .map_err(|e| anyhow::anyhow!("{}", e))?
        {
            t
        } else {
            let t = Topic::new(name, "");
            db.create_topic(&t).map_err(|e| anyhow::anyhow!("{}", e))?;
            t
        }
    };
    let run = PipelineRun::new(&pipeline_def, &issue.title, &topic.id);
    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir)?;

    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    issue.pipeline_run_id = Some(run.id.clone());
    issue.status = IssueStatus::InProgress;
    db.update_issue(&issue)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Started issue: {} — {}", issue.key, issue.title);
            println!(
                "  Pipeline: {} ({})",
                resolved.pipeline_name, resolved.reason
            );
            println!("  Run ID:   {}", run.id);
            println!("  Status:   {}", issue.status);
            println!("\nNext step:");
            println!("  $ popsicle pipeline next --run {}", run.id);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "key": issue.key,
                "pipeline": resolved.pipeline_name,
                "pipeline_reason": resolved.reason,
                "run_id": run.id,
                "status": issue.status.to_string(),
                "next_command": format!("popsicle pipeline next --run {}", run.id),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn update_issue(
    key: &str,
    status: Option<&str>,
    priority: Option<&str>,
    labels: &[String],
    title: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let mut issue = db
        .get_issue(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Issue not found: {}", key))?;

    if let Some(s) = status {
        issue.status = s.parse().map_err(|e: String| anyhow::anyhow!("{}", e))?;
    }
    if let Some(p) = priority {
        issue.priority = p.parse().map_err(|e: String| anyhow::anyhow!("{}", e))?;
    }
    if let Some(t) = title {
        issue.title = t.to_string();
    }
    for l in labels {
        if !issue.labels.contains(l) {
            issue.labels.push(l.clone());
        }
    }

    db.update_issue(&issue)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Updated issue: {}", issue.key);
            println!("  Title:    {}", issue.title);
            println!("  Status:   {}", issue.status);
            println!("  Priority: {}", issue.priority);
            if !issue.labels.is_empty() {
                println!("  Labels:   {}", issue.labels.join(", "));
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "key": issue.key,
                "title": issue.title,
                "status": issue.status.to_string(),
                "priority": issue.priority.to_string(),
                "labels": issue.labels,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
