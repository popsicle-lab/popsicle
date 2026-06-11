//! Document model + file-content round-trip.
//!
//! Mirrors `acceptance.intent` › `DocumentRoundTrips(d)`:
//! `require d.version >= 1; ensure d.body' == d.body, d.id' == d.id,
//! d.version' == d.version` — serializing a document to file content and parsing
//! it back preserves the body (and the id/version frame) exactly.
//!
//! Divergence from legacy (deliberate): legacy `from_file_content` calls
//! `trim_start()` on the parsed body, so bodies with leading whitespace do **not**
//! round-trip. To satisfy the formal `body' == body` for *every* body, this
//! implementation strips only the single known separator and preserves the body
//! verbatim. (Wire-format byte-parity with legacy serde_yaml is a separate
//! equivalence-baseline concern, not this intent.)
//!
//! Domain note: the frontmatter is line-oriented and unescaped (faithful to
//! legacy). Scalar fields (`id`, `doc_type`, `title`, `status`, `parent_id`) are
//! therefore assumed single-line — embedding a newline or a `key: value` line in
//! a scalar would not round-trip. The arbitrary-content field is `body`, which
//! *does* round-trip verbatim (including embedded `---` delimiters). Escaping
//! scalars is intentionally deferred: it would diverge from the legacy wire
//! format and break the future byte-parity baseline.

use std::collections::BTreeMap;

/// Frontmatter keys owned by named `Document` fields; everything else in the
/// frontmatter is preserved opaquely in [`Document::extra_frontmatter`].
const KNOWN_KEYS: [&str; 6] = ["id", "doc_type", "title", "status", "version", "parent_id"];

/// A document artifact: YAML-ish frontmatter metadata + Markdown body.
///
/// Mirrors `type Document` shared across the artifact-system intents
/// (`id`, `body`, `version`, `parentId`, `status`); extra named fields
/// (`doc_type`, `title`) and an opaque `extra_frontmatter` bucket let unknown
/// legacy frontmatter keys round-trip without committing to the full legacy
/// 16-field struct yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: String,
    pub doc_type: String,
    pub title: String,
    /// `"active"` | `"final"`.
    pub status: String,
    /// Revision counter; starts at 1, bumped on each revision. (`version >= 1`.)
    pub version: u32,
    /// Previous version's id; `None` = no parent.
    pub parent_id: Option<String>,
    /// Frontmatter keys not mapped to a named field, preserved verbatim.
    pub extra_frontmatter: BTreeMap<String, String>,
    pub body: String,
}

/// Error parsing raw file content into a [`Document`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Missing the opening or closing `---` frontmatter delimiter.
    MissingFrontmatter(String),
    /// A frontmatter line was not `key: value`.
    MalformedFrontmatter(String),
    /// `version` was absent or not a u32.
    InvalidVersion(String),
}

impl Document {
    /// Construct a fresh document at version 1, no parent, status `active`.
    pub fn new(
        id: impl Into<String>,
        doc_type: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            doc_type: doc_type.into(),
            title: title.into(),
            status: "active".to_string(),
            version: 1,
            parent_id: None,
            extra_frontmatter: BTreeMap::new(),
            body: String::new(),
        }
    }

    /// Create the next revision: version + 1, links `parent_id` to this id,
    /// status reset to `active`, body carried over. (Mirrors legacy
    /// `new_revision` semantics relevant to `version`/`parent`.)
    pub fn new_revision(&self, new_id: impl Into<String>) -> Self {
        Self {
            id: new_id.into(),
            doc_type: self.doc_type.clone(),
            title: self.title.clone(),
            status: "active".to_string(),
            version: self.version + 1,
            parent_id: Some(self.id.clone()),
            extra_frontmatter: self.extra_frontmatter.clone(),
            body: self.body.clone(),
        }
    }

    /// Serialize to file content: `---\n{frontmatter}---\n\n{body}`.
    ///
    /// Frontmatter keys are emitted in a deterministic (sorted) order so the
    /// output is stable regardless of insertion order.
    pub fn to_file_content(&self) -> String {
        let mut kv: BTreeMap<String, String> = self.extra_frontmatter.clone();
        kv.insert("id".into(), self.id.clone());
        kv.insert("doc_type".into(), self.doc_type.clone());
        kv.insert("title".into(), self.title.clone());
        kv.insert("status".into(), self.status.clone());
        kv.insert("version".into(), self.version.to_string());
        // `~` is the conventional null marker; an empty parent omits the key.
        if let Some(ref p) = self.parent_id {
            kv.insert("parent_id".into(), p.clone());
        }

        let mut fm = String::new();
        for (k, v) in &kv {
            fm.push_str(k);
            fm.push_str(": ");
            fm.push_str(v);
            fm.push('\n');
        }
        format!("---\n{fm}---\n\n{body}", fm = fm, body = self.body)
    }

    /// Parse file content produced by [`Document::to_file_content`].
    ///
    /// Body is preserved **verbatim** (only the single `\n\n` separator after the
    /// closing `---` is removed), so `from_file_content(to_file_content(d)).body
    /// == d.body` for any body — satisfying `DocumentRoundTrips`.
    pub fn from_file_content(content: &str) -> Result<Self, ParseError> {
        let after_open = content.strip_prefix("---\n").ok_or_else(|| {
            ParseError::MissingFrontmatter("missing opening '---' delimiter".to_string())
        })?;

        // The closing delimiter is a line containing exactly `---`. Serialization
        // emits `...{last_fm}\n---\n\n{body}`, so the first `\n---\n` after the
        // frontmatter marks the boundary; `after_open` itself begins at the first
        // frontmatter key, so any `\n---\n` belongs to the closing delimiter.
        let close = after_open.find("\n---\n").ok_or_else(|| {
            ParseError::MissingFrontmatter("missing closing '---' delimiter".to_string())
        })?;

        let frontmatter = &after_open[..close];
        // Skip the matched "\n---\n" then the single blank-line "\n" separator.
        let after_close = &after_open[close + "\n---\n".len()..];
        let body = after_close
            .strip_prefix('\n')
            .unwrap_or(after_close)
            .to_string();

        let mut kv: BTreeMap<String, String> = BTreeMap::new();
        for line in frontmatter.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let (k, v) = line.split_once(": ").ok_or_else(|| {
                ParseError::MalformedFrontmatter(format!("expected 'key: value', got '{line}'"))
            })?;
            kv.insert(k.trim().to_string(), v.to_string());
        }

        let version_str = kv
            .get("version")
            .ok_or_else(|| ParseError::InvalidVersion("missing 'version'".to_string()))?;
        let version: u32 = version_str
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidVersion(format!("not a u32: '{version_str}'")))?;

        let take =
            |kv: &mut BTreeMap<String, String>, key: &str| kv.remove(key).unwrap_or_default();
        let id = take(&mut kv, "id");
        let doc_type = take(&mut kv, "doc_type");
        let title = take(&mut kv, "title");
        let status = take(&mut kv, "status");
        let parent_id = kv.remove("parent_id");
        kv.remove("version");

        // Whatever named keys we did not consume remain opaque.
        for k in KNOWN_KEYS {
            kv.remove(k);
        }

        Ok(Document {
            id,
            doc_type,
            title,
            status,
            version,
            parent_id,
            extra_frontmatter: kv,
            body,
        })
    }
}
