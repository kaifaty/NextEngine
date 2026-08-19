#![forbid(unsafe_code)]

use crate::audit::{AuditFluidInput, fluid_input_root};
use crate::calibration::{
    SETTLING_DENSITY_CEILING, SETTLING_MAX_PASSES, SETTLING_MAXIMUM_DISPLACEMENT_UM,
    SETTLING_READY_STREAK, SETTLING_TREND_TRANSITIONS, SettlingComputation, SettlingPassTrace,
    SettlingTrace,
};
use crate::error::{BOUNDARY_ESCAPE, BOUNDARY_PENETRATION_LIMIT};

use super::*;

const GHOST_BOUNDARY_COUNT: usize = 2_648;
const OUTER_MAXIMUM_UM: i64 = 1_000_000;
const PARTICLE_RADIUS_UM: i64 = 25_000;
const MINIMUM_CLEARANCE_UM: i64 = 22_500;

enum StepAttempt {
    Accepted(IndependentStep),
    Rejected { code: String, detail: String },
}

struct IndependentStep {
    positions: Vec<I3>,
    density_iterations: u8,
    density_error_ppb: i64,
    maximum_displacement_um: i64,
    maximum_penetration_um: i64,
    centre_of_mass_y_um: i64,
}

pub(crate) fn compute() -> Result<SettlingComputation, WaterError> {
    let mut positions = generate_fluid()?;
    let boundary_positions = generate_ghost_boundary()?;
    let boundary_volumes = filled(boundary_positions.len(), REST_VOLUME)?;
    let mut final_fluid = fluid_from_positions(&positions)?;
    let mut passes = reserved(usize::from(SETTLING_MAX_PASSES))?;
    let mut ready_streak = 0_u8;
    for pass in 1..=SETTLING_MAX_PASSES {
        let step = match settle_step(&positions, &boundary_positions, &boundary_volumes)? {
            StepAttempt::Accepted(step) => step,
            StepAttempt::Rejected { code, detail } => {
                return Ok(SettlingComputation {
                    trace: SettlingTrace {
                        status: "GENERATOR_REJECTED".to_owned(),
                        requested_maximum_passes: SETTLING_MAX_PASSES,
                        completed_passes: pass - 1,
                        terminal_pass: Some(pass),
                        terminal_code: code,
                        terminal_detail: detail,
                        passes,
                    },
                    final_fluid,
                });
            }
        };
        positions = step.positions;
        final_fluid = fluid_from_positions(&positions)?;
        let production_ceiling_ready = step.density_iterations <= DENSITY_MAX_ITERATIONS
            && step.maximum_displacement_um <= SETTLING_MAXIMUM_DISPLACEMENT_UM;
        if production_ceiling_ready {
            ready_streak = ready_streak.checked_add(1).ok_or_else(|| {
                WaterError::new(AUDIT_INVALID, "independent settling ready streak overflow")
            })?;
        } else {
            ready_streak = 0;
        }
        passes.push(SettlingPassTrace {
            pass,
            density_iterations: step.density_iterations,
            density_error_ppb: step.density_error_ppb,
            maximum_displacement_um: step.maximum_displacement_um,
            maximum_penetration_um: step.maximum_penetration_um,
            centre_of_mass_y_um: step.centre_of_mass_y_um,
            zero_velocity_state_root: fluid_input_root(&final_fluid),
            production_ceiling_ready,
        });
        if ready_streak >= SETTLING_READY_STREAK {
            return Ok(SettlingComputation {
                trace: SettlingTrace {
                    status: "CONVERGED".to_owned(),
                    requested_maximum_passes: SETTLING_MAX_PASSES,
                    completed_passes: pass,
                    terminal_pass: Some(pass),
                    terminal_code: "COMPLETED".to_owned(),
                    terminal_detail: format!(
                        "settling met the production ceiling and displacement target for {SETTLING_READY_STREAK} consecutive passes"
                    ),
                    passes,
                },
                final_fluid,
            });
        }
        if has_adverse_trend(&passes) {
            return Ok(SettlingComputation {
                trace: SettlingTrace {
                    status: "GENERATOR_REJECTED".to_owned(),
                    requested_maximum_passes: SETTLING_MAX_PASSES,
                    completed_passes: pass,
                    terminal_pass: Some(pass),
                    terminal_code: "SETTLING_ADVERSE_TREND".to_owned(),
                    terminal_detail: format!(
                        "density iteration demand and penetration increased for {SETTLING_TREND_TRANSITIONS} consecutive transitions"
                    ),
                    passes,
                },
                final_fluid,
            });
        }
    }
    Ok(SettlingComputation {
        trace: SettlingTrace {
            status: "GENERATOR_REJECTED".to_owned(),
            requested_maximum_passes: SETTLING_MAX_PASSES,
            completed_passes: SETTLING_MAX_PASSES,
            terminal_pass: Some(SETTLING_MAX_PASSES),
            terminal_code: "SETTLING_PASS_LIMIT_EXHAUSTED".to_owned(),
            terminal_detail: format!(
                "settling did not meet its convergence rule in {SETTLING_MAX_PASSES} passes"
            ),
            passes,
        },
        final_fluid,
    })
}

fn settle_step(
    positions: &[I3],
    boundary_positions: &[I3],
    boundary_volumes: &[f64],
) -> Result<StepAttempt, WaterError> {
    let rows = reconstruct(positions, boundary_positions, boundary_volumes)?;
    let count = positions.len();
    let gravity_y = finite(DT * -GRAVITY, "settling gravity velocity")?;
    let mut velocities = filled(count, F3::new(0.0, gravity_y, 0.0))?;
    let mut rho_adv = filled(count, 0.0)?;
    let mut factor = filled(count, 0.0)?;
    let mut multiplier = filled(count, 0.0)?;
    let mut next = filled(count, 0.0)?;
    for index in 0..count {
        let delta = divergence_source(index, &rows, &velocities)?;
        rho_adv[index] = finite(rows[index].rho_ratio + (DT * delta), "settling rho adv")?;
        factor[index] = finite(rows[index].alpha * INV_DT2, "settling density factor")?;
        let error = finite(rho_adv[index] - 1.0, "settling initial density error")?;
        let positive = if error > 0.0 { error } else { 0.0 };
        multiplier[index] = finite(positive * factor[index], "settling initial multiplier")?;
    }

    let mut accepted_acceleration = None;
    let mut accepted_iteration = 0_u8;
    let mut error_ppb = i64::MAX;
    for iteration in 1..=SETTLING_DENSITY_CEILING {
        let acceleration = pressure_acceleration(&rows, &multiplier)?;
        let matrix = matrix_action(&rows, &acceleration)?;
        let mut error_sum = 0.0;
        for index in 0..count {
            let source = finite(1.0 - rho_adv[index], "settling density source")?;
            let dt2_action = finite(DT2 * matrix[index], "settling density matrix action")?;
            let correction = finite(
                (source - dt2_action) * factor[index],
                "settling density correction",
            )?;
            let candidate = finite(
                multiplier[index] - (RELAXATION * correction),
                "settling density candidate",
            )?;
            next[index] = if candidate > 0.0 { candidate } else { 0.0 };
            let error = finite(
                (rho_adv[index] + dt2_action) - 1.0,
                "settling density error",
            )?;
            let error = if error > 0.0 { error } else { 0.0 };
            error_sum = finite(error_sum + error, "settling density error fold")?;
        }
        let mean = finite(error_sum / (count as f64), "settling density error mean")?;
        error_ppb = quantize_ppb(mean)?;
        mem::swap(&mut multiplier, &mut next);
        if iteration >= DENSITY_MIN_ITERATIONS && error_ppb <= DENSITY_THRESHOLD_PPB {
            accepted_iteration = iteration;
            accepted_acceleration = Some(pressure_acceleration(&rows, &multiplier)?);
            break;
        }
    }
    if accepted_iteration == 0 {
        return Ok(StepAttempt::Rejected {
            code: DENSITY_NONCONVERGENCE.to_owned(),
            detail: format!("iteration {SETTLING_DENSITY_CEILING} ended at {error_ppb} ppb"),
        });
    }
    let acceleration = accepted_acceleration.ok_or_else(|| {
        WaterError::new(
            AUDIT_INVALID,
            "independent settling acceleration is missing",
        )
    })?;
    for index in 0..count {
        velocities[index] = finite_vec(
            velocities[index].add(acceleration[index].scale(DT)),
            "settling accepted velocity",
        )?;
    }
    let mut published = reserved(count)?;
    for index in 0..count {
        let decoded = F3::new(
            (positions[index].x as f64) / SCALE,
            (positions[index].y as f64) / SCALE,
            (positions[index].z as f64) / SCALE,
        );
        let position = finite_vec(
            decoded.add(velocities[index].scale(DT)),
            "settling integrated position",
        )?;
        published.push(I3::new(
            quantize_scaled(position.x, 1_000_000)?,
            quantize_scaled(position.y, 1_000_000)?,
            quantize_scaled(position.z, 1_000_000)?,
        ));
    }
    let maximum_penetration_um = match validate_positions(&published) {
        Ok(value) => value,
        Err((code, detail)) => return Ok(StepAttempt::Rejected { code, detail }),
    };
    Ok(StepAttempt::Accepted(IndependentStep {
        maximum_displacement_um: maximum_displacement(positions, &published)?,
        maximum_penetration_um,
        centre_of_mass_y_um: centre_of_mass_y(&published)?,
        positions: published,
        density_iterations: accepted_iteration,
        density_error_ppb: error_ppb,
    }))
}

fn generate_ghost_boundary() -> Result<Vec<I3>, WaterError> {
    let mut result = reserved(GHOST_BOUNDARY_COUNT)?;
    for ix in -1_i64..=20 {
        for iy in -1_i64..=20 {
            for iz in -1_i64..=20 {
                if ix == -1 || ix == 20 || iy == -1 || iy == 20 || iz == -1 || iz == 20 {
                    result.push(I3::new(
                        25_000 + (50_000 * ix),
                        25_000 + (50_000 * iy),
                        25_000 + (50_000 * iz),
                    ));
                }
            }
        }
    }
    require_count(
        "settling ghost boundary",
        result.len(),
        GHOST_BOUNDARY_COUNT,
    )?;
    Ok(result)
}

fn fluid_from_positions(positions: &[I3]) -> Result<Vec<AuditFluidInput>, WaterError> {
    let mut result = reserved(positions.len())?;
    for (index, position) in positions.iter().copied().enumerate() {
        result.push(AuditFluidInput {
            id: u32::try_from(index)
                .map_err(|_| WaterError::new(AUDIT_INVALID, "settling fluid id overflow"))?,
            position_um: position.common(),
            velocity_um_s: Vec3i::new(0, 0, 0),
        });
    }
    Ok(result)
}

fn maximum_displacement(prior: &[I3], next: &[I3]) -> Result<i64, WaterError> {
    if prior.len() != next.len() {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "independent settling displacement counts differ",
        ));
    }
    let mut maximum = 0_u64;
    for (left, right) in prior.iter().zip(next) {
        maximum = maximum
            .max(left.x.abs_diff(right.x))
            .max(left.y.abs_diff(right.y))
            .max(left.z.abs_diff(right.z));
    }
    i64::try_from(maximum).map_err(|_| {
        WaterError::new(
            AUDIT_INVALID,
            "independent settling displacement exceeds i64",
        )
    })
}

fn validate_positions(positions: &[I3]) -> Result<i64, (String, String)> {
    let mut maximum_penetration = 0_i64;
    for (index, position) in positions.iter().copied().enumerate() {
        if position.x < 0
            || position.x > OUTER_MAXIMUM_UM
            || position.y < 0
            || position.y > OUTER_MAXIMUM_UM
            || position.z < 0
            || position.z > OUTER_MAXIMUM_UM
        {
            return Err((
                BOUNDARY_ESCAPE.to_owned(),
                format!("sample {index} escaped the outer box"),
            ));
        }
        let clearance = [
            position.x,
            OUTER_MAXIMUM_UM - position.x,
            position.y,
            OUTER_MAXIMUM_UM - position.y,
            position.z,
            OUTER_MAXIMUM_UM - position.z,
        ]
        .into_iter()
        .min()
        .unwrap_or(i64::MAX);
        if clearance < MINIMUM_CLEARANCE_UM {
            return Err((
                BOUNDARY_PENETRATION_LIMIT.to_owned(),
                format!("sample {index} exceeds the 2500 um penetration limit"),
            ));
        }
        maximum_penetration = maximum_penetration.max((PARTICLE_RADIUS_UM - clearance).max(0));
    }
    Ok(maximum_penetration)
}

fn centre_of_mass_y(positions: &[I3]) -> Result<i64, WaterError> {
    if positions.is_empty() {
        return Ok(0);
    }
    let mut sum = 0_i128;
    for position in positions {
        sum = sum
            .checked_add(i128::from(position.y))
            .ok_or_else(|| WaterError::new(AUDIT_INVALID, "settling centre sum overflow"))?;
    }
    let divisor = i128::try_from(positions.len())
        .map_err(|_| WaterError::new(AUDIT_INVALID, "settling centre divisor overflow"))?;
    let quotient = sum / divisor;
    let remainder = sum % divisor;
    let twice_remainder = remainder
        .checked_mul(2)
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "settling centre remainder overflow"))?;
    let rounded =
        if twice_remainder > divisor || (twice_remainder == divisor && (quotient & 1) != 0) {
            quotient + 1
        } else {
            quotient
        };
    i64::try_from(rounded)
        .map_err(|_| WaterError::new(AUDIT_INVALID, "settling centre result overflow"))
}

fn has_adverse_trend(passes: &[SettlingPassTrace]) -> bool {
    let required = SETTLING_TREND_TRANSITIONS + 1;
    if passes.len() < required {
        return false;
    }
    passes[passes.len() - required..].windows(2).all(|pair| {
        !pair[1].production_ceiling_ready
            && pair[1].density_iterations > pair[0].density_iterations
            && pair[1].maximum_penetration_um > pair[0].maximum_penetration_um
    })
}
