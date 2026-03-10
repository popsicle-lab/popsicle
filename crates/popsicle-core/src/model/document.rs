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
    #[serde(default)]
    pub tags: Vec<String>,
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
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            doc_type: doc_type.into(),
            title: title.into(),
            status: "draft".to_string(),
            skill_name: skill_name.into(),
            pipeline_run_id: pipeline_run_id.into(),
            tags: Vec::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: Some(now),
            updated_at: Some(now),
            body: String::new(),
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
        let mut doc = Document::new("prd", "Test PRD", "product-prd", "run-001");
        doc.body = "## Background\nSome content.".to_string();

        let content = doc.to_file_content().unwrap();
        let parsed = Document::from_file_content(&content, PathBuf::from("test.md")).unwrap();

        assert_eq!(parsed.title, "Test PRD");
        assert_eq!(parsed.doc_type, "prd");
        assert!(parsed.body.contains("## Background"));
    }
}
