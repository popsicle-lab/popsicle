//! `DocumentRow` — persistence-facing mirror of [`artifact_system::document::Document`].

use artifact_system::document::Document;

/// A document row suitable for index / file-backed storage (ADR-004).
///
/// Field set is a deliberate subset of legacy `documents` table columns needed
/// for in-shadow round-trip; extra legacy columns (summary, tags, …) land with
/// full SQLite cutover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRow {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub version: u32,
    pub parent_id: Option<String>,
    /// On-disk path relative to project root (`.popsicle/artifacts/...`).
    pub file_path: String,
    pub body: String,
}

impl DocumentRow {
    pub fn from_document(doc: &Document, file_path: impl Into<String>) -> Self {
        Self {
            id: doc.id.clone(),
            doc_type: doc.doc_type.clone(),
            title: doc.title.clone(),
            status: doc.status.clone(),
            version: doc.version,
            parent_id: doc.parent_id.clone(),
            file_path: file_path.into(),
            body: doc.body.clone(),
        }
    }

    /// Reconstruct a domain [`Document`] (extra frontmatter is empty on this path).
    pub fn to_document(&self) -> Document {
        Document {
            id: self.id.clone(),
            doc_type: self.doc_type.clone(),
            title: self.title.clone(),
            status: self.status.clone(),
            version: self.version,
            parent_id: self.parent_id.clone(),
            extra_frontmatter: Default::default(),
            body: self.body.clone(),
        }
    }

    /// Serialize to on-disk file content (delegates to domain round-trip).
    pub fn to_file_content(&self) -> String {
        self.to_document().to_file_content()
    }
}
