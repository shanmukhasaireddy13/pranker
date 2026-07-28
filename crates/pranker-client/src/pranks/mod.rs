use crate::safety::SafetyManager;
use pranker_core::PrankType;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

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
            PrankType::CDEjectLoop { count } => {
                if enable {
                    self.eject_cd_loop(count);
                }
            }
            PrankType::AudioScream { duration_sec } => {
                if enable {
                    self.audio_scream(duration_sec);
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

            let _ = tokio::process::Command::new("cmd")
                .args(["/C", "start", "cmd", "/K", &cmd_script])
                .spawn();
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
            info!("🔄 Invert Mouse prank started for {}s", duration_sec);
            let start = std::time::Instant::now();

            #[cfg(windows)]
            use windows_sys::Win32::Foundation::POINT;
            #[cfg(windows)]
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};

            #[cfg(windows)]
            let mut last_pos = POINT { x: 0, y: 0 };
            #[cfg(windows)]
            unsafe {
                GetCursorPos(&mut last_pos);
            }

            while running.load(Ordering::Relaxed)
                && start.elapsed() < Duration::from_secs(duration_sec as u64)
                && !safety.is_disarmed()
            {
                if safety.can_execute_input_prank() {
                    #[cfg(windows)]
                    unsafe {
                        let mut curr = POINT { x: 0, y: 0 };
                        if GetCursorPos(&mut curr) != 0 {
                            let dx = curr.x - last_pos.x;
                            let dy = curr.y - last_pos.y;

                            if dx.abs() > 1 || dy.abs() > 1 {
                                let new_x = curr.x - (dx * 2);
                                let new_y = curr.y - (dy * 2);
                                safety.notify_cursor_set(new_x, new_y);
                                SetCursorPos(new_x, new_y);
                            }
                            last_pos = curr;
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(30));
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

    // --- CRAZY PRANKS ---

    fn show_bsod_screen(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        let dur = if duration_sec == 0 { 10 } else { duration_sec };
        info!("💀 Triggering Fake Windows 11 BSOD Screen for {}s...", dur);

        tokio::task::spawn(async move {
            let ps_script = format!(
                "$wshell = New-Object -ComObject WScript.Shell; \
                $wshell.Popup('Your PC ran into a problem and needs to restart. We are just collecting some error info, and then we will restart for you. (0% complete)\\n\\nStop Code: CRITICAL_PROCESS_DIED', {}, 'Windows Diagnostic System Error', 16)",
                dur
            );

            let _ = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_script])
                .spawn();
        });
    }

    fn type_phantom_text(&self, text: String) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        info!("⌨️ Phantom Typist typing string: '{}'", text);

        tokio::task::spawn(async move {
            let safe_text = text.replace("'", "''");
            let ps_script = format!(
                "$wshell = New-Object -ComObject wscript.shell; \
                Start-Sleep -Milliseconds 500; \
                foreach ($char in '{}'.ToCharArray()) {{ \
                $wshell.SendKeys([string]$char); \
                Start-Sleep -Milliseconds 80 \
                }}",
                safe_text
            );

            let _ = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_script])
                .spawn();
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

            let _ = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_script])
                .spawn();
        });
    }

    fn shake_screen(&self, duration_sec: u32, _intensity: u8) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }

        let dur = if duration_sec == 0 { 4 } else { duration_sec };
        info!("🌋 Shaking Active Window for {}s...", dur);

        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::Foundation::RECT;
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetForegroundWindow, GetWindowRect, SetWindowPos, SWP_NOZORDER,
                };

                let hwnd = GetForegroundWindow();
                if hwnd != 0 {
                    let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    if GetWindowRect(hwnd, &mut rect) != 0 {
                        let orig_x = rect.left;
                        let orig_y = rect.top;
                        let w = rect.right - rect.left;
                        let h = rect.bottom - rect.top;

                        let start = std::time::Instant::now();
                        let mut rng = rand::thread_rng();

                        while start.elapsed() < Duration::from_secs(dur as u64) {
                            let dx = rng.gen_range(-20..=20);
                            let dy = rng.gen_range(-20..=20);
                            SetWindowPos(hwnd, 0, orig_x + dx, orig_y + dy, w, h, SWP_NOZORDER);
                            std::thread::sleep(Duration::from_millis(30));
                        }

                        // Restore original position
                        SetWindowPos(hwnd, 0, orig_x, orig_y, w, h, SWP_NOZORDER);
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
                $wshell.Popup('[CRITICAL ALERT] ALL YOUR PRANK FILES ARE ENCRYPTED!\\n\\nTo receive the decrypt key, buy your friend 1 cup of coffee! ☕\\n\\nPress OK to unlock.', {}, 'PRANK MASTER LOCKOUT SYSTEM', 48)",
                dur
            );

            let _ = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_script])
                .spawn();
        });
    }

    // ── NEW HIGH-LEVEL PRANKS ──────────────────────────────────────

    /// Flips the screen 180° upside-down, then restores after duration
    fn flip_screen(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 15 } else { duration_sec };
        info!("🔃 Flipping Screen 180° for {}s...", dur);

        tokio::task::spawn(async move {
            // DMDO_180 = 2, DMDO_DEFAULT = 0
            let ps_flip = "\
                Add-Type -TypeDefinition @'\
                using System;using System.Runtime.InteropServices;\
                public class Display {\
                  [DllImport(\"user32.dll\")] public static extern bool EnumDisplaySettings(string n,int m,ref DEVMODE d);\
                  [DllImport(\"user32.dll\")] public static extern int ChangeDisplaySettingsEx(string n,ref DEVMODE d,IntPtr h,uint f,IntPtr p);\
                  [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Ansi)] public struct DEVMODE {\
                    [MarshalAs(UnmanagedType.ByValTStr,SizeConst=32)] public string dmDeviceName;\
                    public short dmSpecVersion,dmDriverVersion,dmSize,dmDriverExtra;\
                    public int dmFields,dmPositionX,dmPositionY,dmDisplayOrientation,dmDisplayFixedOutput;\
                    public short dmColor,dmDuplex,dmYResolution,dmTTOption,dmCollate;\
                    [MarshalAs(UnmanagedType.ByValTStr,SizeConst=32)] public string dmFormName;\
                    public short dmLogPixels; public int dmBitsPerPel,dmPelsWidth,dmPelsHeight,dmDisplayFlags,dmDisplayFrequency;\
                  }\
                }\
                '@ -Language CSharp;\
                $d=New-Object Display+DEVMODE; $d.dmSize=[System.Runtime.InteropServices.Marshal]::SizeOf($d);\
                [Display]::EnumDisplaySettings($null,-1,[ref]$d)|Out-Null;\
                $d.dmDisplayOrientation=2; $d.dmFields=0x80;\
                [Display]::ChangeDisplaySettingsEx($null,[ref]$d,[IntPtr]::Zero,0,[IntPtr]::Zero)|Out-Null";

            let ps_restore = "\
                Add-Type -TypeDefinition @'\
                using System;using System.Runtime.InteropServices;\
                public class Display2 {\
                  [DllImport(\"user32.dll\")] public static extern bool EnumDisplaySettings(string n,int m,ref DEVMODE d);\
                  [DllImport(\"user32.dll\")] public static extern int ChangeDisplaySettingsEx(string n,ref DEVMODE d,IntPtr h,uint f,IntPtr p);\
                  [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Ansi)] public struct DEVMODE {\
                    [MarshalAs(UnmanagedType.ByValTStr,SizeConst=32)] public string dmDeviceName;\
                    public short dmSpecVersion,dmDriverVersion,dmSize,dmDriverExtra;\
                    public int dmFields,dmPositionX,dmPositionY,dmDisplayOrientation,dmDisplayFixedOutput;\
                    public short dmColor,dmDuplex,dmYResolution,dmTTOption,dmCollate;\
                    [MarshalAs(UnmanagedType.ByValTStr,SizeConst=32)] public string dmFormName;\
                    public short dmLogPixels; public int dmBitsPerPel,dmPelsWidth,dmPelsHeight,dmDisplayFlags,dmDisplayFrequency;\
                  }\
                }\
                '@ -Language CSharp;\
                $d=New-Object Display2+DEVMODE; $d.dmSize=[System.Runtime.InteropServices.Marshal]::SizeOf($d);\
                [Display2]::EnumDisplaySettings($null,-1,[ref]$d)|Out-Null;\
                $d.dmDisplayOrientation=0; $d.dmFields=0x80;\
                [Display2]::ChangeDisplaySettingsEx($null,[ref]$d,[IntPtr]::Zero,0,[IntPtr]::Zero)|Out-Null";

            let _ = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps_flip])
                .spawn();

            tokio::time::sleep(Duration::from_secs(dur as u64)).await;

            let _ = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps_restore])
                .spawn();

            info!("🔃 Screen Flip restored.");
        });
    }

    /// Hides the Windows taskbar for a duration, then restores it
    fn hide_taskbar(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 20 } else { duration_sec };
        info!("📊 Hiding taskbar for {}s...", dur);

        tokio::task::spawn(async move {
            let ps_hide = format!(
                "$tb=@('Shell_TrayWnd','Shell_SecondaryTrayWnd');\
                foreach($n in $tb){{$h=[System.IntPtr]::new([System.Runtime.InteropServices.Marshal]::ReadIntPtr([System.Runtime.InteropServices.Marshal]::GetHINSTAN​CE([System.Reflection.Assembly]::GetExecutingAssembly().GetModules()[0])))|Out-Null}};\
                Add-Type -Name W -Namespace '' -MemberDefinition '[DllImport(\"user32.dll\")] public static extern IntPtr FindWindow(string a,string b); [DllImport(\"user32.dll\")] public static extern bool ShowWindow(IntPtr h,int n);';\
                $h=[W]::FindWindow('Shell_TrayWnd',$null); [W]::ShowWindow($h,0)|Out-Null;\
                Start-Sleep -Seconds {};\
                [W]::ShowWindow($h,4)|Out-Null",
                dur
            );

            let _ = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_hide])
                .spawn();

            info!("📊 Taskbar restored.");
        });
    }

    /// Shows a convincing fullscreen fake Windows Update screen
    fn show_fake_windows_update(&self, duration_sec: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let dur = if duration_sec == 0 { 30 } else { duration_sec };
        info!("💻 Launching Fake Windows Update screen for {}s...", dur);

        tokio::task::spawn(async move {
            // Write a convincing HTA (HTML Application) that goes fullscreen
            let hta_content = format!(r#"<html>
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
  var i=0;
  var t=setInterval(function(){{
    i++;document.getElementById('pct').innerText=i+'% complete';
    if(i>=100){{clearInterval(t);window.close();}}
  }},{}0);
  setTimeout(function(){{window.close();}},{});
</script>
</body></html>"#, dur, dur, dur * 1000);

            let hta_path = std::env::temp_dir().join("winupdate_prank.hta");
            if std::fs::write(&hta_path, hta_content).is_ok() {
                let _ = tokio::process::Command::new("mshta.exe")
                    .arg(hta_path.to_str().unwrap_or(""))
                    .spawn();
            }
        });
    }

    /// Ejects the CD/DVD tray repeatedly
    fn eject_cd_loop(&self, count: u32) {
        if !self.safety.can_execute_visual_prank() {
            return;
        }
        let n = if count == 0 { 5 } else { count };
        info!("💿 CD Eject Loop — {} times...", n);

        tokio::task::spawn(async move {
            for _ in 0..n {
                let ps = "Add-Type -TypeDefinition 'using System;using System.Runtime.InteropServices;public class Mci{[DllImport(\"winmm.dll\",CharSet=CharSet.Ansi)]public static extern int mciSendString(string c,System.Text.StringBuilder r,int l,IntPtr h);}' -Language CSharp; [Mci]::mciSendString('set cdaudio door open',([System.Text.StringBuilder]::new(128)),128,[IntPtr]::Zero)|Out-Null";
                let _ = tokio::process::Command::new("powershell")
                    .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps])
                    .spawn();
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            info!("💿 CD Eject Loop done.");
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
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    keybd_event, KEYEVENTF_KEYUP, VK_VOLUME_UP,
                };
                use windows_sys::Win32::System::Diagnostics::Debug::Beep;

                // Slam volume up 20 times to ensure max volume
                for _ in 0..20 {
                    keybd_event(VK_VOLUME_UP as u8, 0, 0, 0);
                    keybd_event(VK_VOLUME_UP as u8, 0, KEYEVENTF_KEYUP, 0);
                }

                // Blast rapid alarm beeps for the duration
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
}
