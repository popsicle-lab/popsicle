//! Runtime context-layer registry (ADR-004 contract 3 consumer).

use artifact_system::context::{assemble_layers, ContextLayer};

use crate::memory_layer::MemoriesLayer;

/// Owns pluggable [`ContextLayer`]s and assembles a full prompt.
#[derive(Default)]
pub struct ContextRegistry {
    layers: Vec<Box<dyn ContextLayer>>,
}

impl ContextRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a layer (e.g. [`MemoriesLayer`] from [`register_memories`]).
    pub fn register(&mut self, layer: Box<dyn ContextLayer>) {
        self.layers.push(layer);
    }

    /// Convenience: register a [`MemoriesLayer`].
    pub fn register_memories(&mut self, layer: MemoriesLayer) {
        self.register(Box::new(layer));
    }

    /// Deterministic assembly — delegates to artifact-system `assemble_layers`.
    pub fn assemble(self, base_prompt: &str) -> String {
        assemble_layers(self.layers, base_prompt)
    }

    /// Assemble without consuming the registry (re-renders each layer).
    pub fn assemble_borrowed(&self, base_prompt: &str) -> String {
        let mut ordered: Vec<&dyn ContextLayer> =
            self.layers.iter().map(|l| l.as_ref() as &dyn ContextLayer).collect();
        ordered.sort_by(|a, b| {
            a.relevance()
                .cmp(&b.relevance())
                .then(a.priority().cmp(&b.priority()))
                .then_with(|| a.id().cmp(b.id()))
        });
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

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}
