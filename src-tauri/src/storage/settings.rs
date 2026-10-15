use super::cache::StorageCache;
use super::paths::{atomic_write, settings_path};
use super::types::AppSettings;
use std::fs;

pub fn load_settings(
    drive_root: &str,
    cache: &StorageCache,
) -> Result<AppSettings, String> {
    let path = settings_path(drive_root)?;
    let path_str = path.to_string_lossy().to_string();

    let json = cache.get_or_load(&path_str, || {
        if !path.exists() {
            let settings = AppSettings::default();
            let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
            atomic_write(&path, &json)?;
            return Ok(json);
        }
        fs::read_to_string(&path).map_err(|e| e.to_string())
    })?;

    let settings: AppSettings = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(settings)
}

pub fn save_settings(
    drive_root: &str,
    settings: &AppSettings,
    cache: &StorageCache,
) -> Result<(), String> {
    let path = settings_path(drive_root)?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)?;
    cache.update(&path.to_string_lossy(), json);
    Ok(())
}