use super::cache::StorageCache;
use super::paths::{atomic_write, playlists_path};
use super::types::{Playlist, PlaylistDatabase, PlaylistTrack};
use std::fs;

// ==================== LOAD ====================

pub fn load_playlists(
    drive_root: &str,
    cache: &StorageCache,
) -> Result<PlaylistDatabase, String> {
    let path = playlists_path(drive_root)?;
    let path_str = path.to_string_lossy().to_string();

    let json = cache.get_or_load(&path_str, || {
        if !path.exists() {
            // Create defaults
            let db = create_default_playlists();
            let json = serde_json::to_string_pretty(&db).map_err(|e| e.to_string())?;
            atomic_write(&path, &json)?;
            return Ok(json);
        }

        fs::read_to_string(&path).map_err(|e| e.to_string())
    })?;

    let db: PlaylistDatabase = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(db)
}

// ==================== SAVE ====================

pub fn save_playlists(
    drive_root: &str,
    db: &PlaylistDatabase,
    cache: &StorageCache,
) -> Result<(), String> {
    let path = playlists_path(drive_root)?;
    let json = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;

    atomic_write(&path, &json)?;

    // ✅ Update cache
    cache.update(&path.to_string_lossy(), json);

    let total: usize = db.playlists.iter().map(|p| p.tracks.len()).sum();
    println!(
        "[Storage] Saved {} playlists ({} tracks)",
        db.playlists.len(),
        total
    );
    Ok(())
}

// ==================== CRUD ====================

pub fn create_default_playlists() -> PlaylistDatabase {
    let mut db = PlaylistDatabase::new();

    let mut peak = Playlist::new("🔥 Peak Time Festival".to_string());
    peak.icon = "🔥".to_string();
    peak.color = "#f43f5e".to_string();

    let mut house = Playlist::new("⚡ Cyber Bass & House".to_string());
    house.icon = "⚡".to_string();
    house.color = "#a855f7".to_string();

    let mut sunset = Playlist::new("🌴 Deep Melodic Sunset".to_string());
    sunset.icon = "🌴".to_string();
    sunset.color = "#f59e0b".to_string();

    db.playlists = vec![peak, house, sunset];
    db
}

pub fn create_playlist(
    drive_root: &str,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    cache: &StorageCache,
) -> Result<Playlist, String> {
    let mut db = load_playlists(drive_root, cache)?;

    let mut playlist = Playlist::new(name);
    if let Some(i) = icon { playlist.icon = i; }
    if let Some(c) = color { playlist.color = c; }

    db.playlists.push(playlist.clone());
    save_playlists(drive_root, &db, cache)?;

    Ok(playlist)
}

pub fn delete_playlist(
    drive_root: &str,
    playlist_id: &str,
    cache: &StorageCache,
) -> Result<(), String> {
    let mut db = load_playlists(drive_root, cache)?;
    db.playlists.retain(|p| p.id != playlist_id);
    save_playlists(drive_root, &db, cache)?;
    Ok(())
}

pub fn rename_playlist(
    drive_root: &str,
    playlist_id: &str,
    new_name: String,
    cache: &StorageCache,
) -> Result<(), String> {
    let mut db = load_playlists(drive_root, cache)?;

    if let Some(pl) = db.playlists.iter_mut().find(|p| p.id == playlist_id) {
        pl.name = new_name;
        pl.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    save_playlists(drive_root, &db, cache)?;
    Ok(())
}

pub fn add_track(
    drive_root: &str,
    playlist_id: &str,
    track: PlaylistTrack,
    cache: &StorageCache,
) -> Result<(), String> {
    let mut db = load_playlists(drive_root, cache)?;

    if let Some(pl) = db.playlists.iter_mut().find(|p| p.id == playlist_id) {
        // កុំបន្ថែមស្ទួន
        if !pl.tracks.iter().any(|t| t.path == track.path) {
            pl.tracks.push(track);
            pl.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
        }
    }

    save_playlists(drive_root, &db, cache)?;
    Ok(())
}

pub fn remove_track(
    drive_root: &str,
    playlist_id: &str,
    track_path: &str,
    cache: &StorageCache,
) -> Result<(), String> {
    let mut db = load_playlists(drive_root, cache)?;

    if let Some(pl) = db.playlists.iter_mut().find(|p| p.id == playlist_id) {
        pl.tracks.retain(|t| t.path != track_path);
        pl.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    save_playlists(drive_root, &db, cache)?;
    Ok(())
}

pub fn reorder_tracks(
    drive_root: &str,
    playlist_id: &str,
    from_index: usize,
    to_index: usize,
    cache: &StorageCache,
) -> Result<(), String> {
    let mut db = load_playlists(drive_root, cache)?;

    if let Some(pl) = db.playlists.iter_mut().find(|p| p.id == playlist_id) {
        if from_index < pl.tracks.len() && to_index <= pl.tracks.len() {
            let track = pl.tracks.remove(from_index);
            pl.tracks.insert(to_index, track);
        }
    }

    save_playlists(drive_root, &db, cache)?;
    Ok(())
}