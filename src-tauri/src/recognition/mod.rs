pub mod mock;
pub mod openai_http;
pub mod provider;
pub mod websocket_stream;

pub use mock::MockProvider;
pub use openai_http::OpenAiHttpProvider;
pub use provider::SpeechRecognitionProvider;
pub use websocket_stream::WebSocketProvider;

use crate::config::SpeechApiConfig;

/// Return the appropriate provider based on `config.provider`.
pub fn provider_factory(config: &SpeechApiConfig) -> Box<dyn SpeechRecognitionProvider> {
    match config.provider.as_str() {
        "openai_http" => Box::new(OpenAiHttpProvider),
        "websocket" => Box::new(WebSocketProvider),
        _ => Box::new(MockProvider),
    }
}
