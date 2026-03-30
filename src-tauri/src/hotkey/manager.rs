use anyhow::Result;
use rdev::{EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub struct GlobalHotkeyManager {
    #[allow(dead_code)]
    listening: Arc<AtomicBool>,
}

impl GlobalHotkeyManager {
    pub fn new() -> Self {
        Self {
            listening: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Parse a hotkey string into an `rdev::Key`.
    fn parse_key(hotkey_str: &str) -> Option<Key> {
        match hotkey_str {
            "F1" => Some(Key::F1),
            "F2" => Some(Key::F2),
            "F3" => Some(Key::F3),
            "F4" => Some(Key::F4),
            "F5" => Some(Key::F5),
            "F6" => Some(Key::F6),
            "F7" => Some(Key::F7),
            "F8" => Some(Key::F8),
            "F9" => Some(Key::F9),
            "F10" => Some(Key::F10),
            "F11" => Some(Key::F11),
            "F12" => Some(Key::F12),
            // F13-F20: use rdev Unknown with platform-specific scan codes
            "F13" => Some(Key::Unknown(91)),
            "F14" => Some(Key::Unknown(92)),
            "F15" => Some(Key::Unknown(93)),
            "F16" => Some(Key::Unknown(94)),
            "F17" => Some(Key::Unknown(95)),
            "F18" => Some(Key::Unknown(96)),
            "F19" => Some(Key::Unknown(97)),
            "F20" => Some(Key::Unknown(98)),
            "RightCtrl" | "ControlRight" => Some(Key::ControlRight),
            "LeftCtrl" | "ControlLeft" => Some(Key::ControlLeft),
            "RightAlt" | "AltGr" => Some(Key::AltGr),
            "LeftAlt" | "Alt" => Some(Key::Alt),
            "RightShift" | "ShiftRight" => Some(Key::ShiftRight),
            "LeftShift" | "ShiftLeft" => Some(Key::ShiftLeft),
            "CapsLock" => Some(Key::CapsLock),
            "Tab" => Some(Key::Tab),
            "Escape" => Some(Key::Escape),
            _ => {
                log::warn!("Unknown hotkey: {hotkey_str}");
                None
            }
        }
    }

    /// Start listening for the given hotkey.
    /// Emits `hotkey_pressed` and `hotkey_released` events on the Tauri app.
    pub fn start_listening(&self, hotkey_str: String, app_handle: AppHandle) -> Result<()> {
        if self.listening.swap(true, Ordering::SeqCst) {
            return Ok(()); // already listening
        }

        let listening = Arc::clone(&self.listening);

        std::thread::spawn(move || {
            let target_key = match Self::parse_key(&hotkey_str) {
                Some(k) => k,
                None => {
                    log::error!("Cannot listen: unrecognised hotkey '{hotkey_str}'");
                    listening.store(false, Ordering::SeqCst);
                    return;
                }
            };

            let app = app_handle.clone();
            let cb = move |event: rdev::Event| match event.event_type {
                EventType::KeyPress(key) if key == target_key => {
                    let _ = app.emit("hotkey_pressed", hotkey_str.clone());
                }
                EventType::KeyRelease(key) if key == target_key => {
                    let _ = app.emit("hotkey_released", hotkey_str.clone());
                }
                _ => {}
            };

            if let Err(e) = rdev::listen(cb) {
                log::error!("rdev listen error: {e:?}");
            }
            listening.store(false, Ordering::SeqCst);
        });

        Ok(())
    }

    #[allow(dead_code)]
    pub fn stop_listening(&self) {
        self.listening.store(false, Ordering::SeqCst);
    }
}

impl Default for GlobalHotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}
