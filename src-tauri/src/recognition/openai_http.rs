use crate::config::SpeechApiConfig;
use crate::recognition::provider::{SpeechRecognitionProvider, TranscriptStream};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::multipart;

pub struct OpenAiHttpProvider;

#[async_trait]
impl SpeechRecognitionProvider for OpenAiHttpProvider {
    async fn transcribe_full(
        &self,
        audio_wav: Vec<u8>,
        config: &SpeechApiConfig,
    ) -> Result<String> {
        let client = reqwest::Client::new();

        let file_part = multipart::Part::bytes(audio_wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("model", config.model.clone())
            .text("language", config.language.clone())
            .text("response_format", "json");

        let mut req = client.post(&config.endpoint).multipart(form);

        if !config.api_key.is_empty() {
            req = req.bearer_auth(&config.api_key);
        }

        let resp = req
            .send()
            .await
            .context("Failed to send request to Whisper API")?;

        let status = resp.status();
        let body = resp.text().await.context("Failed to read response body")?;

        if !status.is_success() {
            anyhow::bail!("Whisper API error {}: {}", status, body);
        }

        let json: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse Whisper JSON response")?;

        let text = json["text"]
            .as_str()
            .context("Missing 'text' field in Whisper response")?
            .to_string();

        Ok(text)
    }

    async fn start_stream(&self, _config: &SpeechApiConfig) -> Result<Box<dyn TranscriptStream>> {
        anyhow::bail!("OpenAI HTTP provider does not support streaming")
    }

    fn supports_streaming(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_does_not_support_streaming() {
        let p = OpenAiHttpProvider;
        assert!(!p.supports_streaming());
    }
}
