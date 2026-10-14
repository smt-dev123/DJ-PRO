use crate::audio::decoder::WaveBandPeak;
use crate::audio::dsp::DeckDspState;

#[derive(Default)]
pub struct Deck {
    pub samples: Vec<f32>,
    pub position: usize,
    pub is_playing: bool,
    pub volume: f32,
    pub title: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f32,
    pub waveform_peaks: Vec<WaveBandPeak>,
    pub waveform_ready: bool,
    // EQ & Filter (-100 to 100)
    pub eq_high: f32,
    pub eq_mid: f32,
    pub eq_low: f32,
    pub filter: f32,
    pub gain: f32, // -100 to 100 (0 = unity)
}

impl Deck {
    pub fn seek_ratio(&mut self, ratio: f32) {
        if !self.samples.is_empty() {
            let total = self.samples.len();
            let mut new_pos = (ratio.clamp(0.0, 1.0) * total as f32) as usize;
            if self.channels > 1 {
                new_pos -= new_pos % (self.channels as usize);
            }
            self.position = new_pos.min(total.saturating_sub(1));
        }
    }

    pub fn get_progress_ratio(&self) -> f32 {
        if self.samples.is_empty() {
            0.0
        } else {
            (self.position as f32) / (self.samples.len() as f32)
        }
    }

    pub fn get_elapsed_secs(&self) -> f32 {
        let rate = if self.sample_rate > 0 { self.sample_rate } else { 44100 };
        let ch = if self.channels > 0 { self.channels as u32 } else { 2 };
        (self.position as f32) / (rate * ch) as f32
    }
}

pub struct MixerState {
    pub deck_a: Deck,
    pub deck_b: Deck,
    pub crossfader: f32,
    pub master_volume: f32,
    pub dsp_a: DeckDspState,
    pub dsp_b: DeckDspState,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            deck_a: Deck {
                volume: 0.85,
                sample_rate: 44100,
                channels: 2,
                ..Default::default()
            },
            deck_b: Deck {
                volume: 0.85,
                sample_rate: 44100,
                channels: 2,
                ..Default::default()
            },
            crossfader: 0.5,
            master_volume: 0.85,
            dsp_a: DeckDspState::default(),
            dsp_b: DeckDspState::default(),
        }
    }
}