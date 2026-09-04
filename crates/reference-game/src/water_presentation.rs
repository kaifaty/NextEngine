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
    floating_boxes_with_ids(checkpoint)
        .into_iter()
        .map(|(_, floating)| floating)
        .collect()
}

/// Plan 34: the same records with their body ids (the splash cues key on
/// the body).
#[must_use]
pub fn floating_boxes_with_ids(
    checkpoint: &next_contracts::physics::PhysicsWorldCheckpointV1,
) -> Vec<(next_contracts::physics::PhysicsBodyIdV1, WaterFloatingBoxV1)> {
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
            boxes.push((
                *body_id,
                WaterFloatingBoxV1 {
                    minimum_micrometres,
                    maximum_micrometres,
                    vertical_velocity_micrometres_per_second: state
                        .linear_velocity_micrometres_per_second[1],
                },
            ));
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
    /// Plan 21 (SPEC-38 practice 3): one record per edge that moved water
    /// in the published tick, in edge id order; nothing per cell.
    pub edges: Vec<WaterEdgePresentationV1>,
    /// Plan 25: the committed floating boxes the stage read (the PhysX lane
    /// emits from the same exact inputs as the splash).
    pub boxes: Vec<WaterFloatingBoxV1>,
}

/// Plan 21: an open sill whose sink level lies this far below the sill
/// sheds a fall; above it the sill is a foam band only.
pub const WATER_FALL_DROP_MICROMETRES: i64 = 20_000;

/// Plan 22 (SPEC-38 practice 4): the whirlpool over a sink.
pub const WATER_VORTEX_RADIUS_MICROMETRES: i64 = 400_000;
pub const WATER_VORTEX_DEPTH_CAP_MICROMETRES: i64 = 12_000;
pub const WATER_VORTEX_RIPPLE_MICROMETRES: i64 = 3_000;
/// One radial turn of the spiral per this distance.
pub const WATER_VORTEX_RIPPLE_WAVELENGTH_MICROMETRES: i64 = 150_000;

/// Plan 22: one whirlpool of a bound surface, derived from a sink or pump
/// edge's exact flux; presentation only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Vortex {
    centre_micrometres: [i64; 2],
    depth_micrometres: i64,
}

/// The vortices of a volume: every sink on it and every pump drawing from
/// it with non-zero last flux.
fn vortices_of(
    volumes: &WaterVolumeSetV1,
    network: &WaterFlowNetworkV1,
    volume_id: PersistentId,
) -> Vec<Vortex> {
    let mut vortices = Vec::new();
    let Some(definition) = volumes.definitions.get(&volume_id) else {
        return vortices;
    };
    for (edge_id, edge) in &network.edges {
        let Some(flux) = network.edge_flux(*edge_id) else {
            continue;
        };
        if flux == 0 {
            continue;
        }
        let centre = match edge.kind {
            WaterFlowEdgeKindV1::Sink { .. } if edge.cell_a == volume_id => plan_centre(definition),
            WaterFlowEdgeKindV1::Pump { .. } => {
                let Some(cell_b_id) = edge.cell_b else {
                    continue;
                };
                // The pump draws from `cell_a` when the flux is positive.
                let drawn_from = if flux > 0 { edge.cell_a } else { cell_b_id };
                if drawn_from != volume_id {
                    continue;
                }
                let Some(other) = volumes.definitions.get(if drawn_from == edge.cell_a {
                    &cell_b_id
                } else {
                    &edge.cell_a
                }) else {
                    continue;
                };
                shared_face_midpoint(definition, other)
            }
            _ => continue,
        };
        vortices.push(Vortex {
            centre_micrometres: centre,
            depth_micrometres: (flux.abs() / 2).min(WATER_VORTEX_DEPTH_CAP_MICROMETRES),
        });
    }
    vortices
}

/// Plan 22: the height contribution of the vortices at `(x, z)`.
fn vortex_height(x: i64, z: i64, vortices: &[Vortex], frame: i64) -> i64 {
    let radius = WATER_VORTEX_RADIUS_MICROMETRES;
    let mut height = 0_i64;
    for vortex in vortices {
        let dx = x - vortex.centre_micrometres[0];
        let dz = z - vortex.centre_micrometres[1];
        let r_squared = i128::from(dx) * i128::from(dx) + i128::from(dz) * i128::from(dz);
        let r = isqrt_i128(r_squared);
        if r >= i128::from(radius) {
            continue;
        }
        let r = r as i64;
        let f_q15 = (radius - r) * 32_767 / radius;
        // Dip: -depth f^2.
        height -= vortex.depth_micrometres * f_q15 * f_q15 / (32_767 * 32_767);
        // Spiral: amplitude f sin(2 theta + k r - omega t), two arms, one
        // revolution per second of the frame clock.
        let phase = r * 65_536 / WATER_VORTEX_RIPPLE_WAVELENGTH_MICROMETRES
            - frame * 65_536 / i64::from(WATER_PRESENTATION_FRAMES_PER_SECOND);
        let s = sin_q15(phase);
        let c = sin_q15(phase + 16_384);
        let (sin_2theta, cos_2theta) = if r_squared == 0 {
            (0, 32_767)
        } else {
            (
                (2 * i128::from(dx) * i128::from(dz) * 32_767 / r_squared) as i64,
                ((i128::from(dx) * i128::from(dx) - i128::from(dz) * i128::from(dz)) * 32_767
                    / r_squared) as i64,
            )
        };
        let wave = (s * cos_2theta + c * sin_2theta) / 32_767;
        height += WATER_VORTEX_RIPPLE_MICROMETRES * f_q15 / 32_767 * wave / 32_767;
    }
    height
}

/// Plan 21: what an active edge presents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaterEdgePresentationKindV1 {
    /// Pipe or gate: the droplet stream from the mouth.
    Jet,
    /// Open sill with a drop: droplets from the crest.
    Fall,
    /// Open sill with both levels above it: a foam band (record only).
    Sill,
    /// Source, sink or pump: a mouth (record only).
    Mouth,
}

/// Plan 21: the presentation record of one active edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterEdgePresentationV1 {
    pub edge_id: PersistentId,
    pub kind: WaterEdgePresentationKindV1,
    /// Crest or mouth point in world micrometres.
    pub crest_micrometres: [i64; 3],
    /// Plan direction of the flow, q15 unit vector `[x, z]`; `[0, 0]` for
    /// one-cell edges.
    pub direction_q15: [i32; 2],
    pub source_level_micrometres: i64,
    pub sink_level_micrometres: i64,
    /// Signed flux of the published tick, positive from `cell_a`.
    pub flux_cubic_millimetres: i64,
}

/// Which volumes carry an authored surface quad.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterSurfaceBindingV1 {
    pub volume_id: PersistentId,
    pub mesh_asset_id: AssetId,
}

/// The reference scene's surface quads.
#[must_use]
pub fn reference_water_surface_bindings() -> [WaterSurfaceBindingV1; 8] {
    let cells = crate::water::reference_water_stream_cell_ids();
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
        WaterSurfaceBindingV1 {
            volume_id: crate::water::REFERENCE_WATER_POND_ID,
            mesh_asset_id: crate::source::REFERENCE_WATER_POND_SURFACE_MESH_ASSET_ID,
        },
        WaterSurfaceBindingV1 {
            volume_id: crate::water::REFERENCE_WATER_LAKE_ID,
            mesh_asset_id: crate::source::REFERENCE_WATER_LAKE_SURFACE_MESH_ASSET_ID,
        },
        WaterSurfaceBindingV1 {
            volume_id: cells[0],
            mesh_asset_id: crate::source::REFERENCE_WATER_STREAM_SURFACE_MESH_ASSET_IDS[0],
        },
        WaterSurfaceBindingV1 {
            volume_id: cells[1],
            mesh_asset_id: crate::source::REFERENCE_WATER_STREAM_SURFACE_MESH_ASSET_IDS[1],
        },
        WaterSurfaceBindingV1 {
            volume_id: cells[2],
            mesh_asset_id: crate::source::REFERENCE_WATER_STREAM_SURFACE_MESH_ASSET_IDS[2],
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
        // Plan 22: whirlpools over the sinks of this volume.
        let vortices = vortices_of(volumes, network, binding.volume_id);
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
            &vortices,
            frame_index,
        ));
    }
    let (edges, mut jet) = edge_records(volumes, network, tick, frame_index);
    splash_particles(volumes, boxes, tick, frame_index, &mut jet);
    WaterPresentationFrameV1 {
        tick,
        frame_index,
        surfaces,
        jet,
        edges,
        boxes: boxes.to_vec(),
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
    vortices: &[Vortex],
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
        if !vortices.is_empty() {
            height += vortex_height(x, z, vortices, frame);
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

/// A stateless ballistic stream: droplet `k` has age `(frame_index * 65536
/// + k * stride) mod (lifetime * 65536)` frames, so the stream is a pure
/// function of the frame index. `start` is the mouth or crest, `direction`
/// the q15 plan direction, `speed` in micrometres per second; droplets
/// below the destination level or floor are not emitted. The lateral
/// spread is the jet's: across the direction, `+z` for a flow along `+x`
/// and along `-x` alike.
#[allow(
    clippy::too_many_arguments,
    reason = "the stream inputs of the private stage stay explicit"
)]
fn emit_stream(
    jet: &mut WaterJetParticlesV1,
    start: [i64; 3],
    direction_q15: [i32; 2],
    speed: i128,
    destination_level: i64,
    destination_floor: i64,
    flux: i64,
    frame_index: u64,
) {
    let gravity = i128::from(WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
    let fps = i128::from(WATER_PRESENTATION_FRAMES_PER_SECOND);
    let spawn_per_frame = (i128::from(flux.abs())
        / i128::from(WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES))
    .clamp(0, i128::from(WATER_JET_MAX_SPAWN_PER_FRAME));
    if spawn_per_frame == 0 {
        return;
    }
    let lifetime = i128::from(WATER_JET_LIFETIME_FRAMES);
    let total = (spawn_per_frame * lifetime).min(
        i128::from(WATER_JET_MAX_PARTICLES)
            - i128::try_from(jet.positions_micrometres.len()).unwrap_or(0),
    );
    if total <= 0 {
        return;
    }
    let stride = (lifetime * 65_536 / total.max(1)).max(1);
    let (dx, dz) = (i128::from(direction_q15[0]), i128::from(direction_q15[1]));
    // Lateral unit across the direction (the jet's rule: `+z` for `+-x`).
    let (lx, lz) = (-dz, dx.abs());
    for k in 0..total {
        let age_frames = (i128::from(frame_index) * 65_536 + k * stride) % (lifetime * 65_536);
        let t_num = age_frames; // frames * 65536; t seconds = t_num / (65536 * fps)
        let along = dx * speed * t_num / (65_536 * fps * 32_767);
        let along_z = dz * speed * t_num / (65_536 * fps * 32_767);
        let drop = gravity * t_num * t_num / (2 * 65_536 * 65_536 * fps * fps);
        let y = i128::from(start[1]) - drop;
        if y < i128::from(destination_level) || y < i128::from(destination_floor) {
            continue;
        }
        let lateral = (k * 7_919) % 41 - 20; // spread across the mouth width
        let x = i128::from(start[0]) + along + lateral * 2_000 * lx / 32_767;
        let z = i128::from(start[2]) + along_z + lateral * 2_000 * lz / 32_767;
        let vy = -(gravity * t_num / (65_536 * fps));
        jet.positions_micrometres.push([
            i64::try_from(x).unwrap_or(i64::MAX),
            i64::try_from(y).unwrap_or(i64::MAX),
            i64::try_from(z).unwrap_or(i64::MAX),
        ]);
        jet.velocities_micrometres_per_second.push([
            i32::try_from(dx * speed / 32_767).unwrap_or(i32::MAX),
            i32::try_from(vy.clamp(-2_000_000_000, 2_000_000_000)).unwrap_or(0),
            i32::try_from(dz * speed / 32_767).unwrap_or(i32::MAX),
        ]);
    }
}

/// Plan direction from `from` to `to` as a q15 unit vector (integer).
fn plan_direction_q15(from: [i64; 2], to: [i64; 2]) -> [i32; 2] {
    let dx = i128::from(to[0] - from[0]);
    let dz = i128::from(to[1] - from[1]);
    let length = isqrt_i128(dx * dx + dz * dz);
    if length == 0 {
        return [0, 0];
    }
    [
        i32::try_from(dx * 32_767 / length).unwrap_or(0),
        i32::try_from(dz * 32_767 / length).unwrap_or(0),
    ]
}

fn plan_centre(definition: &WaterVolumeDefinitionV1) -> [i64; 2] {
    [
        (definition.minimum_micrometres[0] + definition.maximum_micrometres[0]) / 2,
        (definition.minimum_micrometres[2] + definition.maximum_micrometres[2]) / 2,
    ]
}

/// The midpoint of the shared face of two touching plan rectangles.
fn shared_face_midpoint(a: &WaterVolumeDefinitionV1, b: &WaterVolumeDefinitionV1) -> [i64; 2] {
    let x0 = a.minimum_micrometres[0].max(b.minimum_micrometres[0]);
    let x1 = a.maximum_micrometres[0].min(b.maximum_micrometres[0]);
    let z0 = a.minimum_micrometres[2].max(b.minimum_micrometres[2]);
    let z1 = a.maximum_micrometres[2].min(b.maximum_micrometres[2]);
    [(x0 + x1) / 2, (z0 + z1) / 2]
}

/// Plan 34: the tick's edge records for the audio scene (the droplet
/// streams are the render side's business).
#[must_use]
pub fn water_audio_edge_records(
    volumes: &WaterVolumeSetV1,
    network: &WaterFlowNetworkV1,
    tick: u64,
) -> Vec<WaterEdgePresentationV1> {
    edge_records(volumes, network, tick, 0).0
}

/// Plan 21 (SPEC-38 practice 3): one record per active edge in edge id
/// order, and the droplet streams of jets and falls (the jet rule of plan
/// 09 unchanged for pipes and gates).
fn edge_records(
    volumes: &WaterVolumeSetV1,
    network: &WaterFlowNetworkV1,
    tick: u64,
    frame_index: u64,
) -> (Vec<WaterEdgePresentationV1>, WaterJetParticlesV1) {
    let mut jet = WaterJetParticlesV1::default();
    let mut records = Vec::new();
    let gravity = i128::from(WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
    for (edge_id, edge) in &network.edges {
        let Some(flux) = network.edge_flux(*edge_id) else {
            continue;
        };
        if flux == 0 || records.len() >= next_contracts::physics::MAX_WATER_FLOW_EDGES {
            continue;
        }
        let Some(cell_a) = volumes.definitions.get(&edge.cell_a) else {
            continue;
        };
        let level_a = volumes
            .effective_level(edge.cell_a, tick)
            .unwrap_or(cell_a.minimum_micrometres[1]);
        match edge.kind {
            WaterFlowEdgeKindV1::Pipe {
                invert_micrometres, ..
            }
            | WaterFlowEdgeKindV1::Gate {
                invert_micrometres, ..
            } => {
                let invert = invert_micrometres;
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
                // Mouth: the destination face nearest the source, at the invert,
                // in the middle of the overlapping z range (or the source centre).
                let source_centre_x =
                    (source.minimum_micrometres[0] + source.maximum_micrometres[0]) / 2;
                let destination_centre_x =
                    (destination.minimum_micrometres[0] + destination.maximum_micrometres[0]) / 2;
                let (mouth_x, direction_x) = if source_centre_x <= destination_centre_x {
                    (destination.minimum_micrometres[0], 32_767)
                } else {
                    (destination.maximum_micrometres[0], -32_767)
                };
                let mouth_z = (source.minimum_micrometres[2]
                    .max(destination.minimum_micrometres[2])
                    + source.maximum_micrometres[2].min(destination.maximum_micrometres[2]))
                    / 2;
                let crest = [mouth_x, invert, mouth_z];
                records.push(WaterEdgePresentationV1 {
                    edge_id: *edge_id,
                    kind: WaterEdgePresentationKindV1::Jet,
                    crest_micrometres: crest,
                    direction_q15: [direction_x, 0],
                    source_level_micrometres: source_level,
                    sink_level_micrometres: destination_level,
                    flux_cubic_millimetres: flux,
                });
                if head == 0 {
                    continue;
                }
                let speed = isqrt_i128(2 * gravity * head); // um/s
                emit_stream(
                    &mut jet,
                    crest,
                    [direction_x, 0],
                    speed,
                    destination_level,
                    destination.minimum_micrometres[1],
                    flux,
                    frame_index,
                );
            }
            WaterFlowEdgeKindV1::Open {
                sill_micrometres, ..
            } => {
                let Some(cell_b_id) = edge.cell_b else {
                    continue;
                };
                let Some(cell_b) = volumes.definitions.get(&cell_b_id) else {
                    continue;
                };
                let level_b = volumes
                    .effective_level(cell_b_id, tick)
                    .unwrap_or(cell_b.minimum_micrometres[1]);
                let (source, sink, source_level, sink_level) = if flux > 0 {
                    (cell_a, cell_b, level_a, level_b)
                } else {
                    (cell_b, cell_a, level_b, level_a)
                };
                let face = shared_face_midpoint(source, sink);
                let crest = [face[0], sill_micrometres, face[1]];
                let direction = plan_direction_q15(plan_centre(source), plan_centre(sink));
                let fall = sink_level < sill_micrometres - WATER_FALL_DROP_MICROMETRES;
                records.push(WaterEdgePresentationV1 {
                    edge_id: *edge_id,
                    kind: if fall {
                        WaterEdgePresentationKindV1::Fall
                    } else {
                        WaterEdgePresentationKindV1::Sill
                    },
                    crest_micrometres: crest,
                    direction_q15: direction,
                    source_level_micrometres: source_level,
                    sink_level_micrometres: sink_level,
                    flux_cubic_millimetres: flux,
                });
                if !fall {
                    continue;
                }
                let head = i128::from(source_level.saturating_sub(sill_micrometres)).max(0);
                if head == 0 {
                    continue;
                }
                let speed = isqrt_i128(2 * gravity * head);
                emit_stream(
                    &mut jet,
                    crest,
                    direction,
                    speed,
                    sink_level,
                    sink.minimum_micrometres[1],
                    flux,
                    frame_index,
                );
            }
            // Plan 41: a seep is vertical and silent; no record, no emitter.
            WaterFlowEdgeKindV1::Seep { .. } => continue,
            WaterFlowEdgeKindV1::Pump { .. }
            | WaterFlowEdgeKindV1::Source { .. }
            | WaterFlowEdgeKindV1::Sink { .. } => {
                let (crest, direction, sink_level) = match edge.cell_b {
                    Some(cell_b_id) => {
                        let Some(cell_b) = volumes.definitions.get(&cell_b_id) else {
                            continue;
                        };
                        let level_b = volumes
                            .effective_level(cell_b_id, tick)
                            .unwrap_or(cell_b.minimum_micrometres[1]);
                        let face = shared_face_midpoint(cell_a, cell_b);
                        let (from, to) = if flux > 0 {
                            (cell_a, cell_b)
                        } else {
                            (cell_b, cell_a)
                        };
                        (
                            [face[0], level_a.max(level_b), face[1]],
                            plan_direction_q15(plan_centre(from), plan_centre(to)),
                            level_b,
                        )
                    }
                    None => {
                        let centre = plan_centre(cell_a);
                        ([centre[0], level_a, centre[1]], [0, 0], level_a)
                    }
                };
                records.push(WaterEdgePresentationV1 {
                    edge_id: *edge_id,
                    kind: WaterEdgePresentationKindV1::Mouth,
                    crest_micrometres: crest,
                    direction_q15: direction,
                    source_level_micrometres: level_a,
                    sink_level_micrometres: sink_level,
                    flux_cubic_millimetres: flux,
                });
            }
        }
    }
    (records, jet)
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
        assert_eq!(first.surfaces.len(), 8);
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

    #[test]
    fn active_edges_yield_one_record_each_and_sills_with_a_drop_shed_falls() {
        // Plan 21: a 2 x 1 lattice with a 0.5 m drop pours over its sill (a
        // Fall with droplets), then settles (a Sill without droplets); the
        // reference gate presents a Jet.
        let region = next_contracts::physics::WaterLatticeRegionV1 {
            region_id: PersistentId::from_bytes([0x4e; 16]),
            origin_micrometres: [0; 3],
            cell_size_micrometres: [1_000_000, 1_000_000],
            columns: 2,
            rows: 1,
            ceiling_micrometres: 3_000_000,
            floor_micrometres: vec![500_000, 0],
            initial_level_micrometres: vec![1_500_000, 0],
            sill_coefficient_permille: 600,
            profile_revision: 1,
        };
        let (mut volumes, mut network) = region.build(30).expect("build");
        network.step_in_place(&mut volumes).expect("first step");
        let frame = compute_water_presentation_frame(&volumes, &network, &[], &[], 1, 3);
        assert!(frame.surfaces.is_empty(), "no binding, no surface");
        assert_eq!(frame.edges.len(), 1);
        let record = frame.edges[0];
        assert_eq!(record.kind, WaterEdgePresentationKindV1::Fall);
        assert_eq!(record.crest_micrometres, [1_000_000, 500_000, 500_000]);
        assert_eq!(record.direction_q15, [32_767, 0]);
        assert!(record.flux_cubic_millimetres > 0);
        assert!(
            !frame.jet.positions_micrometres.is_empty(),
            "the fall sheds droplets"
        );
        assert!(frame.jet.positions_micrometres.len() <= WATER_JET_MAX_PARTICLES as usize);
        assert_eq!(
            frame,
            compute_water_presentation_frame(&volumes, &network, &[], &[], 1, 3)
        );
        for _ in 0..3_000 {
            network.step_in_place(&mut volumes).expect("step");
        }
        let settled = compute_water_presentation_frame(&volumes, &network, &[], &[], 3_001, 9);
        let active = network
            .edge_states
            .values()
            .filter(|state| state.last_flux_cubic_millimetres != 0)
            .count();
        assert_eq!(settled.edges.len(), active);
        for record in &settled.edges {
            assert_eq!(record.kind, WaterEdgePresentationKindV1::Sill);
        }
        assert!(settled.jet.positions_micrometres.is_empty());

        let volumes = crate::water::reference_water_volumes().expect("volumes");
        let network = crate::water::reference_water_flow(&volumes).expect("network");
        let stepped = network.step(&volumes).expect("step");
        let frame =
            compute_water_presentation_frame(&stepped.volumes, &stepped.network, &[], &[], 1, 7);
        assert!(
            frame
                .edges
                .iter()
                .any(|record| record.kind == WaterEdgePresentationKindV1::Jet)
        );
        assert!(frame.edges.len() <= next_contracts::physics::MAX_WATER_FLOW_EDGES);
    }

    #[test]
    fn a_draining_sink_dips_its_vessel_ring_and_touches_no_other_surface() {
        // Plan 22: vessel B's ring is lower at the sink while it drains;
        // with the sink's flux at zero the frame is the no-vortex frame.
        let volumes = crate::water::reference_water_volumes().expect("volumes");
        let network = crate::water::reference_water_flow(&volumes).expect("network");
        let mut volumes = volumes;
        let mut network = network;
        // Fill vessel B so the sink drains.
        let vessel_b = volumes
            .states
            .get_mut(&crate::water::REFERENCE_WATER_VESSEL_B_ID)
            .expect("vessel b");
        vessel_b.record_revision += 1;
        vessel_b.level_micrometres = 1_000_000;
        network.step_in_place(&mut volumes).expect("step");
        let sink_flux = network
            .edge_flux(crate::water::REFERENCE_WATER_FLOW_SINK_ID)
            .expect("sink flux");
        assert!(sink_flux < 0, "the sink drains vessel b");
        let bindings = reference_water_surface_bindings();
        let draining = compute_water_presentation_frame(&volumes, &network, &bindings, &[], 2, 11);
        let mut still_network = network.clone();
        still_network
            .edge_states
            .get_mut(&crate::water::REFERENCE_WATER_FLOW_SINK_ID)
            .expect("sink state")
            .last_flux_cubic_millimetres = 0;
        let still =
            compute_water_presentation_frame(&volumes, &still_network, &bindings, &[], 2, 11);
        assert_eq!(draining.surfaces[0], still.surfaces[0], "basin untouched");
        assert_eq!(
            draining.surfaces[1], still.surfaces[1],
            "vessel a untouched"
        );
        let (b_draining, b_still) = (&draining.surfaces[2], &still.surfaces[2]);
        let definition = &volumes.definitions[&crate::water::REFERENCE_WATER_VESSEL_B_ID];
        let centre = plan_centre(definition);
        let nearest = b_draining
            .positions_micrometres
            .iter()
            .zip(&b_still.positions_micrometres)
            .min_by_key(|(position, _)| {
                (position[0] - centre[0]).abs() + (position[2] - centre[1]).abs()
            })
            .expect("vertices");
        assert!(nearest.0[1] < nearest.1[1], "the ring dips at the sink");
        assert!(
            nearest.1[1] - nearest.0[1]
                <= WATER_VORTEX_DEPTH_CAP_MICROMETRES + WATER_VORTEX_RIPPLE_MICROMETRES
        );
        for position in &b_draining.positions_micrometres {
            assert!(
                position[1] >= -WATER_RIPPLE_CAP_MICROMETRES
                    && position[1] < WATER_RIPPLE_CAP_MICROMETRES
            );
        }
        assert_eq!(
            draining,
            compute_water_presentation_frame(&volumes, &network, &bindings, &[], 2, 11)
        );
    }
}
