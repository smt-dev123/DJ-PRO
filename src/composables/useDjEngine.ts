import { reactive, ref, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { DeckId, DeckState, LibraryTrack, WaveformPeak, FxParams } from "../types/dj";
import { usePlaylistStorage } from "./usePlaylistStorage";
import { useCueStorage } from "./useCueStorage";
import { useSettingsStorage } from "./useSettingsStorage";

// ==================== INTERFACES ====================
interface TrackLoadedFast {
  title: string;
  duration: string;
  duration_secs: number;
  sample_rate: number;
  deck_id: string;
}

interface WaveformChunkPayload {
  deckId: "A" | "B";
  chunk: WaveformPeak[];
  chunkIndex: number;
  totalChunks: number;
}

interface TrackReadyPayload {
  deckId: "A" | "B";
  title: string;
  duration: string;
  durationSecs: number;
}

// ==================== DEFAULT VALUES ====================
const defaultFxParams: FxParams = {
  echo_time_beats: 0.375,
  echo_feedback: 0.6,
  echo_mix: 0.8,
  reverb_decay: 0.7,
  reverb_damping: 0.5,
  reverb_mix: 0.7,
  delay_time_ms: 100,
  delay_feedback: 0.3,
  delay_mix: 0.5,
  flanger_rate: 0.5,
  flanger_depth: 0.7,
  flanger_feedback: 0.5,
  filter_resonance: 0.7,
  filter_mode: "lowpass",
  roll_beats: 0.25,
};

// ==================== COMPOSABLE ====================
export function useDjEngine() {
  // ==================== DECK A ====================
  const deckA = reactive<DeckState>({
    id: "A",
    isPlaying: false,
    volume: 0.85,
    pitch: 0.0,
    baseBpm: 126.0,
    bpm: 126.0,
    currentTime: "00:00",
    totalTime: "00:00",
    title: "No Track Loaded",
    artist: "—",
    key: "8A",
    status: "EMPTY",
    eqHigh: 0,
    eqMid: 0,
    eqLow: 0,
    filter: 0,
    gain: 0,
    keyLock: true,
    isSynced: false,
    cueActive: false,
    progress: 0,
    waveform: [],
    durationSecs: 0,
    fxType: "none",
    fxAmount: 0,
    fxActive: false,
    fxParams: { ...defaultFxParams },
    activeLoop: null,
    stems: { vocal: true, drum: true, bass: true, melody: true },
  });

  // ==================== DECK B ====================
  const deckB = reactive<DeckState>({
    id: "B",
    isPlaying: false,
    volume: 0.85,
    pitch: 0.0,
    baseBpm: 128.0,
    bpm: 128.0,
    currentTime: "00:00",
    totalTime: "00:00",
    title: "No Track Loaded",
    artist: "—",
    key: "4B",
    status: "EMPTY",
    eqHigh: 0,
    eqMid: 0,
    eqLow: 0,
    filter: 0,
    gain: 0,
    keyLock: true,
    isSynced: false,
    cueActive: false,
    progress: 0,
    waveform: [],
    durationSecs: 0,
    fxType: "none",
    fxAmount: 0,
    fxActive: false,
    fxParams: { ...defaultFxParams },
    activeLoop: null,
    stems: { vocal: true, drum: true, bass: true, melody: true },
  });

  // ==================== MASTER ====================
  const crossfader = ref(0.5);
  const masterVolume = ref(0.85);
  const isDecoding = ref<{ A: boolean; B: boolean }>({ A: false, B: false });

  // ==================== STORAGE ====================
  const playlist = usePlaylistStorage();
  const cues = useCueStorage();
  const appSettings = useSettingsStorage();

  // ==================== EVENT LISTENERS ====================
  let unlistenChunk: (() => void) | null = null;
  let unlistenReady: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;
  let tickerId: number | null = null;
  let saveTimer: number | null = null;

  async function setupListeners() {
    unlistenChunk = await listen<WaveformChunkPayload>("waveform-chunk", (event) => {
      const { deckId, chunk } = event.payload;
      const deck = deckId === "A" ? deckA : deckB;
      deck.waveform = [...deck.waveform, ...chunk];
    });

    unlistenComplete = await listen<any>("waveform-complete", (event) => {
      const { deckId } = event.payload;
      const deck = deckId === "A" ? deckA : deckB;
      console.log(`[Waveform] Deck ${deckId}: ✅ COMPLETE (${deck.waveform.length} peaks)`);
    });

    unlistenReady = await listen<TrackReadyPayload>("track-ready", (event) => {
      const { deckId, title, duration, durationSecs } = event.payload;
      const deck = deckId === "A" ? deckA : deckB;
      deck.title = title;
      deck.totalTime = duration;
      deck.durationSecs = durationSecs;
      deck.status = "READY";
      deck.waveform = [];
      console.log(`[Track] Deck ${deckId}: ✅ READY`);
    });
  }

  // ==================== PLAYBACK TICKER ====================
  function updatePlaybackPositions() {
    if (deckA.isPlaying) {
      invoke<any>("get_deck_playback", { deckId: "A" })
        .then((res) => {
          if (res) {
            deckA.progress = res.progress;
            deckA.currentTime = res.current_time_str;
          }
        })
        .catch(() => {});
    }
    if (deckB.isPlaying) {
      invoke<any>("get_deck_playback", { deckId: "B" })
        .then((res) => {
          if (res) {
            deckB.progress = res.progress;
            deckB.currentTime = res.current_time_str;
          }
        })
        .catch(() => {});
    }
  }

  // ==================== LOAD TRACKS ====================
  async function openTrack(deckId: DeckId) {
    if (isDecoding.value[deckId]) return;
    const selected = await open({
      multiple: false,
      filters: [{ name: "Audio Files", extensions: ["mp3", "wav", "aac", "m4a", "flac", "ogg"] }],
    });
    if (selected && typeof selected === "string") {
      await loadFileToDeck(deckId, selected);
    }
  }

  async function loadDirectTrack(deckId: DeckId, track: LibraryTrack) {
    if (isDecoding.value[deckId]) return;
    isDecoding.value[deckId] = true;

    const deck = deckId === "A" ? deckA : deckB;
    deck.status = "LOADING...";
    deck.title = track.title;
    deck.artist = track.artist || "Original Artist";
    deck.baseBpm = track.bpm;
    deck.bpm = Number((track.bpm * (1 + deck.pitch / 100)).toFixed(1));
    deck.key = track.key || "8A";
    deck.currentTime = "00:00";
    deck.totalTime = track.duration;
    deck.progress = 0.0;
    deck.isPlaying = false;
    deck.waveform = [];

    try {
      await invoke<TrackLoadedFast>("load_track_fast", { deckId, path: track.path });

      // ✅ Auto-detect drive & load playlists/cues
      if (!playlist.activeDrive.value) {
        await playlist.findAndSetDrive(track.path);
        if (playlist.activeDrive.value) {
          await cues.loadCues(playlist.activeDrive.value);
          await loadSettingsFromDrive();
        }
      }
    } catch (err) {
      console.error(`[Deck ${deckId}] Load error:`, err);
      deck.status = "ERROR";
    } finally {
      isDecoding.value[deckId] = false;
    }
  }

  async function loadFileToDeck(deckId: DeckId, filePath: string) {
    if (isDecoding.value[deckId]) return;
    isDecoding.value[deckId] = true;

    const deck = deckId === "A" ? deckA : deckB;
    deck.status = "LOADING...";
    const fileName = filePath.split(/[\\/]/).pop()?.replace(/\.[^/.]+$/, "") || "Track";
    deck.title = fileName;
    deck.artist = "Local File";
    deck.isPlaying = false;
    deck.currentTime = "00:00";
    deck.progress = 0.0;
    deck.waveform = [];

    try {
      await invoke<TrackLoadedFast>("load_track_fast", { deckId, path: filePath });

      // ✅ Auto-detect drive & load playlists/cues
      if (!playlist.activeDrive.value) {
        await playlist.findAndSetDrive(filePath);
        if (playlist.activeDrive.value) {
          await cues.loadCues(playlist.activeDrive.value);
          await loadSettingsFromDrive();
        }
      }
    } catch (err) {
      console.error(`[Deck ${deckId}] Load error:`, err);
      deck.status = "ERROR";
    } finally {
      isDecoding.value[deckId] = false;
    }
  }

  // ==================== SETTINGS PERSISTENCE ====================
  async function loadSettingsFromDrive() {
    if (!playlist.activeDrive.value) return;
    const s = await appSettings.loadSettings(playlist.activeDrive.value);
    if (!s) return;

    // Apply Deck A
    deckA.volume = s.deck_a.volume;
    deckA.eqHigh = s.deck_a.eq_high;
    deckA.eqMid = s.deck_a.eq_mid;
    deckA.eqLow = s.deck_a.eq_low;
    deckA.filter = s.deck_a.filter;
    deckA.gain = s.deck_a.gain;
    deckA.keyLock = s.deck_a.key_lock;
    deckA.fxType = s.deck_a.fx_type as any;
    deckA.fxAmount = s.deck_a.fx_amount;
    if (s.deck_a.fx_params && Object.keys(s.deck_a.fx_params).length > 0) {
      deckA.fxParams = { ...deckA.fxParams, ...s.deck_a.fx_params };
    }

    // Apply Deck B
    deckB.volume = s.deck_b.volume;
    deckB.eqHigh = s.deck_b.eq_high;
    deckB.eqMid = s.deck_b.eq_mid;
    deckB.eqLow = s.deck_b.eq_low;
    deckB.filter = s.deck_b.filter;
    deckB.gain = s.deck_b.gain;
    deckB.keyLock = s.deck_b.key_lock;
    deckB.fxType = s.deck_b.fx_type as any;
    deckB.fxAmount = s.deck_b.fx_amount;
    if (s.deck_b.fx_params && Object.keys(s.deck_b.fx_params).length > 0) {
      deckB.fxParams = { ...deckB.fxParams, ...s.deck_b.fx_params };
    }

    // Apply master
    crossfader.value = s.crossfader;
    masterVolume.value = s.master_volume;

    // Sync ទៅ backend
    await syncDeckStateToBackend("A");
    await syncDeckStateToBackend("B");

    console.log("[Settings] ✅ Applied from drive");
  }

  async function syncDeckStateToBackend(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    try {
      await invoke("set_deck_eq", {
        deckId,
        high: deck.eqHigh,
        mid: deck.eqMid,
        low: deck.eqLow,
        filter: deck.filter,
      });
      await invoke("set_deck_gain", { deckId, gain: deck.gain });
      await invoke("set_deck_volume", { deckId, vol: deck.volume });
      await invoke("set_fx_params", {
        deckId,
        echoTimeBeats: deck.fxParams.echo_time_beats,
        echoFeedback: deck.fxParams.echo_feedback,
        echoMix: deck.fxParams.echo_mix,
        reverbDecay: deck.fxParams.reverb_decay,
        reverbDamping: deck.fxParams.reverb_damping,
        reverbMix: deck.fxParams.reverb_mix,
        delayTimeMs: deck.fxParams.delay_time_ms,
        delayFeedback: deck.fxParams.delay_feedback,
        delayMix: deck.fxParams.delay_mix,
        flangerRate: deck.fxParams.flanger_rate,
        flangerDepth: deck.fxParams.flanger_depth,
        flangerFeedback: deck.fxParams.flanger_feedback,
        filterResonance: deck.fxParams.filter_resonance,
        filterMode: deck.fxParams.filter_mode,
        rollBeats: deck.fxParams.roll_beats,
        bpm: deck.bpm,
      });
    } catch (err) {
      console.warn(`[Sync ${deckId}] Backend sync failed:`, err);
    }
  }

  function scheduleSettingsSave() {
    if (!playlist.activeDrive.value) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = window.setTimeout(async () => {
      const s = {
        crossfader: crossfader.value,
        master_volume: masterVolume.value,
        deck_a: {
          volume: deckA.volume,
          eq_high: deckA.eqHigh,
          eq_mid: deckA.eqMid,
          eq_low: deckA.eqLow,
          filter: deckA.filter,
          gain: deckA.gain,
          key_lock: deckA.keyLock,
          fx_type: deckA.fxType,
          fx_amount: deckA.fxAmount,
          fx_params: deckA.fxParams,
        },
        deck_b: {
          volume: deckB.volume,
          eq_high: deckB.eqHigh,
          eq_mid: deckB.eqMid,
          eq_low: deckB.eqLow,
          filter: deckB.filter,
          gain: deckB.gain,
          key_lock: deckB.keyLock,
          fx_type: deckB.fxType,
          fx_amount: deckB.fxAmount,
          fx_params: deckB.fxParams,
        },
        recent_tracks: [],
        version: 1,
      };
      await appSettings.saveSettings(s);
    }, 2000);
  }

  // ==================== PLAYBACK CONTROLS ====================
  async function seekDeck(deckId: DeckId, ratio: number) {
    const deck = deckId === "A" ? deckA : deckB;
    const clampedRatio = Math.max(0, Math.min(1, ratio));
    deck.progress = clampedRatio;
    const curSecs = Math.round(clampedRatio * deck.durationSecs);
    const mins = Math.floor(curSecs / 60);
    const secs = curSecs % 60;
    deck.currentTime = `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    try {
      await invoke("seek_deck", { deckId, ratio: clampedRatio });
    } catch {}
  }

  async function togglePlay(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    try {
      const playing = await invoke<boolean>("toggle_play", { deckId });
      deck.isPlaying = playing;
      deck.status = playing ? "PLAYING" : "READY";
    } catch {
      deck.isPlaying = !deck.isPlaying;
      deck.status = deck.isPlaying ? "PLAYING" : "READY";
    }
  }

  async function triggerCue(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.isPlaying = false;
    deck.cueActive = true;
    await seekDeck(deckId, 0.0);
    setTimeout(() => {
      deck.cueActive = false;
    }, 200);
  }

  async function syncDeck(deckId: DeckId) {
    const target = deckId === "A" ? deckA : deckB;
    const source = deckId === "A" ? deckB : deckA;
    target.bpm = source.bpm;
    target.pitch = Number(((source.bpm / target.baseBpm - 1) * 100).toFixed(1));
    target.isSynced = true;
    setTimeout(() => {
      target.isSynced = false;
    }, 1500);
  }

  async function toggleKeyLock(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.keyLock = !deck.keyLock;
  }

  // ==================== VOLUME ====================
  async function setVolume(deckId: DeckId, vol: number) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.volume = Math.max(0, Math.min(1, vol));
    try {
      await invoke("set_deck_volume", { deckId, vol: deck.volume });
    } catch {}
  }

  // ==================== EQ & GAIN ====================
  async function setDeckEq(
    deckId: DeckId,
    high: number,
    mid: number,
    low: number,
    filter: number
  ) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.eqHigh = high;
    deck.eqMid = mid;
    deck.eqLow = low;
    deck.filter = filter;
    try {
      await invoke("set_deck_eq", { deckId, high, mid, low, filter });
    } catch (err) {
      console.error(`[EQ ${deckId}] Error:`, err);
    }
  }

  async function setDeckGain(deckId: DeckId, gain: number) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.gain = gain;
    try {
      await invoke("set_deck_gain", { deckId, gain });
    } catch (err) {
      console.error(`[Gain ${deckId}] Error:`, err);
    }
  }

  // ==================== FX ====================
  async function setDeckFxType(deckId: DeckId, fxType: string) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.fxType = fxType as any;
    try {
      await invoke("set_deck_fx_type", { deckId, fxType });
    } catch (err) {
      console.error(`[FX ${deckId}] Error:`, err);
    }
  }

  async function setDeckFxAmount(deckId: DeckId, amount: number) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.fxAmount = Math.max(0, Math.min(1, amount));
    try {
      await invoke("set_deck_fx_amount", { deckId, amount: deck.fxAmount });
    } catch {}
  }

  async function setDeckFxActive(deckId: DeckId, active: boolean) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.fxActive = active;
    try {
      await invoke("set_deck_fx_active", { deckId, active });
    } catch {}
  }

  async function resetDeckFx(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.fxAmount = 0;
    deck.fxActive = false;
    try {
      await invoke("reset_deck_fx", { deckId });
    } catch {}
  }

  async function updateFxParams(deckId: DeckId, params: Partial<FxParams>) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.fxParams = { ...deck.fxParams, ...params };

    const p = deck.fxParams;
    try {
      await invoke("set_fx_params", {
        deckId,
        echoTimeBeats: p.echo_time_beats,
        echoFeedback: p.echo_feedback,
        echoMix: p.echo_mix,
        reverbDecay: p.reverb_decay,
        reverbDamping: p.reverb_damping,
        reverbMix: p.reverb_mix,
        delayTimeMs: p.delay_time_ms,
        delayFeedback: p.delay_feedback,
        delayMix: p.delay_mix,
        flangerRate: p.flanger_rate,
        flangerDepth: p.flanger_depth,
        flangerFeedback: p.flanger_feedback,
        filterResonance: p.filter_resonance,
        filterMode: p.filter_mode,
        rollBeats: p.roll_beats,
        bpm: deck.bpm,
      });
    } catch (err) {
      console.error(`[FX Params ${deckId}] Error:`, err);
    }
  }

  async function triggerFxOut(
    deckId: DeckId,
    fxType: "echo" | "reverb",
    duration: number = 2.5
  ) {
    const deck = deckId === "A" ? deckA : deckB;
    console.log(`[FX Out ${deckId}] ${fxType} for ${duration}s`);
    deck.fxType = fxType;
    deck.fxActive = true;
    deck.fxAmount = 0;

    try {
      await invoke("trigger_fx_out", { deckId, fxType, durationSecs: duration });
    } catch (err) {
      console.error(`[FX Out ${deckId}] Error:`, err);
    }
  }

  async function cancelFxOut(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.fxActive = false;
    deck.fxAmount = 0;
    try {
      await invoke("cancel_fx_out", { deckId });
    } catch {}
  }

  // ==================== HOT CUE ====================
  async function triggerHotCue(deckId: DeckId, pad: number) {
    const deck = deckId === "A" ? deckA : deckB;

    // Check បើមាន cue saved
    const savedCue = cues.getHotCue(deck.title, pad - 1);

    let pos = 0;
    if (savedCue && savedCue.position >= 0) {
      pos = savedCue.position;
    } else {
      // Default positions
      const cuePositions = [0.05, 0.28, 0.52, 0.78];
      pos = cuePositions[pad - 1] ?? 0;
    }

    console.log(`[HotCue ${deckId}] Pad ${pad} → ${Math.round(pos * 100)}%`);
    await seekDeck(deckId, pos);

    deck.cueActive = true;
    setTimeout(() => {
      deck.cueActive = false;
    }, 200);
  }

  // ==================== LOOP ====================
  async function toggleLoop(deckId: DeckId, beats: string) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.activeLoop = deck.activeLoop === beats ? null : beats;
    console.log(`[Loop ${deckId}] ${deck.activeLoop || "OFF"}`);
  }

  // ==================== STEMS ====================
  async function toggleStem(deckId: DeckId, stem: string, active: boolean) {
    const deck = deckId === "A" ? deckA : deckB;
    if (deck.stems) {
      (deck.stems as any)[stem] = active;
    }
    console.log(`[Stem ${deckId}] ${stem} = ${active}`);
  }

  // ==================== PITCH ====================
  function setPitch(deckId: DeckId, pitchVal: number) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.pitch = Number(pitchVal.toFixed(1));
    deck.bpm = Number((deck.baseBpm * (1 + deck.pitch / 100)).toFixed(1));
  }

  function nudgePitch(deckId: DeckId, delta: number) {
    const deck = deckId === "A" ? deckA : deckB;
    setPitch(deckId, Math.max(-8, Math.min(8, deck.pitch + delta)));
  }

  function resetPitch(deckId: DeckId) {
    setPitch(deckId, 0.0);
  }

  // ==================== CROSSFADER ====================
  async function setCrossfader(val: number) {
    crossfader.value = Math.max(0, Math.min(1, val));
    try {
      await invoke("set_crossfader", { val: crossfader.value });
    } catch {}
  }

  // ==================== WATCHERS ====================
  watch(
    () => [
      crossfader.value,
      masterVolume.value,
      deckA.volume, deckA.eqHigh, deckA.eqMid, deckA.eqLow,
      deckA.filter, deckA.gain, deckA.keyLock, deckA.fxType, deckA.fxParams,
      deckB.volume, deckB.eqHigh, deckB.eqMid, deckB.eqLow,
      deckB.filter, deckB.gain, deckB.keyLock, deckB.fxType, deckB.fxParams,
    ],
    scheduleSettingsSave,
    { deep: true }
  );

  // ==================== LIFECYCLE ====================
  onMounted(async () => {
    await setupListeners();
    tickerId = window.setInterval(updatePlaybackPositions, 200);

    // Try to load last drive settings
    try {
      const lastDrive = localStorage.getItem("djpro_last_drive");
      if (lastDrive) {
        await playlist.setDrive(lastDrive);
        await cues.loadCues(lastDrive);
        await loadSettingsFromDrive();
      }
    } catch (err) {
      console.warn("[Init] Could not load last drive settings");
    }
  });

  onUnmounted(async () => {
    if (unlistenChunk) unlistenChunk();
    if (unlistenComplete) unlistenComplete();
    if (unlistenReady) unlistenReady();
    if (tickerId) clearInterval(tickerId);
    if (saveTimer) clearTimeout(saveTimer);

    // Save ពេលបិទ
    await playlist.saveNow();
    await cues.saveCues();
  });

  // ✅ Save last drive
  watch(() => playlist.activeDrive.value, (newDrive) => {
    if (newDrive) {
      localStorage.setItem("djpro_last_drive", newDrive);
    }
  });

  // ==================== RETURN ====================
  return {
    // Deck states
    deckA,
    deckB,
    crossfader,
    masterVolume,

    // Load
    openTrack,
    loadDirectTrack,
    loadFileToDeck,

    // Playback
    seekDeck,
    togglePlay,
    triggerCue,
    syncDeck,
    toggleKeyLock,

    // Volume & EQ
    setVolume,
    setDeckEq,
    setDeckGain,

    // FX
    setDeckFxType,
    setDeckFxAmount,
    setDeckFxActive,
    resetDeckFx,
    updateFxParams,
    triggerFxOut,
    cancelFxOut,

    // Hot Cue / Loop / Stems
    triggerHotCue,
    toggleLoop,
    toggleStem,

    // Pitch
    setPitch,
    nudgePitch,
    resetPitch,

    // Crossfader
    setCrossfader,

    // ✅ Storage (expose សម្រាប់ components)
    playlist,
    cues,
    appSettings,
  };
}