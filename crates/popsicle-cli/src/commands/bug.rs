use std::env;

use popsicle_core::helpers;
use popsicle_core::model::bug::{Bug, BugSeverity, BugSource};
use popsicle_core::model::issue::Priority;
use popsicle_core::model::testcase::TestRunResult;
use popsicle_core::storage::{IndexDb, ProjectConfig, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum BugCommand {
    /// Create a new bug report
    Create {
        #[arg(long)]
        title: String,
        #[arg(short, long, default_value = "")]
        description: String,
        #[arg(short, long, default_value = "major")]
        severity: String,
        #[arg(short, long, default_value = "medium")]
        priority: String,
        #[arg(long)]
        issue: Option<String>,
        #[arg(long)]
        run: Option<String>,
        #[arg(short, long)]
        label: Vec<String>,
    },
    /// List bugs
    List {
        #[arg(short, long)]
        severity: Option<String>,
        #[arg(short = 'S', long)]
        status: Option<String>,
        #[arg(long)]
        issue: Option<String>,
        #[arg(long)]
        run: Option<String>,
    },
    /// Show bug details
    Show { key: String },
    /// Update a bug
    Update {
        key: String,
        #[arg(short = 'S', long)]
        status: Option<String>,
        #[arg(short, long)]
        severity: Option<String>,
        #[arg(short, long)]
        priority: Option<String>,
        #[arg(long)]
        fix_commit: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    /// Link a bug to a commit
    Link {
        key: String,
        #[arg(long)]
        commit: String,
    },
    /// Record a bug from a test failure (auto-creates TestRunResult + deduplicates)
    Record {
        #[arg(long)]
        from_test: String,
        #[arg(long)]
        error: String,
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        commit: Option<String>,
    },
}

pub fn execute(cmd: BugCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        BugCommand::Create {
            title,
            description,
            severity,
            priority,
            issue,
            run,
            label,
        } => create_bug(
            &title,
            &description,
            &severity,
            &priority,
            issue.as_deref(),
            run.as_deref(),
            &label,
            format,
        ),
        BugCommand::List {
            severity,
            status,
            issue,
            run,
        } => list_bugs(
            severity.as_deref(),
            status.as_deref(),
            issue.as_deref(),
            run.as_deref(),
            format,
        ),
        BugCommand::Show { key } => show_bug(&key, format),
        BugCommand::Update {
            key,
            status,
            severity,
            priority,
            fix_commit,
            title,
        } => update_bug(
            &key,
            status.as_deref(),
            severity.as_deref(),
            priority.as_deref(),
            fix_commit.as_deref(),
            title.as_deref(),
            format,
        ),
        BugCommand::Link { key, commit } => link_bug(&key, &commit, format),
        BugCommand::Record {
            from_test,
            error,
            run,
            commit,
        } => record_bug(
            &from_test,
            &error,
            run.as_deref(),
            commit.as_deref(),
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

#[allow(clippy::too_many_arguments)]
fn create_bug(
    title: &str,
    description: &str,
    severity_str: &str,
    priority_str: &str,
    issue_key: Option<&str>,
    run_id: Option<&str>,
    labels: &[String],
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let severity: BugSeverity = severity_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;
    let priority: Priority = priority_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;

    let layout = project_layout()?;
    let config = load_config(&layout)?;
    let db = IndexDb::open(&layout.db_path())?;

    let prefix = config.project.key_prefix_or_default();
    let seq = db
        .next_bug_seq(prefix)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let key = format!("BUG-{}-{}", prefix, seq);

    let mut bug = Bug::new(key.clone(), title, severity);
    bug.description = description.to_string();
    bug.priority = priority;
    bug.labels = labels.to_vec();

    if let Some(ik) = issue_key {
        let issue = db
            .get_issue(ik)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Issue not found: {}", ik))?;
        bug.issue_id = Some(issue.id);
    }
    if let Some(rid) = run_id {
        bug.pipeline_run_id = Some(rid.to_string());
    }

    db.create_bug(&bug).map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Created bug: {}", key);
            println!("  Title:    {}", title);
            println!("  Severity: {}", bug.severity);
            println!("  Priority: {}", bug.priority);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "key": key, "id": bug.id, "title": title,
                "severity": bug.severity.to_string(), "priority": bug.priority.to_string(),
                "status": bug.status.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn list_bugs(
    severity: Option<&str>,
    status: Option<&str>,
    issue_id: Option<&str>,
    run_id: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let bugs = db
        .query_bugs(severity, status, issue_id, run_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            if bugs.is_empty() {
                println!("No bugs found.");
                return Ok(());
            }
            println!(
                "{:<16} {:<10} {:<10} {:<12} TITLE",
                "KEY", "SEVERITY", "PRIORITY", "STATUS"
            );
            println!("{}", "-".repeat(75));
            for b in &bugs {
                println!(
                    "{:<16} {:<10} {:<10} {:<12} {}",
                    b.key, b.severity, b.priority, b.status, b.title
                );
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = bugs.iter().map(|b| serde_json::json!({
                "id": b.id, "key": b.key, "title": b.title,
                "severity": b.severity.to_string(), "priority": b.priority.to_string(),
                "status": b.status.to_string(), "source": b.source.to_string(),
                "issue_id": b.issue_id, "pipeline_run_id": b.pipeline_run_id,
                "labels": b.labels,
                "created_at": b.created_at.to_rfc3339(), "updated_at": b.updated_at.to_rfc3339(),
            })).collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

fn show_bug(key: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let bug = db
        .get_bug(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Bug not found: {}", key))?;

    match format {
        OutputFormat::Text => {
            println!("{} — {}", bug.key, bug.title);
            println!("  Severity: {}", bug.severity);
            println!("  Priority: {}", bug.priority);
            println!("  Status:   {}", bug.status);
            println!("  Source:   {}", bug.source);
            if !bug.description.is_empty() {
                println!("  Description:\n    {}", bug.description);
            }
            if !bug.steps_to_reproduce.is_empty() {
                println!("  Steps to reproduce:");
                for (i, s) in bug.steps_to_reproduce.iter().enumerate() {
                    println!("    {}. {}", i + 1, s);
                }
            }
            if !bug.expected_behavior.is_empty() {
                println!("  Expected: {}", bug.expected_behavior);
            }
            if !bug.actual_behavior.is_empty() {
                println!("  Actual:   {}", bug.actual_behavior);
            }
            if let Some(ref env) = bug.environment {
                println!("  Environment: {}", env);
            }
            if let Some(ref tc) = bug.related_test_case_id {
                println!("  Related TC: {}", tc);
            }
            if let Some(ref sha) = bug.fix_commit_sha {
                println!("  Fix commit: {}", sha);
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": bug.id, "key": bug.key, "title": bug.title,
                "description": bug.description,
                "severity": bug.severity.to_string(), "priority": bug.priority.to_string(),
                "status": bug.status.to_string(), "source": bug.source.to_string(),
                "steps_to_reproduce": bug.steps_to_reproduce,
                "expected_behavior": bug.expected_behavior,
                "actual_behavior": bug.actual_behavior,
                "environment": bug.environment, "stack_trace": bug.stack_trace,
                "related_test_case_id": bug.related_test_case_id,
                "related_commit_sha": bug.related_commit_sha,
                "fix_commit_sha": bug.fix_commit_sha,
                "issue_id": bug.issue_id, "pipeline_run_id": bug.pipeline_run_id,
                "labels": bug.labels,
                "created_at": bug.created_at.to_rfc3339(), "updated_at": bug.updated_at.to_rfc3339(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn update_bug(
    key: &str,
    status: Option<&str>,
    severity: Option<&str>,
    priority: Option<&str>,
    fix_commit: Option<&str>,
    title: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut bug = db
        .get_bug(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Bug not found: {}", key))?;

    if let Some(s) = status {
        bug.status = s.parse().map_err(|e: String| anyhow::anyhow!("{}", e))?;
    }
    if let Some(s) = severity {
        bug.severity = s.parse().map_err(|e: String| anyhow::anyhow!("{}", e))?;
    }
    if let Some(p) = priority {
        bug.priority = p.parse().map_err(|e: String| anyhow::anyhow!("{}", e))?;
    }
    if let Some(fc) = fix_commit {
        bug.fix_commit_sha = Some(fc.to_string());
    }
    if let Some(t) = title {
        bug.title = t.to_string();
    }

    db.update_bug(&bug).map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Updated bug: {}", bug.key);
            println!("  Status:   {}", bug.status);
            println!("  Severity: {}", bug.severity);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "key": bug.key, "status": bug.status.to_string(),
                "severity": bug.severity.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn link_bug(key: &str, commit: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut bug = db
        .get_bug(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("Bug not found: {}", key))?;

    bug.related_commit_sha = Some(commit.to_string());
    db.update_bug(&bug).map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => println!("Linked bug {} to commit {}", key, commit),
        OutputFormat::Json => {
            let result = serde_json::json!({ "key": key, "commit": commit });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn record_bug(
    from_test: &str,
    error: &str,
    run_id: Option<&str>,
    commit: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let config = load_config(&layout)?;
    let db = IndexDb::open(&layout.db_path())?;

    let tc = db
        .get_test_case(from_test)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("TestCase not found: {}", from_test))?;

    let mut tr = TestRunResult::new(&tc.id, false);
    tr.error_message = Some(error.to_string());
    tr.commit_sha = commit.map(String::from);
    db.insert_test_run(&tr)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Some(existing) = db
        .find_open_bug_by_test_case(&tc.id)
        .map_err(|e| anyhow::anyhow!("{}", e))?
    {
        match format {
            OutputFormat::Text => println!(
                "Existing open bug {} for test case {}. Skipping creation.",
                existing.key, from_test
            ),
            OutputFormat::Json => {
                let result = serde_json::json!({ "action": "deduplicated", "existing_bug": existing.key, "test_run_id": tr.id });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        return Ok(());
    }

    let prefix = config.project.key_prefix_or_default();
    let seq = db
        .next_bug_seq(prefix)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let key = format!("BUG-{}-{}", prefix, seq);

    let mut bug = Bug::new(
        key.clone(),
        &format!("Test failure: {}", tc.title),
        BugSeverity::Major,
    );
    bug.source = BugSource::TestFailure;
    bug.related_test_case_id = Some(tc.id.clone());
    bug.actual_behavior = error.to_string();
    bug.pipeline_run_id = run_id.map(String::from).or(tc.pipeline_run_id.clone());
    bug.issue_id = tc.issue_id.clone();

    db.create_bug(&bug).map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Recorded bug {} from test failure: {}", key, tc.title);
            println!("  Test run: {}", tr.id);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "action": "created", "key": key, "id": bug.id,
                "test_case": from_test, "test_run_id": tr.id,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
