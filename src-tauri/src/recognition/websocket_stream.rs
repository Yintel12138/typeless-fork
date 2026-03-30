use crate::config::SpeechApiConfig;
use crate::recognition::provider::{SpeechRecognitionProvider, TranscriptStream};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct WebSocketProvider;

/// State machine for an active WebSocket streaming session.
#[allow(dead_code)]
pub struct WsStream {
    /// Partial transcripts accumulated while polling.
    partials: Vec<String>,
    final_text: Option<String>,
    /// Encoded audio pending to be sent (not sent until finish in this simple impl).
    audio_buffer: Vec<f32>,
    endpoint: String,
}

impl WsStream {
    #[allow(dead_code)]
    fn new(endpoint: String) -> Self {
        Self {
            partials: Vec::new(),
            final_text: None,
            audio_buffer: Vec::new(),
            endpoint,
        }
    }
}

impl TranscriptStream for WsStream {
    fn feed_audio(&mut self, samples: &[f32]) {
        self.audio_buffer.extend_from_slice(samples);
    }

    fn poll_transcript(&mut self) -> Option<String> {
        self.partials.pop()
    }

    fn finish(&mut self) -> Result<String> {
        // Build a minimal WAV in memory from buffered samples and send over WS.
        let wav_bytes =
            crate::audio::recorder::AudioRecorder::export_wav(&self.audio_buffer, 16000)?;

        let endpoint = self.endpoint.clone();
        let rt = tokio::runtime::Handle::try_current();

        let final_text = match rt {
            Ok(handle) => {
                handle.block_on(async { ws_send_and_receive(&endpoint, wav_bytes).await })?
            }
            Err(_) => {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async { ws_send_and_receive(&endpoint, wav_bytes).await })?
            }
        };

        self.final_text = Some(final_text.clone());
        Ok(final_text)
    }
}

async fn ws_send_and_receive(endpoint: &str, wav_bytes: Vec<u8>) -> Result<String> {
    let (mut ws, _) = connect_async(endpoint)
        .await
        .with_context(|| format!("WebSocket connect failed: {endpoint}"))?;

    // Send audio as binary frame
    ws.send(Message::Binary(wav_bytes)).await?;
    ws.send(Message::Text("{\"type\":\"end\"}".to_string()))
        .await?;

    let mut final_text = String::new();

    while let Some(msg) = ws.next().await {
        let msg = msg.context("WebSocket receive error")?;
        match msg {
            Message::Text(t) => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&t) {
                    let msg_type = val["type"].as_str().unwrap_or("");
                    let text = val["text"].as_str().unwrap_or("").to_string();
                    if msg_type == "final" {
                        final_text = text;
                        break;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(final_text)
}

#[async_trait]
impl SpeechRecognitionProvider for WebSocketProvider {
    async fn transcribe_full(
        &self,
        audio_wav: Vec<u8>,
        config: &SpeechApiConfig,
    ) -> Result<String> {
        ws_send_and_receive(&config.endpoint, audio_wav).await
    }

    async fn start_stream(&self, config: &SpeechApiConfig) -> Result<Box<dyn TranscriptStream>> {
        Ok(Box::new(WsStream::new(config.endpoint.clone())))
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}
