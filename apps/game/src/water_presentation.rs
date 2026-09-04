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
    WATER_JET_MAX_PARTICLES, WATER_JET_RADIUS_MICROMETRES, WATER_SURFACE_INDEX_CAPACITY,
    WATER_SURFACE_VERTEX_CAPACITY, WaterPresentationFrameV1, reference_water_basin_definition,
    reference_water_surface_bindings, reference_water_vessel_definitions,
};

/// Frozen plan 09 shading constants for the jet (the NGQ10 revision 4 look
/// without producer kernels: isotropic droplets, spray off).
const JET_ABSORPTION_PER_METRE: [f32; 3] = [1.2, 0.5, 0.25];
const JET_REFRACTION_STRENGTH: f32 = 0.08;
const JET_BOUNDS_MARGIN_MICROMETRES: i64 = 1_000_000;

pub(crate) struct WaterPresentationFeed {
    surfaces: Vec<(AssetId, AssetRevisionRefV1)>,
    particle_bounds: AabbI64V1,
    sequence: u64,
    last_frame: Option<Arc<WaterPresentationFrameV1>>,
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
            surfaces.push((binding.mesh_asset_id, revision));
        }
        let mut minimum = reference_water_basin_definition().minimum_micrometres;
        let mut maximum = reference_water_basin_definition().maximum_micrometres;
        for vessel in reference_water_vessel_definitions() {
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
        Ok(Self {
            surfaces,
            particle_bounds,
            sequence: 0,
            last_frame: None,
        })
    }

    pub(crate) fn dynamic_surface_profiles(&self) -> Vec<DynamicSurfaceProfileV1> {
        self.surfaces
            .iter()
            .map(|(_, revision)| DynamicSurfaceProfileV1 {
                mesh_revision: *revision,
                vertex_capacity: WATER_SURFACE_VERTEX_CAPACITY,
                index_capacity: WATER_SURFACE_INDEX_CAPACITY,
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
        let mut updates = Vec::with_capacity(frame.surfaces.len());
        for surface in &frame.surfaces {
            let Some((_, revision)) = self
                .surfaces
                .iter()
                .find(|(asset_id, _)| *asset_id == surface.mesh_asset_id)
            else {
                continue;
            };
            updates.push(Arc::new(DynamicSurfaceUpdateV1::new(
                *revision,
                sequence,
                surface.positions_micrometres.clone(),
                surface.normals_snorm16.clone(),
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
