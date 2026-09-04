//! Plan 24 (ADR-106): the PhysX water presentation demo. A GPU fluid over
//! the basin whose floor is the exact water level and whose walls are the
//! rim's inner faces, seeded as a block above the surface so it drops,
//! splashes and spreads on the level, stepped at `60 Hz` from the frame
//! clock and published to the ADR-102 particle pass every frame. Presentation only: no command,
//! query, save, replay or root reads it; without the GPU library the demo
//! reports itself unavailable and the run continues with the stage's
//! droplets.

use std::sync::Arc;
use std::time::Duration;

use next_desktop_sdl_ash::{
    DesktopAdapterError, ParticleSurfaceProfileV1, ParticleSurfaceUpdateV1,
};
use next_physics_physx_ffi::{FluidDesc, NativeFluid, gpu_library_path};
use next_reference_game::{
    REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES, REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES,
    reference_water_basin_definition,
};

const DEMO_SPACING_METRES: f32 = 0.05;
const DEMO_TIMESTEP: Duration = Duration::from_micros(16_667);
const DEMO_MAX_STEPS_PER_FRAME: u32 = 4;
const DEMO_MAX_PARTICLES: u32 = 16_384;
const DEMO_RADIUS_MICROMETRES: u32 = 45_000;
/// The poured block: `1.0 x 0.6 x 1.0 m` centred over the basin, its
/// bottom `1.0 m` above the water level (inside the ADR-102 bounds).
const DEMO_BLOCK_HALF_METRES: f32 = 0.5;
const DEMO_BLOCK_BOTTOM_METRES: f32 = 1.0;
const DEMO_BLOCK_HEIGHT_METRES: f32 = 0.6;

pub(crate) struct PhysxWaterDemo {
    fluid: NativeFluid,
    accumulator: Duration,
    bounds: next_contracts::render_content::AabbI64V1,
}

impl PhysxWaterDemo {
    /// Creates the fluid over the basin; `Err` carries the reason the lane
    /// is unavailable (the run continues without it).
    pub(crate) fn new(bounds: next_contracts::render_content::AabbI64V1) -> Result<Self, String> {
        let basin = reference_water_basin_definition();
        let metres = |value: i64| value as f32 / 1_000_000.0;
        // The presentation fluid rests on the exact level: its floor is the
        // basin's initial level, its walls the rim's inner faces (the rim
        // boxes are `0.15 m` thick around the basin plan).
        let mut box_min = basin.minimum_micrometres.map(metres);
        let mut box_max = basin.maximum_micrometres.map(metres);
        for (centre, half) in REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES {
            let (centre, half) = (centre.map(metres), half.map(metres));
            if half[0] < half[2] {
                // A west or east wall.
                if centre[0] < box_min[0] + 1.0 {
                    box_min[0] = box_min[0].max(centre[0] + half[0]);
                } else {
                    box_max[0] = box_max[0].min(centre[0] - half[0]);
                }
            } else if centre[2] > box_max[2] - 1.0 {
                box_max[2] = box_max[2].min(centre[2] - half[2]);
            } else {
                box_min[2] = box_min[2].max(centre[2] + half[2]);
            }
        }
        box_min[1] = metres(REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES);
        let centre_x = (box_min[0] + box_max[0]) * 0.5;
        let centre_z = (box_min[2] + box_max[2]) * 0.5;
        let path = gpu_library_path().ok_or_else(|| "PhysX GPU library not found".to_owned())?;
        let fluid = NativeFluid::create(&FluidDesc {
            spacing_metres: DEMO_SPACING_METRES,
            box_min_metres: box_min,
            box_max_metres: [box_max[0], box_max[1] + 10.0, box_max[2]],
            seed_min_metres: [
                centre_x - DEMO_BLOCK_HALF_METRES,
                box_min[1] + DEMO_BLOCK_BOTTOM_METRES,
                centre_z - DEMO_BLOCK_HALF_METRES,
            ],
            seed_max_metres: [
                centre_x + DEMO_BLOCK_HALF_METRES,
                box_min[1] + DEMO_BLOCK_BOTTOM_METRES + DEMO_BLOCK_HEIGHT_METRES,
                centre_z + DEMO_BLOCK_HALF_METRES,
            ],
            timestep_seconds: DEMO_TIMESTEP.as_secs_f32(),
            max_particles: DEMO_MAX_PARTICLES,
            gpu_library_path: Some(path.display().to_string()),
        })
        .map_err(|error| format!("PhysX fluid unavailable: {error}"))?;
        Ok(Self {
            fluid,
            accumulator: Duration::ZERO,
            bounds,
        })
    }

    pub(crate) fn particle_surface_profile(&self) -> ParticleSurfaceProfileV1 {
        ParticleSurfaceProfileV1 {
            particle_capacity: DEMO_MAX_PARTICLES,
            radius_micrometres: DEMO_RADIUS_MICROMETRES,
            bounds: self.bounds,
            absorption_per_metre: [1.2, 0.5, 0.25],
            refraction_strength: 0.08,
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

    /// Steps the fluid by the frame's elapsed time (at most four `60 Hz`
    /// steps) and returns the particle update for this frame.
    pub(crate) fn advance(
        &mut self,
        elapsed: Duration,
        sequence: u64,
    ) -> Result<Option<Arc<ParticleSurfaceUpdateV1>>, DesktopAdapterError> {
        self.accumulator = self.accumulator.saturating_add(elapsed);
        let mut steps = 0;
        while self.accumulator >= DEMO_TIMESTEP && steps < DEMO_MAX_STEPS_PER_FRAME {
            self.accumulator -= DEMO_TIMESTEP;
            self.fluid.step().map_err(|error| {
                DesktopAdapterError::client("PHYSX_WATER_STEP_FAILED", error.to_string())
            })?;
            steps += 1;
        }
        if steps == DEMO_MAX_STEPS_PER_FRAME {
            self.accumulator = Duration::ZERO;
        }
        let sample = self.fluid.read().map_err(|error| {
            DesktopAdapterError::client("PHYSX_WATER_READ_FAILED", error.to_string())
        })?;
        // Droplets that left the declared bounds (the pass rejects any
        // position outside them, exclusive at the maximum) are dropped for
        // this frame; they are out of the basin's presentation region.
        let micrometres = |value: f32| (value * 1_000_000.0).round() as i64;
        let mut positions = Vec::with_capacity(sample.positions.len());
        let mut velocities = Vec::with_capacity(sample.positions.len());
        for (position, velocity) in sample.positions.iter().zip(&sample.velocities) {
            if !position.iter().all(|value| value.is_finite()) {
                continue;
            }
            let point = position.map(micrometres);
            if !self.bounds.contains(point) {
                continue;
            }
            positions.push(point);
            velocities.push(velocity.map(|value| {
                if value.is_finite() {
                    (value * 1_000_000.0).clamp(-2.0e9, 2.0e9) as i32
                } else {
                    0
                }
            }));
        }
        if positions.is_empty() {
            return Ok(None);
        }
        Ok(Some(Arc::new(ParticleSurfaceUpdateV1::new(
            sequence,
            positions,
            Vec::new(),
            Vec::new(),
            velocities,
            Vec::new(),
        )?)))
    }
}
