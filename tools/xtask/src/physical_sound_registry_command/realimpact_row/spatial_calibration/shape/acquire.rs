use std::collections::BTreeMap;
use std::io::Read;

use flate2::read::DeflateDecoder;
use serde::Serialize;

use super::super::super::*;
use super::super::dsp::{LISTENER_COUNT, ModeSeed, REFERENCE_LISTENER};
use super::manifest::{ArchiveProfile, MeshDescriptor};

pub(super) struct Acquisition {
    pub(super) object_id: String,
    pub(super) descriptor: MeshDescriptor,
    pub(super) rows: Vec<Vec<f64>>,
    pub(super) modes: Vec<ModeSeed>,
    pub(super) payload: Vec<u8>,
    pub(super) summary: AcquisitionSummary,
}

#[derive(Serialize)]
pub(super) struct AcquisitionSummary {
    pub(super) dataset_object_id: String,
    pub(super) archive_url: String,
    pub(super) archive_content_length: u64,
    pub(super) compressed_prefix_bytes: u64,
    pub(super) compressed_prefix_sha256: String,
    pub(super) http_range_payload_bytes: u64,
    pub(super) full_archive_fraction: f64,
    pub(super) selected_block_path: String,
    pub(super) selected_block_sha256: String,
    pub(super) selected_row_count: usize,
    pub(super) selected_rows: Vec<RowIdentity>,
    pub(super) extracted_mode_count: usize,
    pub(super) persistent_mode_count: usize,
}

#[derive(Serialize)]
pub(super) struct RowIdentity {
    row_index: usize,
    payload_offset_bytes: usize,
    sha256: String,
}

pub(super) fn run(profile: &ArchiveProfile, prefix_bytes: u64) -> Result<Acquisition, String> {
    let frozen = profile.frozen(prefix_bytes)?;
    let resolve = resolve_public_https_endpoint(frozen.archive_url)?
        .ok_or_else(|| "REALIMPACT archive host did not resolve".to_owned())?;
    let mut fetched_bytes = 0_u64;
    let eocd = fetch_range(
        frozen,
        &resolve,
        frozen.archive_bytes - 22,
        22,
        &mut fetched_bytes,
    )?;
    validate_archive_headers(frozen, &eocd)?;
    validate_eocd(frozen, &eocd.body)?;
    let central = fetch_range(
        frozen,
        &resolve,
        frozen.central_offset,
        frozen.central_bytes,
        &mut fetched_bytes,
    )?;
    validate_archive_headers(frozen, &central)?;
    require_hash(
        &central.body,
        frozen.central_sha256,
        "ZIP central directory",
    )?;
    validate_central_entries(frozen, &parse_central_directory(frozen, &central.body)?)?;

    let mut metadata = BTreeMap::new();
    for spec in frozen
        .entries
        .iter()
        .filter(|entry| !entry.raw_sha256.is_empty())
    {
        let compressed = fetch_entry(
            frozen,
            &resolve,
            spec,
            spec.compressed_bytes,
            &mut fetched_bytes,
        )?;
        metadata.insert(spec.name, decompress_complete(spec, &compressed)?);
    }
    validate_base_metadata(frozen, &metadata)?;

    let audio = entry(frozen, frozen.audio_entry_name)?;
    let compressed = fetch_entry(frozen, &resolve, audio, prefix_bytes, &mut fetched_bytes)?;
    let compressed_prefix_sha256 = sha256_hex(&compressed);
    let (rows, payload, selected_rows) = decode_base_rows(frozen, &compressed)?;
    let extracted =
        crate::physical_sound_registry_command::transfer_calibration::extract_v2_spatial_modes(
            &rows[REFERENCE_LISTENER],
        )?;
    let modes = extracted
        .into_iter()
        .map(|(frequency_hz, persistent)| ModeSeed {
            frequency_hz,
            persistent,
        })
        .collect::<Vec<_>>();
    let extracted_mode_count = modes.len();
    let persistent_mode_count = modes.iter().filter(|mode| mode.persistent).count();
    let selected_block_sha256 = sha256_hex(&payload);
    Ok(Acquisition {
        object_id: profile.dataset_object_id.clone(),
        descriptor: profile.mesh_descriptor.clone(),
        rows,
        modes,
        payload,
        summary: AcquisitionSummary {
            dataset_object_id: profile.dataset_object_id.clone(),
            archive_url: profile.archive_url.clone(),
            archive_content_length: profile.archive_bytes,
            compressed_prefix_bytes: prefix_bytes,
            compressed_prefix_sha256,
            http_range_payload_bytes: fetched_bytes,
            full_archive_fraction: fetched_bytes as f64 / profile.archive_bytes as f64,
            selected_block_path: format!("{}-base-selected-block.f32le", profile.dataset_object_id),
            selected_block_sha256,
            selected_row_count: selected_rows.len(),
            selected_rows,
            extracted_mode_count,
            persistent_mode_count,
        },
    })
}

type DecodedBaseRows = (Vec<Vec<f64>>, Vec<u8>, Vec<RowIdentity>);

fn decode_base_rows(profile: &FrozenProfile, compressed: &[u8]) -> Result<DecodedBaseRows, String> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut header = [0_u8; 128];
    decoder
        .read_exact(&mut header)
        .map_err(|error| format!("decompress REALIMPACT transfer NPY header: {error}"))?;
    validate_npy_header_prefix(&header, "<f4", &[3_000, profile.audio_sample_count])?;
    let mut rows = Vec::with_capacity(LISTENER_COUNT);
    let mut payload = Vec::with_capacity(LISTENER_COUNT * profile.audio_sample_count * 4);
    let mut identities = Vec::with_capacity(LISTENER_COUNT);
    for row_index in 0..LISTENER_COUNT {
        let mut bytes = vec![0_u8; profile.audio_sample_count * 4];
        decoder
            .read_exact(&mut bytes)
            .map_err(|error| format!("decompress REALIMPACT transfer row {row_index}: {error}"))?;
        let payload_offset_bytes = payload.len();
        identities.push(RowIdentity {
            row_index,
            payload_offset_bytes,
            sha256: sha256_hex(&bytes),
        });
        rows.push(decode_row(&bytes, row_index)?);
        payload.extend_from_slice(&bytes);
    }
    Ok((rows, payload, identities))
}

fn decode_row(bytes: &[u8], row_index: usize) -> Result<Vec<f64>, String> {
    bytes
        .chunks_exact(4)
        .enumerate()
        .map(|(sample_index, chunk)| {
            let value = f32::from_le_bytes(chunk.try_into().expect("four bytes"));
            value
                .is_finite()
                .then_some(f64::from(value))
                .ok_or_else(|| {
                    format!("REALIMPACT row {row_index} sample {sample_index} is non-finite")
                })
        })
        .collect()
}

fn validate_base_metadata(
    profile: &FrozenProfile,
    entries: &BTreeMap<&'static str, Vec<u8>>,
) -> Result<(), String> {
    let vertex_xyz = f64_array(raw(entries, "vertexXYZ.npy")?, &[3_000, 3])?;
    let listener_xyz = f64_array(raw(entries, "listenerXYZ.npy")?, &[3_000, 3])?;
    let vertex_ids = i64_array(raw(entries, "vertexID.npy")?, &[3_000])?;
    let microphones = i64_array(raw(entries, "micID.npy")?, &[3_000])?;
    let distances = i64_array(raw(entries, "distance.npy")?, &[3_000])?;
    let angles = i64_array(raw(entries, "angle.npy")?, &[3_000])?;
    let (vertices, _, _) = parse_mesh(raw(entries, "transformed.obj")?)?;
    if vertices.len() != profile.expected_mesh_vertex_count
        || vertices.get(profile.expected_impact_vertex_id)
            != Some(&profile.expected_impact_position)
    {
        return Err(format!(
            "REALIMPACT {} mesh identity changed",
            profile.dataset_object_id
        ));
    }
    for microphone in 0..LISTENER_COUNT {
        let coordinate = microphone * 3;
        let expected_listener = [0.23, -0.04345, -0.91 + microphone as f64 / 14.0 * 1.82];
        if vertex_ids[microphone] != profile.expected_impact_vertex_id as i64
            || !approximately_equal(
                &vertex_xyz[coordinate..coordinate + 3],
                &profile.expected_impact_position,
            )
            || microphones[microphone] != microphone as i64
            || distances[microphone] != 0
            || angles[microphone] != 0
            || !approximately_equal(
                &listener_xyz[coordinate..coordinate + 3],
                &expected_listener,
            )
        {
            return Err(format!(
                "REALIMPACT {} base listener identity changed at row {microphone}",
                profile.dataset_object_id
            ));
        }
    }
    Ok(())
}

fn approximately_equal(left: &[f64], right: &[f64]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| (left - right).abs() <= 1.0e-12)
}
