use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicevoxSetting {
    pub speaker: u32,
    pub speed_scale: f64,
    pub pitch_scale: f64,
    pub intonation_scale: f64,
    pub volume_scale: f64,
    pub pre_phoneme_length: f64,
    pub post_phoneme_length: f64,
    pub pause_length: Option<f64>,
    pub pause_length_scale: f64,
}

impl Default for VoicevoxSetting {
    fn default() -> Self {
        Self {
            speaker: 14,
            speed_scale: 1.0,
            pitch_scale: 0.0,
            intonation_scale: 1.0,
            volume_scale: 1.0,
            pre_phoneme_length: 0.1,
            post_phoneme_length: 0.1,
            pause_length: None,
            pause_length_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsNativeSetting {
    pub rate: f32,
    pub volume: f32,
}

impl Default for OsNativeSetting {
    fn default() -> Self {
        Self {
            rate: 1.0,
            volume: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "engine_type", content = "settings")]
pub enum EngineSetting {
    Voicevox(VoicevoxSetting),
    OsNative(OsNativeSetting),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub selected_output_device: Option<String>,
    pub selected_tts_model: Option<String>,
    pub engine_settings: HashMap<String, EngineSetting>,
    pub subtitle_file_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut engine_settings = HashMap::new();
        engine_settings.insert(
            "voicevox".to_string(),
            EngineSetting::Voicevox(VoicevoxSetting::default()),
        );
        engine_settings.insert(
            "native-os".to_string(),
            EngineSetting::OsNative(OsNativeSetting::default()),
        );

        Self {
            selected_output_device: None,
            selected_tts_model: Some("VOICEVOX (Local)".to_string()),
            engine_settings,
            subtitle_file_path: PathBuf::from("subtitles.txt"),
        }
    }
}

pub struct ConfigManager {
    config_file_path: PathBuf,
    current_config: AppConfig,
}

impl ConfigManager {
    pub fn load_or_default() -> Self {
        let config_file_path = Self::resolve_config_path();
        let current_config = if config_file_path.exists() {
            fs::read_to_string(&config_file_path)
                .ok()
                .and_then(|raw_json| serde_json::from_str::<AppConfig>(&raw_json).ok())
                .unwrap_or_default()
        } else {
            let default_config = AppConfig::default();
            let _ = Self::write_to_disk(&config_file_path, &default_config);
            default_config
        };

        Self {
            config_file_path,
            current_config,
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.current_config
    }

    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.current_config
    }

    pub fn update_selected_device(
        &mut self,
        device_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.current_config.selected_output_device = Some(device_name);
        self.save()
    }

    pub fn update_selected_tts_model(
        &mut self,
        model_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.current_config.selected_tts_model = Some(model_name);
        self.save()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        Self::write_to_disk(&self.config_file_path, &self.current_config)
    }

    fn write_to_disk(path: &PathBuf, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(config)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    fn resolve_config_path() -> PathBuf {
        ProjectDirs::from("com", "voice_string_chatter", "voice_string_chatter")
            .map(|project_dirs| project_dirs.config_dir().join("config.json"))
            .unwrap_or_else(|| PathBuf::from("config.json"))
    }
}
