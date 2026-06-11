//! ADR-004: `MemoriesLayer` — skill-runtime-owned [`ContextLayer`].

use artifact_system::context::{ContextLayer, Relevance};

/// A single project memory entry (mirrors legacy `memory::Memory` surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memory {
    pub memory_type: String,
    pub summary: String,
    pub stale: bool,
}

/// Injects ranked project memories into prompt assembly (legacy `MemoriesLayer`).
#[derive(Debug, Clone, Default)]
pub struct MemoriesLayer {
    memories: Vec<Memory>,
}

impl MemoriesLayer {
    pub fn new(memories: Vec<Memory>) -> Self {
        Self { memories }
    }

    pub fn memories(&self) -> &[Memory] {
        &self.memories
    }
}

impl ContextLayer for MemoriesLayer {
    fn id(&self) -> &str {
        "memories"
    }

    fn relevance(&self) -> Relevance {
        Relevance::RelLow
    }

    fn priority(&self) -> i32 {
        1
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
            let stale = if m.stale { " [STALE]" } else { "" };
            lines.push(format!("- [{}] {}{}", m.memory_type, m.summary, stale));
        }
        lines.join("\n")
    }
}
