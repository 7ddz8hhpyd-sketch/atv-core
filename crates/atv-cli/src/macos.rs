//! Native macOS controller: translates Apple TV remote buttons and touch gestures
//! into macOS CoreGraphics keyboard/mouse events, volume adjustments, and display sleep/wake.

use std::ffi::c_void;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use atv_core::{AtvDelegate, EventKind, TouchPhase};
use tracing::{info, warn};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

pub type CGEventRef = *mut c_void;
pub type CGEventSourceRef = *mut c_void;

const K_CGHID_EVENT_TAP: u32 = 0;
const K_CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
const K_CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
const K_CG_EVENT_MOUSE_MOVED: u32 = 5;
const K_CG_MOUSE_BUTTON_LEFT: u32 = 0;

#[link(name = "CoreGraphics", kind = "framework")]
#[link(name = "ApplicationServices", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CGEventCreate(source: CGEventSourceRef) -> CGEventRef;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        virtual_key: u16,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventCreateMouseEvent(
        source: CGEventSourceRef,
        mouse_type: u32,
        mouse_cursor_position: CGPoint,
        mouse_button: u32,
    ) -> CGEventRef;
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CFRelease(cf: *const c_void);
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: *const c_void;
    static kCFBooleanTrue: *const c_void;
    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *const c_void;
}

/// Check if the process has macOS Accessibility permission.
pub fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Request macOS Accessibility permission: triggers system modal dialog and opens System Settings.
pub fn request_accessibility_permission() -> bool {
    unsafe {
        if AXIsProcessTrusted() {
            return true;
        }

        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );

        let trusted = AXIsProcessTrustedWithOptions(dict);
        if !dict.is_null() {
            CFRelease(dict);
        }

        let _ = Command::new("/usr/bin/open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();

        trusted
    }
}

/// Virtual keycodes on macOS.
fn key_code_for_button(name: &str) -> Option<u16> {
    match name {
        "up" => Some(126),       // Arrow Up
        "down" => Some(125),     // Arrow Down
        "left" => Some(123),     // Arrow Left
        "right" => Some(124),    // Arrow Right
        "select" => Some(36),    // Return
        "menu" => Some(53),      // Escape
        "home" => Some(115),     // Home
        "play_pause" => Some(49),// Space
        "channel_up" | "page_up" => Some(116), // Page Up
        "channel_down" | "page_down" => Some(121), // Page Down
        "guide" => Some(48),     // Tab
        _ => None,
    }
}

pub struct MacDelegate {
    mouse_mode: bool,
    mouse_scale: f64,
    display_asleep: Arc<AtomicBool>,
}

impl MacDelegate {
    pub fn new(mouse_mode: bool) -> Self {
        if !is_accessibility_trusted() {
            eprintln!(
                "\n================================================================================\n\
                 ⚠️  [macOS Accessibility Permission Required]\n\
                 atv-cli requires Accessibility permission to simulate keyboard and mouse controls.\n\
                 \n\
                 👉 Requesting permission from macOS (system dialog prompted)...\n\
                 👉 Opening System Settings > Privacy & Security > Accessibility...\n\
                 👉 Please enable the toggle for your Terminal / atv-cli.\n\
                 ================================================================================\n"
            );
            request_accessibility_permission();
        } else {
            info!("macOS Accessibility permission confirmed");
        }

        Self {
            mouse_mode,
            mouse_scale: 2.0,
            display_asleep: Arc::new(AtomicBool::new(false)),
        }
    }

    fn send_key_press(virtual_key: u16) {
        unsafe {
            let event_down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), virtual_key, true);
            if !event_down.is_null() {
                CGEventPost(K_CGHID_EVENT_TAP, event_down);
                CFRelease(event_down as *const c_void);
            }
            let event_up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), virtual_key, false);
            if !event_up.is_null() {
                CGEventPost(K_CGHID_EVENT_TAP, event_up);
                CFRelease(event_up as *const c_void);
            }
        }
    }

    fn current_mouse_location() -> CGPoint {
        unsafe {
            let event = CGEventCreate(std::ptr::null_mut());
            if !event.is_null() {
                let loc = CGEventGetLocation(event);
                CFRelease(event as *const c_void);
                loc
            } else {
                CGPoint { x: 0.0, y: 0.0 }
            }
        }
    }

    fn mouse_move(&self, dx: f64, dy: f64) {
        let cur = Self::current_mouse_location();
        let target = CGPoint {
            x: cur.x + dx * self.mouse_scale,
            y: cur.y + dy * self.mouse_scale,
        };
        unsafe {
            let event = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                K_CG_EVENT_MOUSE_MOVED,
                target,
                K_CG_MOUSE_BUTTON_LEFT,
            );
            if !event.is_null() {
                CGEventPost(K_CGHID_EVENT_TAP, event);
                CFRelease(event as *const c_void);
            }
        }
    }

    fn mouse_click() {
        let cur = Self::current_mouse_location();
        unsafe {
            let down = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                K_CG_EVENT_LEFT_MOUSE_DOWN,
                cur,
                K_CG_MOUSE_BUTTON_LEFT,
            );
            if !down.is_null() {
                CGEventPost(K_CGHID_EVENT_TAP, down);
                CFRelease(down as *const c_void);
            }
            let up = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                K_CG_EVENT_LEFT_MOUSE_UP,
                cur,
                K_CG_MOUSE_BUTTON_LEFT,
            );
            if !up.is_null() {
                CGEventPost(K_CGHID_EVENT_TAP, up);
                CFRelease(up as *const c_void);
            }
        }
    }

    fn wake_display(&self) {
        if self.display_asleep.swap(false, Ordering::SeqCst) {
            let _ = Command::new("/usr/bin/caffeinate")
                .args(["-u", "-t", "1"])
                .spawn();
        }
    }

    fn sleep_display(&self) {
        self.display_asleep.store(true, Ordering::SeqCst);
        let _ = Command::new("/usr/bin/pmset")
            .arg("displaysleepnow")
            .spawn();
    }

    fn change_volume(delta: f64) {
        let script = format!(
            "set v to (output volume of (get volume settings)) + ({delta})\n\
             if v > 100 then set v to 100\n\
             if v < 0 then set v to 0\n\
             set volume output volume v"
        );
        let _ = Command::new("/usr/bin/osascript")
            .args(["-e", &script])
            .spawn();
    }

    fn toggle_mute() {
        let script = "set volume with output muted (not (output muted of (get volume settings)))";
        let _ = Command::new("/usr/bin/osascript")
            .args(["-e", script])
            .spawn();
    }
}

impl AtvDelegate for MacDelegate {
    fn on_button(&self, name: &str) {
        info!("[macOS] button: {name}");
        if name == "power" {
            if self.display_asleep.load(Ordering::SeqCst) {
                self.wake_display();
            } else {
                self.sleep_display();
            }
            return;
        }

        self.wake_display();

        if let Some(vk) = key_code_for_button(name) {
            Self::send_key_press(vk);
        } else {
            match name {
                "volume_up" => Self::change_volume(100.0 / 16.0),
                "volume_down" => Self::change_volume(-(100.0 / 16.0)),
                "mute" => Self::toggle_mute(),
                "screensaver" => {
                    let _ = Command::new("/usr/bin/open")
                        .args(["-a", "ScreenSaverEngine"])
                        .spawn();
                }
                "siri" => {
                    let _ = Command::new("/usr/bin/open")
                        .args(["-a", "Siri"])
                        .spawn();
                }
                "sleep" => self.sleep_display(),
                "wake" => self.wake_display(),
                _ => warn!("[macOS] no handler for button: {name}"),
            }
        }
    }

    fn on_touch(&self, dx: f64, dy: f64, phase: TouchPhase) {
        if self.mouse_mode && phase == TouchPhase::Moved {
            self.mouse_move(dx, dy);
        }
    }

    fn on_audio(&self, volume: f64, muted: bool) {
        info!("[macOS] audio sync: vol={volume:.2} muted={muted}");
        let percent = (volume * 100.0).round() as u32;
        let script = format!("set volume output volume {percent} with output muted {muted}");
        let _ = Command::new("/usr/bin/osascript")
            .args(["-e", &script])
            .spawn();
    }

    fn on_event(&self, kind: EventKind, detail: &str) {
        match kind {
            EventKind::MouseClick => {
                info!("[macOS] mouse click");
                Self::mouse_click();
            }
            EventKind::ClientConnected => {
                info!("[macOS] iPhone connected: {detail}");
            }
            EventKind::ClientDisconnected => {
                info!("[macOS] iPhone disconnected: {detail}");
            }
            _ => {
                info!("[macOS] event: {kind:?} {detail}");
            }
        }
    }
}
