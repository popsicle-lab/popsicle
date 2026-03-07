use rusqlite::{params, Connection};
use std::path::Path;

use crate::error::Result;
use crate::git::{CommitLink, ReviewStatus};
use crate::model::{Document, PipelineRun, StageState};

/// SQLite-backed metadata index for fast queries.
pub struct IndexDb {
    conn: Connection,
}

impl IndexDb {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Create in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                doc_type TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                skill_name TEXT NOT NULL,
                pipeline_run_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                created_at TEXT,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS pipeline_runs (
                id TEXT PRIMARY KEY,
                pipeline_name TEXT NOT NULL,
                title TEXT NOT NULL,
                stage_states_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_doc_skill ON documents(skill_name);
            CREATE INDEX IF NOT EXISTS idx_doc_run ON documents(pipeline_run_id);
            CREATE INDEX IF NOT EXISTS idx_doc_status ON documents(status);

            CREATE TABLE IF NOT EXISTS commit_links (
                sha TEXT NOT NULL,
                doc_id TEXT,
                pipeline_run_id TEXT NOT NULL,
                stage TEXT,
                skill TEXT,
                review_status TEXT NOT NULL DEFAULT 'pending',
                review_summary TEXT,
                linked_at TEXT NOT NULL,
                PRIMARY KEY (sha, pipeline_run_id)
            );

            CREATE INDEX IF NOT EXISTS idx_cl_run ON commit_links(pipeline_run_id);
            CREATE INDEX IF NOT EXISTS idx_cl_doc ON commit_links(doc_id);
            CREATE INDEX IF NOT EXISTS idx_cl_review ON commit_links(review_status);
            ",
        )?;
        Ok(())
    }

    /// Upsert a document's metadata into the index.
    pub fn upsert_document(&self, doc: &Document) -> Result<()> {
        self.conn.execute(
            "INSERT INTO documents (id, doc_type, title, status, skill_name, pipeline_run_id, file_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                title = excluded.title,
                file_path = excluded.file_path,
                updated_at = excluded.updated_at",
            params![
                doc.id,
                doc.doc_type,
                doc.title,
                doc.status,
                doc.skill_name,
                doc.pipeline_run_id,
                doc.file_path.display().to_string(),
                doc.created_at.map(|t| t.to_rfc3339()),
                doc.updated_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    /// Query documents by optional filters.
    pub fn query_documents(
        &self,
        skill: Option<&str>,
        status: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<Vec<DocumentRow>> {
        let mut sql = "SELECT id, doc_type, title, status, skill_name, pipeline_run_id, file_path, created_at, updated_at FROM documents WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(s) = skill {
            sql.push_str(&format!(" AND skill_name = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(s) = status {
            sql.push_str(&format!(" AND status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(r) = run_id {
            sql.push_str(&format!(" AND pipeline_run_id = ?{}", param_values.len() + 1));
            param_values.push(Box::new(r.to_string()));
        }

        sql.push_str(" ORDER BY created_at ASC");

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(DocumentRow {
                id: row.get(0)?,
                doc_type: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                skill_name: row.get(4)?,
                pipeline_run_id: row.get(5)?,
                file_path: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Save a pipeline run.
    pub fn upsert_pipeline_run(&self, run: &PipelineRun) -> Result<()> {
        let states_json = serde_json::to_string(&run.stage_states)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO pipeline_runs (id, pipeline_name, title, stage_states_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                stage_states_json = excluded.stage_states_json,
                updated_at = excluded.updated_at",
            params![
                run.id,
                run.pipeline_name,
                run.title,
                states_json,
                run.created_at.to_rfc3339(),
                run.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Load a pipeline run by ID.
    pub fn get_pipeline_run(&self, run_id: &str) -> Result<Option<PipelineRun>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipeline_name, title, stage_states_json, created_at, updated_at FROM pipeline_runs WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![run_id], |row| {
            let states_json: String = row.get(3)?;
            let created_str: String = row.get(4)?;
            let updated_str: String = row.get(5)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                states_json,
                created_str,
                updated_str,
            ))
        })?;

        if let Some(row) = rows.next() {
            let (id, pipeline_name, title, states_json, created_str, updated_str) = row?;
            let stage_states: std::collections::HashMap<String, StageState> =
                serde_json::from_str(&states_json)
                    .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);

            Ok(Some(PipelineRun {
                id,
                pipeline_name,
                title,
                stage_states,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all pipeline runs.
    pub fn list_pipeline_runs(&self) -> Result<Vec<PipelineRunRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipeline_name, title, created_at, updated_at FROM pipeline_runs ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(PipelineRunRow {
                id: row.get(0)?,
                pipeline_name: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Upsert a commit link.
    pub fn upsert_commit_link(&self, link: &CommitLink) -> Result<()> {
        self.conn.execute(
            "INSERT INTO commit_links (sha, doc_id, pipeline_run_id, stage, skill, review_status, review_summary, linked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(sha, pipeline_run_id) DO UPDATE SET
                doc_id = COALESCE(excluded.doc_id, commit_links.doc_id),
                stage = COALESCE(excluded.stage, commit_links.stage),
                skill = COALESCE(excluded.skill, commit_links.skill),
                review_status = excluded.review_status,
                review_summary = COALESCE(excluded.review_summary, commit_links.review_summary)",
            params![
                link.sha,
                link.doc_id,
                link.pipeline_run_id,
                link.stage,
                link.skill,
                link.review_status.to_string(),
                link.review_summary,
                link.linked_at,
            ],
        )?;
        Ok(())
    }

    /// Query commit links for a pipeline run.
    pub fn query_commit_links(
        &self,
        run_id: Option<&str>,
        doc_id: Option<&str>,
        review_status: Option<&str>,
    ) -> Result<Vec<CommitLink>> {
        let mut sql = "SELECT sha, doc_id, pipeline_run_id, stage, skill, review_status, review_summary, linked_at FROM commit_links WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(r) = run_id {
            sql.push_str(&format!(" AND pipeline_run_id = ?{}", param_values.len() + 1));
            param_values.push(Box::new(r.to_string()));
        }
        if let Some(d) = doc_id {
            sql.push_str(&format!(" AND doc_id = ?{}", param_values.len() + 1));
            param_values.push(Box::new(d.to_string()));
        }
        if let Some(rs) = review_status {
            sql.push_str(&format!(" AND review_status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(rs.to_string()));
        }

        sql.push_str(" ORDER BY linked_at DESC");

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let status_str: String = row.get(5)?;
            let review_status = match status_str.as_str() {
                "passed" => ReviewStatus::Passed,
                "failed" => ReviewStatus::Failed,
                "skipped" => ReviewStatus::Skipped,
                _ => ReviewStatus::Pending,
            };
            Ok(CommitLink {
                sha: row.get(0)?,
                doc_id: row.get(1)?,
                pipeline_run_id: row.get(2)?,
                stage: row.get(3)?,
                skill: row.get(4)?,
                review_status,
                review_summary: row.get(6)?,
                linked_at: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Update review status for a commit.
    pub fn update_commit_review(
        &self,
        sha: &str,
        run_id: &str,
        status: ReviewStatus,
        summary: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE commit_links SET review_status = ?1, review_summary = ?2 WHERE sha = ?3 AND pipeline_run_id = ?4",
            params![status.to_string(), summary, sha, run_id],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DocumentRow {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub skill_name: String,
    pub pipeline_run_id: String,
    pub file_path: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PipelineRunRow {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{CommitLink, ReviewStatus};
    use crate::model::{PipelineDef, StageDef};
    use std::path::PathBuf;

    fn make_doc(id: &str, skill: &str, run_id: &str) -> Document {
        Document {
            id: id.to_string(),
            doc_type: "test".to_string(),
            title: format!("Doc {}", id),
            status: "draft".to_string(),
            skill_name: skill.to_string(),
            pipeline_run_id: run_id.to_string(),
            tags: vec![],
            metadata: serde_yaml_ng::Value::Null,
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            body: "test body".to_string(),
            file_path: PathBuf::from("test.md"),
        }
    }

    fn make_pipeline_def() -> PipelineDef {
        PipelineDef {
            name: "test-pipeline".to_string(),
            description: "Test".to_string(),
            stages: vec![StageDef {
                name: "stage-1".to_string(),
                skills: vec![],
                skill: Some("domain-analysis".to_string()),
                description: "First".to_string(),
                depends_on: vec![],
            }],
        }
    }

    #[test]
    fn test_document_upsert_and_query() {
        let db = IndexDb::open_in_memory().unwrap();
        let doc = make_doc("d1", "product-prd", "run-1");
        db.upsert_document(&doc).unwrap();

        let results = db.query_documents(None, None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d1");
        assert_eq!(results[0].skill_name, "product-prd");
    }

    #[test]
    fn test_document_query_filters() {
        let db = IndexDb::open_in_memory().unwrap();
        db.upsert_document(&make_doc("d1", "prd", "run-1")).unwrap();
        db.upsert_document(&make_doc("d2", "adr", "run-1")).unwrap();
        db.upsert_document(&make_doc("d3", "prd", "run-2")).unwrap();

        let by_skill = db.query_documents(Some("prd"), None, None).unwrap();
        assert_eq!(by_skill.len(), 2);

        let by_run = db.query_documents(None, None, Some("run-1")).unwrap();
        assert_eq!(by_run.len(), 2);

        let by_both = db.query_documents(Some("prd"), None, Some("run-1")).unwrap();
        assert_eq!(by_both.len(), 1);
    }

    #[test]
    fn test_document_upsert_updates_status() {
        let db = IndexDb::open_in_memory().unwrap();
        let mut doc = make_doc("d1", "prd", "run-1");
        db.upsert_document(&doc).unwrap();

        doc.status = "approved".to_string();
        db.upsert_document(&doc).unwrap();

        let results = db.query_documents(None, None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "approved");
    }

    #[test]
    fn test_pipeline_run_roundtrip() {
        let db = IndexDb::open_in_memory().unwrap();
        let def = make_pipeline_def();
        let run = PipelineRun::new(&def, "Feature X");

        db.upsert_pipeline_run(&run).unwrap();
        let loaded = db.get_pipeline_run(&run.id).unwrap().unwrap();

        assert_eq!(loaded.id, run.id);
        assert_eq!(loaded.pipeline_name, "test-pipeline");
        assert_eq!(loaded.title, "Feature X");
        assert_eq!(loaded.stage_states["stage-1"], StageState::Ready);
    }

    #[test]
    fn test_pipeline_run_not_found() {
        let db = IndexDb::open_in_memory().unwrap();
        let result = db.get_pipeline_run("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_pipeline_runs() {
        let db = IndexDb::open_in_memory().unwrap();
        let def = make_pipeline_def();

        let run1 = PipelineRun::new(&def, "Run 1");
        let run2 = PipelineRun::new(&def, "Run 2");
        db.upsert_pipeline_run(&run1).unwrap();
        db.upsert_pipeline_run(&run2).unwrap();

        let runs = db.list_pipeline_runs().unwrap();
        assert_eq!(runs.len(), 2);
    }

    #[test]
    fn test_commit_link_upsert_and_query() {
        let db = IndexDb::open_in_memory().unwrap();
        let link = CommitLink {
            sha: "abc123".to_string(),
            doc_id: Some("d1".to_string()),
            pipeline_run_id: "run-1".to_string(),
            stage: Some("stage-1".to_string()),
            skill: Some("prd".to_string()),
            review_status: ReviewStatus::Pending,
            review_summary: None,
            linked_at: chrono::Utc::now().to_rfc3339(),
        };
        db.upsert_commit_link(&link).unwrap();

        let links = db.query_commit_links(Some("run-1"), None, None).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].sha, "abc123");
        assert_eq!(links[0].review_status, ReviewStatus::Pending);
    }

    #[test]
    fn test_update_commit_review() {
        let db = IndexDb::open_in_memory().unwrap();
        let link = CommitLink {
            sha: "abc123".to_string(),
            doc_id: None,
            pipeline_run_id: "run-1".to_string(),
            stage: None,
            skill: None,
            review_status: ReviewStatus::Pending,
            review_summary: None,
            linked_at: chrono::Utc::now().to_rfc3339(),
        };
        db.upsert_commit_link(&link).unwrap();

        db.update_commit_review("abc123", "run-1", ReviewStatus::Passed, Some("LGTM"))
            .unwrap();

        let links = db.query_commit_links(Some("run-1"), None, None).unwrap();
        assert_eq!(links[0].review_status, ReviewStatus::Passed);
        assert_eq!(links[0].review_summary.as_deref(), Some("LGTM"));
    }
}
