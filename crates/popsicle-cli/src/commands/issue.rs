use std::env;

use popsicle_core::helpers;
use popsicle_core::model::{Issue, IssueStatus, IssueType, PipelineRun, Priority};
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
        /// Explicitly specify topic by name or ID (if omitted, matches by tags)
        #[arg(long)]
        topic: Option<String>,
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
        /// Filter by topic name
        #[arg(long)]
        topic: Option<String>,
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
            topic,
            description,
            priority,
            pipeline,
            label,
        } => create_issue(
            &issue_type,
            &title,
            topic.as_deref(),
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
            topic,
        } => list_issues(
            issue_type.as_deref(),
            status.as_deref(),
            label.as_deref(),
            topic.as_deref(),
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

/// Verify the project is properly set up before issue operations.
/// Checks two gates: (1) namespace must exist, (2) topics must exist (bootstrapped).
fn check_namespace_ready(db: &IndexDb) -> anyhow::Result<()> {
    let namespaces = db.list_namespaces(None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if namespaces.is_empty() {
        anyhow::bail!(
            "No namespace found. Create one first:\n  \
             $ popsicle namespace create --name \"<name>\" --description \"<desc>\"\n\n\
             Then bootstrap the project to create topics and import documents:\n  \
             $ popsicle context bootstrap --generate-prompt"
        );
    }
    let topics = db.list_topics()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if topics.is_empty() {
        anyhow::bail!(
            "Namespace exists but has no topics — not yet bootstrapped.\n  \
             $ popsicle context bootstrap --generate-prompt\n\
             Then apply the plan to create topics and import documents."
        );
    }
    Ok(())
}

fn create_issue(
    type_str: &str,
    title: &str,
    topic_name: Option<&str>,
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
    check_namespace_ready(&db)?;

    if let Some(name) = pipeline {
        let cwd = env::current_dir()?;
        helpers::find_pipeline(&cwd, name)
            .map_err(|_| anyhow::anyhow!("Pipeline template not found: {}", name))?;
    }

    // Resolve topic: explicit name or tag-based matching
    let topic = if let Some(name) = topic_name {
        db.find_topic_by_name(name)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Topic not found: {}. Create it first with 'popsicle topic create'.", name))?
    } else {
        let keywords: Vec<String> = title
            .split_whitespace()
            .filter(|w| w.len() >= 2)
            .map(|w| w.to_string())
            .collect();
        let matches = db
            .match_topics_by_tags(&keywords)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some((matched_topic, score)) = matches.into_iter().next() {
            eprintln!(
                "  Matched topic: {} (score: {}, tags: {})",
                matched_topic.name,
                score,
                matched_topic.tags.join(", ")
            );
            matched_topic
        } else {
            anyhow::bail!(
                "No matching topic found. Use --topic to specify one, or create a topic first with 'popsicle topic create'."
            );
        }
    };

    let prefix = config.project.key_prefix_or_default();
    let seq = db
        .next_issue_seq(prefix)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let key = format!("{}-{}", prefix, seq);

    let mut issue = Issue::new(key.clone(), title, issue_type, &topic.id);
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
            println!("  Title:   {}", title);
            println!("  Type:    {}", issue.issue_type);
            println!("  Priority:{}", issue.priority);
            println!("  Topic:   {}", topic.name);
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
                "topic_id": issue.topic_id,
                "topic_name": topic.name,
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
    topic: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    // Resolve topic name to ID if provided
    let topic_id = if let Some(name) = topic {
        let t = db
            .find_topic_by_name(name)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Topic not found: {}", name))?;
        Some(t.id)
    } else {
        None
    };

    let issues = db
        .query_issues(issue_type, status, label, topic_id.as_deref())
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
                        "topic_id": i.topic_id,
                        "pipeline": i.pipeline,
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

    // Fetch all pipeline runs for this issue
    let runs = db
        .find_runs_by_issue(&issue.id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Fetch topic name
    let topic_name = db
        .get_topic(&issue.topic_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .map(|t| t.name)
        .unwrap_or_else(|| issue.topic_id.clone());

    match format {
        OutputFormat::Text => {
            println!("{} — {}", issue.key, issue.title);
            println!("  Type:     {}", issue.issue_type);
            println!("  Priority: {}", issue.priority);
            println!("  Status:   {}", issue.status);
            println!("  Topic:    {}", topic_name);
            if let Some(ref p) = issue.pipeline {
                println!("  Pipeline: {} (bound)", p);
            }
            if !issue.labels.is_empty() {
                println!("  Labels:   {}", issue.labels.join(", "));
            }
            if !issue.description.is_empty() {
                println!("  Description:\n    {}", issue.description);
            }
            if !runs.is_empty() {
                println!("  Runs:");
                for r in &runs {
                    println!("    {} — {} ({})", r.id, r.pipeline_name, r.run_type);
                }
            }
            println!("  Created:  {}", issue.created_at.format("%Y-%m-%d %H:%M"));
            println!("  Updated:  {}", issue.updated_at.format("%Y-%m-%d %H:%M"));
        }
        OutputFormat::Json => {
            let run_list: Vec<_> = runs
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "pipeline_name": r.pipeline_name,
                        "title": r.title,
                        "run_type": r.run_type,
                    })
                })
                .collect();
            let result = serde_json::json!({
                "id": issue.id,
                "key": issue.key,
                "title": issue.title,
                "description": issue.description,
                "issue_type": issue.issue_type.to_string(),
                "priority": issue.priority.to_string(),
                "status": issue.status.to_string(),
                "topic_id": issue.topic_id,
                "topic_name": topic_name,
                "pipeline": issue.pipeline,
                "pipeline_runs": run_list,
                "labels": issue.labels,
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
    check_namespace_ready(&db)?;

    let mut issue = db
        .get_issue(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Issue not found: {}", key))?;

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

    // Issue already has a topic_id; ensure the topic exists
    let topic = db
        .get_topic(&issue.topic_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Topic not found for issue: {}", issue.topic_id))?;

    // Acquire exclusive lock on the topic
    if let Some(ref existing_lock) = topic.locked_by_run_id {
        anyhow::bail!(
            "Topic '{}' is locked by pipeline run '{}'. Release it first with:\n  popsicle pipeline unlock --topic {}",
            topic.name,
            existing_lock,
            topic.id
        );
    }

    let run = PipelineRun::new(&pipeline_def, &issue.title, &topic.id, &issue.id);

    let run_dir = layout.run_dir(&run.id);
    std::fs::create_dir_all(&run_dir)?;

    // Insert pipeline run BEFORE acquiring topic lock (FK: topics.locked_by_run_id → pipeline_runs.id)
    db.upsert_pipeline_run(&run)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let acquired = db
        .acquire_topic_lock(&topic.id, &run.id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if !acquired {
        anyhow::bail!("Failed to acquire lock on topic '{}'. It may have been locked concurrently.", topic.name);
    }

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
            println!("  Topic:    {}", topic.name);
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
                "topic_id": topic.id,
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
