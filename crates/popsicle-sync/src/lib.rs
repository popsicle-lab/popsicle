//! Popsicle sync client.
//!
//! This crate provides a transport-agnostic [`SyncClient`] trait plus an
//! HTTP/JSON implementation [`HttpSyncClient`] that talks to any server
//! conforming to `docs/sync-api.md` in the Popsicle OSS repo.
//!
//! No server URL is hardcoded. Callers configure the endpoint via
//! `popsicle_core::storage::config::SyncSection`.

pub mod client;
pub mod conflict;
pub mod crdt;
pub mod error;
pub mod http;
pub mod path;
pub mod types;
pub mod ws;

pub use client::SyncClient;
pub use crdt::CrdtDoc;
pub use error::{Result, SyncError};
pub use http::{Credentials, HttpSyncClient};
pub use types::*;
pub use ws::{WsClient, WsEvent};

/// Wire-format schema version recognized by this client.
pub const SCHEMA_VERSION: u32 = 1;
