use crate::audio::AudioEngine;
use crate::config::AppConfig;
use crate::history::HistoryManager;
use crate::tts::TtsEngine;

pub struct AppOrchestrator {
    audio: AudioEngine,
    config: AppConfig,
    history: HistoryManager,
    tts_engines: Vec<Box<dyn TtsEngine>>,
}

impl AppOrchestrator {
    pub fn new() -> Self {
        Self {
            audio: AudioEngine::new(),
            config: AppConfig::default(),
            history: HistoryManager::new(),
            tts_engines: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        // Initialization and main loop connection to Slint
    }
}
