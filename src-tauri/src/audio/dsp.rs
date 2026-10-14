// ==================== BIQUAD FILTER STATE ====================

#[derive(Clone, Copy, Default)]
pub struct BiquadState {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
}

// ==================== DSP STATE PER DECK ====================

pub struct DeckDspState {
    pub low_l: BiquadState,
    pub low_r: BiquadState,
    pub mid_l: BiquadState,
    pub mid_r: BiquadState,
    pub high_l: BiquadState,
    pub high_r: BiquadState,
    pub filter_l: BiquadState,
    pub filter_r: BiquadState,
}

impl Default for DeckDspState {
    fn default() -> Self {
        Self {
            low_l: BiquadState::default(),
            low_r: BiquadState::default(),
            mid_l: BiquadState::default(),
            mid_r: BiquadState::default(),
            high_l: BiquadState::default(),
            high_r: BiquadState::default(),
            filter_l: BiquadState::default(),
            filter_r: BiquadState::default(),
        }
    }
}

// ==================== BIQUAD PROCESSING ====================

/// ✅ Process មួយ sample តាម biquad filter
fn biquad_process(
    state: &mut BiquadState,
    x: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
) -> f32 {
    let y = b0 * x + b1 * state.x1 + b2 * state.x2 - a1 * state.y1 - a2 * state.y2;
    state.x2 = state.x1;
    state.x1 = x;
    state.y2 = state.y1;
    state.y1 = y;
    y
}

// ==================== FILTER COEFFICIENTS ====================

/// Low shelf filter coefficients (RBJ cookbook)
pub fn low_shelf_coeffs(gain_db: f32, fc: f32, fs: f32) -> (f32, f32, f32, f32, f32) {
    let a = 10.0f32.powf(gain_db / 40.0);
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let sqrt_a = a.sqrt();

    // S = 1.0 (shelf slope)
    let alpha = sin_w0 / 2.0 * ((a + 1.0 / a) * (1.0 / 1.0 - 1.0) + 2.0).sqrt();
    let beta = 2.0 * sqrt_a * alpha;

    let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + beta);
    let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
    let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - beta);
    let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + beta;
    let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
    let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - beta;

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}

/// Peaking EQ filter coefficients
pub fn peaking_coeffs(gain_db: f32, fc: f32, q: f32, fs: f32) -> (f32, f32, f32, f32, f32) {
    let a = 10.0f32.powf(gain_db / 40.0);
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * cos_w0;
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha / a;

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}

/// High shelf filter coefficients
pub fn high_shelf_coeffs(gain_db: f32, fc: f32, fs: f32) -> (f32, f32, f32, f32, f32) {
    let a = 10.0f32.powf(gain_db / 40.0);
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let sqrt_a = a.sqrt();

    let alpha = sin_w0 / 2.0 * ((a + 1.0 / a) * (1.0 / 1.0 - 1.0) + 2.0).sqrt();
    let beta = 2.0 * sqrt_a * alpha;

    let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + beta);
    let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
    let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - beta);
    let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + beta;
    let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
    let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - beta;

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}

/// High-pass filter coefficients
pub fn highpass_coeffs(fc: f32, q: f32, fs: f32) -> (f32, f32, f32, f32, f32) {
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let b0 = (1.0 + cos_w0) / 2.0;
    let b1 = -(1.0 + cos_w0);
    let b2 = (1.0 + cos_w0) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha;

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}

/// Low-pass filter coefficients
pub fn lowpass_coeffs(fc: f32, q: f32, fs: f32) -> (f32, f32, f32, f32, f32) {
    let w0 = 2.0 * std::f32::consts::PI * fc / fs;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let b0 = (1.0 - cos_w0) / 2.0;
    let b1 = 1.0 - cos_w0;
    let b2 = (1.0 - cos_w0) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha;

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}

// ==================== EQ PRESET ====================

pub struct EqParams {
    pub gain_db: f32,
    pub eq_high_db: f32,
    pub eq_mid_db: f32,
    pub eq_low_db: f32,
    pub filter_val: f32,
}

impl EqParams {
    /// ✅ បម្លែងពី -100..100 ទៅ dB និង filter values
    pub fn from_deck(
        gain: f32,
        eq_high: f32,
        eq_mid: f32,
        eq_low: f32,
        filter: f32,
    ) -> Self {
        Self {
            gain_db: (gain / 100.0) * 25.0,       // -25dB to +25dB
            eq_high_db: (eq_high / 100.0) * 25.0,
            eq_mid_db: (eq_mid / 100.0) * 25.0,
            eq_low_db: (eq_low / 100.0) * 25.0,
            filter_val: filter,
        }
    }

    pub fn gain_linear(&self) -> f32 {
        10.0f32.powf(self.gain_db / 20.0)
    }
}

// ==================== APPLY EQ ====================

/// ✅ Apply EQ + Filter + Gain ទៅ sample មួយ
pub fn apply_eq(
    dsp: &mut DeckDspState,
    params: &EqParams,
    x: f32,
    channel: usize,
    sample_rate: f32,
) -> f32 {
    let fs = sample_rate;

    // Compute coefficients
    let (lb0, lb1, lb2, la1, la2) = low_shelf_coeffs(params.eq_low_db, 200.0, fs);
    let (mb0, mb1, mb2, ma1, ma2) = peaking_coeffs(params.eq_mid_db, 1000.0, 1.0, fs);
    let (hb0, hb1, hb2, ha1, ha2) = high_shelf_coeffs(params.eq_high_db, 4000.0, fs);

    // Filter (HP/LP)
    let filter_val = params.filter_val;
    let (fb0, fb1, fb2, fa1, fa2) = if filter_val < -5.0 {
        // Low-pass
        let fc = (200.0 + (filter_val + 100.0) / 100.0 * 8000.0).clamp(200.0, 18000.0);
        lowpass_coeffs(fc, 0.7, fs)
    } else if filter_val > 5.0 {
        // High-pass
        let fc = (20.0 + filter_val / 100.0 * 4000.0).clamp(20.0, 18000.0);
        highpass_coeffs(fc, 0.7, fs)
    } else {
        // Bypass
        (1.0, 0.0, 0.0, 0.0, 0.0)
    };

    // Process chain: Low → Mid → High → Filter
    let s1 = if channel == 0 {
        biquad_process(&mut dsp.low_l, x, lb0, lb1, lb2, la1, la2)
    } else {
        biquad_process(&mut dsp.low_r, x, lb0, lb1, lb2, la1, la2)
    };

    let s2 = if channel == 0 {
        biquad_process(&mut dsp.mid_l, s1, mb0, mb1, mb2, ma1, ma2)
    } else {
        biquad_process(&mut dsp.mid_r, s1, mb0, mb1, mb2, ma1, ma2)
    };

    let s3 = if channel == 0 {
        biquad_process(&mut dsp.high_l, s2, hb0, hb1, hb2, ha1, ha2)
    } else {
        biquad_process(&mut dsp.high_r, s2, hb0, hb1, hb2, ha1, ha2)
    };

    let s4 = if channel == 0 {
        biquad_process(&mut dsp.filter_l, s3, fb0, fb1, fb2, fa1, fa2)
    } else {
        biquad_process(&mut dsp.filter_r, s3, fb0, fb1, fb2, fa1, fa2)
    };

    s4 * params.gain_linear()
}