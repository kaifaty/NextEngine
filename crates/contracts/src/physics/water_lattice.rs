//! Water lattice tier (SPEC-38 practice 1, plan `continuum-water/19`): a
//! map-wide water body as a regular lattice of `WaterVolumeDefinitionV1`
//! cells joined by `Open` sill edges to their four neighbours. The region
//! is authored once (cell size and count, per-cell floors and initial
//! levels); `build` derives the cells and edges deterministically and the
//! existing exact table and flow network carry them from then on. No new
//! profile, no new record kind.

use super::error::PhysicsContractError;
use super::water::{WaterVolumeDefinitionV1, WaterVolumeSetV1};
use super::water_flow::{WaterFlowEdgeKindV1, WaterFlowEdgeV1, WaterFlowNetworkV1};
use crate::ids::PersistentId;
use crate::project::domain_hash;

/// Domain of a lattice cell's volume id.
pub const WATER_LATTICE_CELL_ID_DOMAIN: &str = "nextengine.water-lattice.cell.v1";
/// Domain of a lattice sill edge id.
pub const WATER_LATTICE_EDGE_ID_DOMAIN: &str = "nextengine.water-lattice.edge.v1";

/// One authored lattice region: `columns x rows` cells of one plan size
/// over a per-cell floor, one ceiling, one initial level per cell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterLatticeRegionV1 {
    pub region_id: PersistentId,
    /// The minimum corner of cell `(0, 0)` in `x` and `z`; `y` is unused.
    pub origin_micrometres: [i64; 3],
    /// Cell plan size in `x` and `z`.
    pub cell_size_micrometres: [i64; 2],
    pub columns: u32,
    pub rows: u32,
    pub ceiling_micrometres: i64,
    /// Row-major (`row * columns + column`), one floor per cell.
    pub floor_micrometres: Vec<i64>,
    /// Row-major, one initial level per cell, inside `[floor, ceiling]`.
    pub initial_level_micrometres: Vec<i64>,
    /// The weir coefficient of every sill edge.
    pub sill_coefficient_permille: u32,
    pub profile_revision: u32,
}

impl WaterLatticeRegionV1 {
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.columns as usize * self.rows as usize
    }

    /// Sill edges between horizontal and vertical neighbours.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let columns = self.columns as usize;
        let rows = self.rows as usize;
        columns * rows.saturating_sub(1) + rows * columns.saturating_sub(1)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.columns == 0
            || self.rows == 0
            || self.cell_count() > super::water::MAX_WATER_VOLUMES
            || self.edge_count() > super::water_flow::MAX_WATER_FLOW_EDGES
            || self.cell_size_micrometres.iter().any(|size| *size <= 0)
            || self.floor_micrometres.len() != self.cell_count()
            || self.initial_level_micrometres.len() != self.cell_count()
            || self.sill_coefficient_permille == 0
            || self.sill_coefficient_permille > 1000
            || self.profile_revision == 0
        {
            return Err(PhysicsContractError::WaterVolumeInvalid);
        }
        for (floor, level) in self
            .floor_micrometres
            .iter()
            .zip(&self.initial_level_micrometres)
        {
            if *floor >= self.ceiling_micrometres
                || *level < *floor
                || *level > self.ceiling_micrometres
            {
                return Err(PhysicsContractError::WaterVolumeInvalid);
            }
        }
        Ok(())
    }

    /// The volume id of cell `(column, row)`.
    #[must_use]
    pub fn cell_id(&self, column: u32, row: u32) -> PersistentId {
        let mut preimage = Vec::with_capacity(24);
        preimage.extend_from_slice(self.region_id.as_bytes());
        preimage.extend_from_slice(&column.to_le_bytes());
        preimage.extend_from_slice(&row.to_le_bytes());
        persistent_from_hash(WATER_LATTICE_CELL_ID_DOMAIN, &preimage)
    }

    /// The edge id between two cells (ordered by their ids).
    #[must_use]
    pub fn edge_id(&self, cell_a: PersistentId, cell_b: PersistentId) -> PersistentId {
        let (first, second) = if cell_a <= cell_b {
            (cell_a, cell_b)
        } else {
            (cell_b, cell_a)
        };
        let mut preimage = Vec::with_capacity(48);
        preimage.extend_from_slice(self.region_id.as_bytes());
        preimage.extend_from_slice(first.as_bytes());
        preimage.extend_from_slice(second.as_bytes());
        persistent_from_hash(WATER_LATTICE_EDGE_ID_DOMAIN, &preimage)
    }

    /// The cell definitions in row-major order.
    pub fn definitions(&self) -> Result<Vec<WaterVolumeDefinitionV1>, PhysicsContractError> {
        self.validate()?;
        let mut definitions = Vec::with_capacity(self.cell_count());
        for row in 0..self.rows {
            for column in 0..self.columns {
                let index = row as usize * self.columns as usize + column as usize;
                let floor = self.floor_micrometres[index];
                let x0 = self.origin_micrometres[0]
                    .checked_add(i64::from(column) * self.cell_size_micrometres[0])
                    .ok_or(PhysicsContractError::WaterVolumeInvalid)?;
                let z0 = self.origin_micrometres[2]
                    .checked_add(i64::from(row) * self.cell_size_micrometres[1])
                    .ok_or(PhysicsContractError::WaterVolumeInvalid)?;
                definitions.push(WaterVolumeDefinitionV1 {
                    volume_id: self.cell_id(column, row),
                    minimum_micrometres: [x0, floor, z0],
                    maximum_micrometres: [
                        x0 + self.cell_size_micrometres[0],
                        self.ceiling_micrometres,
                        z0 + self.cell_size_micrometres[1],
                    ],
                    initial_level_micrometres: self.initial_level_micrometres[index],
                    swimming_depth_micrometres: self.ceiling_micrometres - floor,
                    level_ramp: None,
                    profile_revision: self.profile_revision,
                });
            }
        }
        Ok(definitions)
    }

    /// The sill edges: `x` neighbours (shared side `cell_size.z`) then `z`
    /// neighbours (shared side `cell_size.x`), sill at the higher floor.
    pub fn edges(&self) -> Result<Vec<WaterFlowEdgeV1>, PhysicsContractError> {
        self.validate()?;
        let mut edges = Vec::with_capacity(self.edge_count());
        let floor = |column: u32, row: u32| {
            self.floor_micrometres[row as usize * self.columns as usize + column as usize]
        };
        let mut push = |a: (u32, u32), b: (u32, u32), width_micrometres: i64| {
            let cell_a = self.cell_id(a.0, a.1);
            let cell_b = self.cell_id(b.0, b.1);
            edges.push(WaterFlowEdgeV1 {
                edge_id: self.edge_id(cell_a, cell_b),
                cell_a,
                cell_b: Some(cell_b),
                kind: WaterFlowEdgeKindV1::Open {
                    sill_micrometres: floor(a.0, a.1).max(floor(b.0, b.1)),
                    width_millimetres: (width_micrometres / 1000).max(1),
                    coefficient_permille: self.sill_coefficient_permille,
                },
            });
        };
        for row in 0..self.rows {
            for column in 0..self.columns.saturating_sub(1) {
                push(
                    (column, row),
                    (column + 1, row),
                    self.cell_size_micrometres[1],
                );
            }
        }
        for row in 0..self.rows.saturating_sub(1) {
            for column in 0..self.columns {
                push(
                    (column, row),
                    (column, row + 1),
                    self.cell_size_micrometres[0],
                );
            }
        }
        Ok(edges)
    }

    /// The validated table and network of the region.
    pub fn build(
        &self,
        ticks_per_second: u32,
    ) -> Result<(WaterVolumeSetV1, WaterFlowNetworkV1), PhysicsContractError> {
        let volumes = WaterVolumeSetV1::from_definitions(self.definitions()?)?;
        let network = WaterFlowNetworkV1::from_edges(ticks_per_second, self.edges()?, &volumes)?;
        Ok((volumes, network))
    }
}

fn persistent_from_hash(domain: &str, preimage: &[u8]) -> PersistentId {
    let hash = domain_hash(domain, preimage);
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);
    PersistentId::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(columns: u32, rows: u32) -> WaterLatticeRegionV1 {
        let count = (columns * rows) as usize;
        WaterLatticeRegionV1 {
            region_id: PersistentId::from_bytes([0x4c; 16]),
            origin_micrometres: [0, 0, 0],
            cell_size_micrometres: [1_000_000, 1_000_000],
            columns,
            rows,
            ceiling_micrometres: 3_000_000,
            floor_micrometres: (0..count)
                .map(|index| 700_000 - 100_000 * (index % columns as usize) as i64)
                .collect(),
            initial_level_micrometres: (0..count)
                .map(|index| {
                    if index % columns as usize == 0 {
                        2_000_000
                    } else {
                        700_000 - 100_000 * (index % columns as usize) as i64
                    }
                })
                .collect(),
            sill_coefficient_permille: 600,
            profile_revision: 1,
        }
    }

    #[test]
    fn eight_by_eight_region_builds_inside_the_bounds_with_face_sharing_cells() {
        let region = region(8, 8);
        assert_eq!(region.cell_count(), 64);
        assert_eq!(region.edge_count(), 112);
        let (volumes, network) = region.build(30).expect("build");
        assert_eq!(volumes.definitions.len(), 64);
        assert_eq!(network.edges.len(), 112);
        // Cells share faces: the closed-interval predicate of the previous
        // rule would call every neighbour pair overlapping.
        let definitions: Vec<_> = volumes.definitions.values().collect();
        let closed_overlap = |a: &WaterVolumeDefinitionV1, b: &WaterVolumeDefinitionV1| {
            (0..3).all(|axis| {
                a.minimum_micrometres[axis] <= b.maximum_micrometres[axis]
                    && b.minimum_micrometres[axis] <= a.maximum_micrometres[axis]
            })
        };
        let touching = definitions
            .iter()
            .enumerate()
            .flat_map(|(index, a)| definitions[..index].iter().map(move |b| (a, b)))
            .filter(|(a, b)| closed_overlap(a, b))
            .count();
        assert!(touching >= 112, "neighbours touch: {touching}");
        // Every sill sits at the higher floor and spans one cell side.
        for edge in network.edges.values() {
            let WaterFlowEdgeKindV1::Open {
                sill_micrometres,
                width_millimetres,
                coefficient_permille,
            } = edge.kind
            else {
                panic!("lattice edges are open sills");
            };
            let a = &volumes.definitions[&edge.cell_a];
            let b = &volumes.definitions[&edge.cell_b.expect("two cells")];
            assert_eq!(
                sill_micrometres,
                a.minimum_micrometres[1].max(b.minimum_micrometres[1])
            );
            assert_eq!(width_millimetres, 1_000);
            assert_eq!(coefficient_permille, 600);
        }
        // Ids are a pure function of the region, column and row.
        let same = WaterLatticeRegionV1 {
            profile_revision: 2,
            ..region.clone()
        };
        assert_eq!(region.cell_id(3, 5), same.cell_id(3, 5));
        assert_ne!(region.cell_id(3, 5), region.cell_id(5, 3));
    }

    #[test]
    fn face_sharing_volumes_are_disjoint_and_positive_overlap_is_rejected() {
        let cell = |x0: i64, id: u8| WaterVolumeDefinitionV1 {
            volume_id: PersistentId::from_bytes([id; 16]),
            minimum_micrometres: [x0, 0, 0],
            maximum_micrometres: [x0 + 1_000_000, 1_000_000, 1_000_000],
            initial_level_micrometres: 500_000,
            swimming_depth_micrometres: 1_000_000,
            level_ramp: None,
            profile_revision: 1,
        };
        assert!(WaterVolumeSetV1::from_definitions([cell(0, 1), cell(1_000_000, 2)]).is_ok());
        assert_eq!(
            WaterVolumeSetV1::from_definitions([cell(0, 1), cell(999_999, 2)])
                .err()
                .map(|error| error.to_string()),
            Some(PhysicsContractError::WaterVolumeInvalid.to_string())
        );
    }

    #[test]
    fn region_bounds_and_levels_are_enforced() {
        let mut too_large = region(9, 8);
        too_large.floor_micrometres = vec![0; 72];
        too_large.initial_level_micrometres = vec![0; 72];
        assert_eq!(
            too_large.validate(),
            Err(PhysicsContractError::WaterVolumeInvalid)
        );
        let mut level_below_floor = region(2, 2);
        level_below_floor.initial_level_micrometres[1] = -1;
        assert_eq!(
            level_below_floor.validate(),
            Err(PhysicsContractError::WaterVolumeInvalid)
        );
        let mut short = region(2, 2);
        short.floor_micrometres.pop();
        assert_eq!(
            short.validate(),
            Err(PhysicsContractError::WaterVolumeInvalid)
        );
        assert_eq!(region(2, 2).validate(), Ok(()));
    }
}
