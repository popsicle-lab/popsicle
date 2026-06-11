//! # storage
//!
//! Stack-bottom persistence types shared by `artifact-system` and `skill-runtime`
//! (ADR-004: `DocumentRow` 下沉). In-shadow: row types + in-memory store only;
//! SQLite wiring lands with `cli-ux` / legacy cutover.

mod document_row;
mod memory_store;
mod sqlite;
mod workspace;

pub use document_row::DocumentRow;
pub use memory_store::{MemoryDocumentStore, StoreError};
pub use sqlite::{SqliteStateDb, StateSnapshot};
pub use workspace::{
    DocCheckRow, DocCreateRow, IssueRow, PipelineStatusRow, RunRow, RunStartRow,
    StageCompleteRow, WorkspaceError, WorkspaceStore,
};
