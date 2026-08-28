use serde::{Deserialize, Serialize};

use super::SourceCondition;

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AngularFamily {
    AxisymmetricMZero,
    FullRealSphericalHarmonics,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct BasisColumn {
    pub(super) degree: usize,
    pub(super) order: usize,
    pub(super) component: &'static str,
}

pub(super) struct FitResult {
    pub(super) columns: Vec<BasisColumn>,
    pub(super) coefficients: Vec<ComplexValue>,
    pub(super) max_peak_normalized_complex_error: f64,
}

pub(super) struct FitRequest<'a> {
    pub(super) rows: &'a [&'a SourceCondition],
    pub(super) directions: &'a [[f64; 3]],
    pub(super) wave_number_reference_length: f64,
    pub(super) reference_length_metres: f64,
    pub(super) family: AngularFamily,
    pub(super) maximum_degree: usize,
    pub(super) ridge: f64,
}

pub(super) fn fit_coefficients(request: &FitRequest<'_>) -> Result<FitResult, String> {
    let columns = basis_columns(request.family, request.maximum_degree)?;
    if request.rows.len() < columns.len() || !request.ridge.is_finite() || request.ridge <= 0.0 {
        return Err("triaxial cooker fit setup is invalid".to_owned());
    }
    let wave_number = request.wave_number_reference_length / request.reference_length_metres;
    let radius =
        request.reference_length_metres * request.rows[0].listener_radius_reference_multiplier;
    let bases = request
        .rows
        .iter()
        .map(|row| {
            basis(
                wave_number,
                radius,
                request.directions[row.direction_index],
                &columns,
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
    let scales = column_scales(&bases)?;
    let parameter_count = columns.len() * 2;
    let mut normal = vec![vec![0.0; parameter_count]; parameter_count];
    let mut rhs = vec![0.0; parameter_count];
    for (basis, row) in bases.iter().zip(request.rows) {
        let real = standardized_row(basis, &scales, false);
        accumulate(&mut normal, &mut rhs, &real, row.computed.real);
        let imaginary = standardized_row(basis, &scales, true);
        accumulate(&mut normal, &mut rhs, &imaginary, row.computed.imaginary);
    }
    for (index, row) in normal.iter_mut().enumerate() {
        row[index] += request.ridge;
    }
    let parameters = solve(normal, rhs)?;
    let coefficient_count = columns.len();
    let coefficients = (0..coefficient_count)
        .map(|index| ComplexValue {
            real: parameters[index] / scales[index],
            imaginary: parameters[index + coefficient_count] / scales[index],
        })
        .collect::<Vec<_>>();
    let peak = request
        .rows
        .iter()
        .map(|row| row.computed.magnitude())
        .max_by(f64::total_cmp)
        .ok_or_else(|| "triaxial cooker fit is empty".to_owned())?;
    if !peak.is_finite() || peak <= 1.0e-30 {
        return Err("triaxial cooker fit peak is invalid".to_owned());
    }
    let max_peak_normalized_complex_error = bases
        .iter()
        .zip(request.rows)
        .map(|(basis, row)| {
            evaluate(basis, &coefficients)
                .map(|value| value.subtract(row.computed).magnitude() / peak)
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .max_by(f64::total_cmp)
        .ok_or_else(|| "triaxial cooker fit is empty".to_owned())?;
    Ok(FitResult {
        columns,
        coefficients,
        max_peak_normalized_complex_error,
    })
}

pub(super) fn basis_columns(
    family: AngularFamily,
    maximum_degree: usize,
) -> Result<Vec<BasisColumn>, String> {
    if maximum_degree > 8 {
        return Err("triaxial cooker maximum degree is invalid".to_owned());
    }
    let mut columns = Vec::new();
    for degree in 0..=maximum_degree {
        columns.push(BasisColumn {
            degree,
            order: 0,
            component: "m_zero",
        });
        if family == AngularFamily::FullRealSphericalHarmonics {
            for order in 1..=degree {
                columns.push(BasisColumn {
                    degree,
                    order,
                    component: "cosine",
                });
                columns.push(BasisColumn {
                    degree,
                    order,
                    component: "sine",
                });
            }
        }
    }
    Ok(columns)
}

pub(super) fn basis(
    wave_number: f64,
    listener_radius: f64,
    direction: [f64; 3],
    columns: &[BasisColumn],
) -> Result<Vec<ComplexValue>, String> {
    let maximum_degree = columns
        .iter()
        .map(|column| column.degree)
        .max()
        .ok_or_else(|| "triaxial cooker basis has no columns".to_owned())?;
    let argument = wave_number * listener_radius;
    let hankel = spherical_hankel_first_kind(argument, maximum_degree)?;
    let (polar_cosine, azimuth) = spherical_direction(direction)?;
    columns
        .iter()
        .map(|column| {
            let legendre = associated_legendre(column.degree, column.order, polar_cosine)?;
            let angular = match column.component {
                "m_zero" => legendre,
                "cosine" => legendre * (column.order as f64 * azimuth).cos(),
                "sine" => legendre * (column.order as f64 * azimuth).sin(),
                _ => return Err("triaxial cooker basis component is invalid".to_owned()),
            };
            Ok(hankel[column.degree].scale(angular))
        })
        .collect()
}

fn spherical_hankel_first_kind(
    argument: f64,
    maximum_degree: usize,
) -> Result<Vec<ComplexValue>, String> {
    if !argument.is_finite() || argument <= 1.0e-6 {
        return Err("triaxial cooker wave argument is invalid".to_owned());
    }
    let sine = argument.sin();
    let cosine = argument.cos();
    let mut result = vec![ComplexValue::ZERO; maximum_degree + 1];
    result[0] = ComplexValue {
        real: sine / argument,
        imaginary: -cosine / argument,
    };
    if maximum_degree > 0 {
        result[1] = ComplexValue {
            real: sine / argument.powi(2) - cosine / argument,
            imaginary: -cosine / argument.powi(2) - sine / argument,
        };
        for degree in 1..maximum_degree {
            let factor = (2 * degree + 1) as f64 / argument;
            result[degree + 1] = result[degree].scale(factor).subtract(result[degree - 1]);
        }
    }
    result
        .iter()
        .all(|value| value.is_finite())
        .then_some(result)
        .ok_or_else(|| "triaxial cooker Hankel basis is non-finite".to_owned())
}

fn spherical_direction(direction: [f64; 3]) -> Result<(f64, f64), String> {
    let norm = direction
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if direction.iter().any(|value| !value.is_finite()) || norm <= 0.0 {
        return Err("triaxial cooker direction is invalid".to_owned());
    }
    Ok((
        (direction[2] / norm).clamp(-1.0, 1.0),
        direction[1].atan2(direction[0]),
    ))
}

fn associated_legendre(degree: usize, order: usize, cosine: f64) -> Result<f64, String> {
    if order > degree || !cosine.is_finite() || !(-1.0..=1.0).contains(&cosine) {
        return Err("triaxial cooker associated Legendre input is invalid".to_owned());
    }
    let mut p_mm = 1.0;
    if order > 0 {
        let root = (1.0 - cosine * cosine).max(0.0).sqrt();
        for index in 1..=order {
            p_mm *= -((2 * index - 1) as f64) * root;
        }
    }
    if degree == order {
        return Ok(p_mm);
    }
    let p_m1m = cosine * (2 * order + 1) as f64 * p_mm;
    if degree == order + 1 {
        return Ok(p_m1m);
    }
    let mut previous = p_mm;
    let mut current = p_m1m;
    for next_degree in order + 2..=degree {
        let next = ((2 * next_degree - 1) as f64 * cosine * current
            - (next_degree + order - 1) as f64 * previous)
            / (next_degree - order) as f64;
        previous = current;
        current = next;
    }
    Ok(current)
}

fn column_scales(bases: &[Vec<ComplexValue>]) -> Result<Vec<f64>, String> {
    let column_count = bases
        .first()
        .map(Vec::len)
        .ok_or_else(|| "triaxial cooker basis is empty".to_owned())?;
    let scales = (0..column_count)
        .map(|column| {
            (bases
                .iter()
                .map(|basis| basis[column].magnitude_squared())
                .sum::<f64>()
                / bases.len() as f64)
                .sqrt()
        })
        .collect::<Vec<_>>();
    if scales
        .iter()
        .all(|scale| scale.is_finite() && *scale > 1.0e-18)
    {
        Ok(scales)
    } else {
        Err("triaxial cooker basis column has zero scale".to_owned())
    }
}

fn standardized_row(basis: &[ComplexValue], scales: &[f64], imaginary_equation: bool) -> Vec<f64> {
    let count = basis.len();
    (0..count * 2)
        .map(|index| {
            let column = index % count;
            match (imaginary_equation, index < count) {
                (false, true) => basis[column].real / scales[column],
                (false, false) => -basis[column].imaginary / scales[column],
                (true, true) => basis[column].imaginary / scales[column],
                (true, false) => basis[column].real / scales[column],
            }
        })
        .collect()
}

fn accumulate(normal: &mut [Vec<f64>], rhs: &mut [f64], row: &[f64], target: f64) {
    for output in 0..row.len() {
        rhs[output] += row[output] * target;
        for input in 0..row.len() {
            normal[output][input] += row[output] * row[input];
        }
    }
}

fn solve(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Result<Vec<f64>, String> {
    let count = rhs.len();
    for pivot in 0..count {
        let best = (pivot..count)
            .max_by(|left, right| {
                matrix[*left][pivot]
                    .abs()
                    .total_cmp(&matrix[*right][pivot].abs())
            })
            .expect("non-empty pivot range");
        if matrix[best][pivot].abs() <= 1.0e-14 {
            return Err("triaxial cooker ridge system is singular".to_owned());
        }
        matrix.swap(pivot, best);
        rhs.swap(pivot, best);
        let divisor = matrix[pivot][pivot];
        for value in matrix[pivot].iter_mut().skip(pivot) {
            *value /= divisor;
        }
        rhs[pivot] /= divisor;
        let pivot_row = matrix[pivot].clone();
        for row in 0..count {
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
        .ok_or_else(|| "triaxial cooker solution is non-finite".to_owned())
}

pub(super) fn evaluate(
    basis: &[ComplexValue],
    coefficients: &[ComplexValue],
) -> Result<ComplexValue, String> {
    if basis.len() != coefficients.len() {
        return Err("triaxial cooker evaluation dimensions changed".to_owned());
    }
    let result = basis
        .iter()
        .zip(coefficients)
        .fold(ComplexValue::ZERO, |sum, (basis, coefficient)| {
            sum.add(basis.multiply(*coefficient))
        });
    result
        .is_finite()
        .then_some(result)
        .ok_or_else(|| "triaxial cooker prediction is non-finite".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outgoing_hankel_zero_order_matches_closed_form() {
        let argument = 2.75;
        let basis = spherical_hankel_first_kind(argument, 4).expect("basis");
        assert!((basis[0].real - argument.sin() / argument).abs() < 1.0e-15);
        assert!((basis[0].imaginary + argument.cos() / argument).abs() < 1.0e-15);
    }

    #[test]
    fn full_degree_two_basis_contains_nine_columns() {
        let columns = basis_columns(AngularFamily::FullRealSphericalHarmonics, 2).expect("columns");
        assert_eq!(columns.len(), 9);
        assert_eq!(
            columns.iter().filter(|column| column.degree == 2).count(),
            5
        );
    }

    #[test]
    fn xz_mode_is_not_axisymmetric() {
        let direction_a = [0.6, 0.0, 0.8];
        let direction_b = [0.0, 0.6, 0.8];
        let axis = basis_columns(AngularFamily::AxisymmetricMZero, 4).expect("axis");
        let full = basis_columns(AngularFamily::FullRealSphericalHarmonics, 2).expect("full");
        let axis_a = basis(7.5, 0.2, direction_a, &axis).expect("axis a");
        let axis_b = basis(7.5, 0.2, direction_b, &axis).expect("axis b");
        assert_eq!(axis_a.len(), axis_b.len());
        for (left, right) in axis_a.iter().zip(&axis_b) {
            assert!((left.real - right.real).abs() < 1.0e-14);
            assert!((left.imaginary - right.imaginary).abs() < 1.0e-14);
        }
        let full_a = basis(7.5, 0.2, direction_a, &full).expect("full a");
        let full_b = basis(7.5, 0.2, direction_b, &full).expect("full b");
        assert!(full_a.iter().zip(&full_b).any(|(left, right)| {
            (left.real - right.real).abs() > 1.0e-8
                || (left.imaginary - right.imaginary).abs() > 1.0e-8
        }));
    }
}
