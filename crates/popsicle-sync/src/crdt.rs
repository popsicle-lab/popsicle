//! Client-side CRDT helpers (yrs/Yjs).
//!
//! A Popsicle document is modelled as a yrs [`Doc`] containing a single
//! shared `Text` named `"content"`. The client persists the doc as a binary
//! `update_v1` blob in `.popsicle/.sync/<doc-id>.crdt`. To push, the client
//! diffs against the server's last known state vector; to pull, the client
//! applies remote updates onto the local doc.

use std::path::Path;

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact, Update};

use crate::error::{Result, SyncError};

const TEXT_NAME: &str = "content";

/// Wraps a yrs [`Doc`] for a single Popsicle document.
pub struct CrdtDoc {
    doc: Doc,
}

impl CrdtDoc {
    /// Create a new empty doc.
    pub fn new() -> Self {
        let doc = Doc::new();
        let _ = doc.get_or_insert_text(TEXT_NAME);
        Self { doc }
    }

    /// Reconstruct from a single `update_v1` blob.
    pub fn from_update_bytes(bytes: &[u8]) -> Result<Self> {
        let doc = Doc::new();
        let _ = doc.get_or_insert_text(TEXT_NAME);
        if !bytes.is_empty() {
            let update = Update::decode_v1(bytes)
                .map_err(|e| SyncError::Other(format!("decode update: {e}")))?;
            doc.transact_mut()
                .apply_update(update)
                .map_err(|e| SyncError::Other(format!("apply update: {e}")))?;
        }
        Ok(Self { doc })
    }

    /// Reconstruct from a base64-encoded `update_v1` blob (server format).
    pub fn from_update_b64(s: &str) -> Result<Self> {
        let bytes = B64
            .decode(s)
            .map_err(|e| SyncError::Other(format!("base64 decode: {e}")))?;
        Self::from_update_bytes(&bytes)
    }

    /// Apply an additional base64-encoded update (e.g. from the server).
    pub fn apply_update_b64(&mut self, s: &str) -> Result<()> {
        let bytes = B64
            .decode(s)
            .map_err(|e| SyncError::Other(format!("base64 decode: {e}")))?;
        let update = Update::decode_v1(&bytes)
            .map_err(|e| SyncError::Other(format!("decode update: {e}")))?;
        self.doc
            .transact_mut()
            .apply_update(update)
            .map_err(|e| SyncError::Other(format!("apply update: {e}")))?;
        Ok(())
    }

    /// Replace the entire text content. Returns the resulting update bytes
    /// (i.e. only the diff, in `update_v1` format) suitable for pushing.
    pub fn replace_text(&mut self, new_text: &str) -> Result<Vec<u8>> {
        let text = self.doc.get_or_insert_text(TEXT_NAME);
        let before_sv = self.doc.transact().state_vector();
        {
            let mut txn = self.doc.transact_mut();
            let len = text.len(&txn);
            text.remove_range(&mut txn, 0, len);
            text.insert(&mut txn, 0, new_text);
        }
        let txn = self.doc.transact();
        Ok(txn.encode_state_as_update_v1(&before_sv))
    }

    /// Current text content of the `"content"` field.
    pub fn text(&self) -> String {
        let text = self.doc.get_or_insert_text(TEXT_NAME);
        let txn = self.doc.transact();
        text.get_string(&txn)
    }

    /// Encode the entire current state as a single `update_v1` blob.
    pub fn encode_state(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    /// Encode a diff against `remote_sv` (base64 state vector from server) as
    /// the bytes a client should push.
    pub fn diff_against(&self, remote_sv_b64: &str) -> Result<Vec<u8>> {
        let sv_bytes = B64
            .decode(remote_sv_b64)
            .map_err(|e| SyncError::Other(format!("base64 decode sv: {e}")))?;
        let sv = StateVector::decode_v1(&sv_bytes)
            .map_err(|e| SyncError::Other(format!("decode sv: {e}")))?;
        let txn = self.doc.transact();
        Ok(txn.encode_state_as_update_v1(&sv))
    }

    /// Persist the doc to `.popsicle/.sync/<doc-id>.crdt`.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SyncError::Other(format!("mkdir: {e}")))?;
        }
        std::fs::write(path, self.encode_state())
            .map_err(|e| SyncError::Other(format!("write crdt: {e}")))?;
        Ok(())
    }

    /// Load from `.popsicle/.sync/<doc-id>.crdt`, or return an empty doc if
    /// absent.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let bytes =
            std::fs::read(path).map_err(|e| SyncError::Other(format!("read crdt: {e}")))?;
        Self::from_update_bytes(&bytes)
    }
}

impl Default for CrdtDoc {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode update bytes as base64 (the wire format).
pub fn b64_encode(bytes: &[u8]) -> String {
    B64.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yrs::updates::encoder::Encode;

    #[test]
    fn round_trip_replace_and_diff() {
        let mut a = CrdtDoc::new();
        let _ = a.replace_text("hello").unwrap();
        let state = a.encode_state();

        let mut b = CrdtDoc::from_update_bytes(&state).unwrap();
        assert_eq!(b.text(), "hello");

        // B updates locally, computes diff against A's current state vector.
        let a_sv_bytes = a.doc.transact().state_vector().encode_v1();
        let a_sv = b64_encode(&a_sv_bytes);
        let _ = b.replace_text("hello world").unwrap();
        let diff = b.diff_against(&a_sv).unwrap();
        a.apply_update_b64(&b64_encode(&diff)).unwrap();
        assert_eq!(a.text(), "hello world");
    }
}
