use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "config")]
pub enum PrankType {
    /// Subtle random mouse movement
    GhostMouse {
        intensity: u8,   // 1 to 10 scale
        speed_ms: u64,   // Delay between jitters
    },
    /// System alert beep sound
    BeepAlert {
        frequency_hz: u32,
        duration_ms: u32,
        repeat_interval_sec: u32,
    },
    /// Custom harmless Windows message box dialog
    MessageBox {
        title: String,
        message: String,
        icon_type: String, // "info", "warning", "error"
    },
    /// Matrix rain harmless overlay window
    MatrixOverlay {
        duration_sec: u32,
    },
    /// Invert mouse axis temporarily
    InvertMouse {
        duration_sec: u32,
    },
    /// Volume fluctuation nudge
    VolumeWobble {
        intensity: u8,
    },
    /// Fullscreen Fake Windows 11 BSOD Screen
    BsodScreen {
        duration_sec: u32,
    },
    /// Types funny text into active focused window
    PhantomTypist {
        text: String,
    },
    /// Text-To-Speech voice output out loud
    TextToSpeech {
        text: String,
    },
    /// Shakes currently focused window like an earthquake
    ScreenEarthquake {
        duration_sec: u32,
        intensity: u8,
    },
    /// Flashes keyboard Caps Lock LED light
    CapsLockStrobe {
        pulses: u32,
    },
    /// Fake Hacker Red Ransomware Takeover Overlay
    FakeRansomware {
        duration_sec: u32,
    },
    /// Rotates the display 180° (upside-down screen)
    ScreenFlip {
        duration_sec: u32,
    },
    /// Hides the Windows taskbar making it look broken
    TaskbarHide {
        duration_sec: u32,
    },
    /// Fullscreen fake Windows Update that blocks input
    FakeWindowsUpdate {
        duration_sec: u32,
    },
    /// Blasts volume to max and plays loud system alert
    AudioScream {
        duration_sec: u32,
    },
    /// NEW v1.2.0 FEATURE: Fullscreen Party Confetti & Disco Lights Celebration
    ConfettiPopup {
        duration_sec: u32,
    },
}

impl PrankType {
    pub fn name(&self) -> &'static str {
        match self {
            PrankType::GhostMouse { .. } => "Ghost Mouse",
            PrankType::BeepAlert { .. } => "Beep Alert",
            PrankType::MessageBox { .. } => "Message Box",
            PrankType::MatrixOverlay { .. } => "Matrix Overlay",
            PrankType::InvertMouse { .. } => "Invert Mouse",
            PrankType::VolumeWobble { .. } => "Volume Wobble",
            PrankType::BsodScreen { .. } => "Fake BSOD",
            PrankType::PhantomTypist { .. } => "Phantom Typist",
            PrankType::TextToSpeech { .. } => "Text To Speech",
            PrankType::ScreenEarthquake { .. } => "Screen Earthquake",
            PrankType::CapsLockStrobe { .. } => "Caps Lock Strobe",
            PrankType::FakeRansomware { .. } => "Fake Ransomware",
            PrankType::ScreenFlip { .. } => "Screen Flip",
            PrankType::TaskbarHide { .. } => "Taskbar Hide",
            PrankType::FakeWindowsUpdate { .. } => "Fake Windows Update",
            PrankType::AudioScream { .. } => "Audio Scream",
            PrankType::ConfettiPopup { .. } => "Party Confetti (v1.2)",
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            PrankType::GhostMouse { .. } => "ghost_mouse",
            PrankType::BeepAlert { .. } => "beep_alert",
            PrankType::MessageBox { .. } => "message_box",
            PrankType::MatrixOverlay { .. } => "matrix_overlay",
            PrankType::InvertMouse { .. } => "invert_mouse",
            PrankType::VolumeWobble { .. } => "volume_wobble",
            PrankType::BsodScreen { .. } => "bsod_screen",
            PrankType::PhantomTypist { .. } => "phantom_typist",
            PrankType::TextToSpeech { .. } => "text_to_speech",
            PrankType::ScreenEarthquake { .. } => "screen_earthquake",
            PrankType::CapsLockStrobe { .. } => "caps_lock_strobe",
            PrankType::FakeRansomware { .. } => "fake_ransomware",
            PrankType::ScreenFlip { .. } => "screen_flip",
            PrankType::TaskbarHide { .. } => "taskbar_hide",
            PrankType::FakeWindowsUpdate { .. } => "fake_windows_update",
            PrankType::AudioScream { .. } => "audio_scream",
            PrankType::ConfettiPopup { .. } => "confetti_popup",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySettings {
    pub auto_pause_on_user_input: bool,
    pub max_prank_timeout_sec: u64,
    pub disarmed: bool,
}

impl Default for SafetySettings {
    fn default() -> Self {
        Self {
            auto_pause_on_user_input: false,
            max_prank_timeout_sec: 60,
            disarmed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_id: String,
    pub hostname: String,
    pub os_info: String,
    pub version: String,
    pub safe_mode_active: bool,
    pub disarmed: bool,
    pub user_active: bool,
    pub active_pranks: Vec<String>,
    pub safety_settings: SafetySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum WsMessage {
    // Sent by Client to Server
    RegisterClient {
        client_id: String,
        hostname: String,
        os_info: String,
        version: String,
    },
    ClientHeartbeat {
        client_id: String,
        safe_mode_active: bool,
        disarmed: bool,
        user_active: bool,
        active_pranks: Vec<String>,
        version: String,
    },

    // Sent by Controller Dashboard to Server
    TogglePrank {
        target_client_id: String,
        prank: PrankType,
        enable: bool,
    },
    TriggerOneShot {
        target_client_id: String,
        prank: PrankType,
    },
    PanicDisarmAll {
        target_client_id: Option<String>,
    },
    UpdateSafetySettings {
        target_client_id: String,
        settings: SafetySettings,
    },
    TriggerAutoUpdate {
        target_client_id: String,
        download_url: String,
    },

    // Server broadcasts to Controllers / Clients
    ClientListUpdate {
        clients: Vec<ClientInfo>,
    },
    PrankCommand {
        prank: PrankType,
        enable: bool,
    },
    DisarmCommand,
    ServerNotification {
        level: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prank_type_serialization() {
        let prank = PrankType::GhostMouse {
            intensity: 5,
            speed_ms: 100,
        };
        let json = serde_json::to_string(&prank).unwrap();
        let serde_prank: PrankType = serde_json::from_str(&json).unwrap();
        assert_eq!(prank, serde_prank);
    }

    #[test]
    fn test_ws_message_panic_disarm() {
        let msg = WsMessage::PanicDisarmAll {
            target_client_id: Some("target-123".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("PanicDisarmAll"));
    }
}
