use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AmplitudeEnvelopeReport {
    pub(crate) frame_count: usize,
    pub(crate) mean_abs_log_rms_slope_db: f64,
    pub(crate) stddev_log_rms_slope_db: f64,
    pub(crate) mean_abs_log_rms_curvature_db: f64,
    pub(crate) monotonicity_violation_fraction: f64,
    pub(crate) direction_change_fraction: f64,
    pub(crate) early_energy_fraction: f64,
    pub(crate) middle_energy_fraction: f64,
    pub(crate) late_energy_fraction: f64,
    pub(crate) early_to_late_energy_db: f64,
    pub(crate) energy_spectral_change_correlation: f64,
    pub(crate) mean_uncoupled_energy_change: f64,
}

pub(super) fn analyze(
    frame_rms: &[f64],
    adjacent_spectral_distance: &[f64],
) -> AmplitudeEnvelopeReport {
    const POSITIVE_SLOPE_TOLERANCE_DB: f64 = 0.75;

    let peak = frame_rms.iter().copied().fold(1.0e-12, f64::max);
    let log_rms = frame_rms
        .iter()
        .map(|rms| amplitude_db(rms / peak).clamp(-120.0, 0.0))
        .collect::<Vec<_>>();
    let slopes = log_rms
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect::<Vec<_>>();
    let curvature = slopes
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect::<Vec<_>>();
    let absolute_slopes = slopes.iter().map(|value| value.abs()).collect::<Vec<_>>();
    let absolute_curvature = curvature
        .iter()
        .map(|value| value.abs())
        .collect::<Vec<_>>();
    let direction_changes = slopes
        .windows(2)
        .filter(|pair| pair[0].signum() != pair[1].signum())
        .count();
    let frame_energy = frame_rms
        .iter()
        .map(|value| value * value)
        .collect::<Vec<_>>();
    let energy_total = frame_energy.iter().sum::<f64>().max(1.0e-24);
    let first_boundary = frame_energy.len().div_ceil(3);
    let second_boundary = (frame_energy.len() * 2).div_ceil(3);
    let early_energy = frame_energy[..first_boundary].iter().sum::<f64>();
    let middle_energy = frame_energy[first_boundary..second_boundary]
        .iter()
        .sum::<f64>();
    let late_energy = frame_energy[second_boundary..].iter().sum::<f64>();
    let normalized_energy_change = absolute_slopes
        .iter()
        .map(|value| (value / 24.0).clamp(0.0, 1.0))
        .collect::<Vec<_>>();
    let uncoupled = normalized_energy_change
        .iter()
        .zip(adjacent_spectral_distance)
        .map(|(energy, spectral)| energy * (1.0 - spectral.clamp(0.0, 1.0)))
        .collect::<Vec<_>>();

    AmplitudeEnvelopeReport {
        frame_count: frame_rms.len(),
        mean_abs_log_rms_slope_db: mean_stddev(&absolute_slopes).0,
        stddev_log_rms_slope_db: mean_stddev(&slopes).1,
        mean_abs_log_rms_curvature_db: mean_stddev(&absolute_curvature).0,
        monotonicity_violation_fraction: ratio_count(
            slopes
                .iter()
                .filter(|slope| **slope > POSITIVE_SLOPE_TOLERANCE_DB)
                .count(),
            slopes.len(),
        ),
        direction_change_fraction: ratio_count(direction_changes, slopes.len().saturating_sub(1)),
        early_energy_fraction: early_energy / energy_total,
        middle_energy_fraction: middle_energy / energy_total,
        late_energy_fraction: late_energy / energy_total,
        early_to_late_energy_db: 10.0
            * (early_energy.max(1.0e-24) / late_energy.max(1.0e-24)).log10(),
        energy_spectral_change_correlation: correlation(
            &normalized_energy_change,
            adjacent_spectral_distance,
        ),
        mean_uncoupled_energy_change: mean_stddev(&uncoupled).0,
    }
}

fn amplitude_db(value: f64) -> f64 {
    20.0 * value.max(1.0e-12).log10()
}

fn ratio_count(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
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

fn correlation(left: &[f64], right: &[f64]) -> f64 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let left_mean = mean_stddev(left).0;
    let right_mean = mean_stddev(right).0;
    let (covariance, left_variance, right_variance) = left.iter().zip(right).fold(
        (0.0_f64, 0.0_f64, 0.0_f64),
        |(covariance, left_variance, right_variance), (left, right)| {
            let left_delta = left - left_mean;
            let right_delta = right - right_mean;
            (
                covariance + left_delta * right_delta,
                left_variance + left_delta * left_delta,
                right_variance + right_delta * right_delta,
            )
        },
    );
    if left_variance <= 1.0e-24 || right_variance <= 1.0e-24 {
        0.0
    } else {
        (covariance / (left_variance * right_variance).sqrt()).clamp(-1.0, 1.0)
    }
}
