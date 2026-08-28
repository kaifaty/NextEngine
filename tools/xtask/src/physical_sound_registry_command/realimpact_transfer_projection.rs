use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::realimpact_row::spatial_calibration::dsp as frozen_spatial;
use super::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_cli_path,
    resolve_output_path, set_once, sha256_hex,
};

fn project(
    samples: &[f64],
    frequencies_hz: &[f64],
    sample_rate_hz: u32,
) -> Result<Vec<frozen_spatial::ComplexValue>, String> {
    let onset = frozen_spatial::onset(samples)?;
    let window = frozen_spatial::hann_window();
    frequencies_hz
        .iter()
        .map(|frequency| {
            frozen_spatial::projection_complex(samples, onset, &window, *frequency, sample_rate_hz)
        })
        .collect()
}

const FREQUENCY_SCHEMA: &str = "nextengine.experimental-realimpact-transfer-projection-input.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-realimpact-transfer-projection.report.v1";
const BOUND_SPATIAL_DSP_SHA256: &str =
    "edfe237492f27075a2e3d4176314942ba4ee328b2d32c974aca072cd20b02e9f";
const SAMPLE_RATE_HZ: u32 = 48_000;
const SAMPLE_COUNT: usize = 230_470;
const ROW_COUNT: usize = 600;
const MODE_COUNT: usize = 16;
const ROW_BYTES: usize = SAMPLE_COUNT * 4;
const BLOCK_BYTES: usize = ROW_COUNT * ROW_BYTES;
const MAX_FREQUENCY_BYTES: usize = 64 * 1024;
static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

struct Request {
    block: PathBuf,
    frequencies: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(
    root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    let mut block = None;
    let mut frequencies = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--block" => set_once(&mut block, PathBuf::from(value), &flag)?,
            "--frequencies" => set_once(&mut frequencies, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => {
                return Err(format!(
                    "unexpected realimpact-transfer-project argument: {flag}"
                ));
            }
        }
    }
    run(
        root,
        &Request {
            block: block
                .ok_or_else(|| "realimpact-transfer-project requires --block".to_owned())?,
            frequencies: frequencies
                .ok_or_else(|| "realimpact-transfer-project requires --frequencies".to_owned())?,
            output: output
                .ok_or_else(|| "realimpact-transfer-project requires --output".to_owned())?,
        },
    )
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    validate_bound_source(root)?;
    let block_path = canonical_external_file(
        root,
        &resolve_cli_path(root, &request.block),
        "REALIMPACT Pitcher decoded block",
    )?;
    let metadata = fs::metadata(&block_path)
        .map_err(|error| format!("stat Pitcher decoded block: {error}"))?;
    if metadata.len() != BLOCK_BYTES as u64 {
        return Err(format!(
            "Pitcher decoded block bytes changed: expected {BLOCK_BYTES}, got {}",
            metadata.len()
        ));
    }
    let frequencies_path = canonical_external_file(
        root,
        &resolve_cli_path(root, &request.frequencies),
        "REALIMPACT Pitcher projection input",
    )?;
    let frequency_bytes = read_bounded_file(
        &frequencies_path,
        MAX_FREQUENCY_BYTES,
        "REALIMPACT projection input",
    )?;
    let input: ProjectionInput = serde_json::from_slice(&frequency_bytes)
        .map_err(|error| format!("parse REALIMPACT projection input: {error}"))?;
    validate_input(&input)?;

    let mut reader = BufReader::new(
        File::open(&block_path).map_err(|error| format!("open Pitcher decoded block: {error}"))?,
    );
    let frequencies_hz = input
        .modes
        .iter()
        .map(|value| value.frequency_hz)
        .collect::<Vec<_>>();
    let mut mode_major = vec![vec![frozen_spatial::ComplexValue::ZERO; ROW_COUNT]; MODE_COUNT];
    let mut row_hashes = Vec::with_capacity(ROW_COUNT);
    #[allow(clippy::needless_range_loop)]
    for row_index in 0..ROW_COUNT {
        let mut bytes = vec![0_u8; ROW_BYTES];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| format!("read Pitcher row {row_index}: {error}"))?;
        row_hashes.push(sha256_hex(&bytes));
        let samples = decode_row(&bytes, row_index)?;
        let projected = project(&samples, &frequencies_hz, SAMPLE_RATE_HZ)?;
        for (mode_index, value) in projected.into_iter().enumerate() {
            mode_major[mode_index][row_index] = value;
        }
    }
    let mut trailing = [0_u8; 1];
    if reader
        .read(&mut trailing)
        .map_err(|error| format!("read Pitcher block trailing byte: {error}"))?
        != 0
    {
        return Err("Pitcher decoded block has trailing bytes".to_owned());
    }
    for component in &mut mode_major {
        let reference = component[input.normalization_row_index];
        for value in component {
            *value = value.divide(reference)?;
        }
    }
    let projection_bytes = encode_projection(&mode_major);
    let block_sha256 = hash_file(&block_path)?;
    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "FrozenRustSpatialProjectionComputed",
        claim: "EXACT_BOUND_SPATIAL_DFT_ONLY / NO_NETWORK_OR_HOLDOUT_ACCESS",
        bound_spatial_dsp_sha256: BOUND_SPATIAL_DSP_SHA256,
        input_sha256: sha256_hex(&frequency_bytes),
        block_sha256,
        block_bytes: BLOCK_BYTES,
        sample_rate_hz: SAMPLE_RATE_HZ,
        sample_count: SAMPLE_COUNT,
        row_count: ROW_COUNT,
        mode_count: MODE_COUNT,
        persistent_mode_count: input.modes.iter().filter(|mode| mode.persistent).count(),
        normalization_row_index: input.normalization_row_index,
        row_sha256: row_hashes,
        projection_path: "relative-complex-projection.f64le",
        projection_sha256: sha256_hex(&projection_bytes),
        projection_bytes: projection_bytes.len(),
        network_requests: 0,
        planter_audio_payload_bytes_read: 0,
    };
    let mut report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("serialize REALIMPACT projection report: {error}"))?;
    report_bytes.push(b'\n');
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, &request.output)?;
    require_empty_output(&output)?;
    publish(&output, &frequency_bytes, &projection_bytes, &report_bytes)?;

    println!("REALIMPACT frozen spatial projection: {}", output.display());
    println!("block sha256: {}", report.block_sha256);
    println!("projection sha256: {}", report.projection_sha256);
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn validate_bound_source(root: &Path) -> Result<(), String> {
    let path = root.join(
        "tools/xtask/src/physical_sound_registry_command/realimpact_row/spatial_calibration/dsp.rs",
    );
    let bytes = fs::read(&path)
        .map_err(|error| format!("read frozen spatial DSP {}: {error}", path.display()))?;
    let actual = sha256_hex(&bytes);
    if actual != BOUND_SPATIAL_DSP_SHA256 {
        return Err(format!(
            "frozen spatial DSP hash changed: expected {BOUND_SPATIAL_DSP_SHA256}, got {actual}"
        ));
    }
    Ok(())
}

fn validate_input(input: &ProjectionInput) -> Result<(), String> {
    if input.schema != FREQUENCY_SCHEMA
        || input.object_id != "65_PitcherCeramic"
        || input.sample_rate_hz != SAMPLE_RATE_HZ
        || input.sample_count != SAMPLE_COUNT
        || input.row_count != ROW_COUNT
        || input.normalization_row_index != 7
        || input.extractor_id != "injective-modal-16-fft65536-v2"
        || input.modes.len() != MODE_COUNT
        || input.modes.iter().any(|mode| {
            !mode.frequency_hz.is_finite()
                || mode.frequency_hz < 250.0
                || mode.frequency_hz > 12_000.0
        })
        || input
            .modes
            .windows(2)
            .any(|pair| pair[0].frequency_hz >= pair[1].frequency_hz)
    {
        return Err("REALIMPACT Pitcher projection input changed".to_owned());
    }
    Ok(())
}

fn decode_row(bytes: &[u8], row_index: usize) -> Result<Vec<f64>, String> {
    bytes
        .chunks_exact(4)
        .enumerate()
        .map(|(sample_index, chunk)| {
            let value = f64::from(f32::from_le_bytes(chunk.try_into().expect("four bytes")));
            value.is_finite().then_some(value).ok_or_else(|| {
                format!("Pitcher row {row_index} sample {sample_index} is non-finite")
            })
        })
        .collect()
}

fn encode_projection(mode_major: &[Vec<frozen_spatial::ComplexValue>]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(MODE_COUNT * ROW_COUNT * 16);
    for component in mode_major {
        for value in component {
            bytes.extend_from_slice(&value.real.to_le_bytes());
            bytes.extend_from_slice(&value.imaginary.to_le_bytes());
        }
    }
    bytes
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("hash {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn publish(output: &Path, input: &[u8], projection: &[u8], report: &[u8]) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "REALIMPACT projection output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-projection-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create REALIMPACT projection staging: {error}"))?;
    let result = (|| {
        fs::write(staging.join("input.json"), input)
            .map_err(|error| format!("write projection input: {error}"))?;
        fs::write(
            staging.join("relative-complex-projection.f64le"),
            projection,
        )
        .map_err(|error| format!("write projection block: {error}"))?;
        fs::write(staging.join("report.json"), report)
            .map_err(|error| format!("write projection report: {error}"))?;
        if output.exists() {
            fs::remove_dir(output)
                .map_err(|error| format!("remove confirmed-empty projection output: {error}"))?;
        }
        fs::rename(&staging, output)
            .map_err(|error| format!("publish REALIMPACT projection: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectionInput {
    schema: String,
    object_id: String,
    extractor_id: String,
    sample_rate_hz: u32,
    sample_count: usize,
    row_count: usize,
    normalization_row_index: usize,
    modes: Vec<ModeInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModeInput {
    frequency_hz: f64,
    persistent: bool,
}

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    bound_spatial_dsp_sha256: &'static str,
    input_sha256: String,
    block_sha256: String,
    block_bytes: usize,
    sample_rate_hz: u32,
    sample_count: usize,
    row_count: usize,
    mode_count: usize,
    persistent_mode_count: usize,
    normalization_row_index: usize,
    row_sha256: Vec<String>,
    projection_path: &'static str,
    projection_sha256: String,
    projection_bytes: usize,
    network_requests: usize,
    planter_audio_payload_bytes_read: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_projection_dimensions_are_exact() {
        assert_eq!(BLOCK_BYTES, 553_128_000);
        assert_eq!(MODE_COUNT * ROW_COUNT * 16, 153_600);
    }

    #[test]
    fn included_spatial_projection_finds_a_finite_component() {
        let mut samples = vec![0.0; SAMPLE_COUNT];
        for (index, sample) in samples.iter_mut().enumerate().skip(240) {
            *sample = (2.0 * std::f64::consts::PI * 1_000.0 * (index - 240) as f64
                / f64::from(SAMPLE_RATE_HZ))
            .sin();
        }
        let value = project(&samples, &[1_000.0], SAMPLE_RATE_HZ).expect("project fixture");
        assert_eq!(value.len(), 1);
        assert!(value[0].magnitude() > 1.0);
    }
}
