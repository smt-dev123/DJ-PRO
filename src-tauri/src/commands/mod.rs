use crate::audio::decoder::{decode_samples_only, extract_waveform_progressive};
use crate::audio::fx::{FilterMode, FxParams, FxType};
use crate::audio::state::MixerState;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

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
    pub kind: String,
}

#[derive(Serialize, Clone)]
pub struct TrackLoadedFast {
    pub title: String,
    pub duration: String,
    pub duration_secs: f32,
    pub sample_rate: u32,
    pub deck_id: String,
}

#[derive(Serialize)]
pub struct DeckPlaybackStatus {
    pub is_playing: bool,
    pub progress: f32,
    pub current_time_str: String,
    pub remaining_time_str: String,
}

// ==================== LOAD TRACK ====================
#[tauri::command]
pub async fn load_track_fast(
    deck_id: String,
    path: String,
    app: AppHandle,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<TrackLoadedFast, String> {
    let path_clone = path.clone();
    let decoded = tokio::task::spawn_blocking(move || decode_samples_only(&path_clone))
        .await
        .map_err(|e| e.to_string())??;

    let duration_secs = decoded.duration_secs;
    let title = decoded.title.clone();
    let sample_rate = decoded.sample_rate;
    let channels = decoded.channels;

    {
        let mut lock = state.lock().map_err(|e| e.to_string())?;
        let deck = if deck_id == "A" {
            &mut lock.deck_a
        } else {
            &mut lock.deck_b
        };

        deck.samples = decoded.samples;
        deck.position = 0;
        deck.is_playing = false;
        deck.title = title.clone();
        deck.sample_rate = sample_rate;
        deck.channels = channels;
        deck.duration_secs = duration_secs;
        deck.waveform_peaks = Vec::new();
        deck.waveform_ready = false;
    }

    let mins = (duration_secs as u32) / 60;
    let secs = (duration_secs as u32) % 60;
    let duration_str = format!("{:02}:{:02}", mins, secs);

    let _ = app.emit(
        "track-ready",
        serde_json::json!({
            "deckId": deck_id,
            "title": title,
            "duration": duration_str,
            "durationSecs": duration_secs,
        }),
    );

    // Progressive waveform in background
    let app_clone = app.clone();
    let state_clone = Arc::clone(&state);
    let deck_id_analysis = deck_id.clone();

    tokio::task::spawn_blocking(move || {
        let (samples, sample_rate, channels, duration_secs) = {
            let lock = match state_clone.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let deck = if deck_id_analysis == "A" {
                &lock.deck_a
            } else {
                &lock.deck_b
            };
            (
                deck.samples.clone(),
                deck.sample_rate,
                deck.channels,
                deck.duration_secs,
            )
        };

        let total_chunks = 10;
        let deck_id_inner = deck_id_analysis.clone();
        let app_inner = app_clone.clone();

        let peaks = extract_waveform_progressive(
            &samples,
            sample_rate,
            channels,
            duration_secs,
            total_chunks,
            |chunk_peaks, chunk_idx, _total| {
                let _ = app_inner.emit(
                    "waveform-chunk",
                    serde_json::json!({
                        "deckId": deck_id_inner,
                        "chunk": chunk_peaks,
                        "chunkIndex": chunk_idx,
                        "totalChunks": total_chunks,
                    }),
                );
            },
        );

        {
            let mut lock = match state_clone.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let deck = if deck_id_analysis == "A" {
                &mut lock.deck_a
            } else {
                &mut lock.deck_b
            };
            deck.waveform_peaks = peaks;
            deck.waveform_ready = true;
        }

        let _ = app_clone.emit(
            "waveform-complete",
            serde_json::json!({ "deckId": deck_id_analysis }),
        );
    });

    Ok(TrackLoadedFast {
        title,
        duration: duration_str,
        duration_secs,
        sample_rate,
        deck_id,
    })
}

// ==================== PLAYBACK ====================
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
    let deck = if deck_id == "A" {
        &lock.deck_a
    } else {
        &lock.deck_b
    };
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

// ==================== VOLUME ====================
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
    deck.volume = vol.clamp(0.0, 1.0);
    Ok(())
}

// ==================== EQ & GAIN ====================
#[tauri::command]
pub fn set_deck_eq(
    deck_id: String,
    high: f32,
    mid: f32,
    low: f32,
    filter: f32,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" {
        &mut lock.deck_a
    } else {
        &mut lock.deck_b
    };
    deck.eq_high = high.clamp(-100.0, 100.0);
    deck.eq_mid = mid.clamp(-100.0, 100.0);
    deck.eq_low = low.clamp(-100.0, 100.0);
    deck.filter = filter.clamp(-100.0, 100.0);
    Ok(())
}

#[tauri::command]
pub fn set_deck_gain(
    deck_id: String,
    gain: f32,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" {
        &mut lock.deck_a
    } else {
        &mut lock.deck_b
    };
    deck.gain = gain.clamp(-100.0, 100.0);
    Ok(())
}

#[tauri::command]
pub fn set_crossfader(val: f32, state: State<'_, Arc<Mutex<MixerState>>>) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    lock.crossfader = val.clamp(0.0, 1.0);
    Ok(())
}

// ==================== FILE SYSTEM ====================
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

#[tauri::command]
pub fn set_deck_fx_type(
    deck_id: String,
    fx_type: String,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let fx = match fx_type.as_str() {
        "echo" => FxType::Echo,
        "reverb" => FxType::Reverb,
        "delay" => FxType::Delay,
        "flanger" => FxType::Flanger,
        "filter" => FxType::Filter,
        "roll" => FxType::Roll,
        _ => FxType::None,
    };
    if deck_id == "A" {
        lock.fx_type_a = fx;
    } else {
        lock.fx_type_b = fx;
    }
    Ok(())
}

// ✅ Set FX amount (0.0 to 1.0)
#[tauri::command]
pub fn set_deck_fx_amount(
    deck_id: String,
    amount: f32,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let amount = amount.clamp(0.0, 1.0);
    if deck_id == "A" {
        lock.fx_amount_a = amount;
    } else {
        lock.fx_amount_b = amount;
    }
    Ok(())
}

// ✅ Enable/Disable FX
#[tauri::command]
pub fn set_deck_fx_active(
    deck_id: String,
    active: bool,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    if deck_id == "A" {
        lock.fx_active_a = active;
        if !active {
            lock.fx_a.reset();
        }
    } else {
        lock.fx_active_b = active;
        if !active {
            lock.fx_b.reset();
        }
    }
    Ok(())
}

// ✅ Reset FX buffer
#[tauri::command]
pub fn reset_deck_fx(
    deck_id: String,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    if deck_id == "A" {
        lock.fx_a.reset();
    } else {
        lock.fx_b.reset();
    }
    Ok(())
}

// ✅ Hot Cue storage (in-memory)
pub struct HotCueStore {
    pub cues: HashMap<String, [f32; 4]>, // deck_id → [pos1, pos2, pos3, pos4]
}

// ✅ Loop state
pub struct LoopState {
    pub active_loop: Option<(f32, f32)>, // start_ratio, end_ratio
}

#[tauri::command]
pub fn trigger_hot_cue(
    deck_id: String,
    pad: usize,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<f32, String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let deck = if deck_id == "A" {
        &mut lock.deck_a
    } else {
        &mut lock.deck_b
    };

    // Default cue positions
    let positions = [0.05, 0.28, 0.52, 0.78];
    let pos = positions.get(pad - 1).copied().unwrap_or(0.0);
    deck.seek_ratio(pos);
    Ok(pos)
}

#[tauri::command]
pub fn toggle_loop(deck_id: String, beats: String) -> Result<Option<(f32, f32)>, String> {
    // TODO: Implement actual looping
    println!("[Loop] Deck {} — {} beats", deck_id, beats);
    Ok(None)
}

#[tauri::command]
pub fn toggle_stem(
    deck_id: String,
    stem: String,
    active: bool,
    _state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    println!("[Stem] Deck {} — {} = {}", deck_id, stem, active);
    Ok(())
}

// ✅ Echo Out / Reverb Out — Auto fade
#[tauri::command]
pub fn trigger_fx_out(
    deck_id: String,
    fx_type: String,    // "echo" or "reverb"
    duration_secs: f32, // e.g. 2.0 seconds
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;

    let fx = match fx_type.as_str() {
        "echo" => FxType::Echo,
        "reverb" => FxType::Reverb,
        _ => return Err("Invalid FX type".into()),
    };

    if deck_id == "A" {
        lock.fx_type_a = fx;
        lock.fx_active_a = true;
        lock.fx_amount_a = 0.0;
        lock.deck_a.fx_out_active = true;
        lock.deck_a.fx_out_progress = 0.0;
        lock.deck_a.fx_out_duration = duration_secs.max(0.5);
        lock.deck_a.fx_out_volume = 1.0;
    } else {
        lock.fx_type_b = fx;
        lock.fx_active_b = true;
        lock.fx_amount_b = 0.0;
        lock.deck_b.fx_out_active = true;
        lock.deck_b.fx_out_progress = 0.0;
        lock.deck_b.fx_out_duration = duration_secs.max(0.5);
        lock.deck_b.fx_out_volume = 1.0;
    }

    println!(
        "[FX Out] Deck {} — {} for {:.1}s",
        deck_id, fx_type, duration_secs
    );
    Ok(())
}

// ✅ Cancel FX Out
#[tauri::command]
pub fn cancel_fx_out(
    deck_id: String,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    if deck_id == "A" {
        lock.deck_a.fx_out_active = false;
        lock.deck_a.fx_out_volume = 1.0;
        lock.fx_active_a = false;
        lock.fx_amount_a = 0.0;
    } else {
        lock.deck_b.fx_out_active = false;
        lock.deck_b.fx_out_volume = 1.0;
        lock.fx_active_b = false;
        lock.fx_amount_b = 0.0;
    }
    Ok(())
}

// ==================== FX PARAMS COMMANDS ====================

#[tauri::command]
pub fn set_fx_params(
    deck_id: String,
    // Echo
    echo_time_beats: Option<f32>,
    echo_feedback: Option<f32>,
    echo_mix: Option<f32>,
    // Reverb
    reverb_decay: Option<f32>,
    reverb_damping: Option<f32>,
    reverb_mix: Option<f32>,
    // Delay
    delay_time_ms: Option<f32>,
    delay_feedback: Option<f32>,
    delay_mix: Option<f32>,
    // Flanger
    flanger_rate: Option<f32>,
    flanger_depth: Option<f32>,
    flanger_feedback: Option<f32>,
    // Filter
    filter_resonance: Option<f32>,
    filter_mode: Option<String>,
    // Roll
    roll_beats: Option<f32>,
    // BPM
    bpm: Option<f32>,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<(), String> {
    let mut lock = state.lock().map_err(|e| e.to_string())?;
    let params = if deck_id == "A" {
        &mut lock.fx_params_a
    } else {
        &mut lock.fx_params_b
    };

    if let Some(v) = echo_time_beats {
        params.echo_time_beats = v.clamp(0.0625, 4.0);
    }
    if let Some(v) = echo_feedback {
        params.echo_feedback = v.clamp(0.0, 0.95);
    }
    if let Some(v) = echo_mix {
        params.echo_mix = v.clamp(0.0, 1.0);
    }

    if let Some(v) = reverb_decay {
        params.reverb_decay = v.clamp(0.0, 1.0);
    }
    if let Some(v) = reverb_damping {
        params.reverb_damping = v.clamp(0.0, 1.0);
    }
    if let Some(v) = reverb_mix {
        params.reverb_mix = v.clamp(0.0, 1.0);
    }

    if let Some(v) = delay_time_ms {
        params.delay_time_ms = v.clamp(10.0, 2000.0);
    }
    if let Some(v) = delay_feedback {
        params.delay_feedback = v.clamp(0.0, 0.95);
    }
    if let Some(v) = delay_mix {
        params.delay_mix = v.clamp(0.0, 1.0);
    }

    if let Some(v) = flanger_rate {
        params.flanger_rate = v.clamp(0.1, 5.0);
    }
    if let Some(v) = flanger_depth {
        params.flanger_depth = v.clamp(0.0, 1.0);
    }
    if let Some(v) = flanger_feedback {
        params.flanger_feedback = v.clamp(0.0, 0.9);
    }

    if let Some(v) = filter_resonance {
        params.filter_resonance = v.clamp(0.1, 10.0);
    }
    if let Some(m) = filter_mode {
        params.filter_mode = match m.as_str() {
            "highpass" => FilterMode::HighPass,
            "bandpass" => FilterMode::BandPass,
            _ => FilterMode::LowPass,
        };
    }

    if let Some(v) = roll_beats {
        params.roll_beats = v.clamp(0.0625, 4.0);
    }
    if let Some(v) = bpm {
        params.echo_bpm = v.clamp(60.0, 200.0);
        params.roll_bpm = v.clamp(60.0, 200.0);
    }

    Ok(())
}

#[tauri::command]
pub fn get_fx_params(
    deck_id: String,
    state: State<'_, Arc<Mutex<MixerState>>>,
) -> Result<FxParams, String> {
    let lock = state.lock().map_err(|e| e.to_string())?;
    let params = if deck_id == "A" {
        lock.fx_params_a
    } else {
        lock.fx_params_b
    };
    Ok(params)
}
