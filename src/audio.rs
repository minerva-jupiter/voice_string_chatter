use rodio::{
    Decoder, DeviceSinkBuilder, DeviceTrait, MixerDeviceSink, Player, cpal::traits::HostTrait,
};
use std::io::Cursor;

pub struct AudioEngine {
    active_device_sink: Option<MixerDeviceSink>,
    playback_player: Option<Player>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            active_device_sink: None,
            playback_player: None,
        }
    }

    pub fn list_output_devices() -> Vec<String> {
        let devices = rodio::cpal::default_host()
            .output_devices()
            .expect("There are no output devices");
        devices
            .filter_map(|d| d.description().ok())
            .map(|desc| desc.name().to_string())
            .collect()
    }

    pub fn play_audio(
        &mut self,
        audio_bytes: Vec<u8>,
        target_device_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut hosts = rodio::cpal::default_host()
            .output_devices()
            .expect("There are no output devices");
        let device = hosts
            .find(|device| {
                device
                    .description()
                    .map(|desc| desc.name() == target_device_name)
                    .unwrap_or(false)
            })
            .ok_or("Audio device not found")?;

        let device_options = DeviceSinkBuilder::from_device(device)?;

        let handle = device_options.open_sink_or_fallback()?;

        let player = Player::connect_new(&handle.mixer());

        let cursor = Cursor::new(audio_bytes);
        let decoded_source = Decoder::try_from(cursor)?;

        player.append(decoded_source);

        self.active_device_sink = Some(handle);
        self.playback_player = Some(player);

        Ok(())
    }

    pub fn stop_playback(&mut self) {
        if let Some(ref player) = self.playback_player {
            player.pause();
        }
        self.playback_player = None;
        self.active_device_sink = None;
    }

    pub fn is_playing(&self) -> bool {
        self.playback_player.is_some()
    }
}
