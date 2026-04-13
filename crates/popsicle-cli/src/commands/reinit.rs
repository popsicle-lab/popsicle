use std::env;
use std::path::{Path, PathBuf};

use popsicle_core::storage::{FileStorage, IndexDb, MigrationMapping, ProjectLayout};

use crate::OutputFormat;

#[derive(clap::Args)]
pub struct ReinitArgs {
    /// Skip backup of the old database
    #[arg(long)]
    no_backup: bool,

    /// Generate an LLM prompt for resolving schema mismatches
    #[arg(long, conflicts_with = "apply_mapping")]
    generate_mapping: bool,

    /// Apply an LLM-generated migration mapping file
    #[arg(long, value_name = "FILE", conflicts_with = "generate_mapping")]
    apply_mapping: Option<PathBuf>,
}

pub fn execute(args: ReinitArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let db_path = layout.db_path();
    let export_path = layout.dot_dir().join("export-backup.json");

    // Mode: --generate-mapping (reads saved export, outputs LLM prompt)
    if args.generate_mapping {
        return generate_mapping_prompt(&db_path, &export_path, format);
    }

    // Mode: --apply-mapping (reads saved export + mapping file, re-imports)
    if let Some(mapping_path) = &args.apply_mapping {
        return apply_mapping_file(&db_path, &export_path, mapping_path, format);
    }

    // Normal reinit flow
    if !db_path.exists() {
        anyhow::bail!(
            "No database found at {}. Run `popsicle init` first.",
            db_path.display()
        );
    }

    // Step 1: Export from old DB (open without migration)
    eprintln!("Exporting data from existing database...");
    let export_data = {
        let old_db = IndexDb::open_readonly(&db_path)?;
        old_db.export_all_json()?
    };

    let table_counts: Vec<(String, usize)> = export_data
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter(|(k, _)| !k.starts_with('_'))
                .map(|(k, v)| (k.clone(), v.as_array().map_or(0, |a| a.len())))
                .filter(|(_, count)| *count > 0)
                .collect()
        })
        .unwrap_or_default();

    // Save export for potential --generate-mapping / --apply-mapping later
    std::fs::write(&export_path, serde_json::to_string_pretty(&export_data)?)?;
    eprintln!("  Saved export to {}", export_path.display());

    // Step 2: Backup old DB
    let backup_path = db_path.with_extension("db.bak");
    if !args.no_backup {
        std::fs::copy(&db_path, &backup_path)?;
        eprintln!("  Backed up to {}", backup_path.display());
    }

    // Step 3: Delete old DB (and WAL/SHM files if present)
    std::fs::remove_file(&db_path)?;
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));

    // Step 4: Create fresh DB with latest schema
    eprintln!("Creating fresh database with latest schema...");
    let new_db = IndexDb::open(&db_path)?;

    // Step 5: Import exported data (best-effort)
    eprintln!("Importing data...");
    let import_result = new_db.import_all_json(&export_data)?;

    // Step 6: Detect schema mismatches
    let mismatches = new_db.detect_schema_mismatches(&export_data)?;

    // Step 7: Scan artifacts for documents on disk but not yet in DB
    let artifacts_dir = layout.artifacts_dir();
    let mut rescanned = 0u64;
    if artifacts_dir.is_dir() {
        rescanned = rescan_artifacts(&artifacts_dir, &new_db)?;
    }

    // Clean up export if no mismatches
    let has_unmapped = !import_result.unmapped_columns.is_empty();
    if !has_unmapped && mismatches.is_empty() {
        let _ = std::fs::remove_file(&export_path);
    }

    match format {
        OutputFormat::Text => {
            println!("Reinit complete.");
            if !table_counts.is_empty() {
                println!("  Exported:");
                for (table, count) in &table_counts {
                    println!("    {}: {} rows", table, count);
                }
            }
            println!(
                "  Imported: {} rows ({} skipped/duplicate)",
                import_result.imported, import_result.skipped
            );
            if rescanned > 0 {
                println!("  Rescanned artifacts: {} new documents", rescanned);
            }
            if !args.no_backup {
                println!("  Backup: {}", backup_path.display());
            }
            if has_unmapped {
                println!();
                println!(
                    "  ⚠ {} unmapped column(s) detected:",
                    import_result.unmapped_columns.len()
                );
                for u in &import_result.unmapped_columns {
                    println!(
                        "    {}.{} (samples: {:?})",
                        u.table, u.column, u.sample_values
                    );
                }
                println!();
                println!("  Run `popsicle reinit --generate-mapping` for AI-assisted recovery.");
            }
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": if has_unmapped { "partial" } else { "ok" },
                "exported": table_counts.iter()
                    .map(|(k, v)| (k.clone(), serde_json::json!(*v)))
                    .collect::<serde_json::Map<String, serde_json::Value>>(),
                "imported": import_result.imported,
                "skipped": import_result.skipped,
                "rescanned": rescanned,
                "unmapped_columns": import_result.unmapped_columns,
                "backup": if args.no_backup {
                    serde_json::Value::Null
                } else {
                    serde_json::json!(backup_path.display().to_string())
                },
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

/// `popsicle reinit --generate-mapping` — outputs an LLM prompt for schema migration.
fn generate_mapping_prompt(
    db_path: &Path,
    export_path: &Path,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    if !export_path.exists() {
        anyhow::bail!(
            "No export file found at {}. Run `popsicle reinit` first.",
            export_path.display()
        );
    }

    let export_data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(export_path)?)?;

    if !db_path.exists() {
        anyhow::bail!("Database not found. Run `popsicle reinit` first to create fresh schema.");
    }

    let db = IndexDb::open(db_path)?;
    let prompt = db.generate_migration_prompt(&export_data)?;

    match format {
        OutputFormat::Text => println!("{}", prompt),
        OutputFormat::Json => {
            let mismatches = db.detect_schema_mismatches(&export_data)?;
            let result = serde_json::json!({
                "prompt": prompt,
                "mismatch_count": mismatches.len(),
                "mismatches": mismatches,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

/// `popsicle reinit --apply-mapping <file>` — re-imports with LLM-generated mapping.
fn apply_mapping_file(
    db_path: &Path,
    export_path: &Path,
    mapping_path: &Path,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    if !export_path.exists() {
        anyhow::bail!(
            "No export file found at {}. Run `popsicle reinit` first.",
            export_path.display()
        );
    }
    if !mapping_path.exists() {
        anyhow::bail!("Mapping file not found: {}", mapping_path.display());
    }

    let export_data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(export_path)?)?;
    let mapping: MigrationMapping = serde_json::from_str(&std::fs::read_to_string(mapping_path)?)?;

    if !db_path.exists() {
        anyhow::bail!("Database not found. Run `popsicle reinit` first to create fresh schema.");
    }

    let db = IndexDb::open(db_path)?;

    eprintln!("Applying migration mapping...");
    eprintln!(
        "  Table renames: {}, Column renames: {}, Defaults: {}",
        mapping.table_renames.len(),
        mapping
            .column_renames
            .values()
            .map(|v| v.len())
            .sum::<usize>(),
        mapping
            .default_values
            .values()
            .map(|v| v.len())
            .sum::<usize>(),
    );

    let result = db.import_with_mapping(&export_data, &mapping)?;

    // Clean up export on success if no remaining unmapped columns
    if result.unmapped_columns.is_empty() {
        let _ = std::fs::remove_file(export_path);
        eprintln!("  Cleaned up export file.");
    }

    match format {
        OutputFormat::Text => {
            println!("Mapping applied.");
            println!(
                "  Imported: {} rows ({} skipped)",
                result.imported, result.skipped
            );
            if !result.unmapped_columns.is_empty() {
                println!(
                    "  ⚠ {} unmapped column(s) remain:",
                    result.unmapped_columns.len()
                );
                for u in &result.unmapped_columns {
                    println!("    {}.{}", u.table, u.column);
                }
            }
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": if result.unmapped_columns.is_empty() { "ok" } else { "partial" },
                "imported": result.imported,
                "skipped": result.skipped,
                "unmapped_columns": result.unmapped_columns,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

/// Scan the artifacts directory for document files and upsert any that are
/// not already tracked in the database. Uses `ON CONFLICT` so duplicates are safe.
fn rescan_artifacts(dir: &Path, db: &IndexDb) -> anyhow::Result<u64> {
    let files = FileStorage::list_documents(dir).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut added = 0u64;

    for file_path in files {
        let doc = match FileStorage::read_document(&file_path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if db
            .upsert_document(&doc)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .is_ok()
        {
            added += 1;
        }
    }

    Ok(added)
}
