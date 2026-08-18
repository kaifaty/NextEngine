#![forbid(unsafe_code)]

use crate::error::{AUDIT_INVALID, WaterError};
use crate::geometry::GeometryObservation;
use crate::model::{CanonicalSample, Geometry, Vec3i};
use sha2::{Digest, Sha256};

use super::{
    ContactFixtureInput, ContactFixtureObservation, DensityFixtureObservation,
    DensitySupportRecord, FeatureActivation,
};

const SPACING_UM: i64 = 50_000;
const RADIUS_UM: i64 = 25_000;
const SUPPORT_RADIUS_SQUARED_UM: i128 = 10_000_000_000;
const SCALE: f64 = f64::from_bits(0x412e_8480_0000_0000);
const H: f64 = f64::from_bits(0x3fb9_9999_9999_999a);
const VOLUME: f64 = f64::from_bits(0x3f20_624d_d2f1_a9fc);
const K: f64 = f64::from_bits(0x40a3_e4f5_4b37_0dcf);
const L: f64 = f64::from_bits(0x40cd_d76f_f0d2_94b6);
const WALL_FEATURE: u32 = 16;
const DT: f64 = f64::from_bits(0x3f71_1111_1111_1111);
const INV_DT: f64 = f64::from_bits(0x406e_0000_0000_0000);
const RADIUS: f64 = f64::from_bits(0x3f99_9999_9999_999a);
const MASS: f64 = f64::from_bits(0x3fc0_0000_0000_0000);
const DIRECTION_GUARD: f64 = 32.0 * f64::EPSILON;

pub(super) fn geometry_bytes(geometry: Geometry) -> Vec<u8> {
    let mut text = String::new();
    text.push_str("GEOMETRY_MANIFEST_V1_BEGIN\n");
    text.push_str("outer.feature_ids=0,1,2,3,4,5\n");
    text.push_str(&format!(
        "outer.box_um=({},{},{})..({},{},{})\n",
        geometry.bounds.min.x,
        geometry.bounds.min.y,
        geometry.bounds.min.z,
        geometry.bounds.max.x,
        geometry.bounds.max.y,
        geometry.bounds.max.z
    ));
    if let Some(opening) = geometry.aperture {
        text.push_str(&format!(
            "patch.16.axis=X;coordinate_um={};y={}..{};z={}..{}\n",
            opening.wall_x_um,
            geometry.bounds.min.y,
            geometry.bounds.max.y,
            geometry.bounds.min.z,
            geometry.bounds.max.z
        ));
        text.push_str(&format!(
            "patch.16.opening.17=y={}..{};z={}..{};membership=closed\n",
            opening.y_min_um, opening.y_max_um, opening.z_min_um, opening.z_max_um
        ));
        text.push_str("patch.16.edge_feature_ids=17,18,19,20;corner_feature_ids=21,22,23,24\n");
    }
    text.push_str("GEOMETRY_MANIFEST_V1_END\n");
    text.into_bytes()
}

pub(super) fn geometry_root(geometry: Geometry) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.continuum-water.geometry-manifest.v1\0");
    hasher.update(geometry_bytes(geometry));
    hasher.finalize().into()
}

pub(super) fn geometry_observations(
    geometry: Geometry,
    positions: &[Vec3i],
) -> Result<Vec<GeometryObservation>, WaterError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(positions.len())
        .map_err(reserve_error)?;
    for position in positions.iter().copied() {
        let outside = position.x < geometry.bounds.min.x
            || position.x > geometry.bounds.max.x
            || position.y < geometry.bounds.min.y
            || position.y > geometry.bounds.max.y
            || position.z < geometry.bounds.min.z
            || position.z > geometry.bounds.max.z;
        let classification = if outside {
            "OUTER_SOLID"
        } else if let Some(opening) = geometry.aperture {
            if position.x == opening.wall_x_um {
                if position.y >= opening.y_min_um
                    && position.y <= opening.y_max_um
                    && position.z >= opening.z_min_um
                    && position.z <= opening.z_max_um
                {
                    "OPENING"
                } else {
                    "INTERNAL_SOLID"
                }
            } else {
                "FLUID_DOMAIN"
            }
        } else {
            "FLUID_DOMAIN"
        };
        let mut closest = (u32::MAX, i128::MAX);
        for (feature, distance) in [
            (0, position.x as i128 - geometry.bounds.min.x as i128),
            (1, position.x as i128 - geometry.bounds.max.x as i128),
            (2, position.y as i128 - geometry.bounds.min.y as i128),
            (3, position.y as i128 - geometry.bounds.max.y as i128),
            (4, position.z as i128 - geometry.bounds.min.z as i128),
            (5, position.z as i128 - geometry.bounds.max.z as i128),
        ] {
            choose(&mut closest, feature, distance.abs() * distance.abs());
        }
        if let Some(opening) = geometry.aperture {
            let dx = (position.x as i128 - opening.wall_x_um as i128).abs();
            let dx2 = dx * dx;
            let in_opening = position.y >= opening.y_min_um
                && position.y <= opening.y_max_um
                && position.z >= opening.z_min_um
                && position.z <= opening.z_max_um;
            if in_opening {
                for (feature, distance) in [
                    (17, position.y as i128 - opening.y_min_um as i128),
                    (18, position.y as i128 - opening.y_max_um as i128),
                    (19, position.z as i128 - opening.z_min_um as i128),
                    (20, position.z as i128 - opening.z_max_um as i128),
                ] {
                    choose(
                        &mut closest,
                        feature,
                        dx2 + (distance.abs() * distance.abs()),
                    );
                }
            } else {
                choose(&mut closest, WALL_FEATURE, dx2);
            }
        }
        result.push(GeometryObservation {
            position_um: position,
            classification: classification.to_owned(),
            closest_feature_id: closest.0,
            closest_distance_squared_um2: closest.1,
        });
    }
    Ok(result)
}

fn choose(closest: &mut (u32, i128), feature: u32, distance: i128) {
    if distance < closest.1 || (distance == closest.1 && feature < closest.0) {
        *closest = (feature, distance);
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Visibility {
    All,
    Left(i64),
    Right(i64),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Support {
    position: Vec3i,
    visibility: Visibility,
    feature: u32,
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
        ((self.x * other.x) + (self.y * other.y)) + (self.z * other.z)
    }
}

#[derive(Clone, Copy)]
struct KernelSample {
    value: f64,
    gradient: F3,
}

#[derive(Clone, Copy)]
struct ContactHit {
    time: f64,
    normal: F3,
    feature: u32,
}

#[derive(Clone, Copy, Default)]
struct ContactResult {
    velocity: F3,
    total_impulse: F3,
    feature_impulses: [F3; 25],
    feature_counts: [u32; 25],
}

pub(super) fn contact_fixtures(
    geometry: Geometry,
    inputs: &[ContactFixtureInput],
) -> Result<Vec<ContactFixtureObservation>, WaterError> {
    let opening = geometry.aperture.ok_or_else(|| {
        WaterError::new(
            AUDIT_INVALID,
            "independent contact fixture requires an aperture",
        )
    })?;
    let mut observations = Vec::new();
    observations
        .try_reserve_exact(inputs.len())
        .map_err(reserve_error)?;
    for input in inputs {
        let start = F3::new(
            input.position_um.x as f64 / SCALE,
            input.position_um.y as f64 / SCALE,
            input.position_um.z as f64 / SCALE,
        );
        let before = F3::new(
            input.velocity_um_s.x as f64 / SCALE,
            input.velocity_um_s.y as f64 / SCALE,
            input.velocity_um_s.z as f64 / SCALE,
        );
        let projected = independent_contact(start, before, opening)?;
        let delta = projected.velocity.sub(before);
        let active_components = [delta.x, delta.y, delta.z]
            .into_iter()
            .filter(|component| *component != 0.0)
            .count();
        let mut features = Vec::new();
        for feature in 0..25 {
            if projected.feature_counts[feature] != 0 {
                features.push(FeatureActivation {
                    feature_id: feature as u32,
                    active_constraints: projected.feature_counts[feature],
                    fluid_impulse_bits: vector_bits(projected.feature_impulses[feature]),
                });
            }
        }
        observations.push(ContactFixtureObservation {
            id: input.id.to_owned(),
            position_um: input.position_um,
            velocity_before_um_s: input.velocity_um_s,
            velocity_after_bits: vector_bits(projected.velocity),
            fluid_impulse_bits: vector_bits(projected.total_impulse),
            active_rows: usize::from(active_components != 0),
            active_components,
            features,
        });
    }
    Ok(observations)
}

fn independent_contact(
    start: F3,
    velocity: F3,
    opening: crate::model::Aperture,
) -> Result<ContactResult, WaterError> {
    let mut current = start;
    let mut remaining = velocity.scale(DT);
    let mut result = ContactResult {
        velocity,
        ..ContactResult::default()
    };
    let mut contacted = false;
    for _ in 0..8 {
        let Some(hit) = independent_earliest_hit(current, remaining, opening)? else {
            current = finite_vec(current.add(remaining), "independent contact final advance")?;
            remaining = F3::ZERO;
            break;
        };
        current = finite_vec(
            current.add(remaining.scale(hit.time)),
            "independent contact impact advance",
        )?;
        let tail = finite_vec(
            remaining.scale(finite(1.0 - hit.time, "independent contact remainder")?),
            "independent contact tail",
        )?;
        let inward = finite(tail.dot(hit.normal), "independent contact inward")?;
        if inward >= 0.0 {
            current = finite_vec(current.add(tail), "independent contact free tail")?;
            remaining = F3::ZERO;
            break;
        }
        let correction = finite_vec(hit.normal.scale(-inward), "independent contact correction")?;
        let delta_velocity = finite_vec(
            correction.scale(INV_DT),
            "independent contact delta velocity",
        )?;
        let feature = hit.feature as usize;
        result.feature_counts[feature] = result.feature_counts[feature]
            .checked_add(1)
            .ok_or_else(|| WaterError::new(AUDIT_INVALID, "independent contact count overflow"))?;
        result.feature_impulses[feature] = finite_vec(
            result.feature_impulses[feature].add(delta_velocity.scale(MASS)),
            "independent feature impulse",
        )?;
        remaining = finite_vec(tail.add(correction), "independent projected tail")?;
        contacted = true;
    }
    if remaining.x != 0.0 || remaining.y != 0.0 || remaining.z != 0.0 {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "independent contact exceeded eight constraints",
        ));
    }
    if contacted {
        result.velocity = finite_vec(
            current.sub(start).scale(INV_DT),
            "independent published contact velocity",
        )?;
    }
    result.total_impulse = finite_vec(
        result.velocity.sub(velocity).scale(MASS),
        "independent total contact impulse",
    )?;
    Ok(result)
}

fn independent_earliest_hit(
    start: F3,
    displacement: F3,
    opening: crate::model::Aperture,
) -> Result<Option<ContactHit>, WaterError> {
    let wall = opening.wall_x_um as f64 / SCALE;
    let y_min = opening.y_min_um as f64 / SCALE;
    let y_max = opening.y_max_um as f64 / SCALE;
    let z_min = opening.z_min_um as f64 / SCALE;
    let z_max = opening.z_max_um as f64 / SCALE;
    let mut best = None;
    if displacement.x > 0.0 && start.x <= wall {
        independent_face(
            &mut best,
            start,
            displacement,
            wall - RADIUS,
            F3::new(-1.0, 0.0, 0.0),
            opening,
        )?;
    }
    if displacement.x < 0.0 && start.x >= wall {
        independent_face(
            &mut best,
            start,
            displacement,
            wall + RADIUS,
            F3::new(1.0, 0.0, 0.0),
            opening,
        )?;
    }
    independent_capsule(
        &mut best,
        start,
        displacement,
        F3::new(wall, y_min, z_min),
        F3::new(wall, y_min, z_max),
        2,
        17,
    )?;
    independent_capsule(
        &mut best,
        start,
        displacement,
        F3::new(wall, y_max, z_min),
        F3::new(wall, y_max, z_max),
        2,
        18,
    )?;
    independent_capsule(
        &mut best,
        start,
        displacement,
        F3::new(wall, y_min, z_min),
        F3::new(wall, y_max, z_min),
        1,
        19,
    )?;
    independent_capsule(
        &mut best,
        start,
        displacement,
        F3::new(wall, y_min, z_max),
        F3::new(wall, y_max, z_max),
        1,
        20,
    )?;
    Ok(best)
}

fn independent_face(
    best: &mut Option<ContactHit>,
    start: F3,
    displacement: F3,
    plane: f64,
    normal: F3,
    opening: crate::model::Aperture,
) -> Result<(), WaterError> {
    let time = finite((plane - start.x) / displacement.x, "independent face time")?;
    if !(0.0..=1.0).contains(&time) {
        return Ok(());
    }
    let point = finite_vec(
        start.add(displacement.scale(time)),
        "independent face point",
    )?;
    let in_opening = point.y >= opening.y_min_um as f64 / SCALE
        && point.y <= opening.y_max_um as f64 / SCALE
        && point.z >= opening.z_min_um as f64 / SCALE
        && point.z <= opening.z_max_um as f64 / SCALE;
    if !in_opening {
        independent_select(
            best,
            ContactHit {
                time,
                normal,
                feature: WALL_FEATURE,
            },
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn independent_capsule(
    best: &mut Option<ContactHit>,
    start: F3,
    displacement: F3,
    segment_start: F3,
    segment_end: F3,
    axis: usize,
    feature: u32,
) -> Result<(), WaterError> {
    let s = [start.x, start.y, start.z];
    let d = [displacement.x, displacement.y, displacement.z];
    let e = [segment_start.x, segment_start.y, segment_start.z];
    let perpendicular = if axis == 1 { [0, 2] } else { [0, 1] };
    let r0 = finite(
        s[perpendicular[0]] - e[perpendicular[0]],
        "independent radial 0",
    )?;
    let r1 = finite(
        s[perpendicular[1]] - e[perpendicular[1]],
        "independent radial 1",
    )?;
    let d0 = d[perpendicular[0]];
    let d1 = d[perpendicular[1]];
    let a = finite((d0 * d0) + (d1 * d1), "independent capsule a")?;
    let b = finite(2.0 * ((r0 * d0) + (r1 * d1)), "independent capsule b")?;
    let c = finite(
        (r0 * r0) + (r1 * r1) - (RADIUS * RADIUS),
        "independent capsule c",
    )?;
    let segment_min = [segment_start.x, segment_start.y, segment_start.z][axis];
    let segment_max = [segment_end.x, segment_end.y, segment_end.z][axis];
    if c <= 0.0 && s[axis] > segment_min && s[axis] < segment_max {
        let normal = independent_radial_normal(r0, r1, perpendicular)?;
        if independent_inward(displacement, normal)? {
            independent_select(
                best,
                ContactHit {
                    time: 0.0,
                    normal,
                    feature,
                },
            );
        }
    }
    if a == 0.0 {
        return Ok(());
    }
    let discriminant = finite((b * b) - (4.0 * a * c), "independent discriminant")?;
    if discriminant < 0.0 {
        return Ok(());
    }
    let time = finite(
        (-b - discriminant.sqrt()) / (2.0 * a),
        "independent capsule time",
    )?;
    if !(0.0..=1.0).contains(&time) {
        return Ok(());
    }
    let along = finite(s[axis] + (d[axis] * time), "independent capsule coordinate")?;
    if along <= segment_min || along >= segment_max {
        return Ok(());
    }
    let normal = independent_radial_normal(
        finite(r0 + (d0 * time), "independent hit radial 0")?,
        finite(r1 + (d1 * time), "independent hit radial 1")?,
        perpendicular,
    )?;
    if independent_inward(displacement, normal)? {
        independent_select(
            best,
            ContactHit {
                time,
                normal,
                feature,
            },
        );
    }
    Ok(())
}

fn independent_radial_normal(
    first: f64,
    second: f64,
    perpendicular: [usize; 2],
) -> Result<F3, WaterError> {
    let mut values = [0.0; 3];
    values[perpendicular[0]] = first;
    values[perpendicular[1]] = second;
    let vector = F3::new(values[0], values[1], values[2]);
    let length = finite(vector.dot(vector), "independent normal length")?.sqrt();
    if length == 0.0 {
        return Err(WaterError::new(AUDIT_INVALID, "independent normal is zero"));
    }
    finite_vec(vector.scale(1.0 / length), "independent normal")
}

fn independent_inward(displacement: F3, normal: F3) -> Result<bool, WaterError> {
    let direction = finite(displacement.dot(normal), "independent contact direction")?;
    let length = finite(
        displacement.dot(displacement),
        "independent displacement length",
    )?
    .sqrt();
    Ok(direction < -(DIRECTION_GUARD * length))
}

fn independent_select(best: &mut Option<ContactHit>, candidate: ContactHit) {
    if best.is_none_or(|current| {
        candidate.time < current.time
            || (candidate.time.to_bits() == current.time.to_bits()
                && candidate.feature < current.feature)
    }) {
        *best = Some(candidate);
    }
}

pub(super) fn density_support_records(
    geometry: Geometry,
) -> Result<Vec<DensitySupportRecord>, WaterError> {
    let support = build_support(geometry)?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(support.len())
        .map_err(reserve_error)?;
    for (index, sample) in support.into_iter().enumerate() {
        records.push(DensitySupportRecord {
            id: u32::try_from(index)
                .map_err(|_| WaterError::new(AUDIT_INVALID, "independent support ID overflow"))?,
            position_um: sample.position,
            volume_bits: format!("0x{:016x}", VOLUME.to_bits()),
            feature_id: sample.feature,
            support: match sample.visibility {
                Visibility::All => "UNRESTRICTED".to_owned(),
                Visibility::Left(wall) => format!("FLUID_X_LESS_THAN:{wall}"),
                Visibility::Right(wall) => format!("FLUID_X_GREATER_THAN:{wall}"),
            },
        });
    }
    Ok(records)
}

pub(super) fn density_fixtures(
    geometry: Geometry,
    samples: &[CanonicalSample],
    selections: &[(&str, u32)],
) -> Result<Vec<DensityFixtureObservation>, WaterError> {
    let mut fluid = samples.to_vec();
    fluid.sort_unstable_by_key(|sample| sample.id);
    let support = build_support(geometry)?;
    let mut observations = Vec::new();
    observations
        .try_reserve_exact(selections.len())
        .map_err(reserve_error)?;
    for (role, sample_id) in selections {
        let index = fluid
            .binary_search_by_key(sample_id, |sample| sample.id)
            .map_err(|_| {
                WaterError::new(
                    AUDIT_INVALID,
                    format!("independent fixture sample {sample_id} is missing"),
                )
            })?;
        let position = fluid[index].position_um;
        let mut fluid_neighbors = Vec::new();
        for (other_index, other) in fluid.iter().enumerate() {
            if other_index == index {
                continue;
            }
            let displacement = subtract(position, other.position_um)?;
            if squared(displacement) <= SUPPORT_RADIUS_SQUARED_UM
                && !blocks_internal_wall(geometry, position, other.position_um)?
            {
                fluid_neighbors.push((other.id, kernel(displacement)?));
            }
        }
        fluid_neighbors.sort_unstable_by_key(|(id, _)| *id);
        let mut boundary_neighbors = Vec::new();
        for (boundary_index, boundary) in support.iter().copied().enumerate() {
            if !visible(boundary.visibility, position.x) {
                continue;
            }
            let displacement = subtract(position, boundary.position)?;
            if squared(displacement) <= SUPPORT_RADIUS_SQUARED_UM {
                boundary_neighbors.push((boundary_index, kernel(displacement)?));
            }
        }
        boundary_neighbors.sort_unstable_by_key(|(index, _)| *index);

        let mut rho_ratio = finite(VOLUME * K, "independent fixture self density")?;
        let mut sum_sq = 0.0;
        let mut central = F3::ZERO;
        let mut fluid_gradient = F3::ZERO;
        for (_, sampled) in &fluid_neighbors {
            rho_ratio = finite(
                rho_ratio + (VOLUME * sampled.value),
                "independent fixture fluid density",
            )?;
            let volume_gradient = finite_vec(
                sampled.gradient.scale(VOLUME),
                "independent fixture fluid volume gradient",
            )?;
            fluid_gradient = finite_vec(
                fluid_gradient.add(volume_gradient),
                "independent fixture fluid gradient reduction",
            )?;
            let g = F3::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z);
            sum_sq = finite(sum_sq + g.dot(g), "independent fixture factor squares")?;
            central = finite_vec(central.sub(g), "independent fixture central fluid gradient")?;
        }
        let mut boundary_gradient = F3::ZERO;
        for (_, sampled) in &boundary_neighbors {
            rho_ratio = finite(
                rho_ratio + (VOLUME * sampled.value),
                "independent fixture boundary density",
            )?;
            let volume_gradient = finite_vec(
                sampled.gradient.scale(VOLUME),
                "independent fixture boundary volume gradient",
            )?;
            boundary_gradient = finite_vec(
                boundary_gradient.add(volume_gradient),
                "independent fixture boundary gradient reduction",
            )?;
            let g = F3::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z);
            central = finite_vec(
                central.sub(g),
                "independent fixture central boundary gradient",
            )?;
        }
        let denominator = finite(
            sum_sq + central.dot(central),
            "independent fixture factor denominator",
        )?;
        let alpha = if denominator > 1.0e-5 {
            finite(1.0 / denominator, "independent fixture factor reciprocal")?
        } else {
            0.0
        };
        let total_gradient = finite_vec(
            fluid_gradient.add(boundary_gradient),
            "independent fixture total gradient",
        )?;
        observations.push(DensityFixtureObservation {
            role: (*role).to_owned(),
            sample_id: *sample_id,
            position_um: position,
            fluid_neighbor_count: fluid_neighbors.len(),
            boundary_neighbor_count: boundary_neighbors.len(),
            rho_ratio_bits: bits(rho_ratio),
            alpha_bits: bits(alpha),
            fluid_gradient_bits: vector_bits(fluid_gradient),
            boundary_gradient_bits: vector_bits(boundary_gradient),
            total_gradient_bits: vector_bits(total_gradient),
        });
    }
    Ok(observations)
}

fn build_support(geometry: Geometry) -> Result<Vec<Support>, WaterError> {
    let spans = [
        axis_cells(geometry.bounds.min.x, geometry.bounds.max.x)?,
        axis_cells(geometry.bounds.min.y, geometry.bounds.max.y)?,
        axis_cells(geometry.bounds.min.z, geometry.bounds.max.z)?,
    ];
    let mut support = Vec::new();
    for ix in -2..=spans[0] + 1 {
        for iy in -2..=spans[1] + 1 {
            for iz in -2..=spans[2] + 1 {
                let feature = if ix < 0 {
                    Some(0)
                } else if ix >= spans[0] {
                    Some(1)
                } else if iy < 0 {
                    Some(2)
                } else if iy >= spans[1] {
                    Some(3)
                } else if iz < 0 {
                    Some(4)
                } else if iz >= spans[2] {
                    Some(5)
                } else {
                    None
                };
                if let Some(feature) = feature {
                    support.push(Support {
                        position: Vec3i::new(
                            centre(geometry.bounds.min.x, ix)?,
                            centre(geometry.bounds.min.y, iy)?,
                            centre(geometry.bounds.min.z, iz)?,
                        ),
                        visibility: Visibility::All,
                        feature,
                    });
                }
            }
        }
    }
    if let Some(opening) = geometry.aperture {
        for side in [
            Visibility::Left(opening.wall_x_um),
            Visibility::Right(opening.wall_x_um),
        ] {
            for layer in 0_i64..2 {
                let distance = RADIUS_UM + (SPACING_UM * layer);
                let x = match side {
                    Visibility::Left(wall) => wall + distance,
                    Visibility::Right(wall) => wall - distance,
                    Visibility::All => unreachable!(),
                };
                for iy in 0..spans[1] {
                    let y = centre(geometry.bounds.min.y, iy)?;
                    for iz in 0..spans[2] {
                        let z = centre(geometry.bounds.min.z, iz)?;
                        if y >= opening.y_min_um
                            && y <= opening.y_max_um
                            && z >= opening.z_min_um
                            && z <= opening.z_max_um
                        {
                            continue;
                        }
                        support.push(Support {
                            position: Vec3i::new(x, y, z),
                            visibility: side,
                            feature: WALL_FEATURE,
                        });
                    }
                }
            }
        }
    }
    support.sort_unstable();
    support.dedup();
    Ok(support)
}

fn blocks_internal_wall(geometry: Geometry, start: Vec3i, end: Vec3i) -> Result<bool, WaterError> {
    let Some(opening) = geometry.aperture else {
        return Ok(false);
    };
    let side0 = start.x - opening.wall_x_um;
    let side1 = end.x - opening.wall_x_um;
    if !((side0 < 0 && side1 > 0) || (side0 > 0 && side1 < 0) || side0 == 0 || side1 == 0) {
        return Ok(false);
    }
    let denominator = i128::from(end.x) - i128::from(start.x);
    if denominator == 0 {
        return Ok(!(opening.y_min_um <= start.y
            && start.y <= opening.y_max_um
            && opening.z_min_um <= start.z
            && start.z <= opening.z_max_um));
    }
    let numerator = i128::from(opening.wall_x_um) - i128::from(start.x);
    let mut denominator = denominator;
    let mut numerator = numerator;
    if denominator < 0 {
        denominator = -denominator;
        numerator = -numerator;
    }
    let y =
        i128::from(start.y) * denominator + (i128::from(end.y) - i128::from(start.y)) * numerator;
    let z =
        i128::from(start.z) * denominator + (i128::from(end.z) - i128::from(start.z)) * numerator;
    let in_opening = y >= i128::from(opening.y_min_um) * denominator
        && y <= i128::from(opening.y_max_um) * denominator
        && z >= i128::from(opening.z_min_um) * denominator
        && z <= i128::from(opening.z_max_um) * denominator;
    Ok(!in_opening)
}

fn visible(visibility: Visibility, x: i64) -> bool {
    match visibility {
        Visibility::All => true,
        Visibility::Left(wall) => x < wall,
        Visibility::Right(wall) => x > wall,
    }
}

fn kernel(displacement_um: Vec3i) -> Result<KernelSample, WaterError> {
    let dx = (displacement_um.x as f64) / SCALE;
    let dy = (displacement_um.y as f64) / SCALE;
    let dz = (displacement_um.z as f64) / SCALE;
    let r2 = finite(((dx * dx) + (dy * dy)) + (dz * dz), "independent kernel r2")?;
    let r = finite(r2.sqrt(), "independent kernel r")?;
    let q = finite(r / H, "independent kernel q")?;
    if q > 1.0 {
        return Ok(KernelSample {
            value: 0.0,
            gradient: F3::ZERO,
        });
    }
    let value = if q <= 0.5 {
        let q2 = finite(q * q, "independent kernel q2")?;
        let q3 = finite(q2 * q, "independent kernel q3")?;
        let six_q3 = finite(6.0 * q3, "independent kernel 6q3")?;
        let six_q2 = finite(6.0 * q2, "independent kernel 6q2")?;
        finite(
            K * ((six_q3 - six_q2) + 1.0),
            "independent kernel inner value",
        )?
    } else {
        let t = finite(1.0 - q, "independent kernel t")?;
        finite(K * (2.0 * ((t * t) * t)), "independent kernel outer value")?
    };
    let gradient = if displacement_um == Vec3i::new(0, 0, 0) {
        F3::ZERO
    } else {
        let grad_q = F3::new(
            finite((dx / r) / H, "independent kernel grad q x")?,
            finite((dy / r) / H, "independent kernel grad q y")?,
            finite((dz / r) / H, "independent kernel grad q z")?,
        );
        let coefficient = if q <= 0.5 {
            let three_q = finite(3.0 * q, "independent kernel 3q")?;
            let lq = finite(L * q, "independent kernel lq")?;
            finite(lq * (three_q - 2.0), "independent kernel inner gradient")?
        } else {
            let t = finite(1.0 - q, "independent kernel gradient t")?;
            finite(-(L * (t * t)), "independent kernel outer gradient")?
        };
        finite_vec(grad_q.scale(coefficient), "independent kernel gradient")?
    };
    Ok(KernelSample { value, gradient })
}

fn axis_cells(minimum: i64, maximum: i64) -> Result<i64, WaterError> {
    let span = maximum - minimum;
    if span <= 0 || span % SPACING_UM != 0 {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "independent axis is off lattice",
        ));
    }
    Ok(span / SPACING_UM)
}

fn centre(minimum: i64, index: i64) -> Result<i64, WaterError> {
    minimum
        .checked_add(RADIUS_UM)
        .and_then(|value| value.checked_add(index.checked_mul(SPACING_UM)?))
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "independent support coordinate overflow"))
}

fn subtract(left: Vec3i, right: Vec3i) -> Result<Vec3i, WaterError> {
    Ok(Vec3i::new(
        left.x
            .checked_sub(right.x)
            .ok_or_else(|| WaterError::new(AUDIT_INVALID, "independent x subtraction overflow"))?,
        left.y
            .checked_sub(right.y)
            .ok_or_else(|| WaterError::new(AUDIT_INVALID, "independent y subtraction overflow"))?,
        left.z
            .checked_sub(right.z)
            .ok_or_else(|| WaterError::new(AUDIT_INVALID, "independent z subtraction overflow"))?,
    ))
}

fn squared(value: Vec3i) -> i128 {
    let x = i128::from(value.x);
    let y = i128::from(value.y);
    let z = i128::from(value.z);
    ((x * x) + (y * y)) + (z * z)
}

fn finite(value: f64, phase: &str) -> Result<f64, WaterError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(WaterError::new(AUDIT_INVALID, format!("nonfinite {phase}")))
    }
}

fn finite_vec(value: F3, phase: &str) -> Result<F3, WaterError> {
    if value.x.is_finite() && value.y.is_finite() && value.z.is_finite() {
        Ok(value)
    } else {
        Err(WaterError::new(AUDIT_INVALID, format!("nonfinite {phase}")))
    }
}

fn bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn vector_bits(value: F3) -> [String; 3] {
    [bits(value.x), bits(value.y), bits(value.z)]
}

fn reserve_error(error: std::collections::TryReserveError) -> WaterError {
    WaterError::new(
        AUDIT_INVALID,
        format!("independent successor allocation failed: {error}"),
    )
}
