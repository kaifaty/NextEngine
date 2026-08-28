use std::f64::consts::PI;

use serde::Serialize;

pub(super) const LISTENER_COUNT: usize = 15;
pub(super) const REFERENCE_LISTENER: usize = 7;
pub(super) const ANCHOR_LISTENERS: [usize; 9] = [0, 2, 4, 6, 7, 8, 10, 12, 14];
pub(super) const HELD_LISTENERS: [usize; 6] = [1, 3, 5, 9, 11, 13];
pub(super) const WINDOW_SAMPLES: usize = 65_536;
pub(super) const ONSET_PEAK_FRACTION: f64 = 0.02;
const AMPLITUDE_EPSILON: f64 = 1.0e-24;
const RIDGE: f64 = 1.0e-3;

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct CandidateProfile {
    pub(super) id: &'static str,
    kind: CandidateKind,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum CandidateKind {
    Linear,
    PolynomialDegree2,
    PolynomialDegree3,
    RbfSigma052,
    RbfDynamic(f64),
}

pub(super) const CANDIDATES: [CandidateProfile; 4] = [
    CandidateProfile {
        id: "vertical-linear-v1",
        kind: CandidateKind::Linear,
    },
    CandidateProfile {
        id: "vertical-polynomial-degree2-ridge001-v1",
        kind: CandidateKind::PolynomialDegree2,
    },
    CandidateProfile {
        id: "vertical-polynomial-degree3-ridge001-v1",
        kind: CandidateKind::PolynomialDegree3,
    },
    CandidateProfile {
        id: "vertical-rbf-sigma052-ridge001-v1",
        kind: CandidateKind::RbfSigma052,
    },
];

#[derive(Clone, Copy, Debug)]
pub(super) struct ModeSeed {
    pub(super) frequency_hz: f64,
    pub(super) persistent: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct ComplexValue {
    pub(super) real: f64,
    pub(super) imaginary: f64,
}

impl ComplexValue {
    pub(super) const ZERO: Self = Self {
        real: 0.0,
        imaginary: 0.0,
    };

    pub(super) fn magnitude(self) -> f64 {
        self.real.hypot(self.imaginary)
    }

    pub(super) fn divide(self, divisor: Self) -> Result<Self, String> {
        let denominator = divisor
            .real
            .mul_add(divisor.real, divisor.imaginary.powi(2));
        if !denominator.is_finite() || denominator <= AMPLITUDE_EPSILON.powi(2) {
            return Err("spatial complex normalization reference is zero".to_owned());
        }
        let result = Self {
            real: (self.real * divisor.real + self.imaginary * divisor.imaginary) / denominator,
            imaginary: (self.imaginary * divisor.real - self.real * divisor.imaginary)
                / denominator,
        };
        result
            .is_finite()
            .then_some(result)
            .ok_or_else(|| "spatial complex normalization is non-finite".to_owned())
    }

    pub(super) fn is_finite(self) -> bool {
        self.real.is_finite() && self.imaginary.is_finite()
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Evaluation {
    pub(super) component_count: usize,
    pub(super) persistent_component_count: usize,
    pub(super) held_listener_count: usize,
    pub(super) median_abs_error_db: f64,
    pub(super) p90_abs_error_db: f64,
    pub(super) persistent_median_abs_error_db: f64,
    pub(super) constant_median_abs_error_db: f64,
    pub(super) median_error_ratio_to_constant: f64,
    pub(super) improved_component_fraction: f64,
    pub(super) calibration_loss: f64,
    pub(super) components: Vec<ComponentEvaluation>,
}

#[derive(Debug, Serialize)]
pub(super) struct ComponentEvaluation {
    frequency_hz: f64,
    persistent: bool,
    constant_median_abs_error_db: f64,
    candidate_median_abs_error_db: f64,
}

pub(super) fn evaluate(
    rows: &[Vec<f64>],
    modes: &[ModeSeed],
    sample_rate_hz: u32,
    candidate: CandidateProfile,
) -> Result<Evaluation, String> {
    evaluate_candidates(rows, modes, sample_rate_hz, &vec![candidate; modes.len()])
}

fn evaluate_candidates(
    rows: &[Vec<f64>],
    modes: &[ModeSeed],
    sample_rate_hz: u32,
    candidates: &[CandidateProfile],
) -> Result<Evaluation, String> {
    validate_projection_dimensions(rows, modes)?;
    if candidates.len() != modes.len() {
        return Err("spatial evaluation candidate lineage differs from modes".to_owned());
    }
    let window = hann_window();
    let mut participation = vec![vec![0.0_f64; LISTENER_COUNT]; modes.len()];
    for (listener_index, row) in rows.iter().enumerate() {
        let onset = onset(row)?;
        for (mode_index, mode) in modes.iter().enumerate() {
            participation[mode_index][listener_index] =
                projection_db(row, onset, &window, mode.frequency_hz, sample_rate_hz)?;
        }
    }
    for component in &mut participation {
        let reference = component[REFERENCE_LISTENER];
        for value in component {
            *value -= reference;
        }
    }

    let z = listener_z();
    let mut all_errors = Vec::new();
    let mut persistent_errors = Vec::new();
    let mut constant_errors = Vec::new();
    let mut improved = 0_usize;
    let mut components = Vec::new();
    for ((mode, target), candidate) in modes.iter().zip(participation).zip(candidates) {
        let predicted = predict(&z, &target, *candidate)?;
        let errors = HELD_LISTENERS
            .iter()
            .map(|index| (predicted[*index] - target[*index]).abs())
            .collect::<Vec<_>>();
        let baseline = HELD_LISTENERS
            .iter()
            .map(|index| target[*index].abs())
            .collect::<Vec<_>>();
        let candidate_median = median(&errors)?;
        let constant_median = median(&baseline)?;
        if candidate_median < constant_median {
            improved += 1;
        }
        if mode.persistent {
            persistent_errors.extend(errors.iter().copied());
        }
        all_errors.extend(errors);
        constant_errors.extend(baseline);
        components.push(ComponentEvaluation {
            frequency_hz: mode.frequency_hz,
            persistent: mode.persistent,
            constant_median_abs_error_db: constant_median,
            candidate_median_abs_error_db: candidate_median,
        });
    }
    build_evaluation(
        modes,
        all_errors,
        persistent_errors,
        constant_errors,
        improved,
        components,
    )
}

fn validate_projection_dimensions(rows: &[Vec<f64>], modes: &[ModeSeed]) -> Result<(), String> {
    if rows.len() != LISTENER_COUNT || modes.is_empty() {
        return Err("spatial evaluation dimensions are not frozen".to_owned());
    }
    if rows.iter().any(|row| row.len() < WINDOW_SAMPLES) {
        return Err("spatial evaluation row is shorter than the frozen window".to_owned());
    }
    Ok(())
}

fn build_evaluation(
    modes: &[ModeSeed],
    all_errors: Vec<f64>,
    persistent_errors: Vec<f64>,
    constant_errors: Vec<f64>,
    improved: usize,
    components: Vec<ComponentEvaluation>,
) -> Result<Evaluation, String> {
    if persistent_errors.is_empty() {
        return Err("spatial evaluation has no persistent component".to_owned());
    }
    let median_abs_error_db = median(&all_errors)?;
    let p90_abs_error_db = quantile_nearest_rank(&all_errors, 0.90)?;
    let persistent_median_abs_error_db = median(&persistent_errors)?;
    let constant_median_abs_error_db = median(&constant_errors)?;
    let median_error_ratio_to_constant = if constant_median_abs_error_db <= 1.0e-12 {
        if median_abs_error_db <= 1.0e-12 {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        median_abs_error_db / constant_median_abs_error_db
    };
    let improved_component_fraction = improved as f64 / modes.len() as f64;
    let calibration_loss = median_abs_error_db
        + 0.25 * p90_abs_error_db
        + 2.0 * (1.0 - improved_component_fraction)
        + 0.25 * persistent_median_abs_error_db;
    for value in [
        median_abs_error_db,
        p90_abs_error_db,
        persistent_median_abs_error_db,
        constant_median_abs_error_db,
        median_error_ratio_to_constant,
        improved_component_fraction,
        calibration_loss,
    ] {
        if !value.is_finite() {
            return Err("spatial evaluation produced a non-finite metric".to_owned());
        }
    }
    Ok(Evaluation {
        component_count: modes.len(),
        persistent_component_count: modes.iter().filter(|mode| mode.persistent).count(),
        held_listener_count: HELD_LISTENERS.len(),
        median_abs_error_db,
        p90_abs_error_db,
        persistent_median_abs_error_db,
        constant_median_abs_error_db,
        median_error_ratio_to_constant,
        improved_component_fraction,
        calibration_loss,
        components,
    })
}

pub(super) fn evaluate_rbf(
    rows: &[Vec<f64>],
    modes: &[ModeSeed],
    sample_rate_hz: u32,
    sigma: f64,
) -> Result<Evaluation, String> {
    if !sigma.is_finite() || sigma <= 0.0 {
        return Err("spatial RBF sigma must be finite and positive".to_owned());
    }
    evaluate(
        rows,
        modes,
        sample_rate_hz,
        CandidateProfile {
            id: "dynamic-rbf",
            kind: CandidateKind::RbfDynamic(sigma),
        },
    )
}

pub(super) fn evaluate_rbf_per_mode(
    rows: &[Vec<f64>],
    modes: &[ModeSeed],
    sample_rate_hz: u32,
    sigmas: &[f64],
) -> Result<Evaluation, String> {
    if sigmas.len() != modes.len()
        || sigmas
            .iter()
            .any(|sigma| !sigma.is_finite() || *sigma <= 0.0)
    {
        return Err("per-mode spatial RBF sigma lineage is invalid".to_owned());
    }
    let candidates = sigmas
        .iter()
        .map(|sigma| CandidateProfile {
            id: "dynamic-per-mode-rbf",
            kind: CandidateKind::RbfDynamic(*sigma),
        })
        .collect::<Vec<_>>();
    evaluate_candidates(rows, modes, sample_rate_hz, &candidates)
}

pub(super) fn component_candidate_median_errors(evaluation: &Evaluation) -> Vec<(f64, f64)> {
    evaluation
        .components
        .iter()
        .map(|component| {
            (
                component.frequency_hz,
                component.candidate_median_abs_error_db,
            )
        })
        .collect()
}

pub(super) fn relative_complex_participation(
    rows: &[Vec<f64>],
    modes: &[ModeSeed],
    sample_rate_hz: u32,
) -> Result<Vec<Vec<ComplexValue>>, String> {
    validate_projection_dimensions(rows, modes)?;
    let window = hann_window();
    let mut participation = vec![vec![ComplexValue::ZERO; LISTENER_COUNT]; modes.len()];
    for (listener_index, row) in rows.iter().enumerate() {
        let onset = onset(row)?;
        for (mode_index, mode) in modes.iter().enumerate() {
            participation[mode_index][listener_index] =
                projection_complex(row, onset, &window, mode.frequency_hz, sample_rate_hz)?;
        }
    }
    for component in &mut participation {
        let reference = component[REFERENCE_LISTENER];
        for value in component {
            *value = value.divide(reference)?;
        }
    }
    Ok(participation)
}

pub(super) fn relative_db_participation(
    rows: &[Vec<f64>],
    modes: &[ModeSeed],
    sample_rate_hz: u32,
) -> Result<Vec<Vec<f64>>, String> {
    validate_projection_dimensions(rows, modes)?;
    let window = hann_window();
    let mut participation = vec![vec![0.0_f64; LISTENER_COUNT]; modes.len()];
    for (listener_index, row) in rows.iter().enumerate() {
        let onset = onset(row)?;
        for (mode_index, mode) in modes.iter().enumerate() {
            participation[mode_index][listener_index] =
                projection_db(row, onset, &window, mode.frequency_hz, sample_rate_hz)?;
        }
    }
    for component in &mut participation {
        let reference = component[REFERENCE_LISTENER];
        for value in component {
            *value -= reference;
        }
    }
    Ok(participation)
}

pub(super) fn evaluate_db_predictions(
    modes: &[ModeSeed],
    target_db: &[Vec<f64>],
    predicted_db: &[Vec<f64>],
) -> Result<Evaluation, String> {
    if modes.is_empty()
        || target_db.len() != modes.len()
        || predicted_db.len() != modes.len()
        || target_db.iter().chain(predicted_db).any(|component| {
            component.len() != LISTENER_COUNT || component.iter().any(|value| !value.is_finite())
        })
    {
        return Err("spatial predicted-participation lineage is invalid".to_owned());
    }
    let mut all_errors = Vec::new();
    let mut persistent_errors = Vec::new();
    let mut constant_errors = Vec::new();
    let mut improved = 0_usize;
    let mut components = Vec::new();
    for ((mode, target), predicted) in modes.iter().zip(target_db).zip(predicted_db) {
        let errors = HELD_LISTENERS
            .iter()
            .map(|index| (predicted[*index] - target[*index]).abs())
            .collect::<Vec<_>>();
        let baseline = HELD_LISTENERS
            .iter()
            .map(|index| target[*index].abs())
            .collect::<Vec<_>>();
        let candidate_median = median(&errors)?;
        let constant_median = median(&baseline)?;
        if candidate_median < constant_median {
            improved += 1;
        }
        if mode.persistent {
            persistent_errors.extend(errors.iter().copied());
        }
        all_errors.extend(errors);
        constant_errors.extend(baseline);
        components.push(ComponentEvaluation {
            frequency_hz: mode.frequency_hz,
            persistent: mode.persistent,
            constant_median_abs_error_db: constant_median,
            candidate_median_abs_error_db: candidate_median,
        });
    }
    build_evaluation(
        modes,
        all_errors,
        persistent_errors,
        constant_errors,
        improved,
        components,
    )
}

pub(super) fn improved_component_fraction(
    candidate: &Evaluation,
    control: &Evaluation,
) -> Result<f64, String> {
    if candidate.components.len() != control.components.len() || candidate.components.is_empty() {
        return Err("spatial candidate/control component lineage differs".to_owned());
    }
    if candidate
        .components
        .iter()
        .zip(&control.components)
        .any(|(candidate, control)| candidate.frequency_hz != control.frequency_hz)
    {
        return Err("spatial candidate/control frequency lineage differs".to_owned());
    }
    let improved = candidate
        .components
        .iter()
        .zip(&control.components)
        .filter(|(candidate, control)| {
            candidate.candidate_median_abs_error_db < control.candidate_median_abs_error_db
        })
        .count();
    Ok(improved as f64 / candidate.components.len() as f64)
}

fn onset(samples: &[f64]) -> Result<usize, String> {
    let peak = samples
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0_f64, f64::max);
    if !peak.is_finite() || peak <= AMPLITUDE_EPSILON {
        return Err("spatial evaluation row has no finite signal".to_owned());
    }
    samples
        .iter()
        .position(|sample| sample.abs() >= peak * ONSET_PEAK_FRACTION)
        .ok_or_else(|| "spatial evaluation onset was not found".to_owned())
}

fn hann_window() -> Vec<f64> {
    (0..WINDOW_SAMPLES)
        .map(|index| 0.5 - 0.5 * (2.0 * PI * index as f64 / (WINDOW_SAMPLES - 1) as f64).cos())
        .collect()
}

fn projection_db(
    samples: &[f64],
    onset: usize,
    window: &[f64],
    frequency_hz: f64,
    sample_rate_hz: u32,
) -> Result<f64, String> {
    let projection = projection_complex(samples, onset, window, frequency_hz, sample_rate_hz)?;
    let magnitude = projection.magnitude().max(AMPLITUDE_EPSILON);
    let decibels = 20.0 * magnitude.log10();
    decibels
        .is_finite()
        .then_some(decibels)
        .ok_or_else(|| "spatial projection is non-finite".to_owned())
}

fn projection_complex(
    samples: &[f64],
    onset: usize,
    window: &[f64],
    frequency_hz: f64,
    sample_rate_hz: u32,
) -> Result<ComplexValue, String> {
    if !frequency_hz.is_finite()
        || frequency_hz <= 0.0
        || frequency_hz >= f64::from(sample_rate_hz) * 0.5
    {
        return Err("spatial component frequency is invalid".to_owned());
    }
    let omega = 2.0 * PI * frequency_hz / f64::from(sample_rate_hz);
    let step_cos = omega.cos();
    let step_sin = omega.sin();
    let mut phase_cos = 1.0_f64;
    let mut phase_sin = 0.0_f64;
    let mut real = 0.0_f64;
    let mut imaginary = 0.0_f64;
    for (index, weight) in window.iter().enumerate() {
        let sample = samples.get(onset + index).copied().unwrap_or(0.0);
        if !sample.is_finite() {
            return Err("spatial evaluation row contains a non-finite sample".to_owned());
        }
        let value = sample * weight;
        real += value * phase_cos;
        imaginary -= value * phase_sin;
        let next_cos = phase_cos * step_cos - phase_sin * step_sin;
        let next_sin = phase_sin * step_cos + phase_cos * step_sin;
        phase_cos = next_cos;
        phase_sin = next_sin;
        if index % 1_024 == 1_023 {
            let phase = omega * (index + 1) as f64;
            phase_cos = phase.cos();
            phase_sin = phase.sin();
        }
    }
    let result = ComplexValue { real, imaginary };
    result
        .is_finite()
        .then_some(result)
        .ok_or_else(|| "spatial projection is non-finite".to_owned())
}

fn listener_z() -> [f64; LISTENER_COUNT] {
    [
        -0.91, -0.78, -0.65, -0.52, -0.39, -0.26, -0.13, 0.0, 0.13, 0.26, 0.39, 0.52, 0.65, 0.78,
        0.91,
    ]
}

fn predict(
    z: &[f64; LISTENER_COUNT],
    target: &[f64],
    candidate: CandidateProfile,
) -> Result<[f64; LISTENER_COUNT], String> {
    match candidate.kind {
        CandidateKind::Linear => Ok(linear_prediction(z, target)),
        CandidateKind::PolynomialDegree2 => polynomial_prediction(z, target, 2),
        CandidateKind::PolynomialDegree3 => polynomial_prediction(z, target, 3),
        CandidateKind::RbfSigma052 => rbf_prediction(z, target, 0.52),
        CandidateKind::RbfDynamic(sigma) => rbf_prediction(z, target, sigma),
    }
}

fn linear_prediction(z: &[f64; LISTENER_COUNT], target: &[f64]) -> [f64; LISTENER_COUNT] {
    let mut predicted = [0.0_f64; LISTENER_COUNT];
    for index in 0..LISTENER_COUNT {
        if ANCHOR_LISTENERS.contains(&index) {
            predicted[index] = target[index];
            continue;
        }
        let right_position = ANCHOR_LISTENERS
            .iter()
            .position(|anchor| *anchor > index)
            .expect("frozen anchors contain the right endpoint");
        let left = ANCHOR_LISTENERS[right_position - 1];
        let right = ANCHOR_LISTENERS[right_position];
        let alpha = (z[index] - z[left]) / (z[right] - z[left]);
        predicted[index] = target[left] * (1.0 - alpha) + target[right] * alpha;
    }
    predicted
}

fn polynomial_prediction(
    z: &[f64; LISTENER_COUNT],
    target: &[f64],
    degree: usize,
) -> Result<[f64; LISTENER_COUNT], String> {
    let width = degree + 1;
    let mut matrix = vec![vec![0.0_f64; width]; width];
    let mut rhs = vec![0.0_f64; width];
    for index in ANCHOR_LISTENERS {
        let x = z[index] / 0.91;
        let powers = (0..width)
            .map(|power| x.powi(power as i32))
            .collect::<Vec<_>>();
        for row in 0..width {
            rhs[row] += powers[row] * target[index];
            for column in 0..width {
                matrix[row][column] += powers[row] * powers[column];
            }
        }
    }
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] += RIDGE;
    }
    let coefficients = solve(matrix, rhs)?;
    let mut predicted = [0.0_f64; LISTENER_COUNT];
    for (index, value) in predicted.iter_mut().enumerate() {
        let x = z[index] / 0.91;
        *value = coefficients
            .iter()
            .enumerate()
            .map(|(power, coefficient)| coefficient * x.powi(power as i32))
            .sum();
    }
    Ok(predicted)
}

fn rbf_prediction(
    z: &[f64; LISTENER_COUNT],
    target: &[f64],
    sigma: f64,
) -> Result<[f64; LISTENER_COUNT], String> {
    let width = ANCHOR_LISTENERS.len();
    let basis =
        |listener: usize, anchor: usize| (-0.5 * ((z[listener] - z[anchor]) / sigma).powi(2)).exp();
    let mut matrix = vec![vec![0.0_f64; width]; width];
    let mut rhs = vec![0.0_f64; width];
    for listener in ANCHOR_LISTENERS {
        for row in 0..width {
            let left = basis(listener, ANCHOR_LISTENERS[row]);
            rhs[row] += left * target[listener];
            for column in 0..width {
                matrix[row][column] += left * basis(listener, ANCHOR_LISTENERS[column]);
            }
        }
    }
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] += RIDGE;
    }
    let weights = solve(matrix, rhs)?;
    let mut predicted = [0.0_f64; LISTENER_COUNT];
    for (listener, value) in predicted.iter_mut().enumerate() {
        *value = ANCHOR_LISTENERS
            .iter()
            .enumerate()
            .map(|(index, anchor)| weights[index] * basis(listener, *anchor))
            .sum();
    }
    Ok(predicted)
}

fn solve(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Result<Vec<f64>, String> {
    let width = rhs.len();
    for pivot in 0..width {
        let best = (pivot..width)
            .max_by(|left, right| {
                matrix[*left][pivot]
                    .abs()
                    .total_cmp(&matrix[*right][pivot].abs())
            })
            .expect("non-empty pivot range");
        if matrix[best][pivot].abs() <= 1.0e-12 {
            return Err("spatial interpolation system is singular".to_owned());
        }
        matrix.swap(pivot, best);
        rhs.swap(pivot, best);
        let divisor = matrix[pivot][pivot];
        for value in matrix[pivot].iter_mut().skip(pivot) {
            *value /= divisor;
        }
        rhs[pivot] /= divisor;
        let pivot_values = matrix[pivot].clone();
        for row in 0..width {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for (column, value) in matrix[row].iter_mut().enumerate().skip(pivot) {
                *value -= factor * pivot_values[column];
            }
            rhs[row] -= factor * rhs[pivot];
        }
    }
    if rhs.iter().all(|value| value.is_finite()) {
        Ok(rhs)
    } else {
        Err("spatial interpolation solution is non-finite".to_owned())
    }
}

fn median(values: &[f64]) -> Result<f64, String> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    match sorted.len() {
        0 => Err("spatial metric has no values".to_owned()),
        length if length % 2 == 1 => Ok(sorted[length / 2]),
        length => Ok((sorted[length / 2 - 1] + sorted[length / 2]) * 0.5),
    }
}

fn quantile_nearest_rank(values: &[f64], quantile: f64) -> Result<f64, String> {
    if values.is_empty() || !(0.0..=1.0).contains(&quantile) {
        return Err("spatial quantile input is invalid".to_owned());
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (quantile * sorted.len() as f64).ceil().max(1.0) as usize;
    Ok(sorted[rank - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_candidate_reconstructs_linear_listener_field() {
        let z = listener_z();
        let target = z.map(|value| 3.0 * value - 2.0);
        let prediction = linear_prediction(&z, &target);
        for index in HELD_LISTENERS {
            assert!((prediction[index] - target[index]).abs() < 1.0e-12);
        }
    }

    #[test]
    fn ridge_candidates_are_finite_on_bounded_field() {
        let z = listener_z();
        let target = z.map(|value| (4.0 * value).sin());
        for candidate in CANDIDATES {
            let prediction = predict(&z, &target, candidate).expect("candidate fits");
            assert!(prediction.iter().all(|value| value.is_finite()));
        }
    }

    #[test]
    fn uniform_per_mode_sigma_matches_single_rbf_evaluation() {
        let frequency_hz = 1_200.0;
        let rows = (0..LISTENER_COUNT)
            .map(|listener| {
                let gain = 0.5 + listener as f64 / LISTENER_COUNT as f64;
                (0..WINDOW_SAMPLES)
                    .map(|sample| {
                        let time = sample as f64 / 48_000.0;
                        gain * (-18.0 * time).exp() * (2.0 * PI * frequency_hz * time).cos()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let modes = [ModeSeed {
            frequency_hz,
            persistent: true,
        }];

        let single = evaluate_rbf(&rows, &modes, 48_000, 0.52).expect("single RBF");
        let per_mode = evaluate_rbf_per_mode(&rows, &modes, 48_000, &[0.52]).expect("per-mode RBF");
        assert_eq!(
            serde_json::to_vec(&single).expect("serialize single"),
            serde_json::to_vec(&per_mode).expect("serialize per-mode")
        );
    }
}
