use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use tauri::State;
use crate::audio::state::MixerState;
use crate::audio::decoder::decode_file;

#[derive(Serialize)]
pub struct FileTrackItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub key: String,
    pub duration: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct DriveItem {
    pub name: String,
    pub path: String,
    pub kind: String, // "drive" or "system"
}

#[derive(Serialize)]
pub struct TrackLoadedInfo {
    pub title: String,
    pub duration: String,
    pub duration_secs: f32,
    pub sample_rate: u32,
    pub waveform: Vec<crate::audio::decoder::WaveBandPeak>,
}

#[derive(Serialize)]
pub struct DeckPlaybackStatus {
    pub is_playing: bool,
    pub progress: f32,
    pub current_time_str: String,
    pub remaining_time_str: String,
}

// Background async track decode - NEVER blocks UI thread!
#[tauri::command]
pub async fn load_track(
    deck_id: String,
    path: String,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<TrackLoadedInfo, String> {
    let decoded = tokio::task::spawn_blocking(move || decode_file(&path))
        .await
        .map_err(|e| e.to_string())??;

    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" {
        &mut lock.deck_a
    } else {
        &mut lock.deck_b
    };

    deck.samples = decoded.samples;
    deck.position = 0;
    deck.is_playing = false;
    deck.title = decoded.title.clone();
    deck.sample_rate = decoded.sample_rate;
    deck.channels = decoded.channels;
    deck.duration_secs = decoded.duration_secs;

    let mins = (decoded.duration_secs as u32) / 60;
    let secs = (decoded.duration_secs as u32) % 60;
    let duration = format!("{:02}:{:02}", mins, secs);

    Ok(TrackLoadedInfo {
        title: decoded.title,
        duration,
        duration_secs: decoded.duration_secs,
        sample_rate: decoded.sample_rate,
        waveform: decoded.waveform_peaks,
    })
}

#[tauri::command]
pub fn seek_deck(
    deck_id: String,
    ratio: f32,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<f32, String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" {
        &mut lock.deck_a
    } else {
        &mut lock.deck_b
    };
    deck.seek_ratio(ratio);
    Ok(deck.get_progress_ratio())
}

#[tauri::command]
pub fn get_deck_playback(
    deck_id: String,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<DeckPlaybackStatus, String> {
    let lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" { &lock.deck_a } else { &lock.deck_b };
    let progress = deck.get_progress_ratio();
    let elapsed = deck.get_elapsed_secs();
    let total = deck.duration_secs;
    let remaining = (total - elapsed).max(0.0);

    let cur_mins = (elapsed as u32) / 60;
    let cur_secs = (elapsed as u32) % 60;
    let rem_mins = (remaining as u32) / 60;
    let rem_secs = (remaining as u32) % 60;

    Ok(DeckPlaybackStatus {
        is_playing: deck.is_playing,
        progress,
        current_time_str: format!("{:02}:{:02}", cur_mins, cur_secs),
        remaining_time_str: format!("-{:02}:{:02}", rem_mins, rem_secs),
    })
}

#[tauri::command]
pub fn toggle_play(
    deck_id: String,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<bool, String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" {
        &mut lock.deck_a
    } else {
        &mut lock.deck_b
    };
    deck.is_playing = !deck.is_playing;
    Ok(deck.is_playing)
}

#[tauri::command]
pub fn set_deck_volume(
    deck_id: String,
    vol: f32,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" {
        &mut lock.deck_a
    } else {
        &mut lock.deck_b
    };
    deck.volume = vol;
    Ok(())
}

#[tauri::command]
pub fn set_crossfader(
    val: f32,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    lock.crossfader = val;
    Ok(())
}

#[tauri::command]
pub fn get_system_drives() -> Vec<DriveItem> {
    let mut list = Vec::new();

    if let Some(user_dir) = std::env::var_os("USERPROFILE") {
        let user_path = std::path::PathBuf::from(user_dir);
        list.push(DriveItem {
            name: "🎵 My Music".into(),
            path: user_path.join("Music").to_string_lossy().to_string(),
            kind: "system".into(),
        });
        list.push(DriveItem {
            name: "📁 Desktop".into(),
            path: user_path.join("Desktop").to_string_lossy().to_string(),
            kind: "system".into(),
        });
        list.push(DriveItem {
            name: "📥 Downloads".into(),
            path: user_path.join("Downloads").to_string_lossy().to_string(),
            kind: "system".into(),
        });
    }

    for drive_letter in b'C'..=b'Z' {
        let drive_str = format!("{}:\\", drive_letter as char);
        if Path::new(&drive_str).exists() {
            list.push(DriveItem {
                name: format!("Drive ({}:)", drive_letter as char),
                path: drive_str,
                kind: "drive".into(),
            });
        }
    }

    list
}

#[tauri::command]
pub fn scan_folder_tracks(folder_path: String) -> Result<Vec<FileTrackItem>, String> {
    let path = Path::new(&folder_path);
    if !path.exists() || !path.is_dir() {
        return Ok(Vec::new());
    }

    let mut tracks = Vec::new();
    let audio_exts = ["mp3", "wav", "flac", "aac", "m4a", "ogg"];

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                    if audio_exts.contains(&ext.to_lowercase().as_str()) {
                        let file_stem = file_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown Track")
                            .to_string();

                        tracks.push(FileTrackItem {
                            id: file_path.to_string_lossy().to_string(),
                            title: file_stem,
                            artist: "Local Artist".into(),
                            bpm: 128.0,
                            key: "5A".into(),
                            duration: "03:45".into(),
                            path: file_path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(tracks)
}