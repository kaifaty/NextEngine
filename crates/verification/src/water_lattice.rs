//! `CONTINUUM-WATER-LATTICE-P1` (SPEC-38 practice 1, plan
//! `continuum-water/19`): an authored lattice region of face-sharing cells
//! joined by open sills conserves volume exactly, runs downhill to a common
//! level, steps deterministically in place and inside the flow step budget.
//! A synthetic fixture: the reference scene is untouched.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Instant;

use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::physics::{
    WaterFlowEdgeKindV1, WaterFlowNetworkV1, WaterLatticeRegionV1, WaterVolumeSetV1,
};

/// Plan 19 constants (frozen).
const COLUMNS: u32 = 8;
const ROWS: u32 = 8;
const CELL_SIZE_MICROMETRES: i64 = 1_000_000;
const CEILING_MICROMETRES: i64 = 3_000_000;
const WEST_FLOOR_MICROMETRES: i64 = 700_000;
const FLOOR_STEP_MICROMETRES: i64 = 100_000;
const WEST_LEVEL_MICROMETRES: i64 = 2_000_000;
const TICKS_PER_SECOND: u32 = 30;
const SILL_COEFFICIENT_PERMILLE: u32 = 600;
const RUN_TICKS: u64 = 1_800;
/// G3: settled communicating cells.
const SETTLE_TOLERANCE_MICROMETRES: i64 = 5_000;
/// A cell counts as wet above this depth over its floor.
const WET_DEPTH_MICROMETRES: i64 = 1_000;
const COST_STEPS: u32 = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterLatticeCheckReportV1 {
    pub region_id: PersistentId,
    pub cells: usize,
    pub edges: usize,
    pub run_ticks: u64,
    pub total_volume_initial_cubic_millimetres: i128,
    pub total_volume_final_cubic_millimetres: i128,
    pub conservation_exact: bool,
    pub east_column_wet_by_tick: Option<u64>,
    pub settled_difference_max_micrometres: i64,
    pub settle_tolerance_micrometres: i64,
    pub levels_in_extent: bool,
    pub dry_cells_final: u32,
    pub final_level_east_micrometres: i64,
    pub repeated_run_identical: bool,
    pub in_place_equals_cloning: bool,
    pub step_cost_cells: usize,
    pub step_cost_edges: usize,
    pub step_cost_max_microseconds: u128,
    pub step_cost_mean_microseconds: u128,
    pub step_cost_debug_build: bool,
    pub final_network_hash: ContentHash,
    pub final_table_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterLatticeCheckErrorV1 {
    context: String,
    detail: String,
}

impl WaterLatticeCheckErrorV1 {
    fn new(context: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            detail: detail.into(),
        }
    }

    fn condition(context: impl Into<String>) -> Self {
        Self::new(context, "condition not met")
    }
}

impl Display for WaterLatticeCheckErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for WaterLatticeCheckErrorV1 {}

/// The frozen region of plan 19: `8 x 8` metre cells over a terrain that
/// falls one decimetre per column from west to east, the west column
/// holding water to `2 m`, everything else dry.
#[must_use]
pub fn plan_19_region() -> WaterLatticeRegionV1 {
    let count = (COLUMNS * ROWS) as usize;
    let floor = |index: usize| {
        WEST_FLOOR_MICROMETRES - FLOOR_STEP_MICROMETRES * (index % COLUMNS as usize) as i64
    };
    WaterLatticeRegionV1 {
        region_id: PersistentId::from_bytes([0x4c; 16]),
        origin_micrometres: [0, 0, 0],
        cell_size_micrometres: [CELL_SIZE_MICROMETRES, CELL_SIZE_MICROMETRES],
        columns: COLUMNS,
        rows: ROWS,
        ceiling_micrometres: CEILING_MICROMETRES,
        floor_micrometres: (0..count).map(floor).collect(),
        initial_level_micrometres: (0..count)
            .map(|index| {
                if index % COLUMNS as usize == 0 {
                    WEST_LEVEL_MICROMETRES
                } else {
                    floor(index)
                }
            })
            .collect(),
        sill_coefficient_permille: SILL_COEFFICIENT_PERMILLE,
        profile_revision: 1,
    }
}

struct RunTrace {
    hashes: Vec<(ContentHash, ContentHash)>,
    network: WaterFlowNetworkV1,
    volumes: WaterVolumeSetV1,
    east_column_wet_by_tick: Option<u64>,
}

fn hashes(
    network: &WaterFlowNetworkV1,
    volumes: &WaterVolumeSetV1,
) -> Result<(ContentHash, ContentHash), WaterLatticeCheckErrorV1> {
    Ok((
        network
            .network_hash()
            .map_err(|error| WaterLatticeCheckErrorV1::new("network hash", error.to_string()))?,
        volumes
            .set_hash()
            .map_err(|error| WaterLatticeCheckErrorV1::new("table hash", error.to_string()))?,
    ))
}

fn east_column_wet(region: &WaterLatticeRegionV1, volumes: &WaterVolumeSetV1) -> bool {
    (0..ROWS).all(|row| {
        let id = region.cell_id(COLUMNS - 1, row);
        let definition = &volumes.definitions[&id];
        let state = &volumes.states[&id];
        state.level_micrometres - definition.minimum_micrometres[1] > WET_DEPTH_MICROMETRES
    })
}

fn run(
    region: &WaterLatticeRegionV1,
    in_place: bool,
) -> Result<RunTrace, WaterLatticeCheckErrorV1> {
    let (mut volumes, mut network) = region
        .build(TICKS_PER_SECOND)
        .map_err(|error| WaterLatticeCheckErrorV1::new("build", error.to_string()))?;
    let mut trace = Vec::with_capacity(RUN_TICKS as usize + 1);
    trace.push(hashes(&network, &volumes)?);
    let mut east_column_wet_by_tick = None;
    for tick in 1..=RUN_TICKS {
        if in_place {
            network.step_in_place(&mut volumes).map_err(|error| {
                WaterLatticeCheckErrorV1::new("step in place", error.to_string())
            })?;
        } else {
            let stepped = network
                .step(&volumes)
                .map_err(|error| WaterLatticeCheckErrorV1::new("step", error.to_string()))?;
            network = stepped.network;
            volumes = stepped.volumes;
        }
        trace.push(hashes(&network, &volumes)?);
        if east_column_wet_by_tick.is_none() && east_column_wet(region, &volumes) {
            east_column_wet_by_tick = Some(tick);
        }
    }
    Ok(RunTrace {
        hashes: trace,
        network,
        volumes,
        east_column_wet_by_tick,
    })
}

fn step_cost(region: &WaterLatticeRegionV1) -> Result<(u128, u128), WaterLatticeCheckErrorV1> {
    let (mut volumes, mut network) = region
        .build(TICKS_PER_SECOND)
        .map_err(|error| WaterLatticeCheckErrorV1::new("cost build", error.to_string()))?;
    // One untimed warm-up step, as in the buoyancy harness.
    network
        .step_in_place(&mut volumes)
        .map_err(|error| WaterLatticeCheckErrorV1::new("cost warm-up", error.to_string()))?;
    let mut maximum = 0_u128;
    let mut total = 0_u128;
    for _ in 0..COST_STEPS {
        let started = Instant::now();
        network
            .step_in_place(&mut volumes)
            .map_err(|error| WaterLatticeCheckErrorV1::new("cost step", error.to_string()))?;
        let elapsed = started.elapsed().as_micros();
        maximum = maximum.max(elapsed);
        total += elapsed;
    }
    Ok((maximum, total / u128::from(COST_STEPS.max(1))))
}

pub fn run_water_lattice_check() -> Result<WaterLatticeCheckReportV1, WaterLatticeCheckErrorV1> {
    let region = plan_19_region();
    region
        .validate()
        .map_err(|error| WaterLatticeCheckErrorV1::new("region", error.to_string()))?;
    let (initial_volumes, initial_network) = region
        .build(TICKS_PER_SECOND)
        .map_err(|error| WaterLatticeCheckErrorV1::new("build", error.to_string()))?;
    if initial_volumes.definitions.len() != (COLUMNS * ROWS) as usize
        || initial_network.edges.len() != region.edge_count()
    {
        return Err(WaterLatticeCheckErrorV1::condition("G1 build counts"));
    }
    let total_volume_initial = initial_network.total_volume();

    let first = run(&region, true)?;
    let repeated = run(&region, true)?;
    let cloning = run(&region, false)?;
    let repeated_run_identical = first.hashes == repeated.hashes;
    let in_place_equals_cloning = first.hashes == cloning.hashes
        && first.network == cloning.network
        && first.volumes == cloning.volumes;

    let total_volume_final = first.network.total_volume();
    let conservation_exact = total_volume_final == total_volume_initial;

    // G3: levels inside their extents, the east column wet, wet
    // neighbours settled.
    let mut levels_in_extent = true;
    let mut dry_cells_final = 0_u32;
    for (id, definition) in &first.volumes.definitions {
        let level = first.volumes.states[id].level_micrometres;
        if level < definition.minimum_micrometres[1] || level > definition.maximum_micrometres[1] {
            levels_in_extent = false;
        }
        if level - definition.minimum_micrometres[1] <= WET_DEPTH_MICROMETRES {
            dry_cells_final += 1;
        }
    }
    // Apparatus (recorded in plan 19): a sill is settled when the heads
    // above it agree, `|max(0, level_a - sill) - max(0, level_b - sill)|`;
    // a cell draining through a sill above its neighbour's level is not
    // a level mismatch.
    let mut settled_difference_max = 0_i64;
    for edge in first.network.edges.values() {
        let WaterFlowEdgeKindV1::Open {
            sill_micrometres, ..
        } = edge.kind
        else {
            return Err(WaterLatticeCheckErrorV1::condition(
                "lattice edges are sills",
            ));
        };
        let Some(cell_b) = edge.cell_b else {
            return Err(WaterLatticeCheckErrorV1::condition(
                "lattice edges join two cells",
            ));
        };
        let head = |id: PersistentId| {
            (first.volumes.states[&id].level_micrometres - sill_micrometres).max(0)
        };
        let difference = (head(edge.cell_a) - head(cell_b)).abs();
        settled_difference_max = settled_difference_max.max(difference);
    }
    let east_id = region.cell_id(COLUMNS - 1, 0);
    let final_level_east = first.volumes.states[&east_id].level_micrometres;

    let (step_cost_max_microseconds, step_cost_mean_microseconds) = step_cost(&region)?;
    let (final_network_hash, final_table_hash) = hashes(&first.network, &first.volumes)?;

    let report = WaterLatticeCheckReportV1 {
        region_id: region.region_id,
        cells: initial_volumes.definitions.len(),
        edges: initial_network.edges.len(),
        run_ticks: RUN_TICKS,
        total_volume_initial_cubic_millimetres: total_volume_initial,
        total_volume_final_cubic_millimetres: total_volume_final,
        conservation_exact,
        east_column_wet_by_tick: first.east_column_wet_by_tick,
        settled_difference_max_micrometres: settled_difference_max,
        settle_tolerance_micrometres: SETTLE_TOLERANCE_MICROMETRES,
        levels_in_extent,
        dry_cells_final,
        final_level_east_micrometres: final_level_east,
        repeated_run_identical,
        in_place_equals_cloning,
        step_cost_cells: initial_volumes.definitions.len(),
        step_cost_edges: initial_network.edges.len(),
        step_cost_max_microseconds,
        step_cost_mean_microseconds,
        step_cost_debug_build: cfg!(debug_assertions),
        final_network_hash,
        final_table_hash,
    };
    if !conservation_exact {
        return Err(WaterLatticeCheckErrorV1::condition("G2 exact conservation"));
    }
    if report.east_column_wet_by_tick.is_none()
        || settled_difference_max > SETTLE_TOLERANCE_MICROMETRES
        || !levels_in_extent
    {
        return Err(WaterLatticeCheckErrorV1::new(
            "G3 downhill and settled",
            format!(
                "east wet by {:?}, settled difference {} um, levels in extent {}",
                report.east_column_wet_by_tick, settled_difference_max, levels_in_extent
            ),
        ));
    }
    if !repeated_run_identical || !in_place_equals_cloning {
        return Err(WaterLatticeCheckErrorV1::condition("G4 determinism"));
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lattice_check_passes() {
        let report = run_water_lattice_check().expect("lattice check passes");
        assert_eq!(report.cells, 64);
        assert_eq!(report.edges, 112);
        assert!(report.conservation_exact);
        assert!(report.repeated_run_identical);
        assert!(report.in_place_equals_cloning);
        assert!(report.east_column_wet_by_tick.is_some());
    }
}
