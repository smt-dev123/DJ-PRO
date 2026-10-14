pub mod audio;
pub mod commands;

use audio::AudioEngine;
use audio::state::MixerState;
use commands::*;
use std::sync::{Arc, Mutex};

pub fn run() {
    let engine = AudioEngine::new().expect("Failed to initialize audio engine");
    let shared_state: Arc<Mutex<MixerState>> = engine.state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![
            load_track_fast,
            toggle_play,
            set_deck_volume,
            set_deck_eq,
            set_deck_gain,
            set_crossfader,
            seek_deck,
            get_deck_playback,
            get_system_drives,
            scan_folder_tracks
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}