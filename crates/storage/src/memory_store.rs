//! In-memory document store for in-shadow tests and golden baselines.

use std::collections::BTreeMap;

use crate::document_row::DocumentRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    NotFound(String),
    AlreadyExists(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "document not found: {id}"),
            Self::AlreadyExists(id) => write!(f, "document already exists: {id}"),
        }
    }
}

impl std::error::Error for StoreError {}

/// Minimal in-memory document index (no SQLite).
#[derive(Debug, Default)]
pub struct MemoryDocumentStore {
    rows: BTreeMap<String, DocumentRow>,
}

impl MemoryDocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, row: DocumentRow) -> Result<(), StoreError> {
        if self.rows.contains_key(&row.id) {
            return Err(StoreError::AlreadyExists(row.id));
        }
        self.rows.insert(row.id.clone(), row);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&DocumentRow, StoreError> {
        self.rows.get(id).ok_or_else(|| StoreError::NotFound(id.into()))
    }

    pub fn update(&mut self, row: DocumentRow) -> Result<(), StoreError> {
        if !self.rows.contains_key(&row.id) {
            return Err(StoreError::NotFound(row.id));
        }
        self.rows.insert(row.id.clone(), row);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
