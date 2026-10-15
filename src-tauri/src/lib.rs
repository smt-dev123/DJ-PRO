pub mod audio;
pub mod commands;
pub mod storage;

use audio::AudioEngine;
use audio::state::MixerState;
use commands::*;
use storage::commands::*;
use storage::STORAGE_CACHE;
use std::sync::{Arc, Mutex};

pub fn run() {
    let engine = AudioEngine::new().expect("Failed to initialize audio engine");
    let shared_state: Arc<Mutex<MixerState>> = engine.state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(shared_state)
        .manage(STORAGE_CACHE.clone())  // ✅ Manage cache
        .invoke_handler(tauri::generate_handler![
            // Audio commands
            load_track_fast,
            toggle_play,
            set_deck_volume,
            set_deck_eq,
            set_deck_gain,
            set_deck_fx_type,
            set_deck_fx_amount,
            set_deck_fx_active,
            reset_deck_fx,
            trigger_fx_out,
            trigger_roll_out,
            trigger_landing,
            cancel_fx_out,
            set_reverb_room,
            set_fx_time,
            set_fx_params,
            get_fx_params,
            trigger_hot_cue,
            toggle_loop,
            toggle_stem,
            set_crossfader,
            seek_deck,
            get_deck_playback,
            get_system_drives,
            scan_folder_tracks,

            // ✅ Storage commands
            find_drive_root_cmd,
            scan_for_metadata_folders,
            load_playlists,
            save_playlists,
            create_playlist,
            delete_playlist,
            rename_playlist,
            add_track_to_playlist,
            remove_track_from_playlist,
            reorder_playlist_tracks,
            load_cues,
            save_cues,
            upsert_track_cues,
            set_hot_cue,
            load_settings,
            save_settings,
            save_all,
            load_all,
            cache_stats,
            cache_clear,
            cache_invalidate,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}