#![forbid(unsafe_code)]

use std::mem;

use crate::error::{AUDIT_INVALID, DENSITY_NONCONVERGENCE, WaterError};
use crate::model::Vec3i;

use super::{
    AuditBoundaryInput, AuditComputation, AuditFluidInput, BoundaryNeighborTrace,
    DensityInitialTrace, DensityIterationRowTrace, HydroAuditTrace, HydroRowTrace, SELECTED_ROWS,
    VolumeMapObservation, empty_volume_map_observation, scalar_bits,
};

mod calibration;
mod initialization;
mod volume_map;

pub(crate) use calibration::{
    compute as compute_calibration, compute_ghost as compute_ghost_calibration,
    compute_support_complete as compute_support_complete_calibration,
    compute_volume_map as compute_volume_map_calibration,
};
pub(crate) use initialization::compute as compute_zero_velocity_settling;

const FLUID_COUNT: usize = 6_000;
const BOUNDARY_COUNT: usize = 2_402;
const MAX_ROW_NEIGHBORS: usize = 128;
const SUPPORT_RADIUS_SQUARED: i128 = 10_000_000_000;
const SCALE: f64 = f64::from_bits(0x412e_8480_0000_0000);
const SUPPORT_RADIUS: f64 = f64::from_bits(0x3fb9_9999_9999_999a);
const REST_VOLUME: f64 = f64::from_bits(0x3f20_624d_d2f1_a9fc);
const KERNEL_K: f64 = f64::from_bits(0x40a3_e4f5_4b37_0dcf);
const KERNEL_L: f64 = f64::from_bits(0x40cd_d76f_f0d2_94b6);
const DT: f64 = f64::from_bits(0x3f71_1111_1111_1111);
const DT2: f64 = f64::from_bits(0x3ef2_3456_789a_bcdf);
const INV_DT2: f64 = f64::from_bits(0x40ec_2000_0000_0000);
const GRAVITY: f64 = f64::from_bits(0x4023_9eb8_51eb_851f);
const SOLVER_EPSILON: f64 = f64::from_bits(0x3ee4_f8b5_88e3_68f1);
const RELAXATION: f64 = f64::from_bits(0x3fe0_0000_0000_0000);
const DENSITY_MAX_ITERATIONS: u8 = 20;
const DENSITY_MIN_ITERATIONS: u8 = 2;
const DENSITY_THRESHOLD_PPB: i64 = 100_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct I3 {
    x: i64,
    y: i64,
    z: i64,
}

impl I3 {
    const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn distance_squared(self) -> i128 {
        let x = i128::from(self.x);
        let y = i128::from(self.y);
        let z = i128::from(self.z);
        ((x * x) + (y * y)) + (z * z)
    }

    const fn common(self) -> Vec3i {
        Vec3i::new(self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct F3 {
    x: f64,
    y: f64,
    z: f64,
}

impl F3 {
    const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn scale(self, value: f64) -> Self {
        Self::new(self.x * value, self.y * value, self.z * value)
    }

    fn dot(self, other: Self) -> f64 {
        let xy = (self.x * other.x) + (self.y * other.y);
        xy + (self.z * other.z)
    }
}

#[derive(Clone, Copy)]
struct Kernel {
    value: f64,
    gradient: F3,
}

#[derive(Clone, Copy)]
struct FluidNeighbor {
    other: usize,
    value: f64,
    gradient: F3,
}

#[derive(Clone, Copy)]
struct BoundaryNeighbor {
    boundary: usize,
    volume: f64,
    value: f64,
    gradient: F3,
    feature_rank: usize,
}

struct Row {
    fluid: Vec<FluidNeighbor>,
    boundary: Vec<BoundaryNeighbor>,
    rho_ratio: f64,
    alpha: f64,
}

pub(super) fn compute() -> Result<AuditComputation, WaterError> {
    let fluid_positions = generate_fluid()?;
    let boundary_positions = generate_boundary()?;
    let boundary_volumes = boundary_volumes(&boundary_positions)?;
    let rows = reconstruct(&fluid_positions, &boundary_positions, &boundary_volumes)?;
    let trace = density_trace(&fluid_positions, &rows)?;

    let mut fluid = reserved(FLUID_COUNT)?;
    for (index, position) in fluid_positions.iter().copied().enumerate() {
        fluid.push(AuditFluidInput {
            id: u32::try_from(index)
                .map_err(|_| WaterError::new(AUDIT_INVALID, "fluid id overflow"))?,
            position_um: position.common(),
            velocity_um_s: Vec3i::new(0, 0, 0),
        });
    }
    let mut boundary = reserved(BOUNDARY_COUNT)?;
    for (index, (position, volume)) in boundary_positions
        .iter()
        .copied()
        .zip(boundary_volumes.iter().copied())
        .enumerate()
    {
        boundary.push(AuditBoundaryInput {
            id: u32::try_from(index)
                .map_err(|_| WaterError::new(AUDIT_INVALID, "boundary id overflow"))?,
            position_um: position.common(),
            volume_bits: scalar_bits(volume),
        });
    }
    Ok(AuditComputation {
        fluid,
        boundary,
        trace,
    })
}

pub(super) fn observe_volume_map(position_um: Vec3i) -> Result<VolumeMapObservation, WaterError> {
    let position = F3::new(
        (position_um.x as f64) / SCALE,
        (position_um.y as f64) / SCALE,
        (position_um.z as f64) / SCALE,
    );
    let Some(sample) = volume_map::sample(position)? else {
        return Ok(empty_volume_map_observation(position_um));
    };
    let contribution = finite(
        sample.volume * sample.value,
        "independent observed volume-map density contribution",
    )?;
    let volume_gradient = finite_vec(
        sample.gradient.scale(sample.volume),
        "independent observed volume-map volume gradient",
    )?;
    Ok(VolumeMapObservation {
        position_um,
        present: true,
        signed_distance_bits: Some(scalar_bits(sample.signed_distance)),
        volume_bits: Some(scalar_bits(sample.volume)),
        virtual_distance_bits: Some(scalar_bits(sample.virtual_distance)),
        displacement_bits: Some([
            scalar_bits(sample.displacement.x),
            scalar_bits(sample.displacement.y),
            scalar_bits(sample.displacement.z),
        ]),
        kernel_value_bits: Some(scalar_bits(sample.value)),
        kernel_gradient_bits: Some([
            scalar_bits(sample.gradient.x),
            scalar_bits(sample.gradient.y),
            scalar_bits(sample.gradient.z),
        ]),
        volume_gradient_bits: Some([
            scalar_bits(volume_gradient.x),
            scalar_bits(volume_gradient.y),
            scalar_bits(volume_gradient.z),
        ]),
        density_contribution_bits: Some(scalar_bits(contribution)),
        density_contribution_ppb: Some(quantize_ppb(contribution)?),
        feature_rank: Some(sample.feature_rank),
    })
}

fn generate_fluid() -> Result<Vec<I3>, WaterError> {
    let mut result = reserved(FLUID_COUNT)?;
    for iy in 0_i64..15 {
        for iz in 0_i64..20 {
            for ix in 0_i64..20 {
                result.push(I3::new(
                    25_000 + (50_000 * ix),
                    25_000 + (50_000 * iy),
                    25_000 + (50_000 * iz),
                ));
            }
        }
    }
    require_count("fluid", result.len(), FLUID_COUNT)?;
    Ok(result)
}

fn generate_boundary() -> Result<Vec<I3>, WaterError> {
    let mut result = reserved(BOUNDARY_COUNT)?;
    for ix in 0_i64..=20 {
        for iy in 0_i64..=20 {
            for iz in 0_i64..=20 {
                if ix == 0 || ix == 20 || iy == 0 || iy == 20 || iz == 0 || iz == 20 {
                    result.push(I3::new(50_000 * ix, 50_000 * iy, 50_000 * iz));
                }
            }
        }
    }
    require_count("boundary", result.len(), BOUNDARY_COUNT)?;
    Ok(result)
}

fn boundary_volumes(positions: &[I3]) -> Result<Vec<f64>, WaterError> {
    let mut volumes = reserved(positions.len())?;
    for (index, position) in positions.iter().copied().enumerate() {
        let mut denominator = KERNEL_K;
        let mut count = 0_usize;
        for (other_index, other) in positions.iter().copied().enumerate() {
            if index == other_index {
                continue;
            }
            let displacement = position.sub(other);
            if displacement.distance_squared() <= SUPPORT_RADIUS_SQUARED {
                count += 1;
                if count > MAX_ROW_NEIGHBORS {
                    return Err(WaterError::new(
                        AUDIT_INVALID,
                        format!("independent boundary row {index} exceeds capacity"),
                    ));
                }
                denominator = finite(
                    denominator + kernel(displacement)?.value,
                    "boundary denominator",
                )?;
            }
        }
        if denominator <= 0.0 {
            return Err(WaterError::new(
                AUDIT_INVALID,
                format!("independent boundary row {index} has nonpositive denominator"),
            ));
        }
        volumes.push(finite(1.0 / denominator, "boundary volume")?);
    }
    Ok(volumes)
}

fn reconstruct(
    fluid_positions: &[I3],
    boundary_positions: &[I3],
    boundary_volumes: &[f64],
) -> Result<Vec<Row>, WaterError> {
    let mut rows = reserved(fluid_positions.len())?;
    for (index, position) in fluid_positions.iter().copied().enumerate() {
        let mut fluid = reserved(MAX_ROW_NEIGHBORS)?;
        let mut boundary = reserved(MAX_ROW_NEIGHBORS)?;
        for (other, other_position) in fluid_positions.iter().copied().enumerate() {
            if other == index {
                continue;
            }
            let displacement = position.sub(other_position);
            if displacement.distance_squared() <= SUPPORT_RADIUS_SQUARED {
                let sampled = kernel(displacement)?;
                fluid.push(FluidNeighbor {
                    other,
                    value: sampled.value,
                    gradient: sampled.gradient,
                });
            }
        }
        for (boundary_index, boundary_position) in boundary_positions.iter().copied().enumerate() {
            let displacement = position.sub(boundary_position);
            if displacement.distance_squared() <= SUPPORT_RADIUS_SQUARED {
                let sampled = kernel(displacement)?;
                boundary.push(BoundaryNeighbor {
                    boundary: boundary_index,
                    volume: boundary_volumes[boundary_index],
                    value: sampled.value,
                    gradient: sampled.gradient,
                    feature_rank: 0,
                });
            }
        }
        if fluid.len() + boundary.len() > MAX_ROW_NEIGHBORS {
            return Err(WaterError::new(
                AUDIT_INVALID,
                format!("independent fluid row {index} exceeds capacity"),
            ));
        }
        let mut rho_ratio = finite(REST_VOLUME * KERNEL_K, "density self")?;
        for neighbor in &fluid {
            rho_ratio = finite(
                rho_ratio + (REST_VOLUME * neighbor.value),
                "density fluid fold",
            )?;
        }
        for neighbor in &boundary {
            rho_ratio = finite(
                rho_ratio + (neighbor.volume * neighbor.value),
                "density boundary fold",
            )?;
        }
        let mut sum_sq = 0.0;
        let mut central = F3::ZERO;
        for neighbor in &fluid {
            let volume_gradient = neighbor.gradient.scale(REST_VOLUME);
            let g = F3::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z);
            sum_sq = finite(sum_sq + g.dot(g), "factor sum squares")?;
            central = finite_vec(central.sub(g), "factor central fluid")?;
        }
        for neighbor in &boundary {
            let volume_gradient = neighbor.gradient.scale(neighbor.volume);
            let g = F3::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z);
            central = finite_vec(central.sub(g), "factor central boundary")?;
        }
        let denominator = finite(sum_sq + central.dot(central), "factor denominator")?;
        let alpha = if denominator > 1.0e-5 {
            finite(1.0 / denominator, "factor reciprocal")?
        } else {
            0.0
        };
        rows.push(Row {
            fluid,
            boundary,
            rho_ratio,
            alpha,
        });
    }
    Ok(rows)
}

fn reconstruct_volume_map(fluid_positions: &[I3]) -> Result<Vec<Row>, WaterError> {
    let mut rows = reserved(fluid_positions.len())?;
    for (index, position) in fluid_positions.iter().copied().enumerate() {
        let mut fluid = reserved(MAX_ROW_NEIGHBORS)?;
        let mut boundary = reserved(1)?;
        for (other, other_position) in fluid_positions.iter().copied().enumerate() {
            if other == index {
                continue;
            }
            let displacement = position.sub(other_position);
            if displacement.distance_squared() <= SUPPORT_RADIUS_SQUARED {
                let sampled = kernel(displacement)?;
                fluid.push(FluidNeighbor {
                    other,
                    value: sampled.value,
                    gradient: sampled.gradient,
                });
            }
        }
        let decoded = F3::new(
            (position.x as f64) / SCALE,
            (position.y as f64) / SCALE,
            (position.z as f64) / SCALE,
        );
        if let Some(sampled) = volume_map::sample(decoded)? {
            boundary.push(BoundaryNeighbor {
                boundary: usize::MAX,
                volume: sampled.volume,
                value: sampled.value,
                gradient: sampled.gradient,
                feature_rank: sampled.feature_rank,
            });
        }
        if fluid.len() + boundary.len() > MAX_ROW_NEIGHBORS {
            return Err(WaterError::new(
                AUDIT_INVALID,
                format!("independent volume-map row {index} exceeds capacity"),
            ));
        }
        let mut rho_ratio = finite(REST_VOLUME * KERNEL_K, "volume-map density self")?;
        for neighbor in &fluid {
            rho_ratio = finite(
                rho_ratio + (REST_VOLUME * neighbor.value),
                "volume-map density fluid fold",
            )?;
        }
        for neighbor in &boundary {
            rho_ratio = finite(
                rho_ratio + (neighbor.volume * neighbor.value),
                "volume-map density boundary fold",
            )?;
        }
        let mut sum_sq = 0.0;
        let mut central = F3::ZERO;
        for neighbor in &fluid {
            let volume_gradient = neighbor.gradient.scale(REST_VOLUME);
            let g = F3::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z);
            sum_sq = finite(sum_sq + g.dot(g), "volume-map factor sum squares")?;
            central = finite_vec(central.sub(g), "volume-map factor central fluid")?;
        }
        for neighbor in &boundary {
            let volume_gradient = neighbor.gradient.scale(neighbor.volume);
            let g = F3::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z);
            central = finite_vec(central.sub(g), "volume-map factor central boundary")?;
        }
        let denominator = finite(
            sum_sq + central.dot(central),
            "volume-map factor denominator",
        )?;
        let alpha = if denominator > 1.0e-5 {
            finite(1.0 / denominator, "volume-map factor reciprocal")?
        } else {
            0.0
        };
        rows.push(Row {
            fluid,
            boundary,
            rho_ratio,
            alpha,
        });
    }
    Ok(rows)
}

fn density_trace(positions: &[I3], rows: &[Row]) -> Result<HydroAuditTrace, WaterError> {
    let count = positions.len();
    let gravity_y = finite(DT * -GRAVITY, "gravity velocity")?;
    let velocities = filled(count, F3::new(0.0, gravity_y, 0.0))?;
    let mut rho_adv = filled(count, 0.0)?;
    let mut factor = filled(count, 0.0)?;
    let mut multiplier = filled(count, 0.0)?;
    let mut next = filled(count, 0.0)?;
    for index in 0..count {
        let delta = divergence_source(index, rows, &velocities)?;
        rho_adv[index] = finite(rows[index].rho_ratio + (DT * delta), "rho adv")?;
        factor[index] = finite(rows[index].alpha * INV_DT2, "density factor")?;
        let error = finite(rho_adv[index] - 1.0, "initial density error")?;
        let positive = if error > 0.0 { error } else { 0.0 };
        multiplier[index] = finite(positive * factor[index], "initial multiplier")?;
    }

    let mut selected_rows = selected_row_traces(positions, rows, &rho_adv, &factor, &multiplier)?;
    let mut errors = reserved(usize::from(DENSITY_MAX_ITERATIONS))?;
    let mut terminal_code = DENSITY_NONCONVERGENCE.to_owned();
    let mut terminal_detail = String::new();
    for iteration in 1..=DENSITY_MAX_ITERATIONS {
        let acceleration = pressure_acceleration(rows, &multiplier)?;
        let matrix = matrix_action(rows, &acceleration)?;
        let mut error_sum = 0.0;
        for index in 0..count {
            let s = finite(1.0 - rho_adv[index], "density s")?;
            let dt2_a = finite(DT2 * matrix[index], "density dt2 A")?;
            let correction = finite((s - dt2_a) * factor[index], "density correction")?;
            let candidate = finite(
                multiplier[index] - (RELAXATION * correction),
                "density candidate",
            )?;
            next[index] = if candidate > 0.0 { candidate } else { 0.0 };
            let row_error = finite((rho_adv[index] + dt2_a) - 1.0, "density error")?;
            let row_error = if row_error > 0.0 { row_error } else { 0.0 };
            error_sum = finite(error_sum + row_error, "density error fold")?;
            if let Some(slot) = SELECTED_ROWS
                .iter()
                .position(|(sample_id, _)| usize::try_from(*sample_id) == Ok(index))
            {
                selected_rows[slot]
                    .iterations
                    .push(DensityIterationRowTrace {
                        iteration,
                        multiplier_in_bits: scalar_bits(multiplier[index]),
                        acceleration_bits: [
                            scalar_bits(acceleration[index].x),
                            scalar_bits(acceleration[index].y),
                            scalar_bits(acceleration[index].z),
                        ],
                        matrix_action_bits: scalar_bits(matrix[index]),
                        error_bits: scalar_bits(row_error),
                        multiplier_out_bits: scalar_bits(next[index]),
                    });
            }
        }
        let mean = finite(error_sum / (count as f64), "density mean")?;
        let error_ppb = quantize_ppb(mean)?;
        errors.push(error_ppb);
        mem::swap(&mut multiplier, &mut next);
        if iteration >= DENSITY_MIN_ITERATIONS && error_ppb <= DENSITY_THRESHOLD_PPB {
            terminal_code = "COMPLETED".to_owned();
            terminal_detail = format!("converged at iteration {iteration} with {error_ppb} ppb");
            break;
        }
    }
    if terminal_detail.is_empty() {
        let final_error = errors.last().copied().ok_or_else(|| {
            WaterError::new(AUDIT_INVALID, "independent density trace has no iterations")
        })?;
        terminal_detail = format!("iteration 20 ended at {final_error} ppb");
    }
    Ok(HydroAuditTrace {
        divergence_iterations: 1,
        divergence_error_ppb: 0,
        density_error_ppb_by_iteration: errors,
        density_terminal_code: terminal_code,
        density_terminal_detail: terminal_detail,
        rows: selected_rows,
    })
}

fn selected_row_traces(
    positions: &[I3],
    rows: &[Row],
    rho_adv: &[f64],
    factor: &[f64],
    multiplier: &[f64],
) -> Result<Vec<HydroRowTrace>, WaterError> {
    let mut result = reserved(SELECTED_ROWS.len())?;
    for (sample_id, role) in SELECTED_ROWS {
        let index = usize::try_from(sample_id)
            .map_err(|_| WaterError::new(AUDIT_INVALID, "selected id conversion overflow"))?;
        let row = rows.get(index).ok_or_else(|| {
            WaterError::new(
                AUDIT_INVALID,
                format!("selected row {sample_id} is missing"),
            )
        })?;
        let mut fluid_neighbor_ids = reserved(row.fluid.len())?;
        for neighbor in &row.fluid {
            fluid_neighbor_ids.push(u32::try_from(neighbor.other).map_err(|_| {
                WaterError::new(AUDIT_INVALID, "fluid neighbor id conversion overflow")
            })?);
        }
        let mut boundary_neighbors = reserved(row.boundary.len())?;
        for neighbor in &row.boundary {
            boundary_neighbors.push(BoundaryNeighborTrace {
                id: u32::try_from(neighbor.boundary).map_err(|_| {
                    WaterError::new(AUDIT_INVALID, "boundary neighbor id conversion overflow")
                })?,
                volume_bits: scalar_bits(neighbor.volume),
            });
        }
        let iterations = reserved(usize::from(DENSITY_MAX_ITERATIONS))?;
        result.push(HydroRowTrace {
            role: role.to_owned(),
            sample_id,
            position_um: positions[index].common(),
            fluid_neighbor_ids,
            boundary_neighbors,
            initial: Some(DensityInitialTrace {
                rho_ratio_bits: scalar_bits(row.rho_ratio),
                alpha_bits: scalar_bits(row.alpha),
                rho_adv_bits: scalar_bits(rho_adv[index]),
                factor_bits: scalar_bits(factor[index]),
                multiplier_bits: scalar_bits(multiplier[index]),
            }),
            iterations,
        });
    }
    Ok(result)
}

fn divergence_source(index: usize, rows: &[Row], velocities: &[F3]) -> Result<f64, WaterError> {
    let mut result = 0.0;
    for neighbor in &rows[index].fluid {
        let relative = velocities[index].sub(velocities[neighbor.other]);
        let term = finite(
            REST_VOLUME * relative.dot(neighbor.gradient),
            "fluid divergence term",
        )?;
        result = finite(result + term, "fluid divergence fold")?;
    }
    for neighbor in &rows[index].boundary {
        let term = finite(
            neighbor.volume * velocities[index].dot(neighbor.gradient),
            "boundary divergence term",
        )?;
        result = finite(result + term, "boundary divergence fold")?;
    }
    Ok(result)
}

fn pressure_acceleration(rows: &[Row], multiplier: &[f64]) -> Result<Vec<F3>, WaterError> {
    let mut result = reserved(rows.len())?;
    for (index, row) in rows.iter().enumerate() {
        let mut value = F3::ZERO;
        for neighbor in &row.fluid {
            let pressure_sum = finite(
                multiplier[index] + multiplier[neighbor.other],
                "pressure sum",
            )?;
            if pressure_sum.abs() > SOLVER_EPSILON {
                let scale = finite(-(REST_VOLUME * pressure_sum), "fluid pressure scale")?;
                value = finite_vec(value.add(neighbor.gradient.scale(scale)), "fluid pressure")?;
            }
        }
        let mut boundary_value = F3::ZERO;
        if multiplier[index].abs() > SOLVER_EPSILON {
            for neighbor in &row.boundary {
                let scale = finite(
                    -(neighbor.volume * multiplier[index]),
                    "boundary pressure scale",
                )?;
                boundary_value = finite_vec(
                    boundary_value.add(neighbor.gradient.scale(scale)),
                    "boundary pressure",
                )?;
            }
        }
        result.push(finite_vec(value.add(boundary_value), "total pressure")?);
    }
    Ok(result)
}

fn matrix_action(rows: &[Row], acceleration: &[F3]) -> Result<Vec<f64>, WaterError> {
    let mut result = reserved(rows.len())?;
    for (index, row) in rows.iter().enumerate() {
        let mut value = 0.0;
        for neighbor in &row.fluid {
            let relative = acceleration[index].sub(acceleration[neighbor.other]);
            let term = finite(
                REST_VOLUME * relative.dot(neighbor.gradient),
                "fluid matrix term",
            )?;
            value = finite(value + term, "fluid matrix fold")?;
        }
        for neighbor in &row.boundary {
            let term = finite(
                neighbor.volume * acceleration[index].dot(neighbor.gradient),
                "boundary matrix term",
            )?;
            value = finite(value + term, "boundary matrix fold")?;
        }
        result.push(value);
    }
    Ok(result)
}

fn kernel(displacement_um: I3) -> Result<Kernel, WaterError> {
    let dx = (displacement_um.x as f64) / SCALE;
    let dy = (displacement_um.y as f64) / SCALE;
    let dz = (displacement_um.z as f64) / SCALE;
    kernel_components(dx, dy, dz, displacement_um.distance_squared() == 0)
}

fn kernel_f3(displacement: F3) -> Result<Kernel, WaterError> {
    kernel_components(
        displacement.x,
        displacement.y,
        displacement.z,
        displacement.x == 0.0 && displacement.y == 0.0 && displacement.z == 0.0,
    )
}

fn kernel_components(
    dx: f64,
    dy: f64,
    dz: f64,
    displacement_is_zero: bool,
) -> Result<Kernel, WaterError> {
    let r2_xy = (dx * dx) + (dy * dy);
    let r2 = finite(r2_xy + (dz * dz), "kernel r2")?;
    let r = finite(r2.sqrt(), "kernel r")?;
    let q = finite(r / SUPPORT_RADIUS, "kernel q")?;
    if q > 1.0 {
        return Ok(Kernel {
            value: 0.0,
            gradient: F3::ZERO,
        });
    }
    let value = if q <= 0.5 {
        let q2 = finite(q * q, "kernel q2")?;
        let q3 = finite(q2 * q, "kernel q3")?;
        let six_q3 = finite(6.0 * q3, "kernel six q3")?;
        let six_q2 = finite(6.0 * q2, "kernel six q2")?;
        let polynomial = finite((six_q3 - six_q2) + 1.0, "kernel polynomial")?;
        finite(KERNEL_K * polynomial, "kernel value inner")?
    } else {
        let t = finite(1.0 - q, "kernel t")?;
        let t2 = finite(t * t, "kernel t2")?;
        let t3 = finite(t2 * t, "kernel t3")?;
        finite(KERNEL_K * (2.0 * t3), "kernel value outer")?
    };
    let gradient = if displacement_is_zero {
        F3::ZERO
    } else {
        let grad_q = F3::new(
            finite((dx / r) / SUPPORT_RADIUS, "kernel gradq x")?,
            finite((dy / r) / SUPPORT_RADIUS, "kernel gradq y")?,
            finite((dz / r) / SUPPORT_RADIUS, "kernel gradq z")?,
        );
        let coefficient = if q <= 0.5 {
            let three_q = finite(3.0 * q, "kernel three q")?;
            let lq = finite(KERNEL_L * q, "kernel lq")?;
            finite(lq * (three_q - 2.0), "kernel inner gradient")?
        } else {
            let t = finite(1.0 - q, "kernel gradient t")?;
            let t2 = finite(t * t, "kernel gradient t2")?;
            finite(-(KERNEL_L * t2), "kernel outer gradient")?
        };
        finite_vec(grad_q.scale(coefficient), "kernel gradient vector")?
    };
    Ok(Kernel { value, gradient })
}

fn quantize_ppb(value: f64) -> Result<i64, WaterError> {
    quantize_scaled(value, 1_000_000_000)
}

fn quantize_scaled(value: f64, scale: u64) -> Result<i64, WaterError> {
    if !value.is_finite() {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "nonfinite independent publication input",
        ));
    }
    let bits = value.to_bits();
    let negative = (bits >> 63) != 0;
    let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    if raw_exponent == 0 && fraction == 0 {
        return Ok(0);
    }
    let (significand, exponent) = if raw_exponent == 0 {
        (u128::from(fraction), -1074)
    } else {
        (
            u128::from((1_u64 << 52) | fraction),
            raw_exponent - 1023 - 52,
        )
    };
    let scaled = significand
        .checked_mul(u128::from(scale))
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "publication multiply overflow"))?;
    let magnitude = if exponent >= 0 {
        let shift = exponent as u32;
        if shift >= u128::BITS || scaled > (u128::MAX >> shift) {
            return Err(WaterError::new(AUDIT_INVALID, "publication shift overflow"));
        }
        scaled << shift
    } else {
        round_power_of_two(scaled, exponent.unsigned_abs())
    };
    let magnitude = i64::try_from(magnitude)
        .map_err(|_| WaterError::new(AUDIT_INVALID, "publication result overflow"))?;
    Ok(if negative { -magnitude } else { magnitude })
}

fn round_power_of_two(numerator: u128, shift: u32) -> u128 {
    if shift > 128 {
        return 0;
    }
    if shift == 128 {
        return u128::from(numerator > (1_u128 << 127));
    }
    let divisor = 1_u128 << shift;
    let quotient = numerator / divisor;
    let remainder = numerator % divisor;
    let half = divisor >> 1;
    if remainder > half || (remainder == half && (quotient & 1) != 0) {
        quotient + 1
    } else {
        quotient
    }
}

fn finite(value: f64, phase: &str) -> Result<f64, WaterError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(WaterError::new(
            AUDIT_INVALID,
            format!("nonfinite independent scalar after {phase}"),
        ))
    }
}

fn finite_vec(value: F3, phase: &str) -> Result<F3, WaterError> {
    if value.x.is_finite() && value.y.is_finite() && value.z.is_finite() {
        Ok(value)
    } else {
        Err(WaterError::new(
            AUDIT_INVALID,
            format!("nonfinite independent vector after {phase}"),
        ))
    }
}

fn reserved<T>(capacity: usize) -> Result<Vec<T>, WaterError> {
    let mut result = Vec::new();
    result.try_reserve_exact(capacity).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("independent audit allocation failed: {error}"),
        )
    })?;
    Ok(result)
}

fn filled<T: Clone>(count: usize, value: T) -> Result<Vec<T>, WaterError> {
    let mut result = reserved(count)?;
    result.resize(count, value);
    Ok(result)
}

fn require_count(label: &str, actual: usize, expected: usize) -> Result<(), WaterError> {
    if actual == expected {
        Ok(())
    } else {
        Err(WaterError::new(
            AUDIT_INVALID,
            format!("independent {label} count {actual}, expected {expected}"),
        ))
    }
}
