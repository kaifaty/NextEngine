#![forbid(unsafe_code)]

use crate::audit::{SELECTED_ROWS, scalar_bits};
use crate::calibration::{
    BoundaryFeatureContribution, DIAGNOSTIC_CHECKPOINTS, DIAGNOSTIC_MAX_ITERATIONS,
    DensityContributionRow, ExtendedDensityCheckpoint, ExtendedDensityTrace, HydroCalibrationTrace,
};

use super::*;

pub(crate) fn compute() -> Result<HydroCalibrationTrace, WaterError> {
    let positions = generate_fluid()?;
    let boundary_positions = generate_boundary()?;
    let boundary_volumes = boundary_volumes(&boundary_positions)?;
    let rows = reconstruct(&positions, &boundary_positions, &boundary_volumes)?;
    let contributions =
        density_contributions(&positions, &boundary_positions, &boundary_volumes, &rows)?;
    let extended = extended_density_trace(&positions, &boundary_volumes, &rows)?;
    Ok(HydroCalibrationTrace {
        contributions,
        density_error_ppb_by_iteration: extended.errors_ppb,
        first_original_threshold_iteration: extended.first_original_threshold_iteration,
        checkpoints: extended.checkpoints,
    })
}

fn density_contributions(
    positions: &[I3],
    boundary_positions: &[I3],
    boundary_volumes: &[f64],
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
            let feature = boundary_feature(boundary_positions[neighbor.boundary])?;
            let term = finite(
                boundary_volumes[neighbor.boundary] * neighbor.value,
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
    let on_planes = [
        position.x == 0 || position.x == 1_000_000,
        position.y == 0 || position.y == 1_000_000,
        position.z == 0 || position.z == 1_000_000,
    ]
    .into_iter()
    .filter(|on_plane| *on_plane)
    .count();
    match on_planes {
        1 => Ok(0),
        2 => Ok(1),
        3 => Ok(2),
        _ => Err(WaterError::new(
            AUDIT_INVALID,
            format!("independent boundary sample {position:?} has {on_planes} outer features"),
        )),
    }
}

fn extended_density_trace(
    positions: &[I3],
    boundary_volumes: &[f64],
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
        let delta = divergence_source(index, rows, boundary_volumes, &velocities)?;
        rho_adv[index] = finite(rows[index].rho_ratio + (DT * delta), "calibration rho adv")?;
        factor[index] = finite(rows[index].alpha * INV_DT2, "calibration density factor")?;
        let error = finite(rho_adv[index] - 1.0, "calibration initial density error")?;
        let positive = if error > 0.0 { error } else { 0.0 };
        multiplier[index] = finite(positive * factor[index], "calibration initial multiplier")?;
    }

    let mut errors = reserved(usize::from(DIAGNOSTIC_MAX_ITERATIONS))?;
    let mut checkpoints = reserved(DIAGNOSTIC_CHECKPOINTS.len())?;
    let mut first_threshold = None;
    for iteration in 1..=DIAGNOSTIC_MAX_ITERATIONS {
        let acceleration = pressure_acceleration(rows, boundary_volumes, &multiplier)?;
        let matrix = matrix_action(rows, boundary_volumes, &acceleration)?;
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
        }
        if DIAGNOSTIC_CHECKPOINTS.contains(&iteration) {
            checkpoints.push(prospective_checkpoint(
                iteration,
                error_ppb,
                positions,
                boundary_volumes,
                rows,
                &velocities,
                &multiplier,
            )?);
        }
    }
    Ok(ExtendedDensityTrace {
        errors_ppb: errors,
        first_original_threshold_iteration: first_threshold,
        checkpoints,
    })
}

#[allow(clippy::too_many_arguments)]
fn prospective_checkpoint(
    iteration: u16,
    density_error_ppb: i64,
    positions: &[I3],
    boundary_volumes: &[f64],
    rows: &[Row],
    velocities: &[F3],
    multiplier: &[f64],
) -> Result<ExtendedDensityCheckpoint, WaterError> {
    let acceleration = pressure_acceleration(rows, boundary_volumes, multiplier)?;
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
