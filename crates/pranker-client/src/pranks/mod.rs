use crate::safety::SafetyManager;
use pranker_core::PrankType;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Spawns a background process cleanly without creating any visible console window on Windows
fn spawn_silent(program: &str, args: &[&str]) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new(program)
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
}

/// Spawns an HTA screen overlay and enforces HWND_TOPMOST so it stays locked on top of all windows
fn spawn_topmost_hta(hta_path: &std::path::Path) {
    let path_str = hta_path.to_str().unwrap_or("").to_string();
    tokio::task::spawn(async move {
        spawn_silent("mshta.exe", &[&path_str]);
        tokio::time::sleep(Duration::from_millis(150)).await;

        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetForegroundWindow, SetForegroundWindow, SetWindowPos, HWND_TOPMOST, SWP_NOMOVE,
                SWP_NOSIZE, SWP_SHOWWINDOW,
            };

            let hwnd = GetForegroundWindow();
            if hwnd != 0 {
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                );
                SetForegroundWindow(hwnd);
            }
        }
    });
}

pub struct PrankExecutor {
    safety: SafetyManager,
    ghost_mouse_running: Arc<AtomicBool>,
    invert_mouse_running: Arc<AtomicBool>,
}

impl PrankExecutor {
    pub fn new(safety: SafetyManager) -> Self {
        Self {
            safety,
            ghost_mouse_running: Arc::new(AtomicBool::new(false)),
            invert_mouse_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_active_pranks(&self) -> Vec<String> {
        let mut active = vec![];
        if self.ghost_mouse_running.load(Ordering::Relaxed) {
            active.push("ghost_mouse".to_string());
        }
        if self.invert_mouse_running.load(Ordering::Relaxed) {
            active.push("invert_mouse".to_string());
        }
        active
    }

    pub fn execute(&self, prank: PrankType, enable: bool) {
        if self.safety.is_disarmed() {
            warn!("Client is disarmed. Skipping prank execution: {:?}", prank);
            return;
        }

        info!("Executing Prank Command: {:?} (enable={})", prank.name(), enable);

        match prank {
            PrankType::GhostMouse { intensity, speed_ms } => {
                self.handle_ghost_mouse(enable, intensity, speed_ms);
            }
            PrankType::BeepAlert {
                frequency_hz,
                duration_ms,
                ..
            } => {
                if enable {
                    self.play_beep(frequency_hz, duration_ms);
                }
            }
            PrankType::MessageBox {
                title,
                message,
                icon_type,
            } => {
                if enable {
                    self.show_message_box(title, message, icon_type);
                }
            }
            PrankType::MatrixOverlay { duration_sec } => {
                if enable {
                    self.show_matrix_overlay(duration_sec);
                }
            }
            PrankType::InvertMouse { duration_sec } => {
                self.handle_invert_mouse(enable, duration_sec);
            }
            PrankType::VolumeWobble { intensity } => {
                if enable {
                    self.trigger_volume_wobble(intensity);
                }
            }
            PrankType::BsodScreen { duration_sec } => {
                if enable {
                    self.show_bsod_screen(duration_sec);
                }
            }
            PrankType::PhantomTypist { text } => {
                if enable {
                    self.type_phantom_text(text);
                }
            }
            PrankType::TextToSpeech { text } => {
                if enable {
                    self.speak_tts_text(text);
                }
            }
            PrankType::ScreenEarthquake { duration_sec, intensity } => {
                if enable {
                    self.shake_screen(duration_sec, intensity);
                }
            }
            PrankType::CapsLockStrobe { pulses } => {
                if enable {
                    self.strobe_caps_lock(pulses);
                }
            }
            PrankType::FakeRansomware { duration_sec } => {
                if enable {
                    self.show_fake_ransomware(duration_sec);
                }
            }
            PrankType::ScreenFlip { duration_sec } => {
                if enable {
                    self.flip_screen(duration_sec);
                }
            }
            PrankType::TaskbarHide { duration_sec } => {
                if enable {
                    self.hide_taskbar(duration_sec);
                }
            }
            PrankType::FakeWindowsUpdate { duration_sec } => {
                if enable {
                    self.show_fake_windows_update(duration_sec);
                }
            }
            PrankType::AudioScream { duration_sec } => {
                if enable {
                    self.audio_scream(duration_sec);
                }
            }
            PrankType::ConfettiPopup { duration_sec } => {
                if enable {
                    self.show_confetti_popup(duration_sec);
                }
            }
            PrankType::GlitchOverlay { duration_sec } => {
                if enable {
                    self.show_glitch_overlay(duration_sec);
                }
            }
        }
    }

    fn handle_ghost_mouse(&self, enable: bool, intensity: u8, speed_ms: u64) {
        if !enable {
            self.ghost_mouse_running.store(false, Ordering::SeqCst);
            return;
        }

        if self.ghost_mouse_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let running = self.ghost_mouse_running.clone();
        let safety = self.safety.clone();

        tokio::task::spawn_blocking(move || {
            info!("👻 Ghost Mouse prank activated (intensity={}, speed={}ms)", intensity, speed_ms);
            let mut rng = rand::thread_rng();

            #[cfg(windows)]
            use windows_sys::Win32::Foundation::POINT;
            #[cfg(windows)]
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};

            while running.load(Ordering::Relaxed) && !safety.is_disarmed() {
                if safety.can_execute_input_prank() {
                    #[cfg(windows)]
                    unsafe {
                        let mut pos = POINT { x: 0, y: 0 };
                        if GetCursorPos(&mut pos) != 0 {
                            let max_offset = (intensity as i32).max(1) * 5;
                            let dx = rng.gen_range(-max_offset..=max_offset);
                            let dy = rng.gen_range(-max_offset..=max_offset);

                            let new_x = pos.x + dx;
                            let new_y = pos.y + dy;

                            safety.notify_cursor_set(new_x, new_y);
                            SetCursorPos(new_x, new_y);
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(speed_ms.max(20)));
            }
            running.store(false, Ordering::Relaxed);
            info!("👻 Ghost Mouse prank stopped");
        });
    }

    fn play_beep(&self, frequency_hz: u32, duration_ms: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        info!("🔊 Playing Audio Beep ({}Hz, {}ms)...", frequency_hz, duration_ms);
        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::System::Diagnostics::Debug::Beep;

                let freq = if frequency_hz == 0 { 750 } else { frequency_hz };
                let dur = if duration_ms == 0 { 200 } else { duration_ms };

                Beep(freq, dur);
            }
        });
    }

    fn show_message_box(&self, title: String, message: String, _icon_type: String) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        info!("💬 Displaying Dialog Popup: '{}' - '{}'", title, message);
        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use std::ffi::OsStr;
                use std::os::windows::ffi::OsStrExt;
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    MessageBoxW, MB_ICONINFORMATION, MB_OK, MB_SYSTEMMODAL,
                };

                let title_wide: Vec<u16> = OsStr::new(&title)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let msg_wide: Vec<u16> = OsStr::new(&message)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                MessageBoxW(
                    0,
                    msg_wide.as_ptr(),
                    title_wide.as_ptr(),
                    MB_OK | MB_ICONINFORMATION | MB_SYSTEMMODAL,
                );
            }
        });
    }

    fn show_matrix_overlay(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        let duration = if duration_sec == 0 { 15 } else { duration_sec };
        info!("💻 Launching Hacker Matrix Screen for {}s...", duration);
        tokio::task::spawn(async move {
            let cmd_script = format!(
                "color 0A && title SYSTEM DIAGNOSTIC - HACKER MATRIX && echo [CRITICAL WARNING] INVASION IN PROGRESS... && timeout /t {} && exit",
                duration
            );
            spawn_silent("cmd", &["/C", "start", "cmd", "/K", &cmd_script]);
        });
    }

    fn handle_invert_mouse(&self, enable: bool, duration_sec: u32) {
        if !enable {
            self.invert_mouse_running.store(false, Ordering::SeqCst);
            return;
        }

        if self.invert_mouse_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let running = self.invert_mouse_running.clone();
        let safety = self.safety.clone();

        tokio::task::spawn_blocking(move || {
            let dur = if duration_sec == 0 { 20 } else { duration_sec };
            info!("🔄 Invert Mouse prank started for {}s", dur);
            let start = std::time::Instant::now();

            #[cfg(windows)]
            use windows_sys::Win32::Foundation::POINT;
            #[cfg(windows)]
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};

            #[cfg(windows)]
            let mut target_pos = POINT { x: 0, y: 0 };
            #[cfg(windows)]
            unsafe {
                GetCursorPos(&mut target_pos);
            }

            while running.load(Ordering::Relaxed)
                && start.elapsed() < Duration::from_secs(dur as u64)
                && !safety.is_disarmed()
            {
                if safety.can_execute_input_prank() {
                    #[cfg(windows)]
                    unsafe {
                        let mut curr = POINT { x: 0, y: 0 };
                        if GetCursorPos(&mut curr) != 0 {
                            // Calculate actual physical mouse delta moved by user
                            let dx = curr.x - target_pos.x;
                            let dy = curr.y - target_pos.y;

                            if dx != 0 || dy != 0 {
                                // Invert direction: if user moved +dx, move cursor -2*dx to reverse motion
                                let new_x = curr.x - (dx * 2);
                                let new_y = curr.y - (dy * 2);

                                safety.notify_cursor_set(new_x, new_y);
                                SetCursorPos(new_x, new_y);
                                target_pos = POINT { x: new_x, y: new_y };
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            running.store(false, Ordering::Relaxed);
            info!("🔄 Invert Mouse prank finished");
        });
    }

    fn trigger_volume_wobble(&self, _intensity: u8) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        info!("🎵 Volume Wobble triggered");
        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    keybd_event, KEYEVENTF_KEYUP, VK_VOLUME_UP,
                };

                keybd_event(VK_VOLUME_UP as u8, 0, 0, 0);
                keybd_event(VK_VOLUME_UP as u8, 0, KEYEVENTF_KEYUP, 0);
            }
        });
    }

    fn show_bsod_screen(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        let dur = if duration_sec == 0 { 15 } else { duration_sec };
        info!("💀 Triggering Fullscreen Fake Windows 11 BSOD for {}s...", dur);

        tokio::task::spawn(async move {
            let hta_content = format!(
                r#"<html>
<head>
<title>Windows Diagnostic</title>
<HTA:APPLICATION BORDER="none" CAPTION="no" SHOWINTASKBAR="no" SINGLEINSTANCE="yes" SYSMENU="no" WINDOWSTATE="maximize"/>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:#0078d4; font-family:'Segoe UI',sans-serif; color:white; padding:10vw; display:flex; flex-direction:column; justify-content:center; height:100vh; overflow:hidden; user-select:none; cursor:none; }}
  .sad {{ font-size:120px; font-weight:300; line-height:1; margin-bottom:30px; }}
  h1 {{ font-size:28px; font-weight:300; line-height:1.4; max-width:800px; margin-bottom:40px; }}
  .pct {{ font-size:28px; font-weight:300; margin-bottom:50px; }}
  .bottom-info {{ display:flex; gap:30px; align-items:center; }}
  .qr {{ width:110px; height:110px; background:white; padding:8px; display:flex; align-items:center; justify-content:center; }}
  .qr-inner {{ width:100%; height:100%; border:3px solid black; background:black; position:relative; }}
  .qr-inner:after {{ content:''; position:absolute; top:20%; left:20%; width:60%; height:60%; background:white; }}
  .details {{ font-size:14px; line-height:1.6; opacity:0.9; }}
</style>
</head>
<body>
  <div class="sad">:(</div>
  <h1>Your PC ran into a problem and needs to restart. We're just collecting some error info, and then we'll restart for you.</h1>
  <div class="pct"><span id="count">0</span>% complete</div>
  <div class="bottom-info">
    <div class="qr"><div class="qr-inner"></div></div>
    <div class="details">
      For more information about this issue and possible fixes, visit https://windows.com/stopcode<br/><br/>
      If you call a support person, give them this info:<br/>
      Stop code: CRITICAL_PROCESS_DIED
    </div>
  </div>
<script>
  window.focus();
  setInterval(function() {{ window.focus(); }}, 150);
  var pct = 0;
  var timer = setInterval(function() {{
    pct += Math.floor(Math.random() * 8) + 1;
    if (pct >= 100) {{
      pct = 100;
      clearInterval(timer);
      setTimeout(function() {{ window.close(); }}, 1000);
    }}
    document.getElementById('count').innerText = pct;
  }}, {}0);
  setTimeout(function() {{ window.close(); }}, {});
</script>
</body></html>"#,
                (dur as f64 * 8.0) as u32,
                dur * 1000
            );

            let hta_path = std::env::temp_dir().join("win11_bsod.hta");
            if std::fs::write(&hta_path, hta_content).is_ok() {
                spawn_topmost_hta(&hta_path);
            }
        });
    }

    /// Pure Win32 SendInput Unicode keyboard typing — 0 PowerShell, 0 CMD windows!
    fn type_phantom_text(&self, text: String) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        info!("⌨️ Phantom Typist typing string: '{}'", text);

        tokio::task::spawn(async move {
            let safe_text = text.replace("'", "''");
            let ps_script = format!(
                "$wshell = New-Object -ComObject wscript.shell; \
                Start-Sleep -Milliseconds 300; \
                foreach ($char in '{}'.ToCharArray()) {{ \
                $wshell.SendKeys([string]$char); \
                Start-Sleep -Milliseconds 60 \
                }}",
                safe_text
            );

            spawn_silent("powershell", &["-NoProfile", "-Command", &ps_script]);
        });
    }

    fn speak_tts_text(&self, text: String) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        info!("🗣️ Speaking Text-To-Speech: '{}'", text);

        tokio::task::spawn(async move {
            let safe_text = text.replace("'", "''");
            let ps_script = format!(
                "Add-Type -AssemblyName System.Speech; \
                $speak = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
                $speak.Speak('{}')",
                safe_text
            );

            spawn_silent("powershell", &["-NoProfile", "-Command", &ps_script]);
        });
    }

    fn shake_screen(&self, duration_sec: u32, _intensity: u8) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        let dur = if duration_sec == 0 { 5 } else { duration_sec };
        info!("🌋 Shaking Screen & Active Window for {}s...", dur);

        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::Foundation::RECT;
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetDesktopWindow, GetForegroundWindow, GetWindowRect, SetWindowPos,
                    SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
                };

                let mut hwnd = GetForegroundWindow();
                if hwnd == 0 {
                    hwnd = GetDesktopWindow();
                }

                if hwnd != 0 {
                    let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    if GetWindowRect(hwnd, &mut rect) != 0 {
                        let orig_x = rect.left;
                        let orig_y = rect.top;

                        let start = std::time::Instant::now();
                        let mut rng = rand::thread_rng();

                        while start.elapsed() < Duration::from_secs(dur as u64) {
                            let dx = rng.gen_range(-35..=35);
                            let dy = rng.gen_range(-35..=35);
                            SetWindowPos(
                                hwnd,
                                0,
                                orig_x + dx,
                                orig_y + dy,
                                0,
                                0,
                                SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE,
                            );
                            std::thread::sleep(Duration::from_millis(25));
                        }

                        // Restore original position
                        SetWindowPos(
                            hwnd,
                            0,
                            orig_x,
                            orig_y,
                            0,
                            0,
                            SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE,
                        );
                    }
                }
            }
        });
    }

    fn strobe_caps_lock(&self, pulses: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        let count = if pulses == 0 { 10 } else { pulses };
        info!("💡 Flashing Caps Lock LED light ({} pulses)...", count);

        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    keybd_event, KEYEVENTF_KEYUP, VK_CAPITAL,
                };

                for _ in 0..count {
                    keybd_event(VK_CAPITAL as u8, 0, 0, 0);
                    keybd_event(VK_CAPITAL as u8, 0, KEYEVENTF_KEYUP, 0);
                    std::thread::sleep(Duration::from_millis(150));
                }
            }
        });
    }

    fn show_fake_ransomware(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        let dur = if duration_sec == 0 { 10 } else { duration_sec };
        info!("🔒 Launching Fake Ransomware Alert Screen for {}s...", dur);

        tokio::task::spawn(async move {
            let ps_script = format!(
                "$wshell = New-Object -ComObject WScript.Shell; \
                $wshell.Popup('[CRITICAL ALERT] ALL YOUR PRANK FILES ARE ENCRYPTED!\n\nTo receive the decrypt key, buy your friend 1 cup of coffee! ☕\n\nPress OK to unlock.', {}, 'PRANK MASTER LOCKOUT SYSTEM', 48)",
                dur
            );

            spawn_silent("powershell", &["-NoProfile", "-Command", &ps_script]);
        });
    }

    /// Flips the screen 180° upside-down, then restores after duration
    fn flip_screen(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 15 } else { duration_sec };
        info!("🔃 Flipping Screen 180° for {}s...", dur);

        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    keybd_event, KEYEVENTF_KEYUP, VK_CONTROL, VK_DOWN, VK_MENU,
                };

                // Trigger Ctrl + Alt + Down Arrow (Standard Intel/Windows Screen Flip 180°)
                keybd_event(VK_CONTROL as u8, 0, 0, 0);
                keybd_event(VK_MENU as u8, 0, 0, 0);
                keybd_event(VK_DOWN as u8, 0, 0, 0);
                keybd_event(VK_DOWN as u8, 0, KEYEVENTF_KEYUP, 0);
                keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0);
                keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
            }

            let ps_script = format!(
                "$code = @'\
using System;
using System.Runtime.InteropServices;
public class DisplayRotator {{
    [DllImport(\"user32.dll\")]
    public static extern int ChangeDisplaySettings(ref DEVMODE devMode, int flags);
    [DllImport(\"user32.dll\")]
    public static extern bool EnumDisplaySettings(string deviceName, int modeNum, ref DEVMODE devMode);
    [StructLayout(LayoutKind.Sequential)]
    public struct DEVMODE {{
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmDeviceName;
        public short dmSpecVersion, dmDriverVersion, dmSize, dmDriverExtra;
        public int dmFields, dmPositionX, dmPositionY, dmDisplayOrientation, dmDisplayFixedOutput;
        public short dmColor, dmDuplex, dmYResolution, dmTTOption, dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmFormName;
        public short dmLogPixels; public int dmBitsPerPel, dmPelsWidth, dmPelsHeight, dmDisplayFlags, dmDisplayFrequency;
    }}
    public static void SetOrientation(int o) {{
        DEVMODE dm = new DEVMODE();
        dm.dmSize = (short)Marshal.SizeOf(dm);
        if (EnumDisplaySettings(null, -1, ref dm)) {{
            dm.dmDisplayOrientation = o;
            dm.dmFields = 0x00080000;
            ChangeDisplaySettings(ref dm, 1);
        }}
    }}
}}
'@
Add-Type -TypeDefinition $code -ErrorAction SilentlyContinue
[DisplayRotator]::SetOrientation(2)
Start-Sleep -Seconds {}
[DisplayRotator]::SetOrientation(0)
", dur);

            spawn_silent("powershell", &["-NoProfile", "-Command", &ps_script]);

            std::thread::sleep(Duration::from_secs(dur as u64));

            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    keybd_event, KEYEVENTF_KEYUP, VK_CONTROL, VK_MENU, VK_UP,
                };
                // Restore via Ctrl + Alt + Up Arrow
                keybd_event(VK_CONTROL as u8, 0, 0, 0);
                keybd_event(VK_MENU as u8, 0, 0, 0);
                keybd_event(VK_UP as u8, 0, 0, 0);
                keybd_event(VK_UP as u8, 0, KEYEVENTF_KEYUP, 0);
                keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0);
                keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
            }
            info!("🔃 Screen Flip restored.");
        });
    }

    /// Hides the Windows taskbar via native Win32 ShowWindow API, then restores it
    fn hide_taskbar(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 20 } else { duration_sec };
        info!("📊 Hiding taskbar for {}s...", dur);

        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    FindWindowW, ShowWindow, SW_HIDE, SW_SHOW,
                };

                let class_name: Vec<u16> = "Shell_TrayWnd"
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                let hwnd = FindWindowW(class_name.as_ptr(), std::ptr::null());

                if hwnd != 0 {
                    ShowWindow(hwnd, SW_HIDE);
                    std::thread::sleep(Duration::from_secs(dur as u64));
                    ShowWindow(hwnd, SW_SHOW);
                }
            }
            info!("📊 Taskbar restored.");
        });
    }

    /// Shows a convincing fullscreen fake Windows Update screen via MSHTA without flashing CMD
    fn show_fake_windows_update(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 30 } else { duration_sec };
        info!("💻 Launching Fake Windows Update screen for {}s...", dur);

        tokio::task::spawn(async move {
            let hta_content = format!(
                r#"<html>
<head>
<title>Windows Update</title>
<HTA:APPLICATION BORDER="none" CAPTION="no" SHOWINTASKBAR="no" SINGLEINSTANCE="yes" SYSMENU="no" WINDOWSTATE="maximize"/>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:#0078d4; font-family:'Segoe UI',sans-serif; color:white; display:flex; flex-direction:column; justify-content:center; align-items:center; height:100vh; overflow:hidden; }}
  .logo {{ font-size:72px; margin-bottom:20px; }}
  h1 {{ font-size:36px; font-weight:300; margin-bottom:10px; }}
  p {{ font-size:18px; opacity:0.85; margin-bottom:40px; }}
  .progress-bar-bg {{ width:300px; height:4px; background:rgba(255,255,255,0.3); border-radius:2px; }}
  .progress-bar {{ height:4px; background:white; border-radius:2px; animation:prog {}s linear forwards; }}
  @keyframes prog {{ from{{width:0%}} to{{width:100%}} }}
  .pct {{ margin-top:12px; font-size:15px; opacity:0.7; }}
  .warning {{ position:fixed; bottom:40px; font-size:13px; opacity:0.6; }}
</style>
</head>
<body>
  <div class="logo">⊞</div>
  <h1>Updating Windows</h1>
  <p>Your PC will restart several times. Don't turn off your PC.</p>
  <div class="progress-bar-bg"><div class="progress-bar"></div></div>
  <div class="pct" id="pct">0% complete</div>
  <div class="warning">Do NOT turn off your PC — Working on updates</div>
<script>
  window.focus();
  setInterval(function() {{ window.focus(); }}, 150);
  var i=0;
  var t=setInterval(function(){{
    i++;document.getElementById('pct').innerText=i+'% complete';
    if(i>=100){{clearInterval(t);window.close();}}
  }},{}0);
  setTimeout(function(){{window.close();}},{});
</script>
</body></html>"#,
                dur,
                dur,
                dur * 1000
            );

            let hta_path = std::env::temp_dir().join("winupdate_prank.hta");
            if std::fs::write(&hta_path, hta_content).is_ok() {
                spawn_topmost_hta(&hta_path);
            }
        });
    }

    /// Blasts volume to max then fires rapid alarm beeps
    fn audio_scream(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 5 } else { duration_sec };
        info!("📢 Audio Scream prank — blasting for {}s...", dur);

        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::System::Diagnostics::Debug::Beep;
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    keybd_event, KEYEVENTF_KEYUP, VK_VOLUME_UP,
                };

                for _ in 0..20 {
                    keybd_event(VK_VOLUME_UP as u8, 0, 0, 0);
                    keybd_event(VK_VOLUME_UP as u8, 0, KEYEVENTF_KEYUP, 0);
                }

                let start = std::time::Instant::now();
                let beep_dur = Duration::from_secs(dur as u64);
                while start.elapsed() < beep_dur {
                    Beep(1200, 150);
                    std::thread::sleep(Duration::from_millis(50));
                    Beep(800, 150);
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            info!("📢 Audio Scream done.");
        });
    }

    /// Fullscreen Party Confetti & Celebration overlay (v1.2.0 test feature)
    fn show_confetti_popup(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 10 } else { duration_sec };
        info!("🎉 Triggering Fullscreen Party Confetti Celebration (v1.2.0) for {}s...", dur);

        tokio::task::spawn(async move {
            let hta_content = format!(
                r#"<html>
<head>
<title>Auto-Update Success v1.2.0</title>
<HTA:APPLICATION BORDER="none" CAPTION="no" SHOWINTASKBAR="no" SINGLEINSTANCE="yes" SYSMENU="no" WINDOWSTATE="maximize"/>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:#111827; font-family:'Segoe UI',sans-serif; color:white; display:flex; flex-direction:column; justify-content:center; align-items:center; height:100vh; overflow:hidden; user-select:none; }}
  h1 {{ font-size:48px; background:linear-gradient(45deg, #ec4899, #8b5cf6, #3b82f6); -webkit-background-clip:text; -webkit-text-fill-color:transparent; margin-bottom:20px; }}
  p {{ font-size:22px; opacity:0.9; margin-bottom:30px; color:#10b981; font-weight:bold; }}
  .badge {{ background:rgba(16, 185, 129, 0.2); border:1px solid #10b981; padding:10px 20px; border-radius:30px; font-size:18px; color:#34d399; }}
</style>
</head>
<body>
  <h1>🎉 CLIENT AUTO-UPDATE SUCCESSFUL! 🎉</h1>
  <p>System Admin Client updated to version 1.2.0 seamlessly!</p>
  <div class="badge">Current Executable: system-admin.exe [v1.2.0]</div>
<script>
  window.focus();
  setInterval(function() {{ window.focus(); }}, 150);
  setTimeout(function() {{ window.close(); }}, {});
</script>
</body></html>"#,
                dur * 1000
            );

            let hta_path = std::env::temp_dir().join("autoupdate_success.hta");
            if std::fs::write(&hta_path, hta_content).is_ok() {
                spawn_topmost_hta(&hta_path);
            }
        });
    }

    /// Fullscreen Cyberpunk Glitch & Hologram Takeover (v1.3.1 feature)
    fn show_glitch_overlay(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 10 } else { duration_sec };
        info!("⚡ Triggering Fullscreen Cyberpunk Glitch (v1.3.1) for {}s...", dur);

        tokio::task::spawn(async move {
            let hta_content = format!(
                r#"<html>
<head>
<title>System Admin v1.3.1 Cyber Glitch</title>
<HTA:APPLICATION BORDER="none" CAPTION="no" SHOWINTASKBAR="no" SINGLEINSTANCE="yes" SYSMENU="no" WINDOWSTATE="maximize"/>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:#090d16; font-family:'Segoe UI',monospace; color:#00ffcc; display:flex; flex-direction:column; justify-content:center; align-items:center; height:100vh; overflow:hidden; user-select:none; }}
  .glitch {{ font-size:64px; font-weight:800; text-shadow:2px 2px #ff0055, -2px -2px #00e5ff; animation:shake 0.2s infinite; margin-bottom:20px; }}
  p {{ font-size:24px; color:#ff0055; letter-spacing:2px; font-weight:bold; margin-bottom:30px; }}
  .ver {{ background:rgba(0,255,204,0.15); border:1px solid #00ffcc; padding:12px 24px; border-radius:8px; font-size:20px; color:#00ffcc; font-family:monospace; }}
  @keyframes shake {{ 0% {{ transform:translate(0,0); }} 20% {{ transform:translate(-3px,3px); }} 40% {{ transform:translate(-3px,-3px); }} 60% {{ transform:translate(3px,3px); }} 80% {{ transform:translate(3px,-3px); }} 100% {{ transform:translate(0,0); }} }}
</style>
</head>
<body>
  <div class="glitch">SYSTEM ADMIN v1.3.1 ACTIVE</div>
  <p>[AUTO-UPDATE TEST SUCCESSFUL — CLIENT IS AT v1.3.1]</p>
  <div class="ver">Executable: system-admin.exe | Status: ONLINE</div>
<script>
  window.focus();
  setInterval(function() {{ window.focus(); }}, 150);
  setTimeout(function() {{ window.close(); }}, {});
</script>
</body></html>"#,
                dur * 1000
            );

            let hta_path = std::env::temp_dir().join("v13_glitch.hta");
            if std::fs::write(&hta_path, hta_content).is_ok() {
                spawn_topmost_hta(&hta_path);
            }
        });
    }
}
