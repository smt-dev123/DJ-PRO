use serde::{Deserialize, Serialize};

// ==================== FX TYPE ====================

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
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

// ==================== REVERB ROOM TYPE ====================

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum ReverbRoom {
    #[default]
    Room, // Small room — decay 0.5s
    Hall,      // Large hall — decay 2.5s
    Plate,     // Plate reverb — decay 1.5s
    Cathedral, // Very large — decay 5s
    Chamber,   // Mid-size — decay 1.2s
}

impl ReverbRoom {
    pub fn decay_secs(self) -> f32 {
        match self {
            ReverbRoom::Room => 0.5,
            ReverbRoom::Hall => 2.5,
            ReverbRoom::Plate => 1.5,
            ReverbRoom::Cathedral => 5.0,
            ReverbRoom::Chamber => 1.2,
        }
    }

    pub fn damping(self) -> f32 {
        match self {
            ReverbRoom::Room => 0.7,      // Very damped
            ReverbRoom::Hall => 0.4,      // Bright
            ReverbRoom::Plate => 0.2,     // Very bright
            ReverbRoom::Cathedral => 0.3, // Bright
            ReverbRoom::Chamber => 0.5,   // Mid
        }
    }
}

// ==================== TIME DIVISION ====================

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum TimeDivision {
    Div1_16, // 0.0625 beats
    Div1_8,  // 0.125
    Div1_4,  // 0.25
    Div3_8,  // 0.375
    Div1_2,  // 0.5
    Div3_4,  // 0.75
    Div1,    // 1.0
    Div1_5,  // 1.5
    Div2,    // 2.0
    Div3,    // 3.0
    Div4,    // 4.0
}

impl TimeDivision {
    pub fn beats(self) -> f32 {
        match self {
            TimeDivision::Div1_16 => 0.0625,
            TimeDivision::Div1_8 => 0.125,
            TimeDivision::Div1_4 => 0.25,
            TimeDivision::Div3_8 => 0.375,
            TimeDivision::Div1_2 => 0.5,
            TimeDivision::Div3_4 => 0.75,
            TimeDivision::Div1 => 1.0,
            TimeDivision::Div1_5 => 1.5,
            TimeDivision::Div2 => 2.0,
            TimeDivision::Div3 => 3.0,
            TimeDivision::Div4 => 4.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TimeDivision::Div1_16 => "1/16",
            TimeDivision::Div1_8 => "1/8",
            TimeDivision::Div1_4 => "1/4",
            TimeDivision::Div3_8 => "3/8",
            TimeDivision::Div1_2 => "1/2",
            TimeDivision::Div3_4 => "3/4",
            TimeDivision::Div1 => "1",
            TimeDivision::Div1_5 => "1½",
            TimeDivision::Div2 => "2",
            TimeDivision::Div3 => "3",
            TimeDivision::Div4 => "4",
        }
    }

    pub fn from_beats(beats: f32) -> Self {
        match beats {
            b if (b - 0.0625).abs() < 0.01 => TimeDivision::Div1_16,
            b if (b - 0.125).abs() < 0.01 => TimeDivision::Div1_8,
            b if (b - 0.25).abs() < 0.01 => TimeDivision::Div1_4,
            b if (b - 0.375).abs() < 0.01 => TimeDivision::Div3_8,
            b if (b - 0.5).abs() < 0.01 => TimeDivision::Div1_2,
            b if (b - 0.75).abs() < 0.01 => TimeDivision::Div3_4,
            b if (b - 1.0).abs() < 0.01 => TimeDivision::Div1,
            b if (b - 1.5).abs() < 0.01 => TimeDivision::Div1_5,
            b if (b - 2.0).abs() < 0.01 => TimeDivision::Div2,
            b if (b - 3.0).abs() < 0.01 => TimeDivision::Div3,
            _ => TimeDivision::Div4,
        }
    }
}

// ==================== FX PARAMS ====================

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FxParams {
    // Echo
    pub echo_time_beats: f32,
    pub echo_feedback: f32,
    pub echo_mix: f32,
    pub echo_bpm: f32,

    // Reverb
    pub reverb_room: ReverbRoom,
    pub reverb_decay: f32,
    pub reverb_damping: f32,
    pub reverb_mix: f32,

    // Delay
    pub delay_time_beats: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub delay_bpm: f32,

    // Flanger
    pub flanger_rate: f32,
    pub flanger_depth: f32,
    pub flanger_feedback: f32,

    // Filter
    pub filter_resonance: f32,
    pub filter_mode: FilterMode,

    // Roll
    pub roll_beats: f32,
    pub roll_bpm: f32,

    // Landing
    pub landing_enabled: bool,
    pub landing_duration: f32, // seconds
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
            echo_time_beats: 0.5,
            echo_feedback: 0.6,
            echo_mix: 0.8,
            echo_bpm: 128.0,

            reverb_room: ReverbRoom::Hall,
            reverb_decay: 0.7,
            reverb_damping: 0.5,
            reverb_mix: 0.7,

            delay_time_beats: 0.25,
            delay_feedback: 0.3,
            delay_mix: 0.5,
            delay_bpm: 128.0,

            flanger_rate: 0.5,
            flanger_depth: 0.7,
            flanger_feedback: 0.5,

            filter_resonance: 0.7,
            filter_mode: FilterMode::LowPass,

            roll_beats: 0.25,
            roll_bpm: 128.0,

            landing_enabled: true,
            landing_duration: 2.5,
        }
    }
}

impl FxParams {
    pub fn echo_delay_samples(&self, sample_rate: f32) -> usize {
        let secs_per_beat = 60.0 / self.echo_bpm;
        let delay_secs = self.echo_time_beats * secs_per_beat;
        (delay_secs * sample_rate) as usize
    }

    pub fn delay_time_samples(&self, sample_rate: f32) -> usize {
        let secs_per_beat = 60.0 / self.delay_bpm;
        let delay_secs = self.delay_time_beats * secs_per_beat;
        (delay_secs * sample_rate) as usize
    }

    pub fn roll_length_samples(&self, sample_rate: f32) -> usize {
        let secs_per_beat = 60.0 / self.roll_bpm;
        let roll_secs = self.roll_beats * secs_per_beat;
        (roll_secs * sample_rate) as usize
    }

    pub fn reverb_decay_samples(&self, sample_rate: f32) -> usize {
        let decay = self.reverb_room.decay_secs() * self.reverb_decay.max(0.1);
        (decay * sample_rate) as usize
    }
}

// ==================== SCHROEDER / FREEVERB DSP COMPONENTS ====================

#[derive(Clone, Default)]
pub struct CombFilter {
    buffer: Vec<f32>,
    buf_size: usize,
    buf_idx: usize,
    filter_store: f32,
}

impl CombFilter {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buf_size: size,
            buf_idx: 0,
            filter_store: 0.0,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32, feedback: f32, damp: f32) -> f32 {
        if self.buf_size == 0 {
            return input;
        }
        let output = self.buffer[self.buf_idx];

        // Lowpass damping in feedback loop
        self.filter_store = output * (1.0 - damp) + self.filter_store * damp;
        if self.filter_store.abs() < 1e-15 {
            self.filter_store = 0.0; // Denormal suppression
        }

        self.buffer[self.buf_idx] = input + self.filter_store * feedback;

        self.buf_idx += 1;
        if self.buf_idx >= self.buf_size {
            self.buf_idx = 0;
        }
        output
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.filter_store = 0.0;
        self.buf_idx = 0;
    }
}

#[derive(Clone, Default)]
pub struct AllpassFilter {
    buffer: Vec<f32>,
    buf_size: usize,
    buf_idx: usize,
    feedback: f32,
}

impl AllpassFilter {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buf_size: size,
            buf_idx: 0,
            feedback: 0.5,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        if self.buf_size == 0 {
            return input;
        }
        let buf_out = self.buffer[self.buf_idx];
        let output = -input + buf_out;
        self.buffer[self.buf_idx] = input + buf_out * self.feedback;

        self.buf_idx += 1;
        if self.buf_idx >= self.buf_size {
            self.buf_idx = 0;
        }
        output
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.buf_idx = 0;
    }
}

const COMB_TUNINGS_L: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNINGS_L: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD: usize = 23;

#[derive(Clone, Default)]
pub struct FreeverbChannel {
    pub combs: Vec<CombFilter>,
    pub allpasses: Vec<AllpassFilter>,
    pub sample_rate: f32,
}

impl FreeverbChannel {
    pub fn init(&mut self, is_right: bool, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() < 1.0 && !self.combs.is_empty() {
            return;
        }
        self.sample_rate = sample_rate;
        let scale = (sample_rate / 44100.0).max(0.5);
        let offset = if is_right { STEREO_SPREAD } else { 0 };

        self.combs = COMB_TUNINGS_L
            .iter()
            .map(|&d| {
                let size = (((d + offset) as f32 * scale).round() as usize).max(16);
                CombFilter::new(size)
            })
            .collect();

        self.allpasses = ALLPASS_TUNINGS_L
            .iter()
            .map(|&d| {
                let size = (((d + offset) as f32 * scale).round() as usize).max(16);
                AllpassFilter::new(size)
            })
            .collect();
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32, feedback: f32, damp: f32) -> f32 {
        let input_scaled = input * 0.015;
        let mut out = 0.0;
        for comb in &mut self.combs {
            out += comb.process(input_scaled, feedback, damp);
        }
        for allpass in &mut self.allpasses {
            out = allpass.process(out);
        }
        out
    }

    pub fn reset(&mut self) {
        for comb in &mut self.combs {
            comb.reset();
        }
        for allpass in &mut self.allpasses {
            allpass.reset();
        }
    }
}

// ==================== FX STATE ====================

#[derive(Default)]
pub struct FxState {
    pub fx_type: FxType,
    pub fx_amount: f32,
    pub fx_active: bool,
    pub params: FxParams,

    // Echo / Delay / Flanger buffers
    pub delay_buffer_l: Vec<f32>,
    pub delay_buffer_r: Vec<f32>,
    pub delay_write_pos: usize,
    pub delay_length_samples: usize,

    // Reverb Engine (Freeverb / Schroeder multi-comb & allpass diffusion)
    pub reverb_l: FreeverbChannel,
    pub reverb_r: FreeverbChannel,

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
    pub fn reset(&mut self) {
        self.delay_buffer_l.fill(0.0);
        self.delay_buffer_r.fill(0.0);
        self.roll_buffer_l.fill(0.0);
        self.roll_buffer_r.fill(0.0);
        self.delay_write_pos = 0;
        self.roll_write_pos = 0;
        self.roll_read_pos = 0;
        self.flanger_phase = 0.0;
        self.filter_state_l = 0.0;
        self.filter_state_r = 0.0;
        self.reverb_l.reset();
        self.reverb_r.reset();
    }

    fn ensure_buffers(&mut self, sample_rate: f32) {
        let sr = sample_rate as usize;
        let max_delay = sr * 4;

        if self.delay_buffer_l.len() < max_delay {
            self.delay_buffer_l.resize(max_delay, 0.0);
            self.delay_buffer_r.resize(max_delay, 0.0);
        }
        if self.roll_buffer_l.len() < sr {
            self.roll_buffer_l.resize(sr, 0.0);
            self.roll_buffer_r.resize(sr, 0.0);
        }

        self.delay_length_samples = max_delay;
        self.roll_length_samples = self.params.roll_length_samples(sample_rate).max(64);

        self.reverb_l.init(false, sample_rate);
        self.reverb_r.init(true, sample_rate);
    }
}

// ==================== APPLY FX ====================

pub fn apply_fx(fx: &mut FxState, x: f32, channel: usize, sample_rate: f32) -> f32 {
    if !fx.fx_active || fx.fx_type == FxType::None {
        return x;
    }

    fx.ensure_buffers(sample_rate);

    let amount = fx.fx_amount.clamp(0.0, 1.0);
    let params = fx.params;

    match fx.fx_type {
        FxType::None => x,

        // ==================== ECHO ====================
        FxType::Echo => {
            let delay_samples = params
                .echo_delay_samples(sample_rate)
                .min(fx.delay_length_samples - 1)
                .max(1);

            let read_pos = (fx.delay_write_pos + fx.delay_length_samples - delay_samples)
                % fx.delay_length_samples;

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

        // ==================== REVERB (True Algorithmic Schroeder/Freeverb Reverb) ====================
        FxType::Reverb => {
            // Room acoustic parameters: (feedback / room decay, high-frequency damping)
            let (base_feedback, base_damp) = match params.reverb_room {
                ReverbRoom::Room => (0.72, 0.45),
                ReverbRoom::Chamber => (0.78, 0.35),
                ReverbRoom::Plate => (0.84, 0.12),
                ReverbRoom::Hall => (0.89, 0.25),
                ReverbRoom::Cathedral => (0.94, 0.18),
            };

            let decay_adj = (params.reverb_decay.clamp(0.0, 1.0) - 0.7) * 0.15;
            let fb = (base_feedback + decay_adj).clamp(0.55, 0.97);

            let damp_adj = params.reverb_damping.clamp(0.05, 1.0) / 0.5;
            let dm = (base_damp * damp_adj).clamp(0.02, 0.75);

            let wet_sample = if channel == 0 {
                fx.reverb_l.process(x, fb, dm)
            } else {
                fx.reverb_r.process(x, fb, dm)
            };

            let wet_mix = params.reverb_mix * amount;
            // Maintain crisp, clear dry audio punch without attenuation, while adding lush ambient reverb tail
            let dry_gain = (1.0 - wet_mix * 0.15).max(0.0);
            let wet_gain = wet_mix * 1.5;

            x * dry_gain + wet_sample * wet_gain
        }

        // ==================== DELAY ====================
        FxType::Delay => {
            let delay_samples = params
                .delay_time_samples(sample_rate)
                .min(fx.delay_length_samples - 1)
                .max(1);

            let read_pos = (fx.delay_write_pos + fx.delay_length_samples - delay_samples)
                % fx.delay_length_samples;

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

            let read_pos = (fx.delay_write_pos + fx.delay_length_samples - delay_samples)
                % fx.delay_length_samples;

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
            let roll_len = fx.params.roll_length_samples(sample_rate).max(64);
            if fx.roll_buffer_l.len() != roll_len {
                fx.roll_buffer_l.resize(roll_len, 0.0);
                fx.roll_buffer_r.resize(roll_len, 0.0);
                fx.roll_length_samples = roll_len;
                fx.roll_write_pos = 0;
                fx.roll_read_pos = 0;
            }

            let buf = if channel == 0 {
                &mut fx.roll_buffer_l
            } else {
                &mut fx.roll_buffer_r
            };

            if amount < 0.2 {
                // Continuously capture audio stream into circular buffer
                buf[fx.roll_write_pos] = x;
                if channel == 0 {
                    fx.roll_write_pos = (fx.roll_write_pos + 1) % fx.roll_length_samples;
                    fx.roll_read_pos = fx.roll_write_pos;
                }
                x
            } else {
                // Loop active: loop captured slice and crossfade
                let looped = buf[fx.roll_read_pos];
                if channel == 0 {
                    fx.roll_read_pos = (fx.roll_read_pos + 1) % fx.roll_length_samples;
                }
                x * (1.0 - amount) + looped * amount
            }
        }
    }
}
