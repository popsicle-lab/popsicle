//! Prompt-context assembly: per-document full-text-vs-summary selection, and the
//! pluggable [`ContextLayer`] registry with a **deterministic** assembly order.
//!
//! Mirrors:
//! - `acceptance.intent` › `ContextAssemblyOrdersByRelevance`:
//!   `relevance == High ==> includedFullText == true`,
//!   `relevance == Low ==> includedFullText == false`.
//! - ADR-004 contract 3: `ContextLayer` is exported here; layers register at
//!   runtime (e.g. skill-runtime injects its `MemoriesLayer`); `assemble_layers`
//!   orders by a deterministic total key that is **independent of registration
//!   order**.

/// Attention relevance of an upstream document / layer. Mirrors `enum Relevance`.
///
/// `Ord` runs `RelLow < RelMedium < RelHigh`; assembly keeps legacy direction
/// (Low first, High last → closest to the appended instruction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Relevance {
    RelLow,
    RelMedium,
    RelHigh,
}

/// A single upstream document considered for context injection. Mirrors
/// `type ContextDoc` (`relevance`, `includedFullText`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextDoc {
    pub relevance: Relevance,
}

/// Whether a document at `relevance` is injected as full text (vs. a summary).
///
/// `High → true`, `Low → false` are the two branches pinned by
/// `ContextAssemblyOrdersByRelevance`; `Medium` is summarized (not full text),
/// which the intent leaves unconstrained.
pub fn context_includes_full_text(relevance: Relevance) -> bool {
    matches!(relevance, Relevance::RelHigh)
}

/// A pluggable source of prompt-time context (ADR-004 contract 3).
pub trait ContextLayer {
    /// Stable identifier, used as the final deterministic tie-breaker.
    fn id(&self) -> &str;
    /// Where this layer sits in the attention ordering.
    fn relevance(&self) -> Relevance;
    /// Fixed priority tie-breaker within the same relevance (lower sorts first).
    /// Defaults to `0`.
    fn priority(&self) -> i32 {
        0
    }
    /// Rendered markdown; an empty/whitespace string is skipped.
    fn render(&self) -> String;
}

/// Order `layers` and join their non-empty renders with `\n\n---\n\n`, appending
/// `base_prompt` last (max attention).
///
/// The sort key is the deterministic total order
/// `(relevance ascending, priority ascending, id ascending)`. Because every
/// component is a property of the layer (never its registration index), the
/// output is identical for any registration permutation — the
/// `ContextOrderIndependentOfRegistration` property (closed by the permutation
/// test in `tests/intent_properties.rs`). Relevance ascending keeps High-relevance
/// content nearest the appended `base_prompt`, matching the legacy attention
/// rationale.
pub fn assemble_layers(mut layers: Vec<Box<dyn ContextLayer>>, base_prompt: &str) -> String {
    layers.sort_by(|a, b| {
        a.relevance()
            .cmp(&b.relevance())
            .then(a.priority().cmp(&b.priority()))
            .then_with(|| a.id().cmp(b.id()))
    });

    let mut sections: Vec<String> = layers
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

/// Convenience: the deterministic ordering key as an owned tuple. Exposed so the
/// property test can assert the key is a strict, registration-independent total
/// order matching [`assemble_layers`].
pub fn ordering_key(layer: &dyn ContextLayer) -> (Relevance, i32, String) {
    (layer.relevance(), layer.priority(), layer.id().to_string())
}
