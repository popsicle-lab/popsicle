use std::env;

use anyhow::Context;
use popsicle_core::model::PipelineDef;
use popsicle_core::registry::PipelineLoader;
use popsicle_core::storage::{FileStorage, IndexDb, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Args)]
pub struct ContextArgs {
    /// Pipeline run ID (omit for latest run)
    #[arg(short, long)]
    run: Option<String>,
    /// Filter to a specific stage
    #[arg(short, long)]
    stage: Option<String>,
}

pub fn execute(args: ContextArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let db = IndexDb::open(&layout.db_path())?;

    let run = if let Some(ref id) = args.run {
        db.get_pipeline_run(id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Pipeline run not found: {}", id))?
    } else {
        let runs = db
            .list_pipeline_runs()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let latest = runs.first().ok_or_else(|| {
            anyhow::anyhow!("No pipeline runs found.")
        })?;
        db.get_pipeline_run(&latest.id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or_else(|| anyhow::anyhow!("Pipeline run not found"))?
    };

    let pipeline_def = find_pipeline(&run.pipeline_name)?;
    let docs = db
        .query_documents(None, None, Some(&run.id))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Text => {
            println!("=== Context: {} ===", run.title);
            println!("Pipeline: {} | Run: {}", run.pipeline_name, run.id);
            println!();

            for stage in &pipeline_def.stages {
                if let Some(ref filter) = args.stage {
                    if &stage.name != filter {
                        continue;
                    }
                }

                let state = run.stage_states.get(&stage.name);
                println!("--- Stage: {} ({}) ---", stage.name, state.map(|s| s.to_string()).unwrap_or_default());

                let stage_docs: Vec<_> = docs
                    .iter()
                    .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
                    .collect();

                if stage_docs.is_empty() {
                    println!("  (no documents yet)\n");
                    continue;
                }

                for doc_row in stage_docs {
                    println!("\n  ## {} [{}] ({})", doc_row.title, doc_row.status, doc_row.doc_type);
                    println!("  ID: {} | Skill: {}", doc_row.id, doc_row.skill_name);

                    if let Ok(doc) = FileStorage::read_document(std::path::Path::new(&doc_row.file_path)) {
                        println!();
                        for line in doc.body.lines() {
                            println!("  {}", line);
                        }
                    }
                    println!();
                }
            }
        }
        OutputFormat::Json => {
            let mut stages_json = Vec::new();

            for stage in &pipeline_def.stages {
                if let Some(ref filter) = args.stage {
                    if &stage.name != filter {
                        continue;
                    }
                }

                let state = run.stage_states.get(&stage.name);
                let stage_docs: Vec<_> = docs
                    .iter()
                    .filter(|d| stage.skill_names().contains(&d.skill_name.as_str()))
                    .collect();

                let docs_json: Vec<_> = stage_docs
                    .iter()
                    .map(|d| {
                        let body = FileStorage::read_document(std::path::Path::new(&d.file_path))
                            .map(|doc| doc.body)
                            .unwrap_or_default();
                        serde_json::json!({
                            "id": d.id,
                            "doc_type": d.doc_type,
                            "title": d.title,
                            "status": d.status,
                            "skill_name": d.skill_name,
                            "body": body,
                        })
                    })
                    .collect();

                stages_json.push(serde_json::json!({
                    "stage": stage.name,
                    "state": state,
                    "documents": docs_json,
                }));
            }

            let result = serde_json::json!({
                "run_id": run.id,
                "pipeline": run.pipeline_name,
                "title": run.title,
                "stages": stages_json,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn find_pipeline(name: &str) -> anyhow::Result<PipelineDef> {
    let cwd = env::current_dir()?;
    let mut all = Vec::new();

    let workspace_pipelines = cwd.join("pipelines");
    if workspace_pipelines.is_dir() {
        all.extend(PipelineLoader::load_dir(&workspace_pipelines).context("Loading pipelines")?);
    }

    all.into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Pipeline template not found: {}", name))
}
