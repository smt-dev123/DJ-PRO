import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// ==================== TYPES ====================
export interface PlaylistTrack {
  id: string;
  title: string;
  artist: string;
  bpm: number;
  key: string;
  duration: string;
  path: string;
  added_at: number;
  play_count: number;
}

export interface Playlist {
  id: string;
  name: string;
  description: string;
  created_at: number;
  updated_at: number;
  tracks: PlaylistTrack[];
  color: string;
  icon: string;
}

export interface PlaylistDatabase {
  playlists: Playlist[];
  version: number;
}

// ==================== COMPOSABLE ====================
export function usePlaylistStorage() {
  const database = ref<PlaylistDatabase>({ playlists: [], version: 1 });
  const activeDrive = ref<string>("");
  const isLoading = ref(false);

  // ✅ Auto-save debounce
  let saveTimer: number | null = null;

  function scheduleSave() {
    if (!activeDrive.value) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = window.setTimeout(async () => {
      await saveToDisk();
    }, 1500);
  }

  // ==================== DRIVE ====================

  async function setDrive(driveRoot: string) {
    activeDrive.value = driveRoot;
    await loadPlaylists();
  }

  async function findAndSetDrive(trackPath: string) {
    try {
      const root = await invoke<string>("find_drive_root_cmd", { trackPath });
      await setDrive(root);
      return root;
    } catch (err) {
      console.warn("[Playlist] Could not find drive root");
      return null;
    }
  }

  // ==================== LOAD / SAVE ====================

  async function loadPlaylists() {
    if (!activeDrive.value) {
      console.warn("[Playlist] No active drive");
      return;
    }

    isLoading.value = true;
    try {
      const db = await invoke<PlaylistDatabase>("load_playlists", {
        driveRoot: activeDrive.value,
      });
      database.value = db;
      console.log(
        `[Playlist] ✅ Loaded ${db.playlists.length} playlists (cached)`
      );
    } catch (err) {
      console.error("[Playlist] ❌ Load failed:", err);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveToDisk() {
    if (!activeDrive.value) return;
    try {
      await invoke("save_playlists", {
        driveRoot: activeDrive.value,
        db: database.value,
      });
      console.log("[Playlist] ✅ Saved");
    } catch (err) {
      console.error("[Playlist] ❌ Save failed:", err);
    }
  }

  async function saveNow() {
    if (saveTimer) clearTimeout(saveTimer);
    await saveToDisk();
  }

  // ==================== CRUD ====================

  async function createPlaylist(
    name: string,
    icon?: string,
    color?: string
  ): Promise<Playlist | null> {
    if (!activeDrive.value) return null;

    try {
      const playlist = await invoke<Playlist>("create_playlist", {
        driveRoot: activeDrive.value,
        name,
        icon: icon ?? null,
        color: color ?? null,
      });
      await loadPlaylists();
      return playlist;
    } catch (err) {
      console.error("[Playlist] ❌ Create failed:", err);
      return null;
    }
  }

  async function deletePlaylist(playlistId: string) {
    if (!activeDrive.value) return;
    await invoke("delete_playlist", {
      driveRoot: activeDrive.value,
      playlistId,
    });
    await loadPlaylists();
  }

  async function renamePlaylist(playlistId: string, newName: string) {
    if (!activeDrive.value) return;
    await invoke("rename_playlist", {
      driveRoot: activeDrive.value,
      playlistId,
      newName,
    });
    await loadPlaylists();
  }

  async function addTrackToPlaylist(
    playlistId: string,
    track: Omit<PlaylistTrack, "added_at" | "play_count">
  ) {
    if (!activeDrive.value) return;
    const fullTrack: PlaylistTrack = {
      ...track,
      added_at: Math.floor(Date.now() / 1000),
      play_count: 0,
    };
    await invoke("add_track_to_playlist", {
      driveRoot: activeDrive.value,
      playlistId,
      track: fullTrack,
    });
    await loadPlaylists();
  }

  async function removeTrackFromPlaylist(
    playlistId: string,
    trackPath: string
  ) {
    if (!activeDrive.value) return;
    await invoke("remove_track_from_playlist", {
      driveRoot: activeDrive.value,
      playlistId,
      trackPath,
    });
    await loadPlaylists();
  }

  async function reorderPlaylistTracks(
    playlistId: string,
    fromIndex: number,
    toIndex: number
  ) {
    if (!activeDrive.value) return;
    await invoke("reorder_playlist_tracks", {
      driveRoot: activeDrive.value,
      playlistId,
      fromIndex,
      toIndex,
    });
    await loadPlaylists();
  }

  function getPlaylist(playlistId: string): Playlist | undefined {
    return database.value.playlists.find((p) => p.id === playlistId);
  }

  return {
    database,
    activeDrive,
    isLoading,
    setDrive,
    findAndSetDrive,
    loadPlaylists,
    saveToDisk,
    saveNow,
    scheduleSave,
    createPlaylist,
    deletePlaylist,
    renamePlaylist,
    addTrackToPlaylist,
    removeTrackFromPlaylist,
    reorderPlaylistTracks,
    getPlaylist,
  };
}