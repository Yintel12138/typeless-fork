use crate::config::LlmConfig;
use anyhow::{Context, Result};

fn system_prompt_for_language(language: &str) -> &'static str {
    if language.starts_with("zh") || language.starts_with("ja") || language.starts_with("ko") {
        // CJK: Chinese system prompt
        "你是语音识别纠错助手。\
只修复明显的ASR识别错误（同音字、断句错误），\
绝对不能改写、润色、删减用户说的正确内容。\
如果没有明显错误，原样返回。"
    } else {
        // Latin scripts: English system prompt
        "You are a speech-recognition error-correction assistant. \
Only fix obvious ASR errors (homophones, mis-segmentation). \
Never rewrite, embellish, or remove correct content. \
Return the original text unchanged if there are no obvious errors."
    }
}

pub struct LlmRefiner;

impl LlmRefiner {
    pub fn new() -> Self {
        Self
    }

    pub async fn refine(&self, text: &str, config: &LlmConfig, language: &str) -> Result<String> {
        if !config.enabled || config.endpoint.is_empty() {
            return Ok(text.to_string());
        }

        let client = reqwest::Client::new();
        let prompt = system_prompt_for_language(language);

        let body = serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": prompt },
                { "role": "user",   "content": text }
            ],
            "temperature": 0.0,
            "max_tokens": 1024
        });

        let mut req = client.post(&config.endpoint).json(&body);

        if !config.api_key.is_empty() {
            req = req.bearer_auth(&config.api_key);
        }

        let resp = req
            .send()
            .await
            .context("Failed to connect to LLM endpoint")?;

        let status = resp.status();
        let resp_body = resp.text().await.context("Failed to read LLM response")?;

        if !status.is_success() {
            log::warn!("LLM API error {status}: {resp_body}. Returning original text.");
            return Ok(text.to_string());
        }

        let val: serde_json::Value =
            serde_json::from_str(&resp_body).context("Failed to parse LLM JSON response")?;

        let refined = val["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or(text)
            .to_string();

        Ok(refined)
    }
}

impl Default for LlmRefiner {
    fn default() -> Self {
        Self::new()
    }
}
