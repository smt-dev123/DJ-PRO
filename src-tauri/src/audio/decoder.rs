use serde::{Deserialize, Serialize};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WaveformPeak {
    pub low: f32,   // < 250 Hz (Bass / Kick drum) - Normalized 0.0 to 1.0
    pub mid: f32,   // 250 Hz - 4000 Hz (Vocals / Snares)
    pub high: f32,  // > 4000 Hz (Hi-hats / Air)
}

pub type WaveBandPeak = WaveformPeak;

pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub title: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f32,
    pub waveform_peaks: Vec<WaveformPeak>,
}

pub fn decode_file(path: &str) -> Result<DecodedAudio, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| e.to_string())?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio track found")?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;

    let track_id = track.id;
    let mut all_samples = Vec::new();

    // Pre-allocate memory to avoid multiple reallocations
    if let Some(n_frames) = track.codec_params.n_frames {
        all_samples.reserve((n_frames * channels as u64) as usize);
    } else {
        all_samples.reserve((sample_rate * channels as u32 * 180) as usize);
    }

    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(pkt) => pkt,
            Err(Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(err.to_string()),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                if sample_buf.is_none() {
                    sample_buf = Some(SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec()));
                }
                if let Some(buf) = sample_buf.as_mut() {
                    buf.copy_interleaved_ref(decoded);
                    all_samples.extend_from_slice(buf.samples());
                }
            }
            Err(Error::DecodeError(_)) => continue,
            Err(err) => return Err(err.to_string()),
        }
    }

    let file_name = std::path::Path::new(path)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Track")
        .to_string();

    let duration_secs = if sample_rate > 0 && channels > 0 {
        (all_samples.len() as f32) / (sample_rate as f32 * channels as f32)
    } else {
        0.0
    };

    // Target ~35 peaks per second for ultra-high-definition 60FPS dual scrolling waveform
    let peaks_per_sec = 35.0f32;
    let target_peaks = if duration_secs > 0.0 {
        ((duration_secs * peaks_per_sec) as usize).clamp(360, 7200)
    } else {
        1200
    };

    let mut waveform_peaks = Vec::with_capacity(target_peaks);

    if !all_samples.is_empty() && sample_rate > 0 && channels > 0 {
        let fs = sample_rate as f32;
        // Lowpass at 250 Hz (< 250 Hz: Bass / Kick drum)
        let fc_low = 250.0f32;
        let w_low = 2.0 * std::f32::consts::PI * fc_low / fs;
        let alpha_low = (w_low / (1.0 + w_low)).clamp(0.001, 0.99);

        // Lowpass at 4000 Hz (Crossover for Vocals / Mid to Cymbals / High)
        let fc_high = 4000.0f32;
        let w_high = 2.0 * std::f32::consts::PI * fc_high / fs;
        let alpha_high = (w_high / (1.0 + w_high)).clamp(0.001, 0.99);

        let total_frames = all_samples.len() / (channels as usize);
        let frames_per_peak = (total_frames / target_peaks).max(1);
        let samples_per_peak = frames_per_peak * (channels as usize);

        let mut lp1_low = 0.0f32;
        let mut lp2_low = 0.0f32;
        let mut lp1_high = 0.0f32;
        let mut lp2_high = 0.0f32;

        for chunk in all_samples.chunks(samples_per_peak) {
            let mut peak_low = 0.0f32;
            let mut peak_mid = 0.0f32;
            let mut peak_high = 0.0f32;

            let ch = channels as usize;
            for frame in chunk.chunks(ch) {
                let x = if frame.len() == 1 {
                    frame[0]
                } else {
                    frame.iter().sum::<f32>() / (frame.len() as f32)
                };

                // 2-pole Linkwitz-Riley lowpass @ 250 Hz (Bass / Kick)
                lp1_low += alpha_low * (x - lp1_low);
                lp2_low += alpha_low * (lp1_low - lp2_low);
                let s_low = lp2_low;

                // 2-pole Linkwitz-Riley lowpass @ 4000 Hz
                lp1_high += alpha_high * (x - lp1_high);
                lp2_high += alpha_high * (lp1_high - lp2_high);
                let s_mid_high = lp2_high;

                // Mid band (250 Hz - 4000 Hz): Vocals / Snares / Melody
                let s_mid = s_mid_high - s_low;

                // High band (> 4000 Hz): Hi-hats / Cymbals / Air
                let s_high = x - s_mid_high;

                let abs_low = s_low.abs();
                let abs_mid = s_mid.abs();
                let abs_high = s_high.abs();

                if abs_low > peak_low { peak_low = abs_low; }
                if abs_mid > peak_mid { peak_mid = abs_mid; }
                if abs_high > peak_high { peak_high = abs_high; }
            }

            // Normalize and apply perceptual loudness balance
            let low = (peak_low * 1.15).clamp(0.04, 1.0);
            let mid = (peak_mid * 1.65).clamp(0.03, 1.0);
            let high = (peak_high * 2.85).clamp(0.02, 1.0);

            waveform_peaks.push(WaveformPeak { low, mid, high });
        }
    }

    if waveform_peaks.is_empty() {
        for _ in 0..target_peaks {
            waveform_peaks.push(WaveformPeak { low: 0.15, mid: 0.12, high: 0.08 });
        }
    }

    Ok(DecodedAudio {
        samples: all_samples,
        title: file_name,
        sample_rate,
        channels,
        duration_secs,
        waveform_peaks,
    })
}