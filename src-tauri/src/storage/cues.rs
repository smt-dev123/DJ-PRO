use super::cache::StorageCache;
use super::paths::{atomic_write, cues_path};
use super::types::{CuesDatabase, HotCue, TrackCues};
use std::fs;

pub fn load_cues(
    drive_root: &str,
    cache: &StorageCache,
) -> Result<CuesDatabase, String> {
    let path = cues_path(drive_root)?;
    let path_str = path.to_string_lossy().to_string();

    let json = cache.get_or_load(&path_str, || {
        if !path.exists() {
            return Ok("{\"tracks\":[],\"version\":1}".to_string());
        }
        fs::read_to_string(&path).map_err(|e| e.to_string())
    })?;

    let db: CuesDatabase = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(db)
}

pub fn save_cues(
    drive_root: &str,
    db: &CuesDatabase,
    cache: &StorageCache,
) -> Result<(), String> {
    let path = cues_path(drive_root)?;
    let json = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)?;
    cache.update(&path.to_string_lossy(), json);
    Ok(())
}

pub fn upsert_track_cues(
    drive_root: &str,
    track_cues: TrackCues,
    cache: &StorageCache,
) -> Result<(), String> {
    let mut db = load_cues(drive_root, cache)?;

    if let Some(existing) = db
        .tracks
        .iter_mut()
        .find(|t| t.track_path == track_cues.track_path)
    {
        *existing = track_cues;
    } else {
        db.tracks.push(track_cues);
    }

    save_cues(drive_root, &db, cache)?;
    Ok(())
}

pub fn set_hot_cue(
    drive_root: &str,
    track_path: &str,
    pad_index: usize,
    cue: HotCue,
    cache: &StorageCache,
) -> Result<(), String> {
    let mut db = load_cues(drive_root, cache)?;

    if let Some(track) = db.tracks.iter_mut().find(|t| t.track_path == track_path) {
        while track.hot_cues.len() <= pad_index {
            track.hot_cues.push(HotCue {
                position: 0.0,
                color: "#22d3ee".to_string(),
                label: String::new(),
                created_at: 0,
            });
        }
        track.hot_cues[pad_index] = cue;
    } else {
        let mut new_track = TrackCues {
            track_path: track_path.to_string(),
            track_hash: String::new(),
            bpm: 0.0,
            key: String::new(),
            hot_cues: vec![],
            saved_loops: vec![],
            play_count: 0,
            last_played: 0,
        };

        while new_track.hot_cues.len() <= pad_index {
            new_track.hot_cues.push(HotCue {
                position: 0.0,
                color: "#22d3ee".to_string(),
                label: String::new(),
                created_at: 0,
            });
        }
        new_track.hot_cues[pad_index] = cue;
        db.tracks.push(new_track);
    }

    save_cues(drive_root, &db, cache)?;
    Ok(())
}