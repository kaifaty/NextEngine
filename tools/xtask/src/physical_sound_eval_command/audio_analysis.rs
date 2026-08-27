use std::f64::consts::PI;

use serde::Serialize;

use super::{
    DecayBandReport, FFT_SIZES, FileAnalysisReport, LOG_SPECTRUM_BINS, MAX_DURATION_SECONDS,
    ModalPeakReport, SignalReport, SpectrumReport,
};

pub(crate) struct WavAudio {
    pub(crate) sample_format: String,
    pub(crate) sample_rate_hz: u32,
    pub(crate) channel_count: u16,
    pub(crate) mono_samples: Vec<f64>,
}

pub(super) struct Analysis {
    pub report: FileAnalysisReport,
    pub log_spectra: Vec<Vec<f64>>,
}

#[derive(Clone, Debug)]
pub(crate) struct BenchmarkAudioAnalysis {
    pub(crate) sample_rate_hz: u32,
    pub(crate) channel_count: u16,
    pub(crate) duration_ms: f64,
    pub(crate) peak_dbfs: f64,
    pub(crate) rms_dbfs: f64,
    pub(crate) hard_failure_tags: Vec<&'static str>,
    pub(crate) feature_values: Vec<f64>,
    pub(crate) temporal_feature_values: Vec<f64>,
    pub(crate) temporal_dynamics: TemporalDynamicsReport,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TemporalDynamicsReport {
    pub(crate) frame_count: usize,
    pub(crate) mean_spectral_flux: f64,
    pub(crate) stddev_spectral_flux: f64,
    pub(crate) mean_adjacent_cosine_distance: f64,
    pub(crate) stddev_adjacent_cosine_distance: f64,
    pub(crate) early_late_cosine_distance: f64,
    pub(crate) mean_centroid_motion_nyquist_fraction: f64,
    pub(crate) centroid_range_nyquist_fraction: f64,
    pub(crate) mean_flatness_motion_db: f64,
    pub(crate) flatness_range_db: f64,
    pub(crate) mean_active_bin_turnover: f64,
}

pub(crate) fn parse_wav(bytes: &[u8]) -> Result<WavAudio, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("expected RIFF/WAVE header".to_owned());
    }
    let mut offset = 12_usize;
    let mut format = None;
    let mut data = None;
    while offset.checked_add(8).is_some_and(|end| end <= bytes.len()) {
        let id = &bytes[offset..offset + 4];
        let size = usize::try_from(read_u32(bytes, offset + 4)?)
            .map_err(|_| "WAV chunk length does not fit usize".to_owned())?;
        let payload = offset + 8;
        let end = payload
            .checked_add(size)
            .ok_or_else(|| "WAV chunk length overflow".to_owned())?;
        if end > bytes.len() {
            return Err("WAV chunk exceeds file length".to_owned());
        }
        match id {
            b"fmt " if format.is_none() => format = Some(parse_wav_format(&bytes[payload..end])?),
            b"data" if data.is_none() => data = Some(&bytes[payload..end]),
            _ => {}
        }
        offset = end
            .checked_add(size & 1)
            .ok_or_else(|| "WAV padding overflow".to_owned())?;
    }
    let format = format.ok_or_else(|| "WAV has no fmt chunk".to_owned())?;
    let data = data.ok_or_else(|| "WAV has no data chunk".to_owned())?;
    decode_wav(format, data)
}

#[derive(Clone, Copy)]
struct WavFormat {
    format_tag: u16,
    channels: u16,
    sample_rate_hz: u32,
    block_align: u16,
    bits_per_sample: u16,
}

fn parse_wav_format(bytes: &[u8]) -> Result<WavFormat, String> {
    if bytes.len() < 16 {
        return Err("WAV fmt chunk is shorter than 16 bytes".to_owned());
    }
    let mut format_tag = read_u16(bytes, 0)?;
    let channels = read_u16(bytes, 2)?;
    let sample_rate_hz = read_u32(bytes, 4)?;
    let block_align = read_u16(bytes, 12)?;
    let bits_per_sample = read_u16(bytes, 14)?;
    if format_tag == 0xfffe {
        if bytes.len() < 40 || read_u16(bytes, 16)? < 22 {
            return Err("invalid WAVE_FORMAT_EXTENSIBLE fmt chunk".to_owned());
        }
        const WAVE_SUBFORMAT_TAIL: [u8; 14] = [
            0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71,
        ];
        if bytes[26..40] != WAVE_SUBFORMAT_TAIL {
            return Err("unsupported WAVE_FORMAT_EXTENSIBLE subformat GUID".to_owned());
        }
        format_tag = read_u16(bytes, 24)?;
    }
    Ok(WavFormat {
        format_tag,
        channels,
        sample_rate_hz,
        block_align,
        bits_per_sample,
    })
}

fn decode_wav(format: WavFormat, data: &[u8]) -> Result<WavAudio, String> {
    if !(1..=8).contains(&format.channels) {
        return Err(format!(
            "unsupported WAV channel count: {}",
            format.channels
        ));
    }
    if !(8_000..=192_000).contains(&format.sample_rate_hz) {
        return Err(format!(
            "unsupported WAV sample rate: {}",
            format.sample_rate_hz
        ));
    }
    let bytes_per_sample = usize::from(format.bits_per_sample.div_ceil(8));
    let expected_align = usize::from(format.channels)
        .checked_mul(bytes_per_sample)
        .ok_or_else(|| "WAV block alignment overflow".to_owned())?;
    if usize::from(format.block_align) != expected_align || expected_align == 0 {
        return Err("WAV block alignment does not match channels/sample width".to_owned());
    }
    if !data.len().is_multiple_of(expected_align) {
        return Err("WAV data is not a complete frame sequence".to_owned());
    }
    let frame_count = data.len() / expected_align;
    let maximum_frames = usize::try_from(format.sample_rate_hz)
        .map_err(|_| "sample rate does not fit usize".to_owned())?
        .checked_mul(MAX_DURATION_SECONDS)
        .ok_or_else(|| "maximum WAV frame count overflow".to_owned())?;
    if frame_count == 0 || frame_count > maximum_frames {
        return Err(format!(
            "WAV frame count must be 1..={maximum_frames}, got {frame_count}"
        ));
    }
    let sample_format = match (format.format_tag, format.bits_per_sample) {
        (1, 8) => "pcm-u8",
        (1, 16) => "pcm-s16",
        (1, 24) => "pcm-s24",
        (1, 32) => "pcm-s32",
        (3, 32) => "ieee-f32",
        pair => return Err(format!("unsupported WAV format: {pair:?}")),
    };
    let mut mono_samples = Vec::with_capacity(frame_count);
    for frame in data.chunks_exact(expected_align) {
        let mut sum = 0.0_f64;
        for channel in 0..usize::from(format.channels) {
            let start = channel * bytes_per_sample;
            let sample = decode_sample(
                format.format_tag,
                format.bits_per_sample,
                &frame[start..start + bytes_per_sample],
            )?;
            if !sample.is_finite() {
                return Err("WAV contains a non-finite sample".to_owned());
            }
            sum += sample;
        }
        mono_samples.push(sum / f64::from(format.channels));
    }
    Ok(WavAudio {
        sample_format: sample_format.to_owned(),
        sample_rate_hz: format.sample_rate_hz,
        channel_count: format.channels,
        mono_samples,
    })
}

fn decode_sample(format_tag: u16, bits: u16, bytes: &[u8]) -> Result<f64, String> {
    match (format_tag, bits) {
        (1, 8) => Ok((f64::from(bytes[0]) - 128.0) / 128.0),
        (1, 16) => Ok(f64::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32_768.0),
        (1, 24) => {
            let sign = if bytes[2] & 0x80 == 0 { 0 } else { 0xff };
            let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], sign]);
            Ok(f64::from(value) / 8_388_608.0)
        }
        (1, 32) => Ok(
            f64::from(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                / 2_147_483_648.0,
        ),
        (3, 32) => Ok(f64::from(f32::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
        ]))),
        _ => Err("unsupported WAV sample encoding".to_owned()),
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "unexpected end of WAV integer".to_owned())?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "unexpected end of WAV integer".to_owned())?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

pub(super) fn analyze_wav(
    manifest_path: &str,
    wav_sha256: &str,
    wav: WavAudio,
) -> Result<Analysis, String> {
    let samples = &wav.mono_samples;
    let sample_rate = f64::from(wav.sample_rate_hz);
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    let sum = samples.iter().sum::<f64>();
    let energy = samples.iter().map(|sample| sample * sample).sum::<f64>();
    let rms = (energy / samples.len() as f64).sqrt();
    let dc_offset = sum / samples.len() as f64;
    let clipped_sample_count = samples
        .iter()
        .filter(|sample| sample.abs() >= 0.999)
        .count();
    let onset_frame = detect_onset(samples, peak);
    let attack_ms = onset_frame.and_then(|onset| attack_time_ms(samples, onset, peak, sample_rate));
    let temporal_centroid_ms = onset_frame.and_then(|onset| {
        temporal_centroid(samples, onset).map(|frames| frames * 1_000.0 / sample_rate)
    });
    let onset = onset_frame.unwrap_or(0);
    let log_spectra = FFT_SIZES
        .iter()
        .map(|size| log_spectrum_signature(samples, onset, wav.sample_rate_hz, *size))
        .collect::<Result<Vec<_>, _>>()?;
    let detailed_power = power_spectrum(samples, onset, 8_192)?;
    let spectrum = spectrum_report(&detailed_power, wav.sample_rate_hz, 8_192);
    let modal_peaks = modal_peaks(&detailed_power, wav.sample_rate_hz, 8_192);
    let decay = decay_reports(samples, onset, wav.sample_rate_hz)?;
    let report = FileAnalysisReport {
        manifest_path: manifest_path.to_owned(),
        wav_sha256: wav_sha256.to_owned(),
        sample_format: wav.sample_format,
        sample_rate_hz: wav.sample_rate_hz,
        channel_count: wav.channel_count,
        frame_count: samples.len(),
        duration_ms: samples.len() as f64 * 1_000.0 / sample_rate,
        signal: SignalReport {
            peak_dbfs: amplitude_db(peak),
            rms_dbfs: amplitude_db(rms),
            dc_offset,
            crest_db: amplitude_db(peak) - amplitude_db(rms),
            clipped_sample_count,
            onset_frame,
            attack_ms,
            temporal_centroid_ms,
        },
        spectrum,
        modal_peaks,
        decay,
    };
    Ok(Analysis {
        report,
        log_spectra,
    })
}

pub(crate) fn analyze_benchmark_wav(
    manifest_path: &str,
    wav_sha256: &str,
    wav: WavAudio,
) -> Result<BenchmarkAudioAnalysis, String> {
    let peak = wav
        .mono_samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    let temporal_onset = detect_onset(&wav.mono_samples, peak).unwrap_or(0);
    let temporal_dynamics =
        temporal_dynamics(&wav.mono_samples, temporal_onset, wav.sample_rate_hz)?;
    let analysis = analyze_wav(manifest_path, wav_sha256, wav)?;
    let report = &analysis.report;
    let mut hard_failure_tags = Vec::new();
    if report.signal.peak_dbfs <= -100.0 {
        hard_failure_tags.push("SILENCE");
    }
    if report.signal.clipped_sample_count * 100 > report.frame_count {
        hard_failure_tags.push("CLIPPING");
    }
    if report.signal.dc_offset.abs() > 0.02 {
        hard_failure_tags.push("EXCESSIVE_DC");
    }
    if report.duration_ms < 50.0 {
        hard_failure_tags.push("TOO_SHORT");
    }
    if report.signal.onset_frame.is_none() {
        hard_failure_tags.push("ONSET_MISSING");
    }

    let mut feature_values = analysis
        .log_spectra
        .iter()
        .flatten()
        .map(|level| (level / 120.0).clamp(-1.0, 0.0))
        .collect::<Vec<_>>();
    let usable_nyquist_hz = (f64::from(report.sample_rate_hz) * 0.5).min(20_000.0);
    feature_values.extend([
        (report.spectrum.centroid_hz / usable_nyquist_hz).clamp(0.0, 1.0),
        (report.spectrum.bandwidth_hz / usable_nyquist_hz).clamp(0.0, 1.0),
        (report.spectrum.flatness_db / 120.0).clamp(-1.0, 0.0),
        (report.signal.crest_db / 80.0).clamp(0.0, 1.0),
        report
            .signal
            .attack_ms
            .map(|value| (value / report.duration_ms).clamp(0.0, 1.0))
            .unwrap_or(0.0),
        report
            .signal
            .temporal_centroid_ms
            .map(|value| (value / report.duration_ms).clamp(0.0, 1.0))
            .unwrap_or(0.0),
        (report.modal_peaks.len() as f64 / 12.0).clamp(0.0, 1.0),
    ]);
    feature_values.extend(report.decay.iter().map(|band| {
        band.t20_ms
            .map(|value| (value / report.duration_ms.max(1.0)).clamp(0.0, 4.0) / 4.0)
            .unwrap_or(0.0)
    }));
    let temporal_feature_values = vec![
        temporal_dynamics.mean_spectral_flux.clamp(0.0, 1.0),
        temporal_dynamics.stddev_spectral_flux.clamp(0.0, 1.0),
        temporal_dynamics
            .mean_adjacent_cosine_distance
            .clamp(0.0, 1.0),
        temporal_dynamics
            .stddev_adjacent_cosine_distance
            .clamp(0.0, 1.0),
        temporal_dynamics.early_late_cosine_distance.clamp(0.0, 1.0),
        temporal_dynamics
            .mean_centroid_motion_nyquist_fraction
            .clamp(0.0, 1.0),
        temporal_dynamics
            .centroid_range_nyquist_fraction
            .clamp(0.0, 1.0),
        (temporal_dynamics.mean_flatness_motion_db / 120.0).clamp(0.0, 1.0),
        (temporal_dynamics.flatness_range_db / 120.0).clamp(0.0, 1.0),
        temporal_dynamics.mean_active_bin_turnover.clamp(0.0, 1.0),
    ];

    Ok(BenchmarkAudioAnalysis {
        sample_rate_hz: report.sample_rate_hz,
        channel_count: report.channel_count,
        duration_ms: report.duration_ms,
        peak_dbfs: report.signal.peak_dbfs,
        rms_dbfs: report.signal.rms_dbfs,
        hard_failure_tags,
        feature_values,
        temporal_feature_values,
        temporal_dynamics,
    })
}

fn temporal_dynamics(
    samples: &[f64],
    onset: usize,
    sample_rate_hz: u32,
) -> Result<TemporalDynamicsReport, String> {
    const WINDOW: usize = 1_024;
    const HOP: usize = 256;
    const MAX_FRAMES: usize = 256;
    const ACTIVE_RELATIVE_POWER: f64 = 1.0e-4;

    let available = samples.len().saturating_sub(onset);
    let frame_count = if available <= WINDOW {
        1
    } else {
        1 + (available - WINDOW) / HOP
    }
    .min(MAX_FRAMES);
    let bin_hz = f64::from(sample_rate_hz) / WINDOW as f64;
    let start_bin = (80.0 / bin_hz).ceil() as usize;
    let end_bin = ((20_000.0 / bin_hz).floor() as usize).min(WINDOW / 2);
    if start_bin > end_bin {
        return Err("temporal dynamics has no usable spectrum bins".to_owned());
    }

    let mut spectra = Vec::with_capacity(frame_count);
    let mut active_bins = Vec::with_capacity(frame_count);
    let mut centroids = Vec::with_capacity(frame_count);
    let mut flatness_db = Vec::with_capacity(frame_count);
    let usable_nyquist_hz = (f64::from(sample_rate_hz) * 0.5).min(20_000.0);
    for frame in 0..frame_count {
        let power = power_spectrum(samples, onset + frame * HOP, WINDOW)?;
        let selected = &power[start_bin..=end_bin];
        let total = selected.iter().sum::<f64>().max(1.0e-24);
        let maximum = selected.iter().copied().fold(1.0e-24, f64::max);
        let normalized = selected
            .iter()
            .map(|value| value / total)
            .collect::<Vec<_>>();
        let active = selected
            .iter()
            .map(|value| *value >= maximum * ACTIVE_RELATIVE_POWER)
            .collect::<Vec<_>>();
        let centroid_hz = selected
            .iter()
            .enumerate()
            .map(|(offset, value)| (start_bin + offset) as f64 * bin_hz * value)
            .sum::<f64>()
            / total;
        let arithmetic = total / selected.len() as f64;
        let geometric = (selected
            .iter()
            .map(|value| value.max(1.0e-24).ln())
            .sum::<f64>()
            / selected.len() as f64)
            .exp();
        spectra.push(normalized);
        active_bins.push(active);
        centroids.push((centroid_hz / usable_nyquist_hz).clamp(0.0, 1.0));
        flatness_db.push(10.0 * (geometric / arithmetic.max(1.0e-24)).log10());
    }

    let mut flux = Vec::with_capacity(frame_count.saturating_sub(1));
    let mut adjacent_distance = Vec::with_capacity(frame_count.saturating_sub(1));
    let mut centroid_motion = Vec::with_capacity(frame_count.saturating_sub(1));
    let mut flatness_motion = Vec::with_capacity(frame_count.saturating_sub(1));
    let mut active_turnover = Vec::with_capacity(frame_count.saturating_sub(1));
    for index in 1..frame_count {
        let previous = &spectra[index - 1];
        let current = &spectra[index];
        flux.push(
            current
                .iter()
                .zip(previous)
                .map(|(now, prior)| (now - prior).max(0.0))
                .sum::<f64>(),
        );
        adjacent_distance.push(cosine_distance(previous, current));
        centroid_motion.push((centroids[index] - centroids[index - 1]).abs());
        flatness_motion.push((flatness_db[index] - flatness_db[index - 1]).abs());
        let (intersection, union) = active_bins[index].iter().zip(&active_bins[index - 1]).fold(
            (0_usize, 0_usize),
            |(intersection, union), (now, prior)| {
                (
                    intersection + usize::from(*now && *prior),
                    union + usize::from(*now || *prior),
                )
            },
        );
        active_turnover.push(if union == 0 {
            0.0
        } else {
            1.0 - intersection as f64 / union as f64
        });
    }

    let (mean_spectral_flux, stddev_spectral_flux) = mean_stddev(&flux);
    let (mean_adjacent_cosine_distance, stddev_adjacent_cosine_distance) =
        mean_stddev(&adjacent_distance);
    Ok(TemporalDynamicsReport {
        frame_count,
        mean_spectral_flux,
        stddev_spectral_flux,
        mean_adjacent_cosine_distance,
        stddev_adjacent_cosine_distance,
        early_late_cosine_distance: cosine_distance(&spectra[0], &spectra[frame_count - 1]),
        mean_centroid_motion_nyquist_fraction: mean_stddev(&centroid_motion).0,
        centroid_range_nyquist_fraction: range(&centroids),
        mean_flatness_motion_db: mean_stddev(&flatness_motion).0,
        flatness_range_db: range(&flatness_db),
        mean_active_bin_turnover: mean_stddev(&active_turnover).0,
    })
}

fn cosine_distance(left: &[f64], right: &[f64]) -> f64 {
    let (dot, left_norm, right_norm) = left.iter().zip(right).fold(
        (0.0_f64, 0.0_f64, 0.0_f64),
        |(dot, left_norm, right_norm), (left, right)| {
            (
                dot + left * right,
                left_norm + left * left,
                right_norm + right * right,
            )
        },
    );
    if left_norm <= 1.0e-24 || right_norm <= 1.0e-24 {
        0.0
    } else {
        (1.0 - dot / (left_norm * right_norm).sqrt()).clamp(0.0, 1.0)
    }
}

fn mean_stddev(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    (mean, variance.sqrt())
}

fn range(values: &[f64]) -> f64 {
    let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if minimum.is_finite() && maximum.is_finite() {
        maximum - minimum
    } else {
        0.0
    }
}

fn detect_onset(samples: &[f64], peak: f64) -> Option<usize> {
    if peak < 1.0e-8 {
        return None;
    }
    let threshold = (peak * 0.02).max(1.0e-4);
    samples.iter().position(|sample| sample.abs() >= threshold)
}

fn attack_time_ms(samples: &[f64], onset: usize, peak: f64, sample_rate: f64) -> Option<f64> {
    let target = peak * 0.9;
    samples[onset..]
        .iter()
        .position(|sample| sample.abs() >= target)
        .map(|frames| frames as f64 * 1_000.0 / sample_rate)
}

fn temporal_centroid(samples: &[f64], onset: usize) -> Option<f64> {
    let mut weighted = 0.0_f64;
    let mut total = 0.0_f64;
    for (index, sample) in samples[onset..].iter().enumerate() {
        let energy = sample * sample;
        weighted += index as f64 * energy;
        total += energy;
    }
    (total > 1.0e-20).then_some(weighted / total)
}

fn amplitude_db(value: f64) -> f64 {
    20.0 * value.max(1.0e-12).log10()
}

fn power_spectrum(samples: &[f64], start: usize, size: usize) -> Result<Vec<f64>, String> {
    if !size.is_power_of_two() || size < 2 {
        return Err("FFT size must be a power of two".to_owned());
    }
    let mut real = vec![0.0_f64; size];
    let mut imaginary = vec![0.0_f64; size];
    let denominator = (size - 1) as f64;
    for (index, value) in real.iter_mut().enumerate() {
        let sample = samples.get(start + index).copied().unwrap_or(0.0);
        let window = 0.5 - 0.5 * (2.0 * PI * index as f64 / denominator).cos();
        *value = sample * window;
    }
    fft_in_place(&mut real, &mut imaginary);
    Ok((0..=size / 2)
        .map(|index| real[index] * real[index] + imaginary[index] * imaginary[index])
        .collect())
}

fn fft_in_place(real: &mut [f64], imaginary: &mut [f64]) {
    let size = real.len();
    let mut target = 0_usize;
    for source in 1..size {
        let mut bit = size >> 1;
        while target & bit != 0 {
            target ^= bit;
            bit >>= 1;
        }
        target ^= bit;
        if source < target {
            real.swap(source, target);
            imaginary.swap(source, target);
        }
    }
    let mut length = 2_usize;
    while length <= size {
        let angle = -2.0 * PI / length as f64;
        let step_real = angle.cos();
        let step_imaginary = angle.sin();
        for block in (0..size).step_by(length) {
            let mut twiddle_real = 1.0_f64;
            let mut twiddle_imaginary = 0.0_f64;
            for offset in 0..length / 2 {
                let even = block + offset;
                let odd = even + length / 2;
                let odd_real = real[odd] * twiddle_real - imaginary[odd] * twiddle_imaginary;
                let odd_imaginary = real[odd] * twiddle_imaginary + imaginary[odd] * twiddle_real;
                real[odd] = real[even] - odd_real;
                imaginary[odd] = imaginary[even] - odd_imaginary;
                real[even] += odd_real;
                imaginary[even] += odd_imaginary;
                let next_real = twiddle_real * step_real - twiddle_imaginary * step_imaginary;
                twiddle_imaginary = twiddle_real * step_imaginary + twiddle_imaginary * step_real;
                twiddle_real = next_real;
            }
        }
        length *= 2;
    }
}

fn log_spectrum_signature(
    samples: &[f64],
    onset: usize,
    sample_rate_hz: u32,
    fft_size: usize,
) -> Result<Vec<f64>, String> {
    let power = power_spectrum(samples, onset, fft_size)?;
    let maximum_hz = (f64::from(sample_rate_hz) * 0.5).min(20_000.0);
    let minimum_hz = 60.0_f64.min(maximum_hz * 0.5);
    let mut signature = Vec::with_capacity(LOG_SPECTRUM_BINS);
    for index in 0..LOG_SPECTRUM_BINS {
        let fraction = index as f64 / (LOG_SPECTRUM_BINS - 1) as f64;
        let frequency = minimum_hz * (maximum_hz / minimum_hz).powf(fraction);
        let bin = ((frequency * fft_size as f64 / f64::from(sample_rate_hz)).round() as usize)
            .min(power.len() - 1);
        signature.push(10.0 * power[bin].max(1.0e-24).log10());
    }
    let maximum = signature.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    for level in &mut signature {
        *level = (*level - maximum).max(-120.0);
    }
    Ok(signature)
}

fn spectrum_report(power: &[f64], sample_rate_hz: u32, fft_size: usize) -> SpectrumReport {
    let bin_hz = f64::from(sample_rate_hz) / fft_size as f64;
    let start = (50.0 / bin_hz).ceil() as usize;
    let end = ((20_000.0 / bin_hz).floor() as usize).min(power.len() - 1);
    let selected = &power[start..=end];
    let total = selected.iter().sum::<f64>().max(1.0e-24);
    let centroid_hz = selected
        .iter()
        .enumerate()
        .map(|(offset, value)| (start + offset) as f64 * bin_hz * value)
        .sum::<f64>()
        / total;
    let bandwidth_hz = (selected
        .iter()
        .enumerate()
        .map(|(offset, value)| {
            let frequency = (start + offset) as f64 * bin_hz;
            (frequency - centroid_hz).powi(2) * value
        })
        .sum::<f64>()
        / total)
        .sqrt();
    let arithmetic = total / selected.len() as f64;
    let geometric = (selected
        .iter()
        .map(|value| value.max(1.0e-24).ln())
        .sum::<f64>()
        / selected.len() as f64)
        .exp();
    SpectrumReport {
        centroid_hz,
        bandwidth_hz,
        flatness_db: 10.0 * (geometric / arithmetic.max(1.0e-24)).log10(),
    }
}

fn modal_peaks(power: &[f64], sample_rate_hz: u32, fft_size: usize) -> Vec<ModalPeakReport> {
    let bin_hz = f64::from(sample_rate_hz) / fft_size as f64;
    let start = (60.0 / bin_hz).ceil() as usize;
    let end = ((20_000.0 / bin_hz).floor() as usize).min(power.len() - 2);
    let maximum = power[start..=end].iter().copied().fold(1.0e-24, f64::max);
    let mut candidates = (start..=end)
        .filter(|index| power[*index] > power[*index - 1] && power[*index] >= power[*index + 1])
        .filter_map(|index| {
            let relative = 10.0 * (power[index].max(1.0e-24) / maximum).log10();
            (relative >= -55.0).then_some((index, relative))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut selected = Vec::<(usize, f64)>::new();
    for candidate in candidates {
        let frequency = candidate.0 as f64 * bin_hz;
        if selected.iter().all(|existing| {
            let other = existing.0 as f64 * bin_hz;
            (frequency / other).ln().abs() >= 0.025
        }) {
            selected.push(candidate);
            if selected.len() == 12 {
                break;
            }
        }
    }
    selected.sort_by_key(|value| value.0);
    selected
        .into_iter()
        .map(|(index, relative_level_db)| ModalPeakReport {
            frequency_hz: index as f64 * bin_hz,
            relative_level_db,
        })
        .collect()
}

fn decay_reports(
    samples: &[f64],
    onset: usize,
    sample_rate_hz: u32,
) -> Result<Vec<DecayBandReport>, String> {
    const WINDOW: usize = 1_024;
    const HOP: usize = 256;
    const BANDS: [(&str, f64, f64); 5] = [
        ("broadband", 50.0, 20_000.0),
        ("low", 80.0, 500.0),
        ("mid", 500.0, 2_000.0),
        ("high", 2_000.0, 8_000.0),
        ("air", 8_000.0, 20_000.0),
    ];
    let available = samples.len().saturating_sub(onset);
    let frame_count = if available <= WINDOW {
        1
    } else {
        1 + (available - WINDOW) / HOP
    };
    let frame_count = frame_count.min(1_024);
    let mut band_energy = vec![Vec::with_capacity(frame_count); BANDS.len()];
    let bin_hz = f64::from(sample_rate_hz) / WINDOW as f64;
    for frame in 0..frame_count {
        let power = power_spectrum(samples, onset + frame * HOP, WINDOW)?;
        for (band_index, (_, lower, upper)) in BANDS.iter().enumerate() {
            let start = (*lower / bin_hz).ceil() as usize;
            let end = ((*upper / bin_hz).floor() as usize).min(power.len() - 1);
            let energy = if start <= end {
                power[start..=end].iter().sum::<f64>()
            } else {
                0.0
            };
            band_energy[band_index].push(energy);
        }
    }
    Ok(BANDS
        .into_iter()
        .zip(band_energy)
        .map(|((band, lower_hz, upper_hz), energy)| {
            let (slope_db_per_second, t20_ms) = decay_fit(&energy, sample_rate_hz, HOP);
            DecayBandReport {
                band,
                lower_hz,
                upper_hz: upper_hz.min(f64::from(sample_rate_hz) * 0.5),
                slope_db_per_second,
                t20_ms,
            }
        })
        .collect())
}

fn decay_fit(energy: &[f64], sample_rate_hz: u32, hop: usize) -> (Option<f64>, Option<f64>) {
    let Some((peak_index, peak)) = energy
        .iter()
        .copied()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    else {
        return (None, None);
    };
    if peak <= 1.0e-20 {
        return (None, None);
    }
    let mut points = Vec::new();
    for (index, value) in energy.iter().copied().enumerate().skip(peak_index) {
        let relative_db = 10.0 * (value.max(1.0e-24) / peak).log10();
        if (-35.0..=-5.0).contains(&relative_db) {
            let time = (index - peak_index) as f64 * hop as f64 / f64::from(sample_rate_hz);
            points.push((time, relative_db));
        }
    }
    if points.len() < 4 {
        return (None, None);
    }
    let mean_x = points.iter().map(|point| point.0).sum::<f64>() / points.len() as f64;
    let mean_y = points.iter().map(|point| point.1).sum::<f64>() / points.len() as f64;
    let numerator = points
        .iter()
        .map(|point| (point.0 - mean_x) * (point.1 - mean_y))
        .sum::<f64>();
    let denominator = points
        .iter()
        .map(|point| (point.0 - mean_x).powi(2))
        .sum::<f64>();
    if denominator <= 1.0e-20 {
        return (None, None);
    }
    let slope = numerator / denominator;
    if !slope.is_finite() || slope >= -0.1 {
        return (Some(slope), None);
    }
    (Some(slope), Some(-20.0 / slope * 1_000.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporal_descriptor_separates_stationary_and_evolving_spectra() {
        let stationary =
            analyze_benchmark_wav("stationary.wav", &"0".repeat(64), test_audio(false))
                .expect("stationary analysis");
        let evolving = analyze_benchmark_wav("evolving.wav", &"1".repeat(64), test_audio(true))
            .expect("evolving analysis");

        assert_eq!(stationary.temporal_feature_values.len(), 10);
        assert!(
            stationary
                .temporal_feature_values
                .iter()
                .chain(&evolving.temporal_feature_values)
                .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        );
        assert!(stationary.temporal_dynamics.early_late_cosine_distance < 0.05);
        assert!(evolving.temporal_dynamics.early_late_cosine_distance > 0.8);
        assert!(
            evolving.temporal_dynamics.centroid_range_nyquist_fraction
                > stationary.temporal_dynamics.centroid_range_nyquist_fraction + 0.05
        );
    }

    #[test]
    fn temporal_descriptor_repeats_exactly() {
        let first = analyze_benchmark_wav("first.wav", &"0".repeat(64), test_audio(true))
            .expect("first analysis");
        let second = analyze_benchmark_wav("second.wav", &"0".repeat(64), test_audio(true))
            .expect("second analysis");
        assert_eq!(
            first.temporal_feature_values,
            second.temporal_feature_values
        );
        assert_eq!(
            serde_json::to_vec(&first.temporal_dynamics).expect("serialize first"),
            serde_json::to_vec(&second.temporal_dynamics).expect("serialize second")
        );
    }

    fn test_audio(evolving: bool) -> WavAudio {
        let sample_rate_hz = 48_000_u32;
        let sample_count = 24_000_usize;
        let mono_samples = (0..sample_count)
            .map(|index| {
                let time = index as f64 / f64::from(sample_rate_hz);
                let frequency = if evolving && index >= sample_count / 2 {
                    3_375.0
                } else {
                    562.5
                };
                (2.0 * PI * frequency * time).sin() * (-6.0 * time).exp() * 0.5
            })
            .collect();
        WavAudio {
            sample_format: "ieee-f32".to_owned(),
            sample_rate_hz,
            channel_count: 1,
            mono_samples,
        }
    }
}
