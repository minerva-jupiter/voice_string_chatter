use slint::{Model, VecModel};
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

slint::include_modules!();

mod app;
mod audio;
mod config;
mod history;
mod tts;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let config_manager = Rc::new(std::cell::RefCell::new(
        config::ConfigManager::load_or_default(),
    ));
    let history_manager = Rc::new(std::cell::RefCell::new(history::HistoryManager::new()));
    let audio_engine = Arc::new(Mutex::new(audio::AudioEngine::new()));

    let engines = tts::available_engines();
    let mut engine_map: HashMap<String, Arc<dyn tts::TtsEngine>> = HashMap::new();
    let mut engine_names: Vec<slint::SharedString> = Vec::new();

    for engine in &engines {
        engine_names.push(engine.name().into());
        engine_map.insert(engine.name().to_string(), engine.clone());
    }

    let default_model_name = engine_names
        .first()
        .cloned()
        .unwrap_or_else(|| "VOICEVOX (Local)".into())
        .to_string();

    let selected_model = Rc::new(std::cell::RefCell::new(
        config_manager
            .borrow()
            .config()
            .selected_tts_model
            .clone()
            .unwrap_or(default_model_name),
    ));

    if let Some(&config::EngineSetting::Voicevox(ref vv_setting)) = config_manager
        .borrow()
        .config()
        .engine_settings
        .get("voicevox")
    {
        ui.set_voicevox_speaker(vv_setting.speaker as i32);
        ui.set_voicevox_speed(vv_setting.speed_scale.to_string().into());
        ui.set_voicevox_pitch(vv_setting.pitch_scale.to_string().into());
        ui.set_voicevox_volume(vv_setting.volume_scale.to_string().into());
    }

    let history_model = Rc::new(VecModel::<HistoryItem>::default());
    ui.set_history_items(history_model.clone().into());

    let tts_model_list = Rc::new(VecModel::from(engine_names));
    ui.set_tts_models(tts_model_list.into());
    ui.set_selected_tts_model(selected_model.borrow().clone().into());

    let detected_devices = audio::AudioEngine::list_output_devices();
    let device_names: Vec<slint::SharedString> = if detected_devices.is_empty() {
        vec!["Default".into()]
    } else {
        detected_devices
            .into_iter()
            .map(slint::SharedString::from)
            .collect()
    };

    let initial_device = config_manager
        .borrow()
        .config()
        .selected_output_device
        .clone()
        .filter(|saved| device_names.iter().any(|d| d.as_str() == saved))
        .unwrap_or_else(|| {
            device_names
                .first()
                .cloned()
                .unwrap_or_else(|| "Default".into())
                .to_string()
        });

    let device_model = Rc::new(VecModel::from(device_names));
    ui.set_output_devices(device_model.into());
    ui.set_selected_output_device(initial_device.clone().into());

    let selected_device = Rc::new(std::cell::RefCell::new(initial_device));

    let ui_handle = ui.as_weak();
    let history_ctrl = history_manager.clone();
    let history_model_ctrl = history_model.clone();
    let audio_ctrl = audio_engine.clone();
    let engine_registry = Arc::new(engine_map);
    let config_ctrl_for_synth = config_manager.clone();
    let selected_device_ctrl = selected_device.clone();
    let selected_model_ctrl = selected_model.clone();
    let subtitle_path = config_manager.borrow().config().subtitle_file_path.clone();

    let registry_for_speak = engine_registry.clone();
    ui.on_speak_requested(move |text| {
        let text_content = text.to_string();
        let mut history = history_ctrl.borrow_mut();
        let id = history.push(text_content.clone());
        let _ = history.export_subtitle(&text_content, &subtitle_path);

        history_model_ctrl.push(HistoryItem {
            id,
            text: text_content.clone().into(),
            timestamp: "".into(),
        });

        let target_device = selected_device_ctrl.borrow().clone();
        let current_model = selected_model_ctrl.borrow().clone();
        let audio_worker = audio_ctrl.clone();
        let registry = registry_for_speak.clone();
        let speech_text = text_content.clone();
        let weak_ui = ui_handle.clone();

        let engine_settings = registry.get(&current_model).map(|engine| {
            config_ctrl_for_synth
                .borrow()
                .config()
                .engine_settings
                .get(engine.id())
                .cloned()
                .unwrap_or_else(|| engine.default_settings())
        });

        std::thread::spawn(move || {
            let _ = weak_ui.upgrade_in_event_loop(|window| window.set_is_playing(true));

            if let Some(engine) = registry.get(&current_model) {
                if let Some(settings) = engine_settings {
                    match engine.synthesize(&speech_text, &settings) {
                        Ok(audio_bytes) => {
                            if !audio_bytes.is_empty() {
                                if let Ok(mut player) = audio_worker.lock() {
                                    let _ = player.play_audio(audio_bytes, &target_device);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("TTS synthesis failed ({current_model}): {err}");
                        }
                    }
                }
            } else {
                eprintln!("Selected TTS engine not found in registry: {current_model}");
            }

            let _ = weak_ui.upgrade_in_event_loop(|window| window.set_is_playing(false));
        });
    });

    let history_ctrl = history_manager.clone();
    let history_model_ctrl = history_model.clone();
    ui.on_delete_requested(move |id| {
        let mut history = history_ctrl.borrow_mut();
        history.remove(id);

        let index = history_model_ctrl.iter().position(|item| item.id == id);
        if let Some(i) = index {
            history_model_ctrl.remove(i);
        }
    });

    let history_ctrl = history_manager.clone();
    let audio_ctrl = audio_engine.clone();
    let registry_for_replay = engine_registry.clone();
    let config_ctrl_for_replay = config_manager.clone();
    let selected_device_ctrl = selected_device.clone();
    let selected_model_ctrl = selected_model.clone();
    let subtitle_path = config_manager.borrow().config().subtitle_file_path.clone();
    let replay_ui_handle = ui.as_weak();

    ui.on_replay_requested(move |id| {
        let history = history_ctrl.borrow();
        if let Some(text) = history.find_text(id) {
            let _ = history.export_subtitle(&text, &subtitle_path);
            let target_device = selected_device_ctrl.borrow().clone();
            let current_model = selected_model_ctrl.borrow().clone();
            let audio_worker = audio_ctrl.clone();
            let registry = registry_for_replay.clone();
            let speech_text = text.clone();
            let weak_ui = replay_ui_handle.clone();

            let engine_settings = registry.get(&current_model).map(|engine| {
                config_ctrl_for_replay
                    .borrow()
                    .config()
                    .engine_settings
                    .get(engine.id())
                    .cloned()
                    .unwrap_or_else(|| engine.default_settings())
            });

            std::thread::spawn(move || {
                let _ = weak_ui.upgrade_in_event_loop(|window| window.set_is_playing(true));

                if let Some(engine) = registry.get(&current_model) {
                    if let Some(settings) = engine_settings {
                        match engine.synthesize(&speech_text, &settings) {
                            Ok(audio_bytes) => {
                                if !audio_bytes.is_empty() {
                                    if let Ok(mut player) = audio_worker.lock() {
                                        let _ = player.play_audio(audio_bytes, &target_device);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("TTS replay synthesis failed ({current_model}): {err}");
                            }
                        }
                    }
                }

                let _ = weak_ui.upgrade_in_event_loop(|window| window.set_is_playing(false));
            });
        }
    });

    let audio_ctrl = audio_engine.clone();
    ui.on_stop_requested(move || {
        if let Ok(mut player) = audio_ctrl.lock() {
            player.stop_playback();
        }
    });

    let active_device = selected_device.clone();
    let config_ctrl = config_manager.clone();
    ui.on_device_changed(move |device| {
        let device_string = device.to_string();
        *active_device.borrow_mut() = device_string.clone();
        let _ = config_ctrl
            .borrow_mut()
            .update_selected_device(device_string);
    });

    let active_model = selected_model.clone();
    let config_ctrl = config_manager.clone();
    ui.on_model_changed(move |model| {
        let model_string = model.to_string();
        *active_model.borrow_mut() = model_string.clone();
        let _ = config_ctrl
            .borrow_mut()
            .update_selected_tts_model(model_string);
    });

    let config_ctrl_for_settings = config_manager.clone();
    ui.on_voicevox_settings_changed(move |speaker, speed, pitch, volume| {
        let speed_val = speed.parse::<f64>().unwrap_or(1.0);
        let pitch_val = pitch.parse::<f64>().unwrap_or(0.0);
        let volume_val = volume.parse::<f64>().unwrap_or(1.0);

        let mut cfg_ref = config_ctrl_for_settings.borrow_mut();
        let cfg = cfg_ref.config_mut();
        cfg.engine_settings.insert(
            "voicevox".to_string(),
            config::EngineSetting::Voicevox(config::VoicevoxSetting {
                speaker: speaker as u32,
                speed_scale: speed_val,
                pitch_scale: pitch_val,
                intonation_scale: 1.0,
                volume_scale: volume_val,
                pre_phoneme_length: 0.1,
                post_phoneme_length: 0.1,
                pause_length: None,
                pause_length_scale: 1.0,
            }),
        );
        let _ = cfg_ref.save();
    });

    ui.run()
}
