use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use self::dsp::{CANDIDATES, CANDIDATES_V2, CandidateProfile, TransferAnalysis};
use self::report::{
    ArtifactAccessReport, CalibrationReport, CandidateSelectionReport, GateCheck,
    HoldoutGateReport, RowAnalysisReport, SpatialParticipationReport,
};
use super::{
    FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file, require_empty_output,
    resolve_cli_path, resolve_output_path, set_once, sha256_hex,
};

pub(super) mod dsp;
mod report;
#[cfg(test)]
mod tests;

const MANIFEST_SCHEMA: &str = "nextengine.experimental-realimpact-transfer-calibration.manifest.v1";
const MANIFEST_SCHEMA_V2: &str =
    "nextengine.experimental-realimpact-transfer-calibration.manifest.v2";
const REPORT_SCHEMA: &str = "nextengine.experimental-realimpact-transfer-calibration.report.v1";
const REPORT_SCHEMA_V2: &str = "nextengine.experimental-realimpact-transfer-calibration.report.v2";
const SOURCE_REPORT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-source-feasibility.report.v1";
const STUDY_ID: &str = "physical-sound-realimpact-normalized-transfer-calibration";
const STUDY_REVISION: &str = "v1";
const STUDY_REVISION_V2: &str = "v2";
const SOURCE_REPORT_SHA256: &str =
    "9a591673fb9567c9a343aeca991e62f1cb299f3a1f6f31e15c38d6050d85b9b1";
const SOURCE_PROFILE_ID: &str = "realimpact-normalized-transfer-calibration-v1";
const OBJECTIVE: &str = "select one relative modal-frequency/damping extractor on an object-disjoint calibration row, evaluate it once on an untouched object holdout, and leave the reserved object unopened";
const OBJECTIVE_V2: &str = "select one injective frequency-resolution-separated modal/damping extractor on Shell Plate after the invalid V1 diagnosis, then evaluate it once on the previously unopened Skull Cup";
const REPORT_CLAIM: &str = "RELATIVE_MODAL_FREQUENCY_AND_DAMPING_CALIBRATION_ONLY / SPATIAL_PARTICIPATION_NOT_EVALUABLE / NO_ABSOLUTE_AMPLITUDE_EXACT_DOMAIN_CORPUS_ADMISSION_VALIDATOR_PASS_OR_RUNTIME_AUTHORITY";

const ALLOWED_CAPABILITIES: [&str; 5] = [
    "force_deconvolved_transfer",
    "impact_vertex_identity",
    "listener_grid_identity",
    "real_object_identity",
    "scanned_mesh_geometry",
];
const PROHIBITED_CLAIMS: [&str; 7] = [
    "absolute_amplitude_claim",
    "exact_material_composition_claim",
    "exact_support_fixture_claim",
    "matched_cross_tier_condition_claim",
    "physical_sound_pass",
    "production_corpus_admission",
    "runtime_content_role",
];
const REQUIRED_E2_CAPABILITIES: [&str; 6] = [
    "force_deconvolved_transfer",
    "geometry",
    "impact_position",
    "listener_position",
    "object_identity",
    "real_recording",
];
const UNAVAILABLE_COMPONENTS: [&str; 4] = [
    "force-profile-bytes",
    "material-composition-revision",
    "repeat-recording-identity",
    "support-fixture-revision",
];

const MINIMUM_SELECTED_MODES: usize = 6;
const MINIMUM_PERSISTENT_MODE_RECALL: f64 = 0.50;
const MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS: f64 = 60.0;
const MINIMUM_DECAYING_MODE_FRACTION: f64 = 0.50;
const MAXIMUM_MEDIAN_TAIL_PREDICTION_RMSE_DB: f64 = 24.0;
const V2_MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS: f64 = 40.0;

const EXPECTED_ROWS: [ExpectedRow; 5] = [
    ExpectedRow {
        profile_row_id: "glass-goblet-row-0-v1",
        partition: Partition::Dev,
        inventory_report_sha256: "f33d82e3b72fa2de23c365c6abc0663a58ee668451c8f0b601416608f3a6b6db",
        inventory_id: "ps2-realimpact-glassgoblet-e2-adapter",
        entry_id: "realimpact-glassgoblet-row0000",
        object_id: "realimpact-94-glassgoblet",
        impact_position_id: "mesh-vertex-12351",
        listener_condition_id: "angle-000-distance-0230mm-mic-00",
        sample_count: 208_457,
        audio_sha256: "15c87b87423e71177e9e3b2ffd3fb0b2ea8ab7c5cbff071f519b2ddda3df325b",
    },
    ExpectedRow {
        profile_row_id: "green-goblet-row-0-v1",
        partition: Partition::Dev,
        inventory_report_sha256: "9cab3bcb504dfd161150e7eb868074ad9176a9210019b4ab75a0a49932aadc53",
        inventory_id: "ps2-realimpact-greengoblet-e2-range-v1",
        entry_id: "realimpact-greengoblet-row0000",
        object_id: "realimpact-93-greengoblet",
        impact_position_id: "mesh-vertex-31676",
        listener_condition_id: "angle-000-distance-0230mm-mic-00",
        sample_count: 208_323,
        audio_sha256: "104dd97391bf6319ccbd4dfdf48569be58097bf1f90cf8cea3ae3cb2f7498ec9",
    },
    ExpectedRow {
        profile_row_id: "blue-bowl-row-0-v1",
        partition: Partition::Calibration,
        inventory_report_sha256: "5132aa224fbe81740de4cec321b2e63d9ea6f8f2df8dc69a04d13f9ea107b8b0",
        inventory_id: "ps2-realimpact-blue-bowl-e2-range-v1",
        entry_id: "realimpact-blue-bowl-row0000",
        object_id: "realimpact-6-bowl",
        impact_position_id: "mesh-vertex-35950",
        listener_condition_id: "angle-000-distance-0230mm-mic-00",
        sample_count: 230_215,
        audio_sha256: "2640223087e3c719f6c2175eba35a652941a09f98209aa2acf93e43b37f1aa23",
    },
    ExpectedRow {
        profile_row_id: "shell-plate-row-0-v1",
        partition: Partition::Holdout,
        inventory_report_sha256: "68a9cee2812143b42d3ac72b6497537f824c31b16a5a3eeb3eb851d287613268",
        inventory_id: "ps2-realimpact-shell-plate-e2-range-v1",
        entry_id: "realimpact-shell-plate-row0000",
        object_id: "realimpact-51-shellplate",
        impact_position_id: "mesh-vertex-15341",
        listener_condition_id: "angle-000-distance-0230mm-mic-00",
        sample_count: 210_424,
        audio_sha256: "e795d04f6bfe12414dd6499d2e29f2772f63ce1784f4d5f6ecdda681b0c31219",
    },
    ExpectedRow {
        profile_row_id: "skull-cup-row-0-v1",
        partition: Partition::Reserved,
        inventory_report_sha256: "f393c8bd0ccadd54ffbf767df2c21384ca57cf5457c8b58d0076a6e1e8e6c45a",
        inventory_id: "ps2-realimpact-skull-cup-e2-range-v1",
        entry_id: "realimpact-skull-cup-row0000",
        object_id: "realimpact-60-skullcup",
        impact_position_id: "mesh-vertex-2764",
        listener_condition_id: "angle-000-distance-0230mm-mic-00",
        sample_count: 209_549,
        audio_sha256: "817da17e6f4b069f0a9f367d2c564a82cf14eb254f699b846c6770543672bb41",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CalibrationVersion {
    V1,
    V2,
}

impl CalibrationVersion {
    const fn report_schema(self) -> &'static str {
        match self {
            Self::V1 => REPORT_SCHEMA,
            Self::V2 => REPORT_SCHEMA_V2,
        }
    }

    const fn revision(self) -> &'static str {
        match self {
            Self::V1 => STUDY_REVISION,
            Self::V2 => STUDY_REVISION_V2,
        }
    }

    const fn objective(self) -> &'static str {
        match self {
            Self::V1 => OBJECTIVE,
            Self::V2 => OBJECTIVE_V2,
        }
    }

    const fn candidates(self) -> &'static [CandidateProfile] {
        match self {
            Self::V1 => &CANDIDATES,
            Self::V2 => &CANDIDATES_V2,
        }
    }
}

fn expected_rows(version: CalibrationVersion) -> [ExpectedRow; 5] {
    let mut rows = EXPECTED_ROWS;
    if version == CalibrationVersion::V2 {
        rows[2].partition = Partition::Dev;
        rows[3].partition = Partition::Calibration;
        rows[4].partition = Partition::Holdout;
    }
    rows
}

struct Request {
    manifest: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

pub(super) fn extract_v2_spatial_modes(samples: &[f64]) -> Result<Vec<(f64, bool)>, String> {
    let analysis = dsp::analyze_v2(samples, 48_000, CANDIDATES_V2[2])?;
    Ok(analysis
        .modes
        .into_iter()
        .map(|mode| (mode.frequency_hz, mode.matched_tail_frequency_hz.is_some()))
        .collect())
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected transfer-calibration argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry transfer-calibration requires --manifest <external-json>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry transfer-calibration requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.manifest),
        "transfer calibration manifest",
    )?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "transfer calibration manifest",
    )?;
    let manifest: CalibrationManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    let version = manifest_version(&manifest)?;
    validate_manifest(&manifest, version)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "transfer calibration manifest has no parent directory".to_owned())?;
    let source_report_bytes = read_reference(
        &root,
        base,
        &manifest.source_feasibility_report,
        "source feasibility report",
    )?;
    validate_source_report(&source_report_bytes)?;

    let mut active_rows = Vec::new();
    let mut holdout_declaration = None;
    let mut access = Vec::new();
    for (declaration, expected) in manifest.rows.iter().zip(expected_rows(version)) {
        if expected.partition == Partition::Reserved {
            access.push(ArtifactAccessReport::sealed(expected));
            continue;
        }
        if expected.partition == Partition::Holdout {
            holdout_declaration = Some((declaration, expected));
        } else {
            active_rows.push(load_row(&root, base, declaration, expected)?);
        }
        access.push(ArtifactAccessReport::opened(expected));
    }

    let dev_rows = active_rows
        .iter()
        .filter(|row| row.expected.partition == Partition::Dev)
        .collect::<Vec<_>>();
    let calibration_row = exactly_one(&active_rows, Partition::Calibration)?;
    let mut candidate_results = Vec::new();
    for candidate in version.candidates().iter().copied() {
        let dev = dev_rows
            .iter()
            .map(|row| analyze_transfer(version, row, candidate))
            .collect::<Result<Vec<_>, _>>()?;
        let calibration = analyze_transfer(version, calibration_row, candidate)?;
        candidate_results.push(CandidateSelectionReport {
            profile_id: candidate.id,
            dev_mean_calibration_loss: mean(dev.iter().map(|analysis| analysis.calibration_loss)),
            calibration_loss: calibration.calibration_loss,
            dev_sanity_pass: dev.iter().all(|analysis| analysis.selected_mode_count >= 4),
        });
    }
    let selected_id = select_candidate(&candidate_results)?;
    let selected_profile = version
        .candidates()
        .iter()
        .copied()
        .find(|candidate| candidate.id == selected_id)
        .ok_or_else(|| "selected transfer profile is not frozen".to_owned())?;

    let selected_dev_and_calibration = active_rows
        .iter()
        .map(|row| analyze_row(version, row, selected_profile))
        .collect::<Result<Vec<_>, _>>()?;
    let (holdout_declaration, holdout_expected) = holdout_declaration
        .ok_or_else(|| "transfer calibration requires exactly one holdout row".to_owned())?;
    let holdout_row = load_row(&root, base, holdout_declaration, holdout_expected)?;
    let holdout = analyze_row(version, &holdout_row, selected_profile)?;
    let holdout_gate = evaluate_holdout(version, &holdout.analysis);
    let decision = if holdout_gate.passed {
        "RelativeModalDampingSupportedSpatialUnavailable"
    } else {
        "RelativeModalDampingRejectedSpatialUnavailable"
    };
    let profile = frozen_profile(version);
    let profile_bytes = serde_json::to_vec(&profile).map_err(|error| error.to_string())?;
    let report = CalibrationReport {
        schema: version.report_schema(),
        status: "Validated",
        decision,
        claim: REPORT_CLAIM,
        study_id: STUDY_ID,
        revision: version.revision(),
        source_profile_id: SOURCE_PROFILE_ID,
        manifest_sha256: sha256_hex(&manifest_bytes),
        source_feasibility_report_sha256: sha256_hex(&source_report_bytes),
        calibration_profile_sha256: sha256_hex(&profile_bytes),
        objective: version.objective(),
        absolute_amplitude_used: false,
        selection_policy: selection_policy(version),
        candidate_selection: candidate_results,
        selected_profile_id: selected_profile.id,
        selected_rows: selected_dev_and_calibration,
        holdout_evaluation: holdout,
        holdout_gate,
        spatial_participation: SpatialParticipationReport {
            decision: "NotEvaluableSingleListenerRowPerObject",
            observed_rows_per_object: 1,
            required_rows_per_object: 2,
            claim: "listener-grid identity is published, but one acquired row per object cannot estimate spatial participation",
        },
        artifact_access: access,
        unopened_reserved_profile_row_id: (version == CalibrationVersion::V1)
            .then_some("skull-cup-row-0-v1"),
        allowed_capabilities: ALLOWED_CAPABILITIES.to_vec(),
        prohibited_claims: PROHIBITED_CLAIMS.to_vec(),
        next_action: next_action(version),
    };
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write transfer calibration report: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CalibrationManifest {
    schema: String,
    study_id: String,
    revision: String,
    source_feasibility_report: FileRef,
    rows: Vec<RowDeclaration>,
    allowed_capabilities: Vec<String>,
    prohibited_claims: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowDeclaration {
    profile_row_id: String,
    partition: Partition,
    inventory_report: FileRef,
    audio_payload: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Partition {
    Dev,
    Calibration,
    Holdout,
    Reserved,
}

impl Partition {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Calibration => "calibration",
            Self::Holdout => "holdout",
            Self::Reserved => "reserved",
        }
    }
}

#[derive(Clone, Copy)]
struct ExpectedRow {
    profile_row_id: &'static str,
    partition: Partition,
    inventory_report_sha256: &'static str,
    inventory_id: &'static str,
    entry_id: &'static str,
    object_id: &'static str,
    impact_position_id: &'static str,
    listener_condition_id: &'static str,
    sample_count: usize,
    audio_sha256: &'static str,
}

fn manifest_version(manifest: &CalibrationManifest) -> Result<CalibrationVersion, String> {
    match (manifest.schema.as_str(), manifest.revision.as_str()) {
        (MANIFEST_SCHEMA, STUDY_REVISION) => Ok(CalibrationVersion::V1),
        (MANIFEST_SCHEMA_V2, STUDY_REVISION_V2) => Ok(CalibrationVersion::V2),
        _ => Err("transfer calibration manifest version mismatch".to_owned()),
    }
}

fn validate_manifest(
    manifest: &CalibrationManifest,
    version: CalibrationVersion,
) -> Result<(), String> {
    if manifest.study_id != STUDY_ID {
        return Err("transfer calibration manifest identity mismatch".to_owned());
    }
    if manifest.source_feasibility_report.sha256 != SOURCE_REPORT_SHA256 {
        return Err(
            "transfer calibration requires the frozen source feasibility report".to_owned(),
        );
    }
    require_exact_strings(
        &manifest.allowed_capabilities,
        &ALLOWED_CAPABILITIES,
        "allowed capabilities",
    )?;
    require_exact_strings(
        &manifest.prohibited_claims,
        &PROHIBITED_CLAIMS,
        "prohibited claims",
    )?;
    let expected_rows = expected_rows(version);
    if manifest.rows.len() != expected_rows.len() {
        return Err(format!(
            "transfer calibration requires {} frozen rows",
            expected_rows.len()
        ));
    }
    let mut objects = BTreeSet::new();
    for (declaration, expected) in manifest.rows.iter().zip(expected_rows) {
        if declaration.profile_row_id != expected.profile_row_id
            || declaration.partition != expected.partition
            || declaration.inventory_report.sha256 != expected.inventory_report_sha256
            || declaration.audio_payload.sha256 != expected.audio_sha256
        {
            return Err(format!(
                "frozen row declaration mismatch for {}",
                expected.profile_row_id
            ));
        }
        if !objects.insert(expected.object_id) {
            return Err("transfer calibration object split is not disjoint".to_owned());
        }
    }
    Ok(())
}

fn require_exact_strings(actual: &[String], expected: &[&str], role: &str) -> Result<(), String> {
    let actual = actual.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!("transfer calibration {role} mismatch"));
    }
    Ok(())
}

#[derive(Deserialize)]
struct SourceFeasibilityReport {
    schema: String,
    status: String,
    decision: String,
    open_v1_blockers: Vec<String>,
    next_route: SourceNextRoute,
}

#[derive(Deserialize)]
struct SourceNextRoute {
    decision: String,
    profile_id: String,
    allowed_capabilities: Vec<String>,
    prohibited_claims: Vec<String>,
    smallest_next_action: String,
}

fn validate_source_report(bytes: &[u8]) -> Result<(), String> {
    if sha256_hex(bytes) != SOURCE_REPORT_SHA256 {
        return Err("source feasibility report hash mismatch".to_owned());
    }
    let report: SourceFeasibilityReport = serde_json::from_slice(bytes)
        .map_err(|error| format!("parse source feasibility report: {error}"))?;
    if report.schema != SOURCE_REPORT_SCHEMA
        || report.status != "Validated"
        || report.decision != "ReviewedSourcesCannotCloseV1"
        || report.open_v1_blockers.len() != 8
        || report.next_route.decision != "InternetNativeTransferCandidate"
        || report.next_route.profile_id != SOURCE_PROFILE_ID
        || report.next_route.smallest_next_action
            != "preregister-and-execute-realimpact-normalized-transfer-calibration-v1"
    {
        return Err("source feasibility report does not authorize transfer calibration".to_owned());
    }
    require_exact_strings(
        &report.next_route.allowed_capabilities,
        &ALLOWED_CAPABILITIES,
        "source-report allowed capabilities",
    )?;
    require_exact_strings(
        &report.next_route.prohibited_claims,
        &PROHIBITED_CLAIMS,
        "source-report prohibited claims",
    )
}

#[derive(Deserialize)]
struct InventoryReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    inventory_id: String,
    entry_count: usize,
    adapter_backed_e2_entry_count: usize,
    entries: Vec<InventoryEntry>,
}

#[derive(Deserialize)]
struct InventoryEntry {
    id: String,
    provisional_outcome: String,
    recording_kind: String,
    material_family: String,
    object_id: String,
    impact_position_id: String,
    listener_condition_id: String,
    sample_rate_hz: u32,
    sample_count: usize,
    audio: InventoryAudio,
    unavailable_components: Vec<String>,
    source_adapter_evidence: AdapterEvidence,
}

#[derive(Deserialize)]
struct InventoryAudio {
    format: String,
    sha256: String,
    byte_count: usize,
}

#[derive(Deserialize)]
struct AdapterEvidence {
    schema: String,
    evidence_tier: String,
    validated_capabilities: Vec<String>,
    row_index: usize,
    sample_rate_hz: u32,
    sample_count: usize,
    transfer_sha256: String,
}

#[derive(Clone, Copy)]
struct InventoryIdentity {
    sample_rate_hz: u32,
}

fn validate_inventory_report(
    bytes: &[u8],
    expected: ExpectedRow,
) -> Result<InventoryIdentity, String> {
    if sha256_hex(bytes) != expected.inventory_report_sha256 {
        return Err(format!(
            "{} inventory report hash mismatch",
            expected.profile_row_id
        ));
    }
    let report: InventoryReport = serde_json::from_slice(bytes).map_err(|error| {
        format!(
            "parse {} inventory report: {error}",
            expected.profile_row_id
        )
    })?;
    if report.schema != "nextengine.experimental-physical-sound-corpus-inventory.report.v2"
        || report.status != "Validated"
        || report.decision != "DevelopmentPilotOnly"
        || report.claim != "INVENTORY_AND_PARTITION_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
        || report.inventory_id != expected.inventory_id
        || report.entry_count != 1
        || report.adapter_backed_e2_entry_count != 1
        || report.entries.len() != 1
    {
        return Err(format!(
            "{} inventory report identity mismatch",
            expected.profile_row_id
        ));
    }
    let entry = &report.entries[0];
    if entry.id != expected.entry_id
        || entry.provisional_outcome != "FallbackOutOfDomain"
        || entry.recording_kind != "controlled_real_force_deconvolved_transfer"
        || entry.material_family != "glass"
        || entry.object_id != expected.object_id
        || entry.impact_position_id != expected.impact_position_id
        || entry.listener_condition_id != expected.listener_condition_id
        || entry.sample_rate_hz != 48_000
        || entry.sample_count != expected.sample_count
        || entry.audio.format != "f32_le_mono"
        || entry.audio.sha256 != expected.audio_sha256
        || entry.audio.byte_count != expected.sample_count * 4
        || entry.source_adapter_evidence.schema != "realimpact_force_deconvolved_transfer_v1"
        || entry.source_adapter_evidence.evidence_tier != "E2TransferResponse"
        || entry.source_adapter_evidence.row_index != 0
        || entry.source_adapter_evidence.sample_rate_hz != 48_000
        || entry.source_adapter_evidence.sample_count != expected.sample_count
        || entry.source_adapter_evidence.transfer_sha256 != expected.audio_sha256
    {
        return Err(format!(
            "{} inventory entry mismatch",
            expected.profile_row_id
        ));
    }
    require_exact_strings(
        &entry.source_adapter_evidence.validated_capabilities,
        &REQUIRED_E2_CAPABILITIES,
        "inventory E2 capabilities",
    )?;
    require_exact_strings(
        &entry.unavailable_components,
        &UNAVAILABLE_COMPONENTS,
        "inventory unavailable components",
    )?;
    Ok(InventoryIdentity {
        sample_rate_hz: entry.sample_rate_hz,
    })
}

fn read_reference(
    root: &Path,
    base: &Path,
    reference: &FileRef,
    role: &str,
) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(root, &base.join(&reference.path), role)?;
    let bytes = read_bounded_file(&path, 16 * 1024 * 1024, role)?;
    let actual = sha256_hex(&bytes);
    if actual != reference.sha256 {
        return Err(format!(
            "{role} hash mismatch: expected {}, got {actual}",
            reference.sha256
        ));
    }
    Ok(bytes)
}

struct LoadedRow {
    expected: ExpectedRow,
    inventory: InventoryIdentity,
    samples: Vec<f64>,
}

fn load_row(
    root: &Path,
    base: &Path,
    declaration: &RowDeclaration,
    expected: ExpectedRow,
) -> Result<LoadedRow, String> {
    let inventory_bytes = read_reference(
        root,
        base,
        &declaration.inventory_report,
        "REALIMPACT inventory report",
    )?;
    let inventory = validate_inventory_report(&inventory_bytes, expected)?;
    let audio_bytes = read_reference(
        root,
        base,
        &declaration.audio_payload,
        "REALIMPACT transfer payload",
    )?;
    let samples = dsp::decode_f32le(&audio_bytes)?;
    if samples.len() != expected.sample_count {
        return Err(format!(
            "{} sample count mismatch: expected {}, got {}",
            expected.profile_row_id,
            expected.sample_count,
            samples.len()
        ));
    }
    Ok(LoadedRow {
        expected,
        inventory,
        samples,
    })
}

fn exactly_one(rows: &[LoadedRow], partition: Partition) -> Result<&LoadedRow, String> {
    let matching = rows
        .iter()
        .filter(|row| row.expected.partition == partition)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(format!(
            "transfer calibration requires exactly one {} row",
            partition.as_str()
        ));
    }
    Ok(matching[0])
}

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let values = values.collect::<Vec<_>>();
    values.iter().sum::<f64>() / values.len() as f64
}

fn select_candidate(candidates: &[CandidateSelectionReport]) -> Result<&'static str, String> {
    candidates
        .iter()
        .filter(|candidate| candidate.dev_sanity_pass)
        .min_by(|left, right| {
            left.calibration_loss
                .total_cmp(&right.calibration_loss)
                .then_with(|| left.profile_id.cmp(right.profile_id))
        })
        .map(|candidate| candidate.profile_id)
        .ok_or_else(|| "no transfer calibration candidate passed dev sanity".to_owned())
}

const fn selection_policy(version: CalibrationVersion) -> &'static str {
    match version {
        CalibrationVersion::V1 => {
            "dev rows provide sanity screening; profile argmin uses only the calibration row; holdout bytes are opened only after selection; reserved bytes are not opened"
        }
        CalibrationVersion::V2 => {
            "three previously opened rows provide sanity screening; profile argmin uses the previously opened Shell Plate calibration row; Skull Cup bytes are opened only after selection"
        }
    }
}

const fn next_action(version: CalibrationVersion) -> &'static str {
    match version {
        CalibrationVersion::V1 => {
            "acquire at least two hash-closed listener rows for one untouched REALIMPACT object before spatial calibration; retain this modal/damping result as transfer-only evidence"
        }
        CalibrationVersion::V2 => {
            "acquire at least two hash-closed listener rows for one REALIMPACT object and preregister an injective cross-listener participation test; retain V1 as invalid and V2 as modal/damping-only evidence"
        }
    }
}

fn analyze_transfer(
    version: CalibrationVersion,
    row: &LoadedRow,
    profile: CandidateProfile,
) -> Result<TransferAnalysis, String> {
    match version {
        CalibrationVersion::V1 => dsp::analyze(&row.samples, row.inventory.sample_rate_hz, profile),
        CalibrationVersion::V2 => {
            dsp::analyze_v2(&row.samples, row.inventory.sample_rate_hz, profile)
        }
    }
}

fn analyze_row(
    version: CalibrationVersion,
    row: &LoadedRow,
    profile: CandidateProfile,
) -> Result<RowAnalysisReport, String> {
    Ok(RowAnalysisReport {
        profile_row_id: row.expected.profile_row_id,
        partition: row.expected.partition.as_str(),
        object_id: row.expected.object_id,
        impact_position_id: row.expected.impact_position_id,
        listener_condition_id: row.expected.listener_condition_id,
        transfer_sha256: row.expected.audio_sha256,
        analysis: analyze_transfer(version, row, profile)?,
    })
}

fn evaluate_holdout(version: CalibrationVersion, analysis: &TransferAnalysis) -> HoldoutGateReport {
    let maximum_frequency_error = match version {
        CalibrationVersion::V1 => MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS,
        CalibrationVersion::V2 => V2_MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS,
    };
    let checks = vec![
        GateCheck::minimum_usize(
            "selected_mode_count",
            analysis.selected_mode_count,
            MINIMUM_SELECTED_MODES,
        ),
        GateCheck::minimum_f64(
            "persistent_mode_recall",
            analysis.persistent_mode_recall,
            MINIMUM_PERSISTENT_MODE_RECALL,
        ),
        GateCheck::maximum_f64(
            "median_frequency_error_cents",
            analysis.median_frequency_error_cents,
            maximum_frequency_error,
        ),
        GateCheck::minimum_f64(
            "decaying_mode_fraction",
            analysis.decaying_mode_fraction,
            MINIMUM_DECAYING_MODE_FRACTION,
        ),
        GateCheck::maximum_f64(
            "median_tail_prediction_rmse_db",
            analysis.median_tail_prediction_rmse_db,
            MAXIMUM_MEDIAN_TAIL_PREDICTION_RMSE_DB,
        ),
    ];
    HoldoutGateReport {
        passed: checks.iter().all(|check| check.passed),
        checks,
    }
}

#[derive(Serialize)]
struct FrozenProfileV1 {
    objective: &'static str,
    candidates: [CandidateProfile; 3],
    dsp: dsp::DspProfileReport,
    split: Vec<ProfileSplitRow>,
    holdout_thresholds: HoldoutThresholds,
    allowed_capabilities: Vec<&'static str>,
    prohibited_claims: Vec<&'static str>,
}

#[derive(Serialize)]
struct FrozenProfileV2 {
    objective: &'static str,
    candidates: [CandidateProfile; 3],
    dsp: dsp::DspProfileReportV2,
    split: Vec<ProfileSplitRow>,
    holdout_thresholds: HoldoutThresholds,
    allowed_capabilities: Vec<&'static str>,
    prohibited_claims: Vec<&'static str>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum FrozenProfile {
    V1(FrozenProfileV1),
    V2(FrozenProfileV2),
}

#[derive(Serialize)]
struct ProfileSplitRow {
    profile_row_id: &'static str,
    object_id: &'static str,
    partition: &'static str,
}

#[derive(Serialize)]
struct HoldoutThresholds {
    minimum_selected_modes: usize,
    minimum_persistent_mode_recall: f64,
    maximum_median_frequency_error_cents: f64,
    minimum_decaying_mode_fraction: f64,
    maximum_median_tail_prediction_rmse_db: f64,
}

fn split_profile(version: CalibrationVersion) -> Vec<ProfileSplitRow> {
    expected_rows(version)
        .iter()
        .map(|row| ProfileSplitRow {
            profile_row_id: row.profile_row_id,
            object_id: row.object_id,
            partition: row.partition.as_str(),
        })
        .collect()
}

fn holdout_thresholds(version: CalibrationVersion) -> HoldoutThresholds {
    HoldoutThresholds {
        minimum_selected_modes: MINIMUM_SELECTED_MODES,
        minimum_persistent_mode_recall: MINIMUM_PERSISTENT_MODE_RECALL,
        maximum_median_frequency_error_cents: match version {
            CalibrationVersion::V1 => MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS,
            CalibrationVersion::V2 => V2_MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS,
        },
        minimum_decaying_mode_fraction: MINIMUM_DECAYING_MODE_FRACTION,
        maximum_median_tail_prediction_rmse_db: MAXIMUM_MEDIAN_TAIL_PREDICTION_RMSE_DB,
    }
}

fn frozen_profile(version: CalibrationVersion) -> FrozenProfile {
    match version {
        CalibrationVersion::V1 => FrozenProfile::V1(FrozenProfileV1 {
            objective: OBJECTIVE,
            candidates: CANDIDATES,
            dsp: dsp::profile_report(),
            split: split_profile(version),
            holdout_thresholds: holdout_thresholds(version),
            allowed_capabilities: ALLOWED_CAPABILITIES.to_vec(),
            prohibited_claims: PROHIBITED_CLAIMS.to_vec(),
        }),
        CalibrationVersion::V2 => FrozenProfile::V2(FrozenProfileV2 {
            objective: OBJECTIVE_V2,
            candidates: CANDIDATES_V2,
            dsp: dsp::profile_report_v2(),
            split: split_profile(version),
            holdout_thresholds: holdout_thresholds(version),
            allowed_capabilities: ALLOWED_CAPABILITIES.to_vec(),
            prohibited_claims: PROHIBITED_CLAIMS.to_vec(),
        }),
    }
}
