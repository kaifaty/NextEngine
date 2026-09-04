//! Plan 25 (ADR-106): the PhysX water presentation lane. A GPU fluid over
//! the basin whose floor is the exact water level and whose walls are the
//! rim's inner faces; particles are born where the exact model says water
//! leaves it (a floating box's waterline while it moves, the crest of a jet
//! or fall inside the box) and die when they settle on the level. Stepped
//! at `60 Hz` from the frame clock and published to the ADR-102 particle
//! pass every frame. Presentation only: no command, query, save, replay or
//! root reads it; without the GPU library the lane reports itself
//! unavailable and the run continues with the stage's droplets.

use std::sync::Arc;
use std::time::{Duration, Instant};

use next_contracts::physics::WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED;
use next_contracts::render_content::AabbI64V1;
use next_desktop_sdl_ash::{
    DesktopAdapterError, ParticleSurfaceProfileV1, ParticleSurfaceUpdateV1,
};
use next_physics_physx_ffi::{FluidDesc, FluidSample, NativeFluid, gpu_library_path};
use next_reference_game::{
    REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES, REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES,
    WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES, WATER_JET_MAX_SPAWN_PER_FRAME,
    WATER_SPLASH_LAUNCH_SPEED_MICROMETRES_PER_SECOND, WATER_SPLASH_MAX_PER_BOX,
    WATER_SPLASH_SPEED_PER_DROPLET_MICROMETRES_PER_SECOND,
    WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND, WaterEdgePresentationKindV1,
    WaterPresentationFrameV1, reference_water_basin_definition,
};

const LANE_SPACING_METRES: f32 = 0.05;
const LANE_TIMESTEP: Duration = Duration::from_micros(16_667);
const LANE_MAX_STEPS_PER_FRAME: u32 = 4;
const LANE_MAX_PARTICLES: u32 = 16_384;
const LANE_RADIUS_MICROMETRES: u32 = 45_000;
/// Plan 27: neighbours within two spacings drive spray and bulk.
const NEIGHBOUR_RADIUS_METRES: f32 = 2.0 * LANE_SPACING_METRES;
const SPRAY_NEIGHBOUR_THRESHOLD: u32 = 6;
const SPRAY_CLUSTER_THRESHOLD: u32 = 8;
const SPRAY_RADIUS_MICROMETRES: u32 = 4_000;
const SPRAY_ALPHA: f32 = 0.5;
const SPRAY_SUBDROPLETS: u32 = 12;
const SPRAY_STREAK_SECONDS: f32 = 1.0 / 60.0;
const BULK_NEIGHBOUR_COUNT: u32 = 20;
const EDGE_RADIUS_SCALE: f32 = 0.5;
/// Plan 25: a particle this close above the level at rest has returned to
/// the exact water (two spacings).
const ABSORB_BAND_METRES: f32 = 2.0 * LANE_SPACING_METRES;
const ABSORB_SPEED_METRES_PER_SECOND: f32 = 0.3;
/// Plan 24's block for the look (`--physx-water-pour`).
const POUR_BLOCK_HALF_METRES: f32 = 0.5;
const POUR_BLOCK_BOTTOM_METRES: f32 = 1.0;
const POUR_BLOCK_HEIGHT_METRES: f32 = 0.6;

/// The lane's fluid box in metres: floor at the level, walls at the rim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FluidBox {
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
}

impl FluidBox {
    pub(crate) fn contains_plan(&self, x: f32, z: f32) -> bool {
        x >= self.min[0] && x <= self.max[0] && z >= self.min[2] && z <= self.max[2]
    }
}

/// Plan 25: the basin's fluid box from the reference scene.
pub(crate) fn basin_fluid_box() -> FluidBox {
    let basin = reference_water_basin_definition();
    let metres = |value: i64| value as f32 / 1_000_000.0;
    let mut min = basin.minimum_micrometres.map(metres);
    let mut max = basin.maximum_micrometres.map(metres);
    for (centre, half) in REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES {
        let (centre, half) = (centre.map(metres), half.map(metres));
        if half[0] < half[2] {
            if centre[0] < min[0] + 1.0 {
                min[0] = min[0].max(centre[0] + half[0]);
            } else {
                max[0] = max[0].min(centre[0] - half[0]);
            }
        } else if centre[2] > max[2] - 1.0 {
            max[2] = max[2].min(centre[2] - half[2]);
        } else {
            min[2] = min[2].max(centre[2] + half[2]);
        }
    }
    min[1] = metres(REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES);
    max[1] += 10.0;
    FluidBox { min, max }
}

/// Plan 25 emission: the particles a stage frame brings into the fluid,
/// positions in metres and velocities in metres per second. Pure.
pub(crate) fn emission_for_frame(
    frame: &WaterPresentationFrameV1,
    fluid_box: &FluidBox,
    level_metres: f32,
    room: usize,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let mut positions = Vec::new();
    let mut velocities = Vec::new();
    let metres = |value: i64| value as f32 / 1_000_000.0;
    let launch =
        metres(WATER_SPLASH_LAUNCH_SPEED_MICROMETRES_PER_SECOND) * std::f32::consts::FRAC_1_SQRT_2;
    // Floating boxes: plan 17's rule on the waterline perimeter.
    for floating in &frame.boxes {
        let speed = floating
            .vertical_velocity_micrometres_per_second
            .abs()
            .saturating_sub(WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND);
        if speed <= 0 {
            continue;
        }
        let count = (speed / WATER_SPLASH_SPEED_PER_DROPLET_MICROMETRES_PER_SECOND)
            .clamp(0, i64::from(WATER_SPLASH_MAX_PER_BOX)) as usize;
        let [x0, _, z0] = floating.minimum_micrometres.map(metres);
        let [x1, _, z1] = floating.maximum_micrometres.map(metres);
        if !fluid_box.contains_plan((x0 + x1) * 0.5, (z0 + z1) * 0.5) {
            continue;
        }
        let width = (x1 - x0).max(0.001);
        let depth = (z1 - z0).max(0.001);
        let perimeter = 2.0 * (width + depth);
        for k in 0..count {
            if positions.len() >= room {
                return (positions, velocities);
            }
            let along = perimeter * k as f32 / count as f32;
            let (x, z, out_x, out_z) = if along < width {
                (x0 + along, z0, 0.0, -1.0)
            } else if along < width + depth {
                (x1, z0 + (along - width), 1.0, 0.0)
            } else if along < 2.0 * width + depth {
                (x1 - (along - width - depth), z1, 0.0, 1.0)
            } else {
                (x0, z1 - (along - 2.0 * width - depth), -1.0, 0.0)
            };
            positions.push([
                x + out_x * LANE_SPACING_METRES,
                level_metres + LANE_SPACING_METRES,
                z + out_z * LANE_SPACING_METRES,
            ]);
            velocities.push([out_x * launch, launch, out_z * launch]);
        }
    }
    // Jets and falls: plan 21's rule at the crest, inside the box only.
    let gravity = metres(WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
    for record in &frame.edges {
        if !matches!(
            record.kind,
            WaterEdgePresentationKindV1::Jet | WaterEdgePresentationKindV1::Fall
        ) {
            continue;
        }
        let crest = record.crest_micrometres.map(metres);
        if !fluid_box.contains_plan(crest[0], crest[2]) {
            continue;
        }
        let head = metres(
            record
                .source_level_micrometres
                .saturating_sub(record.crest_micrometres[1]),
        )
        .max(0.0);
        if head <= 0.0 {
            continue;
        }
        let speed = (2.0 * gravity * head).sqrt();
        let count = (record.flux_cubic_millimetres.abs()
            / WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES)
            .clamp(0, i64::from(WATER_JET_MAX_SPAWN_PER_FRAME)) as usize;
        let direction = [
            record.direction_q15[0] as f32 / 32_767.0,
            record.direction_q15[1] as f32 / 32_767.0,
        ];
        for k in 0..count {
            if positions.len() >= room {
                return (positions, velocities);
            }
            let lateral = ((k as i64 * 7_919) % 41 - 20) as f32 * 0.002;
            positions.push([
                crest[0] - direction[1] * lateral,
                crest[1],
                crest[2] + direction[0].abs() * lateral,
            ]);
            velocities.push([direction[0] * speed, 0.0, direction[1] * speed]);
        }
    }
    (positions, velocities)
}

/// Plan 25 absorption: keeps the particles still in flight; a particle
/// within the band above the level moving slower than the threshold, or
/// below the floor, has returned to the exact water. Pure. Returns the kept
/// set and the number absorbed.
pub(crate) fn absorb(
    sample: &FluidSample,
    level_metres: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, usize) {
    let mut positions = Vec::with_capacity(sample.positions.len());
    let mut velocities = Vec::with_capacity(sample.positions.len());
    let mut absorbed = 0;
    for (position, velocity) in sample.positions.iter().zip(&sample.velocities) {
        if !position.iter().all(|value| value.is_finite())
            || !velocity.iter().all(|value| value.is_finite())
        {
            absorbed += 1;
            continue;
        }
        let speed =
            (velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2])
                .sqrt();
        let height = position[1] - level_metres;
        if height < 0.0 || (height < ABSORB_BAND_METRES && speed < ABSORB_SPEED_METRES_PER_SECOND) {
            absorbed += 1;
            continue;
        }
        positions.push(*position);
        velocities.push(*velocity);
    }
    (positions, velocities, absorbed)
}

/// Plan 26: the colliders a stage frame places in the fluid: one per
/// floating box whose plan centre lies inside the fluid box, as `(centre,
/// half extents)` in metres. Pure.
pub(crate) fn colliders_for_frame(
    frame: &WaterPresentationFrameV1,
    fluid_box: &FluidBox,
) -> Vec<([f32; 3], [f32; 3])> {
    let metres = |value: i64| value as f32 / 1_000_000.0;
    frame
        .boxes
        .iter()
        .filter_map(|floating| {
            let minimum = floating.minimum_micrometres.map(metres);
            let maximum = floating.maximum_micrometres.map(metres);
            let centre = [
                (minimum[0] + maximum[0]) * 0.5,
                (minimum[1] + maximum[1]) * 0.5,
                (minimum[2] + maximum[2]) * 0.5,
            ];
            let half = [
                (maximum[0] - minimum[0]) * 0.5,
                (maximum[1] - minimum[1]) * 0.5,
                (maximum[2] - minimum[2]) * 0.5,
            ];
            (fluid_box.contains_plan(centre[0], centre[2]) && half.iter().all(|value| *value > 0.0))
                .then_some((centre, half))
        })
        .take(16)
        .collect()
}

/// Plan 26: particles strictly inside a collider shrunk by one spacing.
pub(crate) fn particles_inside(
    positions: &[[f32; 3]],
    colliders: &[([f32; 3], [f32; 3])],
) -> usize {
    positions
        .iter()
        .filter(|position| {
            colliders.iter().any(|(centre, half)| {
                (0..3).all(|axis| {
                    (position[axis] - centre[axis]).abs() < half[axis] - LANE_SPACING_METRES
                })
            })
        })
        .count()
}

/// Plan 27: neighbour counts within `radius` (capped at `255`) and the
/// sizes of the connected components over the same links (capped at
/// `65,535`), through a dense uniform grid of cell `radius` over the
/// particles' bounds (linked cells, the forward half of the `27`
/// neighbourhood so every pair is visited once). Pure.
pub(crate) fn neighbour_counts_and_clusters(
    positions: &[[f32; 3]],
    radius: f32,
) -> (Vec<u8>, Vec<u16>) {
    const NONE: u32 = u32::MAX;
    let count = positions.len();
    let mut counts = vec![0_u32; count];
    let mut parent: Vec<u32> = (0..count as u32).collect();
    fn find(parent: &mut [u32], mut index: u32) -> u32 {
        while parent[index as usize] != index {
            parent[index as usize] = parent[parent[index as usize] as usize];
            index = parent[index as usize];
        }
        index
    }
    if count > 0 {
        let mut minimum = [f32::INFINITY; 3];
        for position in positions {
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(position[axis]);
            }
        }
        let mut dims = [1_usize; 3];
        for position in positions {
            for axis in 0..3 {
                let cell = ((position[axis] - minimum[axis]) / radius) as usize + 1;
                dims[axis] = dims[axis].max(cell + 1);
            }
        }
        let cell_of = |position: &[f32; 3]| -> [usize; 3] {
            [
                ((position[0] - minimum[0]) / radius) as usize,
                ((position[1] - minimum[1]) / radius) as usize,
                ((position[2] - minimum[2]) / radius) as usize,
            ]
        };
        let index_of = |cell: [usize; 3]| (cell[2] * dims[1] + cell[1]) * dims[0] + cell[0];
        let mut heads = vec![NONE; dims[0] * dims[1] * dims[2]];
        let mut next = vec![NONE; count];
        for (index, position) in positions.iter().enumerate() {
            let slot = index_of(cell_of(position));
            next[index] = heads[slot];
            heads[slot] = index as u32;
        }
        // The forward half of the neighbourhood: cells after this one in
        // scan order, plus this cell with `other > index`.
        const FORWARD: [[isize; 3]; 13] = [
            [1, 0, 0],
            [-1, 1, 0],
            [0, 1, 0],
            [1, 1, 0],
            [-1, -1, 1],
            [0, -1, 1],
            [1, -1, 1],
            [-1, 0, 1],
            [0, 0, 1],
            [1, 0, 1],
            [-1, 1, 1],
            [0, 1, 1],
            [1, 1, 1],
        ];
        let radius_squared = radius * radius;
        let link = |a: usize, b: usize, counts: &mut [u32], parent: &mut [u32]| {
            let delta = [
                positions[b][0] - positions[a][0],
                positions[b][1] - positions[a][1],
                positions[b][2] - positions[a][2],
            ];
            if delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2] <= radius_squared {
                counts[a] += 1;
                counts[b] += 1;
                let root_a = find(parent, a as u32);
                let root_b = find(parent, b as u32);
                if root_a != root_b {
                    parent[root_a as usize] = root_b;
                }
            }
        };
        for (index, position) in positions.iter().enumerate() {
            let cell = cell_of(position);
            let mut other = next[index];
            while other != NONE {
                link(index, other as usize, &mut counts, &mut parent);
                other = next[other as usize];
            }
            for offset in FORWARD {
                let neighbour = [
                    cell[0] as isize + offset[0],
                    cell[1] as isize + offset[1],
                    cell[2] as isize + offset[2],
                ];
                if neighbour[0] < 0
                    || neighbour[1] < 0
                    || neighbour[2] < 0
                    || neighbour[0] as usize >= dims[0]
                    || neighbour[1] as usize >= dims[1]
                    || neighbour[2] as usize >= dims[2]
                {
                    continue;
                }
                let mut other = heads[index_of([
                    neighbour[0] as usize,
                    neighbour[1] as usize,
                    neighbour[2] as usize,
                ])];
                while other != NONE {
                    link(index, other as usize, &mut counts, &mut parent);
                    other = next[other as usize];
                }
            }
        }
    }
    let mut sizes = vec![0_u32; count];
    let roots: Vec<u32> = (0..count as u32)
        .map(|index| find(&mut parent, index))
        .collect();
    for root in &roots {
        sizes[*root as usize] += 1;
    }
    (
        counts
            .iter()
            .map(|value| u8::try_from(*value).unwrap_or(u8::MAX))
            .collect(),
        roots
            .iter()
            .map(|root| u16::try_from(sizes[*root as usize]).unwrap_or(u16::MAX))
            .collect(),
    )
}

/// Plan 25 statistics, printed at session end.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LaneStats {
    pub(crate) frames: u64,
    pub(crate) peak_particles: usize,
    pub(crate) emitted: u64,
    pub(crate) absorbed: u64,
    pub(crate) last_particles: usize,
    pub(crate) cost_total_microseconds: u128,
    pub(crate) cost_max_microseconds: u128,
    /// Plan 26: the largest count of particles inside a collider.
    pub(crate) inside_colliders_max: usize,
    /// Plan 27: the host analysis per frame and the spray fraction.
    pub(crate) analysis_total_microseconds: u128,
    pub(crate) analysis_max_microseconds: u128,
    pub(crate) spray_fraction_max_permille: u32,
    pub(crate) spray_fraction_last_permille: u32,
}

pub(crate) struct PhysxWaterLane {
    fluid: NativeFluid,
    fluid_box: FluidBox,
    level_metres: f32,
    accumulator: Duration,
    bounds: AabbI64V1,
    last_frame_index: Option<u64>,
    pending_positions: Vec<[f32; 3]>,
    pending_velocities: Vec<[f32; 3]>,
    colliders: Vec<([f32; 3], [f32; 3])>,
    collider_slots: u32,
    stats: LaneStats,
}

impl PhysxWaterLane {
    /// Creates the empty fluid over the basin (plus plan 24's block when
    /// `pour` is set); `Err` carries the reason the lane is unavailable.
    pub(crate) fn new(bounds: AabbI64V1, pour: bool) -> Result<Self, String> {
        let fluid_box = basin_fluid_box();
        let level_metres = fluid_box.min[1];
        let centre_x = (fluid_box.min[0] + fluid_box.max[0]) * 0.5;
        let centre_z = (fluid_box.min[2] + fluid_box.max[2]) * 0.5;
        let (seed_min, seed_max) = if pour {
            (
                [
                    centre_x - POUR_BLOCK_HALF_METRES,
                    level_metres + POUR_BLOCK_BOTTOM_METRES,
                    centre_z - POUR_BLOCK_HALF_METRES,
                ],
                [
                    centre_x + POUR_BLOCK_HALF_METRES,
                    level_metres + POUR_BLOCK_BOTTOM_METRES + POUR_BLOCK_HEIGHT_METRES,
                    centre_z + POUR_BLOCK_HALF_METRES,
                ],
            )
        } else {
            (
                [centre_x, level_metres, centre_z],
                [centre_x, level_metres, centre_z],
            )
        };
        let path = gpu_library_path().ok_or_else(|| "PhysX GPU library not found".to_owned())?;
        let fluid = NativeFluid::create_reporting(&FluidDesc {
            spacing_metres: LANE_SPACING_METRES,
            box_min_metres: fluid_box.min,
            box_max_metres: fluid_box.max,
            seed_min_metres: seed_min,
            seed_max_metres: seed_max,
            timestep_seconds: LANE_TIMESTEP.as_secs_f32(),
            max_particles: LANE_MAX_PARTICLES,
            gpu_library_path: Some(path.display().to_string()),
        })
        .map_err(|(error, reason)| format!("PhysX fluid unavailable: {error} ({reason})"))?;
        Ok(Self {
            fluid,
            fluid_box,
            level_metres,
            accumulator: Duration::ZERO,
            bounds,
            last_frame_index: None,
            pending_positions: Vec::new(),
            pending_velocities: Vec::new(),
            colliders: Vec::new(),
            collider_slots: 0,
            stats: LaneStats::default(),
        })
    }

    pub(crate) fn particle_surface_profile(&self) -> ParticleSurfaceProfileV1 {
        ParticleSurfaceProfileV1 {
            particle_capacity: LANE_MAX_PARTICLES,
            radius_micrometres: LANE_RADIUS_MICROMETRES,
            bounds: self.bounds,
            absorption_per_metre: [1.2, 0.5, 0.25],
            refraction_strength: 0.08,
            thickness_scale: 1.0,
            spray_neighbour_threshold: SPRAY_NEIGHBOUR_THRESHOLD,
            spray_radius_micrometres: SPRAY_RADIUS_MICROMETRES,
            spray_alpha: SPRAY_ALPHA,
            spray_cluster_threshold: SPRAY_CLUSTER_THRESHOLD,
            spray_subdroplets: SPRAY_SUBDROPLETS,
            spray_streak_seconds: SPRAY_STREAK_SECONDS,
            bulk_neighbour_count: BULK_NEIGHBOUR_COUNT,
            edge_radius_scale: EDGE_RADIUS_SCALE,
            cleanup_radius_pixels: 4,
        }
    }

    /// Queues the emission of a newly published stage frame.
    pub(crate) fn observe_frame(&mut self, frame: &WaterPresentationFrameV1) {
        if self.last_frame_index == Some(frame.frame_index) {
            return;
        }
        self.last_frame_index = Some(frame.frame_index);
        let room = LANE_MAX_PARTICLES as usize;
        let (positions, velocities) =
            emission_for_frame(frame, &self.fluid_box, self.level_metres, room);
        self.pending_positions.extend(positions);
        self.pending_velocities.extend(velocities);
        // Plan 26: the frame's boxes as kinematic colliders; a vanished box
        // clears its slot. A failed collider update is reported once by the
        // adapter's client error at the next step.
        self.colliders = colliders_for_frame(frame, &self.fluid_box);
        let used = u32::try_from(self.colliders.len()).unwrap_or(0);
        for (slot, (centre, half)) in self.colliders.iter().enumerate() {
            let slot = u32::try_from(slot).unwrap_or(u32::MAX);
            if self.fluid.set_box(slot, *centre, *half).is_err() {
                break;
            }
        }
        for slot in used..self.collider_slots {
            let _ = self.fluid.clear_box(slot);
        }
        self.collider_slots = used;
    }

    /// Steps the fluid by the frame's elapsed time, absorbs settled
    /// particles, injects the pending emission and returns the particle
    /// update for this frame.
    pub(crate) fn advance(
        &mut self,
        elapsed: Duration,
        sequence: u64,
    ) -> Result<Option<Arc<ParticleSurfaceUpdateV1>>, DesktopAdapterError> {
        let started = Instant::now();
        self.accumulator = self.accumulator.saturating_add(elapsed);
        let mut steps = 0;
        while self.accumulator >= LANE_TIMESTEP && steps < LANE_MAX_STEPS_PER_FRAME {
            self.accumulator -= LANE_TIMESTEP;
            self.fluid.step().map_err(|error| {
                DesktopAdapterError::client("PHYSX_WATER_STEP_FAILED", error.to_string())
            })?;
            steps += 1;
        }
        if steps == LANE_MAX_STEPS_PER_FRAME {
            self.accumulator = Duration::ZERO;
        }
        let sample = self.fluid.read().map_err(|error| {
            DesktopAdapterError::client("PHYSX_WATER_READ_FAILED", error.to_string())
        })?;
        let (mut positions, mut velocities, absorbed) = absorb(&sample, self.level_metres);
        let room = (self.fluid.max_particles() as usize).saturating_sub(positions.len());
        let emitted = self.pending_positions.len().min(room);
        positions.extend(self.pending_positions.drain(..emitted));
        velocities.extend(self.pending_velocities.drain(..emitted));
        self.pending_positions.clear();
        self.pending_velocities.clear();
        let changed = absorbed > 0 || emitted > 0;
        if changed {
            self.fluid.set(&positions, &velocities).map_err(|error| {
                DesktopAdapterError::client("PHYSX_WATER_SET_FAILED", error.to_string())
            })?;
        }
        self.stats.frames += 1;
        self.stats.emitted += emitted as u64;
        self.stats.absorbed += absorbed as u64;
        self.stats.peak_particles = self.stats.peak_particles.max(positions.len());
        self.stats.last_particles = positions.len();
        self.stats.inside_colliders_max = self
            .stats
            .inside_colliders_max
            .max(particles_inside(&positions, &self.colliders));
        let cost = started.elapsed().as_micros();
        self.stats.cost_total_microseconds += cost;
        self.stats.cost_max_microseconds = self.stats.cost_max_microseconds.max(cost);
        // Droplets that left the declared bounds (the pass rejects any
        // position outside them, exclusive at the maximum) are dropped for
        // this frame's picture.
        // Plan 27: neighbours and clusters from the fluid's own density.
        let analysis_started = Instant::now();
        let (neighbours, clusters) =
            neighbour_counts_and_clusters(&positions, NEIGHBOUR_RADIUS_METRES);
        let analysis_cost = analysis_started.elapsed().as_micros();
        self.stats.analysis_total_microseconds += analysis_cost;
        self.stats.analysis_max_microseconds =
            self.stats.analysis_max_microseconds.max(analysis_cost);
        if !positions.is_empty() {
            let spray = neighbours
                .iter()
                .zip(&clusters)
                .filter(|(count, cluster)| {
                    u32::from(**count) < SPRAY_NEIGHBOUR_THRESHOLD
                        || u32::from(**cluster) < SPRAY_CLUSTER_THRESHOLD
                })
                .count();
            let permille = u32::try_from(spray * 1_000 / positions.len()).unwrap_or(u32::MAX);
            self.stats.spray_fraction_last_permille = permille;
            self.stats.spray_fraction_max_permille =
                self.stats.spray_fraction_max_permille.max(permille);
        }
        let micrometres = |value: f32| (value * 1_000_000.0).round() as i64;
        let mut update_positions = Vec::with_capacity(positions.len());
        let mut update_velocities = Vec::with_capacity(positions.len());
        let mut update_neighbours = Vec::with_capacity(positions.len());
        let mut update_clusters = Vec::with_capacity(positions.len());
        for (index, (position, velocity)) in positions.iter().zip(&velocities).enumerate() {
            let point = position.map(micrometres);
            if !self.bounds.contains(point) {
                continue;
            }
            update_positions.push(point);
            update_velocities
                .push(velocity.map(|value| (value * 1_000_000.0).clamp(-2.0e9, 2.0e9) as i32));
            update_neighbours.push(neighbours[index]);
            update_clusters.push(clusters[index]);
        }
        if update_positions.is_empty() {
            return Ok(None);
        }
        Ok(Some(Arc::new(ParticleSurfaceUpdateV1::new(
            sequence,
            update_positions,
            update_neighbours,
            update_clusters,
            update_velocities,
            Vec::new(),
        )?)))
    }

    pub(crate) fn stats(&self) -> &LaneStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_reference_game::{WaterEdgePresentationV1, WaterFloatingBoxV1, WaterJetParticlesV1};

    fn frame(
        boxes: Vec<WaterFloatingBoxV1>,
        edges: Vec<WaterEdgePresentationV1>,
    ) -> WaterPresentationFrameV1 {
        WaterPresentationFrameV1 {
            tick: 1,
            frame_index: 1,
            surfaces: Vec::new(),
            jet: WaterJetParticlesV1::default(),
            edges,
            boxes,
        }
    }

    fn crate_box(vertical_velocity: i64) -> WaterFloatingBoxV1 {
        WaterFloatingBoxV1 {
            minimum_micrometres: [6_250_000, 300_000, 1_750_000],
            maximum_micrometres: [6_750_000, 800_000, 2_250_000],
            vertical_velocity_micrometres_per_second: vertical_velocity,
        }
    }

    #[test]
    fn emission_follows_the_stage_rules_inside_the_box() {
        let fluid_box = basin_fluid_box();
        let level = fluid_box.min[1];
        // Plan 17: 1 m/s above the 0.3 m/s threshold, one droplet per 0.02 m/s.
        let (positions, velocities) = emission_for_frame(
            &frame(vec![crate_box(1_000_000)], Vec::new()),
            &fluid_box,
            level,
            16_384,
        );
        assert_eq!(positions.len(), 35);
        assert!(velocities.iter().all(|velocity| velocity[1] > 0.0));
        assert!(
            positions
                .iter()
                .all(|position| fluid_box.contains_plan(position[0], position[2]))
        );
        assert!(
            emission_for_frame(
                &frame(vec![crate_box(0)], Vec::new()),
                &fluid_box,
                level,
                16_384
            )
            .0
            .is_empty()
        );
        assert!(
            emission_for_frame(&frame(Vec::new(), Vec::new()), &fluid_box, level, 16_384)
                .0
                .is_empty()
        );
        let fall = |crest_x: i64| WaterEdgePresentationV1 {
            edge_id: next_contracts::ids::PersistentId::from_bytes([0x11; 16]),
            kind: WaterEdgePresentationKindV1::Fall,
            crest_micrometres: [crest_x, 500_000, 2_000_000],
            direction_q15: [32_767, 0],
            source_level_micrometres: 700_000,
            sink_level_micrometres: 300_000,
            flux_cubic_millimetres: 5_000_000,
        };
        let inside = emission_for_frame(
            &frame(Vec::new(), vec![fall(6_500_000)]),
            &fluid_box,
            level,
            16_384,
        );
        assert_eq!(inside.0.len(), 10);
        assert!(inside.1.iter().all(|velocity| velocity[0] > 0.0));
        let outside = emission_for_frame(
            &frame(Vec::new(), vec![fall(16_000_000)]),
            &fluid_box,
            level,
            16_384,
        );
        assert!(outside.0.is_empty());
        let capped = emission_for_frame(
            &frame(vec![crate_box(1_000_000)], Vec::new()),
            &fluid_box,
            level,
            10,
        );
        assert_eq!(capped.0.len(), 10);
    }

    #[test]
    fn colliders_follow_the_frame_boxes_inside_the_fluid_box() {
        let fluid_box = basin_fluid_box();
        let inside = colliders_for_frame(&frame(vec![crate_box(0)], Vec::new()), &fluid_box);
        assert_eq!(inside.len(), 1);
        assert_eq!(inside[0].0, [6.5, 0.55, 2.0]);
        assert_eq!(inside[0].1, [0.25, 0.25, 0.25]);
        let mut far = crate_box(0);
        far.minimum_micrometres[0] += 10_000_000;
        far.maximum_micrometres[0] += 10_000_000;
        assert!(colliders_for_frame(&frame(vec![far], Vec::new()), &fluid_box).is_empty());
        assert!(colliders_for_frame(&frame(Vec::new(), Vec::new()), &fluid_box).is_empty());
        // Inside means strictly inside the box shrunk by one spacing.
        let positions = [[6.5, 0.55, 2.0], [6.5, 0.77, 2.0], [7.0, 0.55, 2.0]];
        assert_eq!(particles_inside(&positions, &inside), 1);
    }

    #[test]
    fn neighbour_grid_matches_brute_force_and_clusters_are_consistent() {
        // A deterministic pseudo-random cloud in a 1 m cube plus one
        // isolated particle far away.
        let mut state = 0x9e37_79b9_u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state % 10_000) as f32 / 10_000.0
        };
        let mut positions: Vec<[f32; 3]> = (0..2_000).map(|_| [next(), next(), next()]).collect();
        positions.push([10.0, 10.0, 10.0]);
        let radius = NEIGHBOUR_RADIUS_METRES;
        let (counts, clusters) = neighbour_counts_and_clusters(&positions, radius);
        let count = positions.len();
        let mut brute = vec![0_u32; count];
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); count];
        for a in 0..count {
            for b in (a + 1)..count {
                let d: f32 = (0..3)
                    .map(|axis| (positions[a][axis] - positions[b][axis]).powi(2))
                    .sum();
                if d <= radius * radius {
                    brute[a] += 1;
                    brute[b] += 1;
                    adjacency[a].push(b);
                    adjacency[b].push(a);
                }
            }
        }
        for index in 0..count {
            assert_eq!(
                u32::from(counts[index]),
                brute[index].min(255),
                "particle {index}"
            );
        }
        let mut component = vec![usize::MAX; count];
        let mut sizes = Vec::new();
        for start in 0..count {
            if component[start] != usize::MAX {
                continue;
            }
            let id = sizes.len();
            let mut stack = vec![start];
            let mut size = 0;
            component[start] = id;
            while let Some(index) = stack.pop() {
                size += 1;
                for &other in &adjacency[index] {
                    if component[other] == usize::MAX {
                        component[other] = id;
                        stack.push(other);
                    }
                }
            }
            sizes.push(size);
        }
        for index in 0..count {
            assert_eq!(
                usize::from(clusters[index]),
                sizes[component[index]].min(65_535)
            );
        }
        assert_eq!(counts[count - 1], 0);
        assert_eq!(clusters[count - 1], 1);
    }

    #[test]
    fn absorption_keeps_flying_particles_and_removes_settled_ones() {
        let level = 0.5;
        let sample = FluidSample {
            positions: vec![
                [6.5, 0.55, 2.0],
                [6.5, 1.0, 2.0],
                [6.5, 0.4, 2.0],
                [6.5, 0.55, 2.0],
            ],
            velocities: vec![
                [0.0, -0.1, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
        };
        let (positions, velocities, absorbed) = absorb(&sample, level);
        assert_eq!(absorbed, 2, "the settled one and the one below the floor");
        assert_eq!(positions.len(), 2);
        assert_eq!(velocities.len(), 2);
        assert_eq!(positions[0], [6.5, 1.0, 2.0], "still falling, kept");
        assert_eq!(positions[1], [6.5, 0.55, 2.0], "fast at the level, kept");
    }
}
