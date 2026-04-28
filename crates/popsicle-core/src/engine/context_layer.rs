//! Pluggable context layers for prompt assembly.
//!
//! Each `ContextLayer` produces a markdown section that gets concatenated
//! into the final prompt. Layers are sorted by `relevance` (Low first,
//! High last) so high-importance content lands closest to the instruction.
//!
//! Built-ins:
//! - [`ProjectContextLayer`] — project-wide background (`.popsicle/project-context.md`)
//! - [`MemoriesLayer`] — accumulated memories ranked for the current skill
//! - [`HistoricalRefsLayer`] — cross-run related documents from FTS5 search
//! - [`UpstreamDocsLayer`] — assembled upstream skill outputs
//!
//! Downstream callers can add custom layers (RAG, team conventions, live
//! issue trackers, …) without modifying the engine core.

use crate::engine::context::AssembledContext;
use crate::memory::Memory;
use crate::model::Relevance;
use crate::storage::DocumentRow;

/// A single source of prompt-time context.
pub trait ContextLayer: Send + Sync {
    /// Layer name for ordering / debugging.
    fn name(&self) -> &str;
    /// Where this layer should be placed (Low = front, High = closest to instruction).
    fn relevance(&self) -> Relevance;
    /// Render this layer as a markdown section. Return an empty string to skip.
    fn render(&self) -> String;
}

/// Order layers by relevance (Low → Medium → High), join their non-empty
/// renderings with `---` separators, and append `base_prompt` at the end so
/// it receives maximum LLM attention.
pub fn assemble_layers(layers: Vec<Box<dyn ContextLayer>>, base_prompt: &str) -> String {
    let mut ordered = layers;
    ordered.sort_by_key(|l| l.relevance());

    let mut sections: Vec<String> = ordered
        .iter()
        .map(|l| l.render())
        .filter(|s| !s.trim().is_empty())
        .collect();

    if sections.is_empty() {
        return base_prompt.to_string();
    }

    sections.push(base_prompt.trim().to_string());
    sections.join("\n\n---\n\n")
}

// ── Built-in layers ──

/// Project-wide background loaded from `.popsicle/project-context.md`.
pub struct ProjectContextLayer {
    pub content: String,
}

impl ContextLayer for ProjectContextLayer {
    fn name(&self) -> &str {
        "project_context"
    }
    fn relevance(&self) -> Relevance {
        Relevance::Low
    }
    fn render(&self) -> String {
        if self.content.trim().is_empty() {
            String::new()
        } else {
            format!("## Project Context (background)\n\n{}", self.content.trim())
        }
    }
}

/// Project memories ranked for the current skill / run.
pub struct MemoriesLayer {
    pub memories: Vec<Memory>,
}

impl ContextLayer for MemoriesLayer {
    fn name(&self) -> &str {
        "memories"
    }
    fn relevance(&self) -> Relevance {
        Relevance::Low
    }
    fn render(&self) -> String {
        if self.memories.is_empty() {
            return String::new();
        }
        let mut lines = vec![
            "## Project Memories".to_string(),
            String::new(),
            "以下是项目积累的经验，请在工作中注意避免已知问题：".to_string(),
        ];
        for m in &self.memories {
            let stale_mark = if m.stale { " [STALE]" } else { "" };
            lines.push(format!("- [{}] {}{}", m.memory_type, m.summary, stale_mark));
        }
        lines.join("\n")
    }
}

/// Cross-run related documents discovered via FTS5.
pub struct HistoricalRefsLayer {
    pub refs: Vec<DocumentRow>,
}

impl ContextLayer for HistoricalRefsLayer {
    fn name(&self) -> &str {
        "historical_refs"
    }
    fn relevance(&self) -> Relevance {
        Relevance::Low
    }
    fn render(&self) -> String {
        if self.refs.is_empty() {
            return String::new();
        }
        let mut lines = vec![
            "## Historical References (from previous runs)".to_string(),
            String::new(),
            "以下是项目中可能相关的历史设计文档，如需详细内容请读取对应文件：".to_string(),
            String::new(),
        ];
        for doc in &self.refs {
            lines.push(format!(
                "- **[{}] {}** ({}) — {}",
                doc.doc_type.to_uppercase(),
                doc.title,
                doc.status,
                doc.file_path,
            ));
            if !doc.summary.is_empty() {
                let preview: String = doc.summary.lines().next().unwrap_or("").to_string();
                if !preview.is_empty() {
                    lines.push(format!("  {}", preview));
                }
            }
        }
        lines.join("\n")
    }
}

/// Assembled upstream skill outputs (already relevance-sorted internally).
pub struct UpstreamDocsLayer {
    pub assembled: AssembledContext,
}

impl ContextLayer for UpstreamDocsLayer {
    fn name(&self) -> &str {
        "upstream_docs"
    }
    fn relevance(&self) -> Relevance {
        Relevance::High
    }
    fn render(&self) -> String {
        if self.assembled.full_text.trim().is_empty() {
            return String::new();
        }
        format!(
            "## Input Context (from upstream skills)\n\n{}",
            self.assembled.full_text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StaticLayer {
        n: &'static str,
        r: Relevance,
        body: &'static str,
    }
    impl ContextLayer for StaticLayer {
        fn name(&self) -> &str {
            self.n
        }
        fn relevance(&self) -> Relevance {
            self.r
        }
        fn render(&self) -> String {
            self.body.to_string()
        }
    }

    #[test]
    fn empty_layers_returns_base_prompt() {
        let out = assemble_layers(Vec::new(), "DO THE THING");
        assert_eq!(out, "DO THE THING");
    }

    #[test]
    fn layers_sorted_low_to_high_and_base_last() {
        let layers: Vec<Box<dyn ContextLayer>> = vec![
            Box::new(StaticLayer {
                n: "high",
                r: Relevance::High,
                body: "HIGH",
            }),
            Box::new(StaticLayer {
                n: "low",
                r: Relevance::Low,
                body: "LOW",
            }),
            Box::new(StaticLayer {
                n: "med",
                r: Relevance::Medium,
                body: "MED",
            }),
        ];
        let out = assemble_layers(layers, "BASE");
        let parts: Vec<&str> = out.split("\n\n---\n\n").collect();
        assert_eq!(parts, vec!["LOW", "MED", "HIGH", "BASE"]);
    }

    #[test]
    fn empty_render_is_skipped() {
        let layers: Vec<Box<dyn ContextLayer>> = vec![
            Box::new(StaticLayer {
                n: "blank",
                r: Relevance::Low,
                body: "",
            }),
            Box::new(StaticLayer {
                n: "kept",
                r: Relevance::High,
                body: "KEPT",
            }),
        ];
        let out = assemble_layers(layers, "BASE");
        assert_eq!(out, "KEPT\n\n---\n\nBASE");
    }
}
