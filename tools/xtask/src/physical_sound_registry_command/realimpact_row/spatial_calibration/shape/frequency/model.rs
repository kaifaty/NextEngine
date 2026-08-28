use std::collections::BTreeMap;
use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use super::super::manifest::{MODEL_RIDGE, MeshDescriptor, SIGMA_GRID};

const FEATURE_COUNT: usize = 5;
pub(super) const SPEED_OF_SOUND: f64 = 343.0;

pub(super) struct TrainingSample<'a> {
    pub(super) object_id: &'a str,
    pub(super) descriptor: &'a MeshDescriptor,
    pub(super) frequency_hz: f64,
    pub(super) target_sigma_metres: f64,
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct FittedModel {
    pub(super) id: String,
    pub(super) feature_names: [String; FEATURE_COUNT],
    pub(super) development_means: [f64; FEATURE_COUNT - 1],
    pub(super) development_population_standard_deviations: [f64; FEATURE_COUNT - 1],
    pub(super) coefficients: [f64; FEATURE_COUNT],
    pub(super) ridge_lambda_non_intercept: f64,
    pub(super) speed_of_sound_metres_per_second: f64,
    pub(super) minimum_sigma_metres: f64,
    pub(super) maximum_sigma_metres: f64,
}

pub(super) fn fit(samples: &[TrainingSample<'_>]) -> Result<FittedModel, String> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for sample in samples {
        *counts.entry(sample.object_id).or_default() += 1;
    }
    if counts.len() != 12 || samples.len() < 12 * 12 {
        return Err("frequency model requires twelve development objects and modes".to_owned());
    }
    let raw = samples
        .iter()
        .map(|sample| raw_features(sample.descriptor, sample.frequency_hz))
        .collect::<Result<Vec<_>, String>>()?;
    let mut means = [0.0_f64; FEATURE_COUNT - 1];
    for features in &raw {
        for index in 0..FEATURE_COUNT - 1 {
            means[index] += features[index + 1] / raw.len() as f64;
        }
    }
    let mut deviations = [0.0_f64; FEATURE_COUNT - 1];
    for features in &raw {
        for index in 0..FEATURE_COUNT - 1 {
            deviations[index] += (features[index + 1] - means[index]).powi(2);
        }
    }
    for deviation in &mut deviations {
        *deviation = (*deviation / raw.len() as f64).sqrt();
        if !deviation.is_finite() || *deviation <= 1.0e-12 {
            return Err("frequency model development feature has zero variance".to_owned());
        }
    }
    let mut matrix = [[0.0_f64; FEATURE_COUNT]; FEATURE_COUNT];
    let mut rhs = [0.0_f64; FEATURE_COUNT];
    for (features, sample) in raw.iter().zip(samples) {
        if !SIGMA_GRID.contains(&sample.target_sigma_metres) {
            return Err("frequency model target is outside the frozen sigma grid".to_owned());
        }
        let features = standardize(features, &means, &deviations);
        let object_modes = counts[sample.object_id];
        let weight = 1.0 / counts.len() as f64 / object_modes as f64;
        let target = sample.target_sigma_metres.ln();
        for row in 0..FEATURE_COUNT {
            rhs[row] += weight * features[row] * target;
            for column in 0..FEATURE_COUNT {
                matrix[row][column] += weight * features[row] * features[column];
            }
        }
    }
    for (index, row) in matrix.iter_mut().enumerate().skip(1) {
        row[index] += MODEL_RIDGE;
    }
    Ok(FittedModel {
        id: "acoustic-scale-conditioned-mode-rbf-bandwidth-v1".to_owned(),
        feature_names: [
            "intercept".to_owned(),
            "ln_dimensionless_kL".to_owned(),
            "ln_minor_to_major_extent_ratio".to_owned(),
            "ln_middle_to_major_extent_ratio".to_owned(),
            "impact_radius_over_diagonal".to_owned(),
        ],
        development_means: means,
        development_population_standard_deviations: deviations,
        coefficients: solve(matrix, rhs)?,
        ridge_lambda_non_intercept: MODEL_RIDGE,
        speed_of_sound_metres_per_second: SPEED_OF_SOUND,
        minimum_sigma_metres: SIGMA_GRID[0],
        maximum_sigma_metres: SIGMA_GRID[SIGMA_GRID.len() - 1],
    })
}

impl FittedModel {
    pub(super) fn predict(
        &self,
        descriptor: &MeshDescriptor,
        frequency_hz: f64,
    ) -> Result<f64, String> {
        if self.id != "acoustic-scale-conditioned-mode-rbf-bandwidth-v1"
            || self.ridge_lambda_non_intercept != MODEL_RIDGE
            || self.speed_of_sound_metres_per_second != SPEED_OF_SOUND
            || self.minimum_sigma_metres != SIGMA_GRID[0]
            || self.maximum_sigma_metres != SIGMA_GRID[SIGMA_GRID.len() - 1]
        {
            return Err("frequency model lineage changed".to_owned());
        }
        let raw = raw_features(descriptor, frequency_hz)?;
        let features = standardize(
            &raw,
            &self.development_means,
            &self.development_population_standard_deviations,
        );
        let sigma = self
            .coefficients
            .iter()
            .zip(features)
            .map(|(coefficient, feature)| coefficient * feature)
            .sum::<f64>()
            .exp();
        if !sigma.is_finite() {
            return Err("frequency model prediction is non-finite".to_owned());
        }
        Ok(sigma.clamp(self.minimum_sigma_metres, self.maximum_sigma_metres))
    }
}

fn raw_features(
    descriptor: &MeshDescriptor,
    frequency_hz: f64,
) -> Result<[f64; FEATURE_COUNT], String> {
    let k_l = 2.0 * PI * frequency_hz * descriptor.bbox_diagonal_m / SPEED_OF_SOUND;
    let values = [
        k_l,
        descriptor.minor_to_major_extent_ratio,
        descriptor.middle_to_major_extent_ratio,
        descriptor.impact_radius_over_diagonal,
    ];
    if values
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err("frequency model descriptor is not finite and positive".to_owned());
    }
    Ok([
        1.0,
        values[0].ln(),
        values[1].ln(),
        values[2].ln(),
        values[3],
    ])
}

fn standardize(
    raw: &[f64; FEATURE_COUNT],
    means: &[f64; FEATURE_COUNT - 1],
    deviations: &[f64; FEATURE_COUNT - 1],
) -> [f64; FEATURE_COUNT] {
    let mut result = [1.0_f64; FEATURE_COUNT];
    for index in 1..FEATURE_COUNT {
        result[index] = (raw[index] - means[index - 1]) / deviations[index - 1];
    }
    result
}

fn solve(
    mut matrix: [[f64; FEATURE_COUNT]; FEATURE_COUNT],
    mut rhs: [f64; FEATURE_COUNT],
) -> Result<[f64; FEATURE_COUNT], String> {
    for pivot in 0..FEATURE_COUNT {
        let best = (pivot..FEATURE_COUNT)
            .max_by(|left, right| {
                matrix[*left][pivot]
                    .abs()
                    .total_cmp(&matrix[*right][pivot].abs())
            })
            .expect("non-empty pivot range");
        if matrix[best][pivot].abs() <= 1.0e-12 {
            return Err("frequency model ridge system is singular".to_owned());
        }
        matrix.swap(pivot, best);
        rhs.swap(pivot, best);
        let divisor = matrix[pivot][pivot];
        for value in matrix[pivot].iter_mut().skip(pivot) {
            *value /= divisor;
        }
        rhs[pivot] /= divisor;
        let pivot_row = matrix[pivot];
        for row in 0..FEATURE_COUNT {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for (column, value) in matrix[row].iter_mut().enumerate().skip(pivot) {
                *value -= factor * pivot_row[column];
            }
            rhs[row] -= factor * rhs[pivot];
        }
    }
    Ok(rhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitted_frequency_model_is_finite_and_clamped() {
        let descriptors = (0..12)
            .map(|index| descriptor(index as f64))
            .collect::<Vec<_>>();
        let object_ids = (0..12)
            .map(|index| format!("development-{index}"))
            .collect::<Vec<_>>();
        let mut samples = Vec::new();
        for object_index in 0..12 {
            for mode_index in 0..12 {
                samples.push(TrainingSample {
                    object_id: &object_ids[object_index],
                    descriptor: &descriptors[object_index],
                    frequency_hz: 280.0 + mode_index as f64 * 190.0,
                    target_sigma_metres: SIGMA_GRID[(object_index + mode_index) % SIGMA_GRID.len()],
                });
            }
        }

        let model = fit(&samples).expect("fit frequency model");
        let low = model
            .predict(&descriptors[0], 20.0)
            .expect("predict low frequency");
        let high = model
            .predict(&descriptors[11], 24_000.0)
            .expect("predict high frequency");
        assert!(low.is_finite() && (SIGMA_GRID[0]..=SIGMA_GRID[5]).contains(&low));
        assert!(high.is_finite() && (SIGMA_GRID[0]..=SIGMA_GRID[5]).contains(&high));
    }

    fn descriptor(index: f64) -> MeshDescriptor {
        let diagonal = 0.12 + index * 0.02;
        MeshDescriptor {
            bbox_min_m: vec![0.0, 0.0, 0.0],
            bbox_max_m: vec![diagonal, diagonal * 0.7, diagonal * 0.2],
            bbox_extents_m: vec![diagonal, diagonal * 0.7, diagonal * 0.2],
            bbox_diagonal_m: diagonal,
            minor_to_major_extent_ratio: 0.1 + index * 0.025,
            middle_to_major_extent_ratio: 0.55 + index * 0.025,
            impact_bbox_unit: vec![0.3, 0.4, 0.5],
            impact_radius_over_diagonal: 0.08 + index * 0.02,
        }
    }
}
