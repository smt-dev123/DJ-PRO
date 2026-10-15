export type DeckId = "A" | "B";

export interface WaveformPeak {
  low: number; // < 250 Hz (Bass / Kick drum) - Normalized 0.0 to 1.0
  mid: number; // 250 Hz - 4000 Hz (Vocals / Snares)
  high: number; // > 4000 Hz (Hi-hats / Air)
}

export type WaveBand = WaveformPeak;

// ==================== FX PARAMS ====================
export interface FxParams {
  // Echo
  echo_time_beats: number;
  echo_feedback: number;
  echo_mix: number;
  echo_bpm: number;

  // Reverb
  reverb_room: "room" | "hall" | "plate" | "cathedral" | "chamber";
  reverb_decay: number;
  reverb_damping: number;
  reverb_mix: number;

  // Delay 
  delay_time_beats: number;
  delay_feedback: number;
  delay_mix: number;
  delay_bpm: number;

  // Flanger
  flanger_rate: number;
  flanger_depth: number;
  flanger_feedback: number;

  // Filter
  filter_resonance: number;
  filter_mode: "lowpass" | "highpass" | "bandpass";

  // Roll
  roll_beats: number;
  roll_bpm: number;

  // Landing
  landing_enabled: boolean;
  landing_duration: number;
}

// ==================== DECK STATE ====================
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

  // ✅ FX fields — ត្រូវមានទាំងអស់
  fxType: "none" | "echo" | "reverb" | "delay" | "flanger" | "filter" | "roll";
  fxAmount: number;
  fxActive: boolean;
  fxParams: FxParams;  // ✅ បន្ថែមបន្ទាត់នេះ!

  // Optional
  activeLoop?: string | null;
  stems?: {
    vocal: boolean;
    drum: boolean;
    bass: boolean;
    melody: boolean;
  };
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

export interface LandingSettings {
  duration: number;
  amount: number;
  beats?: number; 
}

export interface DeckLandingSettings {
  echo: LandingSettings;
  reverb: LandingSettings;
  delay: LandingSettings;
  roll: LandingSettings; 
}