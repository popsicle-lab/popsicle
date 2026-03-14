use std::env;

use popsicle_core::engine::extractor;
use popsicle_core::helpers;
use popsicle_core::model::issue::Priority;
use popsicle_core::model::story::UserStory;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectConfig, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum StoryCommand {
    /// List user stories
    List {
        #[arg(long)]
        issue: Option<String>,
        #[arg(short, long)]
        status: Option<String>,
        #[arg(long)]
        run: Option<String>,
    },
    /// Show user story details
    Show { key: String },
    /// Create a user story manually
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        issue: Option<String>,
        #[arg(long)]
        persona: Option<String>,
        #[arg(long)]
        goal: Option<String>,
        #[arg(long)]
        benefit: Option<String>,
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// Extract user stories from a PRD document
    Extract {
        #[arg(long)]
        from_doc: String,
    },
    /// Update a user story
    Update {
        key: String,
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        priority: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    /// Link an acceptance criterion to a test case
    Link {
        key: String,
        #[arg(long)]
        ac: String,
        #[arg(long)]
        test_case: String,
    },
}

pub fn execute(cmd: StoryCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        StoryCommand::List { issue, status, run } => {
            list_stories(issue.as_deref(), status.as_deref(), run.as_deref(), format)
        }
        StoryCommand::Show { key } => show_story(&key, format),
        StoryCommand::Create {
            title,
            issue,
            persona,
            goal,
            benefit,
            priority,
        } => create_story(
            &title,
            issue.as_deref(),
            persona.as_deref(),
            goal.as_deref(),
            benefit.as_deref(),
            &priority,
            format,
        ),
        StoryCommand::Extract { from_doc } => extract_stories(&from_doc, format),
        StoryCommand::Update {
            key,
            status,
            priority,
            title,
        } => update_story(
            &key,
            status.as_deref(),
            priority.as_deref(),
            title.as_deref(),
            format,
        ),
        StoryCommand::Link { key, ac, test_case } => link_ac(&key, &ac, &test_case, format),
    }
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn load_config(layout: &ProjectLayout) -> anyhow::Result<ProjectConfig> {
    ProjectConfig::load(&layout.config_path()).map_err(|e| anyhow::anyhow!("{}", e))
}

fn list_stories(
    issue_id: Option<&str>,
    status: Option<&str>,
    run_id: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let stories = db
        .query_user_stories(status, issue_id, run_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            if stories.is_empty() {
                println!("No user stories found.");
                return Ok(());
            }
            println!(
                "{:<14} {:<10} {:<12} {:<6} TITLE",
                "KEY", "PRIORITY", "STATUS", "AC"
            );
            println!("{}", "-".repeat(70));
            for s in &stories {
                let verified = s.acceptance_criteria.iter().filter(|a| a.verified).count();
                let total = s.acceptance_criteria.len();
                println!(
                    "{:<14} {:<10} {:<12} {}/{:<4} {}",
                    s.key, s.priority, s.status, verified, total, s.title
                );
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = stories.iter().map(|s| {
                let verified = s.acceptance_criteria.iter().filter(|a| a.verified).count();
                serde_json::json!({
                    "id": s.id, "key": s.key, "title": s.title, "persona": s.persona,
                    "priority": s.priority.to_string(), "status": s.status.to_string(),
                    "issue_id": s.issue_id, "pipeline_run_id": s.pipeline_run_id,
                    "ac_count": s.acceptance_criteria.len(), "ac_verified": verified,
                    "created_at": s.created_at.to_rfc3339(), "updated_at": s.updated_at.to_rfc3339(),
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

fn show_story(key: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let story = db
        .get_user_story(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("User story not found: {}", key))?;

    match format {
        OutputFormat::Text => {
            println!("{} — {}", story.key, story.title);
            println!("  Priority: {}", story.priority);
            println!("  Status:   {}", story.status);
            if !story.persona.is_empty() {
                println!("  Persona:  {}", story.persona);
            }
            if !story.goal.is_empty() {
                println!("  Goal:     {}", story.goal);
            }
            if !story.benefit.is_empty() {
                println!("  Benefit:  {}", story.benefit);
            }
            if !story.acceptance_criteria.is_empty() {
                println!("  Acceptance Criteria:");
                for ac in &story.acceptance_criteria {
                    let check = if ac.verified { "x" } else { " " };
                    println!("    [{}] {} ({})", check, ac.description, ac.id);
                    if !ac.test_case_ids.is_empty() {
                        println!("        linked: {}", ac.test_case_ids.join(", "));
                    }
                }
            }
        }
        OutputFormat::Json => {
            let ac_json: Vec<_> = story
                .acceptance_criteria
                .iter()
                .map(|ac| {
                    serde_json::json!({
                        "id": ac.id, "description": ac.description, "verified": ac.verified,
                        "test_case_ids": ac.test_case_ids,
                    })
                })
                .collect();
            let result = serde_json::json!({
                "id": story.id, "key": story.key, "title": story.title,
                "description": story.description, "persona": story.persona,
                "goal": story.goal, "benefit": story.benefit,
                "priority": story.priority.to_string(), "status": story.status.to_string(),
                "source_doc_id": story.source_doc_id,
                "issue_id": story.issue_id, "pipeline_run_id": story.pipeline_run_id,
                "acceptance_criteria": ac_json,
                "created_at": story.created_at.to_rfc3339(), "updated_at": story.updated_at.to_rfc3339(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn create_story(
    title: &str,
    issue_key: Option<&str>,
    persona: Option<&str>,
    goal: Option<&str>,
    benefit: Option<&str>,
    priority_str: &str,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let priority: Priority = priority_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;
    let layout = project_layout()?;
    let config = load_config(&layout)?;
    let db = IndexDb::open(&layout.db_path())?;

    let prefix = config.project.key_prefix_or_default();
    let seq = db
        .next_story_seq(prefix)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let key = format!("US-{}-{}", prefix, seq);

    let mut story = UserStory::new(key.clone(), title);
    story.priority = priority;
    if let Some(p) = persona {
        story.persona = p.to_string();
    }
    if let Some(g) = goal {
        story.goal = g.to_string();
    }
    if let Some(b) = benefit {
        story.benefit = b.to_string();
    }

    if let Some(ik) = issue_key {
        let issue = db
            .get_issue(ik)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Issue not found: {}", ik))?;
        story.issue_id = Some(issue.id);
    }

    db.create_user_story(&story)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Created user story: {}", key);
            println!("  Title: {}", title);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({ "key": key, "id": story.id, "title": title });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn extract_stories(doc_id: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let config = load_config(&layout)?;
    let db = IndexDb::open(&layout.db_path())?;

    let doc_row = db
        .query_documents(None, None, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .into_iter()
        .find(|d| d.id == doc_id)
        .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

    let doc = FileStorage::read_document(std::path::Path::new(&doc_row.file_path))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let extracted = extractor::extract_user_stories(&doc);
    let prefix = config.project.key_prefix_or_default();

    let run_id_opt = if doc.pipeline_run_id.is_empty() {
        None
    } else {
        Some(doc.pipeline_run_id.as_str())
    };
    let issue_id =
        run_id_opt.and_then(|rid| db.find_issue_by_run_id(rid).ok().flatten().map(|i| i.id));

    let mut created = Vec::new();
    for mut story in extracted {
        let seq = db
            .next_story_seq(prefix)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        story.key = format!("US-{}-{}", prefix, seq);
        story.source_doc_id = Some(doc_id.to_string());
        story.pipeline_run_id = run_id_opt.map(String::from);
        story.issue_id = issue_id.clone();
        db.create_user_story(&story)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        created.push(story);
    }

    match format {
        OutputFormat::Text => {
            println!(
                "Extracted {} user stories from document {}",
                created.len(),
                doc_id
            );
            for s in &created {
                println!(
                    "  {} — {} ({} AC)",
                    s.key,
                    s.title,
                    s.acceptance_criteria.len()
                );
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = created
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "key": s.key, "title": s.title,
                        "ac_count": s.acceptance_criteria.len(),
                    })
                })
                .collect();
            let result = serde_json::json!({ "extracted": created.len(), "user_stories": items });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn update_story(
    key: &str,
    status: Option<&str>,
    priority: Option<&str>,
    title: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let mut story = db
        .get_user_story(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("User story not found: {}", key))?;

    if let Some(s) = status {
        story.status = s.parse().map_err(|e: String| anyhow::anyhow!("{}", e))?;
    }
    if let Some(p) = priority {
        story.priority = p.parse().map_err(|e: String| anyhow::anyhow!("{}", e))?;
    }
    if let Some(t) = title {
        story.title = t.to_string();
    }

    db.update_user_story(&story)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("Updated user story: {}", story.key);
            println!("  Status: {}", story.status);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "key": story.key, "status": story.status.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn link_ac(
    _story_key: &str,
    ac_id: &str,
    test_case_key: &str,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;

    let tc = db
        .get_test_case(test_case_key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("TestCase not found: {}", test_case_key))?;

    db.link_ac_to_test_case(ac_id, &tc.id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => println!("Linked AC {} to test case {}", ac_id, test_case_key),
        OutputFormat::Json => {
            let result = serde_json::json!({ "ac_id": ac_id, "test_case": test_case_key });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
