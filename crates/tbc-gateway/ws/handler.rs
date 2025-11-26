use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{StreamExt, SinkExt};
use crate::ws::state::WsState;
use crate::ws::router::route_ws_message;

/// WebSocket upgrade endpoint
pub async fn ws_upgrade(
    State(state): State<Arc<WsState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

/// Main WebSocket loop
async fn handle_ws(mut socket: WebSocket, _state: Arc<WsState>) {
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(body) => {
                // Process JSON → TGP → router → JSON
                match route_ws_message(&body).await {
                    Ok(resp) => {
                        let _ = socket.send(Message::Text(resp)).await;
                    }
                    Err(e) => {
                        let err_msg = format!(r#"{{"phase":"ERROR","id":"err-ws","code":"WS_DISPATCH","message":"{}"}}"#, e);
                        let _ = socket.send(Message::Text(err_msg)).await;
                    }
                }
            }

            Message::Close(_) => break,
            Message::Ping(p) => { let _ = socket.send(Message::Pong(p)).await; }
            Message::Pong(_) => {}
            Message::Binary(_) => {
                let _ = socket.send(Message::Text(
                    r#"{"phase":"ERROR","id":"err-bin","code":"UNSUPPORTED","message":"Binary frames not supported"}"#.into()
                )).await;
            }
        }
    }
}