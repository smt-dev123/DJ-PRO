import { reactive, ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { DeckId, DeckState, LibraryTrack, WaveformPeak } from "../types/dj";

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

interface WaveformCompletePayload {
  deckId: "A" | "B";
}

interface TrackReadyPayload {
  deckId: "A" | "B";
  title: string;
  duration: string;
  durationSecs: number;
}

export function useDjEngine() {
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
    gain: 50,
    keyLock: true,
    isSynced: false,
    cueActive: false,
    progress: 0,
    waveform: [],
    durationSecs: 0,
  });

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
    gain: 50,
    keyLock: true,
    isSynced: false,
    cueActive: false,
    progress: 0,
    waveform: [],
    durationSecs: 0,
  });

  const crossfader = ref(0.5);
  const masterVolume = ref(0.85);
  const isDecoding = ref<{ A: boolean; B: boolean }>({ A: false, B: false });

  // ==================== EVENT LISTENERS ====================
  let unlistenChunk: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;
  let unlistenReady: (() => void) | null = null;
  let tickerId: any = null;

  async function setupListeners() {
    // ✅ Waveform chunk → Append ភ្លាម
    unlistenChunk = await listen<WaveformChunkPayload>("waveform-chunk", (event) => {
      const { deckId, chunk, chunkIndex, totalChunks } = event.payload;
      const deck = deckId === "A" ? deckA : deckB;

      // ✅ Append chunk ទៅ waveform ដែលមាន
      deck.waveform = [...deck.waveform, ...chunk];

      console.log(
        `[Waveform] Deck ${deckId}: chunk ${chunkIndex + 1}/${totalChunks} (${deck.waveform.length} peaks)`
      );
    });

    // ✅ Waveform complete
    unlistenComplete = await listen<WaveformCompletePayload>(
      "waveform-complete",
      (event) => {
        const { deckId } = event.payload;
        const deck = deckId === "A" ? deckA : deckB;
        console.log(`[Waveform] Deck ${deckId}: ✅ COMPLETE (${deck.waveform.length} peaks)`);
      }
    );

    // ✅ Track ready
    unlistenReady = await listen<TrackReadyPayload>("track-ready", (event) => {
      const { deckId, title, duration, durationSecs } = event.payload;
      const deck = deckId === "A" ? deckA : deckB;

      deck.title = title;
      deck.totalTime = duration;
      deck.durationSecs = durationSecs;
      deck.status = "READY";
      deck.waveform = []; // ✅ Reset

      console.log(`[Track] Deck ${deckId}: ✅ READY — PLAY NOW`);
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

  onMounted(() => {
    setupListeners();
    tickerId = setInterval(updatePlaybackPositions, 200);
  });

  onUnmounted(() => {
    if (unlistenChunk) unlistenChunk();
    if (unlistenComplete) unlistenComplete();
    if (unlistenReady) unlistenReady();
    if (tickerId) clearInterval(tickerId);
  });

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
      await invoke<TrackLoadedFast>("load_track_fast", {
        deckId,
        path: track.path,
      });
      // Event "track-ready" នឹង update deck state
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
      await invoke<TrackLoadedFast>("load_track_fast", {
        deckId,
        path: filePath,
      });
      // Event "track-ready" នឹង update deck state
    } catch (err) {
      console.error(`[Deck ${deckId}] Load error:`, err);
      deck.status = "ERROR";
    } finally {
      isDecoding.value[deckId] = false;
    }
  }

  // ==================== CONTROLS ====================
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
    target.pitch = Number((((source.bpm / target.baseBpm) - 1) * 100).toFixed(1));
    target.isSynced = true;
    setTimeout(() => {
      target.isSynced = false;
    }, 1500);
  }

  async function toggleKeyLock(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.keyLock = !deck.keyLock;
  }

  async function setVolume(deckId: DeckId, vol: number) {
    const deck = deckId === "A" ? deckA : deckB;
    deck.volume = Math.max(0, Math.min(1, vol));
    try {
      await invoke("set_deck_volume", { deckId, vol: deck.volume });
    } catch {}
  }

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

  async function setCrossfader(val: number) {
    crossfader.value = Math.max(0, Math.min(1, val));
    try {
      await invoke("set_crossfader", { val: crossfader.value });
    } catch {}
  }

  return {
    deckA,
    deckB,
    crossfader,
    masterVolume,
    openTrack,
    loadDirectTrack,
    loadFileToDeck,
    seekDeck,
    togglePlay,
    triggerCue,
    syncDeck,
    toggleKeyLock,
    setVolume,
    setPitch,
    nudgePitch,
    resetPitch,
    setCrossfader,
  };
}