//! SQLite single-file state backend (ADR-009 Phase 2, PROJ-11).
//!
//! Owns the indexed workspace state (meta counters, issues, runs, documents)
//! at `.popsicle/popsicle.db`. Pipeline session working files intentionally
//! stay as per-run JSON (human-inspectable, atomic per run) — see ADR-013.
//! Document bodies live in artifact files, not in the db.

use std::path::Path;

use rusqlite::{params, Connection};

use crate::{DocumentRow, IssueRow, IssueTaskLink, RunRow, WorkspaceError};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS issues (
    key         TEXT PRIMARY KEY,
    issue_type  TEXT NOT NULL,
    priority    TEXT NOT NULL,
    status      TEXT NOT NULL,
    title       TEXT NOT NULL,
    spec_id     TEXT NOT NULL,
    pipeline    TEXT,
    description TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS runs (
    run_id        TEXT PRIMARY KEY,
    issue_key     TEXT NOT NULL,
    pipeline_name TEXT NOT NULL,
    spec_id       TEXT NOT NULL,
    spec_locked   INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS documents (
    id        TEXT PRIMARY KEY,
    doc_type  TEXT NOT NULL,
    title     TEXT NOT NULL,
    status    TEXT NOT NULL,
    version   INTEGER NOT NULL,
    parent_id TEXT,
    file_path TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS issue_tasks (
    issue_key      TEXT NOT NULL,
    sort_order     INTEGER NOT NULL,
    role           TEXT NOT NULL,
    task_id        TEXT,
    proposed_title TEXT,
    journey_stage  TEXT,
    source         TEXT NOT NULL,
    PRIMARY KEY (issue_key, sort_order)
);
";

/// Full indexed-state snapshot exchanged with the db in one shot. The state is
/// small (tens of rows); whole-snapshot load/save inside a transaction keeps
/// the WorkspaceStore semantics identical to the Phase 1 TSV backend.
#[derive(Debug, Clone, Default)]
pub struct StateSnapshot {
    pub next_issue_num: u32,
    pub next_run_num: u32,
    pub next_doc_num: u32,
    pub issues: Vec<IssueRow>,
    pub issue_tasks: Vec<IssueTaskLink>,
    pub runs: Vec<RunRow>,
    pub documents: Vec<DocumentRow>,
}

/// Handle on the SQLite state database.
pub struct SqliteStateDb {
    conn: Connection,
}

impl SqliteStateDb {
    /// Open (creating schema if needed) the db at `path`.
    pub fn open(path: &Path) -> Result<Self, WorkspaceError> {
        let conn = Connection::open(path).map_err(db_err)?;
        conn.execute_batch(SCHEMA).map_err(db_err)?;
        Self::migrate_schema(&conn)?;
        Ok(Self { conn })
    }

    fn migrate_schema(conn: &Connection) -> Result<(), WorkspaceError> {
        let has_product: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('issues') WHERE name = 'product_id'")
            .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, i64>(0)))
            .unwrap_or(0)
            > 0;
        if !has_product {
            conn.execute(
                "ALTER TABLE issues ADD COLUMN product_id TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(db_err)?;
        }
        let has_epic: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('issues') WHERE name = 'epic_task_id'")
            .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, i64>(0)))
            .unwrap_or(0)
            > 0;
        if !has_epic {
            conn.execute("ALTER TABLE issues ADD COLUMN epic_task_id TEXT", [])
                .map_err(db_err)?;
        }
        Ok(())
    }

    pub fn load(&self) -> Result<StateSnapshot, WorkspaceError> {
        let mut snap = StateSnapshot {
            next_issue_num: self.meta("next_issue_num")?.unwrap_or(1),
            next_run_num: self.meta("next_run_num")?.unwrap_or(1),
            next_doc_num: self.meta("next_doc_num")?.unwrap_or(1),
            ..StateSnapshot::default()
        };

        let mut stmt = self
            .conn
            .prepare(
                "SELECT key, issue_type, priority, status, title, product_id, spec_id, pipeline, description, epic_task_id FROM issues",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                let product_id: String = row.get(5)?;
                let spec_id: String = row.get(6)?;
                Ok(IssueRow {
                    key: row.get(0)?,
                    issue_type: row.get(1)?,
                    priority: row.get(2)?,
                    status: row.get(3)?,
                    title: row.get(4)?,
                    product_id: if product_id.is_empty() {
                        spec_id.clone()
                    } else {
                        product_id
                    },
                    spec_id,
                    pipeline: row.get(7)?,
                    description: row.get(8)?,
                    epic_task_id: row.get(9)?,
                })
            })
            .map_err(db_err)?;
        for row in rows {
            snap.issues.push(row.map_err(db_err)?);
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT issue_key, sort_order, role, task_id, proposed_title, journey_stage, source
                 FROM issue_tasks ORDER BY issue_key, sort_order",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(IssueTaskLink {
                    issue_key: row.get(0)?,
                    sort_order: row.get::<_, i64>(1)? as u32,
                    role: row.get(2)?,
                    task_id: row.get(3)?,
                    proposed_title: row.get(4)?,
                    journey_stage: row.get(5)?,
                    source: row.get(6)?,
                })
            })
            .map_err(db_err)?;
        for row in rows {
            snap.issue_tasks.push(row.map_err(db_err)?);
        }

        let mut stmt = self
            .conn
            .prepare("SELECT run_id, issue_key, pipeline_name, spec_id, spec_locked FROM runs")
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RunRow {
                    run_id: row.get(0)?,
                    issue_key: row.get(1)?,
                    pipeline_name: row.get(2)?,
                    spec_id: row.get(3)?,
                    spec_locked: row.get::<_, i64>(4)? != 0,
                })
            })
            .map_err(db_err)?;
        for row in rows {
            snap.runs.push(row.map_err(db_err)?);
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, doc_type, title, status, version, parent_id, file_path FROM documents",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DocumentRow {
                    id: row.get(0)?,
                    doc_type: row.get(1)?,
                    title: row.get(2)?,
                    status: row.get(3)?,
                    version: row.get(4)?,
                    parent_id: row.get(5)?,
                    file_path: row.get(6)?,
                    body: String::new(),
                })
            })
            .map_err(db_err)?;
        for row in rows {
            snap.documents.push(row.map_err(db_err)?);
        }

        Ok(snap)
    }

    /// Replace the entire indexed state in one transaction.
    pub fn save(&mut self, snap: &StateSnapshot) -> Result<(), WorkspaceError> {
        let tx = self.conn.transaction().map_err(db_err)?;
        tx.execute_batch(
            "DELETE FROM meta; DELETE FROM issues; DELETE FROM issue_tasks; DELETE FROM runs; DELETE FROM documents;",
        )
        .map_err(db_err)?;
        for (key, value) in [
            ("next_issue_num", snap.next_issue_num),
            ("next_run_num", snap.next_run_num),
            ("next_doc_num", snap.next_doc_num),
        ] {
            tx.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(db_err)?;
        }
        for issue in &snap.issues {
            tx.execute(
                "INSERT INTO issues (key, issue_type, priority, status, title, product_id, spec_id, pipeline, description, epic_task_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    issue.key,
                    issue.issue_type,
                    issue.priority,
                    issue.status,
                    issue.title,
                    issue.product_id,
                    issue.spec_id,
                    issue.pipeline,
                    issue.description,
                    issue.epic_task_id,
                ],
            )
            .map_err(db_err)?;
        }
        for link in &snap.issue_tasks {
            tx.execute(
                "INSERT INTO issue_tasks (issue_key, sort_order, role, task_id, proposed_title, journey_stage, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    link.issue_key,
                    link.sort_order as i64,
                    link.role,
                    link.task_id,
                    link.proposed_title,
                    link.journey_stage,
                    link.source,
                ],
            )
            .map_err(db_err)?;
        }
        for run in &snap.runs {
            tx.execute(
                "INSERT INTO runs (run_id, issue_key, pipeline_name, spec_id, spec_locked)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    run.run_id,
                    run.issue_key,
                    run.pipeline_name,
                    run.spec_id,
                    run.spec_locked as i64,
                ],
            )
            .map_err(db_err)?;
        }
        for doc in &snap.documents {
            tx.execute(
                "INSERT INTO documents (id, doc_type, title, status, version, parent_id, file_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    doc.id,
                    doc.doc_type,
                    doc.title,
                    doc.status,
                    doc.version,
                    doc.parent_id,
                    doc.file_path,
                ],
            )
            .map_err(db_err)?;
        }
        tx.commit().map_err(db_err)
    }
}

fn db_err(e: rusqlite::Error) -> WorkspaceError {
    WorkspaceError::Io(format!("sqlite: {e}"))
}

impl SqliteStateDb {
    fn meta(&self, key: &str) -> Result<Option<u32>, WorkspaceError> {
        self.conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(db_err(other)),
            })
    }
}
