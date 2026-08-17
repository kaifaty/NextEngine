#![forbid(unsafe_code)]

use crate::audit::{SELECTED_ROWS, scalar_bits};
use crate::calibration::{
    BoundaryFeatureContribution, DIAGNOSTIC_CHECKPOINTS, DIAGNOSTIC_MAX_ITERATIONS,
    DensityContributionRow, ExtendedDensityCheckpoint, ExtendedDensityTrace, HydroCalibrationTrace,
};
use crate::error::{AUDIT_INVALID, WaterError};
use crate::model::Geometry;
use crate::profile::{PARTICLE_RADIUS_UM, quantize_ppb, quantize_scaled};

use super::super::*;

pub(crate) fn production_hydro_calibration(
    samples: &[CanonicalSample],
    boundary: &[BoundarySample],
    geometry: Geometry,
) -> Result<HydroCalibrationTrace, WaterError> {
    let mut state = decode(samples)?;
    let reconstruction = reconstruct(&state, boundary)?;
    let contributions = density_contributions(&state, &reconstruction, boundary, geometry)?;
    let _divergence = solve_divergence(&reconstruction, boundary, &mut state.velocities)?;
    for velocity in &mut state.velocities {
        velocity.y = checked_scalar(
            velocity.y + (DT * -GRAVITY_MAGNITUDE),
            "calibration production gravity velocity y",
        )?;
    }
    let extended = extended_density_trace(&state, &reconstruction, boundary, geometry)?;
    Ok(HydroCalibrationTrace {
        contributions,
        density_error_ppb_by_iteration: extended.errors_ppb,
        first_original_threshold_iteration: extended.first_original_threshold_iteration,
        checkpoints: extended.checkpoints,
    })
}

fn density_contributions(
    state: &DecodedState,
    reconstruction: &Reconstruction,
    boundary: &[BoundarySample],
    geometry: Geometry,
) -> Result<Vec<DensityContributionRow>, WaterError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(SELECTED_ROWS.len())
        .map_err(super::audit_reserve_error)?;
    for (sample_id, role) in SELECTED_ROWS {
        let index = state
            .samples
            .binary_search_by_key(&sample_id, |sample| sample.id)
            .map_err(|_| {
                WaterError::new(
                    AUDIT_INVALID,
                    format!("selected calibration sample {sample_id} is missing"),
                )
            })?;
        let row = reconstruction.rows[index];
        let self_term = checked_scalar(
            REST_VOLUME * kernel::value_at_zero(),
            "calibration density self term",
        )?;
        let mut fluid_group = 0.0;
        let mut self_plus_fluid = self_term;
        for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
            let term = checked_scalar(
                REST_VOLUME * neighbor.value,
                "calibration fluid contribution",
            )?;
            fluid_group =
                checked_scalar(fluid_group + term, "calibration grouped fluid contribution")?;
            self_plus_fluid = checked_scalar(
                self_plus_fluid + term,
                "calibration self-plus-fluid contribution",
            )?;
        }
        let mut feature_sums = [0.0_f64; 3];
        let mut feature_counts = [0_usize; 3];
        let mut boundary_total = 0.0;
        for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
            let sample = boundary[neighbor.boundary];
            let feature = boundary_feature(sample.position_um, geometry)?;
            let term = checked_scalar(
                sample.volume * neighbor.value,
                "calibration boundary contribution",
            )?;
            feature_counts[feature] += 1;
            feature_sums[feature] = checked_scalar(
                feature_sums[feature] + term,
                "calibration grouped boundary feature",
            )?;
            boundary_total =
                checked_scalar(boundary_total + term, "calibration grouped boundary total")?;
        }
        let reconstructed = reconstruction.rho_ratio[index];
        let grouped = checked_scalar(
            self_plus_fluid + boundary_total,
            "calibration grouped density ratio",
        )?;
        let grouping_delta = checked_scalar(
            reconstructed - grouped,
            "calibration grouping roundoff delta",
        )?;
        let partition_error = checked_scalar(reconstructed - 1.0, "calibration partition error")?;
        let required_scale = if boundary_total > 0.0 {
            Some(checked_scalar(
                (1.0 - self_plus_fluid) / boundary_total,
                "calibration diagnostic boundary scale",
            )?)
        } else {
            None
        };
        result.push(DensityContributionRow {
            role: role.to_owned(),
            sample_id,
            position_um: state.samples[index].position_um,
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
            reconstructed_rho_ratio_bits: scalar_bits(reconstructed),
            grouping_delta_bits: scalar_bits(grouping_delta),
            self_plus_fluid_ppb: quantize_ppb(self_plus_fluid)?,
            reconstructed_rho_ratio_ppb: quantize_ppb(reconstructed)?,
            partition_error_ppb: quantize_ppb(partition_error)?,
            diagnostic_required_boundary_scale_bits: required_scale.map(scalar_bits),
            diagnostic_required_boundary_scale_ppb: required_scale.map(quantize_ppb).transpose()?,
        });
    }
    Ok(result)
}

fn boundary_feature(position: Vec3i, geometry: Geometry) -> Result<usize, WaterError> {
    let on_planes = [
        position.x == geometry.bounds.min.x || position.x == geometry.bounds.max.x,
        position.y == geometry.bounds.min.y || position.y == geometry.bounds.max.y,
        position.z == geometry.bounds.min.z || position.z == geometry.bounds.max.z,
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
            format!("boundary sample {position:?} has {on_planes} outer features"),
        )),
    }
}

fn extended_density_trace(
    state: &DecodedState,
    reconstruction: &Reconstruction,
    boundary: &[BoundarySample],
    geometry: Geometry,
) -> Result<ExtendedDensityTrace, WaterError> {
    let count = state.velocities.len();
    let mut rho_adv = filled_vec(count, 0.0)?;
    let mut factor = filled_vec(count, 0.0)?;
    let mut multiplier = filled_vec(count, 0.0)?;
    let mut next = filled_vec(count, 0.0)?;
    for index in 0..count {
        let delta = divergence_source(
            index,
            reconstruction.rows[index],
            reconstruction,
            boundary,
            &state.velocities,
        )?;
        rho_adv[index] = checked_scalar(
            reconstruction.rho_ratio[index] + (DT * delta),
            "calibration advected density ratio",
        )?;
        factor[index] = checked_scalar(
            reconstruction.alpha[index] * INV_DT2,
            "calibration density factor",
        )?;
        let error = checked_scalar(rho_adv[index] - 1.0, "calibration initial error")?;
        let positive = if error > 0.0 { error } else { 0.0 };
        multiplier[index] =
            checked_scalar(positive * factor[index], "calibration initial multiplier")?;
    }

    let mut errors = Vec::new();
    errors
        .try_reserve_exact(usize::from(DIAGNOSTIC_MAX_ITERATIONS))
        .map_err(super::audit_reserve_error)?;
    let mut checkpoints = Vec::new();
    checkpoints
        .try_reserve_exact(DIAGNOSTIC_CHECKPOINTS.len())
        .map_err(super::audit_reserve_error)?;
    let mut first_threshold = None;
    for iteration in 1..=DIAGNOSTIC_MAX_ITERATIONS {
        let acceleration = pressure_acceleration(reconstruction, boundary, &multiplier)?;
        let matrix = matrix_action(reconstruction, boundary, &acceleration.total)?;
        let mut error_sum = 0.0;
        for index in 0..count {
            let source = checked_scalar(1.0 - rho_adv[index], "calibration density source")?;
            let dt2_action =
                checked_scalar(DT2 * matrix[index], "calibration density matrix action")?;
            let correction = checked_scalar(
                (source - dt2_action) * factor[index],
                "calibration density correction",
            )?;
            let candidate = checked_scalar(
                multiplier[index] - (RELAXATION * correction),
                "calibration density candidate",
            )?;
            next[index] = if candidate > 0.0 { candidate } else { 0.0 };
            let error = checked_scalar(
                (rho_adv[index] + dt2_action) - 1.0,
                "calibration density error",
            )?;
            let error = if error > 0.0 { error } else { 0.0 };
            error_sum = checked_scalar(error_sum + error, "calibration density error fold")?;
        }
        let mean = checked_scalar(error_sum / (count as f64), "calibration density error mean")?;
        let error_ppb = quantize_ppb(mean)?;
        errors.push(error_ppb);
        std::mem::swap(&mut multiplier, &mut next);
        if error_ppb <= DENSITY_THRESHOLD_PPB && first_threshold.is_none() {
            first_threshold = Some(iteration);
        }
        if DIAGNOSTIC_CHECKPOINTS.contains(&iteration) {
            checkpoints.push(prospective_checkpoint(
                iteration,
                error_ppb,
                state,
                reconstruction,
                boundary,
                geometry,
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
    state: &DecodedState,
    reconstruction: &Reconstruction,
    boundary: &[BoundarySample],
    geometry: Geometry,
    multiplier: &[f64],
) -> Result<ExtendedDensityCheckpoint, WaterError> {
    let acceleration = pressure_acceleration(reconstruction, boundary, multiplier)?;
    let mut maximum_speed = 0.0;
    let mut minimum_velocity_y = f64::INFINITY;
    let mut maximum_velocity_y = f64::NEG_INFINITY;
    let mut minimum_clearance = i64::MAX;
    for index in 0..state.samples.len() {
        let velocity = state.velocities[index]
            .add(acceleration.total[index].scale(DT))
            .checked("calibration prospective velocity")?;
        let speed_squared = checked_scalar(
            velocity.dot(velocity),
            "calibration prospective speed squared",
        )?;
        let speed = checked_scalar(speed_squared.sqrt(), "calibration prospective speed")?;
        if speed > maximum_speed {
            maximum_speed = speed;
        }
        if velocity.y < minimum_velocity_y {
            minimum_velocity_y = velocity.y;
        }
        if velocity.y > maximum_velocity_y {
            maximum_velocity_y = velocity.y;
        }
        let position = state.positions[index]
            .add(velocity.scale(DT))
            .checked("calibration prospective position")?;
        let canonical = Vec3i::new(
            quantize_scaled(position.x, 1_000_000)?,
            quantize_scaled(position.y, 1_000_000)?,
            quantize_scaled(position.z, 1_000_000)?,
        );
        for clearance in [
            canonical.x - geometry.bounds.min.x,
            geometry.bounds.max.x - canonical.x,
            canonical.y - geometry.bounds.min.y,
            geometry.bounds.max.y - canonical.y,
            canonical.z - geometry.bounds.min.z,
            geometry.bounds.max.z - canonical.z,
        ] {
            if clearance < minimum_clearance {
                minimum_clearance = clearance;
            }
        }
    }
    if minimum_clearance == i64::MAX {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "calibration prospective checkpoint has no samples",
        ));
    }
    let maximum_penetration = PARTICLE_RADIUS_UM
        .checked_sub(minimum_clearance)
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "prospective penetration overflow"))?
        .max(0);
    Ok(ExtendedDensityCheckpoint {
        iteration,
        density_error_ppb,
        original_threshold_met: density_error_ppb <= DENSITY_THRESHOLD_PPB,
        maximum_multiplier_bits: format!("0x{:016x}", maximum_multiplier_bits(multiplier)?),
        prospective_maximum_speed_um_s: quantize_scaled(maximum_speed, 1_000_000)?,
        prospective_minimum_velocity_y_um_s: quantize_scaled(minimum_velocity_y, 1_000_000)?,
        prospective_maximum_velocity_y_um_s: quantize_scaled(maximum_velocity_y, 1_000_000)?,
        prospective_minimum_outer_clearance_um: minimum_clearance,
        prospective_maximum_penetration_um: maximum_penetration,
        prospective_outer_escape: minimum_clearance < 0,
    })
}
