//! The `work_item` → `task_chunk_entity` rename (PDR-001 / ADR-004).
//!
//! Mirrors `acceptance.intent` › `RenameWorkItemToTaskChunk(t)`:
//! `ensure t.kind' == t.kind; ensure t.fieldsHash' == t.fieldsHash` — renaming
//! the legacy `WorkItem` entity to `TaskChunk` preserves both the `kind` and the
//! free-form `fields` blob (modeled here as a sorted key→value map whose
//! `fields_hash` fingerprint must be unchanged).

use std::collections::BTreeMap;

/// Chunk kind. Mirrors `enum CKind` in `acceptance.intent` and the legacy
/// `WorkItemKind { Bug, Story, TestCase }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CKind {
    CBug,
    CStory,
    CTestCase,
}

/// FNV-1a over the deterministically-ordered `fields` map. The modeled
/// `fieldsHash: Int` of `type TaskChunk`. (Tests also compare the maps directly,
/// so preservation does not rest on hash collisions.)
fn fields_hash(fields: &BTreeMap<String, String>) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    };
    // BTreeMap iterates in sorted key order → stable regardless of insertion order.
    for (k, v) in fields {
        feed(k.as_bytes());
        feed(b"=");
        feed(v.as_bytes());
        feed(b"\n");
    }
    hash
}

/// Legacy entity being migrated. Mirrors the parts of `WorkItem` the rename must
/// preserve: `kind` + the `fields` blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItem {
    pub kind: CKind,
    pub fields: BTreeMap<String, String>,
}

/// Renamed entity. Same `kind` and `fields` as its source `WorkItem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskChunk {
    pub kind: CKind,
    pub fields: BTreeMap<String, String>,
}

impl TaskChunk {
    /// Fingerprint of the `fields` blob (`type TaskChunk.fieldsHash`).
    pub fn fields_hash(&self) -> u64 {
        fields_hash(&self.fields)
    }
}

impl WorkItem {
    /// Fingerprint of the `fields` blob (for pre/post-rename comparison).
    pub fn fields_hash(&self) -> u64 {
        fields_hash(&self.fields)
    }
}

/// Rename a `WorkItem` to a `TaskChunk`, preserving `kind` and `fields` exactly
/// (`RenameWorkItemToTaskChunk`).
pub fn rename_work_item_to_task_chunk(wi: WorkItem) -> TaskChunk {
    TaskChunk {
        kind: wi.kind,
        fields: wi.fields,
    }
}
