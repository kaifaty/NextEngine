# Water lattice tier — regular cells with open sills (SPEC-38 practice 1)

| Field | Value |
| --- | --- |
| Research ID | `WL-LATTICE` |
| Status | `FROZEN / NOT_RUN` |
| Parent | plan `continuum-water/11` section 3 (scale practices, ADR-103 order); SPEC-38 2.2 practice 1 ("lattice cells are the large-body tier"); ADR-103 (exact Jacobi step over `Open` sills); plan 07 revision 2 (`step_in_place`) |
| Purpose | a map-wide water body as a regular lattice of `WaterVolumeDefinitionV1` cells joined by `Open` sill edges to their four neighbours, authored per region with a declared cell size and count, built by one exact helper, stepped by the existing law with no new profile; the first increment stays inside the existing bounds (`64` cells, `256` edges per world) and lives in a verification fixture, not in the reference scene |

## Frozen scope

- **Face-sharing rule (contracts, SPEC-38 2.3 clause).** Two volumes
  overlap when their intersection has positive measure on every axis
  (`min < other.max && other.min < max`); volumes that share a face,
  an edge or a corner are disjoint. Submersion of a point on a shared
  face resolves to the first containing volume in id order (the existing
  iteration of `submersion_at`), so it is deterministic. The reference
  scene's volumes are unchanged by the rule.
- **Lattice authoring (contracts, `water_lattice.rs`).**
  `WaterLatticeRegionV1 { region_id: PersistentId, origin_micrometres:
  [i64; 3], cell_size_micrometres: [i64; 2] (x, z), columns: u32, rows:
  u32, ceiling_micrometres: i64, floor_micrometres: Vec<i64> (row-major,
  one per cell), initial_level_micrometres: Vec<i64> (one per cell, at
  or above the floor), sill_coefficient_permille: u32, profile_revision:
  u64 }` with `validate()` (bounds: `columns * rows <= 64`, edges
  `columns * (rows - 1) + rows * (columns - 1) <= 256`, positive sizes,
  floors below the ceiling, levels inside the extent) and
  `build(&self, ticks_per_second) -> (WaterVolumeSetV1,
  WaterFlowNetworkV1)`: cell `(column, row)` gets the volume id derived
  from the region id, column and row (`PersistentId::derive`-style
  domain hash `nextengine.water-lattice.cell.v1`), extent `[origin.x +
  column * size.x, floor, origin.z + row * size.z]` to `[.. + size.x,
  ceiling, .. + size.z]`, swimming depth `ceiling - floor`; every
  neighbour pair gets one `Open` edge (id from the two cell ids, domain
  `nextengine.water-lattice.edge.v1`) with `sill = max(floor_a,
  floor_b)`, `width = the shared side length in millimetres`, the
  region's coefficient. `WaterVolumeSetV1::from_definitions` and
  `WaterFlowNetworkV1::from_edges` validate the result.
- **Check `CONTINUUM-WATER-LATTICE-P1` (`xtask water-lattice`).** An
  `8 x 8` region of `1 m` cells over a terrain that falls `0.1 m` per
  column from west to east (`floor = 0.7 m - 0.1 m * column`), ceiling
  `3 m`, every cell dry except the west column holding water to
  `2.0 m`; `30 Hz`; `sill_coefficient 600`. The check steps
  `step_in_place` for `1,800` ticks (one minute) and records after every
  tick.
- **Not in scope.** Terrain streaming, region authoring in the project
  JSON, activity stepping (plan 20), presentation of lattice cells.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 build | the region validates and builds `64` cells and `112` edges; every cell pair is disjoint under the new rule and the same set is rejected by the old closed-interval rule (the unit test keeps the old predicate) | pass |
| G2 conservation | the total stored volume after every tick equals the initial total exactly (no source, no sink) | exact |
| G3 downhill | by tick `1,800` the water has reached the east column (its level above its floor) and every open sill between wet neighbours has a level difference `<= 5 mm` (settled communicating cells); no cell level exceeds the ceiling or falls below its floor | pass |
| G4 determinism | a repeated run reproduces every tick's network and table byte-for-byte (`canonical_record` hashes); the run through `step` (cloning) equals the run through `step_in_place` | pass |
| G5 cost | `step_in_place` over the `64 x 112` lattice on the reference host in release | `<= 50 us` max, mean reported |
| G6 no authority change | the reference scene's roots do not move: `play`, `persistence-replay`, `water-volume`, `water-flow`, `water-buoyancy`, `content-package`, `host-check` PASS | pass |

Constants (cell size, slope, coefficient, tick count) and the gates are
frozen; a change after the run is a new revision with its own evidence.
