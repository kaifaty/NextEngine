#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

use crate::audit::{AuditBoundaryInput, SELECTED_ROWS, scalar_bits};
use crate::calibration::{
    BoundaryFeatureContribution, CandidateCalibrationComputation, DIAGNOSTIC_CHECKPOINTS,
    DIAGNOSTIC_MAX_ITERATIONS, DensityContributionRow, ExtendedDensityCheckpoint,
    ExtendedDensityTrace, HydroCalibrationTrace, PressureOperatorDiagonalProbe,
    PressureOperatorProbe, ProjectedPcgFirstStepProbe,
};

use super::*;

pub(crate) fn compute() -> Result<HydroCalibrationTrace, WaterError> {
    let positions = generate_fluid()?;
    let boundary_positions = generate_boundary()?;
    let boundary_volumes = boundary_volumes(&boundary_positions)?;
    compute_trace(&positions, &boundary_positions, &boundary_volumes)
}

pub(crate) fn compute_ghost() -> Result<CandidateCalibrationComputation, WaterError> {
    let positions = generate_fluid()?;
    let boundary_positions = generate_ghost_boundary()?;
    compute_lattice_candidate(&positions, boundary_positions, "ghost")
}

pub(crate) fn compute_support_complete() -> Result<CandidateCalibrationComputation, WaterError> {
    let positions = generate_fluid()?;
    let boundary_positions = generate_support_complete_boundary()?;
    compute_lattice_candidate(&positions, boundary_positions, "support-complete")
}

pub(crate) fn compute_support_complete_pressure_probe() -> Result<PressureOperatorProbe, WaterError>
{
    let positions = generate_fluid()?;
    let boundary_positions = generate_support_complete_boundary()?;
    let boundary_volumes = filled(boundary_positions.len(), REST_VOLUME)?;
    let rows = reconstruct(&positions, &boundary_positions, &boundary_volumes)?;
    let (vector_u, vector_v) = pressure_probe_vectors(positions.len())?;
    let action_u = pressure_operator(&rows, &vector_u)?;
    let action_v = pressure_operator(&rows, &vector_v)?;
    let u_dot_b_u = pressure_dot(&vector_u, &action_u, "independent probe u dot B u")?;
    let v_dot_b_v = pressure_dot(&vector_v, &action_v, "independent probe v dot B v")?;
    let u_dot_b_v = pressure_dot(&vector_u, &action_v, "independent probe u dot B v")?;
    let v_dot_b_u = pressure_dot(&vector_v, &action_u, "independent probe v dot B u")?;
    let mut selected_u_action_bits = reserved(SELECTED_ROWS.len())?;
    let mut selected_v_action_bits = reserved(SELECTED_ROWS.len())?;
    let mut diagonals = reserved(SELECTED_ROWS.len())?;
    for (sample_id, _role) in SELECTED_ROWS {
        let index = usize::try_from(sample_id)
            .map_err(|_| WaterError::new(AUDIT_INVALID, "pressure probe id overflow"))?;
        selected_u_action_bits.push(scalar_bits(action_u[index]));
        selected_v_action_bits.push(scalar_bits(action_v[index]));
        let mut basis = filled(positions.len(), 0.0)?;
        basis[index] = 1.0;
        let diagonal_action = pressure_operator(&rows, &basis)?[index];
        let factor_derived = if rows[index].alpha > 0.0 {
            finite(
                DT2 / rows[index].alpha,
                "independent factor-derived pressure diagonal",
            )?
        } else {
            0.0
        };
        diagonals.push(PressureOperatorDiagonalProbe {
            sample_id,
            action_bits: scalar_bits(diagonal_action),
            factor_derived_bits: scalar_bits(factor_derived),
            relative_difference_ppb: pressure_relative_difference_ppb(
                diagonal_action,
                factor_derived,
                "independent pressure diagonal relative difference",
            )?,
        });
    }
    Ok(PressureOperatorProbe {
        vector_u_action_root: pressure_action_root(&action_u),
        vector_v_action_root: pressure_action_root(&action_v),
        selected_u_action_bits,
        selected_v_action_bits,
        u_dot_b_u_bits: scalar_bits(u_dot_b_u),
        v_dot_b_v_bits: scalar_bits(v_dot_b_v),
        u_dot_b_v_bits: scalar_bits(u_dot_b_v),
        v_dot_b_u_bits: scalar_bits(v_dot_b_u),
        symmetry_relative_difference_ppb: pressure_relative_difference_ppb(
            u_dot_b_v,
            v_dot_b_u,
            "independent pressure symmetry relative difference",
        )?,
        diagonals,
    })
}

pub(crate) fn compute_support_complete_projected_pcg_first_step_probe()
-> Result<ProjectedPcgFirstStepProbe, WaterError> {
    let positions = generate_fluid()?;
    let boundary_positions = generate_support_complete_boundary()?;
    let boundary_volumes = filled(boundary_positions.len(), REST_VOLUME)?;
    let rows = reconstruct(&positions, &boundary_positions, &boundary_volumes)?;
    let count = positions.len();
    let gravity_y = finite(DT * -GRAVITY, "independent PCG gravity velocity")?;
    let velocities = filled(count, F3::new(0.0, gravity_y, 0.0))?;
    let mut right_hand_side = filled(count, 0.0)?;
    let mut preconditioner = filled(count, 0.0)?;
    for index in 0..count {
        let delta = divergence_source(index, &rows, &velocities)?;
        let rho_adv = finite(
            rows[index].rho_ratio + (DT * delta),
            "independent PCG advected density ratio",
        )?;
        right_hand_side[index] = finite(rho_adv - 1.0, "independent PCG right-hand side")?;
        preconditioner[index] = finite(
            rows[index].alpha * INV_DT2,
            "independent PCG diagonal preconditioner",
        )?;
    }
    let mut multiplier = filled(count, 0.0)?;
    let mut residual = filled(count, 0.0)?;
    residual.copy_from_slice(&right_hand_side);
    let mut active = filled(count, false)?;
    let mut preconditioned = filled(count, 0.0)?;
    for index in 0..count {
        active[index] = residual[index] > 0.0;
        preconditioned[index] = if active[index] {
            finite(
                preconditioner[index] * residual[index],
                "independent PCG initial preconditioned residual",
            )?
        } else {
            0.0
        };
    }
    let mut direction = filled(count, 0.0)?;
    direction.copy_from_slice(&preconditioned);
    let mut residual_dot_preconditioned = pressure_dot(
        &residual,
        &preconditioned,
        "independent PCG initial residual product",
    )?;
    let mut error_ppb = pressure_density_residual_ppb(&residual)?;
    let mut accepted_iteration = 0_u8;
    for iteration in 1..=50_u8 {
        if residual_dot_preconditioned > SOLVER_EPSILON {
            let action = pressure_operator(&rows, &direction)?;
            let direction_dot_action = pressure_dot(
                &direction,
                &action,
                "independent PCG direction action product",
            )?;
            if direction_dot_action <= SOLVER_EPSILON {
                break;
            }
            let step = finite(
                residual_dot_preconditioned / direction_dot_action,
                "independent PCG step",
            )?;
            let mut projection_changed = false;
            for index in 0..count {
                let candidate = finite(
                    multiplier[index] + (step * direction[index]),
                    "independent PCG multiplier candidate",
                )?;
                if candidate > 0.0 {
                    multiplier[index] = candidate;
                } else {
                    projection_changed |= candidate < 0.0;
                    multiplier[index] = 0.0;
                }
            }
            let multiplier_action = pressure_operator(&rows, &multiplier)?;
            for index in 0..count {
                residual[index] = finite(
                    right_hand_side[index] - multiplier_action[index],
                    "independent PCG residual",
                )?;
            }
            error_ppb = pressure_density_residual_ppb(&residual)?;
            if iteration >= DENSITY_MIN_ITERATIONS && error_ppb <= DENSITY_THRESHOLD_PPB {
                accepted_iteration = iteration;
                break;
            }
            let mut active_changed = false;
            for index in 0..count {
                let next_active = multiplier[index] > 0.0 || residual[index] > 0.0;
                active_changed |= next_active != active[index];
                active[index] = next_active;
                preconditioned[index] = if next_active {
                    finite(
                        preconditioner[index] * residual[index],
                        "independent PCG preconditioned residual",
                    )?
                } else {
                    0.0
                };
            }
            let next_residual_dot_preconditioned = pressure_dot(
                &residual,
                &preconditioned,
                "independent PCG next residual product",
            )?;
            let beta = if projection_changed
                || active_changed
                || residual_dot_preconditioned <= SOLVER_EPSILON
            {
                0.0
            } else {
                finite(
                    next_residual_dot_preconditioned / residual_dot_preconditioned,
                    "independent PCG beta",
                )?
            };
            for index in 0..count {
                direction[index] = if active[index] {
                    finite(
                        preconditioned[index] + (beta * direction[index]),
                        "independent PCG direction",
                    )?
                } else {
                    0.0
                };
            }
            residual_dot_preconditioned = next_residual_dot_preconditioned;
        } else {
            error_ppb = pressure_density_residual_ppb(&residual)?;
            if iteration >= DENSITY_MIN_ITERATIONS && error_ppb <= DENSITY_THRESHOLD_PPB {
                accepted_iteration = iteration;
                break;
            }
        }
    }
    if accepted_iteration == 0 {
        return Err(WaterError::new(
            DENSITY_NONCONVERGENCE,
            format!("independent projected PCG iteration 50 ended at {error_ppb} ppb"),
        ));
    }
    let mut maximum_multiplier = 0.0;
    for value in multiplier {
        if value > maximum_multiplier {
            maximum_multiplier = value;
        }
    }
    Ok(ProjectedPcgFirstStepProbe {
        density_iterations: accepted_iteration,
        density_error_ppb: error_ppb,
        maximum_multiplier_bits: scalar_bits(maximum_multiplier),
    })
}

fn pressure_probe_vectors(count: usize) -> Result<(Vec<f64>, Vec<f64>), WaterError> {
    let mut vector_u = filled(count, 0.0)?;
    let mut vector_v = filled(count, 0.0)?;
    for index in 0..count {
        let id = u32::try_from(index)
            .map_err(|_| WaterError::new(AUDIT_INVALID, "pressure probe id overflow"))?;
        if id % 5 == 0 {
            vector_u[index] = f64::from((id % 7) + 1) / 1024.0;
        }
        if id % 11 == 0 {
            let magnitude = f64::from((id % 13) + 1) / 2048.0;
            vector_v[index] = if id % 22 == 0 { magnitude } else { -magnitude };
        }
    }
    Ok((vector_u, vector_v))
}

fn pressure_operator(rows: &[Row], multiplier: &[f64]) -> Result<Vec<f64>, WaterError> {
    let acceleration = pressure_acceleration(rows, multiplier)?;
    let matrix = matrix_action(rows, &acceleration)?;
    let mut result = reserved(matrix.len())?;
    for value in matrix {
        result.push(finite(
            -(DT2 * value),
            "independent positive pressure operator",
        )?);
    }
    Ok(result)
}

fn pressure_dot(left: &[f64], right: &[f64], phase: &str) -> Result<f64, WaterError> {
    if left.len() != right.len() {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!("{phase} vector length mismatch"),
        ));
    }
    let mut result = 0.0;
    for (left, right) in left.iter().zip(right) {
        result = finite(result + (left * right), phase)?;
    }
    Ok(result)
}

fn pressure_action_root(values: &[f64]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.continuum-water.pressure-action-probe.v1\0");
    for value in values {
        hasher.update(value.to_bits().to_le_bytes());
    }
    crate::hash::hex(&hasher.finalize().into())
}

fn pressure_relative_difference_ppb(left: f64, right: f64, phase: &str) -> Result<i64, WaterError> {
    let denominator = left.abs().max(right.abs()).max(SOLVER_EPSILON);
    let relative = finite((left - right).abs() / denominator, phase)?;
    quantize_ppb(relative)
}

fn pressure_density_residual_ppb(residual: &[f64]) -> Result<i64, WaterError> {
    let mut error_sum = 0.0;
    for value in residual {
        let compression = if *value > 0.0 { *value } else { 0.0 };
        error_sum = finite(
            error_sum + compression,
            "independent PCG density error reduction",
        )?;
    }
    let mean = finite(
        error_sum / (residual.len() as f64),
        "independent PCG density error mean",
    )?;
    quantize_ppb(mean)
}

fn compute_lattice_candidate(
    positions: &[I3],
    boundary_positions: Vec<I3>,
    label: &str,
) -> Result<CandidateCalibrationComputation, WaterError> {
    let boundary_volumes = filled(boundary_positions.len(), REST_VOLUME)?;
    let trace = compute_trace(positions, &boundary_positions, &boundary_volumes)?;
    let mut boundary = reserved(boundary_positions.len())?;
    for (index, position) in boundary_positions.iter().copied().enumerate() {
        boundary.push(AuditBoundaryInput {
            id: u32::try_from(index).map_err(|_| {
                WaterError::new(AUDIT_INVALID, format!("{label} boundary id overflow"))
            })?,
            position_um: position.common(),
            volume_bits: scalar_bits(boundary_volumes[index]),
        });
    }
    Ok(CandidateCalibrationComputation { boundary, trace })
}

pub(crate) fn compute_volume_map() -> Result<HydroCalibrationTrace, WaterError> {
    let positions = generate_fluid()?;
    let rows = reconstruct_volume_map(&positions)?;
    let contributions = density_contributions(&positions, None, &rows)?;
    let extended = extended_density_trace(&positions, &rows)?;
    Ok(HydroCalibrationTrace {
        contributions,
        density_error_ppb_by_iteration: extended.errors_ppb,
        first_original_threshold_iteration: extended.first_original_threshold_iteration,
        first_original_threshold_checkpoint: extended.first_original_threshold_checkpoint,
        checkpoints: extended.checkpoints,
    })
}

fn compute_trace(
    positions: &[I3],
    boundary_positions: &[I3],
    boundary_volumes: &[f64],
) -> Result<HydroCalibrationTrace, WaterError> {
    let rows = reconstruct(positions, boundary_positions, boundary_volumes)?;
    let contributions = density_contributions(positions, Some(boundary_positions), &rows)?;
    let extended = extended_density_trace(positions, &rows)?;
    Ok(HydroCalibrationTrace {
        contributions,
        density_error_ppb_by_iteration: extended.errors_ppb,
        first_original_threshold_iteration: extended.first_original_threshold_iteration,
        first_original_threshold_checkpoint: extended.first_original_threshold_checkpoint,
        checkpoints: extended.checkpoints,
    })
}

fn generate_ghost_boundary() -> Result<Vec<I3>, WaterError> {
    const EXPECTED_COUNT: usize = 2_648;
    let mut result = reserved(EXPECTED_COUNT)?;
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
    require_count("ghost boundary", result.len(), EXPECTED_COUNT)?;
    Ok(result)
}

fn generate_support_complete_boundary() -> Result<Vec<I3>, WaterError> {
    const EXPECTED_COUNT: usize = 5_824;
    let mut result = reserved(EXPECTED_COUNT)?;
    for ix in -2_i64..=21 {
        for iy in -2_i64..=21 {
            for iz in -2_i64..=21 {
                if !(0..20).contains(&ix) || !(0..20).contains(&iy) || !(0..20).contains(&iz) {
                    result.push(I3::new(
                        25_000 + (50_000 * ix),
                        25_000 + (50_000 * iy),
                        25_000 + (50_000 * iz),
                    ));
                }
            }
        }
    }
    require_count("support-complete boundary", result.len(), EXPECTED_COUNT)?;
    Ok(result)
}

fn density_contributions(
    positions: &[I3],
    boundary_positions: Option<&[I3]>,
    rows: &[Row],
) -> Result<Vec<DensityContributionRow>, WaterError> {
    let mut result = reserved(SELECTED_ROWS.len())?;
    for (sample_id, role) in SELECTED_ROWS {
        let index = usize::try_from(sample_id)
            .map_err(|_| WaterError::new(AUDIT_INVALID, "calibration sample id overflow"))?;
        let row = rows.get(index).ok_or_else(|| {
            WaterError::new(
                AUDIT_INVALID,
                format!("independent calibration sample {sample_id} is missing"),
            )
        })?;
        let self_term = finite(REST_VOLUME * KERNEL_K, "calibration density self term")?;
        let mut fluid_group = 0.0;
        let mut self_plus_fluid = self_term;
        for neighbor in &row.fluid {
            let term = finite(
                REST_VOLUME * neighbor.value,
                "calibration fluid contribution",
            )?;
            fluid_group = finite(fluid_group + term, "calibration grouped fluid contribution")?;
            self_plus_fluid = finite(
                self_plus_fluid + term,
                "calibration self-plus-fluid contribution",
            )?;
        }
        let mut feature_sums = [0.0_f64; 3];
        let mut feature_counts = [0_usize; 3];
        let mut boundary_total = 0.0;
        for neighbor in &row.boundary {
            let feature = if let Some(boundary_positions) = boundary_positions {
                boundary_feature(boundary_positions[neighbor.boundary])?
            } else {
                neighbor.feature_rank.checked_sub(1).ok_or_else(|| {
                    WaterError::new(
                        AUDIT_INVALID,
                        "independent volume-map feature rank is missing",
                    )
                })?
            };
            if feature >= feature_sums.len() {
                return Err(WaterError::new(
                    AUDIT_INVALID,
                    format!(
                        "independent volume-map feature rank {} exceeds corner",
                        feature + 1
                    ),
                ));
            }
            let term = finite(
                neighbor.volume * neighbor.value,
                "calibration boundary contribution",
            )?;
            feature_counts[feature] += 1;
            feature_sums[feature] = finite(
                feature_sums[feature] + term,
                "calibration grouped boundary feature",
            )?;
            boundary_total = finite(boundary_total + term, "calibration grouped boundary total")?;
        }
        let grouped = finite(
            self_plus_fluid + boundary_total,
            "calibration grouped density ratio",
        )?;
        let grouping_delta = finite(
            row.rho_ratio - grouped,
            "calibration grouping roundoff delta",
        )?;
        let partition_error = finite(row.rho_ratio - 1.0, "calibration partition error")?;
        let required_scale = if boundary_total > 0.0 {
            Some(finite(
                (1.0 - self_plus_fluid) / boundary_total,
                "calibration diagnostic boundary scale",
            )?)
        } else {
            None
        };
        result.push(DensityContributionRow {
            role: role.to_owned(),
            sample_id,
            position_um: positions[index].common(),
            self_bits: scalar_bits(self_term),
            fluid_bits: scalar_bits(fluid_group),
            self_plus_fluid_bits: scalar_bits(self_plus_fluid),
            boundary: BoundaryFeatureContribution {
                face_count: feature_counts[0],
                edge_count: feature_counts[1],
                corner_count: feature_counts[2],
                face_bits: scalar_bits(feature_sums[0]),
                edge_bits: scalar_bits(feature_sums[1]),
                corner_bits: scalar_bits(feature_sums[2]),
                total_bits: scalar_bits(boundary_total),
                face_ppb: quantize_ppb(feature_sums[0])?,
                edge_ppb: quantize_ppb(feature_sums[1])?,
                corner_ppb: quantize_ppb(feature_sums[2])?,
                total_ppb: quantize_ppb(boundary_total)?,
            },
            grouped_rho_ratio_bits: scalar_bits(grouped),
            reconstructed_rho_ratio_bits: scalar_bits(row.rho_ratio),
            grouping_delta_bits: scalar_bits(grouping_delta),
            self_plus_fluid_ppb: quantize_ppb(self_plus_fluid)?,
            reconstructed_rho_ratio_ppb: quantize_ppb(row.rho_ratio)?,
            partition_error_ppb: quantize_ppb(partition_error)?,
            diagnostic_required_boundary_scale_bits: required_scale.map(scalar_bits),
            diagnostic_required_boundary_scale_ppb: required_scale.map(quantize_ppb).transpose()?,
        });
    }
    Ok(result)
}

fn boundary_feature(position: I3) -> Result<usize, WaterError> {
    let outside = [
        position.x < 0 || position.x > 1_000_000,
        position.y < 0 || position.y > 1_000_000,
        position.z < 0 || position.z > 1_000_000,
    ]
    .into_iter()
    .filter(|outside| *outside)
    .count();
    let on_planes = [
        position.x == 0 || position.x == 1_000_000,
        position.y == 0 || position.y == 1_000_000,
        position.z == 0 || position.z == 1_000_000,
    ]
    .into_iter()
    .filter(|on_plane| *on_plane)
    .count();
    let feature_count = if outside > 0 { outside } else { on_planes };
    match feature_count {
        1 => Ok(0),
        2 => Ok(1),
        3 => Ok(2),
        _ => Err(WaterError::new(
            AUDIT_INVALID,
            format!("independent boundary sample {position:?} has {feature_count} outer features"),
        )),
    }
}

fn extended_density_trace(
    positions: &[I3],
    rows: &[Row],
) -> Result<ExtendedDensityTrace, WaterError> {
    let count = positions.len();
    let gravity_y = finite(DT * -GRAVITY, "calibration gravity velocity")?;
    let velocities = filled(count, F3::new(0.0, gravity_y, 0.0))?;
    let mut rho_adv = filled(count, 0.0)?;
    let mut factor = filled(count, 0.0)?;
    let mut multiplier = filled(count, 0.0)?;
    let mut next = filled(count, 0.0)?;
    for index in 0..count {
        let delta = divergence_source(index, rows, &velocities)?;
        rho_adv[index] = finite(rows[index].rho_ratio + (DT * delta), "calibration rho adv")?;
        factor[index] = finite(rows[index].alpha * INV_DT2, "calibration density factor")?;
        let error = finite(rho_adv[index] - 1.0, "calibration initial density error")?;
        let positive = if error > 0.0 { error } else { 0.0 };
        multiplier[index] = finite(positive * factor[index], "calibration initial multiplier")?;
    }

    let mut errors = reserved(usize::from(DIAGNOSTIC_MAX_ITERATIONS))?;
    let mut checkpoints = reserved(DIAGNOSTIC_CHECKPOINTS.len())?;
    let mut first_threshold = None;
    let mut first_threshold_checkpoint = None;
    for iteration in 1..=DIAGNOSTIC_MAX_ITERATIONS {
        let acceleration = pressure_acceleration(rows, &multiplier)?;
        let matrix = matrix_action(rows, &acceleration)?;
        let mut error_sum = 0.0;
        for index in 0..count {
            let source = finite(1.0 - rho_adv[index], "calibration density source")?;
            let dt2_action = finite(DT2 * matrix[index], "calibration density matrix action")?;
            let correction = finite(
                (source - dt2_action) * factor[index],
                "calibration density correction",
            )?;
            let candidate = finite(
                multiplier[index] - (RELAXATION * correction),
                "calibration density candidate",
            )?;
            next[index] = if candidate > 0.0 { candidate } else { 0.0 };
            let error = finite(
                (rho_adv[index] + dt2_action) - 1.0,
                "calibration density error",
            )?;
            let error = if error > 0.0 { error } else { 0.0 };
            error_sum = finite(error_sum + error, "calibration density error fold")?;
        }
        let mean = finite(error_sum / (count as f64), "calibration density error mean")?;
        let error_ppb = quantize_ppb(mean)?;
        errors.push(error_ppb);
        mem::swap(&mut multiplier, &mut next);
        if error_ppb <= DENSITY_THRESHOLD_PPB && first_threshold.is_none() {
            first_threshold = Some(iteration);
            first_threshold_checkpoint = Some(prospective_checkpoint(
                iteration,
                error_ppb,
                positions,
                rows,
                &velocities,
                &multiplier,
            )?);
        }
        if DIAGNOSTIC_CHECKPOINTS.contains(&iteration) {
            checkpoints.push(prospective_checkpoint(
                iteration,
                error_ppb,
                positions,
                rows,
                &velocities,
                &multiplier,
            )?);
        }
    }
    Ok(ExtendedDensityTrace {
        errors_ppb: errors,
        first_original_threshold_iteration: first_threshold,
        first_original_threshold_checkpoint: first_threshold_checkpoint,
        checkpoints,
    })
}

#[allow(clippy::too_many_arguments)]
fn prospective_checkpoint(
    iteration: u16,
    density_error_ppb: i64,
    positions: &[I3],
    rows: &[Row],
    velocities: &[F3],
    multiplier: &[f64],
) -> Result<ExtendedDensityCheckpoint, WaterError> {
    let acceleration = pressure_acceleration(rows, multiplier)?;
    let mut maximum_speed = 0.0;
    let mut minimum_velocity_y = f64::INFINITY;
    let mut maximum_velocity_y = f64::NEG_INFINITY;
    let mut minimum_clearance = i64::MAX;
    for index in 0..positions.len() {
        let velocity = finite_vec(
            velocities[index].add(acceleration[index].scale(DT)),
            "calibration prospective velocity",
        )?;
        let speed_squared = finite(
            velocity.dot(velocity),
            "calibration prospective speed squared",
        )?;
        let speed = finite(speed_squared.sqrt(), "calibration prospective speed")?;
        if speed > maximum_speed {
            maximum_speed = speed;
        }
        if velocity.y < minimum_velocity_y {
            minimum_velocity_y = velocity.y;
        }
        if velocity.y > maximum_velocity_y {
            maximum_velocity_y = velocity.y;
        }
        let decoded = F3::new(
            (positions[index].x as f64) / SCALE,
            (positions[index].y as f64) / SCALE,
            (positions[index].z as f64) / SCALE,
        );
        let position = finite_vec(
            decoded.add(velocity.scale(DT)),
            "calibration prospective position",
        )?;
        let canonical = I3::new(
            quantize_scaled(position.x, 1_000_000)?,
            quantize_scaled(position.y, 1_000_000)?,
            quantize_scaled(position.z, 1_000_000)?,
        );
        for clearance in [
            canonical.x,
            1_000_000 - canonical.x,
            canonical.y,
            1_000_000 - canonical.y,
            canonical.z,
            1_000_000 - canonical.z,
        ] {
            if clearance < minimum_clearance {
                minimum_clearance = clearance;
            }
        }
    }
    if minimum_clearance == i64::MAX {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "independent prospective checkpoint has no samples",
        ));
    }
    let maximum_penetration = 25_000_i64
        .checked_sub(minimum_clearance)
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "prospective penetration overflow"))?
        .max(0);
    let mut maximum_multiplier = 0.0;
    for value in multiplier {
        if !value.is_finite() {
            return Err(WaterError::new(
                AUDIT_INVALID,
                "nonfinite independent prospective multiplier",
            ));
        }
        if *value > maximum_multiplier {
            maximum_multiplier = *value;
        }
    }
    Ok(ExtendedDensityCheckpoint {
        iteration,
        density_error_ppb,
        original_threshold_met: density_error_ppb <= DENSITY_THRESHOLD_PPB,
        maximum_multiplier_bits: scalar_bits(maximum_multiplier),
        prospective_maximum_speed_um_s: quantize_scaled(maximum_speed, 1_000_000)?,
        prospective_minimum_velocity_y_um_s: quantize_scaled(minimum_velocity_y, 1_000_000)?,
        prospective_maximum_velocity_y_um_s: quantize_scaled(maximum_velocity_y, 1_000_000)?,
        prospective_minimum_outer_clearance_um: minimum_clearance,
        prospective_maximum_penetration_um: maximum_penetration,
        prospective_outer_escape: minimum_clearance < 0,
    })
}
