use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::warn;

#[derive(Clone)]
pub struct SafetyManager {
    pub disarmed: Arc<AtomicBool>,
    pub user_active: Arc<AtomicBool>,
    pub auto_pause: Arc<AtomicBool>,
    // Track expected cursor position modified by GhostMouse to avoid false user_active triggers
    pub expected_x: Arc<AtomicI32>,
    pub expected_y: Arc<AtomicI32>,
}

impl SafetyManager {
    pub fn new() -> Self {
        let mgr = Self {
            disarmed: Arc::new(AtomicBool::new(false)),
            user_active: Arc::new(AtomicBool::new(false)),
            auto_pause: Arc::new(AtomicBool::new(false)), // Default auto_pause off so pranks execute smoothly by default
            expected_x: Arc::new(AtomicI32::new(-1)),
            expected_y: Arc::new(AtomicI32::new(-1)),
        };

        mgr.start_safety_monitor();
        mgr
    }

    /// Check if client is completely disarmed
    pub fn is_disarmed(&self) -> bool {
        self.disarmed.load(Ordering::Relaxed)
    }

    /// Safe check for visual/sound/popup pranks (only blocked by panic disarm)
    pub fn can_execute_visual_prank(&self) -> bool {
        !self.is_disarmed()
    }

    /// Safe check for mouse/keyboard manipulation pranks
    pub fn can_execute_input_prank(&self) -> bool {
        if self.is_disarmed() {
            return false;
        }
        if self.auto_pause.load(Ordering::Relaxed) && self.user_active.load(Ordering::Relaxed) {
            return false;
        }
        true
    }

    /// Notify safety monitor that GhostMouse moved cursor to (x, y)
    pub fn notify_cursor_set(&self, x: i32, y: i32) {
        self.expected_x.store(x, Ordering::Relaxed);
        self.expected_y.store(y, Ordering::Relaxed);
    }

    pub fn panic_disarm(&self) {
        self.disarmed.store(true, Ordering::SeqCst);
        warn!("🚨 SAFE MODE PANIC: Client has been completely DISARMED!");
    }

    fn start_safety_monitor(&self) {
        let disarmed = self.disarmed.clone();
        let user_active = self.user_active.clone();
        let expected_x = self.expected_x.clone();
        let expected_y = self.expected_y.clone();

        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            use windows_sys::Win32::Foundation::POINT;
            #[cfg(windows)]
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                GetAsyncKeyState, VK_CONTROL, VK_ESCAPE, VK_MENU, VK_SHIFT,
            };
            #[cfg(windows)]
            use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

            #[cfg(windows)]
            let mut last_pos = POINT { x: 0, y: 0 };
            #[cfg(windows)]
            unsafe {
                GetCursorPos(&mut last_pos);
            }

            let mut last_mouse_move = Instant::now();
            let mut esc_press_count = 0;
            let mut last_esc_time = Instant::now();

            loop {
                if disarmed.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }

                #[cfg(windows)]
                unsafe {
                    // 1. Check Panic Hotkey: Ctrl + Alt + Shift + K
                    let ctrl = (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;
                    let alt = (GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) != 0;
                    let shift = (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0;
                    let k_key = (GetAsyncKeyState(0x4B) as u16 & 0x8000) != 0;

                    if ctrl && alt && shift && k_key {
                        disarmed.store(true, Ordering::SeqCst);
                        warn!("🚨 EMERGENCY HOTKEY PRESSED (Ctrl+Alt+Shift+K)! DISARMING CLIENT!");
                    }

                    // 2. Emergency Escape Key Triple Press
                    let esc = (GetAsyncKeyState(VK_ESCAPE as i32) as u16 & 0x8000) != 0;
                    if esc {
                        if last_esc_time.elapsed() < Duration::from_millis(1500) {
                            esc_press_count += 1;
                            if esc_press_count >= 3 {
                                disarmed.store(true, Ordering::SeqCst);
                                warn!("🚨 TRIPLE ESCAPE PRESSED! DISARMING CLIENT!");
                            }
                        } else {
                            esc_press_count = 1;
                        }
                        last_esc_time = Instant::now();
                        std::thread::sleep(Duration::from_millis(200));
                    }

                    // 3. Check Physical Mouse Movement vs Expected Position
                    let mut current_pos = POINT { x: 0, y: 0 };
                    if GetCursorPos(&mut current_pos) != 0 {
                        let exp_x = expected_x.load(Ordering::Relaxed);
                        let exp_y = expected_y.load(Ordering::Relaxed);

                        let ref_x = if exp_x >= 0 { exp_x } else { last_pos.x };
                        let ref_y = if exp_y >= 0 { exp_y } else { last_pos.y };

                        let dx = (current_pos.x - ref_x).abs();
                        let dy = (current_pos.y - ref_y).abs();

                        // Significant deviation indicates actual physical user movement
                        if dx > 35 || dy > 35 {
                            user_active.store(true, Ordering::Relaxed);
                            last_mouse_move = Instant::now();
                        } else if last_mouse_move.elapsed() > Duration::from_millis(1500) {
                            user_active.store(false, Ordering::Relaxed);
                        }

                        last_pos = current_pos;
                        // Reset expected after reading
                        expected_x.store(-1, Ordering::Relaxed);
                        expected_y.store(-1, Ordering::Relaxed);
                    }
                }

                std::thread::sleep(Duration::from_millis(100));
            }
        });
    }
}
