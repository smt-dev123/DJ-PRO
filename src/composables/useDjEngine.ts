import { reactive, ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { DeckId, DeckState, LibraryTrack, WaveformPeak } from "../types/dj";

interface TrackLoadedResponse {
  title: string;
  duration: string;
  duration_secs: number;
  sample_rate: number;
  waveform: WaveformPeak[];
}

function generateMockWaveform(seed: number): WaveformPeak[] {
  const points: WaveformPeak[] = [];
  const total = 1400;

  for (let i = 0; i < total; i++) {
    // Realistic DJ Track Phrasing:
    // 0 - 240: Intro (ភ្លេងបុក / Pure Beat for transition in)
    // 240 - 520: Verse (សំឡេងច្រៀងចូល / Vocals Enter)
    // 520 - 640: Build-up (Melody / Snare Roll)
    // 640 - 960: DROP! (ភ្លេងបុក / Heavy Beat & Sub-Bass Drop)
    // 960 - 1200: Chorus (Vocal Hook + Beat)
    // 1200 - 1400: Outro (ភ្លេងបុក / Beat for transition out)

    const isKick = i % 8 === 0 || i % 8 === 4;
    const isSnare = i % 8 === 4;
    const isHihat = i % 2 === 0;

    let hasVocal = false;
    let isHeavyDrop = false;

    if ((i >= 240 && i < 520) || (i >= 960 && i < 1200)) {
      hasVocal = true;
    } else if (i >= 640 && i < 960) {
      isHeavyDrop = true;
    }

    let bass: number;
    let mid: number;
    let vocalHigh: number;

    if (hasVocal) {
      // 🎤 Singing Section (សំឡេងច្រៀងចូល):
      const vocalMelody = 0.65 + Math.sin((i + seed) * 0.15) * 0.25;
      mid = Math.min(0.98, vocalMelody + (isSnare ? 0.1 : 0));
      vocalHigh = Math.min(0.85, 0.45 + Math.sin(i * 0.2) * 0.3 + (isHihat ? 0.15 : 0));
      bass = isKick ? 0.65 : 0.22;
    } else if (isHeavyDrop) {
      // 🥁 Heavy Beat Drop (ភ្លេងបុក):
      bass = isKick ? Math.min(1.0, 0.88 + Math.random() * 0.12) : 0.48;
      mid = isSnare ? 0.35 : 0.12;
      vocalHigh = isHihat ? 0.35 : 0.1;
    } else {
      // 🎶 Intro / Outro / Build-up (ភ្លេងបុក - Beat for mixing):
      bass = isKick ? 0.82 : 0.26;
      mid = isSnare ? 0.28 : 0.14;
      vocalHigh = isHihat ? 0.3 : 0.1;
    }

    points.push({
      low: Number(bass.toFixed(3)),
      mid: Number(mid.toFixed(3)),
      high: Number(vocalHigh.toFixed(3)),
    });
  }
  return points;
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
    totalTime: "04:12",
    title: "MIDNIGHT CITY",
    artist: "M83",
    key: "8A",
    status: "READY",
    eqHigh: 0,
    eqMid: 0,
    eqLow: 0,
    filter: 0,
    gain: 50,
    keyLock: true,
    isSynced: false,
    cueActive: false,
    progress: 0.25,
    waveform: generateMockWaveform(2),
    durationSecs: 252,
  });

  const deckB = reactive<DeckState>({
    id: "B",
    isPlaying: false,
    volume: 0.85,
    pitch: 0.0,
    baseBpm: 128.0,
    bpm: 128.0,
    currentTime: "00:00",
    totalTime: "05:04",
    title: "TITANIUM VIP",
    artist: "David Guetta",
    key: "4B",
    status: "READY",
    eqHigh: 0,
    eqMid: 0,
    eqLow: 0,
    filter: 0,
    gain: 50,
    keyLock: true,
    isSynced: false,
    cueActive: false,
    progress: 0.15,
    waveform: generateMockWaveform(18),
    durationSecs: 304,
  });

  const crossfader = ref(0.5);
  const masterVolume = ref(0.85);

  // Prevent duplicate concurrent decoding freeze
  const isDecoding = ref<{ A: boolean; B: boolean }>({ A: false, B: false });

  // Load via File Dialog
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

  // Load from Music Library row
  async function loadDirectTrack(deckId: DeckId, track: LibraryTrack) {
    if (isDecoding.value[deckId]) return;
    isDecoding.value[deckId] = true;

    const deck = deckId === "A" ? deckA : deckB;
    deck.status = "DECODING...";
    deck.title = track.title;
    deck.artist = track.artist || "Original Artist";
    deck.baseBpm = track.bpm;
    deck.bpm = Number((track.bpm * (1 + deck.pitch / 100)).toFixed(1));
    deck.key = track.key || "8A";
    deck.currentTime = "00:00";
    deck.totalTime = track.duration;
    deck.progress = 0.0;

    try {
      const res = await invoke<TrackLoadedResponse>("load_track", { deckId, path: track.path });
      if (res) {
        deck.title = res.title || track.title;
        deck.totalTime = res.duration;
        deck.durationSecs = res.duration_secs;
        if (res.waveform && res.waveform.length > 0) {
          deck.waveform = res.waveform;
        }
      }
      deck.status = "READY";
    } catch {
      // Simulation fallback for virtual demo tracks
      deck.durationSecs = 240;
      deck.waveform = generateMockWaveform(Math.random() * 50);
      deck.status = "READY";
    } finally {
      isDecoding.value[deckId] = false;
    }
  }

  // Load directly from File Path (Drag and drop or OS Explorer)
  async function loadFileToDeck(deckId: DeckId, filePath: string) {
    if (isDecoding.value[deckId]) return;
    isDecoding.value[deckId] = true;

    const deck = deckId === "A" ? deckA : deckB;
    deck.status = "DECODING...";
    const fileName = filePath.split(/[\\/]/).pop()?.replace(/\.[^/.]+$/, "") || "Track";
    deck.title = fileName;
    deck.artist = "Local File";
    deck.isPlaying = false;
    deck.currentTime = "00:00";
    deck.progress = 0.0;

    try {
      const res = await invoke<TrackLoadedResponse>("load_track", { deckId, path: filePath });
      if (res) {
        deck.title = res.title || fileName;
        deck.totalTime = res.duration;
        deck.durationSecs = res.duration_secs;
        if (res.waveform && res.waveform.length > 0) {
          deck.waveform = res.waveform;
        }
      }
      deck.status = "READY";
    } catch {
      deck.durationSecs = 240;
      deck.totalTime = "04:00";
      deck.waveform = generateMockWaveform(Math.random() * 50);
      deck.status = "READY";
    } finally {
      isDecoding.value[deckId] = false;
    }
  }

  // Seek / Scrub (ខារចម្រៀង)
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
    } catch {
      // fallback
    }
  }

  async function togglePlay(deckId: DeckId) {
    const deck = deckId === "A" ? deckA : deckB;
    try {
      const playing = await invoke<boolean>("toggle_play", { deckId });
      deck.isPlaying = playing;
      deck.status = playing ? "PLAYING" : "PAUSED";
    } catch {
      deck.isPlaying = !deck.isPlaying;
      deck.status = deck.isPlaying ? "PLAYING" : "PAUSED";
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

  // Real-time playback position ticker (updates progress & time smoothly)
  let tickerId: any = null;

  function updatePlaybackPositions() {
    if (deckA.isPlaying) {
      invoke<any>("get_deck_playback", { deckId: "A" })
        .then((res) => {
          if (res) {
            deckA.progress = res.progress;
            deckA.currentTime = res.current_time_str;
          }
        })
        .catch(() => {
          // simulation fallback
          if (deckA.durationSecs > 0) {
            deckA.progress = Math.min(1, deckA.progress + 0.1 / deckA.durationSecs);
            const curSecs = Math.round(deckA.progress * deckA.durationSecs);
            const m = Math.floor(curSecs / 60);
            const s = curSecs % 60;
            deckA.currentTime = `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
          }
        });
    }

    if (deckB.isPlaying) {
      invoke<any>("get_deck_playback", { deckId: "B" })
        .then((res) => {
          if (res) {
            deckB.progress = res.progress;
            deckB.currentTime = res.current_time_str;
          }
        })
        .catch(() => {
          // simulation fallback
          if (deckB.durationSecs > 0) {
            deckB.progress = Math.min(1, deckB.progress + 0.1 / deckB.durationSecs);
            const curSecs = Math.round(deckB.progress * deckB.durationSecs);
            const m = Math.floor(curSecs / 60);
            const s = curSecs % 60;
            deckB.currentTime = `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
          }
        });
    }
  }

  onMounted(() => {
    tickerId = setInterval(updatePlaybackPositions, 100);
  });

  onUnmounted(() => {
    if (tickerId) {
      clearInterval(tickerId);
    }
  });

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