use super::cache::StorageCache;
use super::paths::*;
use super::playlists;
use super::cues;
use super::settings;
use super::types::*;
use std::sync::Arc;
use tauri::State;

// ==================== DRIVE DETECTION ====================

#[tauri::command]
pub fn find_drive_root_cmd(track_path: String) -> Result<String, String> {
    find_drive_root(&track_path)
}

#[tauri::command]
pub fn scan_for_metadata_folders(
    search_paths: Vec<String>,
) -> Result<Vec<DriveMetadata>, String> {
    let mut results = Vec::new();

    for path in search_paths {
        let meta_path = get_metadata_path(&path);

        if meta_path.exists() {
            let playlists_file = meta_path.join("playlists.json");
            let (playlists_count, tracks_count, last_modified) = if playlists_file.exists() {
                if let Ok(json) = std::fs::read_to_string(&playlists_file) {
                    if let Ok(db) = serde_json::from_str::<PlaylistDatabase>(&json) {
                        let total: usize = db.playlists.iter().map(|p| p.tracks.len()).sum();
                        let modified = std::fs::metadata(&playlists_file)
                            .and_then(|m| m.modified())
                            .map(|t| t
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0))
                            .unwrap_or(0);
                        (db.playlists.len(), total, modified)
                    } else {
                        (0, 0, 0)
                    }
                } else {
                    (0, 0, 0)
                }
            } else {
                (0, 0, 0)
            };

            results.push(DriveMetadata {
                drive_root: path,
                has_metadata: true,
                playlists_count,
                tracks_count,
                last_modified,
            });
        }
    }

    Ok(results)
}

// ==================== PLAYLISTS ====================

#[tauri::command]
pub fn load_playlists(
    drive_root: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<PlaylistDatabase, String> {
    playlists::load_playlists(&drive_root, &cache)
}

#[tauri::command]
pub fn save_playlists(
    drive_root: String,
    db: PlaylistDatabase,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    playlists::save_playlists(&drive_root, &db, &cache)
}

#[tauri::command]
pub fn create_playlist(
    drive_root: String,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<Playlist, String> {
    playlists::create_playlist(&drive_root, name, icon, color, &cache)
}

#[tauri::command]
pub fn delete_playlist(
    drive_root: String,
    playlist_id: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    playlists::delete_playlist(&drive_root, &playlist_id, &cache)
}

#[tauri::command]
pub fn rename_playlist(
    drive_root: String,
    playlist_id: String,
    new_name: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    playlists::rename_playlist(&drive_root, &playlist_id, new_name, &cache)
}

#[tauri::command]
pub fn add_track_to_playlist(
    drive_root: String,
    playlist_id: String,
    track: PlaylistTrack,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    playlists::add_track(&drive_root, &playlist_id, track, &cache)
}

#[tauri::command]
pub fn remove_track_from_playlist(
    drive_root: String,
    playlist_id: String,
    track_path: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    playlists::remove_track(&drive_root, &playlist_id, &track_path, &cache)
}

#[tauri::command]
pub fn reorder_playlist_tracks(
    drive_root: String,
    playlist_id: String,
    from_index: usize,
    to_index: usize,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    playlists::reorder_tracks(&drive_root, &playlist_id, from_index, to_index, &cache)
}

// ==================== CUES ====================

#[tauri::command]
pub fn load_cues(
    drive_root: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<CuesDatabase, String> {
    cues::load_cues(&drive_root, &cache)
}

#[tauri::command]
pub fn save_cues(
    drive_root: String,
    db: CuesDatabase,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    cues::save_cues(&drive_root, &db, &cache)
}

#[tauri::command]
pub fn upsert_track_cues(
    drive_root: String,
    track_cues: TrackCues,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    cues::upsert_track_cues(&drive_root, track_cues, &cache)
}

#[tauri::command]
pub fn set_hot_cue(
    drive_root: String,
    track_path: String,
    pad_index: usize,
    cue: HotCue,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    cues::set_hot_cue(&drive_root, &track_path, pad_index, cue, &cache)
}

// ==================== SETTINGS ====================

#[tauri::command]
pub fn load_settings(
    drive_root: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<AppSettings, String> {
    settings::load_settings(&drive_root, &cache)
}

#[tauri::command]
pub fn save_settings(
    drive_root: String,
    settings: AppSettings,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    settings::save_settings(&drive_root, &settings, &cache)
}

// ==================== BATCH ====================

#[tauri::command]
pub fn save_all(
    req: SaveAllRequest,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    playlists::save_playlists(&req.drive_root, &req.playlists, &cache)?;
    cues::save_cues(&req.drive_root, &req.cues, &cache)?;
    settings::save_settings(&req.drive_root, &req.settings, &cache)?;
    println!("[Storage] ✅ Saved ALL to {}", req.drive_root);
    Ok(())
}

#[tauri::command]
pub fn load_all(
    drive_root: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(PlaylistDatabase, CuesDatabase, AppSettings), String> {
    let playlists = playlists::load_playlists(&drive_root, &cache)?;
    let cues = cues::load_cues(&drive_root, &cache)?;
    let settings = settings::load_settings(&drive_root, &cache)?;
    Ok((playlists, cues, settings))
}

// ==================== CACHE ====================

#[tauri::command]
pub fn cache_stats(
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(usize, u64), String> {
    Ok(cache.stats())
}

#[tauri::command]
pub fn cache_clear(
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    cache.clear();
    Ok(())
}

#[tauri::command]
pub fn cache_invalidate(
    path: String,
    cache: State<'_, Arc<StorageCache>>,
) -> Result<(), String> {
    cache.invalidate(&path);
    Ok(())
}