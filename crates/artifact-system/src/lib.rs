//! # artifact-system
//!
//! Second **in-shadow** implementation slice (see `migration/progress.md` —
//! legacy stays the primary path; this crate is the new path under equivalence
//! observation). It owns popsicle's document-artifact engine: the `Document`
//! model, guard checks, prompt-context assembly, the work-item→task-chunk entity,
//! and structured extraction.
//!
//! This crate is **spec-driven**: every public type and operation mirrors a
//! Z3-verified intent in `products/artifact-system/intents/` or a contract in
//! `products/artifact-system/decisions/adr/ADR-004-artifact-system-seams.md`:
//!
//! - [`document`] — `Document` + `to_file_content`/`from_file_content`
//!   (`acceptance.intent` › `DocumentRoundTrips`). Unlike legacy (which
//!   `trim_start`s the body on parse), this round-trips **any** body byte-exactly
//!   so the formal `body' == body` holds unconditionally.
//! - [`guard`] — `check_guard` + the `UpstreamApprovalChecker` **port** (ADR-004
//!   contract 1; `contracts.intent`). The port references only artifact-owned
//!   types (`Document`/`GuardResult`) so the dependency arrow stays
//!   skill-runtime → artifact-system (no cycle). Totality
//!   (`invariants.intent` › `GuardResultIsTotal` / `EvaluateGuard`): every guard
//!   string yields a `GuardResult` or a deterministic `InvalidSkillDef` error —
//!   never a panic. Checklist completeness mirrors
//!   `acceptance.intent` › `GuardChecklistCompleteIffNoUnchecked`.
//! - [`context`] — `Relevance`, the `ContextLayer` **trait** + runtime
//!   `register`/`assemble_layers` (ADR-004 contract 3) with a **deterministic,
//!   registration-independent** total order, and per-doc full-text-vs-summary
//!   selection (`acceptance.intent` › `ContextAssemblyOrdersByRelevance`).
//! - [`extractor`] — total, kind-preserving extraction
//!   (`acceptance.intent` › `ExtractPreservesKind`; ADR-004 contract 2: 0
//!   production unwrap, no-match → empty `Vec`, never panics).
//! - [`task_chunk`] — the `work_item` → `task_chunk_entity` rename that preserves
//!   `kind` + the `fields` blob (`acceptance.intent` › `RenameWorkItemToTaskChunk`).
//!
//! Layout: this crate lives at `crates/artifact-system/` per **ADR-003**
//! (root-flat `crates/<slice>/`, `members = ["crates/*"]`).

pub mod context;
pub mod document;
pub mod extractor;
pub mod guard;
pub mod task_chunk;

pub use context::{
    assemble_layers, context_includes_full_text, ContextDoc, ContextLayer, Relevance,
};
pub use document::Document;
pub use extractor::{
    extract_bugs, extract_test_cases, extract_user_stories, ChunkKind, ExtractItem,
};
pub use guard::{
    check_guard, checklist_outcome, count_checkboxes, guard_outcome_for, guard_recognized,
    GuardCheck, GuardError, GuardOutcome, GuardResult, UpstreamApprovalChecker,
};
pub use task_chunk::{rename_work_item_to_task_chunk, CKind, TaskChunk, WorkItem};
