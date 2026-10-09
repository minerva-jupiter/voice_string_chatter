use crate::config::EngineSetting;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod tts;
pub mod voicevox;

pub use tts::NativeOsTtsEngine;
pub use voicevox::VoicevoxEngine;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    Japanese,
    English,
}

pub trait TtsEngine: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn default_settings(&self) -> EngineSetting;
    fn synthesize(
        &self,
        text: &str,
        settings: &EngineSetting,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn supported_languages(&self) -> Vec<Language>;
}

pub fn available_engines() -> Vec<Arc<dyn TtsEngine>> {
    vec![
        Arc::new(VoicevoxEngine::with_default_endpoint()),
        Arc::new(NativeOsTtsEngine::new()),
    ]
}
