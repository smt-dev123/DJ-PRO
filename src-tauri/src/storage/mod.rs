// ==================== STORAGE MODULE ====================
// ✅ Drive-based storage + RAM cache + Auto-save

pub mod cache;
pub mod commands;
pub mod cues;
pub mod paths;
pub mod playlists;
pub mod settings;
pub mod types;

pub use cache::StorageCache;
pub use types::*;

// Global cache instance
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static STORAGE_CACHE: Lazy<Arc<StorageCache>> =
    Lazy::new(|| Arc::new(StorageCache::new()));