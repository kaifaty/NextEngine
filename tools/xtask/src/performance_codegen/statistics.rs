use crate::performance::nearest_rank_percentile;

use super::CombinedBootstrapScenario;

pub(super) fn relative_change_basis_points(current: u64, baseline: u64) -> i64 {
    if baseline == 0 {
        return if current == 0 { 0 } else { i64::MAX };
    }
    let value = (i128::from(current) - i128::from(baseline))
        .saturating_mul(10_000)
        .checked_div(i128::from(baseline))
        .unwrap_or(i128::from(i64::MAX));
    i64::try_from(value).unwrap_or_else(|_| {
        if value.is_negative() {
            i64::MIN
        } else {
            i64::MAX
        }
    })
}

pub(super) fn bootstrap_change_interval(
    current: &[u64],
    baseline: &[u64],
    iterations: usize,
) -> Result<[i64; 2], String> {
    if current.is_empty() || baseline.is_empty() || iterations == 0 {
        return Err("CODEGEN_BOOTSTRAP_INPUT_INVALID".to_owned());
    }
    let mut rng = XorShift64::new(0x4e45_5854_434f_4445);
    let mut current_resample = vec![0_u64; current.len()];
    let mut baseline_resample = vec![0_u64; baseline.len()];
    let mut changes = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        for sample in &mut current_resample {
            *sample = current[rng.index(current.len())];
        }
        for sample in &mut baseline_resample {
            *sample = baseline[rng.index(baseline.len())];
        }
        let current_p95 = nearest_rank_percentile(&current_resample, 50)?;
        let baseline_p95 = nearest_rank_percentile(&baseline_resample, 50)?;
        changes.push(relative_change_basis_points(current_p95, baseline_p95));
    }
    changes.sort_unstable();
    Ok([
        nearest_rank_fraction_i64(&changes, 25, 1_000)?,
        nearest_rank_fraction_i64(&changes, 975, 1_000)?,
    ])
}

pub(super) fn bootstrap_combined_change_interval(
    scenarios: &[CombinedBootstrapScenario],
    iterations: usize,
) -> Result<[i64; 2], String> {
    if scenarios.is_empty() || iterations == 0 {
        return Err("CODEGEN_COMBINED_BOOTSTRAP_INPUT_INVALID".to_owned());
    }
    let mut rng = XorShift64::new(0x4e45_5854_434f_4445);
    let mut combined_samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let mut metric_changes = Vec::new();
        for scenario in scenarios {
            let Some(first_metric) = scenario.metrics.first() else {
                return Err("CODEGEN_COMBINED_BOOTSTRAP_METRICS_MISSING".to_owned());
            };
            let baseline_len = first_metric.baseline_run_p95s.len();
            let candidate_len = first_metric.candidate_run_p95s.len();
            if baseline_len == 0 || candidate_len == 0 {
                return Err("CODEGEN_COMBINED_BOOTSTRAP_INPUT_INVALID".to_owned());
            }
            let baseline_indices = (0..baseline_len)
                .map(|_| rng.index(baseline_len))
                .collect::<Vec<_>>();
            let candidate_indices = (0..candidate_len)
                .map(|_| rng.index(candidate_len))
                .collect::<Vec<_>>();
            for metric in &scenario.metrics {
                if metric.baseline_run_p95s.len() != baseline_len
                    || metric.candidate_run_p95s.len() != candidate_len
                {
                    return Err("CODEGEN_COMBINED_BOOTSTRAP_RUN_COUNT_MISMATCH".to_owned());
                }
                let baseline_resample = baseline_indices
                    .iter()
                    .map(|index| metric.baseline_run_p95s[*index])
                    .collect::<Vec<_>>();
                let candidate_resample = candidate_indices
                    .iter()
                    .map(|index| metric.candidate_run_p95s[*index])
                    .collect::<Vec<_>>();
                let baseline_median = nearest_rank_percentile(&baseline_resample, 50)?;
                let candidate_median = nearest_rank_percentile(&candidate_resample, 50)?;
                metric_changes.push(relative_change_basis_points(
                    candidate_median,
                    baseline_median,
                ));
            }
        }
        combined_samples.push(mean_i64(&metric_changes)?);
    }
    combined_samples.sort_unstable();
    Ok([
        nearest_rank_fraction_i64(&combined_samples, 25, 1_000)?,
        nearest_rank_fraction_i64(&combined_samples, 975, 1_000)?,
    ])
}

fn nearest_rank_fraction_i64(
    samples: &[i64],
    numerator: usize,
    denominator: usize,
) -> Result<i64, String> {
    if samples.is_empty() || numerator == 0 || numerator > denominator {
        return Err("CODEGEN_SIGNED_PERCENTILE_INPUT_INVALID".to_owned());
    }
    let rank = numerator
        .checked_mul(samples.len())
        .ok_or_else(|| "CODEGEN_SIGNED_PERCENTILE_OVERFLOW".to_owned())?
        .div_ceil(denominator);
    Ok(samples[rank.saturating_sub(1)])
}

pub(super) fn mean_i64(values: &[i64]) -> Result<i64, String> {
    if values.is_empty() {
        return Err("CODEGEN_MEAN_REQUIRES_VALUES".to_owned());
    }
    let sum = values
        .iter()
        .try_fold(0_i128, |sum, value| sum.checked_add(i128::from(*value)))
        .ok_or_else(|| "CODEGEN_MEAN_OVERFLOW".to_owned())?;
    i64::try_from(sum / values.len() as i128).map_err(|error| error.to_string())
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn index(&mut self, length: usize) -> usize {
        (self.next() as usize) % length
    }
}
