use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use super::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_cli_path,
    resolve_output_path, set_once, sha256_hex,
};

mod calibration;
mod schema;
use self::schema::*;

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-realimpact-geometry-spatial-transfer-preregistration.manifest.v1";
const REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-geometry-spatial-transfer-preregistration.report.v1";
const STUDY_ID: &str = "physical-sound-realimpact-geometry-spatial-transfer";
const REVISION: &str = "v1-preregistration";
const MANIFEST_SHA256: &str = "5be5f195ddc5b124e7220959efc9bb69c0bf469f495b2906e7da9f7515aae576";
const SOURCE_MANIFEST_SCHEMA: &str =
    "nextengine.experimental-realimpact-frequency-spatial-preregistration.manifest.v1";
const SOURCE_ACCESS_STATE: &str = "frequency development blocks and report opened; fresh calibration and preserved holdout deconvolved audio payloads remain unopened";
const ACCESS_STATE: &str = "fourteen prior development/calibration payloads are opened; only central directories, local headers and non-audio metadata are opened for 65_PitcherCeramic and 63_SmallPlanterCeramic; no selected deconvolved audio payload byte from either reserved object is opened";
const CALIBRATION_OBJECT: &str = "65_PitcherCeramic";
const HOLDOUT_OBJECT: &str = "63_SmallPlanterCeramic";
const MAX_MANIFEST_BYTES: usize = 256 * 1024;
const MAX_PREREQUISITE_BYTES: usize = 16 * 1024 * 1024;
static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

const EXPECTED_PREREQUISITES: [ExpectedPrerequisite; 8] = [
    ExpectedPrerequisite {
        id: "realimpact-frequency-calibration-manifest",
        path: "../ps2-realimpact-shape-spatial-v1/frequency-calibration-manifest.json",
        sha256: "92ebe6a7fbb3207dd7d2082076af3209304a0972430d3e59fd24ed8fa99642f8",
        schema: SOURCE_MANIFEST_SCHEMA,
        status: None,
        decision: None,
    },
    ExpectedPrerequisite {
        id: "realimpact-frequency-calibration-report",
        path: "../ps2-realimpact-shape-spatial-v1/frequency-calibration-a/report.json",
        sha256: "42b6605db6537e9c84f31494fd0cbc3acce41b807c00fede2b15d2f879b69983",
        schema: "nextengine.experimental-realimpact-frequency-spatial-calibration.report.v1",
        status: Some("Validated"),
        decision: Some("FrequencyConditionedBandwidthCalibrationRejected"),
    },
    ExpectedPrerequisite {
        id: "realimpact-modal-radiation-manifest",
        path: "../ps2-realimpact-shape-spatial-v1/radiation-development-manifest.json",
        sha256: "686c1d423bc46de81ba66104fbbca90fe8ae8ff865462f5f99cfae850f47a0ff",
        schema: "nextengine.experimental-realimpact-modal-radiation-representation.manifest.v1",
        status: None,
        decision: None,
    },
    ExpectedPrerequisite {
        id: "realimpact-modal-radiation-report",
        path: "../ps2-realimpact-shape-spatial-v1/radiation-development-a/report.json",
        sha256: "7cbf7c59613aad9fce256912355f7f4e5a8e7ce693491b7a5d5797931878cf25",
        schema: "nextengine.experimental-realimpact-modal-radiation-representation.report.v1",
        status: Some("Validated"),
        decision: Some("ComplexMultipoleRepresentationDevelopmentRejected"),
    },
    ExpectedPrerequisite {
        id: "synthetic-fem-bempp-manifest",
        path: "../ps2-bempp-fem-mode-v1/refined-manifest.json",
        sha256: "1adfe3d68ee5c9d806fe2161311a322cb6df2dd6161acbe340369fc9ee5aabf7",
        schema: "nextengine.experimental-physical-sound-bempp-fem-mode.manifest.v1",
        status: None,
        decision: None,
    },
    ExpectedPrerequisite {
        id: "synthetic-fem-bempp-report",
        path: "../ps2-bempp-fem-mode-v1/refined-run-a/report.json",
        sha256: "a71515fa9edf71663986f1a0c8e37627fff499da5eed5b84aff41a69b23da667",
        schema: "nextengine.experimental-physical-sound-bempp-fem-mode.report.v1",
        status: Some("Validated"),
        decision: Some("ElasticFemEigenmodeToBemppCouplingSupported"),
    },
    ExpectedPrerequisite {
        id: "synthetic-full-angular-cooker-manifest",
        path: "../ps2-bempp-fem-mode-v1/cooker-manifest.json",
        sha256: "d4daf0f3fa2e421330634789613f8c40130253bf69088077b140b94291fa4cf1",
        schema: "nextengine.experimental-physical-sound-fem-mode-cooker.manifest.v1",
        status: None,
        decision: None,
    },
    ExpectedPrerequisite {
        id: "synthetic-full-angular-cooker-report",
        path: "../ps2-bempp-fem-mode-v1/cooker-run-a/report.json",
        sha256: "c3101130b11c7f4b6118cf128b037f1ed16ff80d4b5ee2d0fda28551589ab476",
        schema: "nextengine.experimental-physical-sound-fem-mode-cooker.report.v1",
        status: Some("Validated"),
        decision: Some("FullAngularElasticFemModeNearToFarCookerSupported"),
    },
];

const EXPECTED_OBJECTS: [ExpectedObject; 2] = [
    ExpectedObject {
        object_id: CALIBRATION_OBJECT,
        role: "calibration",
        selection_sha256: "65960d7f49af51a8dcdf507d1d0e15e135322c50c370e81c1c7c3099d79b3e1d",
        archive_url: "https://downloads.cs.stanford.edu/viscam/RealImpact/65_PitcherCeramic.zip",
        archive_bytes: 2_379_553_389,
        archive_etag: "6433dab0-8dd51a6d",
        archive_last_modified_http: "Mon, 10 Apr 2023 09:45:20 GMT",
        central_sha256: "bed7116605e187984480c2a807d60c0f10ce838dbe1873d60b01add11253ea0f",
        mesh_entry_name: "65_PitcherCeramic/preprocessed/transformed.obj",
        mesh_raw_sha256: "8c94449de813072f15e026372d5526ee9324dc463a501859f34f03b27053f06c",
        mesh_vertex_count: 48_418,
        audio_entry_name: "65_PitcherCeramic/preprocessed/deconvolved_0db.npy",
        audio_crc32: "53f05f2d",
        audio_data_offset: 4_900_621,
        audio_compressed_bytes: 2_374_012_074,
        audio_uncompressed_bytes: 2_765_640_128,
        audio_sample_count: 230_470,
    },
    ExpectedObject {
        object_id: HOLDOUT_OBJECT,
        role: "holdout",
        selection_sha256: "66324101a530b86a052ca105bc120af0b826c26a4a3a0639fb96d520580dfd13",
        archive_url: "https://downloads.cs.stanford.edu/viscam/RealImpact/63_SmallPlanterCeramic.zip",
        archive_bytes: 2_319_860_240,
        archive_etag: "6433d9a3-8a464210",
        archive_last_modified_http: "Mon, 10 Apr 2023 09:40:51 GMT",
        central_sha256: "a8a0d55bcca6835c7a8866c54960f21e8293be193ba583ab145bccf2e782bcd2",
        mesh_entry_name: "63_SmallPlanterCeramic/preprocessed/transformed.obj",
        mesh_raw_sha256: "b8ff190a13886c8c61c2a2d6f3cd69ba03721918417c4d9838934cd67ecb71ca",
        mesh_vertex_count: 47_732,
        audio_entry_name: "63_SmallPlanterCeramic/preprocessed/deconvolved_0db.npy",
        audio_crc32: "6182e222",
        audio_data_offset: 4_545_702,
        audio_compressed_bytes: 2_315_312_061,
        audio_uncompressed_bytes: 2_504_640_128,
        audio_sample_count: 208_720,
    },
];

pub(super) fn run_cli(
    root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => {
                return Err(format!(
                    "unexpected realimpact-transfer-preregister argument: {flag}"
                ));
            }
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "physical-sound-registry realimpact-transfer-preregister requires --manifest <external-json>"
            .to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "physical-sound-registry realimpact-transfer-preregister requires --output <external-empty-directory>"
            .to_owned()
    })?;
    run(root, &manifest, &output)
}

fn run(root: &Path, manifest_argument: &Path, output_argument: &Path) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT geometry-spatial-transfer preregistration manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "REALIMPACT geometry-spatial-transfer preregistration manifest",
    )?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    if manifest_sha256 == calibration::MANIFEST_SHA256 {
        return calibration::run(root, &manifest_path, &manifest_bytes, output_argument);
    }
    if manifest_sha256 != MANIFEST_SHA256 {
        return Err(format!(
            "REALIMPACT geometry-spatial-transfer manifest hash changed: expected protocol {MANIFEST_SHA256} or Pitcher calibration {}, got {manifest_sha256}",
            calibration::MANIFEST_SHA256
        ));
    }
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse REALIMPACT transfer preregistration: {error}"))?;
    validate_manifest(&manifest)?;

    let base = manifest_path
        .parent()
        .ok_or_else(|| "preregistration manifest has no parent".to_owned())?;
    let mut prerequisite_reports = Vec::with_capacity(manifest.prerequisites.len());
    let mut source_manifest_bytes = None;
    for (index, reference) in manifest.prerequisites.iter().enumerate() {
        let bytes = read_prerequisite(root, base, reference)?;
        validate_prerequisite_payload(reference, &bytes)?;
        if index == 0 {
            source_manifest_bytes = Some(bytes.clone());
        }
        prerequisite_reports.push(PrerequisiteReport {
            id: reference.id.clone(),
            sha256: reference.sha256.clone(),
            byte_count: bytes.len(),
            schema: reference.expected_schema.clone(),
            status: reference.expected_status.clone(),
            decision: reference.expected_decision.clone(),
        });
    }
    validate_source_manifest(
        &manifest,
        &source_manifest_bytes.expect("frozen prerequisite list has a source manifest"),
    )?;

    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "RealSpatialTransferProtocolFrozen",
        claim: "PREREGISTRATION_ONLY / RESERVED_AUDIO_PAYLOAD_BYTES_READ_ZERO / NO_GEOMETRY_SPATIAL_TRANSFER_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        study_id: STUDY_ID,
        revision: REVISION,
        manifest_sha256: MANIFEST_SHA256,
        access_state_at_freeze: &manifest.access_state_at_freeze,
        preregistration_network_requests: 0,
        reserved_audio_payload_bytes_read: 0,
        payload_identity_bound: true,
        calibration_object: &manifest.roles.calibration,
        holdout_object: &manifest.roles.holdout,
        calibration_required_compressed_prefix_bytes: manifest.objects[0]
            .required_compressed_prefix_bytes,
        holdout_required_compressed_prefix_bytes: manifest.objects[1]
            .required_compressed_prefix_bytes,
        anchor_listener_count: manifest.listener_split.anchor_count,
        held_listener_count: manifest.listener_split.held_count,
        minimum_admitted_modes: manifest.mode_admission.minimum_admitted_modes,
        candidate_id: &manifest.candidate.id,
        control_ids: manifest
            .controls
            .iter()
            .map(|control| control.id.as_str())
            .collect(),
        prerequisites: prerequisite_reports,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        next_action: "run the frozen geometry-only preflight twice; only a byte-identical supported preflight may authorize opening the Pitcher calibration prefix, and only a passing immutable calibration may authorize the Planter holdout once",
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish(&output, &manifest_bytes, &report_bytes)?;

    println!(
        "REALIMPACT geometry-spatial-transfer preregistration: {}",
        output.display()
    );
    println!("manifest sha256: {MANIFEST_SHA256}");
    println!("reserved audio payload bytes read: 0");
    println!("calibration object: {}", manifest.roles.calibration);
    println!("holdout object: {}", manifest.roles.holdout);
    println!("anchor listeners: {}", manifest.listener_split.anchor_count);
    println!("held listeners: {}", manifest.listener_split.held_count);
    println!("decision: RealSpatialTransferProtocolFrozen");
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != REVISION
        || manifest.phase != "preregistration"
        || manifest.objective
            != "test whether an object-derived surface-mode field passed through Bempp and the frozen full-angular cooker predicts held REALIMPACT listener transfer better than the fixed coordinate-only RBF control"
        || manifest.access_state_at_freeze != ACCESS_STATE
        || manifest.roles.calibration != CALIBRATION_OBJECT
        || manifest.roles.holdout != HOLDOUT_OBJECT
        || manifest.roles.selection_rule
            != "ascending frozen SHA-256 roster order within the two never-opened ceramic objects; first calibration, second one-shot holdout"
    {
        return Err("REALIMPACT transfer preregistration identity changed".to_owned());
    }
    validate_prerequisites(&manifest.prerequisites)?;
    validate_objects(&manifest.objects)?;
    validate_opening_protocol(&manifest.opening_protocol)?;
    validate_geometry_preflight(&manifest.geometry_preflight)?;
    validate_signal_and_split(&manifest.signal_extractor, &manifest.listener_split)?;
    validate_candidate(&manifest.candidate)?;
    validate_controls(&manifest.controls)?;
    validate_gates(
        &manifest.mode_admission,
        &manifest.condition_gate,
        &manifest.comparison_gate,
    )?;
    validate_fallback_and_claims(manifest)
}

fn validate_prerequisites(prerequisites: &[Prerequisite]) -> Result<(), String> {
    if prerequisites.len() != EXPECTED_PREREQUISITES.len() {
        return Err("REALIMPACT transfer prerequisite count changed".to_owned());
    }
    for (actual, expected) in prerequisites.iter().zip(EXPECTED_PREREQUISITES) {
        if actual.id != expected.id
            || actual.path.as_path() != Path::new(expected.path)
            || actual.sha256 != expected.sha256
            || actual.expected_schema != expected.schema
            || actual.expected_status.as_deref() != expected.status
            || actual.expected_decision.as_deref() != expected.decision
            || actual.path.to_string_lossy().contains("deconvolved_0db")
        {
            return Err(format!(
                "REALIMPACT transfer prerequisite changed: {}",
                actual.id
            ));
        }
    }
    Ok(())
}

fn validate_objects(objects: &[ObjectIdentity]) -> Result<(), String> {
    if objects.len() != EXPECTED_OBJECTS.len() {
        return Err("REALIMPACT reserved object count changed".to_owned());
    }
    for (actual, expected) in objects.iter().zip(EXPECTED_OBJECTS) {
        if actual.object_id != expected.object_id
            || actual.role != expected.role
            || actual.selection_sha256 != expected.selection_sha256
            || actual.archive_url != expected.archive_url
            || actual.archive_bytes != expected.archive_bytes
            || actual.archive_etag != expected.archive_etag
            || actual.archive_last_modified_http != expected.archive_last_modified_http
            || actual.central_sha256 != expected.central_sha256
            || actual.mesh_entry_name != expected.mesh_entry_name
            || actual.mesh_raw_sha256 != expected.mesh_raw_sha256
            || actual.mesh_vertex_count != expected.mesh_vertex_count
            || actual.audio_entry_name != expected.audio_entry_name
            || actual.audio_crc32 != expected.audio_crc32
            || actual.audio_data_offset != expected.audio_data_offset
            || actual.audio_compressed_bytes != expected.audio_compressed_bytes
            || actual.audio_uncompressed_bytes != expected.audio_uncompressed_bytes
            || actual.audio_sample_count != expected.audio_sample_count
            || actual.required_compressed_prefix_bytes != 536_870_912
            || actual.required_compressed_prefix_bytes > actual.audio_compressed_bytes
        {
            return Err(format!(
                "REALIMPACT reserved object identity changed: {}",
                actual.object_id
            ));
        }
    }
    if objects[0].selection_sha256 >= objects[1].selection_sha256 {
        return Err("REALIMPACT reserved role order is not frozen hash order".to_owned());
    }
    Ok(())
}

fn validate_opening_protocol(protocol: &OpeningProtocol) -> Result<(), String> {
    if protocol.preregistration_network_access_allowed
        || !protocol.geometry_preflight_before_audio
        || !protocol.calibration_payload_requires_preregistration_report
        || !protocol.holdout_payload_requires_calibration_pass
        || protocol.holdout_open_count != 1
        || protocol.on_calibration_failure
            != "publish immutable calibration rejection and stop before any holdout payload byte"
        || protocol.on_holdout_failure
            != "publish immutable holdout rejection; do not refit, remap, change thresholds or reuse the holdout as development"
    {
        return Err("REALIMPACT transfer opening protocol changed".to_owned());
    }
    Ok(())
}

fn validate_geometry_preflight(preflight: &GeometryPreflight) -> Result<(), String> {
    if preflight.parser
        != "UTF-8 OBJ v/f only; positive and negative indices resolved per OBJ; triangulated faces required; reject non-finite vertices, invalid indices and zero-area faces"
        || preflight.simplifier
            != "fast-simplification 0.1.13 commit 4a193ef; target_count mode; aggressiveness 7; preserve input face order; record collapse stream"
        || preflight.spectral_face_target != 8_192
        || preflight.bem_face_target != 2_048
        || preflight.field_transfer
            != "simplify spectral mesh to BEM mesh with recorded collapses; area-weight original spectral vertex values by replay mapping; mass-normalize after transfer"
        || preflight.topology_gate
            != "both derived meshes must be one connected orientable two-manifold closed surface with every undirected edge incident to exactly two faces and positive signed-volume orientation"
        || preflight.repeat_requirement
            != "geometry descriptors, collapse streams, eigenvalues, mode vectors after sign canonicalization and cooker input bytes must be byte-identical across two clean runs"
        || preflight.on_failure != "GeometryUnsupportedFallback before calibration audio access"
    {
        return Err("REALIMPACT transfer geometry preflight changed".to_owned());
    }
    Ok(())
}

fn validate_signal_and_split(
    extractor: &SignalExtractor,
    split: &ListenerSplit,
) -> Result<(), String> {
    if extractor.id != "injective-modal-16-fft65536-v2"
        || extractor.sample_rate_hz != 48_000
        || extractor.impact_ordinal != 0
        || extractor.impact_row_count != 600
        || extractor.normalization_selector != "angle=0,distance=0,micID=7"
        || extractor.persistent_definition != "matched_tail_frequency_hz is present"
        || extractor.floor_db != -60.0
        || split.anchor_angles_degrees != [0, 40, 80, 120, 160]
        || split.anchor_distance_offsets_millimetres != [0, 666]
        || split.anchor_microphone_ids != [0, 2, 4, 6, 7, 8, 10, 12, 14]
        || split.anchor_count != 90
        || split.held_rule != "all impact-ordinal-0 rows not selected as anchors"
        || split.held_count != 510
        || split.strata
            != [
                "held-height: anchor angle and distance, held micID",
                "held-angle: non-anchor angle at anchor distance, every micID",
                "held-distance: non-anchor distance, every angle and micID",
            ]
        || split.anchor_count + split.held_count != extractor.impact_row_count
        || split.anchor_angles_degrees.len()
            * split.anchor_distance_offsets_millimetres.len()
            * split.anchor_microphone_ids.len()
            != split.anchor_count
    {
        return Err("REALIMPACT transfer signal/listener split changed".to_owned());
    }
    Ok(())
}

fn validate_candidate(candidate: &Candidate) -> Result<(), String> {
    if candidate.id != "cotangent-biharmonic-normal-mode-bempp-full-angular-v1"
        || candidate.surface_operator
            != "cotangent Laplace-Beltrami stiffness L with barycentric lumped vertex-area mass M; solve L phi=lambda M phi; discard the constant mode; use first 64 nonconstant mass-orthonormal modes"
        || candidate.shell_proxy
            != "free-edge scalar biharmonic normal-displacement proxy with predicted frequency f_hat_hz=alpha_metres_squared_per_second*lambda_per_square_metre; this is not an elastic-shell, thickness, support or material identity claim"
        || candidate.eigensolver
            != "scipy.sparse.linalg.eigsh shift-invert sigma=0, which=LM, k=65, tol=1e-10, maxiter=20000, deterministic all-ones-plus-index-ramp v0; sort by eigenvalue then lexicographic canonical vector hash; sign makes maximum-absolute component positive"
        || candidate.calibration_scale
            != "enumerate alpha=f_measured/lambda for every measured-mode/proxy-mode pair; for each alpha choose the minimum squared-log2-error strictly increasing 16-of-64 assignment by dynamic programming; choose minimum loss, then lower alpha, then lexicographically lower assignment; refit alpha as geometric mean on that assignment once and rerun assignment once"
        || candidate.holdout_mapping
            != "reuse calibration alpha unchanged; choose minimum squared-log2-error strictly increasing assignment by dynamic programming; no alpha refit"
        || candidate.surface_velocity
            != "triangle value is the arithmetic mean of its three transferred phi values and acts along the outward triangle normal; mass-normalize before Bempp"
        || candidate.bempp
            != "Bempp-cl 0.4.2 commit a1eaaef9c40d29f762e5dcd33fb4cedc26f9e1c0; direct Neumann-to-Dirichlet Helmholtz solve at each measured mode frequency; speed of sound 343 m/s; GMRES tolerance 1e-8 and restart 200"
        || candidate.cooker
            != "reuse source manifest d4daf0f3fa2e421330634789613f8c40130253bf69088077b140b94291fa4cf1: fit all 56 directions at radius 2*bbox_diagonal; reject m=0 control; select smallest passing full-real outgoing degree in [2,4,6] using frozen 4L/10L held gates; evaluate selected expansion at exact REALIMPACT listener coordinates"
        || candidate.audio_inputs
            != "calibration uses only 16 normalization-row frequencies for alpha and both phases use no held-listener value for mode mapping or field fitting; spatial audio anchors are consumed only by controls"
    {
        return Err("REALIMPACT transfer candidate changed".to_owned());
    }
    Ok(())
}

fn validate_controls(controls: &[Control]) -> Result<(), String> {
    if controls.len() != 2
        || controls[0].id != "coordinate-only-rbf-sigma052-ridge001-v1"
        || controls[0].definition
            != "fit complex response normalized by angle=0,distance=0,micID=7 on the 90 declared anchor coordinates; sigma 0.52 m; ridge 0.001; predict 510 held coordinates"
        || controls[1].id != "normalization-listener-constant-v1"
        || controls[1].definition
            != "predict 0 dB relative magnitude at every held listener for every admitted mode"
    {
        return Err("REALIMPACT transfer controls changed".to_owned());
    }
    Ok(())
}

fn validate_gates(
    mode: &ModeAdmission,
    condition: &ConditionGate,
    comparison: &ComparisonGate,
) -> Result<(), String> {
    if mode.minimum_admitted_modes != 12
        || mode.maximum_relative_eigen_residual != 1e-8
        || mode.maximum_gmres_residual != 5e-5
        || mode.minimum_reference_magnitude_to_mode_peak != 0.001
        || mode.calibration_maximum_median_frequency_error_octaves != 0.20
        || mode.calibration_maximum_p90_frequency_error_octaves != 0.35
        || mode.holdout_maximum_median_frequency_error_octaves != 0.30
        || mode.holdout_maximum_p90_frequency_error_octaves != 0.50
        || mode.per_mode_failure
            != "use coordinate-only RBF fallback and exclude the mode from candidate aggregate; object fails if fewer than 12 modes remain"
        || !condition.require_every_held_stratum
        || condition.maximum_median_abs_error_db != 7.0
        || condition.maximum_p90_abs_error_db != 18.0
        || condition.maximum_persistent_median_abs_error_db != 8.0
        || condition.minimum_improved_component_fraction_vs_constant != 0.5
        || condition.maximum_median_error_ratio_to_constant != 0.9
        || comparison.calibration_maximum_candidate_to_rbf_median_error_ratio != 0.95
        || comparison.calibration_maximum_p90_regression_db != 2.0
        || comparison.calibration_minimum_improved_component_fraction_vs_rbf != 0.5
        || comparison.holdout_maximum_candidate_to_rbf_median_error_ratio != 0.95
        || comparison.holdout_maximum_p90_regression_db != 2.0
        || comparison.holdout_minimum_improved_component_fraction_vs_rbf != 0.5
        || !comparison.require_byte_identical_repeat_reports
    {
        return Err("REALIMPACT transfer gates changed".to_owned());
    }
    Ok(())
}

fn validate_fallback_and_claims(manifest: &Manifest) -> Result<(), String> {
    if manifest.fallback.mode
        != "coordinate-only-rbf-sigma052-ridge001-v1 for an admitted object and mode with candidate failure"
        || manifest.fallback.object
            != "existing clip baseline when geometry setup, mapping coverage, calibration, holdout or any conjunctive gate fails"
        || !manifest.fallback.no_silent_generalization
        || manifest.allowed_claims
            != [
                "hash_closed_reserved_payload_identity_and_open_order",
                "geometry_spectral_mode_to_bempp_spatial_transfer_on_the_exact_declared_object_condition_only_if_both_calibration_and_holdout_pass",
                "measured_per_mode_and_per_object_fallback_coverage",
            ]
        || manifest.prohibited_claims
            != [
                "elastic_shell_identity",
                "wall_thickness_identity",
                "support_fixture_identity",
                "material_identity",
                "arbitrary_object_or_impact_generalization",
                "absolute_amplitude",
                "perceptual_naturalness",
                "physical_sound_pass",
                "production_corpus_admission",
                "runtime_content_role",
                "neural_training_or_acceleration",
            ]
    {
        return Err("REALIMPACT transfer fallback/claim boundary changed".to_owned());
    }
    Ok(())
}

fn read_prerequisite(
    root: &Path,
    base: &Path,
    reference: &Prerequisite,
) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(
        root,
        &base.join(&reference.path),
        "REALIMPACT transfer prerequisite",
    )?;
    let bytes = read_bounded_file(
        &path,
        MAX_PREREQUISITE_BYTES,
        "REALIMPACT transfer prerequisite",
    )?;
    let actual = sha256_hex(&bytes);
    if actual != reference.sha256 {
        return Err(format!(
            "REALIMPACT transfer prerequisite hash mismatch for {}: expected {}, got {actual}",
            reference.id, reference.sha256
        ));
    }
    Ok(bytes)
}

fn validate_prerequisite_payload(reference: &Prerequisite, bytes: &[u8]) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
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
            "REALIMPACT transfer prerequisite contract changed: {}",
            reference.id
        ));
    }
    if matches!(
        reference.id.as_str(),
        "synthetic-fem-bempp-report" | "synthetic-full-angular-cooker-report"
    ) && !all_gate_booleans_true(value.get("gate"))
    {
        return Err(format!(
            "REALIMPACT transfer positive prerequisite gate is not fully supported: {}",
            reference.id
        ));
    }
    if reference.id == "realimpact-frequency-calibration-report"
        && value.get("claim").and_then(serde_json::Value::as_str)
            != Some(
                "OBJECT_DISJOINT_FREQUENCY_CONDITIONED_CALIBRATION_ONLY / CERAMIC_HOLDOUT_UNOPENED / NO_TRANSFER_ANGLE_DISTANCE_3D_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
            )
    {
        return Err("frequency calibration no longer proves ceramic holdout closure".to_owned());
    }
    Ok(())
}

fn all_gate_booleans_true(gate: Option<&serde_json::Value>) -> bool {
    let Some(fields) = gate.and_then(serde_json::Value::as_object) else {
        return false;
    };
    !fields.is_empty()
        && fields
            .values()
            .all(|value| value.as_bool().is_some_and(|passed| passed))
}

fn validate_source_manifest(manifest: &Manifest, bytes: &[u8]) -> Result<(), String> {
    let source: SourceManifest = serde_json::from_slice(bytes)
        .map_err(|error| format!("parse frozen REALIMPACT source manifest: {error}"))?;
    if source.schema != SOURCE_MANIFEST_SCHEMA
        || source.access_state_at_freeze != SOURCE_ACCESS_STATE
        || source.roles.holdout != [CALIBRATION_OBJECT, HOLDOUT_OBJECT]
    {
        return Err("frozen REALIMPACT source split/access state changed".to_owned());
    }
    for object in &manifest.objects {
        let profile = source
            .archive_profiles
            .iter()
            .find(|profile| profile.dataset_object_id == object.object_id)
            .ok_or_else(|| format!("source profile missing for {}", object.object_id))?;
        let mesh = profile
            .metadata_entries
            .iter()
            .find(|entry| entry.name == object.mesh_entry_name)
            .ok_or_else(|| format!("source mesh entry missing for {}", object.object_id))?;
        if profile.role != "holdout"
            || profile.selection_sha256 != object.selection_sha256
            || profile.archive_url != object.archive_url
            || profile.archive_bytes != object.archive_bytes
            || profile.archive_etag != object.archive_etag
            || profile.archive_last_modified_http != object.archive_last_modified_http
            || profile.central_sha256 != object.central_sha256
            || profile.mesh_vertex_count != object.mesh_vertex_count
            || mesh.raw_sha256 != object.mesh_raw_sha256
            || profile.audio_entry.name != object.audio_entry_name
            || profile.audio_entry.crc32 != object.audio_crc32
            || profile.audio_entry.data_offset != object.audio_data_offset
            || profile.audio_entry.compressed_bytes != object.audio_compressed_bytes
            || profile.audio_entry.uncompressed_bytes != object.audio_uncompressed_bytes
            || profile.audio_entry.sample_count != object.audio_sample_count
        {
            return Err(format!(
                "reserved identity does not match frozen source profile: {}",
                object.object_id
            ));
        }
    }
    Ok(())
}

fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        format!("serialize REALIMPACT transfer preregistration report: {error}")
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn publish(output: &Path, manifest: &[u8], report: &[u8]) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "REALIMPACT transfer preregistration output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-transfer-preregister-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create transfer preregistration staging: {error}"))?;
    let result = (|| {
        fs::write(staging.join("manifest.json"), manifest)
            .map_err(|error| format!("write transfer preregistration manifest: {error}"))?;
        fs::write(staging.join("report.json"), report)
            .map_err(|error| format!("write transfer preregistration report: {error}"))?;
        if output.exists() {
            fs::remove_dir(output).map_err(|error| {
                format!("remove confirmed-empty transfer preregistration output: {error}")
            })?;
        }
        fs::rename(&staging, output)
            .map_err(|error| format!("publish transfer preregistration output: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[derive(Debug, Deserialize)]
struct SourceManifest {
    schema: String,
    access_state_at_freeze: String,
    roles: SourceRoles,
    archive_profiles: Vec<SourceProfile>,
}

#[derive(Debug, Deserialize)]
struct SourceRoles {
    holdout: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SourceProfile {
    dataset_object_id: String,
    role: String,
    selection_sha256: String,
    archive_url: String,
    archive_bytes: u64,
    archive_etag: String,
    archive_last_modified_http: String,
    central_sha256: String,
    audio_entry: SourceEntry,
    metadata_entries: Vec<SourceEntry>,
    mesh_vertex_count: usize,
}

#[derive(Debug, Deserialize)]
struct SourceEntry {
    name: String,
    crc32: String,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
    data_offset: u64,
    #[serde(default)]
    raw_sha256: String,
    #[serde(default)]
    sample_count: usize,
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
    payload_identity_bound: bool,
    calibration_object: &'a str,
    holdout_object: &'a str,
    calibration_required_compressed_prefix_bytes: u64,
    holdout_required_compressed_prefix_bytes: u64,
    anchor_listener_count: usize,
    held_listener_count: usize,
    minimum_admitted_modes: usize,
    candidate_id: &'a str,
    control_ids: Vec<&'a str>,
    prerequisites: Vec<PrerequisiteReport>,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    next_action: &'static str,
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

#[derive(Clone, Copy)]
struct ExpectedPrerequisite {
    id: &'static str,
    path: &'static str,
    sha256: &'static str,
    schema: &'static str,
    status: Option<&'static str>,
    decision: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct ExpectedObject {
    object_id: &'static str,
    role: &'static str,
    selection_sha256: &'static str,
    archive_url: &'static str,
    archive_bytes: u64,
    archive_etag: &'static str,
    archive_last_modified_http: &'static str,
    central_sha256: &'static str,
    mesh_entry_name: &'static str,
    mesh_raw_sha256: &'static str,
    mesh_vertex_count: usize,
    audio_entry_name: &'static str,
    audio_crc32: &'static str,
    audio_data_offset: u64,
    audio_compressed_bytes: u64,
    audio_uncompressed_bytes: u64,
    audio_sample_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_prerequisites_cannot_open_reserved_audio() {
        assert!(EXPECTED_PREREQUISITES.iter().all(|reference| {
            !reference.path.contains("deconvolved_0db") && !reference.id.contains("payload")
        }));
    }

    #[test]
    fn reserved_roles_follow_frozen_hash_order() {
        assert_eq!(EXPECTED_OBJECTS[0].object_id, CALIBRATION_OBJECT);
        assert_eq!(EXPECTED_OBJECTS[1].object_id, HOLDOUT_OBJECT);
        assert!(EXPECTED_OBJECTS[0].selection_sha256 < EXPECTED_OBJECTS[1].selection_sha256);
    }

    #[test]
    fn frozen_listener_partition_is_complete() {
        let anchors = 5 * 2 * 9;
        assert_eq!(anchors, 90);
        assert_eq!(600 - anchors, 510);
    }
}
