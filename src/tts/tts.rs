use crate::config::{EngineSetting, OsNativeSetting};
use crate::tts::{Language, TtsEngine};
use std::sync::Mutex;
use tts::Tts;

pub struct NativeOsTtsEngine {
    engine: Mutex<Option<Tts>>,
}

impl NativeOsTtsEngine {
    pub fn new() -> Self {
        let tts_instance = Tts::default().ok();
        Self {
            engine: Mutex::new(tts_instance),
        }
    }
}

impl TtsEngine for NativeOsTtsEngine {
    fn id(&self) -> &str {
        "native-os"
    }

    fn name(&self) -> &str {
        "TTS(OS Native)"
    }

    fn default_settings(&self) -> EngineSetting {
        EngineSetting::OsNative(OsNativeSetting::default())
    }

    fn synthesize(
        &self,
        text: &str,
        settings: &EngineSetting,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let _os_setting = match settings {
            EngineSetting::OsNative(s) => s,
            _ => return Err("Invalid settings type for OS Native TTS".into()),
        };

        let mut guard = self.engine.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut engine) = *guard {
            engine.speak(text, false)?;
        }
        Ok(Vec::new())
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::English, Language::Japanese]
    }
}
