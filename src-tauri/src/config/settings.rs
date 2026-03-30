use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("typeless")
        .join("settings.json")
}

fn default_language() -> String {
    "zh-CN".to_string()
}

fn default_hotkey() -> String {
    #[cfg(target_os = "macos")]
    {
        "RightCtrl".to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        "F13".to_string()
    }
}

fn default_endpoint() -> String {
    "https://api.openai.com/v1/audio/transcriptions".to_string()
}

fn default_model() -> String {
    "whisper-1".to_string()
}

fn default_speech_language() -> String {
    "zh".to_string()
}

fn default_provider() -> String {
    "mock".to_string()
}

fn default_llm_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_max_history() -> usize {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechApiConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_speech_language")]
    pub language: String,
}

impl Default for SpeechApiConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            endpoint: default_endpoint(),
            api_key: String::new(),
            model: default_model(),
            language: default_speech_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: default_llm_model(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryConfig {
    #[serde(default = "bool_true")]
    pub tech_vocab_enabled: bool,
    #[serde(default)]
    pub user_vocab: Vec<String>,
    #[serde(default = "default_max_history")]
    pub max_history: usize,
}

fn bool_true() -> bool {
    true
}

impl Default for DictionaryConfig {
    fn default() -> Self {
        Self {
            tech_vocab_enabled: true,
            user_vocab: Vec::new(),
            max_history: default_max_history(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub speech_api: SpeechApiConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub dictionary: DictionaryConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
            hotkey: default_hotkey(),
            speech_api: SpeechApiConfig::default(),
            llm: LlmConfig::default(),
            dictionary: DictionaryConfig::default(),
        }
    }
}

impl AppSettings {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read settings from {}", path.display()))?;
        let settings: Self =
            serde_json::from_str(&data).context("Failed to parse settings JSON")?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self).context("Failed to serialize settings")?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write settings to {}", path.display()))?;
        Ok(())
    }
}
