export type DeckId = "A" | "B";

export interface WaveformPeak {
  low: number;   // < 250 Hz (Bass / Kick drum) - Normalized 0.0 to 1.0
  mid: number;   // 250 Hz - 4000 Hz (Vocals / Snares)
  high: number;  // > 4000 Hz (Hi-hats / Air)
}

export type WaveBand = WaveformPeak;

export interface DeckState {
  id: DeckId;
  isPlaying: boolean;
  volume: number;
  pitch: number;      // in percent (-8 to +8)
  bpm: number;
  baseBpm: number;
  currentTime: string;
  totalTime: string;
  title: string;
  artist: string;
  key: string;
  status: "EMPTY" | "LOADING..." | "READY" | "PLAYING" | "PAUSED" | "STOPPED" | "ERROR";
  eqHigh: number;     // -100 to 100 (0 = Center)
  eqMid: number;
  eqLow: number;
  filter: number;     // -100 (Low Pass) to +100 (High Pass)
  gain: number;       // 0 to 100
  keyLock: boolean;
  isSynced: boolean;
  cueActive: boolean;
  progress: number;        // 0.0 to 1.0 (seek progress)
  waveform: WaveBand[];    // High density multi-band slices (ភ្លេង, មេឌី, ច្រៀង)
  durationSecs: number;
}

export interface LibraryTrack {
  id: string;
  title: string;
  artist: string;
  bpm: number;
  key: string;
  duration: string;
  path: string;
}