use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::model::Document;

/// File-system based document storage.
/// Documents are stored as `{slug}.{type}.md` files for Git-friendliness.
pub struct FileStorage;

impl FileStorage {
    /// Write a document to a file, creating parent directories if needed.
    pub fn write_document(doc: &Document, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = doc.to_file_content()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Read a document from a file.
    pub fn read_document(path: &Path) -> Result<Document> {
        let content = std::fs::read_to_string(path)?;
        Document::from_file_content(&content, path.to_path_buf())
    }

    /// List all document files in a directory (recursively).
    pub fn list_documents(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        if !dir.is_dir() {
            return Ok(paths);
        }
        Self::walk_dir(dir, &mut paths)?;
        paths.sort();
        Ok(paths)
    }

    fn walk_dir(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::walk_dir(&path, paths)?;
            } else if path.extension().is_some_and(|ext| ext == "md") {
                paths.push(path);
            }
        }
        Ok(())
    }

    /// Generate the artifact file path from a skill's file_pattern.
    pub fn artifact_path(run_dir: &Path, file_pattern: &str, slug: &str) -> PathBuf {
        let filename = file_pattern.replace("{slug}", slug);
        run_dir.join(filename)
    }

    /// Read the template content from a skill's template file.
    pub fn read_template(template_path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(template_path)?)
    }
}
