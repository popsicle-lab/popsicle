use std::path::Path;

use chrono::NaiveDate;
use regex::Regex;

use crate::error::{PopsicleError, Result};

use super::model::{Memory, MemoryLayer, MemoryType};

/// Default expiration age for unreferenced short-term memories (days).
pub const SHORT_TERM_EXPIRY_DAYS: i64 = 30;

/// Hard limit on the serialized line count of memories.md.
pub const MAX_LINES: usize = 200;

/// Load and save memories from/to a `.popsicle/memories.md` file.
pub struct MemoryStore;

impl MemoryStore {
    /// Parse `memories.md` into a list of `Memory` entries.
    /// Returns an empty vec if the file does not exist.
    pub fn load(path: &Path) -> Result<Vec<Memory>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Serialize memories and write to `memories.md`, creating parent dirs.
    pub fn save(path: &Path, memories: &[Memory]) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = Self::render(memories);
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Count how many lines the serialized output would occupy.
    pub fn line_count(memories: &[Memory]) -> usize {
        let content = Self::render(memories);
        content.lines().count()
    }

    /// Check whether adding new memories would exceed the line limit.
    pub fn would_exceed_limit(existing: &[Memory], new_count: usize) -> bool {
        let mut test = existing.to_vec();
        for _ in 0..new_count {
            test.push(Memory {
                id: 0,
                memory_type: MemoryType::Bug,
                summary: "x".into(),
                created: "2000-01-01".into(),
                layer: MemoryLayer::ShortTerm,
                refs: 0,
                tags: vec![],
                files: vec![],
                run: None,
                stale: false,
                detail: "x".into(),
            });
        }
        Self::line_count(&test) > MAX_LINES
    }

    /// Remove short-term memories with `refs == 0` older than `expiry_days`.
    /// Returns the list of expired memories that were removed.
    pub fn expire_short_term(memories: &mut Vec<Memory>, expiry_days: i64) -> Vec<Memory> {
        let today = chrono::Utc::now().date_naive();
        let mut expired = Vec::new();
        memories.retain(|m| {
            if m.layer != MemoryLayer::ShortTerm || m.refs > 0 {
                return true;
            }
            if let Ok(created) = NaiveDate::parse_from_str(&m.created, "%Y-%m-%d") {
                let age = (today - created).num_days();
                if age > expiry_days {
                    expired.push(m.clone());
                    return false;
                }
            }
            true
        });
        expired
    }

    /// Capacity utilization as a percentage (0–100).
    pub fn capacity_pct(memories: &[Memory]) -> usize {
        let lines = Self::line_count(memories);
        (lines * 100) / MAX_LINES
    }

    // ── Rendering ──

    fn render(memories: &[Memory]) -> String {
        let mut lines = Vec::new();
        lines.push("# Project Memories".to_string());
        lines.push(String::new());

        let long_term: Vec<_> = memories
            .iter()
            .filter(|m| m.layer == MemoryLayer::LongTerm)
            .collect();
        let short_term: Vec<_> = memories
            .iter()
            .filter(|m| m.layer == MemoryLayer::ShortTerm)
            .collect();

        lines.push("## Long-term".to_string());
        lines.push(String::new());

        if long_term.is_empty() {
            lines.push("(none)".to_string());
            lines.push(String::new());
        } else {
            for m in &long_term {
                Self::render_memory(m, &mut lines);
            }
        }

        lines.push("---".to_string());
        lines.push(String::new());
        lines.push("## Short-term".to_string());
        lines.push(String::new());

        if short_term.is_empty() {
            lines.push("(none)".to_string());
            lines.push(String::new());
        } else {
            for m in &short_term {
                Self::render_memory(m, &mut lines);
            }
        }

        lines.join("\n")
    }

    fn render_memory(m: &Memory, lines: &mut Vec<String>) {
        let stale_prefix = if m.stale { "[STALE] " } else { "" };
        lines.push(format!(
            "### {stale_prefix}[{}] {}",
            m.memory_type, m.summary
        ));

        let mut meta1 = format!(
            "- **Created**: {} | **Layer**: {} | **Refs**: {}",
            m.created, m.layer, m.refs
        );
        if let Some(ref run) = m.run {
            meta1.push_str(&format!(" | **Run**: {run}"));
        }
        // Embed the id so we can parse it back.
        meta1.push_str(&format!(" | **ID**: {}", m.id));
        lines.push(meta1);

        let mut meta2_parts = Vec::new();
        if !m.tags.is_empty() {
            meta2_parts.push(format!("**Tags**: {}", m.tags.join(", ")));
        }
        if !m.files.is_empty() {
            meta2_parts.push(format!("**Files**: {}", m.files.join(", ")));
        }
        if !meta2_parts.is_empty() {
            lines.push(format!("- {}", meta2_parts.join(" | ")));
        }

        for detail_line in m.detail.lines() {
            lines.push(format!("- {detail_line}"));
        }
        lines.push(String::new());
    }

    // ── Parsing ──

    fn parse(content: &str) -> Result<Vec<Memory>> {
        let heading_re =
            Regex::new(r"^###\s+(?:\[STALE\]\s+)?\[(\w+)\]\s+(.+)$").expect("valid regex");
        let meta_re = Regex::new(
            r"\*\*Created\*\*:\s*(\S+)\s*\|\s*\*\*Layer\*\*:\s*([^\|]+?)\s*\|\s*\*\*Refs\*\*:\s*(\d+)",
        )
        .expect("valid regex");
        let run_re = Regex::new(r"\*\*Run\*\*:\s*(\S+)").expect("valid regex");
        let id_re = Regex::new(r"\*\*ID\*\*:\s*(\d+)").expect("valid regex");
        let tags_re = Regex::new(r"\*\*Tags\*\*:\s*([^\|]+)").expect("valid regex");
        let files_re = Regex::new(r"\*\*Files\*\*:\s*(.+)").expect("valid regex");

        let mut memories = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            if let Some(caps) = heading_re.captures(line) {
                let stale = line.contains("[STALE]");
                let type_str = caps.get(1).unwrap().as_str();
                let summary = caps.get(2).unwrap().as_str().trim().to_string();
                let memory_type: MemoryType = type_str
                    .parse()
                    .map_err(|e: String| PopsicleError::InvalidDocumentFormat(e))?;

                i += 1;

                let mut created = String::new();
                let mut layer = MemoryLayer::ShortTerm;
                let mut refs = 0u32;
                let mut run = None;
                let mut id = 0u32;
                let mut tags = Vec::new();
                let mut files = Vec::new();
                let mut detail_lines = Vec::new();

                while i < lines.len() && !lines[i].starts_with("###") {
                    let l = lines[i].trim();

                    if l.is_empty() || l == "---" || l.starts_with("## ") || l.starts_with("# ") {
                        i += 1;
                        if l == "---" || l.starts_with("## ") || l.starts_with("# ") {
                            break;
                        }
                        continue;
                    }

                    if let Some(mc) = meta_re.captures(l) {
                        created = mc.get(1).unwrap().as_str().to_string();
                        layer = mc
                            .get(2)
                            .unwrap()
                            .as_str()
                            .trim()
                            .parse()
                            .unwrap_or(MemoryLayer::ShortTerm);
                        refs = mc.get(3).unwrap().as_str().parse().unwrap_or(0);
                        if let Some(rc) = run_re.captures(l) {
                            run = Some(rc.get(1).unwrap().as_str().to_string());
                        }
                        if let Some(ic) = id_re.captures(l) {
                            id = ic.get(1).unwrap().as_str().parse().unwrap_or(0);
                        }
                    } else if let Some(tc) = tags_re.captures(l) {
                        tags = tc
                            .get(1)
                            .unwrap()
                            .as_str()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if let Some(fc) = files_re.captures(l) {
                            files = fc
                                .get(1)
                                .unwrap()
                                .as_str()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                    } else if let Some(fc) = files_re.captures(l) {
                        files = fc
                            .get(1)
                            .unwrap()
                            .as_str()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    } else {
                        let text = l.strip_prefix("- ").unwrap_or(l);
                        if !text.is_empty() {
                            detail_lines.push(text.to_string());
                        }
                    }

                    i += 1;
                }

                memories.push(Memory {
                    id,
                    memory_type,
                    summary,
                    created,
                    layer,
                    refs,
                    tags,
                    files,
                    run,
                    stale,
                    detail: detail_lines.join("\n"),
                });
            } else {
                i += 1;
            }
        }

        Ok(memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_memory(id: u32, layer: MemoryLayer) -> Memory {
        Memory {
            id,
            memory_type: MemoryType::Bug,
            summary: format!("Test bug {id}"),
            created: "2026-03-14".into(),
            layer,
            refs: 2,
            tags: vec!["serde".into(), "yaml".into()],
            files: vec!["model/skill.rs".into()],
            run: Some("run-1".into()),
            stale: false,
            detail: "Root cause: missing default.\nFix: add #[serde(default)].".into(),
        }
    }

    #[test]
    fn roundtrip_single_memory() {
        let m = sample_memory(1, MemoryLayer::LongTerm);
        let rendered = MemoryStore::render(std::slice::from_ref(&m));
        let parsed = MemoryStore::parse(&rendered).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.id, 1);
        assert_eq!(p.memory_type, MemoryType::Bug);
        assert_eq!(p.summary, "Test bug 1");
        assert_eq!(p.created, "2026-03-14");
        assert_eq!(p.layer, MemoryLayer::LongTerm);
        assert_eq!(p.refs, 2);
        assert_eq!(p.tags, vec!["serde", "yaml"]);
        assert_eq!(p.files, vec!["model/skill.rs"]);
        assert_eq!(p.run.as_deref(), Some("run-1"));
        assert!(!p.stale);
        assert!(p.detail.contains("Root cause"));
    }

    #[test]
    fn roundtrip_both_layers() {
        let memories = vec![
            sample_memory(1, MemoryLayer::LongTerm),
            sample_memory(2, MemoryLayer::ShortTerm),
        ];
        let rendered = MemoryStore::render(&memories);
        let parsed = MemoryStore::parse(&rendered).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].layer, MemoryLayer::LongTerm);
        assert_eq!(parsed[1].layer, MemoryLayer::ShortTerm);
    }

    #[test]
    fn parse_stale_memory() {
        let md = r#"# Project Memories

## Long-term

### [STALE] [BUG] Old bug
- **Created**: 2026-01-01 | **Layer**: long-term | **Refs**: 0 | **ID**: 5
- This is outdated.

---

## Short-term

(none)
"#;
        let parsed = MemoryStore::parse(md).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].stale);
        assert_eq!(parsed[0].id, 5);
    }

    #[test]
    fn parse_empty_file() {
        let parsed = MemoryStore::parse("").unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn line_count_empty() {
        let count = MemoryStore::line_count(&[]);
        assert!(count < MAX_LINES);
    }

    #[test]
    fn line_count_grows_with_memories() {
        let m1 = sample_memory(1, MemoryLayer::LongTerm);
        let count1 = MemoryStore::line_count(std::slice::from_ref(&m1));
        let m2 = sample_memory(2, MemoryLayer::LongTerm);
        let count2 = MemoryStore::line_count(&[m1, m2]);
        assert!(count2 > count1);
    }

    #[test]
    fn render_no_tags_no_files() {
        let m = Memory {
            id: 1,
            memory_type: MemoryType::Decision,
            summary: "Chose X over Y".into(),
            created: "2026-03-14".into(),
            layer: MemoryLayer::LongTerm,
            refs: 0,
            tags: vec![],
            files: vec![],
            run: None,
            stale: false,
            detail: "Because X is simpler.".into(),
        };
        let rendered = MemoryStore::render(&[m]);
        assert!(rendered.contains("[DECISION]"));
        assert!(!rendered.contains("**Tags**"));
        assert!(!rendered.contains("**Files**"));
        let parsed = MemoryStore::parse(&rendered).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].summary, "Chose X over Y");
    }
}
