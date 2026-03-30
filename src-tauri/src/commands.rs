use crate::audio::AudioRecorder;
use crate::config::AppSettings;
use crate::dictionary::DictionaryManager;
use crate::injection::TextInjector;
use crate::llm::LlmRefiner;
use crate::recognition::provider_factory;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

/// Application-wide shared state.
pub struct AppState {
    pub settings: AppSettings,
    pub recorder: AudioRecorder,
    pub dictionary: DictionaryManager,
}

pub type SharedState = Mutex<AppState>;

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub code: String,
    pub label: String,
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_settings(state: State<'_, SharedState>) -> AppSettings {
    state.lock().unwrap().settings.clone()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, state: State<'_, SharedState>) -> Result<(), String> {
    settings.save().map_err(|e| e.to_string())?;
    let mut s = state.lock().unwrap();
    s.settings = settings.clone();
    // Re-initialise dictionary with updated config
    s.dictionary = DictionaryManager::new(
        settings.dictionary.tech_vocab_enabled,
        settings.dictionary.user_vocab.clone(),
        settings.dictionary.max_history,
    );
    Ok(())
}

// ── Recording ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_recording(app: AppHandle, state: State<'_, SharedState>) -> Result<(), String> {
    {
        let mut s = state.lock().unwrap();
        s.recorder.start_recording().map_err(|e| e.to_string())?;
    }
    let _ = app.emit("recording_start", ());
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<String, String> {
    let (samples, sample_rate, speech_config, llm_config, app_language) = {
        let mut s = state.lock().unwrap();
        let (samples, rate) = s.recorder.stop_recording();
        let _ = app.emit("recording_stop", ());
        (
            samples,
            rate,
            s.settings.speech_api.clone(),
            s.settings.llm.clone(),
            s.settings.language.clone(),
        )
    };

    let _ = app.emit("processing_start", ());

    let wav = AudioRecorder::export_wav(&samples, sample_rate).map_err(|e| e.to_string())?;

    let provider = provider_factory(&speech_config);
    let raw_text = provider
        .transcribe_full(wav, &speech_config)
        .await
        .map_err(|e| e.to_string())?;

    let refiner = LlmRefiner::new();
    let text = refiner
        .refine(&raw_text, &llm_config, &app_language)
        .await
        .unwrap_or(raw_text);

    {
        let mut s = state.lock().unwrap();
        s.dictionary.record_input(&text);
    }

    let _ = app.emit("transcript_final", text.clone());
    Ok(text)
}

#[tauri::command]
pub fn get_rms(state: State<'_, SharedState>) -> f32 {
    state.lock().unwrap().recorder.get_rms()
}

// ── Text injection ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn inject_text(text: String) -> Result<(), String> {
    TextInjector::new().inject(&text).map_err(|e| e.to_string())
}

// ── Misc ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_supported_languages() -> Vec<LanguageInfo> {
    vec![
        LanguageInfo {
            code: "zh-CN".into(),
            label: "中文 (简体)".into(),
        },
        LanguageInfo {
            code: "zh-TW".into(),
            label: "中文 (繁體)".into(),
        },
        LanguageInfo {
            code: "en".into(),
            label: "English".into(),
        },
        LanguageInfo {
            code: "ja".into(),
            label: "日本語".into(),
        },
        LanguageInfo {
            code: "ko".into(),
            label: "한국어".into(),
        },
    ]
}
