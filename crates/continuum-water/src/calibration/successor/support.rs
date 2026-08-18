use crate::boundary::{BoundarySample, BoundarySupport};
use crate::error::{AUDIT_INVALID, BOUNDARY_CAPACITY_EXCEEDED, WaterError};
use crate::geometry::{
    AxisAlignedGeometryManifest, INTERNAL_PATCH_FEATURE_ID, OUTER_X_MAX_FEATURE_ID,
    OUTER_X_MIN_FEATURE_ID, OUTER_Y_MAX_FEATURE_ID, OUTER_Y_MIN_FEATURE_ID, OUTER_Z_MAX_FEATURE_ID,
    OUTER_Z_MIN_FEATURE_ID,
};
use crate::model::{Geometry, Vec3i};
use crate::profile::{
    LATTICE_SPACING_UM, PARTICLE_RADIUS_UM, REST_VOLUME, SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
};
use crate::scenario::validate_capacity;

const EXTERIOR_LAYERS: i64 = 2;

pub(crate) fn build_density_support(geometry: Geometry) -> Result<Vec<BoundarySample>, WaterError> {
    let manifest = AxisAlignedGeometryManifest::from_geometry(geometry)?;
    let counts = [
        cell_count(geometry.bounds.min.x, geometry.bounds.max.x)?,
        cell_count(geometry.bounds.min.y, geometry.bounds.max.y)?,
        cell_count(geometry.bounds.min.z, geometry.bounds.max.z)?,
    ];
    let outer_count = complement_count(counts)?;
    let internal_count = if let Some(patch) = manifest.internal_patch() {
        let y_count = counts[1];
        let z_count = counts[2];
        let opening_y_count = opening_cell_count(
            geometry.bounds.min.y,
            y_count,
            patch.opening.y_min_um,
            patch.opening.y_max_um,
        )?;
        let opening_z_count = opening_cell_count(
            geometry.bounds.min.z,
            z_count,
            patch.opening.z_min_um,
            patch.opening.z_max_um,
        )?;
        let plane_cells = y_count.checked_mul(z_count).ok_or_else(|| {
            WaterError::new(
                BOUNDARY_CAPACITY_EXCEEDED,
                "internal plane cell count overflow",
            )
        })?;
        let opening_cells = opening_y_count
            .checked_mul(opening_z_count)
            .ok_or_else(|| {
                WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "opening cell count overflow")
            })?;
        usize::try_from(
            plane_cells
                .checked_sub(opening_cells)
                .and_then(|value| value.checked_mul(EXTERIOR_LAYERS * 2))
                .ok_or_else(|| {
                    WaterError::new(
                        BOUNDARY_CAPACITY_EXCEEDED,
                        "internal support count overflow",
                    )
                })?,
        )
        .map_err(|_| {
            WaterError::new(
                BOUNDARY_CAPACITY_EXCEEDED,
                "internal support count conversion overflow",
            )
        })?
    } else {
        0
    };
    let expected = outer_count.checked_add(internal_count).ok_or_else(|| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            "successor support count overflow",
        )
    })?;
    validate_capacity(
        expected,
        SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
        BOUNDARY_CAPACITY_EXCEEDED,
        "successor static boundary samples",
    )?;

    let mut records = Vec::new();
    records.try_reserve_exact(expected).map_err(|error| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            format!("cannot reserve successor density support: {error}"),
        )
    })?;
    append_outer_complement(&mut records, geometry, counts)?;
    if let Some(patch) = manifest.internal_patch() {
        for layer in 0..EXTERIOR_LAYERS {
            let offset = PARTICLE_RADIUS_UM
                .checked_add(layer.checked_mul(LATTICE_SPACING_UM).ok_or_else(|| {
                    WaterError::new(AUDIT_INVALID, "internal support layer overflow")
                })?)
                .ok_or_else(|| {
                    WaterError::new(AUDIT_INVALID, "internal support offset overflow")
                })?;
            for iy in 0..counts[1] {
                let y = ghost_coordinate(geometry.bounds.min.y, iy)?;
                for iz in 0..counts[2] {
                    let z = ghost_coordinate(geometry.bounds.min.z, iz)?;
                    if patch.opening.contains_closed(y, z) {
                        continue;
                    }
                    push_unidentified(
                        &mut records,
                        Vec3i::new(
                            patch.coordinate_um.checked_add(offset).ok_or_else(|| {
                                WaterError::new(
                                    AUDIT_INVALID,
                                    "positive internal support coordinate overflow",
                                )
                            })?,
                            y,
                            z,
                        ),
                        INTERNAL_PATCH_FEATURE_ID,
                        BoundarySupport::FluidXLessThan(patch.coordinate_um),
                    )?;
                    push_unidentified(
                        &mut records,
                        Vec3i::new(
                            patch.coordinate_um.checked_sub(offset).ok_or_else(|| {
                                WaterError::new(
                                    AUDIT_INVALID,
                                    "negative internal support coordinate overflow",
                                )
                            })?,
                            y,
                            z,
                        ),
                        INTERNAL_PATCH_FEATURE_ID,
                        BoundarySupport::FluidXGreaterThan(patch.coordinate_um),
                    )?;
                }
            }
        }
    }
    records.sort_unstable_by_key(|sample| (sample.position_um, sample.support, sample.feature_id));
    records.dedup_by_key(|sample| (sample.position_um, sample.support, sample.feature_id));
    if records.len() != expected {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!(
                "successor support generated {} unique records, expected {expected}",
                records.len()
            ),
        ));
    }
    for (index, sample) in records.iter_mut().enumerate() {
        sample.id = u32::try_from(index).map_err(|_| {
            WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "successor support ID overflow")
        })?;
    }
    Ok(records)
}

fn append_outer_complement(
    output: &mut Vec<BoundarySample>,
    geometry: Geometry,
    counts: [i64; 3],
) -> Result<(), WaterError> {
    let maximum = counts.map(|count| {
        count
            .checked_add(EXTERIOR_LAYERS - 1)
            .ok_or_else(|| WaterError::new(AUDIT_INVALID, "outer support index overflow"))
    });
    let [maximum_x, maximum_y, maximum_z] = maximum;
    let maximum_x = maximum_x?;
    let maximum_y = maximum_y?;
    let maximum_z = maximum_z?;
    for ix in -EXTERIOR_LAYERS..=maximum_x {
        for iy in -EXTERIOR_LAYERS..=maximum_y {
            for iz in -EXTERIOR_LAYERS..=maximum_z {
                let feature_id = if ix < 0 {
                    Some(OUTER_X_MIN_FEATURE_ID)
                } else if ix >= counts[0] {
                    Some(OUTER_X_MAX_FEATURE_ID)
                } else if iy < 0 {
                    Some(OUTER_Y_MIN_FEATURE_ID)
                } else if iy >= counts[1] {
                    Some(OUTER_Y_MAX_FEATURE_ID)
                } else if iz < 0 {
                    Some(OUTER_Z_MIN_FEATURE_ID)
                } else if iz >= counts[2] {
                    Some(OUTER_Z_MAX_FEATURE_ID)
                } else {
                    None
                };
                if let Some(feature_id) = feature_id {
                    push_unidentified(
                        output,
                        Vec3i::new(
                            ghost_coordinate(geometry.bounds.min.x, ix)?,
                            ghost_coordinate(geometry.bounds.min.y, iy)?,
                            ghost_coordinate(geometry.bounds.min.z, iz)?,
                        ),
                        feature_id,
                        BoundarySupport::Unrestricted,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn push_unidentified(
    output: &mut Vec<BoundarySample>,
    position_um: Vec3i,
    feature_id: u32,
    support: BoundarySupport,
) -> Result<(), WaterError> {
    let next = output.len().checked_add(1).ok_or_else(|| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            "successor support growth overflow",
        )
    })?;
    validate_capacity(
        next,
        SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
        BOUNDARY_CAPACITY_EXCEEDED,
        "successor static boundary samples",
    )?;
    output.push(BoundarySample {
        id: 0,
        position_um,
        volume: REST_VOLUME,
        feature_id,
        support,
    });
    Ok(())
}

fn complement_count(counts: [i64; 3]) -> Result<usize, WaterError> {
    let expanded = counts.iter().try_fold(1_i64, |product, count| {
        product.checked_mul(count.checked_add(EXTERIOR_LAYERS * 2)?)
    });
    let interior = counts
        .iter()
        .try_fold(1_i64, |product, count| product.checked_mul(*count));
    usize::try_from(
        expanded
            .and_then(|expanded| interior.and_then(|interior| expanded.checked_sub(interior)))
            .ok_or_else(|| {
                WaterError::new(
                    BOUNDARY_CAPACITY_EXCEEDED,
                    "outer complement count overflow",
                )
            })?,
    )
    .map_err(|_| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            "outer complement count conversion overflow",
        )
    })
}

fn cell_count(minimum: i64, maximum: i64) -> Result<i64, WaterError> {
    let span = maximum
        .checked_sub(minimum)
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "successor support axis overflow"))?;
    if span <= 0 || span % LATTICE_SPACING_UM != 0 {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!("successor support axis {minimum}..{maximum} is not a positive lattice span"),
        ));
    }
    Ok(span / LATTICE_SPACING_UM)
}

fn opening_cell_count(
    minimum: i64,
    count: i64,
    opening_minimum: i64,
    opening_maximum: i64,
) -> Result<i64, WaterError> {
    let mut result = 0_i64;
    for index in 0..count {
        let coordinate = ghost_coordinate(minimum, index)?;
        if coordinate >= opening_minimum && coordinate <= opening_maximum {
            result = result
                .checked_add(1)
                .ok_or_else(|| WaterError::new(AUDIT_INVALID, "opening cell count overflow"))?;
        }
    }
    Ok(result)
}

fn ghost_coordinate(minimum: i64, index: i64) -> Result<i64, WaterError> {
    LATTICE_SPACING_UM
        .checked_mul(index)
        .and_then(|offset| PARTICLE_RADIUS_UM.checked_add(offset))
        .and_then(|offset| minimum.checked_add(offset))
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "successor support coordinate overflow"))
}
