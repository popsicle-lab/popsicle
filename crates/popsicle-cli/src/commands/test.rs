use std::env;

use popsicle_core::engine::extractor;
use popsicle_core::helpers;
use popsicle_core::model::testcase::{TestRunResult, TestType};
use popsicle_core::storage::{FileStorage, IndexDb, ProjectConfig, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Subcommand)]
pub enum TestCommand {
    /// List test cases
    List {
        #[arg(short = 't', long = "type")]
        test_type: Option<String>,
        #[arg(short, long)]
        priority: Option<String>,
        #[arg(short, long)]
        status: Option<String>,
        #[arg(long)]
        story: Option<String>,
        #[arg(long)]
        run: Option<String>,
    },
    /// Show test case details
    Show { key: String },
    /// Extract test cases from a test-spec document
    Extract {
        #[arg(long)]
        from_doc: String,
        #[arg(short = 't', long = "type", default_value = "unit")]
        test_type: String,
    },
    /// Record a test run result
    RunResult {
        key: String,
        #[arg(long, group = "result")]
        passed: bool,
        #[arg(long, group = "result")]
        failed: bool,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        error: Option<String>,
        #[arg(long)]
        duration_ms: Option<u64>,
    },
    /// Show test coverage summary
    Coverage {
        #[arg(long)]
        run: Option<String>,
    },
}

pub fn execute(cmd: TestCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        TestCommand::List {
            test_type,
            priority,
            status,
            story,
            run,
        } => list_test_cases(
            test_type.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            story.as_deref(),
            run.as_deref(),
            format,
        ),
        TestCommand::Show { key } => show_test_case(&key, format),
        TestCommand::Extract {
            from_doc,
            test_type,
        } => extract_test_cases(&from_doc, &test_type, format),
        TestCommand::RunResult {
            key,
            passed,
            failed: _,
            commit,
            error,
            duration_ms,
        } => record_run_result(
            &key,
            passed,
            commit.as_deref(),
            error.as_deref(),
            duration_ms,
            format,
        ),
        TestCommand::Coverage { run } => show_coverage(run.as_deref(), format),
    }
}

fn project_layout() -> anyhow::Result<ProjectLayout> {
    let cwd = env::current_dir()?;
    helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn load_config(layout: &ProjectLayout) -> anyhow::Result<ProjectConfig> {
    ProjectConfig::load(&layout.config_path()).map_err(|e| anyhow::anyhow!("{}", e))
}

fn list_test_cases(
    test_type: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    story_id: Option<&str>,
    run_id: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let cases = db
        .query_test_cases(test_type, priority, status, story_id, run_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            if cases.is_empty() {
                println!("No test cases found.");
                return Ok(());
            }
            println!(
                "{:<14} {:<6} {:<4} {:<12} TITLE",
                "KEY", "TYPE", "PRI", "STATUS"
            );
            println!("{}", "-".repeat(70));
            for tc in &cases {
                println!(
                    "{:<14} {:<6} {:<4} {:<12} {}",
                    tc.key, tc.test_type, tc.priority_level, tc.status, tc.title
                );
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = cases.iter().map(|tc| serde_json::json!({
                "id": tc.id, "key": tc.key, "title": tc.title,
                "test_type": tc.test_type.to_string(), "priority_level": tc.priority_level.to_string(),
                "status": tc.status.to_string(),
                "source_doc_id": tc.source_doc_id, "user_story_id": tc.user_story_id,
                "issue_id": tc.issue_id, "pipeline_run_id": tc.pipeline_run_id,
                "created_at": tc.created_at.to_rfc3339(), "updated_at": tc.updated_at.to_rfc3339(),
            })).collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
    }
    Ok(())
}

fn show_test_case(key: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let tc = db
        .get_test_case(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("TestCase not found: {}", key))?;
    let runs = db
        .query_test_runs(&tc.id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("{} — {}", tc.key, tc.title);
            println!("  Type:     {}", tc.test_type);
            println!("  Priority: {}", tc.priority_level);
            println!("  Status:   {}", tc.status);
            if !tc.description.is_empty() {
                println!("  Description: {}", tc.description);
            }
            if !tc.steps.is_empty() {
                println!("  Steps:");
                for (i, s) in tc.steps.iter().enumerate() {
                    println!("    {}. {}", i + 1, s);
                }
            }
            if !tc.expected_result.is_empty() {
                println!("  Expected: {}", tc.expected_result);
            }
            if !runs.is_empty() {
                println!("  Recent runs:");
                for r in runs.iter().take(5) {
                    let icon = if r.passed { "PASS" } else { "FAIL" };
                    println!(
                        "    [{}] {} {}",
                        icon,
                        r.run_at.format("%Y-%m-%d %H:%M"),
                        r.error_message.as_deref().unwrap_or("")
                    );
                }
            }
        }
        OutputFormat::Json => {
            let runs_json: Vec<_> = runs
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id, "passed": r.passed, "duration_ms": r.duration_ms,
                        "error_message": r.error_message, "commit_sha": r.commit_sha,
                        "run_at": r.run_at.to_rfc3339(),
                    })
                })
                .collect();
            let result = serde_json::json!({
                "id": tc.id, "key": tc.key, "title": tc.title, "description": tc.description,
                "test_type": tc.test_type.to_string(), "priority_level": tc.priority_level.to_string(),
                "status": tc.status.to_string(), "preconditions": tc.preconditions,
                "steps": tc.steps, "expected_result": tc.expected_result,
                "source_doc_id": tc.source_doc_id, "user_story_id": tc.user_story_id,
                "issue_id": tc.issue_id, "pipeline_run_id": tc.pipeline_run_id,
                "labels": tc.labels, "runs": runs_json,
                "created_at": tc.created_at.to_rfc3339(), "updated_at": tc.updated_at.to_rfc3339(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn extract_test_cases(doc_id: &str, type_str: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let test_type: TestType = type_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;
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

    let extracted = extractor::extract_test_cases(&doc, test_type);
    let prefix = config.project.key_prefix_or_default();

    let run_id_opt = if doc.pipeline_run_id.is_empty() {
        None
    } else {
        Some(doc.pipeline_run_id.as_str())
    };
    let issue_id =
        run_id_opt.and_then(|rid| db.find_issue_by_run_id(rid).ok().flatten().map(|i| i.id));

    let mut created = Vec::new();
    for mut tc in extracted {
        let seq = db
            .next_testcase_seq(prefix)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        tc.key = format!("TC-{}-{}", prefix, seq);
        tc.source_doc_id = Some(doc_id.to_string());
        tc.pipeline_run_id = run_id_opt.map(String::from);
        tc.issue_id = issue_id.clone();
        db.create_test_case(&tc)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        created.push(tc);
    }

    match format {
        OutputFormat::Text => {
            println!(
                "Extracted {} test cases from document {}",
                created.len(),
                doc_id
            );
            for tc in &created {
                println!("  {} — {}", tc.key, tc.title);
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = created
                .iter()
                .map(|tc| {
                    serde_json::json!({
                        "key": tc.key, "title": tc.title, "test_type": tc.test_type.to_string(),
                        "priority_level": tc.priority_level.to_string(),
                    })
                })
                .collect();
            let result = serde_json::json!({ "extracted": created.len(), "test_cases": items });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn record_run_result(
    key: &str,
    passed: bool,
    commit: Option<&str>,
    error: Option<&str>,
    duration_ms: Option<u64>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let tc = db
        .get_test_case(key)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .ok_or_else(|| anyhow::anyhow!("TestCase not found: {}", key))?;

    let mut tr = TestRunResult::new(&tc.id, passed);
    tr.commit_sha = commit.map(String::from);
    tr.error_message = error.map(String::from);
    tr.duration_ms = duration_ms;
    db.insert_test_run(&tr)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            let icon = if passed { "PASS" } else { "FAIL" };
            println!("[{}] {} — {}", icon, tc.key, tc.title);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "test_case": key, "passed": passed, "run_id": tr.id,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn show_coverage(run_id: Option<&str>, format: &OutputFormat) -> anyhow::Result<()> {
    let layout = project_layout()?;
    let db = IndexDb::open(&layout.db_path())?;
    let cases = db
        .query_test_cases(None, None, None, None, run_id)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let total = cases.len();
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut no_runs = 0usize;

    for tc in &cases {
        match db
            .latest_test_run(&tc.id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
        {
            Some(tr) if tr.passed => passed += 1,
            Some(_) => failed += 1,
            None => no_runs += 1,
        }
    }

    let pass_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    match format {
        OutputFormat::Text => {
            println!("=== Test Coverage ===");
            println!("Total:   {}", total);
            println!("Passed:  {}", passed);
            println!("Failed:  {}", failed);
            println!("No runs: {}", no_runs);
            println!("Pass rate: {:.1}%", pass_rate);
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "total": total, "passed": passed, "failed": failed,
                "no_runs": no_runs, "pass_rate": pass_rate,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
