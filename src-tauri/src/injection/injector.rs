use anyhow::{Context, Result};
use arboard::Clipboard;
use rdev::{simulate, EventType, Key};
use std::time::Duration;

pub struct TextInjector;

impl TextInjector {
    pub fn new() -> Self {
        Self
    }

    pub fn inject(&self, text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new().context("Failed to open clipboard")?;

        // Save current clipboard content (best-effort)
        let previous = clipboard.get_text().ok();

        // Set clipboard to the new text
        clipboard
            .set_text(text.to_string())
            .context("Failed to set clipboard text")?;

        // Brief pause to allow clipboard to settle
        std::thread::sleep(Duration::from_millis(50));

        // Simulate paste keystroke
        Self::simulate_paste()?;

        // Restore previous clipboard content after a short delay
        let previous_owned = previous.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            if let Ok(mut cb) = Clipboard::new() {
                if let Some(prev) = previous_owned {
                    let _ = cb.set_text(prev);
                } else {
                    // Clear clipboard by setting empty string
                    let _ = cb.set_text(String::new());
                }
            }
        });

        Ok(())
    }

    fn simulate_paste() -> Result<()> {
        let delay = Duration::from_millis(20);

        #[cfg(target_os = "macos")]
        {
            send_key(EventType::KeyPress(Key::MetaLeft))?;
            std::thread::sleep(delay);
            send_key(EventType::KeyPress(Key::KeyV))?;
            std::thread::sleep(delay);
            send_key(EventType::KeyRelease(Key::KeyV))?;
            std::thread::sleep(delay);
            send_key(EventType::KeyRelease(Key::MetaLeft))?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            send_key(EventType::KeyPress(Key::ControlLeft))?;
            std::thread::sleep(delay);
            send_key(EventType::KeyPress(Key::KeyV))?;
            std::thread::sleep(delay);
            send_key(EventType::KeyRelease(Key::KeyV))?;
            std::thread::sleep(delay);
            send_key(EventType::KeyRelease(Key::ControlLeft))?;
        }

        Ok(())
    }
}

fn send_key(event_type: EventType) -> Result<()> {
    simulate(&event_type).map_err(|e| anyhow::anyhow!("Key simulation error: {e:?}"))
}

impl Default for TextInjector {
    fn default() -> Self {
        Self::new()
    }
}
