use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A generic document with YAML frontmatter metadata + Markdown body.
/// All Skill artifacts (PRD, RFC, ADR, TestSpec, etc.) share this model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    /// Omitted in older frontmatter; when empty, callers should fill from index (doc_row.skill_name).
    #[serde(default)]
    pub skill_name: String,
    pub pipeline_run_id: String,
    /// Topic this document belongs to.
    pub topic_id: String,
    /// Revision counter (starts at 1, incremented on each revision).
    #[serde(default = "default_version")]
    pub version: u32,
    /// Previous version's document ID (if this is a revision).
    #[serde(default)]
    pub parent_doc_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Auto-generated summary for document indexing (not persisted in frontmatter).
    #[serde(skip)]
    pub summary: String,
    #[serde(default)]
    pub metadata: serde_yaml_ng::Value,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub body: String,
    #[serde(skip)]
    pub file_path: PathBuf,
}

fn default_version() -> u32 {
    1
}

/// Parsed representation: frontmatter + body split from raw markdown.
#[derive(Debug, Clone)]
pub struct RawDocument {
    pub frontmatter: String,
    pub body: String,
}

impl Document {
    pub fn new(
        doc_type: impl Into<String>,
        title: impl Into<String>,
        skill_name: impl Into<String>,
        pipeline_run_id: impl Into<String>,
        topic_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            doc_type: doc_type.into(),
            title: title.into(),
            status: "draft".to_string(),
            skill_name: skill_name.into(),
            pipeline_run_id: pipeline_run_id.into(),
            topic_id: topic_id.into(),
            version: 1,
            parent_doc_id: None,
            tags: Vec::new(),
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: Some(now),
            updated_at: Some(now),
            body: String::new(),
            file_path: PathBuf::new(),
        }
    }

    /// Create a new revision of this document.
    /// Bumps version, links to parent, resets status to draft.
    pub fn new_revision(&self, new_run_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            doc_type: self.doc_type.clone(),
            title: self.title.clone(),
            status: "draft".to_string(),
            skill_name: self.skill_name.clone(),
            pipeline_run_id: new_run_id.into(),
            topic_id: self.topic_id.clone(),
            version: self.version + 1,
            parent_doc_id: Some(self.id.clone()),
            tags: self.tags.clone(),
            summary: String::new(),
            metadata: self.metadata.clone(),
            created_at: Some(now),
            updated_at: Some(now),
            body: self.body.clone(),
            file_path: PathBuf::new(),
        }
    }

    /// Serialize to file content: YAML frontmatter + Markdown body.
    pub fn to_file_content(&self) -> crate::error::Result<String> {
        let frontmatter = serde_yaml_ng::to_string(self)?;
        Ok(format!("---\n{}---\n\n{}", frontmatter, self.body))
    }

    /// Parse from raw file content (YAML frontmatter + Markdown body).
    pub fn from_file_content(content: &str, file_path: PathBuf) -> crate::error::Result<Self> {
        let raw = parse_frontmatter(content)?;
        let mut doc: Document = serde_yaml_ng::from_str(&raw.frontmatter)?;
        doc.body = raw.body;
        doc.file_path = file_path;
        Ok(doc)
    }
}

/// Split raw file content into YAML frontmatter and Markdown body.
fn parse_frontmatter(content: &str) -> crate::error::Result<RawDocument> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(crate::error::PopsicleError::InvalidDocumentFormat(
            "Missing YAML frontmatter delimiter '---'".to_string(),
        ));
    }

    let after_first = &content[3..];
    let end_pos = after_first.find("\n---").ok_or_else(|| {
        crate::error::PopsicleError::InvalidDocumentFormat(
            "Missing closing frontmatter delimiter '---'".to_string(),
        )
    })?;

    let frontmatter = after_first[..end_pos].trim().to_string();
    let body = after_first[end_pos + 4..].trim_start().to_string();

    Ok(RawDocument { frontmatter, body })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
id: doc-001
title: Test Document
---

## Hello World
Some body content.
"#;
        let raw = parse_frontmatter(content).unwrap();
        assert!(raw.frontmatter.contains("id: doc-001"));
        assert!(raw.body.contains("## Hello World"));
    }

    #[test]
    fn test_document_roundtrip() {
        let mut doc = Document::new("prd", "Test PRD", "product-prd", "run-001", "topic-001");
        doc.body = "## Background\nSome content.".to_string();

        let content = doc.to_file_content().unwrap();
        let parsed = Document::from_file_content(&content, PathBuf::from("test.md")).unwrap();

        assert_eq!(parsed.title, "Test PRD");
        assert_eq!(parsed.doc_type, "prd");
        assert_eq!(parsed.topic_id, "topic-001");
        assert_eq!(parsed.version, 1);
        assert!(parsed.parent_doc_id.is_none());
        assert!(parsed.body.contains("## Background"));
    }

    #[test]
    fn test_document_revision() {
        let doc = Document::new("rfc", "RFC v1", "rfc-writer", "run-001", "topic-001");
        let rev = doc.new_revision("run-002");
        assert_eq!(rev.version, 2);
        assert_eq!(rev.parent_doc_id, Some(doc.id.clone()));
        assert_eq!(rev.topic_id, "topic-001");
        assert_eq!(rev.pipeline_run_id, "run-002");
        assert_eq!(rev.status, "draft");
        assert_ne!(rev.id, doc.id);
    }
}
