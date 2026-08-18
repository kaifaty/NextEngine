#![forbid(unsafe_code)]

use super::*;
use crate::geometry::{
    APERTURE_MAX_MAX_CORNER_FEATURE_ID, APERTURE_MAX_MIN_CORNER_FEATURE_ID,
    APERTURE_MIN_MAX_CORNER_FEATURE_ID, APERTURE_MIN_MIN_CORNER_FEATURE_ID,
    APERTURE_Y_MAX_EDGE_FEATURE_ID, APERTURE_Y_MIN_EDGE_FEATURE_ID, APERTURE_Z_MAX_EDGE_FEATURE_ID,
    APERTURE_Z_MIN_EDGE_FEATURE_ID, AxisAlignedGeometryManifest, InternalPlanePatch,
    OUTER_X_MAX_FEATURE_ID, OUTER_X_MIN_FEATURE_ID, OUTER_Y_MAX_FEATURE_ID, OUTER_Y_MIN_FEATURE_ID,
    OUTER_Z_MAX_FEATURE_ID, OUTER_Z_MIN_FEATURE_ID,
};
use crate::profile::{PARTICLE_RADIUS, UNIFORM_MASS};

mod accelerated;

pub(super) use accelerated::solve_density_accelerated_projected_gradient;

const CONTACT_FEATURE_CAPACITY: usize = 25;
const INTERNAL_CONTACT_ITERATIONS: usize = 8;
const CONTACT_DIRECTION_ROUNDING_GUARD: f64 = 32.0 * f64::EPSILON;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VelocityProjectionResult {
    pub(crate) fluid_impulse: Vec3f,
    pub(crate) kinetic_energy_delta: f64,
    pub(crate) active_rows: usize,
    pub(crate) active_components: usize,
    pub(crate) maximum_absolute_delta_velocity: f64,
    pub(crate) feature_fluid_impulses: [Vec3f; CONTACT_FEATURE_CAPACITY],
    pub(crate) feature_active_constraints: [u32; CONTACT_FEATURE_CAPACITY],
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
        let before_speed_squared = checked_scalar(
            before.dot(before),
            "predictive contact before speed squared",
        )?;
        let after_speed_squared = checked_scalar(
            velocity.dot(*velocity),
            "predictive contact after speed squared",
        )?;
        let energy_delta = checked_scalar(
            0.5 * UNIFORM_MASS * (after_speed_squared - before_speed_squared),
            "predictive contact kinetic energy delta",
        )?;
        result.kinetic_energy_delta = checked_scalar(
            result.kinetic_energy_delta + energy_delta,
            "predictive contact kinetic energy reduction",
        )?;
        for (feature_id, component) in [
            (
                if delta.x > 0.0 {
                    OUTER_X_MIN_FEATURE_ID
                } else {
                    OUTER_X_MAX_FEATURE_ID
                },
                Vec3f::new(delta.x, 0.0, 0.0),
            ),
            (
                if delta.y > 0.0 {
                    OUTER_Y_MIN_FEATURE_ID
                } else {
                    OUTER_Y_MAX_FEATURE_ID
                },
                Vec3f::new(0.0, delta.y, 0.0),
            ),
            (
                if delta.z > 0.0 {
                    OUTER_Z_MIN_FEATURE_ID
                } else {
                    OUTER_Z_MAX_FEATURE_ID
                },
                Vec3f::new(0.0, 0.0, delta.z),
            ),
        ] {
            if component.x != 0.0 || component.y != 0.0 || component.z != 0.0 {
                record_feature_impulse(&mut result, feature_id, component)?;
            }
        }
    }
    Ok(result)
}

pub(super) fn project_predictive_geometry(
    geometry: Geometry,
    positions: &[Vec3f],
    velocities: &mut [Vec3f],
) -> Result<VelocityProjectionResult, WaterError> {
    let manifest = AxisAlignedGeometryManifest::from_geometry(geometry)?;
    let mut result = project_predictive_outer_box(geometry, positions, velocities)?;
    let Some(patch) = manifest.internal_patch() else {
        return Ok(result);
    };
    for (position, velocity) in positions.iter().copied().zip(velocities.iter_mut()) {
        let before = *velocity;
        let projected = sweep_internal_patch(position, before, patch, &mut result)?;
        let delta = projected
            .sub(before)
            .checked("internal contact delta velocity")?;
        if delta.x == 0.0 && delta.y == 0.0 && delta.z == 0.0 {
            continue;
        }
        result.active_rows = result.active_rows.checked_add(1).ok_or_else(|| {
            WaterError::new(NUMERIC_OVERFLOW, "internal contact row count overflow")
        })?;
        result.active_components = result
            .active_components
            .checked_add(
                [delta.x, delta.y, delta.z]
                    .into_iter()
                    .filter(|component| *component != 0.0)
                    .count(),
            )
            .ok_or_else(|| {
                WaterError::new(
                    NUMERIC_OVERFLOW,
                    "internal contact component count overflow",
                )
            })?;
        result.maximum_absolute_delta_velocity = result
            .maximum_absolute_delta_velocity
            .max(delta.x.abs())
            .max(delta.y.abs())
            .max(delta.z.abs());
        let impulse = delta
            .scale(UNIFORM_MASS)
            .checked("internal contact fluid impulse row")?;
        result.fluid_impulse = result
            .fluid_impulse
            .add(impulse)
            .checked("internal contact fluid impulse reduction")?;
        let before_speed_squared =
            checked_scalar(before.dot(before), "internal contact before speed squared")?;
        let projected_speed_squared = checked_scalar(
            projected.dot(projected),
            "internal contact projected speed squared",
        )?;
        let energy_delta = checked_scalar(
            0.5 * UNIFORM_MASS * (projected_speed_squared - before_speed_squared),
            "internal contact kinetic energy delta",
        )?;
        result.kinetic_energy_delta = checked_scalar(
            result.kinetic_energy_delta + energy_delta,
            "internal contact kinetic energy reduction",
        )?;
        *velocity = projected;
    }
    Ok(result)
}

#[derive(Clone, Copy)]
struct SweepHit {
    time: f64,
    normal: Vec3f,
    feature_id: u32,
}

fn sweep_internal_patch(
    start: Vec3f,
    velocity: Vec3f,
    patch: InternalPlanePatch,
    result: &mut VelocityProjectionResult,
) -> Result<Vec3f, WaterError> {
    let mut current = start;
    let mut remaining = velocity
        .scale(DT)
        .checked("internal contact initial displacement")?;
    let mut contacted = false;
    for _ in 0..INTERNAL_CONTACT_ITERATIONS {
        let Some(hit) = earliest_internal_hit(current, remaining, patch)? else {
            current = current
                .add(remaining)
                .checked("internal contact final advance")?;
            remaining = Vec3f::ZERO;
            break;
        };
        current = current
            .add(remaining.scale(hit.time))
            .checked("internal contact time-of-impact advance")?;
        let tail = remaining
            .scale(checked_scalar(
                1.0 - hit.time,
                "internal contact remaining fraction",
            )?)
            .checked("internal contact tail")?;
        let inward = checked_scalar(tail.dot(hit.normal), "internal contact normal displacement")?;
        if inward >= 0.0 {
            current = current
                .add(tail)
                .checked("internal contact non-inward advance")?;
            remaining = Vec3f::ZERO;
            break;
        }
        let correction = hit
            .normal
            .scale(-inward)
            .checked("internal contact displacement correction")?;
        let delta_velocity = correction
            .scale(INV_DT)
            .checked("internal contact feature delta velocity")?;
        record_feature_impulse(result, hit.feature_id, delta_velocity)?;
        contacted = true;
        remaining = tail
            .add(correction)
            .checked("internal contact projected tail")?;
    }
    if remaining.x != 0.0 || remaining.y != 0.0 || remaining.z != 0.0 {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            format!(
                "internal contact exceeded the fixed {INTERNAL_CONTACT_ITERATIONS}-constraint schedule"
            ),
        ));
    }
    if contacted {
        current
            .sub(start)
            .scale(INV_DT)
            .checked("internal contact published velocity")
    } else {
        Ok(velocity)
    }
}

fn earliest_internal_hit(
    start: Vec3f,
    displacement: Vec3f,
    patch: InternalPlanePatch,
) -> Result<Option<SweepHit>, WaterError> {
    let wall = decode_micrometres(patch.coordinate_um)?;
    let y_min = decode_micrometres(patch.opening.y_min_um)?;
    let y_max = decode_micrometres(patch.opening.y_max_um)?;
    let z_min = decode_micrometres(patch.opening.z_min_um)?;
    let z_max = decode_micrometres(patch.opening.z_max_um)?;
    let mut best = None;

    if displacement.x > 0.0 && start.x <= wall {
        let plane = checked_scalar(wall - PARTICLE_RADIUS, "internal left face offset")?;
        consider_face_hit(
            &mut best,
            start,
            displacement,
            plane,
            Vec3f::new(-1.0, 0.0, 0.0),
            patch,
        )?;
    }
    if displacement.x < 0.0 && start.x >= wall {
        let plane = checked_scalar(wall + PARTICLE_RADIUS, "internal right face offset")?;
        consider_face_hit(
            &mut best,
            start,
            displacement,
            plane,
            Vec3f::new(1.0, 0.0, 0.0),
            patch,
        )?;
    }

    consider_axis_capsule_hit(
        &mut best,
        start,
        displacement,
        Vec3f::new(wall, y_min, z_min),
        Vec3f::new(wall, y_min, z_max),
        2,
        APERTURE_Y_MIN_EDGE_FEATURE_ID,
    )?;
    consider_axis_capsule_hit(
        &mut best,
        start,
        displacement,
        Vec3f::new(wall, y_max, z_min),
        Vec3f::new(wall, y_max, z_max),
        2,
        APERTURE_Y_MAX_EDGE_FEATURE_ID,
    )?;
    consider_axis_capsule_hit(
        &mut best,
        start,
        displacement,
        Vec3f::new(wall, y_min, z_min),
        Vec3f::new(wall, y_max, z_min),
        1,
        APERTURE_Z_MIN_EDGE_FEATURE_ID,
    )?;
    consider_axis_capsule_hit(
        &mut best,
        start,
        displacement,
        Vec3f::new(wall, y_min, z_max),
        Vec3f::new(wall, y_max, z_max),
        1,
        APERTURE_Z_MAX_EDGE_FEATURE_ID,
    )?;
    for (centre, feature_id) in [
        (
            Vec3f::new(wall, y_min, z_min),
            APERTURE_MIN_MIN_CORNER_FEATURE_ID,
        ),
        (
            Vec3f::new(wall, y_min, z_max),
            APERTURE_MIN_MAX_CORNER_FEATURE_ID,
        ),
        (
            Vec3f::new(wall, y_max, z_min),
            APERTURE_MAX_MIN_CORNER_FEATURE_ID,
        ),
        (
            Vec3f::new(wall, y_max, z_max),
            APERTURE_MAX_MAX_CORNER_FEATURE_ID,
        ),
    ] {
        consider_sphere_hit(&mut best, start, displacement, centre, feature_id)?;
    }
    Ok(best)
}

fn consider_face_hit(
    best: &mut Option<SweepHit>,
    start: Vec3f,
    displacement: Vec3f,
    plane: f64,
    normal: Vec3f,
    patch: InternalPlanePatch,
) -> Result<(), WaterError> {
    let time = checked_scalar((plane - start.x) / displacement.x, "internal face hit time")?;
    if !(0.0..=1.0).contains(&time) {
        return Ok(());
    }
    let point = start
        .add(displacement.scale(time))
        .checked("internal face hit point")?;
    let y_min = decode_micrometres(patch.y_min_um)?;
    let y_max = decode_micrometres(patch.y_max_um)?;
    let z_min = decode_micrometres(patch.z_min_um)?;
    let z_max = decode_micrometres(patch.z_max_um)?;
    let opening_y_min = decode_micrometres(patch.opening.y_min_um)?;
    let opening_y_max = decode_micrometres(patch.opening.y_max_um)?;
    let opening_z_min = decode_micrometres(patch.opening.z_min_um)?;
    let opening_z_max = decode_micrometres(patch.opening.z_max_um)?;
    let in_patch = point.y >= y_min && point.y <= y_max && point.z >= z_min && point.z <= z_max;
    let in_closed_opening = point.y >= opening_y_min
        && point.y <= opening_y_max
        && point.z >= opening_z_min
        && point.z <= opening_z_max;
    if in_patch && !in_closed_opening {
        select_hit(
            best,
            SweepHit {
                time,
                normal,
                feature_id: patch.feature_id,
            },
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn consider_axis_capsule_hit(
    best: &mut Option<SweepHit>,
    start: Vec3f,
    displacement: Vec3f,
    segment_start: Vec3f,
    segment_end: Vec3f,
    axis: usize,
    feature_id: u32,
) -> Result<(), WaterError> {
    let start_components = [start.x, start.y, start.z];
    let displacement_components = [displacement.x, displacement.y, displacement.z];
    let segment_components = [segment_start.x, segment_start.y, segment_start.z];
    let perpendicular = match axis {
        1 => [0, 2],
        2 => [0, 1],
        _ => {
            return Err(WaterError::new(
                NUMERIC_OVERFLOW,
                "unsupported internal capsule axis",
            ));
        }
    };
    let r0 = checked_scalar(
        start_components[perpendicular[0]] - segment_components[perpendicular[0]],
        "capsule radial start 0",
    )?;
    let r1 = checked_scalar(
        start_components[perpendicular[1]] - segment_components[perpendicular[1]],
        "capsule radial start 1",
    )?;
    let d0 = displacement_components[perpendicular[0]];
    let d1 = displacement_components[perpendicular[1]];
    let a = checked_scalar((d0 * d0) + (d1 * d1), "capsule quadratic a")?;
    let b = checked_scalar(2.0 * ((r0 * d0) + (r1 * d1)), "capsule quadratic b")?;
    let c = checked_scalar(
        (r0 * r0) + (r1 * r1) - (PARTICLE_RADIUS * PARTICLE_RADIUS),
        "capsule quadratic c",
    )?;
    let axis_min = [segment_start.x, segment_start.y, segment_start.z][axis];
    let axis_max = [segment_end.x, segment_end.y, segment_end.z][axis];
    if c <= 0.0 {
        let coordinate = start_components[axis];
        if coordinate > axis_min && coordinate < axis_max {
            let normal = radial_normal(r0, r1, perpendicular)?;
            if is_meaningfully_inward(displacement, normal)? {
                select_hit(
                    best,
                    SweepHit {
                        time: 0.0,
                        normal,
                        feature_id,
                    },
                );
            }
        }
    }
    if a == 0.0 {
        return Ok(());
    }
    let discriminant = checked_scalar((b * b) - (4.0 * a * c), "capsule discriminant")?;
    if discriminant < 0.0 {
        return Ok(());
    }
    let time = checked_scalar((-b - discriminant.sqrt()) / (2.0 * a), "capsule hit time")?;
    if !(0.0..=1.0).contains(&time) {
        return Ok(());
    }
    let coordinate = checked_scalar(
        start_components[axis] + (displacement_components[axis] * time),
        "capsule axial hit coordinate",
    )?;
    if coordinate <= axis_min || coordinate >= axis_max {
        return Ok(());
    }
    let hit_r0 = checked_scalar(r0 + (d0 * time), "capsule radial hit 0")?;
    let hit_r1 = checked_scalar(r1 + (d1 * time), "capsule radial hit 1")?;
    let normal = radial_normal(hit_r0, hit_r1, perpendicular)?;
    if is_meaningfully_inward(displacement, normal)? {
        select_hit(
            best,
            SweepHit {
                time,
                normal,
                feature_id,
            },
        );
    }
    Ok(())
}

fn consider_sphere_hit(
    best: &mut Option<SweepHit>,
    start: Vec3f,
    displacement: Vec3f,
    centre: Vec3f,
    feature_id: u32,
) -> Result<(), WaterError> {
    let radial = start.sub(centre).checked("corner radial start")?;
    let a = checked_scalar(displacement.dot(displacement), "corner quadratic a")?;
    let b = checked_scalar(2.0 * radial.dot(displacement), "corner quadratic b")?;
    let c = checked_scalar(
        radial.dot(radial) - (PARTICLE_RADIUS * PARTICLE_RADIUS),
        "corner quadratic c",
    )?;
    if c <= 0.0 {
        let normal = normalize(radial, "corner initial normal")?;
        if is_meaningfully_inward(displacement, normal)? {
            select_hit(
                best,
                SweepHit {
                    time: 0.0,
                    normal,
                    feature_id,
                },
            );
        }
    }
    if a == 0.0 {
        return Ok(());
    }
    let discriminant = checked_scalar((b * b) - (4.0 * a * c), "corner discriminant")?;
    if discriminant < 0.0 {
        return Ok(());
    }
    let time = checked_scalar((-b - discriminant.sqrt()) / (2.0 * a), "corner hit time")?;
    if !(0.0..=1.0).contains(&time) {
        return Ok(());
    }
    let point = start
        .add(displacement.scale(time))
        .checked("corner hit point")?;
    let normal = normalize(point.sub(centre), "corner hit normal")?;
    if is_meaningfully_inward(displacement, normal)? {
        select_hit(
            best,
            SweepHit {
                time,
                normal,
                feature_id,
            },
        );
    }
    Ok(())
}

fn radial_normal(first: f64, second: f64, perpendicular: [usize; 2]) -> Result<Vec3f, WaterError> {
    let mut components = [0.0; 3];
    components[perpendicular[0]] = first;
    components[perpendicular[1]] = second;
    normalize(
        Vec3f::new(components[0], components[1], components[2]),
        "aperture edge normal",
    )
}

fn normalize(vector: Vec3f, phase: &str) -> Result<Vec3f, WaterError> {
    let length_squared = checked_scalar(vector.dot(vector), phase)?;
    if length_squared <= 0.0 {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            format!("{phase} has no direction"),
        ));
    }
    vector
        .scale(checked_scalar(1.0 / length_squared.sqrt(), phase)?)
        .checked(phase)
}

fn is_meaningfully_inward(displacement: Vec3f, normal: Vec3f) -> Result<bool, WaterError> {
    let direction = checked_scalar(displacement.dot(normal), "contact approach direction")?;
    let length = checked_scalar(
        displacement.dot(displacement),
        "contact approach displacement length",
    )?
    .sqrt();
    Ok(direction < -(CONTACT_DIRECTION_ROUNDING_GUARD * length))
}

fn select_hit(best: &mut Option<SweepHit>, candidate: SweepHit) {
    if best.is_none_or(|current| {
        candidate.time < current.time
            || (candidate.time.to_bits() == current.time.to_bits()
                && candidate.feature_id < current.feature_id)
    }) {
        *best = Some(candidate);
    }
}

fn record_feature_impulse(
    result: &mut VelocityProjectionResult,
    feature_id: u32,
    delta_velocity: Vec3f,
) -> Result<(), WaterError> {
    let index = usize::try_from(feature_id)
        .ok()
        .filter(|index| *index < CONTACT_FEATURE_CAPACITY)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "contact feature ID exceeds capacity"))?;
    let impulse = delta_velocity
        .scale(UNIFORM_MASS)
        .checked("contact feature impulse")?;
    result.feature_fluid_impulses[index] = result.feature_fluid_impulses[index]
        .add(impulse)
        .checked("contact feature impulse reduction")?;
    result.feature_active_constraints[index] = result.feature_active_constraints[index]
        .checked_add(1)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "contact feature count overflow"))?;
    Ok(())
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
        kkt_error_ppb: None,
        maximum_multiplier_bits,
        boundary_impulse,
    })
}

pub(super) fn density_pressure_operator(
    reconstruction: &Reconstruction,
    multiplier: &[f64],
) -> Result<Vec<f64>, WaterError> {
    density_pressure_operator_with_workers(reconstruction, multiplier, None)
}

pub(super) fn density_pressure_operator_with_workers(
    reconstruction: &Reconstruction,
    multiplier: &[f64],
    workers: Option<&DeterministicWorkers>,
) -> Result<Vec<f64>, WaterError> {
    let acceleration = pressure_acceleration_with_workers(reconstruction, multiplier, workers)?;
    let matrix = matrix_action_with_workers(reconstruction, &acceleration.total, workers)?;
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

#[cfg(test)]
mod contact_tests {
    use super::*;
    use crate::geometry::INTERNAL_PATCH_FEATURE_ID;
    use crate::scenario;

    fn project(position: Vec3f, velocity: Vec3f) -> (Vec3f, VelocityProjectionResult) {
        let geometry = scenario::find("CW-ORIFICE-001").unwrap().geometry;
        let mut velocities = [velocity];
        let result = project_predictive_geometry(geometry, &[position], &mut velocities).unwrap();
        (velocities[0], result)
    }

    #[test]
    fn internal_face_separating_resting_direct_and_high_speed_cases_are_bounded() {
        let contact = Vec3f::new(0.975, 0.1, 0.5);
        assert_eq!(project(contact, Vec3f::new(-1.0, 0.0, 0.0)).0.x, -1.0);
        assert_eq!(project(contact, Vec3f::ZERO).0.x, 0.0);
        let (direct, direct_result) = project(contact, Vec3f::new(1.0, 0.0, 0.0));
        assert_eq!(direct.x.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            direct_result.kinetic_energy_delta.to_bits(),
            (-0.0625_f64).to_bits()
        );
        assert_eq!(
            direct_result.feature_active_constraints[INTERNAL_PATCH_FEATURE_ID as usize],
            1
        );

        let (fast, fast_result) = project(Vec3f::new(0.9, 0.1, 0.5), Vec3f::new(30.0, 0.0, 0.0));
        assert_eq!(
            crate::profile::quantize_velocity(fast.x).unwrap(),
            18_000_000
        );
        assert_eq!(fast_result.active_rows, 1);
        assert_eq!(
            fast_result.feature_active_constraints[INTERNAL_PATCH_FEATURE_ID as usize],
            1
        );
    }

    #[test]
    fn aperture_pass_and_exact_edge_graze_remain_unconstrained() {
        let velocity = Vec3f::new(30.0, 0.0, 0.0);
        let (centre, centre_result) = project(Vec3f::new(0.9, 0.3, 0.5), velocity);
        assert_eq!(centre.x.to_bits(), velocity.x.to_bits());
        assert_eq!(centre_result.active_rows, 0);

        let (graze, graze_result) = project(Vec3f::new(0.9, 0.225, 0.5), velocity);
        assert_eq!(graze.x.to_bits(), velocity.x.to_bits());
        assert_eq!(graze_result.active_rows, 0);
    }

    #[test]
    fn aperture_edge_and_simultaneous_corner_constraints_use_stable_feature_ids() {
        let velocity = Vec3f::new(30.0, 0.0, 0.0);
        let (edge, edge_result) = project(Vec3f::new(0.9, 0.22, 0.5), velocity);
        assert!(edge.x < velocity.x);
        assert!(edge.y > 0.0, "edge projection was {edge:?}");
        assert_eq!(
            edge_result.feature_active_constraints[APERTURE_Y_MIN_EDGE_FEATURE_ID as usize],
            1
        );

        let (corner, corner_result) = project(Vec3f::new(0.9, 0.22, 0.42), velocity);
        assert!(corner.x < velocity.x);
        assert!(corner.y > 0.0, "corner projection was {corner:?}");
        assert!(corner.z > 0.0, "corner projection was {corner:?}");
        assert_eq!(
            corner_result.feature_active_constraints[APERTURE_Y_MIN_EDGE_FEATURE_ID as usize],
            1
        );
        assert_eq!(
            corner_result.feature_active_constraints[APERTURE_Z_MIN_EDGE_FEATURE_ID as usize],
            1
        );
    }
}
