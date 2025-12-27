//! WebSocket handler for real-time progress updates

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures_util::SinkExt;
use tokio::sync::broadcast;

use crate::models::ProgressMessage;

/// Global broadcast channel for progress messages
type ProgressTx = broadcast::Sender<ProgressMessage>;

/// WebSocket handler
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

/// Handle WebSocket connection
async fn handle_socket(mut socket: WebSocket) {
    // TODO: Subscribe to progress updates from job processor
    // For now, send a heartbeat message every 10 seconds

    tracing::info!("WebSocket client connected");

    loop {
        tokio::select! {
            // Receive messages from client
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        tracing::debug!("Received message: {}", text);

                        // Client might send subscription requests
                        // e.g., {"type": "subscribe", "job_id": "123"}

                        // For now, just echo back
                        if socket.send(Message::Text(format!("Echo: {}", text))).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("Client closed connection");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }

            // Send heartbeat
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                let heartbeat = serde_json::json!({
                    "type": "heartbeat",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                if socket.send(Message::Text(heartbeat.to_string())).await.is_err() {
                    break;
                }
            }
        }
    }

    tracing::info!("WebSocket client disconnected");
}

/// Progress broadcaster for sending updates to all connected clients
pub struct ProgressBroadcaster {
    tx: ProgressTx,
}

impl ProgressBroadcaster {
    /// Create a new progress broadcaster
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    /// Send a progress message to all subscribers
    pub fn send(&self, message: ProgressMessage) -> Result<usize, broadcast::error::SendError<ProgressMessage>> {
        self.tx.send(message)
    }

    /// Subscribe to progress updates
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressMessage> {
        self.tx.subscribe()
    }
}

impl Default for ProgressBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
