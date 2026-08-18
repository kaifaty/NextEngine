#![forbid(unsafe_code)]

use super::*;
use crate::profile::{PARTICLE_RADIUS, UNIFORM_MASS};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VelocityProjectionResult {
    pub(crate) fluid_impulse: Vec3f,
    pub(crate) active_rows: usize,
    pub(crate) active_components: usize,
    pub(crate) maximum_absolute_delta_velocity: f64,
}

pub(super) fn project_predictive_outer_box(
    geometry: Geometry,
    positions: &[Vec3f],
    velocities: &mut [Vec3f],
) -> Result<VelocityProjectionResult, WaterError> {
    let lower = Vec3f::new(
        checked_scalar(
            decode_micrometres(geometry.bounds.min.x)? + PARTICLE_RADIUS,
            "predictive contact lower x",
        )?,
        checked_scalar(
            decode_micrometres(geometry.bounds.min.y)? + PARTICLE_RADIUS,
            "predictive contact lower y",
        )?,
        checked_scalar(
            decode_micrometres(geometry.bounds.min.z)? + PARTICLE_RADIUS,
            "predictive contact lower z",
        )?,
    );
    let upper = Vec3f::new(
        checked_scalar(
            decode_micrometres(geometry.bounds.max.x)? - PARTICLE_RADIUS,
            "predictive contact upper x",
        )?,
        checked_scalar(
            decode_micrometres(geometry.bounds.max.y)? - PARTICLE_RADIUS,
            "predictive contact upper y",
        )?,
        checked_scalar(
            decode_micrometres(geometry.bounds.max.z)? - PARTICLE_RADIUS,
            "predictive contact upper z",
        )?,
    );
    if lower.x > upper.x || lower.y > upper.y || lower.z > upper.z {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            "predictive contact box is smaller than one particle diameter",
        ));
    }

    let mut result = VelocityProjectionResult::default();
    for (position, velocity) in positions.iter().copied().zip(velocities.iter_mut()) {
        let before = *velocity;
        velocity.x = project_axis(position.x, velocity.x, lower.x, upper.x, "x")?;
        velocity.y = project_axis(position.y, velocity.y, lower.y, upper.y, "y")?;
        velocity.z = project_axis(position.z, velocity.z, lower.z, upper.z, "z")?;
        let delta = velocity
            .sub(before)
            .checked("predictive contact delta velocity")?;
        let components = [delta.x, delta.y, delta.z];
        let active_components = components
            .iter()
            .filter(|component| **component != 0.0)
            .count();
        if active_components == 0 {
            continue;
        }
        result.active_rows = result.active_rows.checked_add(1).ok_or_else(|| {
            WaterError::new(NUMERIC_OVERFLOW, "predictive contact row count overflow")
        })?;
        result.active_components = result
            .active_components
            .checked_add(active_components)
            .ok_or_else(|| {
                WaterError::new(
                    NUMERIC_OVERFLOW,
                    "predictive contact component count overflow",
                )
            })?;
        for component in components {
            result.maximum_absolute_delta_velocity =
                result.maximum_absolute_delta_velocity.max(component.abs());
        }
        let impulse = delta
            .scale(UNIFORM_MASS)
            .checked("predictive contact fluid impulse row")?;
        result.fluid_impulse = result
            .fluid_impulse
            .add(impulse)
            .checked("predictive contact fluid impulse reduction")?;
    }
    Ok(result)
}

fn project_axis(
    position: f64,
    velocity: f64,
    lower: f64,
    upper: f64,
    axis: &str,
) -> Result<f64, WaterError> {
    let minimum_velocity = checked_scalar(
        (lower - position) * INV_DT,
        &format!("predictive contact minimum {axis} velocity"),
    )?;
    let maximum_velocity = checked_scalar(
        (upper - position) * INV_DT,
        &format!("predictive contact maximum {axis} velocity"),
    )?;
    Ok(velocity.clamp(minimum_velocity, maximum_velocity))
}

pub(super) fn solve_density_projected_pcg(
    reconstruction: &Reconstruction,
    velocities: &mut [Vec3f],
    maximum_iterations: u8,
) -> Result<SolveResult, WaterError> {
    let count = velocities.len();
    let mut right_hand_side = filled_vec(count, 0.0)?;
    let mut preconditioner = filled_vec(count, 0.0)?;
    for index in 0..count {
        let delta = divergence_source(
            index,
            reconstruction.rows[index],
            reconstruction,
            velocities,
        )?;
        let rho_adv = checked_scalar(
            reconstruction.rho_ratio[index] + (DT * delta),
            "projected PCG advected density ratio",
        )?;
        right_hand_side[index] = checked_scalar(rho_adv - 1.0, "projected PCG right-hand side")?;
        preconditioner[index] = checked_scalar(
            reconstruction.alpha[index] * INV_DT2,
            "projected PCG diagonal preconditioner",
        )?;
    }

    let mut multiplier = filled_vec(count, 0.0)?;
    let mut residual = filled_vec(count, 0.0)?;
    residual.copy_from_slice(&right_hand_side);
    let mut active = filled_vec(count, false)?;
    let mut preconditioned = filled_vec(count, 0.0)?;
    for index in 0..count {
        active[index] = residual[index] > 0.0;
        preconditioned[index] = if active[index] {
            checked_scalar(
                preconditioner[index] * residual[index],
                "projected PCG initial preconditioned residual",
            )?
        } else {
            0.0
        };
    }
    let mut direction = filled_vec(count, 0.0)?;
    direction.copy_from_slice(&preconditioned);
    let mut residual_dot_preconditioned = checked_dot(
        &residual,
        &preconditioned,
        "projected PCG initial residual product",
    )?;
    let mut error_ppb = density_residual_ppb(&residual)?;
    let mut accepted_iteration = 0_u8;

    for iteration in 1..=maximum_iterations {
        if residual_dot_preconditioned > SOLVER_EPSILON {
            let action = density_pressure_operator(reconstruction, &direction)?;
            let direction_dot_action = checked_dot(
                &direction,
                &action,
                "projected PCG direction action product",
            )?;
            if direction_dot_action <= SOLVER_EPSILON {
                break;
            }
            let step = checked_scalar(
                residual_dot_preconditioned / direction_dot_action,
                "projected PCG step",
            )?;
            let mut projection_changed = false;
            for index in 0..count {
                let candidate = checked_scalar(
                    multiplier[index] + (step * direction[index]),
                    "projected PCG multiplier candidate",
                )?;
                if candidate > 0.0 {
                    multiplier[index] = candidate;
                } else {
                    projection_changed |= candidate < 0.0;
                    multiplier[index] = 0.0;
                }
            }

            let multiplier_action = density_pressure_operator(reconstruction, &multiplier)?;
            for index in 0..count {
                residual[index] = checked_scalar(
                    right_hand_side[index] - multiplier_action[index],
                    "projected PCG residual",
                )?;
            }
            error_ppb = density_residual_ppb(&residual)?;
            if converged(
                iteration,
                DENSITY_MIN_ITERATIONS,
                error_ppb,
                DENSITY_THRESHOLD_PPB,
            ) {
                accepted_iteration = iteration;
                break;
            }

            let mut active_changed = false;
            for index in 0..count {
                let next_active = multiplier[index] > 0.0 || residual[index] > 0.0;
                active_changed |= next_active != active[index];
                active[index] = next_active;
                preconditioned[index] = if next_active {
                    checked_scalar(
                        preconditioner[index] * residual[index],
                        "projected PCG preconditioned residual",
                    )?
                } else {
                    0.0
                };
            }
            let next_residual_dot_preconditioned = checked_dot(
                &residual,
                &preconditioned,
                "projected PCG next residual product",
            )?;
            let beta = if projection_changed
                || active_changed
                || residual_dot_preconditioned <= SOLVER_EPSILON
            {
                0.0
            } else {
                checked_scalar(
                    next_residual_dot_preconditioned / residual_dot_preconditioned,
                    "projected PCG beta",
                )?
            };
            for index in 0..count {
                direction[index] = if active[index] {
                    checked_scalar(
                        preconditioned[index] + (beta * direction[index]),
                        "projected PCG direction",
                    )?
                } else {
                    0.0
                };
            }
            residual_dot_preconditioned = next_residual_dot_preconditioned;
        } else {
            error_ppb = density_residual_ppb(&residual)?;
            if converged(
                iteration,
                DENSITY_MIN_ITERATIONS,
                error_ppb,
                DENSITY_THRESHOLD_PPB,
            ) {
                accepted_iteration = iteration;
                break;
            }
        }
    }

    if accepted_iteration == 0 {
        return Err(WaterError::new(
            DENSITY_NONCONVERGENCE,
            format!("projected PCG iteration {maximum_iterations} ended at {error_ppb} ppb"),
        ));
    }
    let acceleration = pressure_acceleration(reconstruction, &multiplier)?;
    let boundary_impulse = apply_acceleration(velocities, &acceleration)?;
    let maximum_multiplier_bits = maximum_multiplier_bits(&multiplier)?;
    Ok(SolveResult {
        iterations: accepted_iteration,
        error_ppb,
        maximum_multiplier_bits,
        boundary_impulse,
    })
}

pub(super) fn density_pressure_operator(
    reconstruction: &Reconstruction,
    multiplier: &[f64],
) -> Result<Vec<f64>, WaterError> {
    let acceleration = pressure_acceleration(reconstruction, multiplier)?;
    let matrix = matrix_action(reconstruction, &acceleration.total)?;
    let mut result = Vec::new();
    result.try_reserve_exact(matrix.len()).map_err(heap_error)?;
    for value in matrix {
        result.push(checked_scalar(
            -(DT2 * value),
            "projected PCG positive pressure operator",
        )?);
    }
    Ok(result)
}

fn density_residual_ppb(residual: &[f64]) -> Result<i64, WaterError> {
    if residual.is_empty() {
        return Ok(0);
    }
    let mut error_sum = 0.0;
    for value in residual {
        let compression = if *value > 0.0 { *value } else { 0.0 };
        error_sum = checked_scalar(
            error_sum + compression,
            "projected PCG density error reduction",
        )?;
    }
    let mean = checked_scalar(
        error_sum / (residual.len() as f64),
        "projected PCG density error mean",
    )?;
    quantize_ppb(mean)
}

pub(super) fn checked_dot(left: &[f64], right: &[f64], phase: &str) -> Result<f64, WaterError> {
    if left.len() != right.len() {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            format!("{phase} vector length mismatch"),
        ));
    }
    let mut result = 0.0;
    for (left, right) in left.iter().zip(right) {
        result = checked_scalar(result + (left * right), phase)?;
    }
    Ok(result)
}
