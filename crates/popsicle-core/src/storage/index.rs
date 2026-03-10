use rusqlite::{Connection, params};
use std::path::Path;

use crate::error::Result;
use crate::git::{CommitLink, ReviewStatus};
use crate::model::{
    Discussion, DiscussionMessage, DiscussionRole, DiscussionStatus, Document, Issue, IssueStatus,
    IssueType, MessageType, PipelineRun, Priority, RoleSource, StageState,
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
                pipeline_run_id TEXT,
                labels TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_issue_key ON issues(key);
            CREATE INDEX IF NOT EXISTS idx_issue_status ON issues(status);
            CREATE INDEX IF NOT EXISTS idx_issue_type ON issues(issue_type);
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
            "INSERT INTO issues (id, key, title, description, issue_type, priority, status, pipeline_run_id, labels, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                issue.id,
                issue.key,
                issue.title,
                issue.description,
                issue.issue_type.to_string(),
                issue.priority.to_string(),
                issue.status.to_string(),
                issue.pipeline_run_id,
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
            "UPDATE issues SET title=?1, description=?2, priority=?3, status=?4, pipeline_run_id=?5, labels=?6, updated_at=?7 WHERE id=?8",
            params![
                issue.title,
                issue.description,
                issue.priority.to_string(),
                issue.status.to_string(),
                issue.pipeline_run_id,
                labels_json,
                chrono::Utc::now().to_rfc3339(),
                issue.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_issue(&self, id_or_key: &str) -> Result<Option<Issue>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, title, description, issue_type, priority, status, pipeline_run_id, labels, created_at, updated_at
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
                pipeline_run_id: row.get(7)?,
                labels: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(issue_from_row(row?)?)),
            None => Ok(None),
        }
    }

    pub fn find_issue_by_run_id(&self, run_id: &str) -> Result<Option<Issue>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, title, description, issue_type, priority, status, pipeline_run_id, labels, created_at, updated_at
             FROM issues WHERE pipeline_run_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![run_id], |row| {
            Ok(IssueRow {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                issue_type: row.get(4)?,
                priority: row.get(5)?,
                status: row.get(6)?,
                pipeline_run_id: row.get(7)?,
                labels: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(issue_from_row(row?)?)),
            None => Ok(None),
        }
    }

    pub fn query_issues(
        &self,
        issue_type: Option<&str>,
        status: Option<&str>,
        label: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let mut sql = "SELECT id, key, title, description, issue_type, priority, status, pipeline_run_id, labels, created_at, updated_at FROM issues WHERE 1=1".to_string();
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
                pipeline_run_id: row.get(7)?,
                labels: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(issue_from_row(row?)?);
        }
        Ok(results)
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
    pipeline_run_id: Option<String>,
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
        pipeline_run_id: row.pipeline_run_id,
        labels,
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

        let by_both = db
            .query_documents(Some("prd"), None, Some("run-1"))
            .unwrap();
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
