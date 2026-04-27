//! WebSocket client for live invalidation events from popsicle-cloud.
//!
//! Connects to `ws[s]://<endpoint>/v1/sync/ws?token=<jwt>` and yields
//! [`WsEvent`] values until the stream closes or errors.

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;
use uuid::Uuid;

use crate::error::{Result, SyncError};

/// Event emitted by the server hub.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Changed {
        kind: String,
        id: Uuid,
        version: i64,
        #[serde(default)]
        deleted: bool,
    },
    DocUpdate {
        id: Uuid,
        update: String,
    },
}

/// Lightweight WS subscriber. Spawns a task that forwards parsed events
/// onto an [`mpsc`] channel; cancelling the receiver tears down the
/// connection.
pub struct WsClient;

impl WsClient {
    /// Connect to the server's `/v1/sync/ws` endpoint and stream events
    /// onto the returned channel.
    pub async fn connect(base_url: &str, access_token: &str) -> Result<mpsc::Receiver<WsEvent>> {
        let mut url = Url::parse(base_url).map_err(|e| SyncError::Other(format!("url: {e}")))?;
        match url.scheme() {
            "https" => url.set_scheme("wss").ok(),
            "http" => url.set_scheme("ws").ok(),
            "ws" | "wss" => Some(()),
            other => {
                return Err(SyncError::Other(format!("unsupported scheme {other}")));
            }
        };
        url.set_path("/v1/sync/ws");
        url.query_pairs_mut().append_pair("token", access_token);

        let (ws, _resp) = tokio_tungstenite::connect_async(url.as_str())
            .await
            .map_err(|e| SyncError::Other(format!("ws connect: {e}")))?;

        let (tx, rx) = mpsc::channel::<WsEvent>(64);
        tokio::spawn(async move {
            let (mut sink, mut stream) = ws.split();
            while let Some(msg) = stream.next().await {
                let msg = match msg {
                    Ok(m) => m,
                    Err(_) => break,
                };
                match msg {
                    Message::Text(txt) => {
                        if let Ok(ev) = serde_json::from_str::<WsEvent>(&txt)
                            && tx.send(ev).await.is_err()
                        {
                            break;
                        }
                    }
                    Message::Ping(p) => {
                        let _ = sink.send(Message::Pong(p)).await;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });
        Ok(rx)
    }
}
