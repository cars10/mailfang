use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use crate::web::AppState;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketEvent {
    NewMail,
    EmailRead,
    EmailDeleted,
}

#[derive(serde::Serialize, Clone)]
pub struct WebSocketMessage {
    pub event: WebSocketEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<crate::db::EmailListRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipients: Option<Vec<String>>,
}

pub type BroadcastSender = broadcast::Sender<WebSocketMessage>;

pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcast.subscribe();

    loop {
        tokio::select! {
            msg_result = rx.recv() => {
                match msg_result {
                    Ok(msg) => {
                        let json_msg = serde_json::to_string(&msg).unwrap_or_else(|_| "{}".to_string());
                        if sender
                            .send(axum::extract::ws::Message::Text(json_msg.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            client_msg = receiver.next() => {
                match client_msg {
                    Some(Ok(msg)) => {
                        match msg {
                            axum::extract::ws::Message::Ping(payload) => {
                                if sender.send(axum::extract::ws::Message::Pong(payload)).await.is_err() {
                                    break;
                                }
                            }
                            axum::extract::ws::Message::Close(_) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(_)) | None => {
                        break;
                    }
                }
            }
        }
    }
}
