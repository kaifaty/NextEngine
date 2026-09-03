//! Presentation-only water stage (plan `continuum-water/09`, ADR-100/101/102,
//! SPEC-38 2.2 practices 3, 5 and 6).
//!
//! A pure function of the committed physics checkpoint at the published
//! tick and of a presentation frame index: for every water volume with an
//! authored surface quad it produces a `32 x 16` height grid whose height is
//! a bounded ripple layer (the exact level is the binding's translation),
//! and for every gate or pipe with positive flux a ballistic jet of
//! droplets. Every value is an integer; nothing here is read by gameplay,
//! saved or replayed.

use next_contracts::ids::{AssetId, PersistentId};
use next_contracts::physics::{
    WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED, WaterFlowEdgeKindV1, WaterFlowNetworkV1,
    WaterVolumeDefinitionV1, WaterVolumeSetV1, isqrt_i128,
};

/// Presentation frames per second the stage integrates its jet at.
pub const WATER_PRESENTATION_FRAMES_PER_SECOND: u32 = 60;
/// Height grid of one surface quad.
pub const WATER_SURFACE_GRID_COLUMNS: u32 = 32;
pub const WATER_SURFACE_GRID_ROWS: u32 = 16;
/// Declared ring capacities for one surface quad.
pub const WATER_SURFACE_VERTEX_CAPACITY: u32 = WATER_SURFACE_GRID_COLUMNS * WATER_SURFACE_GRID_ROWS;
pub const WATER_SURFACE_INDEX_CAPACITY: u32 =
    (WATER_SURFACE_GRID_COLUMNS - 1) * (WATER_SURFACE_GRID_ROWS - 1) * 6;
/// Ripple amplitude cap; the authored quad bounds admit exactly this.
pub const WATER_RIPPLE_CAP_MICROMETRES: i64 = 20_000;
/// Incoming or outgoing flux per tick that reaches the amplitude cap.
const WATER_RIPPLE_FULL_FLUX_CUBIC_MILLIMETRES: i64 = 1_000_000;

/// One world-space directional wave of the ambient spectrum (plan
/// `continuum-water/14`, WL4): the phase advances by
/// `phase_per_frame_turns_q16` per presentation frame (deep-water
/// dispersion at `g = 9.81`).
#[derive(Clone, Copy, Debug)]
struct AmbientWaveV1 {
    wavelength_micrometres: i64,
    direction_q15: [i64; 2],
    phase_per_frame_turns_q16: i64,
    amplitude_micrometres: i64,
}

/// The frozen ambient spectrum of WL4 (always present, `4 mm` in total at
/// most).
const AMBIENT_WAVES: [AmbientWaveV1; 4] = [
    // lambda 1.6 m, period 1.01 s
    AmbientWaveV1 {
        wavelength_micrometres: 1600000,
        direction_q15: [30791, 11207],
        phase_per_frame_turns_q16: 1079,
        amplitude_micrometres: 2000,
    },
    // lambda 0.9 m, period 0.76 s
    AmbientWaveV1 {
        wavelength_micrometres: 900000,
        direction_q15: [-11207, 30791],
        phase_per_frame_turns_q16: 1439,
        amplitude_micrometres: 1200,
    },
    // lambda 0.5 m, period 0.57 s
    AmbientWaveV1 {
        wavelength_micrometres: 500000,
        direction_q15: [-30791, -11207],
        phase_per_frame_turns_q16: 1930,
        amplitude_micrometres: 500,
    },
    // lambda 0.3 m, period 0.44 s
    AmbientWaveV1 {
        wavelength_micrometres: 300000,
        direction_q15: [18794, -26841],
        phase_per_frame_turns_q16: 2492,
        amplitude_micrometres: 300,
    },
];

/// Height of the ambient spectrum at a world point and frame.
fn ambient_height(x_micrometres: i64, z_micrometres: i64, frame_index: u64) -> i64 {
    let frame = i64::try_from(frame_index % 65_536).unwrap_or(0);
    let mut height = 0_i64;
    for wave in &AMBIENT_WAVES {
        // Positions are bounded by the water position limit (`i64`
        // micrometres well below 2^40), so the products fit `i64`.
        let along = (wave.direction_q15[0] * x_micrometres + wave.direction_q15[1] * z_micrometres)
            / 32_767;
        let turns = along * 65_536 / wave.wavelength_micrometres;
        let angle = turns + frame * wave.phase_per_frame_turns_q16;
        height += wave.amplitude_micrometres * sin_q15(angle) / 32_767;
    }
    height
}
/// Jet: one droplet per this volume of flux per tick, bounded per frame.
pub const WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES: i64 = 500_000;
pub const WATER_JET_MAX_SPAWN_PER_FRAME: u32 = 256;
pub const WATER_JET_MAX_PARTICLES: u32 = 4_096;
pub const WATER_JET_LIFETIME_FRAMES: u32 = 120;
/// Droplet render radius for the particle profile.
pub const WATER_JET_RADIUS_MICROMETRES: u32 = 30_000;

/// Plan 17 (L7): depression of the ring at the edge of a floating box.
pub const WATER_WAKE_DEPTH_MICROMETRES: i64 = 8_000;
/// Plan 17 (L7): the depression falls to zero over this distance.
pub const WATER_WAKE_FALLOFF_MICROMETRES: i64 = 350_000;
/// Plan 17 (L7): a box sheds droplets above this vertical speed.
pub const WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND: i64 = 300_000;
/// Plan 17 (L7): one droplet per this much vertical speed above the threshold.
pub const WATER_SPLASH_SPEED_PER_DROPLET_MICROMETRES_PER_SECOND: i64 = 20_000;
/// Plan 17 (L7): droplets per box per frame at most.
pub const WATER_SPLASH_MAX_PER_BOX: u32 = 64;
/// Plan 17 (L7): launch speed of a splash droplet, outward and up.
pub const WATER_SPLASH_LAUNCH_SPEED_MICROMETRES_PER_SECOND: i64 = 800_000;
/// Plan 17 (L7): lifetime of a splash droplet in presentation frames.
pub const WATER_SPLASH_LIFETIME_FRAMES: u32 = 60;

/// Plan 17: one committed dynamic box of the checkpoint (the union of its
/// box shapes at the committed translation, the ADR-105 bounds rule) and
/// its vertical velocity; the stage's wake and splash input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterFloatingBoxV1 {
    pub minimum_micrometres: [i64; 3],
    pub maximum_micrometres: [i64; 3],
    pub vertical_velocity_micrometres_per_second: i64,
}

/// The floating-box inputs of the stage from a committed checkpoint: every
/// active `Dynamic` body with box shapes, in body id order.
#[must_use]
pub fn floating_boxes(
    checkpoint: &next_contracts::physics::PhysicsWorldCheckpointV1,
) -> Vec<WaterFloatingBoxV1> {
    let mut boxes = Vec::new();
    for (body_id, body) in &checkpoint.catalog.bodies {
        if body.motion_kind != next_contracts::physics::PhysicsMotionKindV1::Dynamic {
            continue;
        }
        let Some(state) = checkpoint.snapshot.sorted_body_states.get(body_id) else {
            continue;
        };
        if !state.active {
            continue;
        }
        let translation = state.pose.translation_micrometres;
        let mut bounds: Option<([i64; 3], [i64; 3])> = None;
        for shape in body.shapes.values() {
            let next_contracts::physics::PhysicsGeometryV1::Box {
                half_extents_micrometres,
            } = shape.geometry
            else {
                continue;
            };
            let mut minimum = [0_i64; 3];
            let mut maximum = [0_i64; 3];
            for axis in 0..3 {
                let centre = translation[axis]
                    .saturating_add(shape.local_pose.translation_micrometres[axis]);
                minimum[axis] = centre.saturating_sub(half_extents_micrometres[axis]);
                maximum[axis] = centre.saturating_add(half_extents_micrometres[axis]);
            }
            bounds = Some(match bounds {
                None => (minimum, maximum),
                Some((low, high)) => (
                    [
                        low[0].min(minimum[0]),
                        low[1].min(minimum[1]),
                        low[2].min(minimum[2]),
                    ],
                    [
                        high[0].max(maximum[0]),
                        high[1].max(maximum[1]),
                        high[2].max(maximum[2]),
                    ],
                ),
            });
        }
        if let Some((minimum_micrometres, maximum_micrometres)) = bounds {
            boxes.push(WaterFloatingBoxV1 {
                minimum_micrometres,
                maximum_micrometres,
                vertical_velocity_micrometres_per_second: state
                    .linear_velocity_micrometres_per_second[1],
            });
        }
    }
    boxes
}

/// One surface quad update in the quad's local space (the binding adds the
/// exact level): a `columns x rows` grid over the volume's plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterSurfaceUpdateV1 {
    pub volume_id: PersistentId,
    pub mesh_asset_id: AssetId,
    pub positions_micrometres: Vec<[i64; 3]>,
    pub normals_snorm16: Vec<[i16; 3]>,
    pub indices: Vec<u32>,
}

/// The jet droplets of one frame in world space.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WaterJetParticlesV1 {
    pub positions_micrometres: Vec<[i64; 3]>,
    pub velocities_micrometres_per_second: Vec<[i32; 3]>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WaterPresentationFrameV1 {
    pub tick: u64,
    pub frame_index: u64,
    pub surfaces: Vec<WaterSurfaceUpdateV1>,
    pub jet: WaterJetParticlesV1,
}

/// Which volumes carry an authored surface quad.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterSurfaceBindingV1 {
    pub volume_id: PersistentId,
    pub mesh_asset_id: AssetId,
}

/// The reference scene's surface quads.
#[must_use]
pub fn reference_water_surface_bindings() -> [WaterSurfaceBindingV1; 3] {
    [
        WaterSurfaceBindingV1 {
            volume_id: crate::water::REFERENCE_WATER_BASIN_ID,
            mesh_asset_id: crate::source::REFERENCE_WATER_SURFACE_MESH_ASSET_ID,
        },
        WaterSurfaceBindingV1 {
            volume_id: crate::water::REFERENCE_WATER_VESSEL_A_ID,
            mesh_asset_id: crate::source::REFERENCE_WATER_VESSEL_A_SURFACE_MESH_ASSET_ID,
        },
        WaterSurfaceBindingV1 {
            volume_id: crate::water::REFERENCE_WATER_VESSEL_B_ID,
            mesh_asset_id: crate::source::REFERENCE_WATER_VESSEL_B_SURFACE_MESH_ASSET_ID,
        },
    ]
}

/// Integer sine, Bhaskara I approximation: input angle in 1/65536 turns,
/// output in Q15.
#[must_use]
pub fn sin_q15(angle_turns_q16: i64) -> i64 {
    let half = 32_768_i64;
    let angle = angle_turns_q16.rem_euclid(65_536);
    let (x, sign) = if angle < half {
        (angle, 1)
    } else {
        (angle - half, -1)
    };
    // sin(pi x / half) ~ 16 x (half - x) / (5 half^2 - 4 x (half - x))
    let product = x * (half - x);
    let numerator = 16 * product;
    let denominator = 5 * half * half - 4 * product;
    sign * numerator * 32_767 / denominator
}

/// Computes the presentation frame for one committed water table and
/// network (the two water fields of the physics checkpoint).
pub fn compute_water_presentation_frame(
    volumes: &WaterVolumeSetV1,
    network: &WaterFlowNetworkV1,
    bindings: &[WaterSurfaceBindingV1],
    boxes: &[WaterFloatingBoxV1],
    tick: u64,
    frame_index: u64,
) -> WaterPresentationFrameV1 {
    let mut surfaces = Vec::with_capacity(bindings.len());
    for binding in bindings {
        let Some(definition) = volumes.definitions.get(&binding.volume_id) else {
            continue;
        };
        let amplitude = ripple_amplitude(network, binding.volume_id);
        // Plan 17: the boxes floating in this volume (plan overlap, bottom
        // below the level) depress the ring around their plan rectangle.
        let level = volumes.effective_level(binding.volume_id, tick);
        let wakes: Vec<&WaterFloatingBoxV1> = boxes
            .iter()
            .filter(|floating| level.is_some_and(|level| floats_in(floating, definition, level)))
            .collect();
        surfaces.push(surface_grid(
            binding,
            definition,
            amplitude,
            &wakes,
            frame_index,
        ));
    }
    let mut jet = jet_particles(volumes, network, tick, frame_index);
    splash_particles(volumes, boxes, tick, frame_index, &mut jet);
    WaterPresentationFrameV1 {
        tick,
        frame_index,
        surfaces,
        jet,
    }
}

fn floats_in(
    floating: &WaterFloatingBoxV1,
    definition: &WaterVolumeDefinitionV1,
    level: i64,
) -> bool {
    floating.minimum_micrometres[0] < definition.maximum_micrometres[0]
        && floating.maximum_micrometres[0] > definition.minimum_micrometres[0]
        && floating.minimum_micrometres[2] < definition.maximum_micrometres[2]
        && floating.maximum_micrometres[2] > definition.minimum_micrometres[2]
        && floating.minimum_micrometres[1] < level
}

/// Wake depression at `(x, z)`: full depth at the box's plan rectangle,
/// linear to zero over the falloff distance (Chebyshev distance).
fn wake_depth(x: i64, z: i64, wakes: &[&WaterFloatingBoxV1]) -> i64 {
    let mut depth = 0_i64;
    for floating in wakes {
        let dx = (floating.minimum_micrometres[0] - x).max(x - floating.maximum_micrometres[0]);
        let dz = (floating.minimum_micrometres[2] - z).max(z - floating.maximum_micrometres[2]);
        let distance = dx.max(dz).max(0);
        if distance >= WATER_WAKE_FALLOFF_MICROMETRES {
            continue;
        }
        depth = depth.max(
            WATER_WAKE_DEPTH_MICROMETRES * (WATER_WAKE_FALLOFF_MICROMETRES - distance)
                / WATER_WAKE_FALLOFF_MICROMETRES,
        );
    }
    depth
}

/// Plan 17 (L7): stateless splash droplets around the waterline of every
/// box whose vertical speed exceeds the threshold; the same age rule as
/// the jet, appended to the jet list inside its particle bound.
fn splash_particles(
    volumes: &WaterVolumeSetV1,
    boxes: &[WaterFloatingBoxV1],
    tick: u64,
    frame_index: u64,
    jet: &mut WaterJetParticlesV1,
) {
    let gravity = i128::from(WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
    let fps = i128::from(WATER_PRESENTATION_FRAMES_PER_SECOND);
    // 0.8 m/s split evenly between outward and up (cos 45 in q15).
    let launch = i128::from(WATER_SPLASH_LAUNCH_SPEED_MICROMETRES_PER_SECOND) * 23_170 / 32_767;
    for floating in boxes {
        let speed = floating
            .vertical_velocity_micrometres_per_second
            .abs()
            .saturating_sub(WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND);
        if speed <= 0 {
            continue;
        }
        let Some(level) = volumes
            .definitions
            .iter()
            .find_map(|(volume_id, definition)| {
                let level = volumes.effective_level(*volume_id, tick)?;
                floats_in(floating, definition, level).then_some(level)
            })
        else {
            continue;
        };
        let count = (speed / WATER_SPLASH_SPEED_PER_DROPLET_MICROMETRES_PER_SECOND)
            .clamp(0, i64::from(WATER_SPLASH_MAX_PER_BOX));
        let remaining = i64::from(WATER_JET_MAX_PARTICLES)
            - i64::try_from(jet.positions_micrometres.len()).unwrap_or(i64::MAX);
        let count = count.min(remaining.max(0));
        if count == 0 {
            continue;
        }
        let [x0, _, z0] = floating.minimum_micrometres;
        let [x1, _, z1] = floating.maximum_micrometres;
        let width = (x1 - x0).max(1);
        let depth = (z1 - z0).max(1);
        let perimeter = 2 * (width + depth);
        let lifetime = i128::from(WATER_SPLASH_LIFETIME_FRAMES);
        let stride = (lifetime * 65_536 / i128::from(count)).max(1);
        for k in 0..count {
            // Droplet `k` starts on the waterline at perimeter offset
            // `k * perimeter / count` and flies outward from that edge.
            let along = k * perimeter / count;
            let (start_x, start_z, out_x, out_z) = if along < width {
                (x0 + along, z0, 0, -1)
            } else if along < width + depth {
                (x1, z0 + (along - width), 1, 0)
            } else if along < 2 * width + depth {
                (x1 - (along - width - depth), z1, 0, 1)
            } else {
                (x0, z1 - (along - 2 * width - depth), -1, 0)
            };
            let age =
                (i128::from(frame_index) * 65_536 + i128::from(k) * stride) % (lifetime * 65_536);
            // t seconds = age / (65536 * fps)
            let travel = launch * age / (65_536 * fps);
            let rise = travel - gravity * age * age / (2 * 65_536 * 65_536 * fps * fps);
            if rise < 0 {
                continue;
            }
            let x = i128::from(start_x) + i128::from(out_x) * travel;
            let z = i128::from(start_z) + i128::from(out_z) * travel;
            let vy = launch - gravity * age / (65_536 * fps);
            jet.positions_micrometres.push([
                i64::try_from(x).unwrap_or(i64::MAX),
                i64::try_from(i128::from(level) + rise).unwrap_or(i64::MAX),
                i64::try_from(z).unwrap_or(i64::MAX),
            ]);
            jet.velocities_micrometres_per_second.push([
                i32::try_from(i128::from(out_x) * launch).unwrap_or(0),
                i32::try_from(vy.clamp(-2_000_000_000, 2_000_000_000)).unwrap_or(0),
                i32::try_from(i128::from(out_z) * launch).unwrap_or(0),
            ]);
        }
    }
}

/// Ripple amplitude of a volume from the largest incident edge flux of the
/// published tick, capped; still water is flat.
fn ripple_amplitude(
    network: &next_contracts::physics::WaterFlowNetworkV1,
    volume_id: PersistentId,
) -> i64 {
    let mut flux: i64 = 0;
    for (edge_id, edge) in &network.edges {
        if edge.cell_a != volume_id && edge.cell_b != Some(volume_id) {
            continue;
        }
        let Some(last) = network.edge_flux(*edge_id) else {
            continue;
        };
        flux = flux.max(last.abs());
    }
    (i128::from(flux) * i128::from(WATER_RIPPLE_CAP_MICROMETRES)
        / i128::from(WATER_RIPPLE_FULL_FLUX_CUBIC_MILLIMETRES))
    .min(i128::from(WATER_RIPPLE_CAP_MICROMETRES)) as i64
}

fn surface_grid(
    binding: &WaterSurfaceBindingV1,
    definition: &WaterVolumeDefinitionV1,
    amplitude: i64,
    wakes: &[&WaterFloatingBoxV1],
    frame_index: u64,
) -> WaterSurfaceUpdateV1 {
    let columns = WATER_SURFACE_GRID_COLUMNS as usize;
    let rows = WATER_SURFACE_GRID_ROWS as usize;
    let [x0, _, z0] = definition.minimum_micrometres;
    let [x1, _, z1] = definition.maximum_micrometres;
    let mut positions = Vec::with_capacity(columns * rows);
    let mut normals = Vec::with_capacity(columns * rows);
    let frame = i64::try_from(frame_index % 65_536).unwrap_or(0);
    let world_x = |column: usize| x0 + (x1 - x0) * column as i64 / (columns as i64 - 1);
    let world_z = |row: usize| z0 + (z1 - z0) * row as i64 / (rows as i64 - 1);
    let height_at = |column: usize, row: usize| -> i64 {
        let (x, z) = (world_x(column), world_z(row));
        // WL4: the ambient spectrum in world space plus the WP1 flux ripple
        // over the two shortest ambient waves at double frequency.
        let mut height = ambient_height(x, z, frame_index);
        if amplitude != 0 {
            let mut ripple = 0_i64;
            for wave in &AMBIENT_WAVES[2..] {
                let along = (wave.direction_q15[0] * x + wave.direction_q15[1] * z) / 32_767;
                let turns = along * 131_072 / wave.wavelength_micrometres;
                ripple += sin_q15(turns + frame * wave.phase_per_frame_turns_q16 * 2);
            }
            height += amplitude * ripple / (2 * 32_767);
        }
        if !wakes.is_empty() {
            height -= wake_depth(x, z, wakes);
        }
        // Below the cap: the catalog mesh bounds are exclusive at their
        // maximum, and the ripple cap is the authored bound.
        height.clamp(
            -WATER_RIPPLE_CAP_MICROMETRES,
            WATER_RIPPLE_CAP_MICROMETRES - 1,
        )
    };
    let mut heights = Vec::with_capacity(columns * rows);
    for row in 0..rows {
        for column in 0..columns {
            let height = height_at(column, row);
            heights.push(height);
            positions.push([world_x(column), height, world_z(row)]);
        }
    }
    let cell_x = ((x1 - x0) / (columns as i64 - 1)).max(1);
    let cell_z = ((z1 - z0) / (rows as i64 - 1)).max(1);
    let height_of = |column: usize, row: usize| heights[row * columns + column];
    for row in 0..rows {
        for column in 0..columns {
            let left = height_of(column.saturating_sub(1), row);
            let right = height_of((column + 1).min(columns - 1), row);
            let back = height_of(column, row.saturating_sub(1));
            let front = height_of(column, (row + 1).min(rows - 1));
            // Normal ~ (-dh/dx, 1, -dh/dz), scaled to snorm16 with the up
            // component dominant; slopes are at most 20 mm per cell.
            let slope_x = (left - right) * 32_767 / (2 * cell_x);
            let slope_z = (back - front) * 32_767 / (2 * cell_z);
            normals.push([
                slope_x.clamp(-16_000, 16_000) as i16,
                32_767,
                slope_z.clamp(-16_000, 16_000) as i16,
            ]);
        }
    }
    let mut indices = Vec::with_capacity((columns - 1) * (rows - 1) * 6);
    for row in 0..rows - 1 {
        for column in 0..columns - 1 {
            let a = (row * columns + column) as u32;
            let b = a + 1;
            let c = a + columns as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, d, a, d, b]);
        }
    }
    WaterSurfaceUpdateV1 {
        volume_id: binding.volume_id,
        mesh_asset_id: binding.mesh_asset_id,
        positions_micrometres: positions,
        normals_snorm16: normals,
        indices,
    }
}

/// Stateless ballistic jet: droplet `k` of an edge has age
/// `(frame_index + k * stride) mod lifetime` frames, so the stream is
/// continuous and the frame is a pure function of the checkpoint.
fn jet_particles(
    volumes: &WaterVolumeSetV1,
    network: &WaterFlowNetworkV1,
    tick: u64,
    frame_index: u64,
) -> WaterJetParticlesV1 {
    let mut jet = WaterJetParticlesV1::default();
    let gravity = i128::from(WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
    let fps = i128::from(WATER_PRESENTATION_FRAMES_PER_SECOND);
    for (edge_id, edge) in &network.edges {
        let invert = match edge.kind {
            WaterFlowEdgeKindV1::Pipe {
                invert_micrometres, ..
            }
            | WaterFlowEdgeKindV1::Gate {
                invert_micrometres, ..
            } => invert_micrometres,
            _ => continue,
        };
        let Some(flux) = network.edge_flux(*edge_id) else {
            continue;
        };
        let (source_id, destination_id) = match (edge.cell_b, flux.signum()) {
            (Some(b), 1) => (edge.cell_a, b),
            (Some(b), -1) => (b, edge.cell_a),
            _ => continue,
        };
        let (Some(source), Some(destination)) = (
            volumes.definitions.get(&source_id),
            volumes.definitions.get(&destination_id),
        ) else {
            continue;
        };
        let Some(source_level) = volumes.effective_level(source_id, tick) else {
            continue;
        };
        let destination_level = volumes
            .effective_level(destination_id, tick)
            .unwrap_or(invert);
        let head = i128::from(source_level.saturating_sub(invert)).max(0);
        if head == 0 {
            continue;
        }
        // Mouth: the destination face nearest the source, at the invert,
        // in the middle of the overlapping z range (or the source centre).
        let source_centre_x = (source.minimum_micrometres[0] + source.maximum_micrometres[0]) / 2;
        let destination_centre_x =
            (destination.minimum_micrometres[0] + destination.maximum_micrometres[0]) / 2;
        let (mouth_x, direction_x) = if source_centre_x <= destination_centre_x {
            (destination.minimum_micrometres[0], 1_i128)
        } else {
            (destination.maximum_micrometres[0], -1_i128)
        };
        let mouth_z = (source.minimum_micrometres[2].max(destination.minimum_micrometres[2])
            + source.maximum_micrometres[2].min(destination.maximum_micrometres[2]))
            / 2;
        let speed = isqrt_i128(2 * gravity * head); // um/s
        let spawn_per_frame = (i128::from(flux.abs())
            / i128::from(WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES))
        .clamp(0, i128::from(WATER_JET_MAX_SPAWN_PER_FRAME));
        if spawn_per_frame == 0 {
            continue;
        }
        let lifetime = i128::from(WATER_JET_LIFETIME_FRAMES);
        let total = (spawn_per_frame * lifetime).min(
            i128::from(WATER_JET_MAX_PARTICLES)
                - i128::try_from(jet.positions_micrometres.len()).unwrap_or(0),
        );
        if total <= 0 {
            break;
        }
        let stride = (lifetime * 65_536 / total.max(1)).max(1);
        for k in 0..total {
            let age_frames = (i128::from(frame_index) * 65_536 + k * stride) % (lifetime * 65_536);
            let t_num = age_frames; // frames * 65536
            // t seconds = t_num / (65536 * fps)
            let x = i128::from(mouth_x) + direction_x * speed * t_num / (65_536 * fps);
            let drop = gravity * t_num * t_num / (2 * 65_536 * 65_536 * fps * fps);
            let y = i128::from(invert) - drop;
            if y < i128::from(destination_level)
                || y < i128::from(destination.minimum_micrometres[1])
            {
                continue;
            }
            let lateral = (k * 7_919) % 41 - 20; // spread across the mouth width
            let z = i128::from(mouth_z) + lateral * 2_000;
            let vy = -(gravity * t_num / (65_536 * fps));
            jet.positions_micrometres.push([
                i64::try_from(x).unwrap_or(i64::MAX),
                i64::try_from(y).unwrap_or(i64::MAX),
                i64::try_from(z).unwrap_or(i64::MAX),
            ]);
            jet.velocities_micrometres_per_second.push([
                i32::try_from(direction_x * speed).unwrap_or(i32::MAX),
                i32::try_from(vy.clamp(-2_000_000_000, 2_000_000_000)).unwrap_or(0),
                0,
            ]);
        }
    }
    jet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_sine_is_bounded_and_periodic() {
        for angle in [0, 16_384, 32_768, 49_152, 65_536, 70_000, -5_000] {
            let value = sin_q15(angle);
            assert!((-32_767..=32_767).contains(&value));
            assert_eq!(value, sin_q15(angle + 65_536));
        }
        assert_eq!(sin_q15(0), 0);
        assert!(sin_q15(16_384) > 32_000);
        assert!(sin_q15(49_152) < -32_000);
    }

    #[test]
    fn floating_box_depresses_the_ring_and_sheds_droplets_when_fast() {
        let volumes = crate::water::reference_water_volumes().expect("volumes");
        let network = crate::water::reference_water_flow(&volumes).expect("network");
        let bindings = reference_water_surface_bindings();
        let basin = &volumes.definitions[&crate::water::REFERENCE_WATER_BASIN_ID];
        let level = volumes
            .effective_level(crate::water::REFERENCE_WATER_BASIN_ID, 1)
            .expect("basin level");
        let centre_x = (basin.minimum_micrometres[0] + basin.maximum_micrometres[0]) / 2;
        let centre_z = (basin.minimum_micrometres[2] + basin.maximum_micrometres[2]) / 2;
        let floating = |vertical_velocity| WaterFloatingBoxV1 {
            minimum_micrometres: [centre_x - 250_000, level - 200_000, centre_z - 250_000],
            maximum_micrometres: [centre_x + 250_000, level + 300_000, centre_z + 250_000],
            vertical_velocity_micrometres_per_second: vertical_velocity,
        };
        let still = compute_water_presentation_frame(&volumes, &network, &bindings, &[], 1, 7);
        let slow = compute_water_presentation_frame(
            &volumes,
            &network,
            &bindings,
            &[floating(200_000)],
            1,
            7,
        );
        let fast = compute_water_presentation_frame(
            &volumes,
            &network,
            &bindings,
            &[floating(1_000_000)],
            1,
            7,
        );
        // The wake: vertices near the box sit lower, far vertices are unchanged,
        // every vertex stays inside the ripple cap.
        let basin_surface = |frame: &WaterPresentationFrameV1| {
            frame
                .surfaces
                .iter()
                .find(|surface| surface.volume_id == crate::water::REFERENCE_WATER_BASIN_ID)
                .expect("basin surface")
                .positions_micrometres
                .clone()
        };
        let (still_positions, slow_positions) = (basin_surface(&still), basin_surface(&slow));
        let mut lowered = 0;
        for (a, b) in still_positions.iter().zip(&slow_positions) {
            let distance = (a[0] - centre_x).abs().max(a[2] - centre_z).abs();
            assert!(b[1] >= -WATER_RIPPLE_CAP_MICROMETRES);
            if distance > 250_000 + WATER_WAKE_FALLOFF_MICROMETRES {
                assert_eq!(a[1], b[1]);
            } else if b[1] < a[1] {
                lowered += 1;
            }
        }
        assert!(lowered > 0);
        // Droplets only above the speed threshold, inside the particle bound.
        assert_eq!(slow.jet, still.jet);
        assert!(fast.jet.positions_micrometres.len() > still.jet.positions_micrometres.len());
        assert!(fast.jet.positions_micrometres.len() <= WATER_JET_MAX_PARTICLES as usize);
        assert_eq!(
            fast.jet.positions_micrometres.len(),
            fast.jet.velocities_micrometres_per_second.len()
        );
        for position in &fast.jet.positions_micrometres[still.jet.positions_micrometres.len()..] {
            assert!(position[1] >= level);
            assert!((position[0] - centre_x).abs() <= 400_000);
            assert!((position[2] - centre_z).abs() <= 400_000);
        }
    }

    #[test]
    fn frame_is_pure_bounded_and_reads_only_the_checkpoint() {
        let volumes = crate::water::reference_water_volumes().expect("volumes");
        let network = crate::water::reference_water_flow(&volumes).expect("network");
        let stepped = network.step(&volumes).expect("step");
        let (volumes, network) = (stepped.volumes, stepped.network);
        let bindings = reference_water_surface_bindings();
        let first = compute_water_presentation_frame(&volumes, &network, &bindings, &[], 1, 7);
        let again = compute_water_presentation_frame(&volumes, &network, &bindings, &[], 1, 7);
        assert_eq!(first, again);
        assert_eq!(first.surfaces.len(), 3);
        for surface in &first.surfaces {
            assert_eq!(
                surface.positions_micrometres.len(),
                WATER_SURFACE_VERTEX_CAPACITY as usize
            );
            assert_eq!(surface.indices.len(), WATER_SURFACE_INDEX_CAPACITY as usize);
            let definition = &volumes.definitions[&surface.volume_id];
            for position in &surface.positions_micrometres {
                assert!(position[1].abs() <= WATER_RIPPLE_CAP_MICROMETRES);
                assert!(position[0] >= definition.minimum_micrometres[0]);
                assert!(position[0] <= definition.maximum_micrometres[0]);
                assert!(position[2] >= definition.minimum_micrometres[2]);
                assert!(position[2] <= definition.maximum_micrometres[2]);
            }
        }
        assert!(first.jet.positions_micrometres.len() <= WATER_JET_MAX_PARTICLES as usize);
        assert_eq!(
            first.jet.positions_micrometres.len(),
            first.jet.velocities_micrometres_per_second.len()
        );
        // The gate carries water after one step, so the jet exists and
        // moves between the frames.
        assert!(!first.jet.positions_micrometres.is_empty());
        let later = compute_water_presentation_frame(&volumes, &network, &bindings, &[], 1, 8);
        assert_ne!(first.jet, later.jet);
    }
}
