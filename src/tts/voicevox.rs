use crate::config::{EngineSetting, VoicevoxSetting};
use crate::tts::{Language, TtsEngine};
use serde_json::Value;

pub struct VoicevoxEngine {
    base_url: String,
}

impl VoicevoxEngine {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn with_default_endpoint() -> Self {
        Self::new("http://127.0.0.1:50021")
    }

    fn create_audio_query(
        &self,
        text: &str,
        speaker_id: u32,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/audio_query", self.base_url);
        let mut response = ureq::post(&endpoint)
            .query("text", text)
            .query("speaker", &speaker_id.to_string())
            .send_empty()?;

        let query_json: Value = response.body_mut().read_json()?;
        Ok(query_json)
    }

    fn apply_settings_to_query(&self, mut query_json: Value, setting: &VoicevoxSetting) -> Value {
        if let Some(obj) = query_json.as_object_mut() {
            obj.insert("speedScale".to_string(), Value::from(setting.speed_scale));
            obj.insert("pitchScale".to_string(), Value::from(setting.pitch_scale));
            obj.insert(
                "intonationScale".to_string(),
                Value::from(setting.intonation_scale),
            );
            obj.insert("volumeScale".to_string(), Value::from(setting.volume_scale));
            obj.insert(
                "prePhonemeLength".to_string(),
                Value::from(setting.pre_phoneme_length),
            );
            obj.insert(
                "postPhonemeLength".to_string(),
                Value::from(setting.post_phoneme_length),
            );
            obj.insert(
                "pauseLength".to_string(),
                setting.pause_length.map(Value::from).unwrap_or(Value::Null),
            );
            obj.insert(
                "pauseLengthScale".to_string(),
                Value::from(setting.pause_length_scale),
            );
        }
        query_json
    }

    fn execute_synthesis(
        &self,
        query_json: Value,
        speaker_id: u32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/synthesis", self.base_url);
        let mut response = ureq::post(&endpoint)
            .query("speaker", &speaker_id.to_string())
            .send_json(query_json)?;

        let audio_bytes = response.body_mut().read_to_vec()?;
        Ok(audio_bytes)
    }
}

impl TtsEngine for VoicevoxEngine {
    fn id(&self) -> &str {
        "voicevox"
    }

    fn name(&self) -> &str {
        "VOICEVOX (Local)"
    }

    fn default_settings(&self) -> EngineSetting {
        EngineSetting::Voicevox(VoicevoxSetting::default())
    }

    fn synthesize(
        &self,
        text: &str,
        settings: &EngineSetting,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let voicevox_setting = match settings {
            EngineSetting::Voicevox(s) => s,
            _ => return Err("Invalid settings type for VOICEVOX".into()),
        };

        let base_query = self.create_audio_query(text, voicevox_setting.speaker)?;
        let modified_query = self.apply_settings_to_query(base_query, voicevox_setting);
        self.execute_synthesis(modified_query, voicevox_setting.speaker)
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Japanese]
    }
}
