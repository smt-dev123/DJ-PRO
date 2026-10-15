use serde::{Deserialize, Serialize};

// ==================== PLAYLIST ====================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaylistTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub key: String,
    pub duration: String,
    pub path: String,
    #[serde(default)]
    pub added_at: u64,
    #[serde(default)]
    pub play_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub tracks: Vec<PlaylistTrack>,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub icon: String,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id: format!("pl_{}", now),
            name,
            description: String::new(),
            created_at: now,
            updated_at: now,
            tracks: Vec::new(),
            color: "#22d3ee".to_string(),
            icon: "🎵".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlaylistDatabase {
    pub playlists: Vec<Playlist>,
    pub version: u32,
}

impl PlaylistDatabase {
    pub fn new() -> Self {
        Self {
            playlists: Vec::new(),
            version: 1,
        }
    }
}

// ==================== HOT CUE ====================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotCue {
    pub position: f32,
    pub color: String,
    pub label: String,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedLoop {
    pub start: f32,
    pub end: f32,
    pub beats: f32,
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackCues {
    pub track_path: String,
    pub track_hash: String,
    pub bpm: f32,
    pub key: String,
    pub hot_cues: Vec<HotCue>,
    pub saved_loops: Vec<SavedLoop>,
    #[serde(default)]
    pub play_count: u32,
    #[serde(default)]
    pub last_played: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CuesDatabase {
    pub tracks: Vec<TrackCues>,
    pub version: u32,
}

// ==================== SETTINGS ====================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeckSettings {
    pub volume: f32,
    pub eq_high: f32,
    pub eq_mid: f32,
    pub eq_low: f32,
    pub filter: f32,
    pub gain: f32,
    pub key_lock: bool,
    pub fx_type: String,
    pub fx_amount: f32,
    #[serde(default)]
    pub fx_params: serde_json::Value,
}

impl Default for DeckSettings {
    fn default() -> Self {
        Self {
            volume: 0.85,
            eq_high: 0.0,
            eq_mid: 0.0,
            eq_low: 0.0,
            filter: 0.0,
            gain: 0.0,
            key_lock: true,
            fx_type: "none".to_string(),
            fx_amount: 0.0,
            fx_params: serde_json::json!({}),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub crossfader: f32,
    pub master_volume: f32,
    pub deck_a: DeckSettings,
    pub deck_b: DeckSettings,
    pub recent_tracks: Vec<String>,
    pub version: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            crossfader: 0.5,
            master_volume: 0.85,
            deck_a: DeckSettings::default(),
            deck_b: DeckSettings::default(),
            recent_tracks: Vec::new(),
            version: 1,
        }
    }
}

// ==================== DRIVE METADATA ====================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DriveMetadata {
    pub drive_root: String,
    pub has_metadata: bool,
    pub playlists_count: usize,
    pub tracks_count: usize,
    pub last_modified: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveAllRequest {
    pub drive_root: String,
    pub playlists: PlaylistDatabase,
    pub cues: CuesDatabase,
    pub settings: AppSettings,
}