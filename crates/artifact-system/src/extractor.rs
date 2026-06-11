//! Total, kind-preserving structured extraction.
//!
//! Mirrors `acceptance.intent` › `ExtractPreservesKind(e)`: `e.kind' == e.kind`
//! — every item an extraction function produces carries the kind of *that*
//! function. Mirrors ADR-004 contract 2: extraction is **total** — legacy's 19
//! production `regex` `.unwrap()`s (extractor.rs) are gone; this dependency-free
//! line scanner never panics and returns an empty `Vec` when nothing matches.

/// Kind of an extracted chunk. Mirrors `enum ChunkKind` in `acceptance.intent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    KindBug,
    KindStory,
    KindTestCase,
}

/// A single extracted item. Mirrors `type ExtractItem { kind }` (title carried
/// for usefulness; the verified property only constrains `kind`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractItem {
    pub kind: ChunkKind,
    pub title: String,
}

/// Collect `### ` (H3) heading titles within the `## section_name` H2 section.
/// When `section_name` is `None`, scans the whole body. Total: any input,
/// including malformed/empty, yields a (possibly empty) `Vec` and never panics.
fn h3_titles(body: &str, section_name: Option<&str>) -> Vec<String> {
    let scope: &str = match section_name {
        Some(name) => {
            let header = format!("## {name}");
            match body.find(&header) {
                Some(pos) => {
                    let after = &body[pos + header.len()..];
                    // Up to the next H2.
                    match after.find("\n## ") {
                        Some(end) => &after[..end],
                        None => after,
                    }
                }
                None => return Vec::new(),
            }
        }
        None => body,
    };

    scope
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// Extract user stories (kind = `KindStory`) from the "User Stories" section.
pub fn extract_user_stories(body: &str) -> Vec<ExtractItem> {
    let titles = {
        let primary = h3_titles(body, Some("User Stories & Acceptance Criteria"));
        if primary.is_empty() {
            h3_titles(body, Some("User Stories"))
        } else {
            primary
        }
    };
    titles
        .into_iter()
        .map(|title| ExtractItem {
            kind: ChunkKind::KindStory,
            title,
        })
        .collect()
}

/// Extract test cases (kind = `KindTestCase`) — every `### ` heading in the body.
pub fn extract_test_cases(body: &str) -> Vec<ExtractItem> {
    h3_titles(body, None)
        .into_iter()
        .map(|title| ExtractItem {
            kind: ChunkKind::KindTestCase,
            title,
        })
        .collect()
}

/// Extract bugs (kind = `KindBug`) from the "Bugs" section.
pub fn extract_bugs(body: &str) -> Vec<ExtractItem> {
    h3_titles(body, Some("Bugs"))
        .into_iter()
        .map(|title| ExtractItem {
            kind: ChunkKind::KindBug,
            title,
        })
        .collect()
}
