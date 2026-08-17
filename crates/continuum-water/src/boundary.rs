#![forbid(unsafe_code)]

use crate::error::{
    BOUNDARY_CAPACITY_EXCEEDED, BOUNDARY_ESCAPE, BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
    BOUNDARY_PENETRATION_LIMIT, NUMERIC_OVERFLOW, SCENARIO_INVALID, WaterError,
};
use crate::kernel;
use crate::model::{CanonicalSample, Geometry, Vec3i, checked_scalar};
use crate::profile::{
    GRID_CELL_WIDTH_UM, LATTICE_SPACING_UM, MAXIMUM_BOUNDARY_SAMPLES,
    MAXIMUM_DIRECTED_BOUNDARY_NEIGHBORS, MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW, MINIMUM_CLEARANCE_UM,
    PARTICLE_RADIUS_UM, SUPPORT_RADIUS_UM,
};
use crate::scenario::validate_capacity;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BoundarySample {
    pub(crate) id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) volume: f64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CellKey {
    x: i64,
    y: i64,
    z: i64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GridEntry {
    key: CellKey,
    id: u32,
    index: usize,
}

pub(crate) fn build(geometry: Geometry) -> Result<Vec<BoundarySample>, WaterError> {
    let mut positions = outer_lattice(geometry)?;
    if let Some(aperture) = geometry.aperture {
        for y in lattice_axis(geometry.bounds.min.y, geometry.bounds.max.y)? {
            for z in lattice_axis(geometry.bounds.min.z, geometry.bounds.max.z)? {
                let inside_opening = y >= aperture.y_min_um
                    && y <= aperture.y_max_um
                    && z >= aperture.z_min_um
                    && z <= aperture.z_max_um;
                if !inside_opening {
                    push_position(&mut positions, Vec3i::new(aperture.wall_x_um, y, z))?;
                }
            }
        }
    }
    positions.sort_unstable();
    positions.dedup();
    validate_capacity(
        positions.len(),
        MAXIMUM_BOUNDARY_SAMPLES,
        BOUNDARY_CAPACITY_EXCEEDED,
        "boundary samples",
    )?;

    let mut entries = Vec::new();
    entries
        .try_reserve_exact(positions.len())
        .map_err(|error| {
            WaterError::new(
                BOUNDARY_CAPACITY_EXCEEDED,
                format!("cannot reserve boundary grid: {error}"),
            )
        })?;
    for (index, position) in positions.iter().copied().enumerate() {
        entries.push(GridEntry {
            key: cell_key(position),
            id: u32::try_from(index)
                .map_err(|_| WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "boundary id overflow"))?,
            index,
        });
    }
    entries.sort_unstable();

    let mut volumes = Vec::new();
    volumes
        .try_reserve_exact(positions.len())
        .map_err(|error| {
            WaterError::new(
                BOUNDARY_CAPACITY_EXCEEDED,
                format!("cannot reserve boundary volumes: {error}"),
            )
        })?;
    let mut directed_count = 0_usize;
    for (index, position) in positions.iter().copied().enumerate() {
        let neighbors = admitted_neighbors(index, position, &positions, &entries)?;
        directed_count = directed_count.checked_add(neighbors.len()).ok_or_else(|| {
            WaterError::new(
                BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
                "directed boundary neighbor count overflow",
            )
        })?;
        validate_capacity(
            directed_count,
            MAXIMUM_DIRECTED_BOUNDARY_NEIGHBORS,
            BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
            "directed boundary neighbors",
        )?;
        let mut denominator = kernel::value_at_zero();
        for other_index in neighbors {
            let displacement = position.checked_sub(positions[other_index])?;
            let value = kernel::sample(displacement)?.value;
            denominator =
                checked_scalar(denominator + value, "boundary volume denominator reduction")?;
        }
        if denominator <= 0.0 {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("boundary sample {index} has nonpositive volume denominator"),
            ));
        }
        volumes.push(checked_scalar(1.0 / denominator, "boundary pseudo-volume")?);
    }

    let mut result = Vec::new();
    result.try_reserve_exact(positions.len()).map_err(|error| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            format!("cannot reserve boundary samples: {error}"),
        )
    })?;
    for (index, (position_um, volume)) in positions.into_iter().zip(volumes).enumerate() {
        result.push(BoundarySample {
            id: u32::try_from(index)
                .map_err(|_| WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "boundary id overflow"))?,
            position_um,
            volume,
        });
    }
    Ok(result)
}

pub(crate) fn validate_centres(
    geometry: Geometry,
    samples: &[CanonicalSample],
) -> Result<i64, WaterError> {
    let mut maximum_penetration = 0_i64;
    for sample in samples {
        let position = sample.position_um;
        let bounds = geometry.bounds;
        if position.x < bounds.min.x
            || position.x > bounds.max.x
            || position.y < bounds.min.y
            || position.y > bounds.max.y
            || position.z < bounds.min.z
            || position.z > bounds.max.z
        {
            return Err(WaterError::new(
                BOUNDARY_ESCAPE,
                format!("sample {} escaped the outer box", sample.id),
            ));
        }
        let mut distance_squared = outer_distance_squared(geometry, position)?;
        if let Some(aperture) = geometry.aperture {
            let wall_distance = i128::from(position.x)
                .checked_sub(i128::from(aperture.wall_x_um))
                .ok_or_else(|| {
                    WaterError::new(NUMERIC_OVERFLOW, "wall distance subtraction overflow")
                })?;
            let inside_projection = position.y >= aperture.y_min_um
                && position.y <= aperture.y_max_um
                && position.z >= aperture.z_min_um
                && position.z <= aperture.z_max_um;
            let wall_distance_squared =
                wall_distance.checked_mul(wall_distance).ok_or_else(|| {
                    WaterError::new(NUMERIC_OVERFLOW, "wall distance square overflow")
                })?;
            let solid_distance_squared = if inside_projection {
                let edge_distances = [
                    position.y.checked_sub(aperture.y_min_um),
                    aperture.y_max_um.checked_sub(position.y),
                    position.z.checked_sub(aperture.z_min_um),
                    aperture.z_max_um.checked_sub(position.z),
                ];
                let mut edge_distance = i64::MAX;
                for distance in edge_distances {
                    let distance = distance.ok_or_else(|| {
                        WaterError::new(NUMERIC_OVERFLOW, "aperture edge subtraction overflow")
                    })?;
                    if distance < edge_distance {
                        edge_distance = distance;
                    }
                }
                let edge = i128::from(edge_distance);
                wall_distance_squared
                    .checked_add(edge.checked_mul(edge).ok_or_else(|| {
                        WaterError::new(NUMERIC_OVERFLOW, "aperture edge square overflow")
                    })?)
                    .ok_or_else(|| {
                        WaterError::new(NUMERIC_OVERFLOW, "aperture distance overflow")
                    })?
            } else {
                wall_distance_squared
            };
            if solid_distance_squared < distance_squared {
                distance_squared = solid_distance_squared;
            }
        }
        let clearance_squared = i128::from(MINIMUM_CLEARANCE_UM) * i128::from(MINIMUM_CLEARANCE_UM);
        if distance_squared < clearance_squared {
            return Err(WaterError::new(
                BOUNDARY_PENETRATION_LIMIT,
                format!("sample {} exceeds the 2500 um penetration limit", sample.id),
            ));
        }
        let distance_um = (distance_squared as f64).sqrt();
        let rounded_distance = crate::profile::quantize_scaled(distance_um, 1)?;
        let penetration = PARTICLE_RADIUS_UM
            .checked_sub(rounded_distance)
            .unwrap_or(0)
            .max(0);
        if penetration > maximum_penetration {
            maximum_penetration = penetration;
        }
    }
    Ok(maximum_penetration)
}

pub(crate) fn validate_transition(
    geometry: Geometry,
    prior: &[CanonicalSample],
    next: &[CanonicalSample],
) -> Result<(), WaterError> {
    let Some(aperture) = geometry.aperture else {
        return Ok(());
    };
    if prior.len() != next.len() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "boundary transition sample counts differ",
        ));
    }
    for (prior, next) in prior.iter().zip(next) {
        if prior.id != next.id {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "boundary transition SampleId order differs",
            ));
        }
        let x0 = i128::from(prior.position_um.x);
        let x1 = i128::from(next.position_um.x);
        let wall = i128::from(aperture.wall_x_um);
        let side0 = x0.checked_sub(wall).ok_or_else(|| {
            WaterError::new(NUMERIC_OVERFLOW, "prior wall side subtraction overflow")
        })?;
        let side1 = x1.checked_sub(wall).ok_or_else(|| {
            WaterError::new(NUMERIC_OVERFLOW, "next wall side subtraction overflow")
        })?;
        let crosses = (side0 < 0 && side1 >= 0) || (side0 > 0 && side1 <= 0);
        if !crosses {
            continue;
        }
        let mut denominator = x1.checked_sub(x0).ok_or_else(|| {
            WaterError::new(NUMERIC_OVERFLOW, "wall crossing denominator overflow")
        })?;
        let mut numerator = wall
            .checked_sub(x0)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "wall crossing numerator overflow"))?;
        if denominator < 0 {
            denominator = -denominator;
            numerator = -numerator;
        }
        let y_at_wall = interpolated_numerator(
            prior.position_um.y,
            next.position_um.y,
            numerator,
            denominator,
        )?;
        let z_at_wall = interpolated_numerator(
            prior.position_um.z,
            next.position_um.z,
            numerator,
            denominator,
        )?;
        let y_min = i128::from(aperture.y_min_um + MINIMUM_CLEARANCE_UM)
            .checked_mul(denominator)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "aperture y min overflow"))?;
        let y_max = i128::from(aperture.y_max_um - MINIMUM_CLEARANCE_UM)
            .checked_mul(denominator)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "aperture y max overflow"))?;
        let z_min = i128::from(aperture.z_min_um + MINIMUM_CLEARANCE_UM)
            .checked_mul(denominator)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "aperture z min overflow"))?;
        let z_max = i128::from(aperture.z_max_um - MINIMUM_CLEARANCE_UM)
            .checked_mul(denominator)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "aperture z max overflow"))?;
        if y_at_wall < y_min || y_at_wall > y_max || z_at_wall < z_min || z_at_wall > z_max {
            return Err(WaterError::new(
                BOUNDARY_PENETRATION_LIMIT,
                format!(
                    "sample {} crossed the internal wall outside the clearance-safe aperture",
                    prior.id
                ),
            ));
        }
    }
    Ok(())
}

fn interpolated_numerator(
    start: i64,
    end: i64,
    numerator: i128,
    denominator: i128,
) -> Result<i128, WaterError> {
    let start_term = i128::from(start)
        .checked_mul(denominator)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "crossing start term overflow"))?;
    let delta = i128::from(end)
        .checked_sub(i128::from(start))
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "crossing delta overflow"))?;
    let delta_term = delta
        .checked_mul(numerator)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "crossing delta term overflow"))?;
    start_term
        .checked_add(delta_term)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "crossing interpolation overflow"))
}

fn outer_lattice(geometry: Geometry) -> Result<Vec<Vec3i>, WaterError> {
    let xs = lattice_axis(geometry.bounds.min.x, geometry.bounds.max.x)?;
    let ys = lattice_axis(geometry.bounds.min.y, geometry.bounds.max.y)?;
    let zs = lattice_axis(geometry.bounds.min.z, geometry.bounds.max.z)?;
    let product = xs
        .len()
        .checked_mul(ys.len())
        .and_then(|value| value.checked_mul(zs.len()))
        .ok_or_else(|| {
            WaterError::new(
                BOUNDARY_CAPACITY_EXCEEDED,
                "boundary lattice product overflow",
            )
        })?;
    let mut positions = Vec::new();
    positions
        .try_reserve(product.min(MAXIMUM_BOUNDARY_SAMPLES))
        .map_err(|error| {
            WaterError::new(
                BOUNDARY_CAPACITY_EXCEEDED,
                format!("cannot reserve outer boundary candidates: {error}"),
            )
        })?;
    for x in xs {
        for y in ys.iter().copied() {
            for z in zs.iter().copied() {
                if x == geometry.bounds.min.x
                    || x == geometry.bounds.max.x
                    || y == geometry.bounds.min.y
                    || y == geometry.bounds.max.y
                    || z == geometry.bounds.min.z
                    || z == geometry.bounds.max.z
                {
                    push_position(&mut positions, Vec3i::new(x, y, z))?;
                }
            }
        }
    }
    Ok(positions)
}

fn lattice_axis(minimum: i64, maximum: i64) -> Result<Vec<i64>, WaterError> {
    let span = maximum
        .checked_sub(minimum)
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "boundary axis span overflow"))?;
    if span < 0 || span % LATTICE_SPACING_UM != 0 {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!("boundary axis {minimum}..{maximum} is not a 50000 um lattice"),
        ));
    }
    let count = usize::try_from(span / LATTICE_SPACING_UM + 1)
        .map_err(|_| WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "boundary axis count overflow"))?;
    let mut result = Vec::new();
    result.try_reserve_exact(count).map_err(|error| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            format!("cannot reserve boundary axis: {error}"),
        )
    })?;
    for index in 0..count {
        let index = i64::try_from(index).map_err(|_| {
            WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "boundary axis index overflow")
        })?;
        let value = LATTICE_SPACING_UM
            .checked_mul(index)
            .and_then(|offset| minimum.checked_add(offset))
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "boundary coordinate overflow"))?;
        result.push(value);
    }
    Ok(result)
}

fn cell_key(position: Vec3i) -> CellKey {
    CellKey {
        x: position.x.div_euclid(GRID_CELL_WIDTH_UM),
        y: position.y.div_euclid(GRID_CELL_WIDTH_UM),
        z: position.z.div_euclid(GRID_CELL_WIDTH_UM),
    }
}

fn admitted_neighbors(
    index: usize,
    position: Vec3i,
    positions: &[Vec3i],
    entries: &[GridEntry],
) -> Result<Vec<usize>, WaterError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW + 1)
        .map_err(|error| {
            WaterError::new(
                BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
                format!("cannot reserve boundary neighbor row: {error}"),
            )
        })?;
    visit_neighbor_cells(entries, cell_key(position), |entry| {
        if entry.index == index {
            return Ok(());
        }
        let displacement = position.checked_sub(positions[entry.index])?;
        if displacement.squared_length_i128()? <= support_radius_squared() {
            result.push(entry.index);
            validate_capacity(
                result.len(),
                MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
                BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
                "boundary row neighbors",
            )?;
        }
        Ok(())
    })?;
    result.sort_unstable();
    result.dedup();
    Ok(result)
}

fn visit_neighbor_cells(
    entries: &[GridEntry],
    centre: CellKey,
    mut visitor: impl FnMut(&GridEntry) -> Result<(), WaterError>,
) -> Result<(), WaterError> {
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let key = CellKey {
                    x: centre.x + dx,
                    y: centre.y + dy,
                    z: centre.z + dz,
                };
                let start = entries.partition_point(|entry| entry.key < key);
                let end = entries.partition_point(|entry| entry.key <= key);
                for entry in &entries[start..end] {
                    visitor(entry)?;
                }
            }
        }
    }
    Ok(())
}

fn push_position(positions: &mut Vec<Vec3i>, position: Vec3i) -> Result<(), WaterError> {
    let next_count = positions.len().checked_add(1).ok_or_else(|| {
        WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "boundary sample count overflow")
    })?;
    validate_capacity(
        next_count,
        MAXIMUM_BOUNDARY_SAMPLES,
        BOUNDARY_CAPACITY_EXCEEDED,
        "boundary samples",
    )?;
    positions.try_reserve(1).map_err(|error| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            format!("cannot grow boundary positions: {error}"),
        )
    })?;
    positions.push(position);
    Ok(())
}

fn support_radius_squared() -> i128 {
    i128::from(SUPPORT_RADIUS_UM) * i128::from(SUPPORT_RADIUS_UM)
}

fn outer_distance_squared(geometry: Geometry, position: Vec3i) -> Result<i128, WaterError> {
    let distances = [
        position.x.checked_sub(geometry.bounds.min.x),
        geometry.bounds.max.x.checked_sub(position.x),
        position.y.checked_sub(geometry.bounds.min.y),
        geometry.bounds.max.y.checked_sub(position.y),
        position.z.checked_sub(geometry.bounds.min.z),
        geometry.bounds.max.z.checked_sub(position.z),
    ];
    let mut minimum = i64::MAX;
    for distance in distances {
        let distance =
            distance.ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "outer distance overflow"))?;
        if distance < minimum {
            minimum = distance;
        }
    }
    let minimum = i128::from(minimum);
    minimum
        .checked_mul(minimum)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "outer distance square overflow"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Aperture, Box3i};
    use crate::scenario;

    #[test]
    fn frozen_product_and_orifice_boundary_counts_match() {
        let sealed = scenario::find("CW-SEALED-001").unwrap();
        assert_eq!(build(sealed.geometry).unwrap().len(), 11_202);
        let orifice = scenario::find("CW-ORIFICE-001").unwrap();
        assert_eq!(build(orifice.geometry).unwrap().len(), 4_338);
    }

    #[test]
    fn analytical_outer_boundary_uses_exact_clearance_branch() {
        let geometry = Geometry {
            bounds: Box3i {
                min: Vec3i::new(0, 0, 0),
                max: Vec3i::new(100_000, 100_000, 100_000),
            },
            aperture: None,
        };
        let sample = |id, x| CanonicalSample {
            id,
            position_um: Vec3i::new(x, 50_000, 50_000),
            velocity_um_s: Vec3i::new(0, 0, 0),
        };
        assert_eq!(
            validate_centres(geometry, &[sample(0, 22_500)]).unwrap(),
            2_500
        );
        assert_eq!(
            validate_centres(geometry, &[sample(0, 22_499)])
                .unwrap_err()
                .code(),
            BOUNDARY_PENETRATION_LIMIT
        );
        assert_eq!(
            validate_centres(geometry, &[sample(0, -1)])
                .unwrap_err()
                .code(),
            BOUNDARY_ESCAPE
        );
    }

    #[test]
    fn closed_aperture_projection_measures_nearest_solid_edge() {
        let geometry = Geometry {
            bounds: Box3i {
                min: Vec3i::new(0, 0, 0),
                max: Vec3i::new(200_000, 200_000, 200_000),
            },
            aperture: Some(Aperture {
                wall_x_um: 100_000,
                y_min_um: 50_000,
                y_max_um: 150_000,
                z_min_um: 50_000,
                z_max_um: 150_000,
            }),
        };
        let through_centre = CanonicalSample {
            id: 0,
            position_um: Vec3i::new(100_000, 100_000, 100_000),
            velocity_um_s: Vec3i::new(0, 0, 0),
        };
        assert!(validate_centres(geometry, &[through_centre]).is_ok());
        let at_edge = CanonicalSample {
            id: 0,
            position_um: Vec3i::new(100_000, 50_000, 100_000),
            velocity_um_s: Vec3i::new(0, 0, 0),
        };
        assert_eq!(
            validate_centres(geometry, &[at_edge]).unwrap_err().code(),
            BOUNDARY_PENETRATION_LIMIT
        );

        let sample = |id, x, y| CanonicalSample {
            id,
            position_um: Vec3i::new(x, y, 100_000),
            velocity_um_s: Vec3i::new(0, 0, 0),
        };
        assert!(
            validate_transition(
                geometry,
                &[sample(0, 70_000, 100_000)],
                &[sample(0, 130_000, 100_000)]
            )
            .is_ok()
        );
        assert_eq!(
            validate_transition(
                geometry,
                &[sample(0, 70_000, 60_000)],
                &[sample(0, 130_000, 60_000)]
            )
            .unwrap_err()
            .code(),
            BOUNDARY_PENETRATION_LIMIT
        );
    }

    #[test]
    fn boundary_capacities_accept_n_minus_one_and_n() {
        for value in [MAXIMUM_BOUNDARY_SAMPLES - 1, MAXIMUM_BOUNDARY_SAMPLES] {
            assert!(
                validate_capacity(
                    value,
                    MAXIMUM_BOUNDARY_SAMPLES,
                    BOUNDARY_CAPACITY_EXCEEDED,
                    "boundary"
                )
                .is_ok()
            );
        }
        assert_eq!(
            validate_capacity(
                MAXIMUM_BOUNDARY_SAMPLES + 1,
                MAXIMUM_BOUNDARY_SAMPLES,
                BOUNDARY_CAPACITY_EXCEEDED,
                "boundary"
            )
            .unwrap_err()
            .code(),
            BOUNDARY_CAPACITY_EXCEEDED
        );
        for value in [
            MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW - 1,
            MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
        ] {
            assert!(
                validate_capacity(
                    value,
                    MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
                    BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
                    "row"
                )
                .is_ok()
            );
        }
        assert_eq!(
            validate_capacity(
                MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW + 1,
                MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
                BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
                "row"
            )
            .unwrap_err()
            .code(),
            BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED
        );
    }
}
