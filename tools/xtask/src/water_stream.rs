//! Binary presentation-surface stream emitted by
//! `nonlocal-feasibility --game-surface-stream`.
//!
//! The research solver stays a separate process; only bounded, versioned
//! surface frames cross the boundary. Nothing flows back.

#![cfg_attr(not(feature = "desktop-sdl-ash"), allow(dead_code))]

use std::io::Read;

pub(super) const STREAM_MAGIC: [u8; 4] = *b"NEWS";
pub(super) const STREAM_VERSION: u32 = 3;
/// Version 2 carried particle centres without neighbour counts.
const STREAM_VERSION_WITH_PARTICLES: u32 = 2;
/// Version 1 frames carry no particle set; version 2 appends it.
const STREAM_VERSION_WITHOUT_PARTICLES: u32 = 1;
/// ADR-102 bound on one published particle set.
pub(super) const MAX_STREAM_PARTICLES: usize = 65_536;
/// Fixed header: magic, version, step, cycle, box, counts, three timings.
const HEADER_BYTES: usize = 4 + 4 + 4 + 4 + 6 * 8 + 8 + 8 + 3 * 8;
/// Surface pixel pitch of the extractor (`GAME_SPACING / 4`).
pub(super) const SURFACE_PIXEL_PITCH_METRES: f64 = 0.0125;
/// Physics cadence of the extractor lanes.
pub(super) const SIMULATION_STEP_SECONDS: f64 = 1.0 / 240.0;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StreamFrame {
    pub(super) step: i32,
    pub(super) cycle: i32,
    pub(super) box_min_metres: [f64; 3],
    pub(super) box_max_metres: [f64; 3],
    pub(super) simulation_seconds: f64,
    pub(super) extraction_ms: f64,
    pub(super) physics_ms: f64,
    pub(super) positions_micrometres: Vec<[i64; 3]>,
    pub(super) indices: Vec<u32>,
    /// Fluid particle centres (version 2+), empty for version 1 frames.
    pub(super) particles_micrometres: Vec<[i64; 3]>,
    /// Fluid neighbours per particle within the producer's presentation
    /// radius (version 3), empty for earlier versions.
    pub(super) particle_neighbours: Vec<u8>,
}

/// Reads one frame; `Ok(None)` is a clean end of stream before a header.
pub(super) fn read_frame(
    reader: &mut impl Read,
    max_vertices: usize,
    max_triangles: usize,
) -> Result<Option<StreamFrame>, String> {
    let mut header = [0_u8; HEADER_BYTES];
    match reader.read_exact(&mut header[..1]) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(format!("surface stream read failed: {error}")),
    }
    reader
        .read_exact(&mut header[1..])
        .map_err(|error| format!("surface stream header truncated: {error}"))?;
    if header[..4] != STREAM_MAGIC {
        return Err("surface stream magic mismatch".to_owned());
    }
    let mut cursor = 4;
    let mut take_u32 = || {
        let value = u32::from_le_bytes(header[cursor..cursor + 4].try_into().unwrap_or([0; 4]));
        cursor += 4;
        value
    };
    let version = take_u32();
    let step = take_u32() as i32;
    let cycle = take_u32() as i32;
    if version != STREAM_VERSION
        && version != STREAM_VERSION_WITH_PARTICLES
        && version != STREAM_VERSION_WITHOUT_PARTICLES
    {
        return Err(format!("surface stream version {version} is unsupported"));
    }
    let mut take_f64 = || {
        let value = f64::from_le_bytes(header[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
        cursor += 8;
        value
    };
    let box_min_metres = [take_f64(), take_f64(), take_f64()];
    let box_max_metres = [take_f64(), take_f64(), take_f64()];
    let mut take_u64 = || {
        let value = u64::from_le_bytes(header[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
        cursor += 8;
        value
    };
    let vertex_count = take_u64();
    let triangle_count = take_u64();
    let mut take_f64 = || {
        let value = f64::from_le_bytes(header[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
        cursor += 8;
        value
    };
    let simulation_seconds = take_f64();
    let extraction_ms = take_f64();
    let physics_ms = take_f64();
    if box_min_metres
        .iter()
        .chain(&box_max_metres)
        .any(|value| !value.is_finite())
        || (0..3).any(|axis| box_min_metres[axis] >= box_max_metres[axis])
        || !simulation_seconds.is_finite()
        || !extraction_ms.is_finite()
        || !physics_ms.is_finite()
        || step < 0
        || cycle < 0
    {
        return Err("surface stream header is outside the bounded profile".to_owned());
    }
    let vertex_count = usize::try_from(vertex_count)
        .ok()
        .filter(|count| *count >= 3 && *count <= max_vertices)
        .ok_or_else(|| "surface stream vertex count is outside the bounded profile".to_owned())?;
    let triangle_count = usize::try_from(triangle_count)
        .ok()
        .filter(|count| *count >= 1 && *count <= max_triangles)
        .ok_or_else(|| "surface stream triangle count is outside the bounded profile".to_owned())?;
    let mut vertex_bytes = vec![0_u8; vertex_count * 24];
    reader
        .read_exact(&mut vertex_bytes)
        .map_err(|error| format!("surface stream vertices truncated: {error}"))?;
    let mut positions_micrometres = Vec::with_capacity(vertex_count);
    for vertex in vertex_bytes.chunks_exact(24) {
        let mut position = [0_i64; 3];
        for (axis, component) in vertex.chunks_exact(8).enumerate() {
            let metres = f64::from_le_bytes(component.try_into().unwrap_or([0; 8]));
            if !metres.is_finite()
                || metres < box_min_metres[axis] - SURFACE_PIXEL_PITCH_METRES
                || metres > box_max_metres[axis] + SURFACE_PIXEL_PITCH_METRES
            {
                return Err("surface stream vertex lies outside the declared box".to_owned());
            }
            position[axis] = (metres * 1_000_000.0).round() as i64;
        }
        positions_micrometres.push(position);
    }
    let mut index_bytes = vec![0_u8; triangle_count * 12];
    reader
        .read_exact(&mut index_bytes)
        .map_err(|error| format!("surface stream indices truncated: {error}"))?;
    let indices = index_bytes
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap_or([0; 4])))
        .collect::<Vec<_>>();
    if indices
        .iter()
        .any(|index| usize::try_from(*index).map_or(true, |index| index >= vertex_count))
    {
        return Err("surface stream face index is outside the vertex array".to_owned());
    }
    let mut particles_micrometres = Vec::new();
    let mut particle_neighbours = Vec::new();
    if version >= STREAM_VERSION_WITH_PARTICLES {
        let mut count_bytes = [0_u8; 8];
        reader
            .read_exact(&mut count_bytes)
            .map_err(|error| format!("surface stream particle count truncated: {error}"))?;
        let particle_count = usize::try_from(u64::from_le_bytes(count_bytes))
            .ok()
            .filter(|count| *count <= MAX_STREAM_PARTICLES)
            .ok_or_else(|| {
                "surface stream particle count is outside the bounded profile".to_owned()
            })?;
        let mut particle_bytes = vec![0_u8; particle_count * 12];
        reader
            .read_exact(&mut particle_bytes)
            .map_err(|error| format!("surface stream particles truncated: {error}"))?;
        particles_micrometres.reserve(particle_count);
        for particle in particle_bytes.chunks_exact(12) {
            let mut position = [0_i64; 3];
            for (axis, component) in particle.chunks_exact(4).enumerate() {
                let metres = f64::from(f32::from_le_bytes(component.try_into().unwrap_or([0; 4])));
                if !metres.is_finite()
                    || metres < box_min_metres[axis] - SURFACE_PIXEL_PITCH_METRES
                    || metres > box_max_metres[axis] + SURFACE_PIXEL_PITCH_METRES
                {
                    return Err("surface stream particle lies outside the declared box".to_owned());
                }
                position[axis] = (metres * 1_000_000.0).round() as i64;
            }
            particles_micrometres.push(position);
        }
        if version == STREAM_VERSION {
            particle_neighbours = vec![0_u8; particle_count];
            reader
                .read_exact(&mut particle_neighbours)
                .map_err(|error| format!("surface stream neighbour counts truncated: {error}"))?;
        }
    }
    Ok(Some(StreamFrame {
        step,
        cycle,
        box_min_metres,
        box_max_metres,
        simulation_seconds,
        extraction_ms,
        physics_ms,
        positions_micrometres,
        indices,
        particles_micrometres,
        particle_neighbours,
    }))
}

/// Surface grid size and ring capacity implied by the extractor box.
pub(super) fn surface_capacity(
    box_min_metres: [f64; 3],
    box_max_metres: [f64; 3],
) -> Result<(u32, u32), String> {
    let cells = |axis: usize| -> Result<u32, String> {
        let extent = box_max_metres[axis] - box_min_metres[axis];
        let cells = (extent / SURFACE_PIXEL_PITCH_METRES).round();
        if !cells.is_finite() || !(2.0..=4096.0).contains(&cells) {
            return Err("surface stream box does not form a bounded pixel grid".to_owned());
        }
        Ok(cells as u32)
    };
    let width = cells(0)?;
    let height = cells(2)?;
    let vertices = width
        .checked_mul(height)
        .ok_or_else(|| "surface capacity overflow".to_owned())?;
    let indices = (width - 1)
        .checked_mul(height - 1)
        .and_then(|cells| cells.checked_mul(6))
        .ok_or_else(|| "surface capacity overflow".to_owned())?;
    Ok((vertices, indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_bytes(vertex_count: u64, triangle_count: u64, indices: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&STREAM_MAGIC);
        bytes.extend_from_slice(&STREAM_VERSION.to_le_bytes());
        bytes.extend_from_slice(&7_i32.to_le_bytes());
        bytes.extend_from_slice(&0_i32.to_le_bytes());
        for value in [0.0_f64, 0.0, 0.0, 2.0, 0.75, 1.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&vertex_count.to_le_bytes());
        bytes.extend_from_slice(&triangle_count.to_le_bytes());
        for value in [7.0_f64 / 240.0, 3.5, 0.4] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for vertex in [
            [0.00625, 0.3, 0.00625],
            [0.01875, 0.31, 0.00625],
            [0.00625, 0.32, 0.01875],
        ] {
            for component in vertex {
                bytes.extend_from_slice(&f64::to_le_bytes(component));
            }
        }
        for index in indices {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        // Version 2 trailer: an empty particle set.
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes
    }

    #[test]
    fn stream_frame_round_trips_and_ends_cleanly() {
        let mut bytes = frame_bytes(3, 1, &[0, 2, 1]);
        bytes.extend(frame_bytes(3, 1, &[0, 2, 1]));
        let mut reader = bytes.as_slice();
        let frame = read_frame(&mut reader, 100, 100)
            .expect("first frame")
            .expect("frame present");
        assert_eq!(frame.step, 7);
        assert_eq!(frame.positions_micrometres[1], [18_750, 310_000, 6_250]);
        assert_eq!(frame.indices, [0, 2, 1]);
        assert_eq!(frame.box_max_metres, [2.0, 0.75, 1.0]);
        assert!((frame.simulation_seconds - 7.0 / 240.0).abs() < 1e-12);
        assert!(
            read_frame(&mut reader, 100, 100)
                .expect("second frame")
                .is_some()
        );
        assert!(
            read_frame(&mut reader, 100, 100)
                .expect("clean end")
                .is_none()
        );
    }

    #[test]
    fn stream_frame_rejects_bad_magic_bounds_and_indices() {
        let mut bad_magic = frame_bytes(3, 1, &[0, 1, 2]);
        bad_magic[0] = b'X';
        assert!(read_frame(&mut bad_magic.as_slice(), 100, 100).is_err());
        let out_of_range = frame_bytes(3, 1, &[0, 1, 3]);
        assert!(read_frame(&mut out_of_range.as_slice(), 100, 100).is_err());
        let over_capacity = frame_bytes(3, 1, &[0, 1, 2]);
        assert!(read_frame(&mut over_capacity.as_slice(), 2, 100).is_err());
        let truncated = &frame_bytes(3, 1, &[0, 1, 2])[..40];
        assert!(read_frame(&mut &truncated[..], 100, 100).is_err());
    }

    #[test]
    fn capacity_follows_the_extractor_pixel_grid() {
        assert_eq!(
            surface_capacity([0.0; 3], [2.0, 0.75, 1.0]).expect("4k grid"),
            (12_800, 75_366)
        );
        assert_eq!(
            surface_capacity([0.0; 3], [4.0, 0.75, 2.0]).expect("16k grid"),
            (51_200, 304_326)
        );
        assert!(surface_capacity([0.0; 3], [0.01, 0.75, 1.0]).is_err());
    }
}
