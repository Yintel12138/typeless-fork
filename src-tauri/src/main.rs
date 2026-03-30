#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod audio;
mod commands;
mod config;
mod dictionary;
mod hotkey;
mod injection;
mod llm;
mod recognition;
mod tray;

use commands::{AppState, SharedState};
use config::AppSettings;
use dictionary::DictionaryManager;
use hotkey::GlobalHotkeyManager;
use tauri::Manager;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let settings = AppSettings::load().unwrap_or_else(|e| {
        log::warn!("Failed to load settings, using defaults: {e}");
        AppSettings::default()
    });

    let dictionary = DictionaryManager::new(
        settings.dictionary.tech_vocab_enabled,
        settings.dictionary.user_vocab.clone(),
        settings.dictionary.max_history,
    );

    let state = SharedState::new(AppState {
        settings: settings.clone(),
        recorder: audio::AudioRecorder::new(),
        dictionary,
    });

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Build system tray
            tray::build_tray(&app_handle)?;

            // Start global hotkey listener
            let hotkey_mgr = GlobalHotkeyManager::new();
            hotkey_mgr
                .start_listening(settings.hotkey.clone(), app_handle.clone())
                .unwrap_or_else(|e| log::warn!("Hotkey setup failed: {e}"));

            // Position overlay at bottom-center of primary monitor
            if let Some(overlay) = app_handle.get_webview_window("overlay") {
                if let Ok(Some(monitor)) = overlay.primary_monitor() {
                    let msize = monitor.size();
                    let wsize = overlay.outer_size().unwrap_or(tauri::PhysicalSize {
                        width: 480,
                        height: 80,
                    });
                    let x = (msize.width as i32 - wsize.width as i32) / 2;
                    let y = msize.height as i32 - wsize.height as i32 - 40;
                    let _ = overlay.set_position(tauri::PhysicalPosition { x, y });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::start_recording,
            commands::stop_recording,
            commands::get_rms,
            commands::inject_text,
            commands::get_supported_languages,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Typeless");
}
