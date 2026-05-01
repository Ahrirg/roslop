// TODO REWRITE THIS GARBAGE HALF CHATGPT CODE "want to show prototype today, so yeah till then it will work"

use axum::{
    Router,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::{broadcast, oneshot};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub pending: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
}

#[derive(Serialize)]
struct WsMessage {
    id: String,
    command: String,
}

#[derive(Deserialize, Debug)]
struct ClientMessage {
    id: Option<String>,
    response: Option<String>,
    error: Option<String>,
}

impl AppState {
    pub async fn send(&self, msg: impl Into<String>) -> Option<String> {
        let id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();

        self.pending.lock().unwrap().insert(id.clone(), tx);

        let payload = WsMessage {
            id: id.clone(),
            command: msg.into(),
        };

        let json = serde_json::to_string(&payload).ok()?;
        let _ = self.tx.send(json);

        rx.await.ok()
    }
}

pub fn run_server() -> AppState {
    let (tx, _) = broadcast::channel::<String>(100);

    let state = AppState {
        tx: tx.clone(),
        pending: Arc::new(Mutex::new(HashMap::new())),
    };

    let state_for_server = state.clone();

    tokio::spawn(async move {
        async fn ws_handler(
            ws: WebSocketUpgrade,
            State(state): State<AppState>,
        ) -> impl IntoResponse {
            ws.on_upgrade(move |socket| handle_socket(socket, state))
        }

        async fn handle_socket(mut socket: WebSocket, state: AppState) {
            let mut rx = state.tx.subscribe();

            loop {
                tokio::select! {
                    msg = socket.recv() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(parsed) = serde_json::from_str::<ClientMessage>(&text) {
                                    if let Some(id) = parsed.id {
                                        if let Some(resp) = parsed.response {

                                            if let Some(tx) = state.pending.lock().unwrap().remove(&id) {
                                                let _ = tx.send(resp);
                                            }
                                        }
                                    }
                                }
                            }

                            Some(Ok(Message::Close(_))) | None => break,
                            _ => {}
                        }
                    }

                    msg = rx.recv() => {
                        if let Ok(text) = msg {
                            let _ = socket.send(Message::Text(text.into())).await;
                        }
                    }
                }
            }
        }

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state_for_server);

        let addr = SocketAddr::from(([0, 0, 0, 0], 6967));
        println!("ws://{}/ws", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    state
}