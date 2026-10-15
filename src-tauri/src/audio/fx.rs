// ==================== FX TYPES ====================

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum FxType {
    #[default]
    None,
    Echo,
    Reverb,
    Delay,
    Flanger,
    Filter,
    Roll,
}

// ==================== FX PARAMETERS ====================

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FxParams {
    // ✅ Echo/Delay
    pub echo_time_beats: f32,      // 0.25, 0.375, 0.5, 0.75, 1.0 (beats)
    pub echo_feedback: f32,        // 0.0 - 0.95
    pub echo_mix: f32,             // 0.0 - 1.0
    pub echo_bpm: f32,             // សម្រាប់ convert beats → ms

    // ✅ Reverb
    pub reverb_decay: f32,         // 0.0 - 1.0 (room size)
    pub reverb_damping: f32,       // 0.0 - 1.0 (high-freq damping)
    pub reverb_mix: f32,           // 0.0 - 1.0

    // ✅ Delay
    pub delay_time_ms: f32,        // 50 - 1000 ms
    pub delay_feedback: f32,       // 0.0 - 0.95
    pub delay_mix: f32,            // 0.0 - 1.0

    // ✅ Flanger
    pub flanger_rate: f32,         // 0.1 - 5.0 Hz
    pub flanger_depth: f32,        // 0.0 - 1.0
    pub flanger_feedback: f32,     // 0.0 - 0.9

    // ✅ Filter
    pub filter_resonance: f32,     // 0.1 - 10.0 (Q)
    pub filter_mode: FilterMode,   // LP / HP / BP

    // ✅ Roll
    pub roll_beats: f32,           // 0.125, 0.25, 0.5, 1.0 (beats)
    pub roll_bpm: f32,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum FilterMode {
    #[default]
    LowPass,
    HighPass,
    BandPass,
}

impl Default for FxParams {
    fn default() -> Self {
        Self {
            echo_time_beats: 0.375,   // 3/8 beat
            echo_feedback: 0.6,
            echo_mix: 0.8,
            echo_bpm: 128.0,

            reverb_decay: 0.7,
            reverb_damping: 0.5,
            reverb_mix: 0.7,

            delay_time_ms: 100.0,
            delay_feedback: 0.3,
            delay_mix: 0.5,

            flanger_rate: 0.5,
            flanger_depth: 0.7,
            flanger_feedback: 0.5,

            filter_resonance: 0.7,
            filter_mode: FilterMode::LowPass,

            roll_beats: 0.25,
            roll_bpm: 128.0,
        }
    }
}

impl FxParams {
    /// ✅ Convert beats → samples សម្រាប់ Echo
    pub fn echo_delay_samples(&self, sample_rate: f32) -> usize {
        let secs_per_beat = 60.0 / self.echo_bpm;
        let delay_secs = self.echo_time_beats * secs_per_beat;
        (delay_secs * sample_rate) as usize
    }

    /// ✅ Convert ms → samples សម្រាប់ Delay
    pub fn delay_time_samples(&self, sample_rate: f32) -> usize {
        (self.delay_time_ms / 1000.0 * sample_rate) as usize
    }

    /// ✅ Convert beats → samples សម្រាប់ Roll
    pub fn roll_length_samples(&self, sample_rate: f32) -> usize {
        let secs_per_beat = 60.0 / self.roll_bpm;
        let roll_secs = self.roll_beats * secs_per_beat;
        (roll_secs * sample_rate) as usize
    }
}

// ==================== FX STATE ====================

#[derive(Default)]
pub struct FxState {
    pub fx_type: FxType,
    pub fx_amount: f32,
    pub fx_active: bool,
    pub params: FxParams,

    // Echo / Delay
    pub delay_buffer_l: Vec<f32>,
    pub delay_buffer_r: Vec<f32>,
    pub delay_write_pos: usize,
    pub delay_length_samples: usize,

    // Reverb
    pub reverb_buffer: Vec<f32>,
    pub reverb_write_pos: usize,

    // Flanger
    pub flanger_phase: f32,

    // Filter
    pub filter_state_l: f32,
    pub filter_state_r: f32,

    // Roll
    pub roll_buffer_l: Vec<f32>,
    pub roll_buffer_r: Vec<f32>,
    pub roll_write_pos: usize,
    pub roll_length_samples: usize,
    pub roll_read_pos: usize,
}

impl FxState {
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate as usize;
        Self {
            fx_type: FxType::None,
            fx_amount: 0.0,
            fx_active: false,
            params: FxParams::default(),

            delay_buffer_l: vec![0.0; sr],
            delay_buffer_r: vec![0.0; sr],
            delay_write_pos: 0,
            delay_length_samples: sr,

            reverb_buffer: vec![0.0; sr / 2],
            reverb_write_pos: 0,

            flanger_phase: 0.0,
            filter_state_l: 0.0,
            filter_state_r: 0.0,

            roll_buffer_l: vec![0.0; sr / 8],
            roll_buffer_r: vec![0.0; sr / 8],
            roll_write_pos: 0,
            roll_length_samples: sr / 8,
            roll_read_pos: 0,
        }
    }

    pub fn reset(&mut self) {
        self.delay_buffer_l.fill(0.0);
        self.delay_buffer_r.fill(0.0);
        self.reverb_buffer.fill(0.0);
        self.roll_buffer_l.fill(0.0);
        self.roll_buffer_r.fill(0.0);
        self.delay_write_pos = 0;
        self.reverb_write_pos = 0;
        self.roll_write_pos = 0;
        self.roll_read_pos = 0;
        self.flanger_phase = 0.0;
        self.filter_state_l = 0.0;
        self.filter_state_r = 0.0;
    }

    /// ✅ Resize roll buffer បើ roll_beats ផ្លាស់ប្តូរ
    pub fn resize_roll_buffer(&mut self, sample_rate: f32) {
        let new_len = self.params.roll_length_samples(sample_rate).max(64);
        if self.roll_buffer_l.len() != new_len {
            self.roll_buffer_l = vec![0.0; new_len];
            self.roll_buffer_r = vec![0.0; new_len];
            self.roll_length_samples = new_len;
            self.roll_write_pos = 0;
            self.roll_read_pos = 0;
        }
    }
}

// ==================== APPLY FX ====================

pub fn apply_fx(fx: &mut FxState, x: f32, channel: usize, sample_rate: f32) -> f32 {
    if !fx.fx_active || fx.fx_type == FxType::None {
        return x;
    }

    // Lazy init
    if fx.delay_buffer_l.is_empty() {
        let sr = sample_rate as usize;
        fx.delay_buffer_l = vec![0.0; sr];
        fx.delay_buffer_r = vec![0.0; sr];
        fx.delay_length_samples = sr;
        fx.reverb_buffer = vec![0.0; sr / 2];
    }

    let amount = fx.fx_amount.clamp(0.0, 1.0);
    let params = fx.params;

    match fx.fx_type {
        FxType::None => x,

        // ==================== ECHO ====================
        FxType::Echo => {
            let delay_samples = params
                .echo_delay_samples(sample_rate)
                .min(fx.delay_length_samples - 1);

            let read_pos = if fx.delay_write_pos >= delay_samples {
                fx.delay_write_pos - delay_samples
            } else {
                fx.delay_length_samples + fx.delay_write_pos - delay_samples
            };

            let buf = if channel == 0 {
                &mut fx.delay_buffer_l
            } else {
                &mut fx.delay_buffer_r
            };

            let delayed = buf[read_pos];
            let feedback = params.echo_feedback.clamp(0.0, 0.95);
            buf[fx.delay_write_pos] = x + delayed * feedback;

            if channel == 0 {
                fx.delay_write_pos = (fx.delay_write_pos + 1) % fx.delay_length_samples;
            }

            let mix = params.echo_mix * amount;
            x * (1.0 - mix) + delayed * mix
        }

        // ==================== REVERB ====================
        FxType::Reverb => {
            let buf_len = fx.reverb_buffer.len();
            if buf_len == 0 {
                return x;
            }

            // ✅ Decay ប៉ះពាល់ feedback
            let feedback = 0.3 + params.reverb_decay * 0.65;

            // ✅ Damping ប៉ះពាល់ low-pass ក្នុង feedback
            let damping = params.reverb_damping.clamp(0.0, 1.0);

            // Multi-tap delay for simple reverb
            let tap1 = (buf_len / 4) % buf_len;
            let tap2 = (buf_len / 3) % buf_len;
            let tap3 = (buf_len / 2) % buf_len;

            let read1 = (fx.reverb_write_pos + buf_len - tap1) % buf_len;
            let read2 = (fx.reverb_write_pos + buf_len - tap2) % buf_len;
            let read3 = (fx.reverb_write_pos + buf_len - tap3) % buf_len;

            let delayed = (fx.reverb_buffer[read1] + fx.reverb_buffer[read2] * 0.7
                + fx.reverb_buffer[read3] * 0.5)
                / 2.2;

            // ✅ Apply damping (simple low-pass)
            let damped = delayed * (1.0 - damping * 0.5);
            fx.reverb_buffer[fx.reverb_write_pos] = x + damped * feedback;
            fx.reverb_write_pos = (fx.reverb_write_pos + 1) % buf_len;

            let mix = params.reverb_mix * amount;
            x * (1.0 - mix) + delayed * mix
        }

        // ==================== DELAY ====================
        FxType::Delay => {
            let delay_samples = params
                .delay_time_samples(sample_rate)
                .min(fx.delay_length_samples - 1);

            let read_pos = if fx.delay_write_pos >= delay_samples {
                fx.delay_write_pos - delay_samples
            } else {
                fx.delay_length_samples + fx.delay_write_pos - delay_samples
            };

            let buf = if channel == 0 {
                &mut fx.delay_buffer_l
            } else {
                &mut fx.delay_buffer_r
            };

            let delayed = buf[read_pos];
            let feedback = params.delay_feedback.clamp(0.0, 0.95);
            buf[fx.delay_write_pos] = x + delayed * feedback;

            if channel == 0 {
                fx.delay_write_pos = (fx.delay_write_pos + 1) % fx.delay_length_samples;
            }

            let mix = params.delay_mix * amount;
            x * (1.0 - mix) + delayed * mix
        }

        // ==================== FLANGER ====================
        FxType::Flanger => {
            let lfo_rate = params.flanger_rate.clamp(0.1, 5.0);
            fx.flanger_phase += lfo_rate / sample_rate;
            if fx.flanger_phase > 1.0 {
                fx.flanger_phase -= 1.0;
            }

            let lfo = (fx.flanger_phase * 2.0 * std::f32::consts::PI).sin();
            let depth = params.flanger_depth.clamp(0.0, 1.0);
            let delay_ms = 1.0 + (lfo + 1.0) * 2.0 * depth;
            let delay_samples = ((sample_rate * delay_ms / 1000.0) as usize)
                .min(fx.delay_length_samples - 1)
                .max(1);

            let read_pos = if fx.delay_write_pos >= delay_samples {
                fx.delay_write_pos - delay_samples
            } else {
                fx.delay_length_samples + fx.delay_write_pos - delay_samples
            };

            let buf = if channel == 0 {
                &mut fx.delay_buffer_l
            } else {
                &mut fx.delay_buffer_r
            };

            let delayed = buf[read_pos];
            let fb = params.flanger_feedback.clamp(0.0, 0.9);
            buf[fx.delay_write_pos] = x + delayed * fb;

            if channel == 0 {
                fx.delay_write_pos = (fx.delay_write_pos + 1) % fx.delay_length_samples;
            }

            x + delayed * amount * 0.7
        }

        // ==================== FILTER ====================
        FxType::Filter => {
            let resonance = params.filter_resonance.clamp(0.1, 10.0);
            let alpha = (0.1 + amount * 0.85) / resonance;
            let state = if channel == 0 {
                &mut fx.filter_state_l
            } else {
                &mut fx.filter_state_r
            };

            *state = *state + alpha * (x - *state);

            match params.filter_mode {
                FilterMode::LowPass => x * (1.0 - amount * 0.8) + *state * amount * 0.8,
                FilterMode::HighPass => x - *state * amount * 0.8,
                FilterMode::BandPass => (*state - x * 0.5) * amount,
            }
        }

        // ==================== ROLL ====================
        FxType::Roll => {
            fx.resize_roll_buffer(sample_rate);

            let buf = if channel == 0 {
                &mut fx.roll_buffer_l
            } else {
                &mut fx.roll_buffer_r
            };

            if buf.is_empty() {
                return x;
            }

            buf[fx.roll_write_pos] = x;

            if channel == 0 {
                fx.roll_write_pos = (fx.roll_write_pos + 1) % fx.roll_length_samples;
                fx.roll_read_pos = (fx.roll_read_pos + 1) % fx.roll_length_samples;
            }

            buf[fx.roll_read_pos]
        }
    }
}