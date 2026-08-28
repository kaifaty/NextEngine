use serde::{Deserialize, Serialize};

use super::manifest::{MODEL_RIDGE, MeshDescriptor, SIGMA_GRID};

const FEATURE_COUNT: usize = 5;

#[derive(Clone, Serialize)]
pub(super) struct TrainingSample<'a> {
    pub(super) object_id: &'a str,
    pub(super) descriptor: &'a MeshDescriptor,
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
    pub(super) minimum_sigma_metres: f64,
    pub(super) maximum_sigma_metres: f64,
}

pub(super) fn fit(samples: &[TrainingSample<'_>]) -> Result<FittedModel, String> {
    if samples.len() < FEATURE_COUNT * 2 {
        return Err("shape model requires at least ten development objects".to_owned());
    }
    let raw = samples
        .iter()
        .map(|sample| raw_features(sample.descriptor))
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
            return Err("shape model development feature has zero variance".to_owned());
        }
    }
    let standardized = raw
        .iter()
        .map(|features| standardize(features, &means, &deviations))
        .collect::<Vec<_>>();
    let mut matrix = [[0.0_f64; FEATURE_COUNT]; FEATURE_COUNT];
    let mut rhs = [0.0_f64; FEATURE_COUNT];
    for ((features, sample), weight) in standardized
        .iter()
        .zip(samples)
        .zip(std::iter::repeat(1.0 / samples.len() as f64))
    {
        if !sample.target_sigma_metres.is_finite()
            || !(SIGMA_GRID[0]..=SIGMA_GRID[SIGMA_GRID.len() - 1])
                .contains(&sample.target_sigma_metres)
        {
            return Err("shape model target sigma is outside the frozen grid".to_owned());
        }
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
    let coefficients = solve(matrix, rhs)?;
    Ok(FittedModel {
        id: "mesh-bbox-conditioned-object-rbf-bandwidth-v1".to_owned(),
        feature_names: [
            "intercept".to_owned(),
            "ln_bbox_diagonal_m".to_owned(),
            "ln_minor_to_major_extent_ratio".to_owned(),
            "ln_middle_to_major_extent_ratio".to_owned(),
            "impact_radius_over_diagonal".to_owned(),
        ],
        development_means: means,
        development_population_standard_deviations: deviations,
        coefficients,
        ridge_lambda_non_intercept: MODEL_RIDGE,
        minimum_sigma_metres: SIGMA_GRID[0],
        maximum_sigma_metres: SIGMA_GRID[SIGMA_GRID.len() - 1],
    })
}

impl FittedModel {
    pub(super) fn predict(&self, descriptor: &MeshDescriptor) -> Result<f64, String> {
        if self.id != "mesh-bbox-conditioned-object-rbf-bandwidth-v1"
            || self.feature_names
                != [
                    "intercept".to_owned(),
                    "ln_bbox_diagonal_m".to_owned(),
                    "ln_minor_to_major_extent_ratio".to_owned(),
                    "ln_middle_to_major_extent_ratio".to_owned(),
                    "impact_radius_over_diagonal".to_owned(),
                ]
            || self.ridge_lambda_non_intercept != MODEL_RIDGE
            || self.minimum_sigma_metres != SIGMA_GRID[0]
            || self.maximum_sigma_metres != SIGMA_GRID[SIGMA_GRID.len() - 1]
        {
            return Err("shape model lineage changed".to_owned());
        }
        if self
            .development_population_standard_deviations
            .iter()
            .any(|value| !value.is_finite() || *value <= 1.0e-12)
            || self.coefficients.iter().any(|value| !value.is_finite())
        {
            return Err("shape model contains non-finite coefficients".to_owned());
        }
        let features = raw_features(descriptor)?;
        let features = standardize(
            &features,
            &self.development_means,
            &self.development_population_standard_deviations,
        );
        let log_sigma = self
            .coefficients
            .iter()
            .zip(features)
            .map(|(coefficient, feature)| coefficient * feature)
            .sum::<f64>();
        let sigma = log_sigma.exp();
        if !sigma.is_finite() {
            return Err("shape model prediction is non-finite".to_owned());
        }
        Ok(sigma.clamp(self.minimum_sigma_metres, self.maximum_sigma_metres))
    }
}

fn raw_features(descriptor: &MeshDescriptor) -> Result<[f64; FEATURE_COUNT], String> {
    let values = [
        descriptor.bbox_diagonal_m,
        descriptor.minor_to_major_extent_ratio,
        descriptor.middle_to_major_extent_ratio,
        descriptor.impact_radius_over_diagonal,
    ];
    if values
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err("shape model descriptor is not finite and positive".to_owned());
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
            return Err("shape model ridge system is singular".to_owned());
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
    if rhs.iter().any(|value| !value.is_finite()) {
        return Err("shape model ridge solution is non-finite".to_owned());
    }
    Ok(rhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(diagonal: f64, minor: f64, middle: f64, radius: f64) -> MeshDescriptor {
        MeshDescriptor {
            bbox_min_m: vec![0.0; 3],
            bbox_max_m: vec![1.0; 3],
            bbox_extents_m: vec![1.0; 3],
            bbox_diagonal_m: diagonal,
            minor_to_major_extent_ratio: minor,
            middle_to_major_extent_ratio: middle,
            impact_bbox_unit: vec![0.5; 3],
            impact_radius_over_diagonal: radius,
        }
    }

    #[test]
    fn fitted_model_is_finite_and_clamped() {
        let descriptors = (0..10)
            .map(|index| {
                descriptor(
                    0.15 + index as f64 * 0.02,
                    0.2 + index as f64 * 0.03,
                    0.5 + index as f64 * 0.02,
                    0.1 + index as f64 * 0.01,
                )
            })
            .collect::<Vec<_>>();
        let samples = descriptors
            .iter()
            .enumerate()
            .map(|(index, descriptor)| TrainingSample {
                object_id: "test",
                descriptor,
                target_sigma_metres: SIGMA_GRID[index % SIGMA_GRID.len()],
            })
            .collect::<Vec<_>>();
        let model = fit(&samples).expect("fit model");
        let prediction = model.predict(&descriptors[3]).expect("predict sigma");
        assert!((SIGMA_GRID[0]..=SIGMA_GRID[SIGMA_GRID.len() - 1]).contains(&prediction));
    }
}
