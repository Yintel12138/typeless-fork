use crate::config::SpeechApiConfig;
use anyhow::Result;
use async_trait::async_trait;

/// Streaming transcript interface.
#[allow(dead_code)]
pub trait TranscriptStream: Send {
    /// Feed a chunk of raw f32 audio samples into the stream.
    fn feed_audio(&mut self, samples: &[f32]);
    /// Poll for the latest partial transcript, if available.
    fn poll_transcript(&mut self) -> Option<String>;
    /// Signal end-of-audio and wait for the final transcript.
    fn finish(&mut self) -> Result<String>;
}

/// Common interface for all speech-recognition back-ends.
#[async_trait]
pub trait SpeechRecognitionProvider: Send + Sync {
    /// Transcribe a fully-recorded WAV file (bytes) and return text.
    async fn transcribe_full(&self, audio_wav: Vec<u8>, config: &SpeechApiConfig)
        -> Result<String>;

    /// Open a streaming session for live transcription.
    async fn start_stream(&self, config: &SpeechApiConfig) -> Result<Box<dyn TranscriptStream>>;

    /// Whether this provider supports low-latency streaming.
    fn supports_streaming(&self) -> bool;
}
