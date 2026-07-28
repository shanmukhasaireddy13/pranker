use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use pranker_core::{ClientInfo, WsMessage};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;
use tracing::info;

type Tx = mpsc::UnboundedSender<WsMessage>;

#[derive(Clone, Default)]
struct AppState {
    clients: Arc<Mutex<HashMap<String, (ClientInfo, Tx)>>>,
    dashboards: Arc<Mutex<Vec<Tx>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::default();

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/ws", get(ws_handler))
        .with_state(state);

    // Render injects $PORT; fall back to 3030 for local dev
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Pranker Control Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind port {}", port));

    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> impl IntoResponse {
    let html = include_str!("../public/index.html");
    Html(html)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    // Forward internal channel messages to WebSocket output
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut current_client_id: Option<String> = None;

    // Register dashboard connection by default
    {
        let mut dash = state.dashboards.lock().unwrap();
        dash.push(tx.clone());
    }

    // Broadcast current state to newly connected client/dashboard
    broadcast_client_list(&state);

    // Read incoming WebSocket messages
    while let Some(Ok(msg)) = receiver.next().await {
        let text = match msg {
            Message::Text(t) => t,
            _ => continue,
        };

        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
            match ws_msg {
                WsMessage::RegisterClient {
                    client_id,
                    hostname,
                    os_info,
                } => {
                    info!("💻 Client registered: {} ({})", hostname, client_id);
                    current_client_id = Some(client_id.clone());

                    let info = ClientInfo {
                        client_id: client_id.clone(),
                        hostname,
                        os_info,
                        safe_mode_active: true,
                        disarmed: false,
                        user_active: false,
                        active_pranks: vec![],
                        safety_settings: Default::default(),
                    };

                    {
                        let mut clients = state.clients.lock().unwrap();
                        clients.insert(client_id, (info, tx.clone()));
                    }

                    broadcast_client_list(&state);
                }

                WsMessage::ClientHeartbeat {
                    client_id,
                    safe_mode_active,
                    disarmed,
                    user_active,
                    active_pranks,
                } => {
                    let mut clients = state.clients.lock().unwrap();
                    if let Some((info, _)) = clients.get_mut(&client_id) {
                        info.safe_mode_active = safe_mode_active;
                        info.disarmed = disarmed;
                        info.user_active = user_active;
                        info.active_pranks = active_pranks;
                    }
                    drop(clients);
                    broadcast_client_list(&state);
                }

                WsMessage::TogglePrank {
                    target_client_id,
                    prank,
                    enable,
                } => {
                    send_to_target(&state, &target_client_id, WsMessage::PrankCommand { prank, enable });
                }

                WsMessage::TriggerOneShot {
                    target_client_id,
                    prank,
                } => {
                    send_to_target(&state, &target_client_id, WsMessage::PrankCommand { prank, enable: true });
                }

                WsMessage::PanicDisarmAll { target_client_id } => {
                    info!("🚨 Panic disarm triggered for target: {:?}", target_client_id);
                    let target = target_client_id.unwrap_or_else(|| "all".to_string());
                    send_to_target(&state, &target, WsMessage::DisarmCommand);
                }

                WsMessage::UpdateSafetySettings {
                    target_client_id,
                    settings,
                } => {
                    let mut clients = state.clients.lock().unwrap();
                    if target_client_id == "all" {
                        for (info, _) in clients.values_mut() {
                            info.safety_settings = settings.clone();
                        }
                    } else if let Some((info, _)) = clients.get_mut(&target_client_id) {
                        info.safety_settings = settings;
                    }
                    drop(clients);
                    broadcast_client_list(&state);
                }

                _ => {}
            }
        }
    }

    // Cleanup on disconnect
    send_task.abort();
    if let Some(id) = current_client_id {
        info!("Client disconnected: {}", id);
        let mut clients = state.clients.lock().unwrap();
        clients.remove(&id);
    }
    broadcast_client_list(&state);
}

fn send_to_target(state: &AppState, target: &str, msg: WsMessage) {
    let clients = state.clients.lock().unwrap();
    if target == "all" {
        for (_, tx) in clients.values() {
            let _ = tx.send(msg.clone());
        }
    } else if let Some((_, tx)) = clients.get(target) {
        let _ = tx.send(msg);
    }
}

fn broadcast_client_list(state: &AppState) {
    let clients_list: Vec<ClientInfo> = {
        let clients = state.clients.lock().unwrap();
        clients.values().map(|(info, _)| info.clone()).collect()
    };

    let update_msg = WsMessage::ClientListUpdate {
        clients: clients_list,
    };

    let dashboards = state.dashboards.lock().unwrap();
    for dash_tx in dashboards.iter() {
        let _ = dash_tx.send(update_msg.clone());
    }
}
