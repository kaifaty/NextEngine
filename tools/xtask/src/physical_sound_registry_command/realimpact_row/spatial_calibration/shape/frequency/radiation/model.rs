use std::f64::consts::PI;

use super::super::super::super::dsp::{
    ANCHOR_LISTENERS, ComplexValue, LISTENER_COUNT, ModeSeed, REFERENCE_LISTENER,
};
use super::super::super::manifest::MeshDescriptor;

const PARAMETER_COUNT: usize = 8;
const AMPLITUDE_EPSILON: f64 = 1.0e-24;

pub(super) fn predict(
    descriptor: &MeshDescriptor,
    modes: &[ModeSeed],
    participation: &[Vec<ComplexValue>],
    maximum_order: usize,
    ridge: f64,
    speed_of_sound: f64,
) -> Result<Vec<Vec<f64>>, String> {
    if maximum_order != 3
        || !ridge.is_finite()
        || ridge <= 0.0
        || !speed_of_sound.is_finite()
        || speed_of_sound <= 0.0
        || participation.len() != modes.len()
        || participation
            .iter()
            .any(|component| component.len() != LISTENER_COUNT)
    {
        return Err("modal-radiation model lineage is invalid".to_owned());
    }
    modes
        .iter()
        .zip(participation)
        .map(|(mode, target)| {
            predict_mode(descriptor, mode.frequency_hz, target, ridge, speed_of_sound)
        })
        .collect()
}

fn predict_mode(
    descriptor: &MeshDescriptor,
    frequency_hz: f64,
    target: &[ComplexValue],
    ridge: f64,
    speed_of_sound: f64,
) -> Result<Vec<f64>, String> {
    if target.len() != LISTENER_COUNT
        || target.iter().any(|value| !value.is_finite())
        || !frequency_hz.is_finite()
        || frequency_hz <= 0.0
    {
        return Err("modal-radiation mode target is invalid".to_owned());
    }
    let basis = listener_basis(descriptor, frequency_hz, speed_of_sound)?;
    let scales = column_scales(&basis)?;
    let mut normal = [[0.0_f64; PARAMETER_COUNT]; PARAMETER_COUNT];
    let mut rhs = [0.0_f64; PARAMETER_COUNT];
    for listener in ANCHOR_LISTENERS {
        let row = standardized_real_row(&basis[listener], &scales);
        accumulate(&mut normal, &mut rhs, &row, target[listener].real);
        let row = standardized_imaginary_row(&basis[listener], &scales);
        accumulate(&mut normal, &mut rhs, &row, target[listener].imaginary);
    }
    for (index, row) in normal.iter_mut().enumerate() {
        row[index] += ridge;
    }
    let parameters = solve(normal, rhs)?;
    let predicted = basis
        .iter()
        .map(|row| evaluate_row(row, &scales, &parameters))
        .collect::<Result<Vec<_>, String>>()?;
    let reference = predicted[REFERENCE_LISTENER];
    predicted
        .into_iter()
        .map(|value| {
            let normalized = value.divide(reference)?;
            let db = 20.0 * normalized.magnitude().max(AMPLITUDE_EPSILON).log10();
            db.is_finite()
                .then_some(db)
                .ok_or_else(|| "modal-radiation prediction is non-finite".to_owned())
        })
        .collect()
}

fn listener_basis(
    descriptor: &MeshDescriptor,
    frequency_hz: f64,
    speed_of_sound: f64,
) -> Result<Vec<[ComplexValue; 4]>, String> {
    if descriptor.bbox_min_m.len() != 3 || descriptor.bbox_max_m.len() != 3 {
        return Err("modal-radiation bbox must have three coordinates".to_owned());
    }
    let center = [
        (descriptor.bbox_min_m[0] + descriptor.bbox_max_m[0]) * 0.5,
        (descriptor.bbox_min_m[1] + descriptor.bbox_max_m[1]) * 0.5,
        (descriptor.bbox_min_m[2] + descriptor.bbox_max_m[2]) * 0.5,
    ];
    if center.iter().any(|value| !value.is_finite()) {
        return Err("modal-radiation bbox center is non-finite".to_owned());
    }
    let wave_number = 2.0 * PI * frequency_hz / speed_of_sound;
    (0..LISTENER_COUNT)
        .map(|listener| {
            let position = [0.23, -0.04345, -0.91 + listener as f64 / 14.0 * 1.82];
            let delta = [
                position[0] - center[0],
                position[1] - center[1],
                position[2] - center[2],
            ];
            let radius = delta.iter().map(|value| value.powi(2)).sum::<f64>().sqrt();
            if !radius.is_finite() || radius <= 1.0e-9 {
                return Err("modal-radiation listener is at the expansion center".to_owned());
            }
            let argument = wave_number * radius;
            let cosine = delta[2] / radius;
            let hankel = spherical_hankel(argument)?;
            let legendre = legendre_order_three(cosine)?;
            Ok(std::array::from_fn(|order| ComplexValue {
                real: hankel[order].real * legendre[order],
                imaginary: hankel[order].imaginary * legendre[order],
            }))
        })
        .collect()
}

fn spherical_hankel(argument: f64) -> Result<[ComplexValue; 4], String> {
    if !argument.is_finite() || argument <= 1.0e-6 {
        return Err("modal-radiation wave argument is invalid".to_owned());
    }
    let sine = argument.sin();
    let cosine = argument.cos();
    let mut result = [ComplexValue::ZERO; 4];
    result[0] = ComplexValue {
        real: sine / argument,
        imaginary: cosine / argument,
    };
    result[1] = ComplexValue {
        real: sine / argument.powi(2) - cosine / argument,
        imaginary: cosine / argument.powi(2) + sine / argument,
    };
    for order in 1..3 {
        let factor = (2 * order + 1) as f64 / argument;
        result[order + 1] = ComplexValue {
            real: factor * result[order].real - result[order - 1].real,
            imaginary: factor * result[order].imaginary - result[order - 1].imaginary,
        };
    }
    if result.iter().all(|value| value.is_finite()) {
        Ok(result)
    } else {
        Err("modal-radiation Hankel basis is non-finite".to_owned())
    }
}

fn legendre_order_three(cosine: f64) -> Result<[f64; 4], String> {
    if !cosine.is_finite() || !(-1.0..=1.0).contains(&cosine) {
        return Err("modal-radiation polar cosine is invalid".to_owned());
    }
    Ok([
        1.0,
        cosine,
        0.5 * (3.0 * cosine.powi(2) - 1.0),
        0.5 * (5.0 * cosine.powi(3) - 3.0 * cosine),
    ])
}

fn column_scales(basis: &[[ComplexValue; 4]]) -> Result<[f64; 4], String> {
    let scales = std::array::from_fn(|order| {
        (ANCHOR_LISTENERS
            .iter()
            .map(|listener| basis[*listener][order].magnitude().powi(2))
            .sum::<f64>()
            / ANCHOR_LISTENERS.len() as f64)
            .sqrt()
    });
    if scales
        .iter()
        .all(|scale| scale.is_finite() && *scale > 1.0e-12)
    {
        Ok(scales)
    } else {
        Err("modal-radiation basis column has zero scale".to_owned())
    }
}

fn standardized_real_row(basis: &[ComplexValue; 4], scales: &[f64; 4]) -> [f64; 8] {
    std::array::from_fn(|index| {
        if index < 4 {
            basis[index].real / scales[index]
        } else {
            -basis[index - 4].imaginary / scales[index - 4]
        }
    })
}

fn standardized_imaginary_row(basis: &[ComplexValue; 4], scales: &[f64; 4]) -> [f64; 8] {
    std::array::from_fn(|index| {
        if index < 4 {
            basis[index].imaginary / scales[index]
        } else {
            basis[index - 4].real / scales[index - 4]
        }
    })
}

fn accumulate(
    normal: &mut [[f64; PARAMETER_COUNT]; PARAMETER_COUNT],
    rhs: &mut [f64; PARAMETER_COUNT],
    row: &[f64; PARAMETER_COUNT],
    target: f64,
) {
    for output in 0..PARAMETER_COUNT {
        rhs[output] += row[output] * target;
        for input in 0..PARAMETER_COUNT {
            normal[output][input] += row[output] * row[input];
        }
    }
}

fn evaluate_row(
    basis: &[ComplexValue; 4],
    scales: &[f64; 4],
    parameters: &[f64; PARAMETER_COUNT],
) -> Result<ComplexValue, String> {
    let mut result = ComplexValue::ZERO;
    for order in 0..4 {
        let coefficient = ComplexValue {
            real: parameters[order],
            imaginary: parameters[order + 4],
        };
        let value = ComplexValue {
            real: basis[order].real / scales[order],
            imaginary: basis[order].imaginary / scales[order],
        };
        result.real += value.real * coefficient.real - value.imaginary * coefficient.imaginary;
        result.imaginary += value.real * coefficient.imaginary + value.imaginary * coefficient.real;
    }
    result
        .is_finite()
        .then_some(result)
        .ok_or_else(|| "modal-radiation basis evaluation is non-finite".to_owned())
}

fn solve(
    mut matrix: [[f64; PARAMETER_COUNT]; PARAMETER_COUNT],
    mut rhs: [f64; PARAMETER_COUNT],
) -> Result<[f64; PARAMETER_COUNT], String> {
    for pivot in 0..PARAMETER_COUNT {
        let best = (pivot..PARAMETER_COUNT)
            .max_by(|left, right| {
                matrix[*left][pivot]
                    .abs()
                    .total_cmp(&matrix[*right][pivot].abs())
            })
            .expect("non-empty pivot range");
        if matrix[best][pivot].abs() <= 1.0e-12 {
            return Err("modal-radiation ridge system is singular".to_owned());
        }
        matrix.swap(pivot, best);
        rhs.swap(pivot, best);
        let divisor = matrix[pivot][pivot];
        for value in matrix[pivot].iter_mut().skip(pivot) {
            *value /= divisor;
        }
        rhs[pivot] /= divisor;
        let pivot_row = matrix[pivot];
        for row in 0..PARAMETER_COUNT {
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
    if rhs.iter().all(|value| value.is_finite()) {
        Ok(rhs)
    } else {
        Err("modal-radiation solution is non-finite".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::dsp::HELD_LISTENERS;
    use super::*;

    #[test]
    fn order_three_basis_reconstructs_its_own_field() {
        let descriptor = MeshDescriptor {
            bbox_min_m: vec![-0.05, -0.04, -0.06],
            bbox_max_m: vec![0.05, 0.04, 0.06],
            bbox_extents_m: vec![0.1, 0.08, 0.12],
            bbox_diagonal_m: 0.175,
            minor_to_major_extent_ratio: 0.67,
            middle_to_major_extent_ratio: 0.83,
            impact_bbox_unit: vec![0.2, 0.3, 0.7],
            impact_radius_over_diagonal: 0.25,
        };
        let frequency_hz = 1_800.0;
        let basis = listener_basis(&descriptor, frequency_hz, 343.0).expect("basis");
        let coefficients = [
            ComplexValue {
                real: 0.8,
                imaginary: -0.2,
            },
            ComplexValue {
                real: -0.3,
                imaginary: 0.4,
            },
            ComplexValue {
                real: 0.2,
                imaginary: 0.1,
            },
            ComplexValue {
                real: -0.1,
                imaginary: -0.05,
            },
        ];
        let raw = basis
            .iter()
            .map(|row| {
                let mut value = ComplexValue::ZERO;
                for order in 0..4 {
                    value.real += row[order].real * coefficients[order].real
                        - row[order].imaginary * coefficients[order].imaginary;
                    value.imaginary += row[order].real * coefficients[order].imaginary
                        + row[order].imaginary * coefficients[order].real;
                }
                value
            })
            .collect::<Vec<_>>();
        let reference = raw[REFERENCE_LISTENER];
        let target = raw
            .into_iter()
            .map(|value| value.divide(reference).expect("normalize"))
            .collect::<Vec<_>>();
        let predicted =
            predict_mode(&descriptor, frequency_hz, &target, 0.001, 343.0).expect("predict field");
        for listener in HELD_LISTENERS {
            let expected = 20.0 * target[listener].magnitude().max(AMPLITUDE_EPSILON).log10();
            assert!((predicted[listener] - expected).abs() < 0.05);
        }
    }
}
