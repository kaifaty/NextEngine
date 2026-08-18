#![forbid(unsafe_code)]

use crate::boundary::BoundarySample;
use crate::error::{NEIGHBOR_CAPACITY_EXCEEDED, WaterError};
use crate::geometry::AxisAlignedGeometryManifest;
use crate::model::{CanonicalSample, Vec3i};
use crate::profile::{GRID_CELL_WIDTH_UM, MAXIMUM_NEIGHBORS_PER_FLUID_ROW, SUPPORT_RADIUS_UM};
use crate::scenario::validate_capacity;

use super::heap_error;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CellKey {
    x: i64,
    y: i64,
    z: i64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct GridEntry {
    key: CellKey,
    id: u32,
    index: usize,
}

pub(super) fn build_fluid_grid(samples: &[CanonicalSample]) -> Result<Vec<GridEntry>, WaterError> {
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(samples.len())
        .map_err(heap_error)?;
    for (index, sample) in samples.iter().enumerate() {
        entries.push(GridEntry {
            key: cell_key(sample.position_um),
            id: sample.id,
            index,
        });
    }
    entries.sort_unstable();
    Ok(entries)
}

pub(super) fn build_boundary_grid(
    boundary: &[BoundarySample],
) -> Result<Vec<GridEntry>, WaterError> {
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(boundary.len())
        .map_err(heap_error)?;
    for (index, sample) in boundary.iter().enumerate() {
        entries.push(GridEntry {
            key: cell_key(sample.position_um),
            id: sample.id,
            index,
        });
    }
    entries.sort_unstable();
    Ok(entries)
}

pub(super) fn admitted_fluid(
    sample: &CanonicalSample,
    samples: &[CanonicalSample],
    entries: &[GridEntry],
    geometry: &AxisAlignedGeometryManifest,
) -> Result<Vec<usize>, WaterError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(MAXIMUM_NEIGHBORS_PER_FLUID_ROW + 1)
        .map_err(heap_error)?;
    visit_neighbor_cells(entries, cell_key(sample.position_um), |entry| {
        if entry.id == sample.id {
            return Ok(());
        }
        let displacement = sample
            .position_um
            .checked_sub(samples[entry.index].position_um)?;
        if displacement.squared_length_i128()? <= support_radius_squared()
            && !geometry.blocks_segment(sample.position_um, samples[entry.index].position_um)?
        {
            result.push(entry.index);
            validate_capacity(
                result.len(),
                MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
                NEIGHBOR_CAPACITY_EXCEEDED,
                "fluid row neighbors",
            )?;
        }
        Ok(())
    })?;
    result.sort_unstable_by_key(|index| samples[*index].id);
    result.dedup();
    Ok(result)
}

pub(super) fn admitted_boundary(
    sample: &CanonicalSample,
    boundary: &[BoundarySample],
    entries: &[GridEntry],
) -> Result<Vec<usize>, WaterError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(MAXIMUM_NEIGHBORS_PER_FLUID_ROW + 1)
        .map_err(heap_error)?;
    visit_neighbor_cells(entries, cell_key(sample.position_um), |entry| {
        let displacement = sample
            .position_um
            .checked_sub(boundary[entry.index].position_um)?;
        if boundary[entry.index].support.admits(sample.position_um)
            && displacement.squared_length_i128()? <= support_radius_squared()
        {
            result.push(entry.index);
            validate_capacity(
                result.len(),
                MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
                NEIGHBOR_CAPACITY_EXCEEDED,
                "fluid boundary row neighbors",
            )?;
        }
        Ok(())
    })?;
    result.sort_unstable_by_key(|index| boundary[*index].id);
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

fn cell_key(position: Vec3i) -> CellKey {
    CellKey {
        x: position.x.div_euclid(GRID_CELL_WIDTH_UM),
        y: position.y.div_euclid(GRID_CELL_WIDTH_UM),
        z: position.z.div_euclid(GRID_CELL_WIDTH_UM),
    }
}

fn support_radius_squared() -> i128 {
    i128::from(SUPPORT_RADIUS_UM) * i128::from(SUPPORT_RADIUS_UM)
}
