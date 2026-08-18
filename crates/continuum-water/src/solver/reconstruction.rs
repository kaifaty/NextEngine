#![forbid(unsafe_code)]

use crate::volume_map::{VolumeMapBoundary, VolumeMapSample};

use super::neighborhood::GridEntry;
use super::*;

pub(super) fn particles(
    state: &DecodedState,
    geometry: Geometry,
    boundary: &[BoundarySample],
) -> Result<Reconstruction, WaterError> {
    particles_with_execution(state, geometry, boundary, None)
}

pub(super) fn particles_with_workers(
    state: &DecodedState,
    geometry: Geometry,
    boundary: &[BoundarySample],
    workers: &DeterministicWorkers,
) -> Result<Reconstruction, WaterError> {
    particles_with_execution(state, geometry, boundary, Some(workers))
}

fn particles_with_execution(
    state: &DecodedState,
    geometry: Geometry,
    boundary: &[BoundarySample],
    workers: Option<&DeterministicWorkers>,
) -> Result<Reconstruction, WaterError> {
    let manifest = crate::geometry::AxisAlignedGeometryManifest::from_geometry(geometry)?;
    let fluid_grid = build_fluid_grid(&state.samples)?;
    let boundary_grid = build_boundary_grid(boundary)?;
    let plan = if let Some(workers) = workers {
        discover_particle_indices_parallel(
            state,
            boundary,
            &fluid_grid,
            &boundary_grid,
            &manifest,
            workers,
        )?
    } else {
        discover_particle_indices_range(
            state,
            boundary,
            &fluid_grid,
            &boundary_grid,
            &manifest,
            0,
            state.samples.len(),
        )?
        .into_plan()?
    };
    let ParticleIndexPlan {
        rows,
        fluid_indices,
        solid_indices,
        peak_scratch_capacity,
    } = plan;
    let total_fluid = fluid_indices.len();
    let total_solid = solid_indices.len();
    validate_directed_total(total_fluid, total_solid)?;
    let scratch_capacity = fluid_indices
        .capacity()
        .checked_add(solid_indices.capacity())
        .ok_or_else(|| WaterError::new(NEIGHBOR_CAPACITY_EXCEEDED, "scratch capacity overflow"))?;
    validate_heap_plan(state.samples.len(), 0, 0, peak_scratch_capacity)?;
    validate_heap_plan(
        state.samples.len(),
        total_fluid,
        total_solid,
        scratch_capacity,
    )?;
    drop(fluid_grid);
    drop(boundary_grid);

    let (fluid_neighbors, solid_neighbors) = if let Some(workers) = workers {
        let mut fluid_neighbors = filled_vec(total_fluid, FluidNeighbor::default())?;
        let mut solid_neighbors = filled_vec(total_solid, SolidNeighbor::default())?;
        workers.try_fill_partitioned_pair(
            rows.len(),
            &mut fluid_neighbors,
            &mut solid_neighbors,
            |index| {
                if index == rows.len() {
                    (total_fluid, total_solid)
                } else {
                    (rows[index].fluid_start, rows[index].solid_start)
                }
            },
            |partition, fluid_output, solid_output| {
                fill_particle_neighbor_partition(
                    partition,
                    fluid_output,
                    solid_output,
                    state,
                    boundary,
                    &rows,
                    &fluid_indices,
                    &solid_indices,
                )
            },
        )?;
        (fluid_neighbors, solid_neighbors)
    } else {
        fill_particle_neighbors_serial(
            state,
            boundary,
            &rows,
            &fluid_indices,
            &solid_indices,
            total_fluid,
            total_solid,
        )?
    };
    drop(fluid_indices);
    drop(solid_indices);

    finish(
        state.samples.len(),
        rows,
        fluid_neighbors,
        solid_neighbors,
        workers,
    )
}

#[allow(clippy::too_many_arguments)]
fn fill_particle_neighbor_partition(
    partition: parallel::LogicalPartition,
    fluid_output: &mut [FluidNeighbor],
    solid_output: &mut [SolidNeighbor],
    state: &DecodedState,
    boundary: &[BoundarySample],
    rows: &[Row],
    fluid_indices: &[usize],
    solid_indices: &[usize],
) -> Result<(), WaterError> {
    let mut fluid_slot = 0_usize;
    let mut solid_slot = 0_usize;
    for (offset, row) in rows[partition.start..partition.end]
        .iter()
        .copied()
        .enumerate()
    {
        let index = partition.start + offset;
        let sample = &state.samples[index];
        for other in &fluid_indices[row.fluid_start..row.fluid_end] {
            let displacement = sample
                .position_um
                .checked_sub(state.samples[*other].position_um)?;
            let sampled = kernel::sample(displacement)?;
            fluid_output[fluid_slot] = FluidNeighbor {
                other: *other,
                value: sampled.value,
                gradient: sampled.gradient,
            };
            fluid_slot += 1;
        }
        for boundary_index in &solid_indices[row.solid_start..row.solid_end] {
            let displacement = sample
                .position_um
                .checked_sub(boundary[*boundary_index].position_um)?;
            let sampled = kernel::sample(displacement)?;
            solid_output[solid_slot] = SolidNeighbor {
                boundary: *boundary_index,
                volume: boundary[*boundary_index].volume,
                value: sampled.value,
                gradient: sampled.gradient,
                feature_rank: 0,
            };
            solid_slot += 1;
        }
    }
    if fluid_slot != fluid_output.len() || solid_slot != solid_output.len() {
        return Err(WaterError::new(
            crate::error::WORKER_FAILURE,
            "neighbor fragment did not fill its declared output ranges",
        ));
    }
    Ok(())
}

fn fill_particle_neighbors_serial(
    state: &DecodedState,
    boundary: &[BoundarySample],
    rows: &[Row],
    fluid_indices: &[usize],
    solid_indices: &[usize],
    total_fluid: usize,
    total_solid: usize,
) -> Result<(Vec<FluidNeighbor>, Vec<SolidNeighbor>), WaterError> {
    let mut fluid_neighbors = filled_vec(total_fluid, FluidNeighbor::default())?;
    let mut solid_neighbors = filled_vec(total_solid, SolidNeighbor::default())?;
    fill_particle_neighbor_partition(
        parallel::LogicalPartition {
            ordinal: 0,
            start: 0,
            end: rows.len(),
        },
        &mut fluid_neighbors,
        &mut solid_neighbors,
        state,
        boundary,
        rows,
        fluid_indices,
        solid_indices,
    )?;
    Ok((fluid_neighbors, solid_neighbors))
}

struct ParticleIndexPlan {
    rows: Vec<Row>,
    fluid_indices: Vec<usize>,
    solid_indices: Vec<usize>,
    peak_scratch_capacity: usize,
}

struct ParticleIndexFragment {
    rows: Vec<Row>,
    fluid_indices: Vec<usize>,
    solid_indices: Vec<usize>,
}

impl ParticleIndexFragment {
    fn into_plan(self) -> Result<ParticleIndexPlan, WaterError> {
        let peak_scratch_capacity = self
            .fluid_indices
            .capacity()
            .checked_add(self.solid_indices.capacity())
            .ok_or_else(scratch_capacity_overflow)?;
        Ok(ParticleIndexPlan {
            rows: self.rows,
            fluid_indices: self.fluid_indices,
            solid_indices: self.solid_indices,
            peak_scratch_capacity,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn discover_particle_indices_range(
    state: &DecodedState,
    boundary: &[BoundarySample],
    fluid_grid: &[GridEntry],
    boundary_grid: &[GridEntry],
    manifest: &crate::geometry::AxisAlignedGeometryManifest,
    start: usize,
    end: usize,
) -> Result<ParticleIndexFragment, WaterError> {
    let mut rows = Vec::new();
    let mut fluid_indices = Vec::new();
    let mut solid_indices = Vec::new();
    rows.try_reserve_exact(end - start).map_err(heap_error)?;
    for sample in &state.samples[start..end] {
        let fluid = append_admitted_fluid_to(
            sample,
            &state.samples,
            fluid_grid,
            manifest,
            &mut fluid_indices,
        )?;
        let solid =
            append_admitted_boundary_to(sample, boundary, boundary_grid, &mut solid_indices)?;
        validate_particle_row(fluid.len(), solid.len())?;
        rows.push(Row {
            fluid_start: fluid.start,
            fluid_end: fluid.end,
            solid_start: solid.start,
            solid_end: solid.end,
        });
    }
    Ok(ParticleIndexFragment {
        rows,
        fluid_indices,
        solid_indices,
    })
}

fn discover_particle_indices_parallel(
    state: &DecodedState,
    boundary: &[BoundarySample],
    fluid_grid: &[GridEntry],
    boundary_grid: &[GridEntry],
    manifest: &crate::geometry::AxisAlignedGeometryManifest,
    workers: &DeterministicWorkers,
) -> Result<ParticleIndexPlan, WaterError> {
    let fragments = workers.try_map_partitions(state.samples.len(), |partition| {
        discover_particle_indices_range(
            state,
            boundary,
            fluid_grid,
            boundary_grid,
            manifest,
            partition.start,
            partition.end,
        )
    })?;
    merge_particle_index_fragments(state.samples.len(), fragments)
}

fn merge_particle_index_fragments(
    sample_count: usize,
    fragments: Vec<ParticleIndexFragment>,
) -> Result<ParticleIndexPlan, WaterError> {
    let mut total_fluid = 0_usize;
    let mut total_solid = 0_usize;
    let mut fragment_capacity = 0_usize;
    let mut fragment_row_capacity = 0_usize;
    for fragment in &fragments {
        total_fluid = total_fluid
            .checked_add(fragment.fluid_indices.len())
            .ok_or_else(neighbor_aggregate_overflow)?;
        total_solid = total_solid
            .checked_add(fragment.solid_indices.len())
            .ok_or_else(neighbor_aggregate_overflow)?;
        fragment_capacity = fragment_capacity
            .checked_add(fragment.fluid_indices.capacity())
            .and_then(|value| value.checked_add(fragment.solid_indices.capacity()))
            .ok_or_else(scratch_capacity_overflow)?;
        fragment_row_capacity = fragment_row_capacity
            .checked_add(fragment.rows.capacity())
            .ok_or_else(scratch_capacity_overflow)?;
    }
    validate_directed_total(total_fluid, total_solid)?;
    let mut rows = Vec::new();
    let mut fluid_indices = Vec::new();
    let mut solid_indices = Vec::new();
    rows.try_reserve_exact(sample_count).map_err(heap_error)?;
    fluid_indices
        .try_reserve_exact(total_fluid)
        .map_err(heap_error)?;
    solid_indices
        .try_reserve_exact(total_solid)
        .map_err(heap_error)?;
    let global_capacity = fluid_indices
        .capacity()
        .checked_add(solid_indices.capacity())
        .ok_or_else(scratch_capacity_overflow)?;
    let extra_row_indices = fragment_row_capacity
        .checked_mul(size_of::<Row>())
        .and_then(|bytes| bytes.checked_add(size_of::<usize>() - 1))
        .map(|bytes| bytes / size_of::<usize>())
        .ok_or_else(scratch_capacity_overflow)?;
    let peak_scratch_capacity = fragment_capacity
        .checked_add(global_capacity)
        .and_then(|value| value.checked_add(extra_row_indices))
        .ok_or_else(scratch_capacity_overflow)?;
    for mut fragment in fragments {
        let fluid_base = fluid_indices.len();
        let solid_base = solid_indices.len();
        for row in fragment.rows {
            rows.push(Row {
                fluid_start: fluid_base + row.fluid_start,
                fluid_end: fluid_base + row.fluid_end,
                solid_start: solid_base + row.solid_start,
                solid_end: solid_base + row.solid_end,
            });
        }
        fluid_indices.append(&mut fragment.fluid_indices);
        solid_indices.append(&mut fragment.solid_indices);
    }
    Ok(ParticleIndexPlan {
        rows,
        fluid_indices,
        solid_indices,
        peak_scratch_capacity,
    })
}

fn validate_particle_row(fluid_count: usize, solid_count: usize) -> Result<(), WaterError> {
    let row_count = fluid_count
        .checked_add(solid_count)
        .ok_or_else(|| WaterError::new(NEIGHBOR_CAPACITY_EXCEEDED, "fluid row count overflow"))?;
    validate_capacity(
        row_count,
        MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
        NEIGHBOR_CAPACITY_EXCEEDED,
        "fluid-plus-boundary row",
    )
}

fn validate_directed_total(fluid_count: usize, solid_count: usize) -> Result<(), WaterError> {
    let total = fluid_count
        .checked_add(solid_count)
        .ok_or_else(neighbor_aggregate_overflow)?;
    validate_capacity(
        total,
        MAXIMUM_DIRECTED_FLUID_NEIGHBORS,
        NEIGHBOR_CAPACITY_EXCEEDED,
        "directed fluid-row neighbors",
    )
}

fn neighbor_aggregate_overflow() -> WaterError {
    WaterError::new(
        NEIGHBOR_CAPACITY_EXCEEDED,
        "directed neighbor aggregate overflow",
    )
}

fn scratch_capacity_overflow() -> WaterError {
    WaterError::new(NEIGHBOR_CAPACITY_EXCEEDED, "scratch capacity overflow")
}

pub(super) fn volume_map(
    state: &DecodedState,
    geometry: Geometry,
) -> Result<Reconstruction, WaterError> {
    let manifest = crate::geometry::AxisAlignedGeometryManifest::from_geometry(geometry)?;
    let fluid_grid = build_fluid_grid(&state.samples)?;
    let map = VolumeMapBoundary::new(geometry)?;
    let mut rows = Vec::new();
    let mut fluid_indices = Vec::new();
    let mut map_samples: Vec<Option<VolumeMapSample>> = Vec::new();
    rows.try_reserve_exact(state.samples.len())
        .map_err(heap_error)?;
    map_samples
        .try_reserve_exact(state.samples.len())
        .map_err(heap_error)?;
    let mut total_fluid = 0_usize;
    let mut total_solid = 0_usize;
    let mut total_directed = 0_usize;
    for (index, sample) in state.samples.iter().enumerate() {
        let fluid = append_admitted_fluid_to(
            sample,
            &state.samples,
            &fluid_grid,
            &manifest,
            &mut fluid_indices,
        )?;
        let map_sample = map.sample(state.positions[index])?;
        let solid_count = usize::from(map_sample.is_some());
        let row_count = fluid.len().checked_add(solid_count).ok_or_else(|| {
            WaterError::new(NEIGHBOR_CAPACITY_EXCEEDED, "volume-map row count overflow")
        })?;
        validate_capacity(
            row_count,
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
            NEIGHBOR_CAPACITY_EXCEEDED,
            "fluid-plus-volume-map row",
        )?;
        total_fluid = total_fluid.checked_add(fluid.len()).ok_or_else(|| {
            WaterError::new(
                NEIGHBOR_CAPACITY_EXCEEDED,
                "volume-map fluid neighbor aggregate overflow",
            )
        })?;
        total_solid = total_solid.checked_add(solid_count).ok_or_else(|| {
            WaterError::new(
                NEIGHBOR_CAPACITY_EXCEEDED,
                "volume-map solid neighbor aggregate overflow",
            )
        })?;
        total_directed = total_directed.checked_add(row_count).ok_or_else(|| {
            WaterError::new(
                NEIGHBOR_CAPACITY_EXCEEDED,
                "volume-map directed neighbor aggregate overflow",
            )
        })?;
        validate_capacity(
            total_directed,
            MAXIMUM_DIRECTED_FLUID_NEIGHBORS,
            NEIGHBOR_CAPACITY_EXCEEDED,
            "volume-map directed fluid-row neighbors",
        )?;
        rows.push(Row {
            fluid_start: fluid.start,
            fluid_end: fluid.end,
            solid_start: total_solid - solid_count,
            solid_end: total_solid,
        });
        map_samples.push(map_sample);
    }
    validate_heap_plan(
        state.samples.len(),
        total_fluid,
        total_solid,
        fluid_indices.capacity(),
    )?;
    drop(fluid_grid);

    let mut fluid_neighbors = Vec::new();
    let mut solid_neighbors = Vec::new();
    fluid_neighbors
        .try_reserve_exact(total_fluid)
        .map_err(heap_error)?;
    solid_neighbors
        .try_reserve_exact(total_solid)
        .map_err(heap_error)?;
    for (row_index, (row, sample)) in rows.iter().copied().zip(&state.samples).enumerate() {
        for other in &fluid_indices[row.fluid_start..row.fluid_end] {
            let displacement = sample
                .position_um
                .checked_sub(state.samples[*other].position_um)?;
            let sampled = kernel::sample(displacement)?;
            fluid_neighbors.push(FluidNeighbor {
                other: *other,
                value: sampled.value,
                gradient: sampled.gradient,
            });
        }
        if let Some(sampled) = map_samples[row_index] {
            solid_neighbors.push(SolidNeighbor {
                boundary: usize::MAX,
                volume: sampled.volume,
                value: sampled.value,
                gradient: sampled.gradient,
                feature_rank: sampled.feature_rank,
            });
        }
    }
    drop(fluid_indices);
    finish(
        state.samples.len(),
        rows,
        fluid_neighbors,
        solid_neighbors,
        None,
    )
}

fn finish(
    sample_count: usize,
    rows: Vec<Row>,
    fluid_neighbors: Vec<FluidNeighbor>,
    solid_neighbors: Vec<SolidNeighbor>,
    workers: Option<&DeterministicWorkers>,
) -> Result<Reconstruction, WaterError> {
    let mut rho_ratio = Vec::new();
    let mut alpha = Vec::new();
    rho_ratio
        .try_reserve_exact(sample_count)
        .map_err(heap_error)?;
    alpha.try_reserve_exact(sample_count).map_err(heap_error)?;
    if let Some(workers) = workers {
        let factors = workers.try_map(rows.len(), |index| {
            finish_row(index, rows[index], &fluid_neighbors, &solid_neighbors)
        })?;
        for (ratio, factor) in factors {
            rho_ratio.push(ratio);
            alpha.push(factor);
        }
        return Ok(Reconstruction {
            rows,
            fluid: fluid_neighbors,
            solid: solid_neighbors,
            rho_ratio,
            alpha,
        });
    }
    for (index, row) in rows.iter().copied().enumerate() {
        let (ratio, factor) = finish_row(index, row, &fluid_neighbors, &solid_neighbors)?;
        rho_ratio.push(ratio);
        alpha.push(factor);
    }
    Ok(Reconstruction {
        rows,
        fluid: fluid_neighbors,
        solid: solid_neighbors,
        rho_ratio,
        alpha,
    })
}

fn finish_row(
    index: usize,
    row: Row,
    fluid_neighbors: &[FluidNeighbor],
    solid_neighbors: &[SolidNeighbor],
) -> Result<(f64, f64), WaterError> {
    let mut ratio = checked_scalar(REST_VOLUME * kernel::value_at_zero(), "density self term")?;
    for neighbor in &fluid_neighbors[row.fluid_start..row.fluid_end] {
        ratio = checked_scalar(
            ratio + (REST_VOLUME * neighbor.value),
            "density fluid reduction",
        )?;
    }
    for neighbor in &solid_neighbors[row.solid_start..row.solid_end] {
        ratio = checked_scalar(
            ratio + (neighbor.volume * neighbor.value),
            "density boundary reduction",
        )?;
    }
    let _density = checked_scalar(RHO0 * ratio, "density")?;
    let mut sum_sq = 0.0;
    let mut central = Vec3f::ZERO;
    for neighbor in &fluid_neighbors[row.fluid_start..row.fluid_end] {
        let volume_gradient = neighbor
            .gradient
            .scale(REST_VOLUME)
            .checked("factor fluid volume gradient")?;
        let g = Vec3f::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z)
            .checked("factor fluid g")?;
        sum_sq = checked_scalar(sum_sq + g.dot(g), "factor sum squares")?;
        central = central.sub(g).checked("factor central fluid")?;
    }
    for neighbor in &solid_neighbors[row.solid_start..row.solid_end] {
        let volume_gradient = neighbor
            .gradient
            .scale(neighbor.volume)
            .checked("factor boundary volume gradient")?;
        let g = Vec3f::new(-volume_gradient.x, -volume_gradient.y, -volume_gradient.z)
            .checked("factor boundary g")?;
        central = central.sub(g).checked("factor central boundary")?;
    }
    let denominator = checked_scalar(
        sum_sq + central.dot(central),
        &format!("factor denominator sample {index}"),
    )?;
    let factor = if denominator > 1.0e-5 {
        checked_scalar(1.0 / denominator, "factor reciprocal")?
    } else {
        0.0
    };
    Ok((ratio, factor))
}
