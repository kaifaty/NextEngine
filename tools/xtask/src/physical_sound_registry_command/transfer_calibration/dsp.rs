use std::f64::consts::{LN_2, PI};

use serde::Serialize;

const SAMPLE_EPSILON: f64 = 1.0e-24;
const ONSET_PEAK_FRACTION: f64 = 0.02;
const MINIMUM_FREQUENCY_HZ: f64 = 80.0;
const MAXIMUM_FREQUENCY_HZ: f64 = 12_000.0;
const PEAK_FLOOR_DB: f64 = -55.0;
const PEAK_SEPARATION_CENTS: f64 = 45.0;
const MATCH_TOLERANCE_CENTS: f64 = 60.0;
const TAIL_START_MS: usize = 750;
const DAMPING_START_MS: usize = 50;
const DAMPING_SPLIT_MS: usize = 750;
const DAMPING_END_MS: usize = 1_800;
const DAMPING_FFT_SIZE: usize = 4_096;
const DAMPING_HOP_SIZE: usize = 1_024;
const V2_MINIMUM_FREQUENCY_HZ: f64 = 250.0;
const V2_PEAK_FLOOR_DB: f64 = -45.0;
const V2_MINIMUM_SEPARATION_HZ: f64 = 12.0;
const V2_MATCH_TOLERANCE_CENTS: f64 = 40.0;
const V2_TAIL_START_MS: usize = 900;
const V2_DAMPING_SPLIT_MS: usize = 900;
const V2_DAMPING_END_MS: usize = 2_400;
const V2_DAMPING_FFT_SIZE: usize = 16_384;
const V2_DAMPING_HOP_SIZE: usize = 2_048;

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct CandidateProfile {
    pub(super) id: &'static str,
    pub(super) mode_limit: usize,
    pub(super) fft_size: usize,
}

pub(super) const CANDIDATES: [CandidateProfile; 3] = [
    CandidateProfile {
        id: "modal-8-fft32768-v1",
        mode_limit: 8,
        fft_size: 32_768,
    },
    CandidateProfile {
        id: "modal-12-fft32768-v1",
        mode_limit: 12,
        fft_size: 32_768,
    },
    CandidateProfile {
        id: "modal-16-fft65536-v1",
        mode_limit: 16,
        fft_size: 65_536,
    },
];

pub(super) const CANDIDATES_V2: [CandidateProfile; 3] = [
    CandidateProfile {
        id: "injective-modal-8-fft65536-v2",
        mode_limit: 8,
        fft_size: 65_536,
    },
    CandidateProfile {
        id: "injective-modal-12-fft65536-v2",
        mode_limit: 12,
        fft_size: 65_536,
    },
    CandidateProfile {
        id: "injective-modal-16-fft65536-v2",
        mode_limit: 16,
        fft_size: 65_536,
    },
];

#[derive(Clone, Debug, Serialize)]
pub(super) struct TransferAnalysis {
    pub(super) onset_sample: usize,
    pub(super) selected_mode_count: usize,
    pub(super) persistent_mode_count: usize,
    pub(super) persistent_mode_recall: f64,
    pub(super) median_frequency_error_cents: f64,
    pub(super) decaying_mode_fraction: f64,
    pub(super) median_tail_prediction_rmse_db: f64,
    pub(super) calibration_loss: f64,
    pub(super) modes: Vec<ModeAnalysis>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ModeAnalysis {
    pub(super) frequency_hz: f64,
    pub(super) relative_level_db: f64,
    pub(super) matched_tail_frequency_hz: Option<f64>,
    pub(super) frequency_error_cents: Option<f64>,
    pub(super) fit_decay_db_per_second: f64,
    pub(super) tail_decay_db_per_second: f64,
    pub(super) tail_prediction_rmse_db: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct DspProfileReport {
    onset_peak_fraction: f64,
    minimum_frequency_hz: f64,
    maximum_frequency_hz: f64,
    peak_floor_db: f64,
    peak_separation_cents: f64,
    match_tolerance_cents: f64,
    tail_start_ms: usize,
    damping_start_ms: usize,
    damping_split_ms: usize,
    damping_end_ms: usize,
    damping_fft_size: usize,
    damping_hop_size: usize,
}

pub(super) const fn profile_report() -> DspProfileReport {
    DspProfileReport {
        onset_peak_fraction: ONSET_PEAK_FRACTION,
        minimum_frequency_hz: MINIMUM_FREQUENCY_HZ,
        maximum_frequency_hz: MAXIMUM_FREQUENCY_HZ,
        peak_floor_db: PEAK_FLOOR_DB,
        peak_separation_cents: PEAK_SEPARATION_CENTS,
        match_tolerance_cents: MATCH_TOLERANCE_CENTS,
        tail_start_ms: TAIL_START_MS,
        damping_start_ms: DAMPING_START_MS,
        damping_split_ms: DAMPING_SPLIT_MS,
        damping_end_ms: DAMPING_END_MS,
        damping_fft_size: DAMPING_FFT_SIZE,
        damping_hop_size: DAMPING_HOP_SIZE,
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct DspProfileReportV2 {
    onset_peak_fraction: f64,
    minimum_frequency_hz: f64,
    maximum_frequency_hz: f64,
    peak_floor_db: f64,
    peak_separation_cents: f64,
    minimum_separation_hz: f64,
    match_tolerance_cents: f64,
    tail_matching: &'static str,
    tail_start_ms: usize,
    damping_start_ms: usize,
    damping_split_ms: usize,
    damping_end_ms: usize,
    damping_fft_size: usize,
    damping_hop_size: usize,
}

pub(super) const fn profile_report_v2() -> DspProfileReportV2 {
    DspProfileReportV2 {
        onset_peak_fraction: ONSET_PEAK_FRACTION,
        minimum_frequency_hz: V2_MINIMUM_FREQUENCY_HZ,
        maximum_frequency_hz: MAXIMUM_FREQUENCY_HZ,
        peak_floor_db: V2_PEAK_FLOOR_DB,
        peak_separation_cents: PEAK_SEPARATION_CENTS,
        minimum_separation_hz: V2_MINIMUM_SEPARATION_HZ,
        match_tolerance_cents: V2_MATCH_TOLERANCE_CENTS,
        tail_matching: "minimum-error greedy one-to-one assignment",
        tail_start_ms: V2_TAIL_START_MS,
        damping_start_ms: DAMPING_START_MS,
        damping_split_ms: V2_DAMPING_SPLIT_MS,
        damping_end_ms: V2_DAMPING_END_MS,
        damping_fft_size: V2_DAMPING_FFT_SIZE,
        damping_hop_size: V2_DAMPING_HOP_SIZE,
    }
}

pub(super) fn decode_f32le(bytes: &[u8]) -> Result<Vec<f64>, String> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(4) {
        return Err("transfer payload must contain complete non-empty f32_le samples".to_owned());
    }
    bytes
        .chunks_exact(4)
        .enumerate()
        .map(|(index, bytes)| {
            let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            if value.is_finite() {
                Ok(f64::from(value))
            } else {
                Err(format!("transfer sample {index} is non-finite"))
            }
        })
        .collect()
}

pub(super) fn analyze(
    samples: &[f64],
    sample_rate_hz: u32,
    profile: CandidateProfile,
) -> Result<TransferAnalysis, String> {
    if sample_rate_hz != 48_000 {
        return Err(format!(
            "normalized transfer calibration requires 48000 Hz, got {sample_rate_hz}"
        ));
    }
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    if peak <= 1.0e-12 {
        return Err("transfer payload has no usable signal".to_owned());
    }
    let onset = samples
        .iter()
        .position(|sample| sample.abs() >= peak * ONSET_PEAK_FRACTION)
        .ok_or_else(|| "transfer onset was not found".to_owned())?;
    let required_end = onset
        .checked_add(milliseconds_to_samples(DAMPING_END_MS, sample_rate_hz))
        .and_then(|value| value.checked_add(DAMPING_FFT_SIZE))
        .ok_or_else(|| "transfer analysis window overflow".to_owned())?;
    if required_end > samples.len() {
        return Err(format!(
            "transfer is too short for the frozen analysis window: need {required_end}, got {}",
            samples.len()
        ));
    }

    let fit_peaks = modal_peaks(samples, onset, sample_rate_hz, profile)?;
    let tail_start = onset + milliseconds_to_samples(TAIL_START_MS, sample_rate_hz);
    let tail_peaks = modal_peaks(samples, tail_start, sample_rate_hz, profile)?;
    let mut modes = Vec::with_capacity(fit_peaks.len());
    for peak in &fit_peaks {
        let matched = nearest_match(peak.frequency_hz, &tail_peaks);
        let levels = modal_level_track(samples, onset, sample_rate_hz, peak.frequency_hz)?;
        let split_seconds = DAMPING_SPLIT_MS as f64 / 1_000.0;
        let fit = levels
            .iter()
            .copied()
            .filter(|(time, _)| *time < split_seconds)
            .collect::<Vec<_>>();
        let tail = levels
            .iter()
            .copied()
            .filter(|(time, _)| *time >= split_seconds)
            .collect::<Vec<_>>();
        let fit_slope = linear_slope(&fit)?;
        let tail_slope = linear_slope(&tail)?;
        let tail_prediction_rmse_db = anchored_tail_rmse(&tail, fit_slope)?;
        let (matched_tail_frequency_hz, frequency_error_cents) = matched
            .map(|value| {
                (
                    Some(value.frequency_hz),
                    Some(cents_between(peak.frequency_hz, value.frequency_hz).abs()),
                )
            })
            .unwrap_or((None, None));
        modes.push(ModeAnalysis {
            frequency_hz: peak.frequency_hz,
            relative_level_db: peak.relative_level_db,
            matched_tail_frequency_hz,
            frequency_error_cents,
            fit_decay_db_per_second: fit_slope,
            tail_decay_db_per_second: tail_slope,
            tail_prediction_rmse_db,
        });
    }
    let persistent_mode_count = modes
        .iter()
        .filter(|mode| mode.matched_tail_frequency_hz.is_some())
        .count();
    let selected_mode_count = modes.len();
    let persistent_mode_recall = persistent_mode_count as f64 / selected_mode_count as f64;
    let median_frequency_error_cents = median(
        modes
            .iter()
            .filter_map(|mode| mode.frequency_error_cents)
            .collect(),
    )
    .unwrap_or(MATCH_TOLERANCE_CENTS * 2.0);
    let decaying_mode_fraction = modes
        .iter()
        .filter(|mode| mode.fit_decay_db_per_second < -1.0)
        .count() as f64
        / selected_mode_count as f64;
    let median_tail_prediction_rmse_db = median(
        modes
            .iter()
            .map(|mode| mode.tail_prediction_rmse_db)
            .collect(),
    )
    .unwrap_or(120.0);
    let calibration_loss = 4.0 * (1.0 - persistent_mode_recall)
        + (median_frequency_error_cents / MATCH_TOLERANCE_CENTS).min(2.0)
        + 2.0 * (1.0 - decaying_mode_fraction)
        + (median_tail_prediction_rmse_db / 20.0).min(3.0);
    Ok(TransferAnalysis {
        onset_sample: onset,
        selected_mode_count,
        persistent_mode_count,
        persistent_mode_recall,
        median_frequency_error_cents,
        decaying_mode_fraction,
        median_tail_prediction_rmse_db,
        calibration_loss,
        modes,
    })
}

pub(super) fn analyze_v2(
    samples: &[f64],
    sample_rate_hz: u32,
    profile: CandidateProfile,
) -> Result<TransferAnalysis, String> {
    if sample_rate_hz != 48_000 {
        return Err(format!(
            "normalized transfer calibration requires 48000 Hz, got {sample_rate_hz}"
        ));
    }
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    if peak <= 1.0e-12 {
        return Err("transfer payload has no usable signal".to_owned());
    }
    let onset = samples
        .iter()
        .position(|sample| sample.abs() >= peak * ONSET_PEAK_FRACTION)
        .ok_or_else(|| "transfer onset was not found".to_owned())?;
    let required_end = onset
        .checked_add(milliseconds_to_samples(V2_DAMPING_END_MS, sample_rate_hz))
        .and_then(|value| value.checked_add(V2_DAMPING_FFT_SIZE))
        .ok_or_else(|| "transfer V2 analysis window overflow".to_owned())?;
    if required_end > samples.len() {
        return Err(format!(
            "transfer is too short for the frozen V2 window: need {required_end}, got {}",
            samples.len()
        ));
    }

    let fit_peaks = modal_peaks_v2(samples, onset, sample_rate_hz, profile)?;
    let tail_start = onset + milliseconds_to_samples(V2_TAIL_START_MS, sample_rate_hz);
    let tail_peaks = modal_peaks_v2(samples, tail_start, sample_rate_hz, profile)?;
    let matches = injective_matches(&fit_peaks, &tail_peaks);
    let level_tracks = modal_level_tracks_v2(samples, onset, sample_rate_hz, &fit_peaks)?;
    let mut modes = Vec::with_capacity(fit_peaks.len());
    for ((peak, matched_index), levels) in fit_peaks.iter().zip(matches).zip(level_tracks) {
        let matched = matched_index.map(|index| &tail_peaks[index]);
        let split_seconds = V2_DAMPING_SPLIT_MS as f64 / 1_000.0;
        let fit = levels
            .iter()
            .copied()
            .filter(|(time, _)| *time < split_seconds)
            .collect::<Vec<_>>();
        let tail = levels
            .iter()
            .copied()
            .filter(|(time, _)| *time >= split_seconds)
            .collect::<Vec<_>>();
        let fit_slope = linear_slope(&fit)?;
        let tail_slope = linear_slope(&tail)?;
        let tail_prediction_rmse_db = anchored_tail_rmse(&tail, fit_slope)?;
        let (matched_tail_frequency_hz, frequency_error_cents) = matched
            .map(|value| {
                (
                    Some(value.frequency_hz),
                    Some(cents_between(peak.frequency_hz, value.frequency_hz).abs()),
                )
            })
            .unwrap_or((None, None));
        modes.push(ModeAnalysis {
            frequency_hz: peak.frequency_hz,
            relative_level_db: peak.relative_level_db,
            matched_tail_frequency_hz,
            frequency_error_cents,
            fit_decay_db_per_second: fit_slope,
            tail_decay_db_per_second: tail_slope,
            tail_prediction_rmse_db,
        });
    }
    summarize(onset, modes, V2_MATCH_TOLERANCE_CENTS)
}

fn summarize(
    onset: usize,
    modes: Vec<ModeAnalysis>,
    match_tolerance_cents: f64,
) -> Result<TransferAnalysis, String> {
    let persistent_mode_count = modes
        .iter()
        .filter(|mode| mode.matched_tail_frequency_hz.is_some())
        .count();
    let selected_mode_count = modes.len();
    if selected_mode_count == 0 {
        return Err("modal analysis selected no modes".to_owned());
    }
    let persistent_mode_recall = persistent_mode_count as f64 / selected_mode_count as f64;
    let median_frequency_error_cents = median(
        modes
            .iter()
            .filter_map(|mode| mode.frequency_error_cents)
            .collect(),
    )
    .unwrap_or(match_tolerance_cents * 2.0);
    let decaying_mode_fraction = modes
        .iter()
        .filter(|mode| mode.fit_decay_db_per_second < -1.0)
        .count() as f64
        / selected_mode_count as f64;
    let median_tail_prediction_rmse_db = median(
        modes
            .iter()
            .map(|mode| mode.tail_prediction_rmse_db)
            .collect(),
    )
    .unwrap_or(120.0);
    let calibration_loss = 4.0 * (1.0 - persistent_mode_recall)
        + (median_frequency_error_cents / match_tolerance_cents).min(2.0)
        + 2.0 * (1.0 - decaying_mode_fraction)
        + (median_tail_prediction_rmse_db / 20.0).min(3.0);
    Ok(TransferAnalysis {
        onset_sample: onset,
        selected_mode_count,
        persistent_mode_count,
        persistent_mode_recall,
        median_frequency_error_cents,
        decaying_mode_fraction,
        median_tail_prediction_rmse_db,
        calibration_loss,
        modes,
    })
}

#[derive(Clone, Copy)]
struct SpectralPeak {
    frequency_hz: f64,
    relative_level_db: f64,
}

fn modal_peaks(
    samples: &[f64],
    start: usize,
    sample_rate_hz: u32,
    profile: CandidateProfile,
) -> Result<Vec<SpectralPeak>, String> {
    let power = power_spectrum(samples, start, profile.fft_size)?;
    let bin_hz = f64::from(sample_rate_hz) / profile.fft_size as f64;
    let first = (MINIMUM_FREQUENCY_HZ / bin_hz).ceil() as usize;
    let last = ((MAXIMUM_FREQUENCY_HZ / bin_hz).floor() as usize).min(power.len() - 2);
    let maximum = power[first..=last]
        .iter()
        .copied()
        .fold(SAMPLE_EPSILON, f64::max);
    let mut candidates = (first..=last)
        .filter(|index| power[*index] > power[*index - 1] && power[*index] >= power[*index + 1])
        .filter_map(|index| {
            let relative_level_db = 10.0 * (power[index].max(SAMPLE_EPSILON) / maximum).log10();
            if relative_level_db < PEAK_FLOOR_DB {
                return None;
            }
            let frequency_hz = interpolated_frequency(&power, index, bin_hz);
            Some(SpectralPeak {
                frequency_hz,
                relative_level_db,
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .relative_level_db
            .total_cmp(&left.relative_level_db)
            .then_with(|| left.frequency_hz.total_cmp(&right.frequency_hz))
    });
    let mut selected = Vec::<SpectralPeak>::new();
    for candidate in candidates {
        if selected.iter().all(|other| {
            cents_between(candidate.frequency_hz, other.frequency_hz).abs() >= PEAK_SEPARATION_CENTS
        }) {
            selected.push(candidate);
            if selected.len() == profile.mode_limit {
                break;
            }
        }
    }
    if selected.len() < 4 {
        return Err(format!(
            "candidate {} found fewer than four modal peaks",
            profile.id
        ));
    }
    selected.sort_by(|left, right| left.frequency_hz.total_cmp(&right.frequency_hz));
    Ok(selected)
}

fn modal_peaks_v2(
    samples: &[f64],
    start: usize,
    sample_rate_hz: u32,
    profile: CandidateProfile,
) -> Result<Vec<SpectralPeak>, String> {
    let power = power_spectrum(samples, start, profile.fft_size)?;
    let bin_hz = f64::from(sample_rate_hz) / profile.fft_size as f64;
    let first = (V2_MINIMUM_FREQUENCY_HZ / bin_hz).ceil() as usize;
    let last = ((MAXIMUM_FREQUENCY_HZ / bin_hz).floor() as usize).min(power.len() - 2);
    let maximum = power[first..=last]
        .iter()
        .copied()
        .fold(SAMPLE_EPSILON, f64::max);
    let mut candidates = (first..=last)
        .filter(|index| power[*index] > power[*index - 1] && power[*index] >= power[*index + 1])
        .filter_map(|index| {
            let relative_level_db = 10.0 * (power[index].max(SAMPLE_EPSILON) / maximum).log10();
            (relative_level_db >= V2_PEAK_FLOOR_DB).then_some(SpectralPeak {
                frequency_hz: interpolated_frequency(&power, index, bin_hz),
                relative_level_db,
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .relative_level_db
            .total_cmp(&left.relative_level_db)
            .then_with(|| left.frequency_hz.total_cmp(&right.frequency_hz))
    });
    let mut selected = Vec::<SpectralPeak>::new();
    for candidate in candidates {
        if selected.iter().all(|other| {
            (candidate.frequency_hz - other.frequency_hz).abs() >= V2_MINIMUM_SEPARATION_HZ
                && cents_between(candidate.frequency_hz, other.frequency_hz).abs()
                    >= PEAK_SEPARATION_CENTS
        }) {
            selected.push(candidate);
            if selected.len() == profile.mode_limit {
                break;
            }
        }
    }
    if selected.len() < 4 {
        return Err(format!(
            "candidate {} found fewer than four identifiable modal peaks",
            profile.id
        ));
    }
    selected.sort_by(|left, right| left.frequency_hz.total_cmp(&right.frequency_hz));
    Ok(selected)
}

fn injective_matches(fit: &[SpectralPeak], tail: &[SpectralPeak]) -> Vec<Option<usize>> {
    let mut pairs = Vec::new();
    for (fit_index, fit_peak) in fit.iter().enumerate() {
        for (tail_index, tail_peak) in tail.iter().enumerate() {
            let error = cents_between(fit_peak.frequency_hz, tail_peak.frequency_hz).abs();
            if error <= V2_MATCH_TOLERANCE_CENTS {
                pairs.push((error, fit_index, tail_index));
            }
        }
    }
    pairs.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    let mut matches = vec![None; fit.len()];
    let mut used_tail = vec![false; tail.len()];
    for (_, fit_index, tail_index) in pairs {
        if matches[fit_index].is_none() && !used_tail[tail_index] {
            matches[fit_index] = Some(tail_index);
            used_tail[tail_index] = true;
        }
    }
    matches
}

fn nearest_match(frequency_hz: f64, peaks: &[SpectralPeak]) -> Option<&SpectralPeak> {
    peaks
        .iter()
        .map(|peak| (cents_between(frequency_hz, peak.frequency_hz).abs(), peak))
        .filter(|(distance, _)| *distance <= MATCH_TOLERANCE_CENTS)
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, peak)| peak)
}

fn modal_level_track(
    samples: &[f64],
    onset: usize,
    sample_rate_hz: u32,
    frequency_hz: f64,
) -> Result<Vec<(f64, f64)>, String> {
    let first = milliseconds_to_samples(DAMPING_START_MS, sample_rate_hz);
    let last = milliseconds_to_samples(DAMPING_END_MS, sample_rate_hz);
    let bin_hz = f64::from(sample_rate_hz) / DAMPING_FFT_SIZE as f64;
    let bin = (frequency_hz / bin_hz).round() as usize;
    let mut levels = Vec::new();
    for offset in (first..=last).step_by(DAMPING_HOP_SIZE) {
        let power = power_spectrum(samples, onset + offset, DAMPING_FFT_SIZE)?;
        let lower = bin.saturating_sub(1);
        let upper = (bin + 1).min(power.len() - 1);
        let energy = power[lower..=upper].iter().sum::<f64>();
        let center = offset + DAMPING_FFT_SIZE / 2;
        levels.push((
            center as f64 / f64::from(sample_rate_hz),
            10.0 * energy.max(SAMPLE_EPSILON).log10(),
        ));
    }
    Ok(levels)
}

fn modal_level_tracks_v2(
    samples: &[f64],
    onset: usize,
    sample_rate_hz: u32,
    peaks: &[SpectralPeak],
) -> Result<Vec<Vec<(f64, f64)>>, String> {
    let first = milliseconds_to_samples(DAMPING_START_MS, sample_rate_hz);
    let last = milliseconds_to_samples(V2_DAMPING_END_MS, sample_rate_hz);
    let bin_hz = f64::from(sample_rate_hz) / V2_DAMPING_FFT_SIZE as f64;
    let bins = peaks
        .iter()
        .map(|peak| (peak.frequency_hz / bin_hz).round() as usize)
        .collect::<Vec<_>>();
    let mut tracks = vec![Vec::new(); peaks.len()];
    for offset in (first..=last).step_by(V2_DAMPING_HOP_SIZE) {
        let power = power_spectrum(samples, onset + offset, V2_DAMPING_FFT_SIZE)?;
        let center = offset + V2_DAMPING_FFT_SIZE / 2;
        let time = center as f64 / f64::from(sample_rate_hz);
        for (track, bin) in tracks.iter_mut().zip(&bins) {
            let lower = bin.saturating_sub(1);
            let upper = (bin + 1).min(power.len() - 1);
            let energy = power[lower..=upper].iter().sum::<f64>();
            track.push((time, 10.0 * energy.max(SAMPLE_EPSILON).log10()));
        }
    }
    Ok(tracks)
}

fn linear_slope(points: &[(f64, f64)]) -> Result<f64, String> {
    if points.len() < 3 {
        return Err("damping regression has fewer than three points".to_owned());
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
    if denominator <= 0.0 {
        return Err("damping regression has no time span".to_owned());
    }
    Ok(numerator / denominator)
}

fn anchored_tail_rmse(points: &[(f64, f64)], slope: f64) -> Result<f64, String> {
    let anchor = points
        .first()
        .copied()
        .ok_or_else(|| "tail prediction has no points".to_owned())?;
    let squared = points
        .iter()
        .map(|point| {
            let prediction = anchor.1 + slope * (point.0 - anchor.0);
            (point.1 - prediction).powi(2)
        })
        .sum::<f64>();
    Ok((squared / points.len() as f64).sqrt())
}

fn interpolated_frequency(power: &[f64], index: usize, bin_hz: f64) -> f64 {
    let left = power[index - 1].max(SAMPLE_EPSILON).ln();
    let center = power[index].max(SAMPLE_EPSILON).ln();
    let right = power[index + 1].max(SAMPLE_EPSILON).ln();
    let denominator = left - 2.0 * center + right;
    let offset = if denominator.abs() > 1.0e-12 {
        (0.5 * (left - right) / denominator).clamp(-0.5, 0.5)
    } else {
        0.0
    };
    (index as f64 + offset) * bin_hz
}

fn power_spectrum(samples: &[f64], start: usize, size: usize) -> Result<Vec<f64>, String> {
    if !size.is_power_of_two() || size < 2 {
        return Err("FFT size must be a power of two".to_owned());
    }
    let end = start
        .checked_add(size)
        .ok_or_else(|| "FFT window overflow".to_owned())?;
    if end > samples.len() {
        return Err("FFT window exceeds transfer samples".to_owned());
    }
    let mut real = vec![0.0_f64; size];
    let mut imaginary = vec![0.0_f64; size];
    let denominator = (size - 1) as f64;
    for (index, value) in real.iter_mut().enumerate() {
        let window = 0.5 - 0.5 * (2.0 * PI * index as f64 / denominator).cos();
        *value = samples[start + index] * window;
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

fn milliseconds_to_samples(milliseconds: usize, sample_rate_hz: u32) -> usize {
    milliseconds * sample_rate_hz as usize / 1_000
}

fn cents_between(left_hz: f64, right_hz: f64) -> f64 {
    1_200.0 * (right_hz / left_hz).ln() / LN_2
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[middle - 1] + values[middle]) * 0.5)
    } else {
        Some(values[middle])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damped_modal_analysis_is_deterministic_and_finds_decay() {
        let sample_rate = 48_000_u32;
        let mut samples = vec![0.0_f64; sample_rate as usize * 4];
        let frequencies = [
            233.0, 421.0, 773.0, 1_337.0, 2_219.0, 3_587.0, 5_203.0, 7_111.0,
        ];
        for (index, sample) in samples.iter_mut().enumerate().skip(240) {
            let time = (index - 240) as f64 / f64::from(sample_rate);
            *sample = frequencies
                .iter()
                .enumerate()
                .map(|(mode, frequency)| {
                    let decay = (-time * (1.4 + mode as f64 * 0.15)).exp();
                    decay * (2.0 * PI * frequency * time + mode as f64 * 0.17).sin()
                        / (mode + 1) as f64
                })
                .sum();
        }
        let first = analyze(&samples, sample_rate, CANDIDATES[0]).expect("analyze signal");
        let second = analyze(&samples, sample_rate, CANDIDATES[0]).expect("repeat analysis");
        assert_eq!(
            serde_json::to_vec(&first).expect("serialize first"),
            serde_json::to_vec(&second).expect("serialize second")
        );
        assert_eq!(first.selected_mode_count, 8);
        assert!(first.persistent_mode_recall >= 0.75);
        assert!(first.decaying_mode_fraction >= 0.75);
    }

    #[test]
    fn non_finite_transfer_sample_rejects() {
        let mut bytes = 1.0_f32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&f32::NAN.to_le_bytes());
        assert!(
            decode_f32le(&bytes)
                .expect_err("NaN rejects")
                .contains("non-finite")
        );
    }

    #[test]
    fn injective_matching_never_reuses_a_tail_peak() {
        let fit = [
            SpectralPeak {
                frequency_hz: 1_000.0,
                relative_level_db: 0.0,
            },
            SpectralPeak {
                frequency_hz: 1_010.0,
                relative_level_db: -1.0,
            },
        ];
        let tail = [SpectralPeak {
            frequency_hz: 1_005.0,
            relative_level_db: 0.0,
        }];
        let matches = injective_matches(&fit, &tail);
        assert_eq!(matches.iter().flatten().count(), 1);
    }
}
