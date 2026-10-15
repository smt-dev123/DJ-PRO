use std::fs;
use std::path::{Path, PathBuf};

pub const METADATA_FOLDER: &str = "_DjPro_";

/// ✅ យក path នៃ _DjPro_ folder
pub fn get_metadata_path(drive_root: &str) -> PathBuf {
    Path::new(drive_root).join(METADATA_FOLDER)
}

/// ✅ បង្កើត _DjPro_ folder បើមិនមាន
pub fn ensure_metadata_folder(drive_root: &str) -> Result<PathBuf, String> {
    let meta_path = get_metadata_path(drive_root);

    if !meta_path.exists() {
        fs::create_dir_all(&meta_path).map_err(|e| e.to_string())?;
        println!("[Storage] Created metadata folder: {:?}", meta_path);
    }

    Ok(meta_path)
}

/// ✅ យក drive root ពី track path
/// ឧទាហរណ៍: D:\Music\Album\Track.mp3 → D:\Music
pub fn find_drive_root(track_path: &str) -> Result<String, String> {
    let path = Path::new(track_path);

    // ដើរឡើងលើរក _DjPro_ folder
    let mut current = path.parent();

    while let Some(dir) = current {
        let meta = dir.join(METADATA_FOLDER);
        if meta.exists() {
            return Ok(dir.to_string_lossy().to_string());
        }
        current = dir.parent();
    }

    // បើរកមិនឃើញ → យក root drive
    if cfg!(target_os = "windows") {
        let s = track_path.to_string();
        if s.len() >= 3 {
            return Ok(s[..3].to_string());
        }
    }

    Err("Could not find drive root".to_string())
}

/// ✅ Atomic write (safe save)
pub fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

/// ✅ Playlists path
pub fn playlists_path(drive_root: &str) -> Result<PathBuf, String> {
    let meta = ensure_metadata_folder(drive_root)?;
    Ok(meta.join("playlists.json"))
}

/// ✅ Cues path
pub fn cues_path(drive_root: &str) -> Result<PathBuf, String> {
    let meta = ensure_metadata_folder(drive_root)?;
    Ok(meta.join("cues.json"))
}

/// ✅ Settings path
pub fn settings_path(drive_root: &str) -> Result<PathBuf, String> {
    let meta = ensure_metadata_folder(drive_root)?;
    Ok(meta.join("settings.json"))
}