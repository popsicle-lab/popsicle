use rusqlite::{Connection, params};
use rusqlite::OptionalExtension;
use std::path::Path;

use crate::error::Result;
use crate::git::{CommitLink, ReviewStatus};
use crate::model::{
    AcceptanceCriterion, Bug, BugSeverity, BugSource, BugStatus, Discussion, DiscussionMessage,
    DiscussionRole, DiscussionStatus, Document, Issue, IssueStatus, IssueType, MessageType,
    PipelineRun, Priority, Project, ProjectStatus, RoleSource, RunType, StageState, TestCase,
    TestCaseStatus, TestPriority, TestRunResult, TestType, UserStory, UserStoryStatus,
};

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
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                slug TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'active',
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS topics (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                slug TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                doc_type TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                skill_name TEXT NOT NULL,
                pipeline_run_id TEXT NOT NULL,
                topic_id TEXT NOT NULL REFERENCES topics(id),
                version INTEGER NOT NULL DEFAULT 1,
                parent_doc_id TEXT,
                file_path TEXT NOT NULL,
                created_at TEXT,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS pipeline_runs (
                id TEXT PRIMARY KEY,
                pipeline_name TEXT NOT NULL,
                title TEXT NOT NULL,
                stage_states_json TEXT NOT NULL,
                topic_id TEXT NOT NULL REFERENCES topics(id),
                parent_run_id TEXT,
                run_type TEXT NOT NULL DEFAULT 'new',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_doc_skill ON documents(skill_name);
            CREATE INDEX IF NOT EXISTS idx_doc_run ON documents(pipeline_run_id);
            CREATE INDEX IF NOT EXISTS idx_doc_status ON documents(status);
            CREATE INDEX IF NOT EXISTS idx_doc_topic ON documents(topic_id);
            CREATE INDEX IF NOT EXISTS idx_run_topic ON pipeline_runs(topic_id);

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

            CREATE TABLE IF NOT EXISTS discussions (
                id TEXT PRIMARY KEY,
                document_id TEXT,
                skill TEXT NOT NULL,
                pipeline_run_id TEXT NOT NULL,
                topic TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                user_confidence INTEGER,
                created_at TEXT NOT NULL,
                concluded_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_disc_run ON discussions(pipeline_run_id);
            CREATE INDEX IF NOT EXISTS idx_disc_skill ON discussions(skill);
            CREATE INDEX IF NOT EXISTS idx_disc_doc ON discussions(document_id);

            CREATE TABLE IF NOT EXISTS discussion_messages (
                id TEXT PRIMARY KEY,
                discussion_id TEXT NOT NULL REFERENCES discussions(id),
                phase TEXT NOT NULL,
                role_id TEXT NOT NULL,
                role_name TEXT NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                reply_to TEXT,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_dm_disc ON discussion_messages(discussion_id);

            CREATE TABLE IF NOT EXISTS discussion_roles (
                discussion_id TEXT NOT NULL REFERENCES discussions(id),
                role_id TEXT NOT NULL,
                role_name TEXT NOT NULL,
                perspective TEXT,
                source TEXT NOT NULL,
                PRIMARY KEY (discussion_id, role_id)
            );

            CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                issue_type TEXT NOT NULL,
                priority TEXT NOT NULL DEFAULT 'medium',
                status TEXT NOT NULL DEFAULT 'backlog',
                pipeline TEXT,
                topic_id TEXT NOT NULL DEFAULT '' REFERENCES topics(id),
                labels TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_issue_key ON issues(key);
            CREATE INDEX IF NOT EXISTS idx_issue_status ON issues(status);
            CREATE INDEX IF NOT EXISTS idx_issue_type ON issues(issue_type);
            CREATE INDEX IF NOT EXISTS idx_issue_topic ON issues(topic_id);

            CREATE TABLE IF NOT EXISTS bugs (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                severity TEXT NOT NULL DEFAULT 'major',
                priority TEXT NOT NULL DEFAULT 'medium',
                status TEXT NOT NULL DEFAULT 'open',
                steps_to_reproduce TEXT NOT NULL DEFAULT '[]',
                expected_behavior TEXT NOT NULL DEFAULT '',
                actual_behavior TEXT NOT NULL DEFAULT '',
                environment TEXT,
                stack_trace TEXT,
                source TEXT NOT NULL DEFAULT 'manual',
                related_test_case_id TEXT,
                related_commit_sha TEXT,
                fix_commit_sha TEXT,
                issue_id TEXT,
                pipeline_run_id TEXT,
                labels TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_bug_key ON bugs(key);
            CREATE INDEX IF NOT EXISTS idx_bug_status ON bugs(status);
            CREATE INDEX IF NOT EXISTS idx_bug_severity ON bugs(severity);
            CREATE INDEX IF NOT EXISTS idx_bug_issue ON bugs(issue_id);
            CREATE INDEX IF NOT EXISTS idx_bug_run ON bugs(pipeline_run_id);

            CREATE TABLE IF NOT EXISTS test_cases (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                test_type TEXT NOT NULL DEFAULT 'unit',
                priority_level TEXT NOT NULL DEFAULT 'p1',
                status TEXT NOT NULL DEFAULT 'draft',
                preconditions TEXT NOT NULL DEFAULT '[]',
                steps TEXT NOT NULL DEFAULT '[]',
                expected_result TEXT NOT NULL DEFAULT '',
                source_doc_id TEXT,
                user_story_id TEXT,
                issue_id TEXT,
                pipeline_run_id TEXT,
                labels TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tc_key ON test_cases(key);
            CREATE INDEX IF NOT EXISTS idx_tc_type ON test_cases(test_type);
            CREATE INDEX IF NOT EXISTS idx_tc_priority ON test_cases(priority_level);
            CREATE INDEX IF NOT EXISTS idx_tc_status ON test_cases(status);
            CREATE INDEX IF NOT EXISTS idx_tc_story ON test_cases(user_story_id);
            CREATE INDEX IF NOT EXISTS idx_tc_run ON test_cases(pipeline_run_id);

            CREATE TABLE IF NOT EXISTS test_runs (
                id TEXT PRIMARY KEY,
                test_case_id TEXT NOT NULL,
                passed INTEGER NOT NULL,
                duration_ms INTEGER,
                error_message TEXT,
                commit_sha TEXT,
                run_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tr_tc ON test_runs(test_case_id);

            CREATE TABLE IF NOT EXISTS user_stories (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                persona TEXT NOT NULL DEFAULT '',
                goal TEXT NOT NULL DEFAULT '',
                benefit TEXT NOT NULL DEFAULT '',
                priority TEXT NOT NULL DEFAULT 'medium',
                status TEXT NOT NULL DEFAULT 'draft',
                source_doc_id TEXT,
                issue_id TEXT,
                pipeline_run_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_us_key ON user_stories(key);
            CREATE INDEX IF NOT EXISTS idx_us_status ON user_stories(status);
            CREATE INDEX IF NOT EXISTS idx_us_issue ON user_stories(issue_id);
            CREATE INDEX IF NOT EXISTS idx_us_run ON user_stories(pipeline_run_id);

            CREATE TABLE IF NOT EXISTS acceptance_criteria (
                id TEXT PRIMARY KEY,
                user_story_id TEXT NOT NULL,
                description TEXT NOT NULL,
                verified INTEGER NOT NULL DEFAULT 0,
                test_case_ids TEXT NOT NULL DEFAULT '[]'
            );

            CREATE INDEX IF NOT EXISTS idx_ac_story ON acceptance_criteria(user_story_id);
            ",
        )?;

        // Migration: add `pipeline` column to existing issues tables
        let has_pipeline_col: bool = self
            .conn
            .prepare("SELECT pipeline FROM issues LIMIT 0")
            .is_ok();
        if !has_pipeline_col {
            self.conn
                .execute_batch("ALTER TABLE issues ADD COLUMN pipeline TEXT;")?;
        }

        // Migration: add `summary` and `doc_tags` columns to documents table
        let has_summary_col: bool = self
            .conn
            .prepare("SELECT summary FROM documents LIMIT 0")
            .is_ok();
        if !has_summary_col {
            self.conn.execute_batch(
                "ALTER TABLE documents ADD COLUMN summary TEXT DEFAULT '';
                 ALTER TABLE documents ADD COLUMN doc_tags TEXT DEFAULT '[]';",
            )?;
        }

        // Migration: add project_id column to topics
        let has_project_id: bool = self
            .conn
            .prepare("SELECT project_id FROM topics LIMIT 0")
            .is_ok();
        if !has_project_id {
            self.conn
                .execute_batch("ALTER TABLE topics ADD COLUMN project_id TEXT REFERENCES projects(id);")?;
        }

        // Migration: add topic lock columns
        let has_topic_lock: bool = self
            .conn
            .prepare("SELECT locked_by_run_id FROM topics LIMIT 0")
            .is_ok();
        if !has_topic_lock {
            self.conn.execute_batch(
                "ALTER TABLE topics ADD COLUMN locked_by_run_id TEXT REFERENCES pipeline_runs(id);
                 ALTER TABLE topics ADD COLUMN locked_at TEXT;",
            )?;
        }

        // Migration: add issue_id column to pipeline_runs
        let has_run_issue_id: bool = self
            .conn
            .prepare("SELECT issue_id FROM pipeline_runs LIMIT 0")
            .is_ok();
        if !has_run_issue_id {
            self.conn
                .execute_batch("ALTER TABLE pipeline_runs ADD COLUMN issue_id TEXT REFERENCES issues(id);
                                CREATE INDEX IF NOT EXISTS idx_run_issue ON pipeline_runs(issue_id);")?;
        }

        // Migration: add topic_id column to issues (for existing DBs)
        let has_issue_topic_id: bool = self
            .conn
            .prepare("SELECT topic_id FROM issues LIMIT 0")
            .is_ok();
        if !has_issue_topic_id {
            self.conn
                .execute_batch("ALTER TABLE issues ADD COLUMN topic_id TEXT NOT NULL DEFAULT '' REFERENCES topics(id);
                                CREATE INDEX IF NOT EXISTS idx_issue_topic ON issues(topic_id);")?;
        }

        // Migration: backfill issue.topic_id from pipeline_runs and move pipeline_run_id to pipeline_runs.issue_id
        let _ = self.conn.execute_batch(
            "UPDATE pipeline_runs SET issue_id = (
                SELECT id FROM issues WHERE issues.pipeline_run_id = pipeline_runs.id
             ) WHERE issue_id IS NULL AND EXISTS (
                SELECT 1 FROM issues WHERE issues.pipeline_run_id = pipeline_runs.id
             );

             UPDATE issues SET topic_id = (
                SELECT topic_id FROM pipeline_runs WHERE pipeline_runs.id = issues.pipeline_run_id
             ) WHERE topic_id = '' AND pipeline_run_id IS NOT NULL AND pipeline_run_id != '';",
        );

        // FTS5 virtual table for full-text search on documents
        self.conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                title,
                summary,
                doc_tags,
                content=documents,
                content_rowid=rowid
            );

            CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, title, summary, doc_tags)
                VALUES (new.rowid, new.title, new.summary, new.doc_tags);
            END;

            CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, summary, doc_tags)
                VALUES ('delete', old.rowid, old.title, old.summary, old.doc_tags);
                INSERT INTO documents_fts(rowid, title, summary, doc_tags)
                VALUES (new.rowid, new.title, new.summary, new.doc_tags);
            END;

            CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, summary, doc_tags)
                VALUES ('delete', old.rowid, old.title, old.summary, old.doc_tags);
            END;",
        )?;

        Ok(())
    }

    /// Upsert a document's metadata into the index.
    pub fn upsert_document(&self, doc: &Document) -> Result<()> {
        let tags_json = serde_json::to_string(&doc.tags).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "INSERT INTO documents (id, doc_type, title, status, skill_name, pipeline_run_id, topic_id, version, parent_doc_id, file_path, created_at, updated_at, summary, doc_tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                title = excluded.title,
                file_path = excluded.file_path,
                version = excluded.version,
                parent_doc_id = excluded.parent_doc_id,
                updated_at = excluded.updated_at,
                summary = CASE WHEN excluded.summary != '' THEN excluded.summary ELSE documents.summary END,
                doc_tags = CASE WHEN excluded.doc_tags != '[]' THEN excluded.doc_tags ELSE documents.doc_tags END",
            params![
                doc.id,
                doc.doc_type,
                doc.title,
                doc.status,
                doc.skill_name,
                doc.pipeline_run_id,
                doc.topic_id,
                doc.version,
                doc.parent_doc_id,
                doc.file_path.display().to_string(),
                doc.created_at.map(|t| t.to_rfc3339()),
                doc.updated_at.map(|t| t.to_rfc3339()),
                doc.summary,
                tags_json,
            ],
        )?;
        Ok(())
    }

    /// Update summary and tags for a document.
    pub fn update_document_summary(
        &self,
        doc_id: &str,
        summary: &str,
        tags: &[String],
    ) -> Result<()> {
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "UPDATE documents SET summary = ?1, doc_tags = ?2 WHERE id = ?3",
            params![summary, tags_json, doc_id],
        )?;
        Ok(())
    }

    /// Update only the status of a document.
    pub fn update_document_status(&self, doc_id: &str, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE documents SET status = ?1 WHERE id = ?2",
            params![status, doc_id],
        )?;
        Ok(())
    }

    /// Full-text search across documents using FTS5.
    ///
    /// Returns documents matching the query, optionally filtered by status,
    /// skill, and excluding a specific pipeline run.
    pub fn search_documents(
        &self,
        query: &str,
        status_filter: Option<&str>,
        skill_filter: Option<&str>,
        exclude_run: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(DocumentRow, f64)>> {
        let mut sql = String::from(
            "SELECT d.id, d.doc_type, d.title, d.status, d.skill_name, d.pipeline_run_id,
                    d.topic_id, d.version, d.parent_doc_id,
                    d.file_path, d.created_at, d.updated_at,
                    COALESCE(d.summary, '') AS summary, COALESCE(d.doc_tags, '[]') AS doc_tags,
                    bm25(documents_fts) AS rank
             FROM documents_fts
             JOIN documents d ON d.rowid = documents_fts.rowid
             WHERE documents_fts MATCH ?1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        param_values.push(Box::new(query.to_string()));

        if let Some(s) = status_filter {
            sql.push_str(&format!(" AND d.status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(s) = skill_filter {
            sql.push_str(&format!(" AND d.skill_name = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(r) = exclude_run {
            sql.push_str(&format!(
                " AND d.pipeline_run_id != ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(r.to_string()));
        }

        sql.push_str(&format!(" ORDER BY rank LIMIT ?{}", param_values.len() + 1));
        param_values.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let rank: f64 = row.get(14)?;
            Ok((
                DocumentRow {
                    id: row.get(0)?,
                    doc_type: row.get(1)?,
                    title: row.get(2)?,
                    status: row.get(3)?,
                    skill_name: row.get(4)?,
                    pipeline_run_id: row.get(5)?,
                    topic_id: row.get(6)?,
                    version: row.get(7)?,
                    parent_doc_id: row.get(8)?,
                    file_path: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    summary: row.get(12)?,
                    doc_tags: row.get(13)?,
                },
                rank,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query documents by optional filters.
    pub fn query_documents(
        &self,
        skill: Option<&str>,
        status: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<Vec<DocumentRow>> {
        let mut sql = "SELECT id, doc_type, title, status, skill_name, pipeline_run_id, topic_id, version, parent_doc_id, file_path, created_at, updated_at, COALESCE(summary, '') AS summary, COALESCE(doc_tags, '[]') AS doc_tags FROM documents WHERE 1=1".to_string();
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
            sql.push_str(&format!(
                " AND pipeline_run_id = ?{}",
                param_values.len() + 1
            ));
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
                topic_id: row.get(6)?,
                version: row.get(7)?,
                parent_doc_id: row.get(8)?,
                file_path: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                summary: row.get(12)?,
                doc_tags: row.get(13)?,
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

        // Store empty issue_id as NULL for FK compatibility
        let issue_id_db: Option<&str> = if run.issue_id.is_empty() {
            None
        } else {
            Some(&run.issue_id)
        };

        self.conn.execute(
            "INSERT INTO pipeline_runs (id, pipeline_name, title, stage_states_json, topic_id, parent_run_id, run_type, created_at, updated_at, issue_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                stage_states_json = excluded.stage_states_json,
                updated_at = excluded.updated_at",
            params![
                run.id,
                run.pipeline_name,
                run.title,
                states_json,
                run.topic_id,
                run.parent_run_id,
                run.run_type.to_string(),
                run.created_at.to_rfc3339(),
                run.updated_at.to_rfc3339(),
                issue_id_db,
            ],
        )?;
        Ok(())
    }

    /// Load a pipeline run by ID.
    pub fn get_pipeline_run(&self, run_id: &str) -> Result<Option<PipelineRun>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipeline_name, title, stage_states_json, topic_id, parent_run_id, run_type, created_at, updated_at, issue_id FROM pipeline_runs WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![run_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;

        if let Some(row) = rows.next() {
            let (
                id,
                pipeline_name,
                title,
                states_json,
                topic_id,
                parent_run_id,
                run_type_str,
                created_str,
                updated_str,
                issue_id_opt,
            ) = row?;
            let stage_states: std::collections::HashMap<String, StageState> =
                serde_json::from_str(&states_json)
                    .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let run_type = match run_type_str.as_str() {
                "revision" => RunType::Revision,
                "continuation" => RunType::Continuation,
                _ => RunType::New,
            };

            Ok(Some(PipelineRun {
                id,
                pipeline_name,
                title,
                stage_states,
                topic_id,
                issue_id: issue_id_opt.unwrap_or_default(),
                parent_run_id,
                run_type,
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
            "SELECT id, pipeline_name, title, topic_id, run_type, parent_run_id, created_at, updated_at, issue_id FROM pipeline_runs ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(PipelineRunRow {
                id: row.get(0)?,
                pipeline_name: row.get(1)?,
                title: row.get(2)?,
                topic_id: row.get(3)?,
                run_type: row.get(4)?,
                parent_run_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                issue_id: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // ── Topic CRUD ──────────────────────────────────────────────────

    /// Create a new topic.
    pub fn create_topic(&self, topic: &crate::model::Topic) -> Result<()> {
        let tags_json = serde_json::to_string(&topic.tags).unwrap_or_else(|_| "[]".to_string());
        let project_id_param: Option<&str> = if topic.project_id.is_empty() {
            None
        } else {
            Some(&topic.project_id)
        };
        self.conn.execute(
            "INSERT INTO topics (id, name, slug, description, tags, created_at, updated_at, project_id, locked_by_run_id, locked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                topic.id,
                topic.name,
                topic.slug,
                topic.description,
                tags_json,
                topic.created_at.to_rfc3339(),
                topic.updated_at.to_rfc3339(),
                project_id_param,
                topic.locked_by_run_id.as_deref(),
                topic.locked_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    /// Get a topic by ID.
    pub fn get_topic(&self, id: &str) -> Result<Option<crate::model::Topic>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, slug, description, tags, created_at, updated_at, project_id, locked_by_run_id, locked_at FROM topics WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;
        Self::parse_topic_row(rows.next())
    }

    /// Find a topic by name.
    pub fn find_topic_by_name(&self, name: &str) -> Result<Option<crate::model::Topic>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, slug, description, tags, created_at, updated_at, project_id, locked_by_run_id, locked_at FROM topics WHERE name = ?1",
        )?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;
        Self::parse_topic_row(rows.next())
    }

    #[allow(clippy::type_complexity)]
    fn parse_topic_row(
        row: Option<
            std::result::Result<
                (String, String, String, String, String, String, String, Option<String>, Option<String>, Option<String>),
                rusqlite::Error,
            >,
        >,
    ) -> Result<Option<crate::model::Topic>> {
        match row {
            Some(Ok((id, name, slug, description, tags_json, created_str, updated_str, project_id, locked_by_run_id, locked_at_str))) => {
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                    .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                    .with_timezone(&chrono::Utc);
                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                    .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                    .with_timezone(&chrono::Utc);
                let locked_at = locked_at_str
                    .as_deref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));
                Ok(Some(crate::model::Topic {
                    id,
                    name,
                    slug,
                    description,
                    project_id: project_id.unwrap_or_default(),
                    tags,
                    locked_by_run_id,
                    locked_at,
                    created_at,
                    updated_at,
                }))
            }
            Some(Err(e)) => Err(crate::error::PopsicleError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    /// List all topics.
    pub fn list_topics(&self) -> Result<Vec<crate::model::Topic>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, slug, description, tags, created_at, updated_at, project_id, locked_by_run_id, locked_at FROM topics ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id, name, slug, description, tags_json, created_str, updated_str, project_id, locked_by_run_id, locked_at_str) = row?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let locked_at = locked_at_str
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            results.push(crate::model::Topic {
                id,
                name,
                slug,
                description,
                project_id: project_id.unwrap_or_default(),
                tags,
                locked_by_run_id,
                locked_at,
                created_at,
                updated_at,
            });
        }
        Ok(results)
    }

    /// Find topics whose tags overlap with the given keywords.
    /// Returns topics sorted by match score (number of matching tags, descending).
    pub fn match_topics_by_tags(&self, keywords: &[String]) -> Result<Vec<(crate::model::Topic, usize)>> {
        if keywords.is_empty() {
            return Ok(vec![]);
        }
        let topics = self.list_topics()?;
        let mut scored: Vec<(crate::model::Topic, usize)> = topics
            .into_iter()
            .filter_map(|t| {
                if t.tags.is_empty() {
                    return None;
                }
                let score = t.tags.iter()
                    .filter(|tag| keywords.iter().any(|kw| {
                        let tag_lower = tag.to_lowercase();
                        let kw_lower = kw.to_lowercase();
                        tag_lower.contains(&kw_lower) || kw_lower.contains(&tag_lower)
                    }))
                    .count();
                if score > 0 { Some((t, score)) } else { None }
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(scored)
    }

    /// Delete a topic by ID.
    pub fn delete_topic(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM topics WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Atomically acquire an exclusive lock on a topic for the given run.
    /// Returns Ok(true) if lock acquired, Ok(false) if already locked by another run.
    pub fn acquire_topic_lock(&self, topic_id: &str, run_id: &str) -> Result<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE topics SET locked_by_run_id = ?2, locked_at = ?3, updated_at = ?3
             WHERE id = ?1 AND (locked_by_run_id IS NULL OR locked_by_run_id = ?2)",
            params![topic_id, run_id, now],
        )?;
        Ok(rows > 0)
    }

    /// Release the lock on a topic. If `run_id` is Some, only release if held by that run.
    pub fn release_topic_lock(&self, topic_id: &str, run_id: Option<&str>) -> Result<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        let rows = match run_id {
            Some(rid) => self.conn.execute(
                "UPDATE topics SET locked_by_run_id = NULL, locked_at = NULL, updated_at = ?3
                 WHERE id = ?1 AND locked_by_run_id = ?2",
                params![topic_id, rid, now],
            )?,
            None => self.conn.execute(
                "UPDATE topics SET locked_by_run_id = NULL, locked_at = NULL, updated_at = ?2
                 WHERE id = ?1",
                params![topic_id, now],
            )?,
        };
        Ok(rows > 0)
    }

    /// Check if a topic is locked and return the locking run ID if so.
    pub fn get_topic_lock(&self, topic_id: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT locked_by_run_id FROM topics WHERE id = ?1",
        )?;
        let result: Option<Option<String>> = stmt
            .query_row(params![topic_id], |row| row.get::<_, Option<String>>(0))
            .ok();
        Ok(result.flatten())
    }

    // ── Project CRUD ─────────────────────────────────────────────────

    pub fn create_project(&self, project: &Project) -> Result<()> {
        let tags_json = serde_json::to_string(&project.tags).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "INSERT INTO projects (id, name, slug, description, status, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                project.id,
                project.name,
                project.slug,
                project.description,
                project.status.to_string(),
                tags_json,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_project(&self, id_or_name: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, slug, description, status, tags, created_at, updated_at FROM projects WHERE id = ?1 OR name = ?1",
        )?;
        let mut rows = stmt.query_map(params![id_or_name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;
        match rows.next() {
            Some(Ok((id, name, slug, description, status_str, tags_json, created_str, updated_str))) => {
                let status: ProjectStatus = status_str
                    .parse()
                    .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                    .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                    .with_timezone(&chrono::Utc);
                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                    .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                    .with_timezone(&chrono::Utc);
                Ok(Some(Project {
                    id,
                    name,
                    slug,
                    description,
                    status,
                    tags,
                    created_at,
                    updated_at,
                }))
            }
            Some(Err(e)) => Err(crate::error::PopsicleError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    pub fn list_projects(&self, status: Option<&str>) -> Result<Vec<Project>> {
        let mut sql = "SELECT id, name, slug, description, status, tags, created_at, updated_at FROM projects".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(s) = status {
            sql.push_str(&format!(" WHERE status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC");

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id, name, slug, description, status_str, tags_json, created_str, updated_str) = row?;
            let status: ProjectStatus = status_str
                .parse()
                .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            results.push(Project {
                id,
                name,
                slug,
                description,
                status,
                tags,
                created_at,
                updated_at,
            });
        }
        Ok(results)
    }

    pub fn update_project(&self, project: &Project) -> Result<()> {
        let tags_json = serde_json::to_string(&project.tags).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "UPDATE projects SET name=?1, slug=?2, description=?3, status=?4, tags=?5, updated_at=?6 WHERE id=?7",
            params![
                project.name,
                project.slug,
                project.description,
                project.status.to_string(),
                tags_json,
                chrono::Utc::now().to_rfc3339(),
                project.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn find_project_by_name(&self, name: &str) -> Result<Option<Project>> {
        self.get_project(name)
    }

    /// List topics optionally filtered by project.
    pub fn list_topics_by_project(&self, project_id: Option<&str>) -> Result<Vec<crate::model::Topic>> {
        match project_id {
            Some(pid) => self.list_project_topics(pid),
            None => self.list_topics(),
        }
    }

    pub fn list_project_topics(&self, project_id: &str) -> Result<Vec<crate::model::Topic>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, slug, description, tags, created_at, updated_at, project_id, locked_by_run_id, locked_at FROM topics WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id, name, slug, description, tags_json, created_str, updated_str, project_id, locked_by_run_id, locked_at_str) = row?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let locked_at = locked_at_str
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            results.push(crate::model::Topic {
                id,
                name,
                slug,
                description,
                project_id: project_id.unwrap_or_default(),
                tags,
                locked_by_run_id,
                locked_at,
                created_at,
                updated_at,
            });
        }
        Ok(results)
    }

    /// Query documents across a topic (all runs), returning the latest version of each artifact type.
    pub fn query_topic_documents(&self, topic_id: &str) -> Result<Vec<DocumentRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, doc_type, title, status, skill_name, pipeline_run_id, topic_id, version, parent_doc_id, file_path, created_at, updated_at, COALESCE(summary, '') AS summary, COALESCE(doc_tags, '[]') AS doc_tags
             FROM documents
             WHERE topic_id = ?1
             ORDER BY doc_type, version DESC",
        )?;
        let rows = stmt.query_map(params![topic_id], |row| {
            Ok(DocumentRow {
                id: row.get(0)?,
                doc_type: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                skill_name: row.get(4)?,
                pipeline_run_id: row.get(5)?,
                topic_id: row.get(6)?,
                version: row.get(7)?,
                parent_doc_id: row.get(8)?,
                file_path: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                summary: row.get(12)?,
                doc_tags: row.get(13)?,
            })
        })?;
        // Deduplicate: keep highest version per (doc_type, skill_name)
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for row in rows {
            let r = row?;
            let key = (r.doc_type.clone(), r.skill_name.clone());
            if seen.insert(key) {
                results.push(r);
            }
        }
        Ok(results)
    }

    /// Query all documents in a topic for a specific skill, across all pipeline runs.
    /// Returns docs ordered by version DESC (latest first).
    pub fn query_topic_skill_docs(&self, topic_id: &str, skill_name: &str) -> Result<Vec<DocumentRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, doc_type, title, status, skill_name, pipeline_run_id, topic_id, version, parent_doc_id, file_path, created_at, updated_at, COALESCE(summary, '') AS summary, COALESCE(doc_tags, '[]') AS doc_tags \
             FROM documents \
             WHERE topic_id = ?1 AND skill_name = ?2 \
             ORDER BY version DESC",
        )?;
        let rows = stmt.query_map(params![topic_id, skill_name], |row| {
            Ok(DocumentRow {
                id: row.get(0)?,
                doc_type: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                skill_name: row.get(4)?,
                pipeline_run_id: row.get(5)?,
                topic_id: row.get(6)?,
                version: row.get(7)?,
                parent_doc_id: row.get(8)?,
                file_path: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                summary: row.get(12)?,
                doc_tags: row.get(13)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// List pipeline runs for a specific topic.
    pub fn list_topic_runs(&self, topic_id: &str) -> Result<Vec<PipelineRunRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipeline_name, title, topic_id, run_type, parent_run_id, created_at, updated_at, issue_id FROM pipeline_runs WHERE topic_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![topic_id], |row| {
            Ok(PipelineRunRow {
                id: row.get(0)?,
                pipeline_name: row.get(1)?,
                title: row.get(2)?,
                topic_id: row.get(3)?,
                run_type: row.get(4)?,
                parent_run_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                issue_id: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn find_runs_by_issue(&self, issue_id: &str) -> Result<Vec<PipelineRunRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipeline_name, title, topic_id, run_type, parent_run_id, created_at, updated_at, issue_id FROM pipeline_runs WHERE issue_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![issue_id], |row| {
            Ok(PipelineRunRow {
                id: row.get(0)?,
                pipeline_name: row.get(1)?,
                title: row.get(2)?,
                topic_id: row.get(3)?,
                run_type: row.get(4)?,
                parent_run_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                issue_id: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
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
            sql.push_str(&format!(
                " AND pipeline_run_id = ?{}",
                param_values.len() + 1
            ));
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

    // ── Discussion operations ──

    pub fn upsert_discussion(&self, disc: &Discussion) -> Result<()> {
        self.conn.execute(
            "INSERT INTO discussions (id, document_id, skill, pipeline_run_id, topic, status, user_confidence, created_at, concluded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                document_id = excluded.document_id,
                status = excluded.status,
                user_confidence = excluded.user_confidence,
                concluded_at = excluded.concluded_at",
            params![
                disc.id,
                disc.document_id,
                disc.skill,
                disc.pipeline_run_id,
                disc.topic,
                disc.status.to_string(),
                disc.user_confidence,
                disc.created_at.to_rfc3339(),
                disc.concluded_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get_discussion(&self, id: &str) -> Result<Option<Discussion>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, document_id, skill, pipeline_run_id, topic, status, user_confidence, created_at, concluded_at
             FROM discussions WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(DiscussionRow {
                id: row.get(0)?,
                document_id: row.get(1)?,
                skill: row.get(2)?,
                pipeline_run_id: row.get(3)?,
                topic: row.get(4)?,
                status: row.get(5)?,
                user_confidence: row.get(6)?,
                created_at: row.get(7)?,
                concluded_at: row.get(8)?,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(discussion_from_row(row?)?))
        } else {
            Ok(None)
        }
    }

    pub fn query_discussions(
        &self,
        run_id: Option<&str>,
        skill: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<Discussion>> {
        let mut sql = "SELECT id, document_id, skill, pipeline_run_id, topic, status, user_confidence, created_at, concluded_at FROM discussions WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(r) = run_id {
            sql.push_str(&format!(
                " AND pipeline_run_id = ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(r.to_string()));
        }
        if let Some(s) = skill {
            sql.push_str(&format!(" AND skill = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(st) = status {
            sql.push_str(&format!(" AND status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(st.to_string()));
        }

        sql.push_str(" ORDER BY created_at DESC");

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(DiscussionRow {
                id: row.get(0)?,
                document_id: row.get(1)?,
                skill: row.get(2)?,
                pipeline_run_id: row.get(3)?,
                topic: row.get(4)?,
                status: row.get(5)?,
                user_confidence: row.get(6)?,
                created_at: row.get(7)?,
                concluded_at: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(discussion_from_row(row?)?);
        }
        Ok(results)
    }

    pub fn insert_discussion_message(&self, msg: &DiscussionMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO discussion_messages (id, discussion_id, phase, role_id, role_name, content, message_type, reply_to, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                msg.id,
                msg.discussion_id,
                msg.phase,
                msg.role_id,
                msg.role_name,
                msg.content,
                msg.message_type.to_string(),
                msg.reply_to,
                msg.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_discussion_messages(&self, discussion_id: &str) -> Result<Vec<DiscussionMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, discussion_id, phase, role_id, role_name, content, message_type, reply_to, timestamp
             FROM discussion_messages WHERE discussion_id = ?1 ORDER BY timestamp ASC",
        )?;
        let rows = stmt.query_map(params![discussion_id], |row| {
            let msg_type_str: String = row.get(6)?;
            let ts_str: String = row.get(8)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                msg_type_str,
                row.get::<_, Option<String>>(7)?,
                ts_str,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (id, disc_id, phase, role_id, role_name, content, msg_type_str, reply_to, ts_str) =
                row?;
            let message_type: MessageType = msg_type_str
                .parse()
                .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
            let timestamp = chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
                .with_timezone(&chrono::Utc);
            results.push(DiscussionMessage {
                id,
                discussion_id: disc_id,
                phase,
                role_id,
                role_name,
                content,
                message_type,
                reply_to,
                timestamp,
            });
        }
        Ok(results)
    }

    pub fn upsert_discussion_role(&self, role: &DiscussionRole) -> Result<()> {
        self.conn.execute(
            "INSERT INTO discussion_roles (discussion_id, role_id, role_name, perspective, source)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(discussion_id, role_id) DO UPDATE SET
                role_name = excluded.role_name,
                perspective = excluded.perspective,
                source = excluded.source",
            params![
                role.discussion_id,
                role.role_id,
                role.role_name,
                role.perspective,
                role.source.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn get_discussion_roles(&self, discussion_id: &str) -> Result<Vec<DiscussionRole>> {
        let mut stmt = self.conn.prepare(
            "SELECT discussion_id, role_id, role_name, perspective, source
             FROM discussion_roles WHERE discussion_id = ?1",
        )?;
        let rows = stmt.query_map(params![discussion_id], |row| {
            let source_str: String = row.get(4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                source_str,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (disc_id, role_id, role_name, perspective, source_str) = row?;
            let source: RoleSource = source_str
                .parse()
                .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
            results.push(DiscussionRole {
                discussion_id: disc_id,
                role_id,
                role_name,
                perspective,
                source,
            });
        }
        Ok(results)
    }

    // ── Issue methods ──

    pub fn next_issue_seq(&self, prefix: &str) -> Result<u32> {
        let pattern = format!("{}-", prefix);
        let max_seq: Option<u32> = self
            .conn
            .query_row(
                "SELECT MAX(CAST(SUBSTR(key, ?1) AS INTEGER)) FROM issues WHERE key LIKE ?2",
                params![pattern.len() + 1, format!("{}%", pattern)],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        Ok(max_seq.unwrap_or(0) + 1)
    }

    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        let labels_json = serde_json::to_string(&issue.labels)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "INSERT INTO issues (id, key, title, description, issue_type, priority, status, pipeline, topic_id, labels, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                issue.id,
                issue.key,
                issue.title,
                issue.description,
                issue.issue_type.to_string(),
                issue.priority.to_string(),
                issue.status.to_string(),
                issue.pipeline,
                issue.topic_id,
                labels_json,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_issue(&self, issue: &Issue) -> Result<()> {
        let labels_json = serde_json::to_string(&issue.labels)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "UPDATE issues SET title=?1, description=?2, priority=?3, status=?4, pipeline=?5, topic_id=?6, labels=?7, updated_at=?8 WHERE id=?9",
            params![
                issue.title,
                issue.description,
                issue.priority.to_string(),
                issue.status.to_string(),
                issue.pipeline,
                issue.topic_id,
                labels_json,
                chrono::Utc::now().to_rfc3339(),
                issue.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_issue(&self, id_or_key: &str) -> Result<Option<Issue>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, title, description, issue_type, priority, status, pipeline, topic_id, labels, created_at, updated_at
             FROM issues WHERE id = ?1 OR key = ?1",
        )?;
        let mut rows = stmt.query_map(params![id_or_key], |row| {
            Ok(IssueRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                issue_type: row.get(4)?,
                priority: row.get(5)?,
                status: row.get(6)?,
                pipeline: row.get(7)?,
                topic_id: row.get(8)?,
                labels: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(issue_from_row(row?)?)),
            None => Ok(None),
        }
    }

    pub fn find_issue_by_run_id(&self, run_id: &str) -> Result<Option<Issue>> {
        let issue_id: Option<Option<String>> = self
            .conn
            .query_row(
                "SELECT issue_id FROM pipeline_runs WHERE id = ?1",
                params![run_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        match issue_id.flatten() {
            Some(id) => self.get_issue(&id),
            None => Ok(None),
        }
    }

    pub fn query_issues(
        &self,
        issue_type: Option<&str>,
        status: Option<&str>,
        label: Option<&str>,
        topic_id: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let mut sql = "SELECT id, key, title, description, issue_type, priority, status, pipeline, topic_id, labels, created_at, updated_at FROM issues WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(t) = issue_type {
            sql.push_str(&format!(" AND issue_type = ?{}", param_values.len() + 1));
            param_values.push(Box::new(t.to_string()));
        }
        if let Some(s) = status {
            sql.push_str(&format!(" AND status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(l) = label {
            sql.push_str(&format!(" AND labels LIKE ?{}", param_values.len() + 1));
            param_values.push(Box::new(format!("%\"{}\"%", l)));
        }
        if let Some(tid) = topic_id {
            sql.push_str(&format!(" AND topic_id = ?{}", param_values.len() + 1));
            param_values.push(Box::new(tid.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC");

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(IssueRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                issue_type: row.get(4)?,
                priority: row.get(5)?,
                status: row.get(6)?,
                pipeline: row.get(7)?,
                topic_id: row.get(8)?,
                labels: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(issue_from_row(row?)?);
        }
        Ok(results)
    }

    // ── Bug methods ──

    pub fn next_bug_seq(&self, prefix: &str) -> Result<u32> {
        let pattern = format!("BUG-{}-", prefix);
        let max_seq: Option<u32> = self
            .conn
            .query_row(
                "SELECT MAX(CAST(SUBSTR(key, ?1) AS INTEGER)) FROM bugs WHERE key LIKE ?2",
                params![pattern.len() + 1, format!("{}%", pattern)],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        Ok(max_seq.unwrap_or(0) + 1)
    }

    pub fn create_bug(&self, bug: &Bug) -> Result<()> {
        let labels_json = serde_json::to_string(&bug.labels)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        let steps_json = serde_json::to_string(&bug.steps_to_reproduce)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "INSERT INTO bugs (id, key, title, description, severity, priority, status, steps_to_reproduce, expected_behavior, actual_behavior, environment, stack_trace, source, related_test_case_id, related_commit_sha, fix_commit_sha, issue_id, pipeline_run_id, labels, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                bug.id, bug.key, bug.title, bug.description,
                bug.severity.to_string(), bug.priority.to_string(), bug.status.to_string(),
                steps_json, bug.expected_behavior, bug.actual_behavior,
                bug.environment, bug.stack_trace, bug.source.to_string(),
                bug.related_test_case_id, bug.related_commit_sha, bug.fix_commit_sha,
                bug.issue_id, bug.pipeline_run_id, labels_json,
                bug.created_at.to_rfc3339(), bug.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_bug(&self, bug: &Bug) -> Result<()> {
        let labels_json = serde_json::to_string(&bug.labels)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        let steps_json = serde_json::to_string(&bug.steps_to_reproduce)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "UPDATE bugs SET title=?1, description=?2, severity=?3, priority=?4, status=?5, steps_to_reproduce=?6, expected_behavior=?7, actual_behavior=?8, environment=?9, stack_trace=?10, related_test_case_id=?11, related_commit_sha=?12, fix_commit_sha=?13, issue_id=?14, pipeline_run_id=?15, labels=?16, updated_at=?17 WHERE id=?18",
            params![
                bug.title, bug.description, bug.severity.to_string(), bug.priority.to_string(),
                bug.status.to_string(), steps_json, bug.expected_behavior, bug.actual_behavior,
                bug.environment, bug.stack_trace, bug.related_test_case_id, bug.related_commit_sha,
                bug.fix_commit_sha, bug.issue_id, bug.pipeline_run_id, labels_json,
                chrono::Utc::now().to_rfc3339(), bug.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_bug(&self, id_or_key: &str) -> Result<Option<Bug>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, title, description, severity, priority, status, steps_to_reproduce, expected_behavior, actual_behavior, environment, stack_trace, source, related_test_case_id, related_commit_sha, fix_commit_sha, issue_id, pipeline_run_id, labels, created_at, updated_at
             FROM bugs WHERE id = ?1 OR key = ?1",
        )?;
        let mut rows = stmt.query_map(params![id_or_key], |row| {
            Ok(BugRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                severity: row.get(4)?,
                priority: row.get(5)?,
                status: row.get(6)?,
                steps_to_reproduce: row.get(7)?,
                expected_behavior: row.get(8)?,
                actual_behavior: row.get(9)?,
                environment: row.get(10)?,
                stack_trace: row.get(11)?,
                source: row.get(12)?,
                related_test_case_id: row.get(13)?,
                related_commit_sha: row.get(14)?,
                fix_commit_sha: row.get(15)?,
                issue_id: row.get(16)?,
                pipeline_run_id: row.get(17)?,
                labels: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(bug_from_row(row?)?)),
            None => Ok(None),
        }
    }

    pub fn find_open_bug_by_test_case(&self, test_case_id: &str) -> Result<Option<Bug>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, title, description, severity, priority, status, steps_to_reproduce, expected_behavior, actual_behavior, environment, stack_trace, source, related_test_case_id, related_commit_sha, fix_commit_sha, issue_id, pipeline_run_id, labels, created_at, updated_at
             FROM bugs WHERE related_test_case_id = ?1 AND status IN ('open', 'confirmed', 'in_progress')",
        )?;
        let mut rows = stmt.query_map(params![test_case_id], |row| {
            Ok(BugRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                severity: row.get(4)?,
                priority: row.get(5)?,
                status: row.get(6)?,
                steps_to_reproduce: row.get(7)?,
                expected_behavior: row.get(8)?,
                actual_behavior: row.get(9)?,
                environment: row.get(10)?,
                stack_trace: row.get(11)?,
                source: row.get(12)?,
                related_test_case_id: row.get(13)?,
                related_commit_sha: row.get(14)?,
                fix_commit_sha: row.get(15)?,
                issue_id: row.get(16)?,
                pipeline_run_id: row.get(17)?,
                labels: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(bug_from_row(row?)?)),
            None => Ok(None),
        }
    }

    pub fn query_bugs(
        &self,
        severity: Option<&str>,
        status: Option<&str>,
        issue_id: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<Vec<Bug>> {
        let mut sql = "SELECT id, key, title, description, severity, priority, status, steps_to_reproduce, expected_behavior, actual_behavior, environment, stack_trace, source, related_test_case_id, related_commit_sha, fix_commit_sha, issue_id, pipeline_run_id, labels, created_at, updated_at FROM bugs WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(s) = severity {
            sql.push_str(&format!(" AND severity = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(s) = status {
            sql.push_str(&format!(" AND status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(i) = issue_id {
            sql.push_str(&format!(" AND issue_id = ?{}", param_values.len() + 1));
            param_values.push(Box::new(i.to_string()));
        }
        if let Some(r) = run_id {
            sql.push_str(&format!(
                " AND pipeline_run_id = ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(r.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC");

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(BugRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                severity: row.get(4)?,
                priority: row.get(5)?,
                status: row.get(6)?,
                steps_to_reproduce: row.get(7)?,
                expected_behavior: row.get(8)?,
                actual_behavior: row.get(9)?,
                environment: row.get(10)?,
                stack_trace: row.get(11)?,
                source: row.get(12)?,
                related_test_case_id: row.get(13)?,
                related_commit_sha: row.get(14)?,
                fix_commit_sha: row.get(15)?,
                issue_id: row.get(16)?,
                pipeline_run_id: row.get(17)?,
                labels: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(bug_from_row(row?)?);
        }
        Ok(results)
    }

    // ── TestCase methods ──

    pub fn next_testcase_seq(&self, prefix: &str) -> Result<u32> {
        let pattern = format!("TC-{}-", prefix);
        let max_seq: Option<u32> = self
            .conn
            .query_row(
                "SELECT MAX(CAST(SUBSTR(key, ?1) AS INTEGER)) FROM test_cases WHERE key LIKE ?2",
                params![pattern.len() + 1, format!("{}%", pattern)],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        Ok(max_seq.unwrap_or(0) + 1)
    }

    pub fn create_test_case(&self, tc: &TestCase) -> Result<()> {
        let labels_json = serde_json::to_string(&tc.labels)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        let preconditions_json = serde_json::to_string(&tc.preconditions)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        let steps_json = serde_json::to_string(&tc.steps)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "INSERT INTO test_cases (id, key, title, description, test_type, priority_level, status, preconditions, steps, expected_result, source_doc_id, user_story_id, issue_id, pipeline_run_id, labels, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                tc.id, tc.key, tc.title, tc.description,
                tc.test_type.to_string(), tc.priority_level.to_string(), tc.status.to_string(),
                preconditions_json, steps_json, tc.expected_result,
                tc.source_doc_id, tc.user_story_id, tc.issue_id, tc.pipeline_run_id,
                labels_json, tc.created_at.to_rfc3339(), tc.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_test_case(&self, tc: &TestCase) -> Result<()> {
        let labels_json = serde_json::to_string(&tc.labels)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        let steps_json = serde_json::to_string(&tc.steps)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "UPDATE test_cases SET title=?1, description=?2, test_type=?3, priority_level=?4, status=?5, steps=?6, expected_result=?7, user_story_id=?8, issue_id=?9, labels=?10, updated_at=?11 WHERE id=?12",
            params![
                tc.title, tc.description, tc.test_type.to_string(), tc.priority_level.to_string(),
                tc.status.to_string(), steps_json, tc.expected_result,
                tc.user_story_id, tc.issue_id, labels_json,
                chrono::Utc::now().to_rfc3339(), tc.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_test_case(&self, id_or_key: &str) -> Result<Option<TestCase>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, title, description, test_type, priority_level, status, preconditions, steps, expected_result, source_doc_id, user_story_id, issue_id, pipeline_run_id, labels, created_at, updated_at
             FROM test_cases WHERE id = ?1 OR key = ?1",
        )?;
        let mut rows = stmt.query_map(params![id_or_key], |row| {
            Ok(TestCaseRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                test_type: row.get(4)?,
                priority_level: row.get(5)?,
                status: row.get(6)?,
                preconditions: row.get(7)?,
                steps: row.get(8)?,
                expected_result: row.get(9)?,
                source_doc_id: row.get(10)?,
                user_story_id: row.get(11)?,
                issue_id: row.get(12)?,
                pipeline_run_id: row.get(13)?,
                labels: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(test_case_from_row(row?)?)),
            None => Ok(None),
        }
    }

    pub fn query_test_cases(
        &self,
        test_type: Option<&str>,
        priority: Option<&str>,
        status: Option<&str>,
        user_story_id: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<Vec<TestCase>> {
        let mut sql = "SELECT id, key, title, description, test_type, priority_level, status, preconditions, steps, expected_result, source_doc_id, user_story_id, issue_id, pipeline_run_id, labels, created_at, updated_at FROM test_cases WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(t) = test_type {
            sql.push_str(&format!(" AND test_type = ?{}", param_values.len() + 1));
            param_values.push(Box::new(t.to_string()));
        }
        if let Some(p) = priority {
            sql.push_str(&format!(
                " AND priority_level = ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(p.to_string()));
        }
        if let Some(s) = status {
            sql.push_str(&format!(" AND status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(u) = user_story_id {
            sql.push_str(&format!(" AND user_story_id = ?{}", param_values.len() + 1));
            param_values.push(Box::new(u.to_string()));
        }
        if let Some(r) = run_id {
            sql.push_str(&format!(
                " AND pipeline_run_id = ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(r.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC");

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(TestCaseRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                test_type: row.get(4)?,
                priority_level: row.get(5)?,
                status: row.get(6)?,
                preconditions: row.get(7)?,
                steps: row.get(8)?,
                expected_result: row.get(9)?,
                source_doc_id: row.get(10)?,
                user_story_id: row.get(11)?,
                issue_id: row.get(12)?,
                pipeline_run_id: row.get(13)?,
                labels: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(test_case_from_row(row?)?);
        }
        Ok(results)
    }

    // ── TestRunResult methods ──

    pub fn insert_test_run(&self, tr: &TestRunResult) -> Result<()> {
        self.conn.execute(
            "INSERT INTO test_runs (id, test_case_id, passed, duration_ms, error_message, commit_sha, run_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                tr.id, tr.test_case_id, tr.passed as i32,
                tr.duration_ms.map(|v| v as i64), tr.error_message, tr.commit_sha,
                tr.run_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn query_test_runs(&self, test_case_id: &str) -> Result<Vec<TestRunResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, test_case_id, passed, duration_ms, error_message, commit_sha, run_at
             FROM test_runs WHERE test_case_id = ?1 ORDER BY run_at DESC",
        )?;
        let rows = stmt.query_map(params![test_case_id], |row| {
            Ok(TestRunRow {
                id: row.get(0)?,
                test_case_id: row.get(1)?,
                passed: row.get(2)?,
                duration_ms: row.get(3)?,
                error_message: row.get(4)?,
                commit_sha: row.get(5)?,
                run_at: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(test_run_from_row(row?)?);
        }
        Ok(results)
    }

    pub fn latest_test_run(&self, test_case_id: &str) -> Result<Option<TestRunResult>> {
        let runs = self.query_test_runs(test_case_id)?;
        Ok(runs.into_iter().next())
    }

    // ── UserStory methods ──

    pub fn next_story_seq(&self, prefix: &str) -> Result<u32> {
        let pattern = format!("US-{}-", prefix);
        let max_seq: Option<u32> = self
            .conn
            .query_row(
                "SELECT MAX(CAST(SUBSTR(key, ?1) AS INTEGER)) FROM user_stories WHERE key LIKE ?2",
                params![pattern.len() + 1, format!("{}%", pattern)],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        Ok(max_seq.unwrap_or(0) + 1)
    }

    pub fn create_user_story(&self, story: &UserStory) -> Result<()> {
        self.conn.execute(
            "INSERT INTO user_stories (id, key, title, description, persona, goal, benefit, priority, status, source_doc_id, issue_id, pipeline_run_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                story.id, story.key, story.title, story.description,
                story.persona, story.goal, story.benefit,
                story.priority.to_string(), story.status.to_string(),
                story.source_doc_id, story.issue_id, story.pipeline_run_id,
                story.created_at.to_rfc3339(), story.updated_at.to_rfc3339(),
            ],
        )?;
        for ac in &story.acceptance_criteria {
            self.upsert_acceptance_criterion(&story.id, ac)?;
        }
        Ok(())
    }

    pub fn update_user_story(&self, story: &UserStory) -> Result<()> {
        self.conn.execute(
            "UPDATE user_stories SET title=?1, description=?2, persona=?3, goal=?4, benefit=?5, priority=?6, status=?7, issue_id=?8, updated_at=?9 WHERE id=?10",
            params![
                story.title, story.description, story.persona, story.goal, story.benefit,
                story.priority.to_string(), story.status.to_string(), story.issue_id,
                chrono::Utc::now().to_rfc3339(), story.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_user_story(&self, id_or_key: &str) -> Result<Option<UserStory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, title, description, persona, goal, benefit, priority, status, source_doc_id, issue_id, pipeline_run_id, created_at, updated_at
             FROM user_stories WHERE id = ?1 OR key = ?1",
        )?;
        let mut rows = stmt.query_map(params![id_or_key], |row| {
            Ok(UserStoryRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                persona: row.get(4)?,
                goal: row.get(5)?,
                benefit: row.get(6)?,
                priority: row.get(7)?,
                status: row.get(8)?,
                source_doc_id: row.get(9)?,
                issue_id: row.get(10)?,
                pipeline_run_id: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?;
        match rows.next() {
            Some(row) => {
                let mut story = user_story_from_row(row?)?;
                story.acceptance_criteria = self.get_acceptance_criteria(&story.id)?;
                Ok(Some(story))
            }
            None => Ok(None),
        }
    }

    pub fn query_user_stories(
        &self,
        status: Option<&str>,
        issue_id: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<Vec<UserStory>> {
        let mut sql = "SELECT id, key, title, description, persona, goal, benefit, priority, status, source_doc_id, issue_id, pipeline_run_id, created_at, updated_at FROM user_stories WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(s) = status {
            sql.push_str(&format!(" AND status = ?{}", param_values.len() + 1));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(i) = issue_id {
            sql.push_str(&format!(" AND issue_id = ?{}", param_values.len() + 1));
            param_values.push(Box::new(i.to_string()));
        }
        if let Some(r) = run_id {
            sql.push_str(&format!(
                " AND pipeline_run_id = ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(r.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC");

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(UserStoryRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                persona: row.get(4)?,
                goal: row.get(5)?,
                benefit: row.get(6)?,
                priority: row.get(7)?,
                status: row.get(8)?,
                source_doc_id: row.get(9)?,
                issue_id: row.get(10)?,
                pipeline_run_id: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            let mut story = user_story_from_row(row?)?;
            story.acceptance_criteria = self.get_acceptance_criteria(&story.id)?;
            results.push(story);
        }
        Ok(results)
    }

    pub fn upsert_acceptance_criterion(
        &self,
        story_id: &str,
        ac: &AcceptanceCriterion,
    ) -> Result<()> {
        let tc_ids_json = serde_json::to_string(&ac.test_case_ids)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "INSERT INTO acceptance_criteria (id, user_story_id, description, verified, test_case_ids)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                description = excluded.description,
                verified = excluded.verified,
                test_case_ids = excluded.test_case_ids",
            params![ac.id, story_id, ac.description, ac.verified as i32, tc_ids_json],
        )?;
        Ok(())
    }

    pub fn get_acceptance_criteria(&self, story_id: &str) -> Result<Vec<AcceptanceCriterion>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, description, verified, test_case_ids FROM acceptance_criteria WHERE user_story_id = ?1",
        )?;
        let rows = stmt.query_map(params![story_id], |row| {
            let tc_ids_str: String = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                tc_ids_str,
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id, description, verified, tc_ids_str) = row?;
            let test_case_ids: Vec<String> = serde_json::from_str(&tc_ids_str).unwrap_or_default();
            results.push(AcceptanceCriterion {
                id,
                description,
                verified: verified != 0,
                test_case_ids,
            });
        }
        Ok(results)
    }

    pub fn link_ac_to_test_case(&self, ac_id: &str, test_case_id: &str) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT test_case_ids FROM acceptance_criteria WHERE id = ?1")?;
        let tc_ids_str: String = stmt
            .query_row(params![ac_id], |row| row.get(0))
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        let mut tc_ids: Vec<String> = serde_json::from_str(&tc_ids_str).unwrap_or_default();
        if !tc_ids.contains(&test_case_id.to_string()) {
            tc_ids.push(test_case_id.to_string());
        }
        let tc_ids_json = serde_json::to_string(&tc_ids)
            .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?;
        self.conn.execute(
            "UPDATE acceptance_criteria SET test_case_ids = ?1 WHERE id = ?2",
            params![tc_ids_json, ac_id],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct DiscussionRow {
    id: String,
    document_id: Option<String>,
    skill: String,
    pipeline_run_id: String,
    topic: String,
    status: String,
    user_confidence: Option<i32>,
    created_at: String,
    concluded_at: Option<String>,
}

fn discussion_from_row(row: DiscussionRow) -> Result<Discussion> {
    let status: DiscussionStatus = row
        .status
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let concluded_at = row
        .concluded_at
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|t| t.with_timezone(&chrono::Utc))
                .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))
        })
        .transpose()?;

    Ok(Discussion {
        id: row.id,
        document_id: row.document_id,
        skill: row.skill,
        pipeline_run_id: row.pipeline_run_id,
        topic: row.topic,
        status,
        user_confidence: row.user_confidence,
        created_at,
        concluded_at,
    })
}

#[derive(Debug, Clone)]
struct IssueRow {
    id: String,
    key: String,
    title: String,
    description: String,
    issue_type: String,
    priority: String,
    status: String,
    pipeline: Option<String>,
    topic_id: String,
    labels: String,
    created_at: String,
    updated_at: String,
}

fn issue_from_row(row: IssueRow) -> Result<Issue> {
    let issue_type: IssueType = row
        .issue_type
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let priority: Priority = row
        .priority
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let status: IssueStatus = row
        .status
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let labels: Vec<String> = serde_json::from_str(&row.labels).unwrap_or_default();

    Ok(Issue {
        id: row.id,
        key: row.key,
        title: row.title,
        description: row.description,
        issue_type,
        priority,
        status,
        topic_id: row.topic_id,
        pipeline: row.pipeline,
        labels,
        created_at,
        updated_at,
    })
}

#[derive(Debug, Clone)]
struct BugRow {
    id: String,
    key: String,
    title: String,
    description: String,
    severity: String,
    priority: String,
    status: String,
    steps_to_reproduce: String,
    expected_behavior: String,
    actual_behavior: String,
    environment: Option<String>,
    stack_trace: Option<String>,
    source: String,
    related_test_case_id: Option<String>,
    related_commit_sha: Option<String>,
    fix_commit_sha: Option<String>,
    issue_id: Option<String>,
    pipeline_run_id: Option<String>,
    labels: String,
    created_at: String,
    updated_at: String,
}

fn bug_from_row(row: BugRow) -> Result<Bug> {
    let severity: BugSeverity = row
        .severity
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let priority: Priority = row
        .priority
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let status: BugStatus = row
        .status
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let source: BugSource = row
        .source
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let labels: Vec<String> = serde_json::from_str(&row.labels).unwrap_or_default();
    let steps: Vec<String> = serde_json::from_str(&row.steps_to_reproduce).unwrap_or_default();

    Ok(Bug {
        id: row.id,
        key: row.key,
        title: row.title,
        description: row.description,
        severity,
        priority,
        status,
        steps_to_reproduce: steps,
        expected_behavior: row.expected_behavior,
        actual_behavior: row.actual_behavior,
        environment: row.environment,
        stack_trace: row.stack_trace,
        source,
        related_test_case_id: row.related_test_case_id,
        related_commit_sha: row.related_commit_sha,
        fix_commit_sha: row.fix_commit_sha,
        issue_id: row.issue_id,
        pipeline_run_id: row.pipeline_run_id,
        labels,
        created_at,
        updated_at,
    })
}

#[derive(Debug, Clone)]
struct TestCaseRow {
    id: String,
    key: String,
    title: String,
    description: String,
    test_type: String,
    priority_level: String,
    status: String,
    preconditions: String,
    steps: String,
    expected_result: String,
    source_doc_id: Option<String>,
    user_story_id: Option<String>,
    issue_id: Option<String>,
    pipeline_run_id: Option<String>,
    labels: String,
    created_at: String,
    updated_at: String,
}

fn test_case_from_row(row: TestCaseRow) -> Result<TestCase> {
    let test_type: TestType = row
        .test_type
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let priority_level: TestPriority = row
        .priority_level
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let status: TestCaseStatus = row
        .status
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let labels: Vec<String> = serde_json::from_str(&row.labels).unwrap_or_default();
    let preconditions: Vec<String> = serde_json::from_str(&row.preconditions).unwrap_or_default();
    let steps: Vec<String> = serde_json::from_str(&row.steps).unwrap_or_default();

    Ok(TestCase {
        id: row.id,
        key: row.key,
        title: row.title,
        description: row.description,
        test_type,
        priority_level,
        status,
        preconditions,
        steps,
        expected_result: row.expected_result,
        source_doc_id: row.source_doc_id,
        user_story_id: row.user_story_id,
        issue_id: row.issue_id,
        pipeline_run_id: row.pipeline_run_id,
        labels,
        created_at,
        updated_at,
    })
}

#[derive(Debug, Clone)]
struct TestRunRow {
    id: String,
    test_case_id: String,
    passed: i32,
    duration_ms: Option<i64>,
    error_message: Option<String>,
    commit_sha: Option<String>,
    run_at: String,
}

fn test_run_from_row(row: TestRunRow) -> Result<TestRunResult> {
    let run_at = chrono::DateTime::parse_from_rfc3339(&row.run_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    Ok(TestRunResult {
        id: row.id,
        test_case_id: row.test_case_id,
        passed: row.passed != 0,
        duration_ms: row.duration_ms.map(|v| v as u64),
        error_message: row.error_message,
        commit_sha: row.commit_sha,
        run_at,
    })
}

#[derive(Debug, Clone)]
struct UserStoryRow {
    id: String,
    key: String,
    title: String,
    description: String,
    persona: String,
    goal: String,
    benefit: String,
    priority: String,
    status: String,
    source_doc_id: Option<String>,
    issue_id: Option<String>,
    pipeline_run_id: Option<String>,
    created_at: String,
    updated_at: String,
}

fn user_story_from_row(row: UserStoryRow) -> Result<UserStory> {
    let priority: Priority = row
        .priority
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let status: UserStoryStatus = row
        .status
        .parse()
        .map_err(|e: String| crate::error::PopsicleError::Storage(e))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
        .map_err(|e| crate::error::PopsicleError::Storage(e.to_string()))?
        .with_timezone(&chrono::Utc);

    Ok(UserStory {
        id: row.id,
        key: row.key,
        title: row.title,
        description: row.description,
        persona: row.persona,
        goal: row.goal,
        benefit: row.benefit,
        priority,
        status,
        source_doc_id: row.source_doc_id,
        issue_id: row.issue_id,
        pipeline_run_id: row.pipeline_run_id,
        acceptance_criteria: Vec::new(),
        created_at,
        updated_at,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DocumentRow {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub skill_name: String,
    pub pipeline_run_id: String,
    pub topic_id: String,
    pub version: u32,
    pub parent_doc_id: Option<String>,
    pub file_path: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub summary: String,
    pub doc_tags: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PipelineRunRow {
    pub id: String,
    pub pipeline_name: String,
    pub title: String,
    pub topic_id: String,
    pub run_type: String,
    pub parent_run_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub issue_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{CommitLink, ReviewStatus};
    use crate::model::{PipelineDef, StageDef, Topic};
    use std::path::PathBuf;

    fn seed_test_topic(db: &IndexDb) {
        let mut topic = Topic::new("Test Topic", "Topic for tests", "");
        topic.id = "test-topic".to_string();
        db.create_topic(&topic).unwrap();
    }

    fn make_doc(id: &str, skill: &str, run_id: &str) -> Document {
        Document {
            id: id.to_string(),
            doc_type: "test".to_string(),
            title: format!("Doc {}", id),
            status: "active".to_string(),
            skill_name: skill.to_string(),
            pipeline_run_id: run_id.to_string(),
            topic_id: "test-topic".to_string(),
            version: 1,
            parent_doc_id: None,
            tags: vec![],
            summary: String::new(),
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
                requires_approval: false,
            }],
            keywords: vec![],
            scale: None,
        }
    }

    #[test]
    fn test_document_upsert_and_query() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
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
        seed_test_topic(&db);
        db.upsert_document(&make_doc("d1", "prd", "run-1")).unwrap();
        db.upsert_document(&make_doc("d2", "adr", "run-1")).unwrap();
        db.upsert_document(&make_doc("d3", "prd", "run-2")).unwrap();

        let by_skill = db.query_documents(Some("prd"), None, None).unwrap();
        assert_eq!(by_skill.len(), 2);

        let by_run = db.query_documents(None, None, Some("run-1")).unwrap();
        assert_eq!(by_run.len(), 2);

        let by_both = db
            .query_documents(Some("prd"), None, Some("run-1"))
            .unwrap();
        assert_eq!(by_both.len(), 1);
    }

    #[test]
    fn test_document_upsert_updates_status() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let mut doc = make_doc("d1", "prd", "run-1");
        db.upsert_document(&doc).unwrap();

        doc.status = "final".to_string();
        db.upsert_document(&doc).unwrap();

        let results = db.query_documents(None, None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "final");
    }

    #[test]
    fn test_pipeline_run_roundtrip() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let def = make_pipeline_def();
        let run = PipelineRun::new(&def, "Feature X", "test-topic".to_string(), "");

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
        seed_test_topic(&db);
        let def = make_pipeline_def();

        let run1 = PipelineRun::new(&def, "Run 1", "test-topic".to_string(), "");
        let run2 = PipelineRun::new(&def, "Run 2", "test-topic".to_string(), "");
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

    #[test]
    fn test_update_document_summary() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let doc = make_doc("d1", "rfc-writer", "run-1");
        db.upsert_document(&doc).unwrap();

        db.update_document_summary("d1", "This is a summary", &["rfc".into(), "auth".into()])
            .unwrap();

        let docs = db.query_documents(None, None, Some("run-1")).unwrap();
        assert_eq!(docs[0].summary, "This is a summary");
        assert_eq!(docs[0].doc_tags, r#"["rfc","auth"]"#);
    }

    #[test]
    fn test_upsert_preserves_existing_summary() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let mut doc = make_doc("d1", "rfc-writer", "run-1");
        doc.summary = "Original summary".to_string();
        doc.tags = vec!["original".to_string()];
        db.upsert_document(&doc).unwrap();

        // Upsert again with empty summary — should preserve existing
        let mut doc2 = make_doc("d1", "rfc-writer", "run-1");
        doc2.status = "review".to_string();
        db.upsert_document(&doc2).unwrap();

        let docs = db.query_documents(None, None, Some("run-1")).unwrap();
        assert_eq!(docs[0].status, "review");
        assert_eq!(docs[0].summary, "Original summary");
    }

    #[test]
    fn test_search_documents_fts5() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);

        let mut doc1 = make_doc("d1", "rfc-writer", "run-1");
        doc1.title = "JWT Authentication RFC".to_string();
        db.upsert_document(&doc1).unwrap();
        db.update_document_summary(
            "d1",
            "Design for JWT-based user authentication",
            &["rfc".into(), "auth".into(), "jwt".into()],
        )
        .unwrap();

        let mut doc2 = make_doc("d2", "prd-writer", "run-1");
        doc2.title = "Payment Gateway PRD".to_string();
        db.upsert_document(&doc2).unwrap();
        db.update_document_summary(
            "d2",
            "Product requirements for payment processing",
            &["prd".into(), "payment".into()],
        )
        .unwrap();

        // Search for auth-related docs
        let results = db
            .search_documents("authentication", None, None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "d1");

        // Search for payment-related docs
        let results = db
            .search_documents("payment", None, None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "d2");
    }

    #[test]
    fn test_search_documents_exclude_run() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);

        let mut doc1 = make_doc("d1", "rfc-writer", "run-1");
        doc1.title = "Auth RFC".to_string();
        db.upsert_document(&doc1).unwrap();
        db.update_document_summary("d1", "auth design", &["auth".into()])
            .unwrap();

        let mut doc2 = make_doc("d2", "rfc-writer", "run-2");
        doc2.title = "Auth V2 RFC".to_string();
        db.upsert_document(&doc2).unwrap();
        db.update_document_summary("d2", "auth v2 design", &["auth".into()])
            .unwrap();

        // Exclude run-2, should only find run-1
        let results = db
            .search_documents("auth", None, None, Some("run-2"), 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.pipeline_run_id, "run-1");
    }

    #[test]
    fn test_search_documents_no_results() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let doc = make_doc("d1", "rfc-writer", "run-1");
        db.upsert_document(&doc).unwrap();

        let results = db
            .search_documents("nonexistent", None, None, None, 10)
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_document_summary_and_tags_in_query() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let doc = make_doc("d1", "rfc-writer", "run-1");
        db.upsert_document(&doc).unwrap();

        // Before summary update
        let docs = db.query_documents(None, None, Some("run-1")).unwrap();
        assert_eq!(docs[0].summary, "");
        assert_eq!(docs[0].doc_tags, "[]");

        // After summary update
        db.update_document_summary("d1", "my summary", &["tag1".into()])
            .unwrap();
        let docs = db.query_documents(None, None, Some("run-1")).unwrap();
        assert_eq!(docs[0].summary, "my summary");
        assert!(docs[0].doc_tags.contains("tag1"));
    }

    #[test]
    fn test_llm_summary_overwrites_rule_based() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let doc = make_doc("d1", "rfc-writer", "run-1");
        db.upsert_document(&doc).unwrap();

        // Simulate rule-based fallback
        db.update_document_summary("d1", "rule-based summary", &["rule-tag".into()])
            .unwrap();
        let docs = db.query_documents(None, None, None).unwrap();
        assert_eq!(docs[0].summary, "rule-based summary");

        // Simulate LLM overwrite
        db.update_document_summary(
            "d1",
            "LLM-generated high quality summary of the JWT auth RFC",
            &["jwt".into(), "authentication".into(), "security".into()],
        )
        .unwrap();
        let docs = db.query_documents(None, None, None).unwrap();
        assert_eq!(
            docs[0].summary,
            "LLM-generated high quality summary of the JWT auth RFC"
        );
        assert!(docs[0].doc_tags.contains("jwt"));
        assert!(docs[0].doc_tags.contains("authentication"));
    }

    #[test]
    fn test_fts_search_finds_llm_summary_content() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let doc = make_doc("d1", "rfc-writer", "run-1");
        db.upsert_document(&doc).unwrap();

        db.update_document_summary(
            "d1",
            "This RFC describes the JWT-based authentication flow including token refresh and session management",
            &["jwt".into(), "authentication".into(), "session".into()],
        )
        .unwrap();

        // Search by summary content
        let results = db
            .search_documents("authentication token refresh", None, None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "d1");

        // Search by LLM-generated tag
        let results = db
            .search_documents("session", None, None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_summary_preserved_on_upsert_without_summary() {
        let db = IndexDb::open_in_memory().unwrap();
        seed_test_topic(&db);
        let doc = make_doc("d1", "rfc-writer", "run-1");
        db.upsert_document(&doc).unwrap();

        // Write LLM summary
        db.update_document_summary(
            "d1",
            "LLM summary that should survive upsert",
            &["preserved".into()],
        )
        .unwrap();

        // Re-upsert the document (simulating status change via doc transition)
        let mut doc2 = make_doc("d1", "rfc-writer", "run-1");
        doc2.status = "final".to_string();
        db.upsert_document(&doc2).unwrap();

        // Summary should be preserved since Document.summary is empty
        let docs = db.query_documents(None, None, None).unwrap();
        assert_eq!(docs[0].summary, "LLM summary that should survive upsert");
        assert!(docs[0].doc_tags.contains("preserved"));
    }

    #[test]
    fn test_match_topics_by_tags() {
        let db = IndexDb::open_in_memory().unwrap();

        // Create a project first
        let project = crate::model::Project::new("test-project", "Test");
        db.create_project(&project).unwrap();

        // Create topics with tags
        let mut t1 = Topic::new("Auth Module", "Authentication", &project.id);
        t1.tags = vec!["auth".to_string(), "login".to_string(), "security".to_string()];
        db.create_topic(&t1).unwrap();

        let mut t2 = Topic::new("Payment System", "Payments", &project.id);
        t2.tags = vec!["payment".to_string(), "billing".to_string()];
        db.create_topic(&t2).unwrap();

        let mut t3 = Topic::new("User Management", "Users", &project.id);
        t3.tags = vec!["user".to_string(), "auth".to_string()];
        db.create_topic(&t3).unwrap();

        // Match "auth login" → should match t1 (score 2) and t3 (score 1)
        let keywords = vec!["auth".to_string(), "login".to_string()];
        let matches = db.match_topics_by_tags(&keywords).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].0.name, "Auth Module"); // score 2
        assert_eq!(matches[0].1, 2);
        assert_eq!(matches[1].0.name, "User Management"); // score 1
        assert_eq!(matches[1].1, 1);

        // Match "billing" → should match t2 only
        let keywords = vec!["billing".to_string()];
        let matches = db.match_topics_by_tags(&keywords).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0.name, "Payment System");

        // Match "unrelated" → should match nothing
        let keywords = vec!["unrelated".to_string()];
        let matches = db.match_topics_by_tags(&keywords).unwrap();
        assert!(matches.is_empty());
    }

    fn seed_test_db() -> IndexDb {
        IndexDb::open_in_memory().unwrap()
    }

    fn seed_test_run(db: &IndexDb) {
        let def = make_pipeline_def();
        let mut run = PipelineRun::new(&def, "Test Run", "test-topic".to_string(), "");
        run.id = "run-001".to_string();
        db.upsert_pipeline_run(&run).unwrap();
    }

    fn seed_pipeline_run(db: &IndexDb, run_id: &str) {
        let def = make_pipeline_def();
        let mut run = PipelineRun::new(&def, "Run", "test-topic".to_string(), "");
        run.id = run_id.to_string();
        db.upsert_pipeline_run(&run).unwrap();
    }

    #[test]
    fn test_topic_lock_acquire_release() {
        let db = seed_test_db();
        seed_test_topic(&db);
        seed_pipeline_run(&db, "run-1");

        // Acquire lock
        let acquired = db.acquire_topic_lock("test-topic", "run-1").unwrap();
        assert!(acquired);

        // Check lock state
        let lock = db.get_topic_lock("test-topic").unwrap();
        assert_eq!(lock, Some("run-1".to_string()));

        // Topic should have lock fields populated
        let topic = db.get_topic("test-topic").unwrap().unwrap();
        assert_eq!(topic.locked_by_run_id, Some("run-1".to_string()));
        assert!(topic.locked_at.is_some());

        // Release lock
        let released = db.release_topic_lock("test-topic", Some("run-1")).unwrap();
        assert!(released);

        // Should be unlocked
        let lock = db.get_topic_lock("test-topic").unwrap();
        assert!(lock.is_none());
    }

    #[test]
    fn test_topic_lock_exclusive() {
        let db = seed_test_db();
        seed_test_topic(&db);
        seed_pipeline_run(&db, "run-1");
        seed_pipeline_run(&db, "run-2");

        // First run acquires
        let acquired = db.acquire_topic_lock("test-topic", "run-1").unwrap();
        assert!(acquired);

        // Second run can't acquire
        let acquired2 = db.acquire_topic_lock("test-topic", "run-2").unwrap();
        assert!(!acquired2);

        // Same run can re-acquire (idempotent)
        let acquired_same = db.acquire_topic_lock("test-topic", "run-1").unwrap();
        assert!(acquired_same);
    }

    #[test]
    fn test_topic_lock_force_release() {
        let db = seed_test_db();
        seed_test_topic(&db);
        seed_pipeline_run(&db, "run-1");
        seed_pipeline_run(&db, "run-2");

        db.acquire_topic_lock("test-topic", "run-1").unwrap();

        // Force release (no run_id check)
        let released = db.release_topic_lock("test-topic", None).unwrap();
        assert!(released);

        // Now another run can acquire
        let acquired = db.acquire_topic_lock("test-topic", "run-2").unwrap();
        assert!(acquired);
    }

    #[test]
    fn test_update_document_status() {
        let db = seed_test_db();
        seed_test_topic(&db);
        seed_test_run(&db);

        // Seed a document first
        let doc = crate::model::Document::new("prd", "Test PRD", "prd-writer", "run-001", "test-topic");
        db.upsert_document(&doc).unwrap();

        // Update status
        db.update_document_status(&doc.id, "final").unwrap();

        // Verify
        let docs = db.query_documents(None, Some("final"), None).unwrap();
        assert!(docs.iter().any(|d| d.id == doc.id && d.status == "final"));
    }
}
