export type DeckId = "A" | "B";

export interface WaveformPeak {
  low: number;   // < 250 Hz (Bass / Kick drum) - Normalized 0.0 to 1.0
  mid: number;   // 250 Hz - 4000 Hz (Vocals / Snares)
  high: number;  // > 4000 Hz (Hi-hats / Air)
}

export type WaveBand = WaveformPeak;

export interface DeckState {
  id: "A" | "B";
  isPlaying: boolean;
  volume: number;
  pitch: number;
  baseBpm: number;
  bpm: number;
  currentTime: string;
  totalTime: string;
  title: string;
  artist: string;
  key: string;
  status: string;
  eqHigh: number; 
  eqMid: number;  
  eqLow: number;  
  filter: number; 
  gain: number;   
  keyLock: boolean;
  isSynced: boolean;
  cueActive: boolean;
  progress: number;
  waveform: WaveformPeak[];
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