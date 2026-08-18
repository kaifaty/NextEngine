#![forbid(unsafe_code)]

use crate::volume_map::{VolumeMapBoundary, VolumeMapSample};

use super::*;

pub(super) fn particles(
    state: &DecodedState,
    geometry: Geometry,
    boundary: &[BoundarySample],
) -> Result<Reconstruction, WaterError> {
    let manifest = crate::geometry::AxisAlignedGeometryManifest::from_geometry(geometry)?;
    let fluid_grid = build_fluid_grid(&state.samples)?;
    let boundary_grid = build_boundary_grid(boundary)?;
    let mut rows = Vec::new();
    let mut fluid_indices = Vec::new();
    let mut solid_indices = Vec::new();
    rows.try_reserve_exact(state.samples.len())
        .map_err(heap_error)?;
    let mut total_fluid = 0_usize;
    let mut total_solid = 0_usize;
    let mut total_directed = 0_usize;
    for sample in &state.samples {
        let fluid = append_admitted_fluid_to(
            sample,
            &state.samples,
            &fluid_grid,
            &manifest,
            &mut fluid_indices,
        )?;
        let solid =
            append_admitted_boundary_to(sample, boundary, &boundary_grid, &mut solid_indices)?;
        let row_count = fluid.len().checked_add(solid.len()).ok_or_else(|| {
            WaterError::new(NEIGHBOR_CAPACITY_EXCEEDED, "fluid row count overflow")
        })?;
        validate_capacity(
            row_count,
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
            NEIGHBOR_CAPACITY_EXCEEDED,
            "fluid-plus-boundary row",
        )?;
        total_fluid = total_fluid.checked_add(fluid.len()).ok_or_else(|| {
            WaterError::new(
                NEIGHBOR_CAPACITY_EXCEEDED,
                "fluid neighbor aggregate overflow",
            )
        })?;
        total_solid = total_solid.checked_add(solid.len()).ok_or_else(|| {
            WaterError::new(
                NEIGHBOR_CAPACITY_EXCEEDED,
                "solid neighbor aggregate overflow",
            )
        })?;
        total_directed = total_directed.checked_add(row_count).ok_or_else(|| {
            WaterError::new(
                NEIGHBOR_CAPACITY_EXCEEDED,
                "directed neighbor aggregate overflow",
            )
        })?;
        validate_capacity(
            total_directed,
            MAXIMUM_DIRECTED_FLUID_NEIGHBORS,
            NEIGHBOR_CAPACITY_EXCEEDED,
            "directed fluid-row neighbors",
        )?;
        rows.push(Row {
            fluid_start: fluid.start,
            fluid_end: fluid.end,
            solid_start: solid.start,
            solid_end: solid.end,
        });
    }
    let scratch_capacity = fluid_indices
        .capacity()
        .checked_add(solid_indices.capacity())
        .ok_or_else(|| WaterError::new(NEIGHBOR_CAPACITY_EXCEEDED, "scratch capacity overflow"))?;
    validate_heap_plan(
        state.samples.len(),
        total_fluid,
        total_solid,
        scratch_capacity,
    )?;
    drop(fluid_grid);
    drop(boundary_grid);

    let mut fluid_neighbors = Vec::new();
    let mut solid_neighbors = Vec::new();
    fluid_neighbors
        .try_reserve_exact(total_fluid)
        .map_err(heap_error)?;
    solid_neighbors
        .try_reserve_exact(total_solid)
        .map_err(heap_error)?;
    for (row, sample) in rows.iter().copied().zip(&state.samples) {
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
        for boundary_index in &solid_indices[row.solid_start..row.solid_end] {
            let displacement = sample
                .position_um
                .checked_sub(boundary[*boundary_index].position_um)?;
            let sampled = kernel::sample(displacement)?;
            solid_neighbors.push(SolidNeighbor {
                boundary: *boundary_index,
                volume: boundary[*boundary_index].volume,
                value: sampled.value,
                gradient: sampled.gradient,
                feature_rank: 0,
            });
        }
    }
    drop(fluid_indices);
    drop(solid_indices);

    finish(state.samples.len(), rows, fluid_neighbors, solid_neighbors)
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
    finish(state.samples.len(), rows, fluid_neighbors, solid_neighbors)
}

fn finish(
    sample_count: usize,
    rows: Vec<Row>,
    fluid_neighbors: Vec<FluidNeighbor>,
    solid_neighbors: Vec<SolidNeighbor>,
) -> Result<Reconstruction, WaterError> {
    let mut rho_ratio = Vec::new();
    let mut alpha = Vec::new();
    rho_ratio
        .try_reserve_exact(sample_count)
        .map_err(heap_error)?;
    alpha.try_reserve_exact(sample_count).map_err(heap_error)?;
    for (index, row) in rows.iter().copied().enumerate() {
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
        rho_ratio.push(ratio);

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
