//! Server role invariant: coordination only (contracts.intent#ServerNeverExecutesAgent).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerRole {
    pub executes_agent: bool,
    pub holds_api_keys: bool,
}

/// Compile-time / test-time anchor for `ServerNeverExecutesAgent`.
pub const fn server_role() -> ServerRole {
    ServerRole {
        executes_agent: false,
        holds_api_keys: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_never_executes_agent() {
        let role = server_role();
        assert!(!role.executes_agent);
        assert!(!role.holds_api_keys);
    }
}
