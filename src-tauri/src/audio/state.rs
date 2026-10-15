use crate::audio::decoder::WaveBandPeak;
use crate::audio::dsp::DeckDspState;
use crate::audio::fx::{FxParams, FxState, FxType};

// ==================== DECK ====================

#[derive(Default)]
pub struct Deck {
    // Playback
    pub samples: Vec<f32>,
    pub position: usize,
    pub is_playing: bool,
    pub volume: f32,
    pub title: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f32,

    // Waveform
    pub waveform_peaks: Vec<WaveBandPeak>,
    pub waveform_ready: bool,

    // EQ & Filter (-100 to 100)
    pub eq_high: f32,
    pub eq_mid: f32,
    pub eq_low: f32,
    pub filter: f32,

    // Gain / Trim (-100 to 100, 0 = unity)
    pub gain: f32,

    // ✅ FX Out fade state
    pub fx_out_active: bool,
    pub fx_out_progress: f32,   // 0.0 to 1.0
    pub fx_out_duration: f32,   // seconds
    pub fx_out_volume: f32,     // 1.0 → 0.0 multiplier
    pub landing_active: bool,
    pub landing_progress: f32,
    pub landing_duration: f32,
    pub landing_volume: f32,
}

impl Deck {
    /// ✅ Seek តាម ratio (0.0 to 1.0)
    pub fn seek_ratio(&mut self, ratio: f32) {
        if !self.samples.is_empty() {
            let total = self.samples.len();
            let mut new_pos = (ratio.clamp(0.0, 1.0) * total as f32) as usize;

            // Align ទៅ channel boundary
            if self.channels > 1 {
                new_pos -= new_pos % (self.channels as usize);
            }

            self.position = new_pos.min(total.saturating_sub(1));
        }
    }

    /// ✅ Progress ratio (0.0 to 1.0)
    pub fn get_progress_ratio(&self) -> f32 {
        if self.samples.is_empty() {
            0.0
        } else {
            (self.position as f32) / (self.samples.len() as f32)
        }
    }

    /// ✅ Elapsed time គិតជាវិនាទី
    pub fn get_elapsed_secs(&self) -> f32 {
        let rate = if self.sample_rate > 0 { self.sample_rate } else { 44100 };
        let ch = if self.channels > 0 { self.channels as u32 } else { 2 };
        (self.position as f32) / (rate * ch) as f32
    }

    /// ✅ Remaining time គិតជាវិនាទី
    pub fn get_remaining_secs(&self) -> f32 {
        (self.duration_secs - self.get_elapsed_secs()).max(0.0)
    }

    /// ✅ Reset playback position
    pub fn reset(&mut self) {
        self.position = 0;
        self.is_playing = false;
    }
}

// ==================== MIXER STATE ====================

pub struct MixerState {
    // Decks
    pub deck_a: Deck,
    pub deck_b: Deck,

    // Master
    pub crossfader: f32,      // 0.0 (A) → 1.0 (B)
    pub master_volume: f32,   // 0.0 to 1.0

    // ✅ DSP state (EQ/Filter)
    pub dsp_a: DeckDspState,
    pub dsp_b: DeckDspState,

    // ✅ FX state (Echo/Reverb/Delay/etc.)
    pub fx_a: FxState,
    pub fx_b: FxState,

    // ✅ FX type & control
    pub fx_type_a: FxType,
    pub fx_type_b: FxType,
    pub fx_amount_a: f32,       // 0.0 to 1.0
    pub fx_amount_b: f32,
    pub fx_active_a: bool,
    pub fx_active_b: bool,

    // ✅ FX Parameters (Echo time, Reverb decay, etc.)
    pub fx_params_a: FxParams,
    pub fx_params_b: FxParams,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            deck_a: Deck {
                volume: 0.85,
                sample_rate: 44100,
                channels: 2,
                gain: 0.0,
                eq_high: 0.0,
                eq_mid: 0.0,
                eq_low: 0.0,
                filter: 0.0,
                fx_out_volume: 1.0,
                ..Default::default()
            },
            deck_b: Deck {
                volume: 0.85,
                sample_rate: 44100,
                channels: 2,
                gain: 0.0,
                eq_high: 0.0,
                eq_mid: 0.0,
                eq_low: 0.0,
                filter: 0.0,
                fx_out_volume: 1.0,
                ..Default::default()
            },
            crossfader: 0.5,
            master_volume: 0.85,
            dsp_a: DeckDspState::default(),
            dsp_b: DeckDspState::default(),
            fx_a: FxState::default(),
            fx_b: FxState::default(),
            fx_type_a: FxType::None,
            fx_type_b: FxType::None,
            fx_amount_a: 0.8,
            fx_amount_b: 0.8,
            fx_active_a: false,
            fx_active_b: false,
            fx_params_a: FxParams::default(),
            fx_params_b: FxParams::default(),
        }
    }
}

impl MixerState {
    /// ✅ Get deck reference by ID
    pub fn get_deck(&self, deck_id: &str) -> Option<&Deck> {
        match deck_id {
            "A" => Some(&self.deck_a),
            "B" => Some(&self.deck_b),
            _ => None,
        }
    }

    /// ✅ Get mutable deck reference by ID
    pub fn get_deck_mut(&mut self, deck_id: &str) -> Option<&mut Deck> {
        match deck_id {
            "A" => Some(&mut self.deck_a),
            "B" => Some(&mut self.deck_b),
            _ => None,
        }
    }

    /// ✅ Get FX params reference
    pub fn get_fx_params(&self, deck_id: &str) -> Option<&FxParams> {
        match deck_id {
            "A" => Some(&self.fx_params_a),
            "B" => Some(&self.fx_params_b),
            _ => None,
        }
    }

    /// ✅ Get mutable FX params reference
    pub fn get_fx_params_mut(&mut self, deck_id: &str) -> Option<&mut FxParams> {
        match deck_id {
            "A" => Some(&mut self.fx_params_a),
            "B" => Some(&mut self.fx_params_b),
            _ => None,
        }
    }

    /// ✅ Reset FX សម្រាប់ deck
    pub fn reset_fx(&mut self, deck_id: &str) {
        match deck_id {
            "A" => {
                self.fx_a.reset();
                self.fx_active_a = false;
            }
            "B" => {
                self.fx_b.reset();
                self.fx_active_b = false;
            }
            _ => {}
        }
    }
}