import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface HotCue {
  position: number;
  color: string;
  label: string;
  created_at: number;
}

export interface TrackCues {
  track_path: string;
  track_hash: string;
  bpm: number;
  key: string;
  hot_cues: HotCue[];
  saved_loops: any[];
  play_count: number;
  last_played: number;
}

export interface CuesDatabase {
  tracks: TrackCues[];
  version: number;
}

export function useCueStorage() {
  const database = ref<CuesDatabase>({ tracks: [], version: 1 });
  const activeDrive = ref<string>("");

  async function loadCues(driveRoot: string) {
    activeDrive.value = driveRoot;
    try {
      const db = await invoke<CuesDatabase>("load_cues", { driveRoot });
      database.value = db;
      console.log(`[Cues] ✅ Loaded ${db.tracks.length} tracks with cues`);
    } catch (err) {
      console.error("[Cues] ❌ Load failed:", err);
    }
  }

  async function saveCues() {
    if (!activeDrive.value) return;
    try {
      await invoke("save_cues", {
        driveRoot: activeDrive.value,
        db: database.value,
      });
    } catch (err) {
      console.error("[Cues] ❌ Save failed:", err);
    }
  }

  async function setHotCue(
    trackPath: string,
    padIndex: number,
    position: number,
    color: string = "#22d3ee",
    label: string = ""
  ) {
    if (!activeDrive.value) return;

    const cue: HotCue = {
      position,
      color,
      label,
      created_at: Math.floor(Date.now() / 1000),
    };

    await invoke("set_hot_cue", {
      driveRoot: activeDrive.value,
      trackPath,
      padIndex,
      cue,
    });

    await loadCues(activeDrive.value);
  }

  function getTrackCues(trackPath: string): TrackCues | undefined {
    return database.value.tracks.find((t) => t.track_path === trackPath);
  }

  function getHotCue(trackPath: string, padIndex: number): HotCue | null {
    const track = getTrackCues(trackPath);
    if (!track) return null;
    return track.hot_cues[padIndex] ?? null;
  }

  return {
    database,
    activeDrive,
    loadCues,
    saveCues,
    setHotCue,
    getTrackCues,
    getHotCue,
  };
}