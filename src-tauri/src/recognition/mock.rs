use crate::config::SpeechApiConfig;
use crate::recognition::provider::{SpeechRecognitionProvider, TranscriptStream};
use anyhow::Result;
use async_trait::async_trait;

const ZH_TRANSCRIPT: &str = "这是一段测试语音转录文本。";
const EN_TRANSCRIPT: &str = "This is a test speech transcription.";

pub struct MockProvider;

pub struct MockStream {
    #[allow(dead_code)]
    chunks: Vec<String>,
    #[allow(dead_code)]
    index: usize,
}

impl MockStream {
    fn new(text: &str) -> Self {
        // Split into roughly 3 partial chunks
        let chars: Vec<char> = text.chars().collect();
        let chunk_size = (chars.len() / 3).max(1);
        let chunks: Vec<String> = chars
            .chunks(chunk_size)
            .map(|c| c.iter().collect())
            .collect();
        Self { chunks, index: 0 }
    }
}

impl TranscriptStream for MockStream {
    fn feed_audio(&mut self, _samples: &[f32]) {
        // Mock: no real processing
    }

    fn poll_transcript(&mut self) -> Option<String> {
        if self.index < self.chunks.len() {
            let partial: String = self.chunks[..=self.index].concat();
            self.index += 1;
            Some(partial)
        } else {
            None
        }
    }

    fn finish(&mut self) -> Result<String> {
        Ok(self.chunks.concat())
    }
}

#[async_trait]
impl SpeechRecognitionProvider for MockProvider {
    async fn transcribe_full(
        &self,
        _audio_wav: Vec<u8>,
        config: &SpeechApiConfig,
    ) -> Result<String> {
        // Simulate a short processing delay
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let text = if config.language.starts_with("zh") {
            ZH_TRANSCRIPT
        } else {
            EN_TRANSCRIPT
        };
        Ok(text.to_string())
    }

    async fn start_stream(&self, config: &SpeechApiConfig) -> Result<Box<dyn TranscriptStream>> {
        let text = if config.language.starts_with("zh") {
            ZH_TRANSCRIPT
        } else {
            EN_TRANSCRIPT
        };
        Ok(Box::new(MockStream::new(text)))
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_stream_produces_full_text() {
        let mut stream = MockStream::new(ZH_TRANSCRIPT);
        while stream.poll_transcript().is_some() {}
        let result = stream.finish().unwrap();
        assert_eq!(result, ZH_TRANSCRIPT);
    }
}
