pub mod decoder;
pub mod state;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream, StreamConfig};
use state::MixerState;
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    pub state: Arc<Mutex<MixerState>>,
    _stream: Stream,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mixer_state = Arc::new(Mutex::new(MixerState::default()));

        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("No audio output device found")?;
        let supported_config = device.default_output_config()?;
        let stream_config: StreamConfig = supported_config.clone().into();

        let state_clone = Arc::clone(&mixer_state);

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                stream_config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    let mut lock = state_clone.lock().unwrap();
                    let cf = lock.crossfader;
                    let gain_a = (1.0 - cf).clamp(0.0, 1.0);
                    let gain_b = cf.clamp(0.0, 1.0);

                    for sample in data.iter_mut() {
                        let mut out_a = 0.0;
                        let mut out_b = 0.0;

                        if lock.deck_a.is_playing && lock.deck_a.position < lock.deck_a.samples.len() {
                            out_a = lock.deck_a.samples[lock.deck_a.position] * lock.deck_a.volume * gain_a;
                            lock.deck_a.position += 1;
                        }

                        if lock.deck_b.is_playing && lock.deck_b.position < lock.deck_b.samples.len() {
                            out_b = lock.deck_b.samples[lock.deck_b.position] * lock.deck_b.volume * gain_b;
                            lock.deck_b.position += 1;
                        }

                        *sample = (out_a + out_b).clamp(-1.0, 1.0);
                    }
                },
                |err| eprintln!("Audio stream error: {:?}", err),
                None,
            )?,
            _ => return Err("Only F32 audio format supported".into()),
        };

        stream.play()?;

        Ok(Self {
            state: mixer_state,
            _stream: stream,
        })
    }
}