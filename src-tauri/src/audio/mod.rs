use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream, StreamConfig};
use dsp::{apply_eq, EqParams};
use fx::apply_fx;
use state::MixerState;
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    pub state: Arc<Mutex<MixerState>>,
}
pub mod decoder;
pub mod dsp;
pub mod fx;
pub mod state;

thread_local! {
    static AUDIO_STREAM: std::cell::RefCell<Option<Stream>> = std::cell::RefCell::new(None);
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mixer_state = Arc::new(Mutex::new(MixerState::default()));

        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("No audio output device")?;
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

                    let sample_rate = lock.deck_a.sample_rate.max(1) as f32;
                    let block_size = data.len() as f32;
                    let block_duration = block_size / (sample_rate * 2.0); // stereo

                    // ==================== FX OUT FADE (Deck A) ====================
                    if lock.deck_a.fx_out_active {
                        // Ramp amount 0 → 1
                        let amount_step = block_duration / lock.deck_a.fx_out_duration;
                        lock.fx_amount_a = (lock.fx_amount_a + amount_step).min(1.0);

                        // ✅ Fade volume: 1.0 → 0.0 (exponential fade for smoothness)
                        lock.deck_a.fx_out_progress += amount_step;
                        let progress = lock.deck_a.fx_out_progress.clamp(0.0, 1.0);
                        lock.deck_a.fx_out_volume = 1.0 - progress.powf(0.7); // S-curve fade

                        // ✅ Auto-complete
                        if progress >= 1.0 {
                            lock.deck_a.fx_out_active = false;
                            lock.deck_a.fx_out_volume = 1.0;
                            lock.deck_a.is_playing = false;
                            lock.fx_active_a = false;
                            lock.fx_a.reset();
                            println!("[FX Out] Deck A complete");
                        }
                    } else {
                        lock.deck_a.fx_out_volume = 1.0;
                    }

                    // ==================== FX OUT FADE (Deck B) ====================
                    if lock.deck_b.fx_out_active {
                        let amount_step = block_duration / lock.deck_b.fx_out_duration;
                        lock.fx_amount_b = (lock.fx_amount_b + amount_step).min(1.0);

                        lock.deck_b.fx_out_progress += amount_step;
                        let progress = lock.deck_b.fx_out_progress.clamp(0.0, 1.0);
                        lock.deck_b.fx_out_volume = 1.0 - progress.powf(0.7);

                        if progress >= 1.0 {
                            lock.deck_b.fx_out_active = false;
                            lock.deck_b.fx_out_volume = 1.0;
                            lock.deck_b.is_playing = false;
                            lock.fx_active_b = false;
                            lock.fx_b.reset();
                            println!("[FX Out] Deck B complete");
                        }
                    } else {
                        lock.deck_b.fx_out_volume = 1.0;
                    }

                    // ==================== DSP & FX STATE ====================
                    let mut dsp_a = std::mem::take(&mut lock.dsp_a);
                    let mut dsp_b = std::mem::take(&mut lock.dsp_b);
                    let mut fx_a = std::mem::take(&mut lock.fx_a);
                    let mut fx_b = std::mem::take(&mut lock.fx_b);

                    fx_a.fx_type = lock.fx_type_a;
                    fx_a.fx_amount = lock.fx_amount_a;
                    fx_a.fx_active = lock.fx_active_a;
                    fx_a.params = lock.fx_params_a;

                    fx_b.fx_type = lock.fx_type_b;
                    fx_b.fx_amount = lock.fx_amount_b;
                    fx_b.fx_active = lock.fx_active_b;
                    fx_b.params = lock.fx_params_b;

                    let params_a = EqParams::from_deck(
                        lock.deck_a.gain, lock.deck_a.eq_high,
                        lock.deck_a.eq_mid, lock.deck_a.eq_low, lock.deck_a.filter,
                    );
                    let params_b = EqParams::from_deck(
                        lock.deck_b.gain, lock.deck_b.eq_high,
                        lock.deck_b.eq_mid, lock.deck_b.eq_low, lock.deck_b.filter,
                    );

                    let sr_a = lock.deck_a.sample_rate as f32;
                    let sr_b = lock.deck_b.sample_rate as f32;
                    let ch_a = lock.deck_a.channels as usize;
                    let ch_b = lock.deck_b.channels as usize;

                    // ✅ Volume × FX Out volume (independent per deck)
                    let vol_a = lock.deck_a.volume * lock.deck_a.fx_out_volume;
                    let vol_b = lock.deck_b.volume * lock.deck_b.fx_out_volume;

                    let mut pos_a = lock.deck_a.position;
                    let mut pos_b = lock.deck_b.position;
                    let play_a = lock.deck_a.is_playing;
                    let play_b = lock.deck_b.is_playing;
                    let len_a = lock.deck_a.samples.len();
                    let len_b = lock.deck_b.samples.len();

                    for (frame_idx, sample) in data.iter_mut().enumerate() {
                        let mut out_a = 0.0;
                        let mut out_b = 0.0;

                        // DECK A — EQ → FX → Volume (× FX Out)
                        if play_a && pos_a < len_a {
                            let raw = lock.deck_a.samples[pos_a];
                            let ch = frame_idx % ch_a.max(2);
                            let eq = apply_eq(&mut dsp_a, &params_a, raw, ch, sr_a);
                            let dry_feed = if lock.deck_a.fx_out_active {
                                eq * (1.0 - (lock.deck_a.fx_out_progress * 2.0).min(1.0))
                            } else {
                                eq
                            };
                            let fx_out = apply_fx(&mut fx_a, dry_feed, ch, sr_a);
                            out_a = fx_out * vol_a * gain_a;
                            pos_a += 1;
                        }

                        // DECK B
                        if play_b && pos_b < len_b {
                            let raw = lock.deck_b.samples[pos_b];
                            let ch = frame_idx % ch_b.max(2);
                            let eq = apply_eq(&mut dsp_b, &params_b, raw, ch, sr_b);
                            let dry_feed = if lock.deck_b.fx_out_active {
                                eq * (1.0 - (lock.deck_b.fx_out_progress * 2.0).min(1.0))
                            } else {
                                eq
                            };
                            let fx_out = apply_fx(&mut fx_b, dry_feed, ch, sr_b);
                            out_b = fx_out * vol_b * gain_b;
                            pos_b += 1;
                        }

                        *sample = (out_a + out_b).clamp(-1.0, 1.0);
                    }

                    lock.dsp_a = dsp_a;
                    lock.dsp_b = dsp_b;
                    lock.fx_a = fx_a;
                    lock.fx_b = fx_b;
                    lock.deck_a.position = pos_a;
                    lock.deck_b.position = pos_b;
                },
                |err| eprintln!("Audio error: {:?}", err),
                None,
            )?,
            _ => return Err("Only F32 supported".into()),
        };

        stream.play()?;
        AUDIO_STREAM.with(|s| { *s.borrow_mut() = Some(stream); });

        Ok(Self { state: mixer_state })
    }
}