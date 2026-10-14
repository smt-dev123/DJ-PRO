use serde::{Deserialize, Serialize};
use std::time::Instant;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

// ==================== STRUCTS ====================

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WaveformPeak {
    pub low: f32,   // < 250 Hz (Bass / Kick drum)
    pub mid: f32,   // 250 Hz - 4000 Hz (Vocals / Snares)
    pub high: f32,  // > 4000 Hz (Hi-hats / Air)
}

pub type WaveBandPeak = WaveformPeak;

pub struct DecodedSamples {
    pub samples: Vec<f32>,
    pub title: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f32,
}

// ==================== PHASE 1: DECODE SAMPLES ONLY ====================

pub fn decode_samples_only(path: &str) -> Result<DecodedSamples, String> {
    let t_start = Instant::now();

    // --- Open file ---
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    println!(
        "[Decoder] File open: {}ms (size: {} bytes)",
        t_start.elapsed().as_millis(),
        file_size
    );

    // --- Probe format ---
    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| e.to_string())?;

    let mut format = probed.format;

    // --- Find audio track ---
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio track found")?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

    // --- Create decoder ---
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;

    let track_id = track.id;

    // --- Pre-allocate samples ---
    let mut all_samples: Vec<f32> = Vec::new();
    if let Some(n_frames) = track.codec_params.n_frames {
        all_samples.reserve((n_frames * channels as u64) as usize);
    } else {
        all_samples.reserve((sample_rate * channels as u32 * 180) as usize);
    }

    // --- Decode loop ---
    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let t_decode = Instant::now();

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
                    sample_buf = Some(SampleBuffer::<f32>::new(
                        decoded.capacity() as u64,
                        *decoded.spec(),
                    ));
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

    println!(
        "[Decoder] Decode loop: {}ms ({} samples)",
        t_decode.elapsed().as_millis(),
        all_samples.len()
    );

    // --- Metadata ---
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

    println!(
        "[Decoder] ✅ Phase 1 COMPLETE: {}ms total, {:.1}s duration",
        t_start.elapsed().as_millis(),
        duration_secs
    );

    Ok(DecodedSamples {
        samples: all_samples,
        title: file_name,
        sample_rate,
        channels,
        duration_secs,
    })
}

// ==================== PHASE 2: PROGRESSIVE WAVEFORM ====================

pub fn extract_waveform_progressive<F>(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    duration_secs: f32,
    total_chunks: usize,
    mut on_chunk: F,
) -> Vec<WaveformPeak>
where
    F: FnMut(Vec<WaveformPeak>, usize, usize),
{
    let t_start = Instant::now();

    if samples.is_empty() || sample_rate == 0 || channels == 0 {
        return Vec::new();
    }

    // ✅ 10 peaks/sec — លឿន + គុណភាពគ្រប់គ្រាន់
    let peaks_per_sec = 10.0f32;
    let target_peaks = if duration_secs > 0.0 {
        ((duration_secs * peaks_per_sec) as usize).clamp(240, 2400)
    } else {
        600
    };

    let mut waveform_peaks = Vec::with_capacity(target_peaks);

    // Filter coefficients
    let fs = sample_rate as f32;
    let fc_low = 250.0f32;
    let w_low = 2.0 * std::f32::consts::PI * fc_low / fs;
    let alpha_low = (w_low / (1.0 + w_low)).clamp(0.001, 0.99);

    let fc_high = 4000.0f32;
    let w_high = 2.0 * std::f32::consts::PI * fc_high / fs;
    let alpha_high = (w_high / (1.0 + w_high)).clamp(0.001, 0.99);

    let total_frames = samples.len() / (channels as usize);
    let frames_per_peak = (total_frames / target_peaks).max(1);
    let samples_per_peak = frames_per_peak * (channels as usize);

    let peaks_per_chunk = (target_peaks / total_chunks.max(1)).max(1);

    let mut lp1_low = 0.0f32;
    let mut lp2_low = 0.0f32;
    let mut lp1_high = 0.0f32;
    let mut lp2_high = 0.0f32;

    let mut current_chunk: Vec<WaveformPeak> = Vec::with_capacity(peaks_per_chunk);
    let mut chunk_idx = 0usize;

    for chunk in samples.chunks(samples_per_peak) {
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

            // Low band @ 250 Hz
            lp1_low += alpha_low * (x - lp1_low);
            lp2_low += alpha_low * (lp1_low - lp2_low);
            let s_low = lp2_low;

            // High band @ 4000 Hz
            lp1_high += alpha_high * (x - lp1_high);
            lp2_high += alpha_high * (lp1_high - lp2_high);
            let s_mid_high = lp2_high;

            let s_mid = s_mid_high - s_low;
            let s_high = x - s_mid_high;

            let abs_low = s_low.abs();
            let abs_mid = s_mid.abs();
            let abs_high = s_high.abs();

            if abs_low > peak_low { peak_low = abs_low; }
            if abs_mid > peak_mid { peak_mid = abs_mid; }
            if abs_high > peak_high { peak_high = abs_high; }
        }

        // Normalize + perceptual balance
        let low = (peak_low * 1.15).clamp(0.04, 1.0);
        let mid = (peak_mid * 1.65).clamp(0.03, 1.0);
        let high = (peak_high * 2.85).clamp(0.02, 1.0);

        let peak = WaveformPeak { low, mid, high };
        waveform_peaks.push(peak.clone());
        current_chunk.push(peak);

        // ✅ Emit chunk រាល់ peaks_per_chunk
        if current_chunk.len() >= peaks_per_chunk {
            let chunk_to_emit = std::mem::take(&mut current_chunk);
            on_chunk(chunk_to_emit, chunk_idx, total_chunks);
            chunk_idx += 1;
        }
    }

    // ✅ Emit chunk ចុងក្រោយ
    if !current_chunk.is_empty() {
        on_chunk(current_chunk, chunk_idx, total_chunks);
    }

    println!(
        "[Decoder] ✅ Progressive waveform: {}ms ({} peaks, {} chunks)",
        t_start.elapsed().as_millis(),
        waveform_peaks.len(),
        total_chunks
    );

    waveform_peaks
}

// ==================== SYNC WRAPPER (backward compat) ====================

pub fn extract_waveform(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    duration_secs: f32,
) -> Vec<WaveformPeak> {
    extract_waveform_progressive(
        samples,
        sample_rate,
        channels,
        duration_secs,
        1,
        |_, _, _| {},
    )
}

// ==================== FALLBACK ====================

pub fn generate_fallback_waveform(count: usize) -> Vec<WaveformPeak> {
    (0..count)
        .map(|i| WaveformPeak {
            low: 0.2 + (i as f32 * 0.15).sin().abs() * 0.5,
            mid: 0.15 + (i as f32 * 0.2).cos().abs() * 0.45,
            high: 0.1 + (i as f32 * 0.35).sin().abs() * 0.4,
        })
        .collect()
}