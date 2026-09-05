//! Plan 09 (ADR-101/ADR-102): converts the presentation-only water frame
//! published by the simulation worker into the desktop adapter's dynamic
//! surface and particle surface updates. Presentation only; nothing here is
//! read back into gameplay.

use std::sync::Arc;

use next_contracts::ids::AssetId;
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::render_content::{AabbI64V1, RenderContentCatalogV1};
use next_desktop_sdl_ash::{
    DesktopAdapterError, DynamicSurfaceProfileV1, DynamicSurfaceResidencyV1,
    DynamicSurfaceShadingV1, DynamicSurfaceUpdateV1, ParticleSurfaceProfileV1,
    ParticleSurfaceUpdateV1,
};
use next_reference_game::{
    WATER_JET_MAX_PARTICLES, WATER_JET_RADIUS_MICROMETRES, WaterPresentationFrameV1,
    reference_water_basin_definition, reference_water_pond_definition,
    reference_water_surface_bindings, reference_water_vessel_definitions,
};

/// Frozen plan 09 shading constants for the jet (the NGQ10 revision 4 look
/// without producer kernels: isotropic droplets, spray off).
const JET_ABSORPTION_PER_METRE: [f32; 3] = [1.2, 0.5, 0.25];
const JET_REFRACTION_STRENGTH: f32 = 0.08;
const JET_BOUNDS_MARGIN_MICROMETRES: i64 = 1_000_000;

pub(crate) struct WaterPresentationFeed {
    surfaces: Vec<(
        AssetId,
        AssetRevisionRefV1,
        next_reference_game::WaterSurfaceGridV1,
    )>,
    particle_bounds: AabbI64V1,
    sequence: u64,
    last_frame: Option<Arc<WaterPresentationFrameV1>>,
    /// Plan 38: the wave grid per surface mesh (none when unstable).
    waves: Vec<(AssetId, Option<crate::water_waves::WaveGridV1>)>,
    /// Plan 38: the last published frame index, so a republished frame
    /// does not excite the grid twice.
    last_excited_frame: Option<u64>,
}

/// Dynamic surface updates plus the optional jet particle update for one
/// published frame.
pub(crate) type WaterPublicationV1 = (
    Vec<Arc<DynamicSurfaceUpdateV1>>,
    Option<Arc<ParticleSurfaceUpdateV1>>,
);

impl WaterPresentationFeed {
    pub(crate) fn new(catalog: &RenderContentCatalogV1) -> Result<Self, String> {
        let mut surfaces = Vec::new();
        for binding in reference_water_surface_bindings() {
            let mesh = catalog
                .meshes()
                .iter()
                .find(|mesh| mesh.asset_id() == binding.mesh_asset_id)
                .ok_or_else(|| {
                    format!(
                        "water surface mesh {} is missing from the render catalog",
                        binding.mesh_asset_id.to_hex()
                    )
                })?;
            let revision = mesh.asset_revision().map_err(|error| error.to_string())?;
            surfaces.push((binding.mesh_asset_id, revision, binding.grid));
        }
        let mut minimum = reference_water_basin_definition().minimum_micrometres;
        let mut maximum = reference_water_basin_definition().maximum_micrometres;
        let showcase = next_reference_game::reference_water_showcase_definitions()
            .map_err(|error| error.to_string())?;
        for vessel in reference_water_vessel_definitions()
            .into_iter()
            .chain([reference_water_pond_definition()])
            .chain(showcase.iter().cloned())
        {
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(vessel.minimum_micrometres[axis]);
                maximum[axis] = maximum[axis].max(vessel.maximum_micrometres[axis]);
            }
        }
        for axis in 0..3 {
            minimum[axis] -= JET_BOUNDS_MARGIN_MICROMETRES;
            maximum[axis] += JET_BOUNDS_MARGIN_MICROMETRES;
        }
        let particle_bounds =
            AabbI64V1::new(minimum, maximum).map_err(|error| error.to_string())?;
        // Plan 38: one wave grid per ring at its authored depth.
        let definitions: Vec<_> = [reference_water_basin_definition()]
            .into_iter()
            .chain(reference_water_vessel_definitions())
            .chain([reference_water_pond_definition()])
            .chain(showcase)
            .collect();
        let waves = reference_water_surface_bindings()
            .into_iter()
            .map(|binding| {
                let grid = definitions
                    .iter()
                    .find(|definition| definition.volume_id == binding.volume_id)
                    .and_then(|definition| {
                        crate::water_waves::WaveGridV1::new(
                            binding.grid,
                            [
                                definition.minimum_micrometres[0],
                                definition.minimum_micrometres[2],
                            ],
                            [
                                definition.maximum_micrometres[0],
                                definition.maximum_micrometres[2],
                            ],
                            definition.initial_level_micrometres
                                - definition.minimum_micrometres[1],
                        )
                    });
                (binding.mesh_asset_id, grid)
            })
            .collect();
        Ok(Self {
            surfaces,
            particle_bounds,
            sequence: 0,
            last_frame: None,
            waves,
            last_excited_frame: None,
        })
    }

    pub(crate) fn dynamic_surface_profiles(&self) -> Vec<DynamicSurfaceProfileV1> {
        self.surfaces
            .iter()
            .map(|(_, revision, grid)| DynamicSurfaceProfileV1 {
                mesh_revision: *revision,
                // Plan 42: each ring declares its own grid's counts.
                vertex_capacity: grid.vertex_count(),
                index_capacity: grid.index_count(),
                residency: DynamicSurfaceResidencyV1::DeviceLocal,
                shading: DynamicSurfaceShadingV1::WaterSurface,
            })
            .collect()
    }

    pub(crate) fn particle_surface_profile(&self) -> ParticleSurfaceProfileV1 {
        ParticleSurfaceProfileV1 {
            particle_capacity: WATER_JET_MAX_PARTICLES,
            radius_micrometres: WATER_JET_RADIUS_MICROMETRES,
            bounds: self.particle_bounds,
            absorption_per_metre: JET_ABSORPTION_PER_METRE,
            refraction_strength: JET_REFRACTION_STRENGTH,
            thickness_scale: 1.0,
            spray_neighbour_threshold: 0,
            spray_radius_micrometres: 0,
            spray_alpha: 0.0,
            spray_cluster_threshold: 0,
            spray_subdroplets: 1,
            spray_streak_seconds: 0.0,
            bulk_neighbour_count: 0,
            edge_radius_scale: 1.0,
            cleanup_radius_pixels: 4,
        }
    }

    /// Plan 24: the next publication sequence (shared with the particle
    /// updates of the PhysX lane so every update stays strictly increasing).
    pub(crate) fn next_sequence(&mut self) -> u64 {
        self.sequence = self.sequence.saturating_add(1);
        self.sequence
    }

    #[cfg(feature = "physx-water")]
    pub(crate) const fn particle_bounds(&self) -> AabbI64V1 {
        self.particle_bounds
    }

    /// Converts the worker's water frame into adapter updates. A missing
    /// frame (menu republication) republishes the last one under a new
    /// sequence so the ring stays declared and current.
    pub(crate) fn publication(
        &mut self,
        frame: Option<&WaterPresentationFrameV1>,
    ) -> Result<WaterPublicationV1, DesktopAdapterError> {
        let frame: Arc<WaterPresentationFrameV1> = match (frame, self.last_frame.as_ref()) {
            (Some(frame), _) => Arc::new(frame.clone()),
            (None, Some(last)) => Arc::clone(last),
            (None, None) => return Ok((Vec::new(), None)),
        };
        let sequence = self.next_sequence();
        // Plan 38: excite once per stage frame, step once per publication.
        let excite = self.last_excited_frame != Some(frame.frame_index);
        self.last_excited_frame = Some(frame.frame_index);
        let mut updates = Vec::with_capacity(frame.surfaces.len());
        for surface in &frame.surfaces {
            let Some((_, revision, _)) = self
                .surfaces
                .iter()
                .find(|(asset_id, _, _)| *asset_id == surface.mesh_asset_id)
            else {
                continue;
            };
            let grid = self
                .waves
                .iter_mut()
                .find(|(asset_id, _)| *asset_id == surface.mesh_asset_id)
                .and_then(|(_, grid)| grid.as_mut());
            let (positions, normals) = match grid {
                Some(grid) => {
                    if excite {
                        grid.excite(
                            &frame.boxes,
                            &frame.edges,
                            next_reference_game::REFERENCE_WATER_FLOW_TICKS_PER_SECOND,
                        );
                    }
                    grid.step();
                    waved_surface(surface, grid)
                }
                None => (
                    surface.positions_micrometres.clone(),
                    surface.normals_snorm16.clone(),
                ),
            };
            updates.push(Arc::new(DynamicSurfaceUpdateV1::new(
                *revision,
                sequence,
                positions,
                normals,
                surface.indices.clone(),
            )?));
        }
        let particles = if frame.jet.positions_micrometres.is_empty() {
            None
        } else {
            Some(Arc::new(ParticleSurfaceUpdateV1::new(
                sequence,
                frame.jet.positions_micrometres.clone(),
                Vec::new(),
                Vec::new(),
                frame.jet.velocities_micrometres_per_second.clone(),
                Vec::new(),
            )?))
        };
        self.last_frame = Some(frame);
        Ok((updates, particles))
    }
}

/// Plan 38: the ring's vertices with the grid's heights added (under the
/// catalog's `±20 mm` cap) and the normals recomputed by the stage's rule.
fn waved_surface(
    surface: &next_reference_game::WaterSurfaceUpdateV1,
    grid: &crate::water_waves::WaveGridV1,
) -> (Vec<[i64; 3]>, Vec<[i16; 3]>) {
    let (columns, rows) = grid.dimensions();
    let cap = next_reference_game::WATER_RIPPLE_CAP_MICROMETRES;
    let mut positions = surface.positions_micrometres.clone();
    if positions.len() != columns * rows {
        return (positions, surface.normals_snorm16.clone());
    }
    for row in 0..rows {
        for column in 0..columns {
            let index = row * columns + column;
            positions[index][1] =
                (positions[index][1] + grid.height_micrometres(column, row)).clamp(-cap, cap - 1);
        }
    }
    let cell_x = ((positions[columns - 1][0] - positions[0][0]) / (columns as i64 - 1)).max(1);
    let cell_z =
        ((positions[(rows - 1) * columns][2] - positions[0][2]) / (rows as i64 - 1)).max(1);
    let height_of = |column: usize, row: usize| positions[row * columns + column][1];
    let mut normals = Vec::with_capacity(columns * rows);
    for row in 0..rows {
        for column in 0..columns {
            let left = height_of(column.saturating_sub(1), row);
            let right = height_of((column + 1).min(columns - 1), row);
            let back = height_of(column, row.saturating_sub(1));
            let front = height_of(column, (row + 1).min(rows - 1));
            let slope_x = (left - right) * 32_767 / (2 * cell_x);
            let slope_z = (back - front) * 32_767 / (2 * cell_z);
            normals.push([
                slope_x.clamp(-16_000, 16_000) as i16,
                32_767,
                slope_z.clamp(-16_000, 16_000) as i16,
            ]);
        }
    }
    (positions, normals)
}
