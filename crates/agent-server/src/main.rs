//! `agent-server` binary — coordination API with optional PostgreSQL persistence.

use std::net::SocketAddr;

use agent_server::{router, AppState, Backend};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("AGENT_RUNTIME_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8787);
    let backend = Backend::from_env().await.expect("connect storage backend");
    eprintln!("agent-server storage={:?}", backend.kind());
    let state = AppState::new(backend);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let app = router(state);
    let listener = TcpListener::bind(addr).await.expect("bind");
    eprintln!("agent-server listening on http://{addr}");
    axum::serve(listener, app).await.expect("serve");
}
