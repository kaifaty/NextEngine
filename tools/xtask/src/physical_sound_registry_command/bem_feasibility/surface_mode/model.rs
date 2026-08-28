use serde::{Deserialize, Serialize};

use super::{SourceCondition, SourceManifest};

pub(super) const ORDER_COUNT: usize = 4;
const PARAMETER_COUNT: usize = ORDER_COUNT * 2;

pub(super) fn fit_coefficients(
    rows: &[&SourceCondition],
    source_manifest: &SourceManifest,
    wave_number_radius: f64,
    ridge: f64,
) -> Result<FitResult, String> {
    if rows.len() < ORDER_COUNT || !ridge.is_finite() || ridge <= 0.0 {
        return Err("surface-mode cooker fit setup is invalid".to_owned());
    }
    let wave_number = wave_number_radius / source_manifest.fixture.radius_metres;
    let radius = source_manifest.fixture.radius_metres * rows[0].listener_radius_multiplier;
    let bases = rows
        .iter()
        .map(|row| {
            let direction = source_manifest.fixture.listener_directions[row.direction_index];
            multipole_basis(wave_number, radius, normalized_cosine(direction)?)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let scales = column_scales(&bases)?;
    let mut normal = [[0.0_f64; PARAMETER_COUNT]; PARAMETER_COUNT];
    let mut rhs = [0.0_f64; PARAMETER_COUNT];
    for (basis, row) in bases.iter().zip(rows) {
        let real = standardized_real_row(basis, &scales);
        accumulate(&mut normal, &mut rhs, &real, row.computed.real);
        let imaginary = standardized_imaginary_row(basis, &scales);
        accumulate(&mut normal, &mut rhs, &imaginary, row.computed.imaginary);
    }
    for (index, row) in normal.iter_mut().enumerate() {
        row[index] += ridge;
    }
    let parameters = solve(normal, rhs)?;
    let coefficients = std::array::from_fn(|order| ComplexValue {
        real: parameters[order] / scales[order],
        imaginary: parameters[order + ORDER_COUNT] / scales[order],
    });
    let max_peak_normalized_complex_error = bases
        .iter()
        .zip(rows)
        .map(|(basis, row)| {
            let predicted = evaluate(basis, &coefficients)?;
            Ok(predicted.subtract(row.computed).magnitude() / row.analytical_peak_magnitude)
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .max_by(f64::total_cmp)
        .ok_or_else(|| "surface-mode cooker fit is empty".to_owned())?;
    Ok(FitResult {
        coefficients,
        max_peak_normalized_complex_error,
    })
}

pub(super) fn multipole_basis(
    wave_number: f64,
    listener_radius: f64,
    cosine: f64,
) -> Result<[ComplexValue; ORDER_COUNT], String> {
    let argument = wave_number * listener_radius;
    let hankel = spherical_hankel_first_kind(argument)?;
    let legendre = legendre_order_three(cosine)?;
    Ok(std::array::from_fn(|order| {
        hankel[order].scale(legendre[order])
    }))
}

fn spherical_hankel_first_kind(argument: f64) -> Result<[ComplexValue; ORDER_COUNT], String> {
    if !argument.is_finite() || argument <= 1.0e-6 {
        return Err("surface-mode cooker wave argument is invalid".to_owned());
    }
    let sine = argument.sin();
    let cosine = argument.cos();
    let mut result = [ComplexValue::ZERO; ORDER_COUNT];
    result[0] = ComplexValue {
        real: sine / argument,
        imaginary: -cosine / argument,
    };
    result[1] = ComplexValue {
        real: sine / argument.powi(2) - cosine / argument,
        imaginary: -cosine / argument.powi(2) - sine / argument,
    };
    for order in 1..ORDER_COUNT - 1 {
        let factor = (2 * order + 1) as f64 / argument;
        result[order + 1] = result[order].scale(factor).subtract(result[order - 1]);
    }
    result
        .iter()
        .all(|value| value.is_finite())
        .then_some(result)
        .ok_or_else(|| "surface-mode cooker Hankel basis is non-finite".to_owned())
}

fn legendre_order_three(cosine: f64) -> Result<[f64; ORDER_COUNT], String> {
    if !cosine.is_finite() || !(-1.0..=1.0).contains(&cosine) {
        return Err("surface-mode cooker polar cosine is invalid".to_owned());
    }
    Ok([
        1.0,
        cosine,
        0.5 * (3.0 * cosine.powi(2) - 1.0),
        0.5 * (5.0 * cosine.powi(3) - 3.0 * cosine),
    ])
}

pub(super) fn normalized_cosine(direction: [f64; 3]) -> Result<f64, String> {
    let norm = direction
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if direction.iter().any(|value| !value.is_finite()) || norm <= 0.0 {
        return Err("surface-mode cooker direction is invalid".to_owned());
    }
    Ok(direction[2] / norm)
}

fn column_scales(bases: &[[ComplexValue; ORDER_COUNT]]) -> Result<[f64; ORDER_COUNT], String> {
    let scales = std::array::from_fn(|order| {
        (bases
            .iter()
            .map(|basis| basis[order].magnitude_squared())
            .sum::<f64>()
            / bases.len() as f64)
            .sqrt()
    });
    if scales
        .iter()
        .all(|scale| scale.is_finite() && *scale > 1.0e-18)
    {
        Ok(scales)
    } else {
        Err("surface-mode cooker basis column has zero scale".to_owned())
    }
}

fn standardized_real_row(
    basis: &[ComplexValue; ORDER_COUNT],
    scales: &[f64; ORDER_COUNT],
) -> [f64; PARAMETER_COUNT] {
    std::array::from_fn(|index| {
        if index < ORDER_COUNT {
            basis[index].real / scales[index]
        } else {
            -basis[index - ORDER_COUNT].imaginary / scales[index - ORDER_COUNT]
        }
    })
}

fn standardized_imaginary_row(
    basis: &[ComplexValue; ORDER_COUNT],
    scales: &[f64; ORDER_COUNT],
) -> [f64; PARAMETER_COUNT] {
    std::array::from_fn(|index| {
        if index < ORDER_COUNT {
            basis[index].imaginary / scales[index]
        } else {
            basis[index - ORDER_COUNT].real / scales[index - ORDER_COUNT]
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
        if matrix[best][pivot].abs() <= 1.0e-14 {
            return Err("surface-mode cooker ridge system is singular".to_owned());
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
    rhs.iter()
        .all(|value| value.is_finite())
        .then_some(rhs)
        .ok_or_else(|| "surface-mode cooker solution is non-finite".to_owned())
}

pub(super) fn evaluate(
    basis: &[ComplexValue; ORDER_COUNT],
    coefficients: &[ComplexValue; ORDER_COUNT],
) -> Result<ComplexValue, String> {
    let mut result = ComplexValue::ZERO;
    for order in 0..ORDER_COUNT {
        result = result.add(basis[order].multiply(coefficients[order]));
    }
    result
        .is_finite()
        .then_some(result)
        .ok_or_else(|| "surface-mode cooker prediction is non-finite".to_owned())
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(super) struct ComplexValue {
    pub(super) real: f64,
    pub(super) imaginary: f64,
}

impl ComplexValue {
    pub(super) const ZERO: Self = Self {
        real: 0.0,
        imaginary: 0.0,
    };

    pub(super) fn is_finite(self) -> bool {
        self.real.is_finite() && self.imaginary.is_finite()
    }

    pub(super) fn add(self, other: Self) -> Self {
        Self {
            real: self.real + other.real,
            imaginary: self.imaginary + other.imaginary,
        }
    }

    pub(super) fn subtract(self, other: Self) -> Self {
        Self {
            real: self.real - other.real,
            imaginary: self.imaginary - other.imaginary,
        }
    }

    pub(super) fn multiply(self, other: Self) -> Self {
        Self {
            real: self.real * other.real - self.imaginary * other.imaginary,
            imaginary: self.real * other.imaginary + self.imaginary * other.real,
        }
    }

    pub(super) fn conjugate(self) -> Self {
        Self {
            real: self.real,
            imaginary: -self.imaginary,
        }
    }

    fn scale(self, factor: f64) -> Self {
        Self {
            real: self.real * factor,
            imaginary: self.imaginary * factor,
        }
    }

    pub(super) fn magnitude_squared(self) -> f64 {
        self.real * self.real + self.imaginary * self.imaginary
    }

    pub(super) fn magnitude(self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    pub(super) fn phase(self) -> f64 {
        self.imaginary.atan2(self.real)
    }
}

pub(super) struct FitResult {
    pub(super) coefficients: [ComplexValue; ORDER_COUNT],
    pub(super) max_peak_normalized_complex_error: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outgoing_hankel_zero_order_matches_closed_form() {
        let argument = 2.75;
        let basis = spherical_hankel_first_kind(argument).expect("basis");
        assert!((basis[0].real - argument.sin() / argument).abs() < 1.0e-15);
        assert!((basis[0].imaginary + argument.cos() / argument).abs() < 1.0e-15);
    }

    #[test]
    fn order_three_fit_recovers_pure_quadrupole() {
        let directions = [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 0.0, -1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, -1.0],
        ];
        let wave_number = 7.5;
        let radius = 0.15;
        let bases = directions
            .map(|direction| {
                multipole_basis(
                    wave_number,
                    radius,
                    normalized_cosine(direction).expect("cosine"),
                )
                .expect("basis")
            })
            .to_vec();
        let scales = column_scales(&bases).expect("scales");
        let coefficient = ComplexValue {
            real: 0.31,
            imaginary: -0.17,
        };
        let mut normal = [[0.0; PARAMETER_COUNT]; PARAMETER_COUNT];
        let mut rhs = [0.0; PARAMETER_COUNT];
        for basis in &bases {
            let target = basis[2].multiply(coefficient);
            let row = standardized_real_row(basis, &scales);
            accumulate(&mut normal, &mut rhs, &row, target.real);
            let row = standardized_imaginary_row(basis, &scales);
            accumulate(&mut normal, &mut rhs, &row, target.imaginary);
        }
        for (index, row) in normal.iter_mut().enumerate() {
            row[index] += 1.0e-12;
        }
        let parameters = solve(normal, rhs).expect("fit");
        let recovered: [ComplexValue; ORDER_COUNT] = std::array::from_fn(|order| ComplexValue {
            real: parameters[order] / scales[order],
            imaginary: parameters[order + ORDER_COUNT] / scales[order],
        });
        assert!(recovered[2].subtract(coefficient).magnitude() < 1.0e-10);
        let leakage = [0, 1, 3]
            .into_iter()
            .map(|order| recovered[order].magnitude_squared())
            .sum::<f64>();
        assert!(leakage < 1.0e-18);
    }
}
