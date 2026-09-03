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
/// Jet: one droplet per this volume of flux per tick, bounded per frame.
pub const WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES: i64 = 500_000;
pub const WATER_JET_MAX_SPAWN_PER_FRAME: u32 = 256;
pub const WATER_JET_MAX_PARTICLES: u32 = 4_096;
pub const WATER_JET_LIFETIME_FRAMES: u32 = 120;
/// Droplet render radius for the particle profile.
pub const WATER_JET_RADIUS_MICROMETRES: u32 = 30_000;

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
    tick: u64,
    frame_index: u64,
) -> WaterPresentationFrameV1 {
    let mut surfaces = Vec::with_capacity(bindings.len());
    for binding in bindings {
        let Some(definition) = volumes.definitions.get(&binding.volume_id) else {
            continue;
        };
        let amplitude = ripple_amplitude(network, binding.volume_id);
        surfaces.push(surface_grid(binding, definition, amplitude, frame_index));
    }
    WaterPresentationFrameV1 {
        tick,
        frame_index,
        surfaces,
        jet: jet_particles(volumes, network, tick, frame_index),
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
    frame_index: u64,
) -> WaterSurfaceUpdateV1 {
    let columns = WATER_SURFACE_GRID_COLUMNS as usize;
    let rows = WATER_SURFACE_GRID_ROWS as usize;
    let [x0, _, z0] = definition.minimum_micrometres;
    let [x1, _, z1] = definition.maximum_micrometres;
    let mut positions = Vec::with_capacity(columns * rows);
    let mut normals = Vec::with_capacity(columns * rows);
    let phase = i64::try_from(frame_index % 65_536).unwrap_or(0) * 1_200;
    let height_at = |column: usize, row: usize| -> i64 {
        if amplitude == 0 {
            return 0;
        }
        // Two crossing waves in 1/65536 turns per grid cell.
        let a = phase + column as i64 * 6_000 + row as i64 * 2_500;
        let b = -phase * 2 / 3 + column as i64 * 2_200 - row as i64 * 5_100;
        let wave = (sin_q15(a) * 6 + sin_q15(b) * 4) / 10;
        (amplitude * wave / 32_767)
            .clamp(-WATER_RIPPLE_CAP_MICROMETRES, WATER_RIPPLE_CAP_MICROMETRES)
    };
    for row in 0..rows {
        for column in 0..columns {
            let x = x0 + (x1 - x0) * column as i64 / (columns as i64 - 1);
            let z = z0 + (z1 - z0) * row as i64 / (rows as i64 - 1);
            positions.push([x, height_at(column, row), z]);
        }
    }
    let cell_x = ((x1 - x0) / (columns as i64 - 1)).max(1);
    let cell_z = ((z1 - z0) / (rows as i64 - 1)).max(1);
    for row in 0..rows {
        for column in 0..columns {
            let left = height_at(column.saturating_sub(1), row);
            let right = height_at((column + 1).min(columns - 1), row);
            let back = height_at(column, row.saturating_sub(1));
            let front = height_at(column, (row + 1).min(rows - 1));
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
    fn frame_is_pure_bounded_and_reads_only_the_checkpoint() {
        let volumes = crate::water::reference_water_volumes().expect("volumes");
        let network = crate::water::reference_water_flow(&volumes).expect("network");
        let stepped = network.step(&volumes).expect("step");
        let (volumes, network) = (stepped.volumes, stepped.network);
        let bindings = reference_water_surface_bindings();
        let first = compute_water_presentation_frame(&volumes, &network, &bindings, 1, 7);
        let again = compute_water_presentation_frame(&volumes, &network, &bindings, 1, 7);
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
        let later = compute_water_presentation_frame(&volumes, &network, &bindings, 1, 8);
        assert_ne!(first.jet, later.jet);
    }
}
