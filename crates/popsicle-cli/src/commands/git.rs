use std::env;

use popsicle_core::git::{CommitLink, GitTracker, ReviewStatus};
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum GitCommand {
    /// Install post-commit git hook for automatic commit tracking
    Init,
    /// Link a commit to a document or pipeline run
    Link {
        /// Commit SHA (defaults to HEAD)
        #[arg(short, long)]
        sha: Option<String>,
        /// Document ID to link to
        #[arg(short, long)]
        doc: Option<String>,
        /// Pipeline run ID (defaults to latest)
        #[arg(short, long)]
        run: Option<String>,
        /// Stage name
        #[arg(long)]
        stage: Option<String>,
        /// Skill name
        #[arg(long)]
        skill: Option<String>,
    },
    /// Show git status with pipeline context
    Status {
        /// Pipeline run ID (defaults to latest)
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Show commit log with document associations
    Log {
        /// Number of commits to show
        #[arg(short, long, default_value = "20")]
        count: usize,
        /// Pipeline run ID filter
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Update review status for a commit
    Review {
        /// Commit SHA
        sha: String,
        /// Review verdict
        #[arg(value_enum)]
        verdict: ReviewVerdict,
        /// Review summary
        #[arg(short, long)]
        summary: Option<String>,
        /// Pipeline run ID (defaults to latest)
        #[arg(short, long)]
        run: Option<String>,
    },
    /// Called by post-commit hook — auto-link HEAD to active pipeline run
    OnCommit,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ReviewVerdict {
    Passed,
    Failed,
    Skipped,
}

pub fn execute(cmd: GitCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        GitCommand::Init => init_hook(format),
        GitCommand::Link {
            sha,
            doc,
            run,
            stage,
            skill,
        } => link_commit(sha.as_deref(), doc.as_deref(), run.as_deref(), stage, skill, format),
        GitCommand::Status { run } => show_status(run.as_deref(), format),
        GitCommand::Log { count, run } => show_log(count, run.as_deref(), format),
        GitCommand::Review {
            sha,
            verdict,
            summary,
            run,
        } => update_review(&sha, verdict, summary.as_deref(), run.as_deref(), format),
        GitCommand::OnCommit => on_commit(format),
    }
}

fn project_layout() -> anyhow::Result<(ProjectLayout, std::path::PathBuf)> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok((layout, cwd))
}

fn get_latest_run_id(db: &IndexDb) -> anyhow::Result<String> {
    let runs = db
        .list_pipeline_runs()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    runs.first()
        .map(|r| r.id.clone())
        .ok_or_else(|| anyhow::anyhow!("No pipeline runs found"))
}

fn init_hook(format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    GitTracker::install_hook(&cwd).map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Installed post-commit hook at .git/hooks/post-commit");
            println!("Commits will be automatically tracked in the active pipeline run.");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({"status": "ok", "hook": "post-commit"}))?
            );
        }
    }
    Ok(())
}

fn link_commit(
    sha: Option<&str>,
    doc_id: Option<&str>,
    run_id: Option<&str>,
    stage: Option<String>,
    skill: Option<String>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let (layout, cwd) = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let sha = match sha {
        Some(s) => s.to_string(),
        None => GitTracker::head_sha(&cwd).map_err(|e| anyhow::anyhow!("{}", e))?,
    };

    let run_id = match run_id {
        Some(r) => r.to_string(),
        None => get_latest_run_id(&db)?,
    };

    let commit = GitTracker::commit_info(&cwd, &sha).map_err(|e| anyhow::anyhow!("{}", e))?;

    let link = CommitLink {
        sha: commit.sha.clone(),
        doc_id: doc_id.map(|s| s.to_string()),
        pipeline_run_id: run_id.clone(),
        stage,
        skill,
        review_status: ReviewStatus::Pending,
        review_summary: None,
        linked_at: chrono::Utc::now().to_rfc3339(),
    };

    db.upsert_commit_link(&link)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!(
                "Linked commit {} to pipeline run {}",
                commit.short_sha, &run_id[..8]
            );
            println!("  Message: {}", commit.message);
            if let Some(ref d) = link.doc_id {
                println!("  Document: {}", d);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&link)?);
        }
    }
    Ok(())
}

fn show_status(run_id: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let (layout, cwd) = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let run_id = match run_id {
        Some(r) => r.to_string(),
        None => get_latest_run_id(&db)?,
    };

    let branch = GitTracker::current_branch(&cwd).unwrap_or_else(|_| "unknown".into());
    let has_changes =
        GitTracker::has_uncommitted_changes(&cwd).unwrap_or(false);
    let head = GitTracker::head_sha(&cwd)
        .map(|s| s[..8].to_string())
        .unwrap_or_else(|_| "unknown".into());

    let links = db
        .query_commit_links(Some(&run_id), None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let total = links.len();
    let pending = links.iter().filter(|l| l.review_status == ReviewStatus::Pending).count();
    let passed = links.iter().filter(|l| l.review_status == ReviewStatus::Passed).count();
    let failed = links.iter().filter(|l| l.review_status == ReviewStatus::Failed).count();

    match format {
        OutputFormat::Text => {
            println!("Git Status");
            println!("  Branch: {}", branch);
            println!("  HEAD: {}", head);
            println!(
                "  Working tree: {}",
                if has_changes { "dirty" } else { "clean" }
            );
            println!("  Pipeline run: {}", &run_id[..8]);
            println!();
            println!("Tracked Commits: {}", total);
            println!("  Pending review: {}", pending);
            println!("  Passed: {}", passed);
            println!("  Failed: {}", failed);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "branch": branch,
                "head": head,
                "uncommitted_changes": has_changes,
                "pipeline_run_id": run_id,
                "commits": { "total": total, "pending": pending, "passed": passed, "failed": failed },
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn show_log(
    count: usize,
    run_id: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let (layout, cwd) = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let commits =
        GitTracker::recent_commits(&cwd, count).map_err(|e| anyhow::anyhow!("{}", e))?;

    let run_filter = match run_id {
        Some(r) => Some(r.to_string()),
        None => get_latest_run_id(&db).ok(),
    };

    let links = db
        .query_commit_links(run_filter.as_deref(), None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!(
                "{:<10} {:<12} {:<10} {:<10} MESSAGE",
                "SHA", "REVIEW", "STAGE", "SKILL"
            );
            println!("{}", "-".repeat(80));

            for commit in &commits {
                let link = links.iter().find(|l| l.sha == commit.sha);
                let review = link
                    .map(|l| l.review_status.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let stage = link
                    .and_then(|l| l.stage.as_deref())
                    .unwrap_or("-");
                let skill = link
                    .and_then(|l| l.skill.as_deref())
                    .unwrap_or("-");
                let msg: String = commit.message.chars().take(40).collect();
                println!(
                    "{:<10} {:<12} {:<10} {:<10} {}",
                    &commit.short_sha, review, stage, skill, msg
                );
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = commits
                .iter()
                .map(|c| {
                    let link = links.iter().find(|l| l.sha == c.sha);
                    serde_json::json!({
                        "sha": c.sha,
                        "short_sha": c.short_sha,
                        "message": c.message,
                        "author": c.author,
                        "timestamp": c.timestamp,
                        "link": link,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

fn update_review(
    sha: &str,
    verdict: ReviewVerdict,
    summary: Option<&str>,
    run_id: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let (layout, cwd) = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let full_sha = GitTracker::commit_info(&cwd, sha)
        .map(|c| c.sha)
        .unwrap_or_else(|_| sha.to_string());

    let run_id = match run_id {
        Some(r) => r.to_string(),
        None => get_latest_run_id(&db)?,
    };

    let status = match verdict {
        ReviewVerdict::Passed => ReviewStatus::Passed,
        ReviewVerdict::Failed => ReviewStatus::Failed,
        ReviewVerdict::Skipped => ReviewStatus::Skipped,
    };

    db.update_commit_review(&full_sha, &run_id, status, summary)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Updated review: {} → {}", &sha[..8.min(sha.len())], status);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "sha": sha, "review_status": status.to_string(), "summary": summary,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn on_commit(format: &OutputFormat) -> anyhow::Result<()> {
    let (layout, cwd) = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let run_id = match get_latest_run_id(&db) {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    let sha = GitTracker::head_sha(&cwd).map_err(|e| anyhow::anyhow!("{}", e))?;
    let commit = GitTracker::commit_info(&cwd, &sha).map_err(|e| anyhow::anyhow!("{}", e))?;

    let link = CommitLink {
        sha: commit.sha.clone(),
        doc_id: None,
        pipeline_run_id: run_id.clone(),
        stage: None,
        skill: None,
        review_status: ReviewStatus::Pending,
        review_summary: None,
        linked_at: chrono::Utc::now().to_rfc3339(),
    };

    db.upsert_commit_link(&link)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!(
                "Tracked commit {} in pipeline run {}",
                commit.short_sha,
                &run_id[..8]
            );
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&link)?);
        }
    }
    Ok(())
}
