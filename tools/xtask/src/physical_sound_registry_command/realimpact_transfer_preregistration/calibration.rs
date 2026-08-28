use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{MAX_PREREQUISITE_BYTES, all_gate_booleans_true, pretty_json, publish};
use crate::physical_sound_registry_command::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_output_path,
    sha256_hex,
};

pub(super) const MANIFEST_SHA256: &str =
    "c60621ccf30442ba8fe4c25533325782ef1807c1be98101eae78fb04d18ad4a7";
const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-realimpact-geometry-spatial-transfer-calibration.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-realimpact-geometry-spatial-transfer-calibration-preregistration.report.v1";
const STUDY_ID: &str = "physical-sound-realimpact-geometry-spatial-transfer";
const REVISION: &str = "v1-pitcher-calibration-preregistration";
const OBJECT_ID: &str = "65_PitcherCeramic";
const GEOMETRY_SHA256: &str = "bcd54087f27167cbde16077479004deb6be2f8857cec5af8f3381e85e9539acc";
const GEOMETRY_BYTES: usize = 3_718_246;
const PREFIX_BYTES: u64 = 536_870_912;
const SAMPLE_COUNT: usize = 230_470;
const ROW_COUNT: usize = 600;
const PLANAR_ROW_BYTES: usize = SAMPLE_COUNT * 4;
const DECODED_PAYLOAD_BYTES: usize = ROW_COUNT * PLANAR_ROW_BYTES;
const MAX_GEOMETRY_BYTES: usize = 4 * 1024 * 1024;

const IMPLEMENTATION_FILES: [(&str, &str); 4] = [
    (
        "tools/xtask/src/physical_sound_registry_command/transfer_calibration.rs",
        "2624656ebefbdf20d210ddef3138824219ab8044435dc5440365908a720cbf80",
    ),
    (
        "tools/xtask/src/physical_sound_registry_command/transfer_calibration/dsp.rs",
        "131bbf42d01e633b6cac0c4e0340178f3f5a0a83324bb64630d8423da8179ca4",
    ),
    (
        "tools/xtask/src/physical_sound_registry_command/realimpact_row/spatial_calibration/dsp.rs",
        "f3de418974b8a7676151c5f0d616accd6fc46f854ebcb7a06d8e2acc9ee3f834",
    ),
    (
        "lab/scripts/physical_sound_realimpact_geometry_preflight_v2.py",
        "ac3f0b7688ccf2b57674f0b0ff21ddbc94d369e7539079ba5cdd3481df9a8a36",
    ),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    study_id: String,
    revision: String,
    phase: String,
    objective: String,
    access_state_at_freeze: String,
    prerequisites: Vec<Prerequisite>,
    pitcher_geometry: PitcherGeometry,
    pitcher_audio: PitcherAudio,
    metadata_contract: MetadataContract,
    implementation_binding: ImplementationBinding,
    signal_extractor: serde_json::Value,
    listener_split: ListenerSplit,
    frequency_mapping: FrequencyMapping,
    candidate: Candidate,
    controls: Vec<Control>,
    mode_admission: ModeAdmission,
    condition_gate: serde_json::Value,
    comparison_gate: serde_json::Value,
    opening_protocol: OpeningProtocol,
    fallback: serde_json::Value,
    allowed_claims: Vec<String>,
    prohibited_claims: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Prerequisite {
    id: String,
    path: PathBuf,
    sha256: String,
    expected_schema: String,
    #[serde(default)]
    expected_status: Option<String>,
    #[serde(default)]
    expected_decision: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PitcherGeometry {
    object_id: String,
    role: String,
    path: PathBuf,
    sha256: String,
    bytes: usize,
    format: String,
    bem_vertex_count: usize,
    bem_face_count: usize,
    mode_count: usize,
    eigenvalue_minimum_per_square_metre: f64,
    eigenvalue_maximum_per_square_metre: f64,
    maximum_relative_eigen_residual: f64,
    eigenvalues_sha256: String,
    bem_points_sha256: String,
    bem_faces_sha256: String,
    bem_modes_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PitcherAudio {
    object_id: String,
    archive_url: String,
    archive_bytes: u64,
    archive_etag: String,
    archive_last_modified_http: String,
    central_sha256: String,
    entry_name: String,
    entry_crc32: String,
    entry_data_offset: u64,
    entry_compressed_bytes: u64,
    entry_uncompressed_bytes: u64,
    compressed_prefix_bytes: u64,
    npy_sample_count: usize,
    npy_shape: Vec<usize>,
    npy_dtype: String,
    npy_fortran_order: bool,
    npy_header_bytes: usize,
    decoded_row_range: Vec<usize>,
    decoded_payload_bytes: usize,
    required_deflate_end_condition: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataContract {
    impact_ordinal: usize,
    row_count: usize,
    row_order: String,
    angles_degrees: Vec<u32>,
    distance_offsets_millimetres: Vec<u32>,
    microphone_ids: Vec<usize>,
    normalization_row_index: usize,
    normalization_selector: String,
    validation: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImplementationBinding {
    extractor_id: String,
    rust_entry_sha256: String,
    rust_dsp_sha256: String,
    spatial_projection_dsp_sha256: String,
    geometry_v2_script_sha256: String,
    port_requirement: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListenerSplit {
    anchor_angles_degrees: Vec<u32>,
    anchor_distance_offsets_millimetres: Vec<u32>,
    anchor_microphone_ids: Vec<usize>,
    anchor_count: usize,
    held_rule: String,
    held_count: usize,
    strata: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrequencyMapping {
    proxy_mode_count: usize,
    measured_mode_count: usize,
    model: String,
    calibration: String,
    audio_values_allowed_for_fit: String,
    spatial_audio_values_allowed_for_fit: usize,
    maximum_median_frequency_error_octaves: f64,
    maximum_p90_frequency_error_octaves: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    id: String,
    surface_velocity: String,
    bempp: String,
    cooker: String,
    spatial_fit_inputs: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Control {
    id: String,
    fit: String,
    #[serde(default)]
    sigma_metres: Option<f64>,
    #[serde(default)]
    ridge: Option<f64>,
    predict: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModeAdmission {
    minimum_admitted_modes: usize,
    maximum_relative_eigen_residual: f64,
    maximum_gmres_residual: f64,
    minimum_reference_magnitude_to_mode_peak: f64,
    per_mode_failure: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpeningProtocol {
    preregistration_network_requests_allowed: usize,
    preregistration_reserved_audio_payload_bytes_allowed: usize,
    calibration_object: String,
    calibration_http_range_requests_allowed: usize,
    calibration_compressed_prefix_bytes_exact: u64,
    prefix_growth_allowed: bool,
    retry_with_changed_decoder_or_thresholds_allowed: bool,
    planter_audio_allowed: bool,
    on_prefix_decode_failure: String,
    on_calibration_failure: String,
    on_calibration_pass: String,
}

pub(super) fn run(
    root: &Path,
    manifest_path: &Path,
    manifest_bytes: &[u8],
    output_argument: &Path,
) -> Result<(), String> {
    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("parse Pitcher calibration preregistration: {error}"))?;
    validate_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "Pitcher calibration manifest has no parent".to_owned())?;
    let prerequisites = validate_prerequisites(root, base, &manifest.prerequisites)?;
    let geometry = validate_geometry(root, base, &manifest.pitcher_geometry)?;
    let implementations = validate_implementations(root, &manifest.implementation_binding)?;

    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "PitcherCalibrationProtocolFrozen",
        claim: "CALIBRATION_PREREGISTRATION_ONLY / PITCHER_AND_PLANTER_RESERVED_AUDIO_PAYLOAD_BYTES_READ_ZERO / NO_REAL_SPATIAL_TRANSFER_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        study_id: STUDY_ID,
        revision: REVISION,
        manifest_sha256: MANIFEST_SHA256,
        access_state_at_freeze: &manifest.access_state_at_freeze,
        preregistration_network_requests: 0,
        reserved_audio_payload_bytes_read: 0,
        calibration_object: &manifest.pitcher_audio.object_id,
        geometry,
        compressed_prefix_bytes_exact: manifest.pitcher_audio.compressed_prefix_bytes,
        decoded_row_count: ROW_COUNT,
        decoded_payload_bytes: DECODED_PAYLOAD_BYTES,
        anchor_listener_count: manifest.listener_split.anchor_count,
        held_listener_count: manifest.listener_split.held_count,
        extractor_id: &manifest.implementation_binding.extractor_id,
        proxy_mode_count: manifest.frequency_mapping.proxy_mode_count,
        measured_mode_count: manifest.frequency_mapping.measured_mode_count,
        minimum_admitted_modes: manifest.mode_admission.minimum_admitted_modes,
        candidate_id: &manifest.candidate.id,
        control_ids: manifest
            .controls
            .iter()
            .map(|value| value.id.as_str())
            .collect(),
        prerequisites,
        implementation_files: implementations,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        next_action: "implement the parity-checked bounded calibration runner, then open exactly one 512 MiB Pitcher prefix request and repeat from the cached immutable prefix; Planter remains sealed unless a separate holdout manifest is frozen after calibration passes",
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish(&output, manifest_bytes, &report_bytes)?;

    println!(
        "REALIMPACT Pitcher calibration preregistration: {}",
        output.display()
    );
    println!("manifest sha256: {MANIFEST_SHA256}");
    println!("geometry sha256: {GEOMETRY_SHA256}");
    println!("reserved audio payload bytes read: 0");
    println!("compressed prefix bytes frozen: {PREFIX_BYTES}");
    println!("decision: PitcherCalibrationProtocolFrozen");
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != REVISION
        || manifest.phase != "calibration-preregistration"
        || manifest.objective
            != "test whether the frozen exact-weld Pitcher geometry proxy passed through Bempp and the full-angular cooker predicts 510 held listener responses better than the frozen coordinate-only RBF control"
        || manifest.pitcher_geometry.object_id != OBJECT_ID
        || manifest.pitcher_geometry.role != "calibration"
        || manifest.pitcher_geometry.sha256 != GEOMETRY_SHA256
        || manifest.pitcher_geometry.bytes != GEOMETRY_BYTES
        || manifest.pitcher_geometry.format != "NEPSGEO1 canonical little-endian named-array block"
        || manifest.pitcher_geometry.bem_vertex_count != 1_024
        || manifest.pitcher_geometry.bem_face_count != 2_048
        || manifest.pitcher_geometry.mode_count != 64
        || manifest
            .pitcher_geometry
            .eigenvalue_minimum_per_square_metre
            != 166.1834702333827
        || manifest
            .pitcher_geometry
            .eigenvalue_maximum_per_square_metre
            != 8273.014851559905
        || manifest.pitcher_geometry.maximum_relative_eigen_residual != 3.140759987712305e-13
        || manifest.pitcher_geometry.eigenvalues_sha256
            != "b7428eee1120fb039b5ea4fcd6abab24e07ae07970b78383ac2d4eace0dd7d29"
        || manifest.pitcher_geometry.bem_points_sha256
            != "ff2acf6a175807d7d57f00e7ad6cfcd0df1fba8ae9ae6e1a3fecc5377f68a02f"
        || manifest.pitcher_geometry.bem_faces_sha256
            != "19d19bdb40bbdb0177329cca580c2d03d57fc4abe8ffa2dfde8e78dc2d1c8590"
        || manifest.pitcher_geometry.bem_modes_sha256
            != "c01e3a21554728010ac9c2ccf2d429012f18aeaade6ca2666c0764ad2f57c16a"
    {
        return Err("Pitcher calibration identity or geometry changed".to_owned());
    }
    validate_audio(&manifest.pitcher_audio)?;
    validate_metadata(&manifest.metadata_contract)?;
    validate_split_and_mapping(&manifest.listener_split, &manifest.frequency_mapping)?;
    validate_candidate_and_controls(manifest)?;
    validate_opening(&manifest.opening_protocol)?;
    let _ = (
        &manifest.signal_extractor,
        &manifest.condition_gate,
        &manifest.comparison_gate,
        &manifest.fallback,
    );
    Ok(())
}

fn validate_audio(audio: &PitcherAudio) -> Result<(), String> {
    if audio.object_id != OBJECT_ID
        || audio.archive_url
            != "https://downloads.cs.stanford.edu/viscam/RealImpact/65_PitcherCeramic.zip"
        || audio.archive_bytes != 2_379_553_389
        || audio.archive_etag != "6433dab0-8dd51a6d"
        || audio.archive_last_modified_http != "Mon, 10 Apr 2023 09:45:20 GMT"
        || audio.central_sha256
            != "bed7116605e187984480c2a807d60c0f10ce838dbe1873d60b01add11253ea0f"
        || audio.entry_name != "65_PitcherCeramic/preprocessed/deconvolved_0db.npy"
        || audio.entry_crc32 != "53f05f2d"
        || audio.entry_data_offset != 4_900_621
        || audio.entry_compressed_bytes != 2_374_012_074
        || audio.entry_uncompressed_bytes != 2_765_640_128
        || audio.compressed_prefix_bytes != PREFIX_BYTES
        || audio.npy_sample_count != SAMPLE_COUNT
        || audio.npy_shape != [3_000, SAMPLE_COUNT]
        || audio.npy_dtype != "<f4"
        || audio.npy_fortran_order
        || audio.npy_header_bytes != 128
        || audio.decoded_row_range != [0, ROW_COUNT]
        || audio.decoded_payload_bytes != DECODED_PAYLOAD_BYTES
        || audio.entry_uncompressed_bytes as usize != 128 + 3_000 * PLANAR_ROW_BYTES
        || audio.required_deflate_end_condition
            != "the fixed compressed prefix must decode the complete 128-byte NPY header and exactly rows 0 through 599; incomplete row 599 rejects calibration without another request or prefix increase"
    {
        return Err("Pitcher calibration audio decoder contract changed".to_owned());
    }
    Ok(())
}

fn validate_metadata(metadata: &MetadataContract) -> Result<(), String> {
    if metadata.impact_ordinal != 0
        || metadata.row_count != ROW_COUNT
        || metadata.row_order != "angle major, then distance, then micID"
        || metadata.angles_degrees != [0, 20, 40, 60, 80, 100, 120, 140, 160, 180]
        || metadata.distance_offsets_millimetres != [0, 333, 666, 1_000]
        || metadata.microphone_ids != (0..15).collect::<Vec<_>>()
        || metadata.normalization_row_index != 7
        || metadata.normalization_selector != "angle=0,distance=0,micID=7"
        || metadata.validation
            != "the published angle, distance, micID, vertexID, vertexXYZ and listenerXYZ metadata arrays must reproduce every selected row identity before signal analysis"
        || metadata.angles_degrees.len()
            * metadata.distance_offsets_millimetres.len()
            * metadata.microphone_ids.len()
            != ROW_COUNT
    {
        return Err("Pitcher calibration metadata contract changed".to_owned());
    }
    Ok(())
}

fn validate_split_and_mapping(
    split: &ListenerSplit,
    mapping: &FrequencyMapping,
) -> Result<(), String> {
    if split.anchor_angles_degrees != [0, 40, 80, 120, 160]
        || split.anchor_distance_offsets_millimetres != [0, 666]
        || split.anchor_microphone_ids != [0, 2, 4, 6, 7, 8, 10, 12, 14]
        || split.anchor_count != 90
        || split.held_rule != "all impact-ordinal-0 rows not selected as anchors"
        || split.held_count != 510
        || split.strata.len() != 3
        || split.anchor_count + split.held_count != ROW_COUNT
        || mapping.proxy_mode_count != 64
        || mapping.measured_mode_count != 16
        || mapping.model != "f_hat_hz=alpha_metres_squared_per_second*lambda_per_square_metre"
        || mapping.calibration
            != "enumerate alpha=f_measured/lambda for every 16x64 pair; for each alpha use dynamic programming for the minimum squared-log2-error strictly increasing 16-of-64 assignment; tie-break by lower alpha then lexicographically lower assignment; refit alpha once as the geometric mean of f_measured/lambda on the assignment and rerun assignment once"
        || mapping.audio_values_allowed_for_fit
            != "the 16 frequencies from normalization row 7 only"
        || mapping.spatial_audio_values_allowed_for_fit != 0
        || mapping.maximum_median_frequency_error_octaves != 0.20
        || mapping.maximum_p90_frequency_error_octaves != 0.35
    {
        return Err("Pitcher calibration split or frequency mapping changed".to_owned());
    }
    Ok(())
}

fn validate_candidate_and_controls(manifest: &Manifest) -> Result<(), String> {
    let candidate = &manifest.candidate;
    let admission = &manifest.mode_admission;
    if candidate.id != "cotangent-biharmonic-normal-mode-bempp-full-angular-v1"
        || !candidate
            .surface_velocity
            .contains("outward triangle normal")
        || !candidate.bempp.contains("Bempp-cl 0.4.2 commit a1eaaef")
        || !candidate
            .cooker
            .contains("d4daf0f3fa2e421330634789613f8c40130253bf69088077b140b94291fa4cf1")
        || !candidate
            .spatial_fit_inputs
            .ends_with("no spatial audio anchor or held value")
        || manifest.controls.len() != 2
        || manifest.controls[0].id != "coordinate-only-rbf-sigma052-ridge001-v1"
        || manifest.controls[0].fit
            != "complex response normalized by row 7 on the 90 declared anchor coordinates"
        || manifest.controls[0].sigma_metres != Some(0.52)
        || manifest.controls[0].ridge != Some(0.001)
        || manifest.controls[0].predict != "the 510 held coordinates"
        || manifest.controls[1].id != "normalization-listener-constant-v1"
        || manifest.controls[1].fit != "none"
        || manifest.controls[1].sigma_metres.is_some()
        || manifest.controls[1].ridge.is_some()
        || manifest.controls[1].predict != "0 dB relative magnitude at every held listener"
        || admission.minimum_admitted_modes != 12
        || admission.maximum_relative_eigen_residual != 1e-8
        || admission.maximum_gmres_residual != 5e-5
        || admission.minimum_reference_magnitude_to_mode_peak != 0.001
        || !admission.per_mode_failure.contains("fewer than 12 modes")
    {
        return Err("Pitcher calibration candidate, controls or admission changed".to_owned());
    }
    Ok(())
}

fn validate_opening(protocol: &OpeningProtocol) -> Result<(), String> {
    if protocol.preregistration_network_requests_allowed != 0
        || protocol.preregistration_reserved_audio_payload_bytes_allowed != 0
        || protocol.calibration_object != OBJECT_ID
        || protocol.calibration_http_range_requests_allowed != 1
        || protocol.calibration_compressed_prefix_bytes_exact != PREFIX_BYTES
        || protocol.prefix_growth_allowed
        || protocol.retry_with_changed_decoder_or_thresholds_allowed
        || protocol.planter_audio_allowed
        || !protocol
            .on_prefix_decode_failure
            .contains("without another Pitcher request")
        || !protocol
            .on_calibration_failure
            .ends_with("Planter remains sealed")
        || !protocol
            .on_calibration_pass
            .contains("separate Planter one-shot holdout manifest")
    {
        return Err("Pitcher calibration opening protocol changed".to_owned());
    }
    Ok(())
}

fn validate_prerequisites(
    root: &Path,
    base: &Path,
    references: &[Prerequisite],
) -> Result<Vec<PrerequisiteReport>, String> {
    if references.len() != 5
        || references.iter().any(|value| {
            value.path.to_string_lossy().contains("deconvolved_0db")
                || value.id.contains("audio-payload")
        })
    {
        return Err("Pitcher calibration prerequisite closure changed".to_owned());
    }
    let mut reports = Vec::with_capacity(references.len());
    for reference in references {
        let path = canonical_external_file(
            root,
            &base.join(&reference.path),
            "Pitcher calibration prerequisite",
        )?;
        let bytes = read_bounded_file(
            &path,
            MAX_PREREQUISITE_BYTES,
            "Pitcher calibration prerequisite",
        )?;
        let actual = sha256_hex(&bytes);
        if actual != reference.sha256 {
            return Err(format!(
                "Pitcher calibration prerequisite hash changed for {}: expected {}, got {actual}",
                reference.id, reference.sha256
            ));
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("parse prerequisite {}: {error}", reference.id))?;
        if value.get("schema").and_then(serde_json::Value::as_str)
            != Some(reference.expected_schema.as_str())
            || reference
                .expected_status
                .as_deref()
                .is_some_and(|expected| {
                    value.get("status").and_then(serde_json::Value::as_str) != Some(expected)
                })
            || reference
                .expected_decision
                .as_deref()
                .is_some_and(|expected| {
                    value.get("decision").and_then(serde_json::Value::as_str) != Some(expected)
                })
        {
            return Err(format!(
                "Pitcher calibration prerequisite contract changed for {}",
                reference.id
            ));
        }
        if reference.id == "exact-weld-geometry-v2-report"
            && (value
                .get("reserved_audio_payload_bytes_read")
                .and_then(serde_json::Value::as_u64)
                != Some(0)
                || !all_gate_booleans_true(value.get("gate")))
        {
            return Err("exact-weld V2 no longer proves supported zero-audio geometry".to_owned());
        }
        reports.push(PrerequisiteReport {
            id: reference.id.clone(),
            sha256: reference.sha256.clone(),
            byte_count: bytes.len(),
            schema: reference.expected_schema.clone(),
            status: reference.expected_status.clone(),
            decision: reference.expected_decision.clone(),
        });
    }
    Ok(reports)
}

fn validate_geometry(
    root: &Path,
    base: &Path,
    geometry: &PitcherGeometry,
) -> Result<GeometryReport, String> {
    let path = canonical_external_file(
        root,
        &base.join(&geometry.path),
        "Pitcher exact-weld geometry block",
    )?;
    let bytes = read_bounded_file(&path, MAX_GEOMETRY_BYTES, "Pitcher geometry block")?;
    let actual = sha256_hex(&bytes);
    if bytes.len() != GEOMETRY_BYTES || actual != GEOMETRY_SHA256 || !bytes.starts_with(b"NEPSGEO1")
    {
        return Err(format!(
            "Pitcher exact-weld geometry block changed: expected {GEOMETRY_SHA256}/{GEOMETRY_BYTES}, got {actual}/{}",
            bytes.len()
        ));
    }
    Ok(GeometryReport {
        path: geometry.path.to_string_lossy().into_owned(),
        sha256: actual,
        byte_count: bytes.len(),
        format_magic: "NEPSGEO1",
        mode_count: geometry.mode_count,
        maximum_relative_eigen_residual: geometry.maximum_relative_eigen_residual,
    })
}

fn validate_implementations(
    root: &Path,
    binding: &ImplementationBinding,
) -> Result<Vec<ImplementationReport<'static>>, String> {
    let expected_in_manifest = [
        binding.rust_entry_sha256.as_str(),
        binding.rust_dsp_sha256.as_str(),
        binding.spatial_projection_dsp_sha256.as_str(),
        binding.geometry_v2_script_sha256.as_str(),
    ];
    if binding.extractor_id != "injective-modal-16-fft65536-v2"
        || !binding.port_requirement.contains("numeric parity")
    {
        return Err("Pitcher calibration implementation binding changed".to_owned());
    }
    let mut reports = Vec::with_capacity(IMPLEMENTATION_FILES.len());
    for ((path, expected), manifest_hash) in IMPLEMENTATION_FILES.iter().zip(expected_in_manifest) {
        if manifest_hash != *expected {
            return Err(format!(
                "Pitcher calibration implementation manifest hash changed: {path}"
            ));
        }
        let bytes = fs::read(root.join(path))
            .map_err(|error| format!("read bound implementation {path}: {error}"))?;
        let actual = sha256_hex(&bytes);
        if actual != *expected {
            return Err(format!(
                "Pitcher calibration implementation changed for {path}: expected {expected}, got {actual}"
            ));
        }
        reports.push(ImplementationReport {
            path,
            sha256: expected,
            byte_count: bytes.len(),
        });
    }
    Ok(reports)
}

#[derive(Serialize)]
struct Report<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    manifest_sha256: &'static str,
    access_state_at_freeze: &'a str,
    preregistration_network_requests: usize,
    reserved_audio_payload_bytes_read: usize,
    calibration_object: &'a str,
    geometry: GeometryReport,
    compressed_prefix_bytes_exact: u64,
    decoded_row_count: usize,
    decoded_payload_bytes: usize,
    anchor_listener_count: usize,
    held_listener_count: usize,
    extractor_id: &'a str,
    proxy_mode_count: usize,
    measured_mode_count: usize,
    minimum_admitted_modes: usize,
    candidate_id: &'a str,
    control_ids: Vec<&'a str>,
    prerequisites: Vec<PrerequisiteReport>,
    implementation_files: Vec<ImplementationReport<'static>>,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    next_action: &'static str,
}

#[derive(Serialize)]
struct GeometryReport {
    path: String,
    sha256: String,
    byte_count: usize,
    format_magic: &'static str,
    mode_count: usize,
    maximum_relative_eigen_residual: f64,
}

#[derive(Serialize)]
struct PrerequisiteReport {
    id: String,
    sha256: String,
    byte_count: usize,
    schema: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<String>,
}

#[derive(Serialize)]
struct ImplementationReport<'a> {
    path: &'a str,
    sha256: &'a str,
    byte_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_pitcher_decoder_dimensions_are_exact() {
        assert_eq!(DECODED_PAYLOAD_BYTES, 553_128_000);
        assert_eq!(128 + 3_000 * PLANAR_ROW_BYTES, 2_765_640_128);
    }

    #[test]
    fn calibration_preregistration_binds_no_audio_path() {
        assert!(
            IMPLEMENTATION_FILES
                .iter()
                .all(|(path, _)| !path.contains("deconvolved_0db"))
        );
    }
}
