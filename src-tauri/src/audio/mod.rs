pub mod decoder;
pub mod dsp;
pub mod state;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream, StreamConfig};
use dsp::{apply_eq, EqParams};
use state::MixerState;
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    pub state: Arc<Mutex<MixerState>>,
}

thread_local! {
    static AUDIO_STREAM: std::cell::RefCell<Option<Stream>> = std::cell::RefCell::new(None);
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mixer_state = Arc::new(Mutex::new(MixerState::default()));

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No audio output device found")?;
        let supported_config = device.default_output_config()?;
        let stream_config: StreamConfig = supported_config.clone().into();

        let state_clone = Arc::clone(&mixer_state);

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    let mut lock = match state_clone.lock() {
                        Ok(g) => g,
                        Err(_) => return,
                    };

                    let cf = lock.crossfader;
                    let gain_a = (1.0 - cf).clamp(0.0, 1.0);
                    let gain_b = cf.clamp(0.0, 1.0);

                    // ✅ ចាប់យក DSP state + EQ params មុន loop
                    let mut dsp_a = std::mem::take(&mut lock.dsp_a);
                    let mut dsp_b = std::mem::take(&mut lock.dsp_b);

                    let params_a = EqParams::from_deck(
                        lock.deck_a.gain,
                        lock.deck_a.eq_high,
                        lock.deck_a.eq_mid,
                        lock.deck_a.eq_low,
                        lock.deck_a.filter,
                    );
                    let params_b = EqParams::from_deck(
                        lock.deck_b.gain,
                        lock.deck_b.eq_high,
                        lock.deck_b.eq_mid,
                        lock.deck_b.eq_low,
                        lock.deck_b.filter,
                    );

                    let sample_rate_a = lock.deck_a.sample_rate as f32;
                    let sample_rate_b = lock.deck_b.sample_rate as f32;
                    let channels_a = lock.deck_a.channels as usize;
                    let channels_b = lock.deck_b.channels as usize;
                    let vol_a = lock.deck_a.volume;
                    let vol_b = lock.deck_b.volume;

                    let mut pos_a = lock.deck_a.position;
                    let mut pos_b = lock.deck_b.position;
                    let play_a = lock.deck_a.is_playing;
                    let play_b = lock.deck_b.is_playing;
                    let len_a = lock.deck_a.samples.len();
                    let len_b = lock.deck_b.samples.len();

                    for (frame_idx, sample) in data.iter_mut().enumerate() {
                        let mut out_a = 0.0;
                        let mut out_b = 0.0;

                        // ==================== DECK A ====================
                        if play_a && pos_a < len_a {
                            let raw = lock.deck_a.samples[pos_a];
                            let channel = frame_idx % channels_a.max(2);
                            let processed = apply_eq(&mut dsp_a, &params_a, raw, channel, sample_rate_a);
                            out_a = processed * vol_a * gain_a;
                            pos_a += 1;
                        }

                        // ==================== DECK B ====================
                        if play_b && pos_b < len_b {
                            let raw = lock.deck_b.samples[pos_b];
                            let channel = frame_idx % channels_b.max(2);
                            let processed = apply_eq(&mut dsp_b, &params_b, raw, channel, sample_rate_b);
                            out_b = processed * vol_b * gain_b;
                            pos_b += 1;
                        }

                        *sample = (out_a + out_b).clamp(-1.0, 1.0);
                    }

                    // ✅ Save ត្រឡប់
                    lock.dsp_a = dsp_a;
                    lock.dsp_b = dsp_b;
                    lock.deck_a.position = pos_a;
                    lock.deck_b.position = pos_b;
                },
                |err| eprintln!("Audio stream error: {:?}", err),
                None,
            )?,
            _ => return Err("Only F32 audio format supported".into()),
        };

        stream.play()?;

        AUDIO_STREAM.with(|s| {
            *s.borrow_mut() = Some(stream);
        });

        Ok(Self { state: mixer_state })
    }
}