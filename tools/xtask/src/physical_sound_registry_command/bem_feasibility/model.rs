use std::collections::BTreeMap;
use std::f64::consts::PI;

use serde::Serialize;

use super::{Fixture, Solver};

const THREE_POINT_QUADRATURE: [QuadraturePoint; 3] = [
    QuadraturePoint::new([2.0 / 3.0, 1.0 / 6.0, 1.0 / 6.0], 1.0 / 3.0),
    QuadraturePoint::new([1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0], 1.0 / 3.0),
    QuadraturePoint::new([1.0 / 6.0, 1.0 / 6.0, 2.0 / 3.0], 1.0 / 3.0),
];
const SEVEN_POINT_QUADRATURE: [QuadraturePoint; 7] = [
    QuadraturePoint::new([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0], 0.225),
    QuadraturePoint::new(
        [
            0.059_715_871_789_77,
            0.470_142_064_105_115,
            0.470_142_064_105_115,
        ],
        0.132_394_152_788_506,
    ),
    QuadraturePoint::new(
        [
            0.470_142_064_105_115,
            0.059_715_871_789_77,
            0.470_142_064_105_115,
        ],
        0.132_394_152_788_506,
    ),
    QuadraturePoint::new(
        [
            0.470_142_064_105_115,
            0.470_142_064_105_115,
            0.059_715_871_789_77,
        ],
        0.132_394_152_788_506,
    ),
    QuadraturePoint::new(
        [
            0.797_426_985_353_087,
            0.101_286_507_323_456,
            0.101_286_507_323_456,
        ],
        0.125_939_180_544_827,
    ),
    QuadraturePoint::new(
        [
            0.101_286_507_323_456,
            0.797_426_985_353_087,
            0.101_286_507_323_456,
        ],
        0.125_939_180_544_827,
    ),
    QuadraturePoint::new(
        [
            0.101_286_507_323_456,
            0.101_286_507_323_456,
            0.797_426_985_353_087,
        ],
        0.125_939_180_544_827,
    ),
];

pub(super) fn solve_fixture(fixture: &Fixture, solver: &Solver) -> Result<SolverResult, String> {
    if solver.constant_panel_subdivision_levels.len() != 2 {
        return Err("BEM feasibility requires exactly two mesh levels".to_owned());
    }
    let coarse = solve_level(fixture, solver, solver.constant_panel_subdivision_levels[0])?;
    let fine = solve_level(fixture, solver, solver.constant_panel_subdivision_levels[1])?;
    Ok(SolverResult { coarse, fine })
}

fn solve_level(
    fixture: &Fixture,
    solver: &Solver,
    subdivision_level: usize,
) -> Result<LevelResult, String> {
    let quadrature = Quadrature::parse(&solver.off_diagonal_panel_quadrature)?;
    let mesh = Mesh::icosphere(fixture.radius_metres, subdivision_level)?;
    let panels = mesh.panels()?;
    let mut conditions = Vec::new();
    for wave_number_radius in &fixture.wave_number_radius_values {
        let wave_number = *wave_number_radius / fixture.radius_metres;
        let frequency_hz = wave_number * fixture.speed_of_sound_metres_per_second / (2.0 * PI);
        let boundary_derivative = Complex {
            real: fixture.normal_pressure_derivative_real,
            imaginary: fixture.normal_pressure_derivative_imaginary,
        };
        let density = solve_density(
            &panels,
            wave_number,
            boundary_derivative,
            solver.pivot_floor,
            quadrature,
        )?;
        for listener_radius_multiplier in &fixture.listener_radius_multipliers {
            for (direction_index, direction) in fixture.listener_directions.iter().enumerate() {
                let listener = normalized_scaled(
                    *direction,
                    fixture.radius_metres * listener_radius_multiplier,
                )?;
                let computed =
                    evaluate_potential(&panels, &density, listener, wave_number, quadrature)?;
                let analytical = analytical_pulsating_sphere(
                    fixture.radius_metres,
                    listener_radius_multiplier * fixture.radius_metres,
                    wave_number,
                    boundary_derivative,
                )?;
                let difference = computed - analytical;
                let relative_complex_error = difference.magnitude() / analytical.magnitude();
                let absolute_magnitude_error_db =
                    (20.0 * (computed.magnitude() / analytical.magnitude()).log10()).abs();
                let absolute_phase_error_degrees =
                    wrap_phase(computed.argument() - analytical.argument()).abs() * 180.0 / PI;
                if !relative_complex_error.is_finite()
                    || !absolute_magnitude_error_db.is_finite()
                    || !absolute_phase_error_degrees.is_finite()
                {
                    return Err("BEM condition metric is non-finite".to_owned());
                }
                conditions.push(ConditionResult {
                    wave_number_radius: *wave_number_radius,
                    frequency_hz,
                    listener_radius_multiplier: *listener_radius_multiplier,
                    direction_index,
                    computed,
                    analytical,
                    relative_complex_error,
                    absolute_magnitude_error_db,
                    absolute_phase_error_degrees,
                });
            }
        }
    }
    Ok(LevelResult {
        subdivision_level,
        panel_count: panels.len(),
        conditions,
    })
}

fn solve_density(
    panels: &[Panel],
    wave_number: f64,
    boundary_derivative: Complex,
    pivot_floor: f64,
    quadrature: Quadrature,
) -> Result<Vec<Complex>, String> {
    let count = panels.len();
    let mut matrix = vec![Complex::ZERO; count * count];
    let rhs = vec![boundary_derivative; count];
    for (row, target) in panels.iter().enumerate() {
        for (column, source) in panels.iter().enumerate() {
            matrix[row * count + column] = if row == column {
                Complex {
                    real: -0.5,
                    imaginary: 0.0,
                }
            } else {
                integrate_normal_derivative(target, source, wave_number, quadrature)?
            };
        }
    }
    solve_linear_system(matrix, rhs, pivot_floor)
}

fn integrate_normal_derivative(
    target: &Panel,
    source: &Panel,
    wave_number: f64,
    quadrature: Quadrature,
) -> Result<Complex, String> {
    let mut value = Complex::ZERO;
    for sample in quadrature.points() {
        let point = source.quadrature_point(sample.barycentric);
        let delta = subtract(target.centroid, point);
        let distance = norm(delta);
        if distance <= 1.0e-12 {
            return Err("BEM off-diagonal quadrature reached a singularity".to_owned());
        }
        let phase = wave_number * distance;
        let exponential = Complex::exp_i(phase);
        let factor = Complex {
            real: -1.0,
            imaginary: phase,
        } * (dot(target.normal, delta) / (4.0 * PI * distance.powi(3)));
        let panel_weight = match quadrature {
            Quadrature::ThreePoint => source.area / 3.0,
            Quadrature::SevenPoint => source.area * sample.weight,
        };
        value += exponential * factor * panel_weight;
    }
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| "BEM normal-derivative integral is non-finite".to_owned())
}

fn evaluate_potential(
    panels: &[Panel],
    density: &[Complex],
    listener: [f64; 3],
    wave_number: f64,
    quadrature: Quadrature,
) -> Result<Complex, String> {
    if panels.len() != density.len() {
        return Err("BEM density length changed".to_owned());
    }
    let mut field = Complex::ZERO;
    for (panel, coefficient) in panels.iter().zip(density) {
        let mut integral = Complex::ZERO;
        for sample in quadrature.points() {
            let point = panel.quadrature_point(sample.barycentric);
            let distance = norm(subtract(listener, point));
            if distance <= 1.0e-12 {
                return Err("BEM listener reached the boundary".to_owned());
            }
            let potential_weight = match quadrature {
                Quadrature::ThreePoint => panel.area / (3.0 * 4.0 * PI * distance),
                Quadrature::SevenPoint => panel.area * sample.weight / (4.0 * PI * distance),
            };
            integral += Complex::exp_i(wave_number * distance) * potential_weight;
        }
        field += integral * *coefficient;
    }
    field
        .is_finite()
        .then_some(field)
        .ok_or_else(|| "BEM field is non-finite".to_owned())
}

fn analytical_pulsating_sphere(
    sphere_radius: f64,
    listener_radius: f64,
    wave_number: f64,
    boundary_derivative: Complex,
) -> Result<Complex, String> {
    if listener_radius <= sphere_radius {
        return Err("analytical listener must be outside the sphere".to_owned());
    }
    let numerator = boundary_derivative
        * sphere_radius.powi(2)
        * Complex::exp_i(wave_number * (listener_radius - sphere_radius));
    let denominator = Complex {
        real: -1.0,
        imaginary: wave_number * sphere_radius,
    } * listener_radius;
    let result = numerator.divide(denominator)?;
    result
        .is_finite()
        .then_some(result)
        .ok_or_else(|| "analytical pulsating-sphere field is non-finite".to_owned())
}

fn solve_linear_system(
    mut matrix: Vec<Complex>,
    mut rhs: Vec<Complex>,
    pivot_floor: f64,
) -> Result<Vec<Complex>, String> {
    let count = rhs.len();
    if matrix.len() != count * count || count == 0 {
        return Err("BEM linear system shape is invalid".to_owned());
    }
    for pivot in 0..count {
        let best = (pivot..count)
            .max_by(|left, right| {
                matrix[*left * count + pivot]
                    .magnitude_squared()
                    .total_cmp(&matrix[*right * count + pivot].magnitude_squared())
            })
            .expect("non-empty BEM pivot range");
        if matrix[best * count + pivot].magnitude() <= pivot_floor {
            return Err(format!("BEM linear system is singular at pivot {pivot}"));
        }
        if best != pivot {
            for column in 0..count {
                matrix.swap(pivot * count + column, best * count + column);
            }
            rhs.swap(pivot, best);
        }
        let pivot_value = matrix[pivot * count + pivot];
        for row in (pivot + 1)..count {
            let factor = matrix[row * count + pivot].divide(pivot_value)?;
            matrix[row * count + pivot] = Complex::ZERO;
            for column in (pivot + 1)..count {
                let pivot_entry = matrix[pivot * count + column];
                matrix[row * count + column] -= factor * pivot_entry;
            }
            let pivot_rhs = rhs[pivot];
            rhs[row] -= factor * pivot_rhs;
        }
    }
    let mut solution = vec![Complex::ZERO; count];
    for row in (0..count).rev() {
        let mut value = rhs[row];
        for column in (row + 1)..count {
            value -= matrix[row * count + column] * solution[column];
        }
        solution[row] = value.divide(matrix[row * count + row])?;
    }
    if solution.iter().all(|value| value.is_finite()) {
        Ok(solution)
    } else {
        Err("BEM linear solution is non-finite".to_owned())
    }
}

fn wrap_phase(mut phase: f64) -> f64 {
    while phase > PI {
        phase -= 2.0 * PI;
    }
    while phase < -PI {
        phase += 2.0 * PI;
    }
    phase
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct Complex {
    pub(super) real: f64,
    pub(super) imaginary: f64,
}

impl Complex {
    const ZERO: Self = Self {
        real: 0.0,
        imaginary: 0.0,
    };

    fn exp_i(phase: f64) -> Self {
        Self {
            real: phase.cos(),
            imaginary: phase.sin(),
        }
    }

    pub(super) fn magnitude(self) -> f64 {
        self.real.hypot(self.imaginary)
    }

    fn magnitude_squared(self) -> f64 {
        self.real
            .mul_add(self.real, self.imaginary * self.imaginary)
    }

    fn argument(self) -> f64 {
        self.imaginary.atan2(self.real)
    }

    fn divide(self, divisor: Self) -> Result<Self, String> {
        let denominator = divisor.magnitude_squared();
        if !denominator.is_finite() || denominator <= 1.0e-30 {
            return Err("complex BEM division by zero".to_owned());
        }
        Ok(Self {
            real: (self.real * divisor.real + self.imaginary * divisor.imaginary) / denominator,
            imaginary: (self.imaginary * divisor.real - self.real * divisor.imaginary)
                / denominator,
        })
    }

    fn is_finite(self) -> bool {
        self.real.is_finite() && self.imaginary.is_finite()
    }
}

impl std::ops::AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        self.real += rhs.real;
        self.imaginary += rhs.imaginary;
    }
}

impl std::ops::Add for Complex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real + rhs.real,
            imaginary: self.imaginary + rhs.imaginary,
        }
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real - rhs.real,
            imaginary: self.imaginary - rhs.imaginary,
        }
    }
}

impl std::ops::SubAssign for Complex {
    fn sub_assign(&mut self, rhs: Self) {
        self.real -= rhs.real;
        self.imaginary -= rhs.imaginary;
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real * rhs.real - self.imaginary * rhs.imaginary,
            imaginary: self.real * rhs.imaginary + self.imaginary * rhs.real,
        }
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            real: self.real * rhs,
            imaginary: self.imaginary * rhs,
        }
    }
}

#[derive(Debug)]
pub(super) struct SolverResult {
    pub(super) coarse: LevelResult,
    pub(super) fine: LevelResult,
}

#[derive(Debug)]
pub(super) struct LevelResult {
    pub(super) subdivision_level: usize,
    pub(super) panel_count: usize,
    pub(super) conditions: Vec<ConditionResult>,
}

#[derive(Debug, Serialize)]
pub(super) struct ConditionResult {
    pub(super) wave_number_radius: f64,
    frequency_hz: f64,
    pub(super) listener_radius_multiplier: f64,
    direction_index: usize,
    pub(super) computed: Complex,
    analytical: Complex,
    pub(super) relative_complex_error: f64,
    pub(super) absolute_magnitude_error_db: f64,
    pub(super) absolute_phase_error_degrees: f64,
}

#[derive(Clone, Copy, Debug)]
struct Panel {
    vertices: [[f64; 3]; 3],
    centroid: [f64; 3],
    normal: [f64; 3],
    area: f64,
}

impl Panel {
    fn quadrature_point(self, weights: [f64; 3]) -> [f64; 3] {
        [
            weights[0] * self.vertices[0][0]
                + weights[1] * self.vertices[1][0]
                + weights[2] * self.vertices[2][0],
            weights[0] * self.vertices[0][1]
                + weights[1] * self.vertices[1][1]
                + weights[2] * self.vertices[2][1],
            weights[0] * self.vertices[0][2]
                + weights[1] * self.vertices[1][2]
                + weights[2] * self.vertices[2][2],
        ]
    }
}

#[derive(Clone, Copy, Debug)]
struct QuadraturePoint {
    barycentric: [f64; 3],
    weight: f64,
}

impl QuadraturePoint {
    const fn new(barycentric: [f64; 3], weight: f64) -> Self {
        Self {
            barycentric,
            weight,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Quadrature {
    ThreePoint,
    SevenPoint,
}

impl Quadrature {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "symmetric-three-point-triangle" => Ok(Self::ThreePoint),
            "symmetric-seven-point-triangle" => Ok(Self::SevenPoint),
            _ => Err(format!("unsupported BEM panel quadrature: {value}")),
        }
    }

    fn points(self) -> &'static [QuadraturePoint] {
        match self {
            Self::ThreePoint => &THREE_POINT_QUADRATURE,
            Self::SevenPoint => &SEVEN_POINT_QUADRATURE,
        }
    }
}

#[derive(Debug)]
struct Mesh {
    vertices: Vec<[f64; 3]>,
    faces: Vec<[usize; 3]>,
}

impl Mesh {
    fn icosphere(radius: f64, subdivision_level: usize) -> Result<Self, String> {
        let golden = (1.0 + 5.0_f64.sqrt()) * 0.5;
        let raw_vertices = [
            [-1.0, golden, 0.0],
            [1.0, golden, 0.0],
            [-1.0, -golden, 0.0],
            [1.0, -golden, 0.0],
            [0.0, -1.0, golden],
            [0.0, 1.0, golden],
            [0.0, -1.0, -golden],
            [0.0, 1.0, -golden],
            [golden, 0.0, -1.0],
            [golden, 0.0, 1.0],
            [-golden, 0.0, -1.0],
            [-golden, 0.0, 1.0],
        ];
        let mut mesh = Self {
            vertices: raw_vertices
                .into_iter()
                .map(|vertex| normalized_scaled(vertex, radius))
                .collect::<Result<Vec<_>, _>>()?,
            faces: vec![
                [0, 11, 5],
                [0, 5, 1],
                [0, 1, 7],
                [0, 7, 10],
                [0, 10, 11],
                [1, 5, 9],
                [5, 11, 4],
                [11, 10, 2],
                [10, 7, 6],
                [7, 1, 8],
                [3, 9, 4],
                [3, 4, 2],
                [3, 2, 6],
                [3, 6, 8],
                [3, 8, 9],
                [4, 9, 5],
                [2, 4, 11],
                [6, 2, 10],
                [8, 6, 7],
                [9, 8, 1],
            ],
        };
        for _ in 0..subdivision_level {
            mesh.subdivide(radius)?;
        }
        Ok(mesh)
    }

    fn subdivide(&mut self, radius: f64) -> Result<(), String> {
        let mut midpoints = BTreeMap::<(usize, usize), usize>::new();
        let mut faces = Vec::with_capacity(self.faces.len() * 4);
        for face in self.faces.clone() {
            let ab = self.midpoint(face[0], face[1], radius, &mut midpoints)?;
            let bc = self.midpoint(face[1], face[2], radius, &mut midpoints)?;
            let ca = self.midpoint(face[2], face[0], radius, &mut midpoints)?;
            faces.extend_from_slice(&[
                [face[0], ab, ca],
                [face[1], bc, ab],
                [face[2], ca, bc],
                [ab, bc, ca],
            ]);
        }
        self.faces = faces;
        Ok(())
    }

    fn midpoint(
        &mut self,
        left: usize,
        right: usize,
        radius: f64,
        cache: &mut BTreeMap<(usize, usize), usize>,
    ) -> Result<usize, String> {
        let key = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        if let Some(index) = cache.get(&key) {
            return Ok(*index);
        }
        let midpoint = normalized_scaled(
            [
                (self.vertices[left][0] + self.vertices[right][0]) * 0.5,
                (self.vertices[left][1] + self.vertices[right][1]) * 0.5,
                (self.vertices[left][2] + self.vertices[right][2]) * 0.5,
            ],
            radius,
        )?;
        let index = self.vertices.len();
        self.vertices.push(midpoint);
        cache.insert(key, index);
        Ok(index)
    }

    fn panels(&self) -> Result<Vec<Panel>, String> {
        self.faces
            .iter()
            .map(|face| {
                let mut vertices = [
                    self.vertices[face[0]],
                    self.vertices[face[1]],
                    self.vertices[face[2]],
                ];
                let mut cross_value = cross(
                    subtract(vertices[1], vertices[0]),
                    subtract(vertices[2], vertices[0]),
                );
                let mut centroid = average(vertices);
                if dot(cross_value, centroid) < 0.0 {
                    vertices.swap(1, 2);
                    cross_value = cross(
                        subtract(vertices[1], vertices[0]),
                        subtract(vertices[2], vertices[0]),
                    );
                    centroid = average(vertices);
                }
                let cross_norm = norm(cross_value);
                if cross_norm <= 1.0e-12 {
                    return Err("BEM mesh contains a degenerate panel".to_owned());
                }
                Ok(Panel {
                    vertices,
                    centroid,
                    normal: scale(cross_value, 1.0 / cross_norm),
                    area: cross_norm * 0.5,
                })
            })
            .collect()
    }
}

fn normalized_scaled(vector: [f64; 3], length: f64) -> Result<[f64; 3], String> {
    let vector_norm = norm(vector);
    if !vector_norm.is_finite() || vector_norm <= 1.0e-15 || !length.is_finite() || length <= 0.0 {
        return Err("BEM vector normalization is invalid".to_owned());
    }
    Ok(scale(vector, length / vector_norm))
}

fn average(vertices: [[f64; 3]; 3]) -> [f64; 3] {
    [
        (vertices[0][0] + vertices[1][0] + vertices[2][0]) / 3.0,
        (vertices[0][1] + vertices[1][1] + vertices[2][1]) / 3.0,
        (vertices[0][2] + vertices[1][2] + vertices[2][2]) / 3.0,
    ]
}

fn subtract(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0].mul_add(right[0], left[1].mul_add(right[1], left[2] * right[2]))
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn norm(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosphere_panel_counts_and_orientation_are_stable() {
        let level_one = Mesh::icosphere(0.1, 1).expect("level one mesh");
        let level_two = Mesh::icosphere(0.1, 2).expect("level two mesh");
        assert_eq!(level_one.faces.len(), 80);
        assert_eq!(level_two.faces.len(), 320);
        assert!(
            level_two
                .panels()
                .expect("panels")
                .iter()
                .all(|panel| dot(panel.normal, panel.centroid) > 0.0)
        );
    }

    #[test]
    fn complex_solver_recovers_known_solution() {
        let matrix = vec![
            Complex {
                real: 2.0,
                imaginary: 1.0,
            },
            Complex {
                real: -1.0,
                imaginary: 0.5,
            },
            Complex {
                real: 0.25,
                imaginary: -0.5,
            },
            Complex {
                real: 1.5,
                imaginary: -0.25,
            },
        ];
        let expected = vec![
            Complex {
                real: 0.75,
                imaginary: -0.2,
            },
            Complex {
                real: -0.4,
                imaginary: 0.6,
            },
        ];
        let rhs = vec![
            matrix[0] * expected[0] + matrix[1] * expected[1],
            matrix[2] * expected[0] + matrix[3] * expected[1],
        ];
        let solved = solve_linear_system(matrix, rhs, 1.0e-12).expect("solve");
        for (actual, expected) in solved.iter().zip(expected) {
            assert!((*actual - expected).magnitude() < 1.0e-12);
        }
    }

    #[test]
    fn analytical_field_has_prescribed_boundary_derivative() {
        let radius = 0.1;
        let wave_number = 7.5;
        let derivative = Complex {
            real: 1.0,
            imaginary: 0.0,
        };
        let step = 1.0e-7;
        let below = analytical_pulsating_sphere(radius, radius + step, wave_number, derivative)
            .expect("below");
        let above =
            analytical_pulsating_sphere(radius, radius + 2.0 * step, wave_number, derivative)
                .expect("above");
        let finite_difference = (above - below) * (1.0 / step);
        assert!((finite_difference - derivative).magnitude() < 1.0e-4);
    }

    #[test]
    fn frozen_quadrature_rules_integrate_a_constant() {
        for quadrature in [Quadrature::ThreePoint, Quadrature::SevenPoint] {
            let weight = quadrature
                .points()
                .iter()
                .map(|point| point.weight)
                .sum::<f64>();
            assert!((weight - 1.0).abs() < 1.0e-14);
            assert!(
                quadrature
                    .points()
                    .iter()
                    .all(|point| { (point.barycentric.iter().sum::<f64>() - 1.0).abs() < 1.0e-14 })
            );
        }
    }
}
