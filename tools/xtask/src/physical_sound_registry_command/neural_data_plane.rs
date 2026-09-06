use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

mod evidence_record;
mod row_projection;
#[cfg(test)]
mod tests;

use evidence_record::*;
use row_projection::{
    audit_leakage, project_row, resolve_cached_artifact, validate_lineage_report,
};

use super::{
    FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file, require_empty_output,
    resolve_artifact, resolve_cli_path, resolve_output_path, set_once, sha256_hex,
    validate_file_ref, validate_label, validate_sorted_labels,
};

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.manifest.v2";
const MANIFEST_SCHEMA_V3: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.manifest.v3";
const PROJECTION_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.projection.v2";
const PROJECTION_SCHEMA_V3: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.projection.v3";
const COMMITMENT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.sealed-roles.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-neural-data-plane.report.v1";
const REPORT_SCHEMA_V2: &str = "nextengine.experimental-physical-sound-neural-data-plane.report.v2";
const REPORT_CLAIM: &str =
    "HASH_CLOSED_NEURAL_DATA_PROJECTION_ONLY / NO_MODEL_TRAINING_QUALITY_OR_ADMISSION_AUTHORITY";
const MAX_ROWS: usize = 65_536;
const MAX_LINEAGE_REPORTS: usize = 256;
const MAX_LINEAGE_REPORT_BYTES: usize = 4 * 1024 * 1024;
const NORMAL_SQUARED_MINIMUM: f64 = 0.998_001;
const NORMAL_SQUARED_MAXIMUM: f64 = 1.002_001;
const TEACHER_REPRESENTATION: &str = "sorted-modal-contact-field-v1";
const MAX_TEACHER_MODES: usize = 256;
static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);
const SPLIT_ROLES: [SplitRole; 5] = [
    SplitRole::Train,
    SplitRole::Development,
    SplitRole::Calibration,
    SplitRole::MethodHoldout,
    SplitRole::AdmissionShadow,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DataPlaneVersion {
    V2,
    V3,
}

impl DataPlaneVersion {
    fn from_schema(schema: &str) -> Result<Self, String> {
        match schema {
            MANIFEST_SCHEMA => Ok(Self::V2),
            MANIFEST_SCHEMA_V3 => Ok(Self::V3),
            _ => Err(format!(
                "unsupported physical sound neural data plane schema: {schema}"
            )),
        }
    }

    const fn projection_schema(self) -> &'static str {
        match self {
            Self::V2 => PROJECTION_SCHEMA,
            Self::V3 => PROJECTION_SCHEMA_V3,
        }
    }

    const fn report_schema(self) -> &'static str {
        match self {
            Self::V2 => REPORT_SCHEMA,
            Self::V3 => REPORT_SCHEMA_V2,
        }
    }
}

pub(super) struct Request {
    manifest: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
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
            _ => return Err(format!("unexpected neural-data-plane argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry neural-data-plane requires --manifest <external-json>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry neural-data-plane requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

#[derive(Serialize)]
struct ProjectionFile {
    schema: &'static str,
    projection_id: String,
    revision: String,
    task_scope: &'static str,
    manifest_sha256: String,
    role_scope: Vec<&'static str>,
    lineage_reports: Vec<ProjectedLineageReport>,
    rows: Vec<ProjectedRow>,
}

#[derive(Serialize)]
struct SealedRoleCommitments {
    schema: &'static str,
    projection_id: String,
    revision: String,
    task_scope: &'static str,
    manifest_sha256: String,
    commitments: Vec<SealedRoleCommitment>,
}

#[derive(Serialize)]
struct SealedRoleCommitment {
    split_role: &'static str,
    row_count: usize,
    object_group_count: usize,
    source_group_count: usize,
    canonical_rows_sha256: String,
    materialized: bool,
}

#[derive(Serialize)]
struct DataPlaneReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    projection_id: String,
    revision: String,
    task_scope: &'static str,
    manifest_sha256: String,
    lineage_report_count: usize,
    row_count: usize,
    role_counts: Vec<RoleCountReport>,
    capability_counts: CapabilityCounts,
    #[serde(skip_serializing_if = "Option::is_none")]
    lane_capability_counts: Option<LaneCapabilityCounts>,
    leakage_audit: LeakageAudit,
    emitted_files: Vec<&'static str>,
    model_training_authorized: bool,
    method_holdout_materialized: bool,
    admission_shadow_materialized: bool,
}

#[derive(Serialize)]
struct RoleCountReport {
    split_role: &'static str,
    rows: usize,
    target_context_rows: usize,
    target_query_rows: usize,
    reject_parent_rows: usize,
    object_groups: usize,
    source_groups: usize,
    complete_modal_field_rows: usize,
}

#[derive(Default, Serialize)]
struct CapabilityCounts {
    audio: usize,
    recorded_impact_waveform: usize,
    force_deconvolved_transfer_response: usize,
    material: usize,
    geometry: usize,
    support: usize,
    impact: usize,
    impact_normal: usize,
    listener: usize,
    excitation: usize,
    complete_modal_field: usize,
}

#[derive(Default, Serialize)]
struct LaneCapabilityCounts {
    synthetic_teacher: usize,
    exact_real_transfer: usize,
    identified_real_recording: usize,
    teacher_target: usize,
    lane_contract_complete: usize,
}

#[derive(Serialize)]
struct LeakageAudit {
    result: &'static str,
    object_groups_disjoint: bool,
    source_groups_disjoint: bool,
    recording_parents_disjoint: bool,
    condition_groups_disjoint: bool,
    mutation_parents_disjoint: bool,
    identical_audio_disjoint: bool,
    family_group_cross_role_overlap_count: usize,
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.manifest),
        "neural data plane manifest",
    )?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "neural data plane manifest",
    )?;
    let manifest: NeuralDataPlaneManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "neural data plane manifest has no parent directory".to_owned())?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let built = build_projection(&root, manifest_directory, &manifest, &manifest_sha256)?;

    let fit_json = serialize_json("fit projection", &built.fit_projection)?;
    let calibration_json = serialize_json("calibration projection", &built.calibration_projection)?;
    let commitments_json = serialize_json("sealed role commitments", &built.sealed_commitments)?;
    let report_json = serde_json::to_vec_pretty(&built.report)
        .map_err(|error| format!("serialize neural data plane report: {error}"))?;
    publish_output(
        &output,
        [
            ("calibration-projection.json", calibration_json),
            ("fit-projection.json", fit_json),
            ("report.json", report_json.clone()),
            ("sealed-role-commitments.json", commitments_json),
        ],
    )?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn serialize_json(role: &str, value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value).map_err(|error| format!("serialize {role}: {error}"))
}

fn publish_output(output: &Path, files: [(&str, Vec<u8>); 4]) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "neural data plane output has no parent directory".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-neural-data-plane-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create neural data plane staging: {error}"))?;
    let guard = StagingGuard(staging.clone());
    for (name, bytes) in files {
        fs::write(staging.join(name), bytes)
            .map_err(|error| format!("write neural data plane {name}: {error}"))?;
    }
    if output.exists() {
        fs::remove_dir(output)
            .map_err(|error| format!("remove confirmed-empty neural output: {error}"))?;
    }
    fs::rename(&staging, output)
        .map_err(|error| format!("publish neural data plane output: {error}"))?;
    std::mem::forget(guard);
    Ok(())
}

struct StagingGuard(PathBuf);

impl Drop for StagingGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn validate_manifest(manifest: &NeuralDataPlaneManifest) -> Result<DataPlaneVersion, String> {
    let version = DataPlaneVersion::from_schema(&manifest.schema)?;
    validate_label(&manifest.projection_id, "neural projection id")?;
    validate_label(&manifest.revision, "neural projection revision")?;
    validate_split_policy(&manifest.split_policy)?;
    if manifest.lineage_reports.is_empty() || manifest.lineage_reports.len() > MAX_LINEAGE_REPORTS {
        return Err(format!(
            "lineage report count must be 1..={MAX_LINEAGE_REPORTS}"
        ));
    }
    let mut lineage_ids = BTreeSet::new();
    let mut previous_lineage: Option<&str> = None;
    for lineage in &manifest.lineage_reports {
        validate_label(&lineage.id, "lineage report id")?;
        validate_report_marker(&lineage.expected_schema, "lineage report schema")?;
        validate_report_marker(&lineage.expected_claim, "lineage report claim")?;
        validate_file_ref(&lineage.artifact, "lineage report artifact")?;
        if previous_lineage.is_some_and(|previous| previous >= lineage.id.as_str()) {
            return Err("lineage reports must be strictly sorted by id".to_owned());
        }
        previous_lineage = Some(&lineage.id);
        lineage_ids.insert(lineage.id.as_str());
    }
    if manifest.rows.is_empty() || manifest.rows.len() > MAX_ROWS {
        return Err(format!("neural row count must be 1..={MAX_ROWS}"));
    }
    let mut previous_row: Option<&str> = None;
    for row in &manifest.rows {
        validate_row(row, &lineage_ids, version)?;
        if previous_row.is_some_and(|previous| previous >= row.row_id.as_str()) {
            return Err("neural rows must be strictly sorted by row_id".to_owned());
        }
        previous_row = Some(&row.row_id);
    }
    validate_role_coverage(&manifest.rows)?;
    validate_lane_coverage(&manifest.rows, version)?;
    Ok(version)
}

fn validate_split_policy(policy: &SplitPolicy) -> Result<(), String> {
    if !policy.object_groups_disjoint_across_roles
        || !policy.source_groups_disjoint_across_roles
        || !policy.recording_parents_disjoint_across_roles
        || !policy.condition_groups_disjoint_across_roles
        || !policy.mutation_parents_disjoint_across_roles
        || !policy.identical_audio_disjoint_across_roles
        || !policy.method_holdout_sealed_before_candidate_freeze
        || !policy.admission_shadow_sealed_until_validator_release
    {
        return Err(
            "neural data plane split policy must enable every fail-closed disjointness and sealing rule"
                .to_owned(),
        );
    }
    Ok(())
}

fn validate_row(
    row: &NeuralRow,
    lineage_ids: &BTreeSet<&str>,
    version: DataPlaneVersion,
) -> Result<(), String> {
    for (value, role) in [
        (&row.row_id, "neural row id"),
        (&row.source_group_id, "source group id"),
        (&row.family_group_id, "family group id"),
        (&row.object_group_id, "object group id"),
        (&row.recording_parent_id, "recording parent id"),
        (&row.condition_group_id, "condition group id"),
    ] {
        validate_label(value, role)?;
    }
    if let Some(parent) = &row.mutation_parent_id {
        validate_label(parent, "mutation parent id")?;
    }
    validate_sorted_labels(&row.lineage_report_ids, "lineage report ids")?;
    if let Some(missing) = row
        .lineage_report_ids
        .iter()
        .find(|id| !lineage_ids.contains(id.as_str()))
    {
        return Err(format!(
            "neural row {} references missing lineage report {missing}",
            row.row_id
        ));
    }
    validate_file_ref(&row.audio, "neural row audio")?;
    validate_file_ref(&row.audio_provenance, "neural row audio provenance")?;
    validate_axis_claims(&row.axes)?;
    validate_lane_binding(row, version)
}

fn validate_axis_claims(axes: &AxisClaims) -> Result<(), String> {
    for (claim, role) in [
        (axes.material.as_ref(), "material claim"),
        (axes.support.as_ref(), "support claim"),
    ] {
        if let Some(claim) = claim {
            validate_label(&claim.value_id, role)?;
            validate_file_ref(&claim.evidence, role)?;
        }
    }
    if let Some(claim) = &axes.geometry {
        validate_label(&claim.geometry_id, "geometry claim id")?;
        validate_file_ref(&claim.feature_artifact, "geometry feature artifact")?;
        validate_file_ref(&claim.evidence, "geometry claim evidence")?;
    }
    if let Some(claim) = &axes.impact {
        validate_label(&claim.coordinate_profile, "impact coordinate profile")?;
        validate_finite_vector(claim.point_metres, "impact point")?;
        if let Some(normal) = claim.outward_normal {
            validate_unit_normal(normal)?;
        }
        validate_file_ref(&claim.evidence, "impact claim evidence")?;
    }
    if let Some(claim) = &axes.listener {
        validate_label(&claim.coordinate_profile, "listener coordinate profile")?;
        validate_finite_vector(claim.point_metres, "listener point")?;
        validate_file_ref(&claim.evidence, "listener claim evidence")?;
    }
    if let Some(claim) = &axes.excitation {
        if claim.impulse_newton_seconds.is_none()
            && claim.energy_joules.is_none()
            && claim.force_profile.is_none()
        {
            return Err(
                "excitation claim requires impulse, energy or a force profile artifact".to_owned(),
            );
        }
        for (value, role) in [
            (claim.impulse_newton_seconds, "impact impulse"),
            (claim.energy_joules, "impact energy"),
        ] {
            if value.is_some_and(|number| !number.is_finite() || number < 0.0) {
                return Err(format!("{role} must be finite and non-negative"));
            }
        }
        if let Some(profile) = &claim.force_profile {
            validate_file_ref(profile, "force profile artifact")?;
        }
        validate_file_ref(&claim.evidence, "excitation claim evidence")?;
    }
    if let Some(claim) = &axes.teacher_target {
        if claim.representation_id != TEACHER_REPRESENTATION {
            return Err(format!(
                "teacher target representation must be {TEACHER_REPRESENTATION}"
            ));
        }
        if !(1..=MAX_TEACHER_MODES).contains(&claim.mode_count) {
            return Err(format!(
                "teacher target mode count must be in 1..={MAX_TEACHER_MODES}"
            ));
        }
        validate_file_ref(&claim.modal_parameters, "teacher modal parameters")?;
        validate_file_ref(&claim.contact_gain_field, "teacher contact gain field")?;
        validate_file_ref(&claim.evidence, "teacher target evidence")?;
    }
    Ok(())
}

fn validate_lane_binding(row: &NeuralRow, version: DataPlaneVersion) -> Result<(), String> {
    if version == DataPlaneVersion::V2 {
        if row.evidence_lane.is_some()
            || row.audio_semantics == AudioSemantics::SyntheticModalRender
            || row.axes.teacher_target.is_some()
        {
            return Err("V2 rows cannot carry V3 evidence-lane fields".to_owned());
        }
        return Ok(());
    }
    let lane = row
        .evidence_lane
        .ok_or_else(|| "V3 row requires evidence_lane".to_owned())?;
    match lane {
        EvidenceLane::SyntheticTeacher => {
            if row.audio_semantics != AudioSemantics::SyntheticModalRender {
                return Err(
                    "synthetic_teacher requires synthetic_modal_render semantics".to_owned(),
                );
            }
            if !row_has_complete_physical_axes(row) || row.axes.teacher_target.is_none() {
                return Err(
                    "synthetic_teacher requires every physical axis and teacher_target".to_owned(),
                );
            }
        }
        EvidenceLane::ExactRealTransfer => {
            if row.audio_semantics != AudioSemantics::ForceDeconvolvedTransferResponse {
                return Err(
                    "exact_real_transfer requires force_deconvolved_transfer_response semantics"
                        .to_owned(),
                );
            }
            if row.axes.teacher_target.is_some() {
                return Err("exact_real_transfer forbids teacher_target".to_owned());
            }
        }
        EvidenceLane::IdentifiedRealRecording => {
            if row.audio_semantics != AudioSemantics::RecordedImpactWaveform {
                return Err(
                    "identified_real_recording requires recorded_impact_waveform semantics"
                        .to_owned(),
                );
            }
            if row.axes.teacher_target.is_some() {
                return Err("identified_real_recording forbids teacher_target".to_owned());
            }
        }
    }
    Ok(())
}

fn row_has_complete_physical_axes(row: &NeuralRow) -> bool {
    row.axes.material.is_some()
        && row.axes.geometry.is_some()
        && row.axes.support.is_some()
        && row.axes.impact.is_some()
        && row.axes.listener.is_some()
        && row.axes.excitation.is_some()
}

fn validate_lane_coverage(rows: &[NeuralRow], version: DataPlaneVersion) -> Result<(), String> {
    if version == DataPlaneVersion::V2 {
        return Ok(());
    }
    for lane in [
        EvidenceLane::SyntheticTeacher,
        EvidenceLane::ExactRealTransfer,
        EvidenceLane::IdentifiedRealRecording,
    ] {
        if !rows.iter().any(|row| row.evidence_lane == Some(lane)) {
            return Err(format!(
                "V3 evidence plane requires at least one {} row",
                lane.as_str()
            ));
        }
    }
    Ok(())
}

fn validate_finite_vector(vector: [f64; 3], role: &str) -> Result<(), String> {
    if vector.into_iter().any(|component| !component.is_finite()) {
        return Err(format!("{role} must contain finite coordinates"));
    }
    Ok(())
}

fn validate_report_marker(value: &str, role: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 512
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ')
    {
        return Err(format!("{role} must be 1..=512 printable ASCII characters"));
    }
    Ok(())
}

fn validate_unit_normal(normal: [f64; 3]) -> Result<(), String> {
    validate_finite_vector(normal, "impact normal")?;
    let squared_norm = normal
        .into_iter()
        .map(|component| component * component)
        .sum::<f64>();
    if !(NORMAL_SQUARED_MINIMUM..=NORMAL_SQUARED_MAXIMUM).contains(&squared_norm) {
        return Err(format!(
            "impact normal squared norm must be in {NORMAL_SQUARED_MINIMUM}..={NORMAL_SQUARED_MAXIMUM}"
        ));
    }
    Ok(())
}

fn validate_role_coverage(rows: &[NeuralRow]) -> Result<(), String> {
    for role in SPLIT_ROLES {
        let target_context = rows.iter().filter(|row| row.split_role == role).any(|row| {
            row.corpus_role == CorpusRole::Target && row.sample_role == SampleRole::Context
        });
        let target_query = rows.iter().filter(|row| row.split_role == role).any(|row| {
            row.corpus_role == CorpusRole::Target && row.sample_role == SampleRole::Query
        });
        if !target_context || !target_query {
            return Err(format!(
                "split role {} requires at least one target context and target query row",
                role.as_str()
            ));
        }
    }
    Ok(())
}

struct BuiltProjection {
    fit_projection: ProjectionFile,
    calibration_projection: ProjectionFile,
    sealed_commitments: SealedRoleCommitments,
    report: DataPlaneReport,
}

fn build_projection(
    root: &Path,
    manifest_directory: &Path,
    manifest: &NeuralDataPlaneManifest,
    manifest_sha256: &str,
) -> Result<BuiltProjection, String> {
    let version = DataPlaneVersion::from_schema(&manifest.schema)?;
    let mut artifact_cache = BTreeMap::<(String, String), VerifiedArtifact>::new();
    let mut lineage_reports = BTreeMap::<String, ProjectedLineageReport>::new();
    for lineage in &manifest.lineage_reports {
        let artifact = resolve_cached_artifact(
            root,
            manifest_directory,
            &lineage.artifact,
            "neural lineage report",
            &mut artifact_cache,
        )?;
        validate_lineage_report(root, manifest_directory, lineage)?;
        lineage_reports.insert(
            lineage.id.clone(),
            ProjectedLineageReport {
                id: lineage.id.clone(),
                schema: lineage.expected_schema.clone(),
                claim: lineage.expected_claim.clone(),
                artifact,
            },
        );
    }
    let mut rows = Vec::with_capacity(manifest.rows.len());
    for row in &manifest.rows {
        rows.push(project_row(
            root,
            manifest_directory,
            row,
            &mut artifact_cache,
        )?);
    }
    let leakage_audit = audit_leakage(&rows)?;
    let role_counts = build_role_counts(&rows);
    let capability_counts = build_capability_counts(&rows);
    let lane_capability_counts =
        (version == DataPlaneVersion::V3).then(|| build_lane_capability_counts(&rows));

    let fit_rows = rows
        .iter()
        .filter(|row| matches!(row.split_role, "train" | "development"))
        .cloned()
        .collect::<Vec<_>>();
    let calibration_rows = rows
        .iter()
        .filter(|row| row.split_role == "calibration")
        .cloned()
        .collect::<Vec<_>>();
    let fit_lineage = select_lineage_reports(&fit_rows, &lineage_reports)?;
    let calibration_lineage = select_lineage_reports(&calibration_rows, &lineage_reports)?;
    let sealed_commitments = build_sealed_commitments(manifest, manifest_sha256, &rows)?;
    let decision = if capability_counts.complete_modal_field == rows.len() {
        "DeclaredAxisCoverageComplete"
    } else {
        "DeclaredAxisCoverageIncomplete"
    };
    let common_projection = |role_scope, lineage_reports, rows| ProjectionFile {
        schema: version.projection_schema(),
        projection_id: manifest.projection_id.clone(),
        revision: manifest.revision.clone(),
        task_scope: manifest.task_scope.as_str(),
        manifest_sha256: manifest_sha256.to_owned(),
        role_scope,
        lineage_reports,
        rows,
    };
    Ok(BuiltProjection {
        fit_projection: common_projection(
            vec![SplitRole::Train.as_str(), SplitRole::Development.as_str()],
            fit_lineage,
            fit_rows,
        ),
        calibration_projection: common_projection(
            vec![SplitRole::Calibration.as_str()],
            calibration_lineage,
            calibration_rows,
        ),
        sealed_commitments,
        report: DataPlaneReport {
            schema: version.report_schema(),
            status: "Validated",
            decision,
            claim: REPORT_CLAIM,
            projection_id: manifest.projection_id.clone(),
            revision: manifest.revision.clone(),
            task_scope: manifest.task_scope.as_str(),
            manifest_sha256: manifest_sha256.to_owned(),
            lineage_report_count: manifest.lineage_reports.len(),
            row_count: rows.len(),
            role_counts,
            capability_counts,
            lane_capability_counts,
            leakage_audit,
            emitted_files: vec![
                "calibration-projection.json",
                "fit-projection.json",
                "report.json",
                "sealed-role-commitments.json",
            ],
            model_training_authorized: false,
            method_holdout_materialized: false,
            admission_shadow_materialized: false,
        },
    })
}

fn select_lineage_reports(
    rows: &[ProjectedRow],
    lineage_reports: &BTreeMap<String, ProjectedLineageReport>,
) -> Result<Vec<ProjectedLineageReport>, String> {
    let ids = rows
        .iter()
        .flat_map(|row| row.lineage_report_ids.iter())
        .collect::<BTreeSet<_>>();
    ids.into_iter()
        .map(|id| {
            lineage_reports
                .get(id)
                .cloned()
                .ok_or_else(|| format!("missing verified lineage report {id}"))
        })
        .collect()
}

fn build_sealed_commitments(
    manifest: &NeuralDataPlaneManifest,
    manifest_sha256: &str,
    rows: &[ProjectedRow],
) -> Result<SealedRoleCommitments, String> {
    let mut commitments = Vec::new();
    for role in SPLIT_ROLES.into_iter().filter(|role| role.is_sealed()) {
        let role_rows = rows
            .iter()
            .filter(|row| row.split_role == role.as_str())
            .collect::<Vec<_>>();
        let canonical_rows = serde_json::to_vec(&role_rows)
            .map_err(|error| format!("serialize sealed {} rows: {error}", role.as_str()))?;
        commitments.push(SealedRoleCommitment {
            split_role: role.as_str(),
            row_count: role_rows.len(),
            object_group_count: role_rows
                .iter()
                .map(|row| row.object_group_id.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            source_group_count: role_rows
                .iter()
                .map(|row| row.source_group_id.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            canonical_rows_sha256: sha256_hex(&canonical_rows),
            materialized: false,
        });
    }
    Ok(SealedRoleCommitments {
        schema: COMMITMENT_SCHEMA,
        projection_id: manifest.projection_id.clone(),
        revision: manifest.revision.clone(),
        task_scope: manifest.task_scope.as_str(),
        manifest_sha256: manifest_sha256.to_owned(),
        commitments,
    })
}

fn build_role_counts(rows: &[ProjectedRow]) -> Vec<RoleCountReport> {
    SPLIT_ROLES
        .into_iter()
        .map(|role| {
            let role_rows = rows
                .iter()
                .filter(|row| row.split_role == role.as_str())
                .collect::<Vec<_>>();
            RoleCountReport {
                split_role: role.as_str(),
                rows: role_rows.len(),
                target_context_rows: role_rows
                    .iter()
                    .filter(|row| row.corpus_role == "target" && row.sample_role == "context")
                    .count(),
                target_query_rows: role_rows
                    .iter()
                    .filter(|row| row.corpus_role == "target" && row.sample_role == "query")
                    .count(),
                reject_parent_rows: role_rows
                    .iter()
                    .filter(|row| row.corpus_role == "reject_parent")
                    .count(),
                object_groups: role_rows
                    .iter()
                    .map(|row| row.object_group_id.as_str())
                    .collect::<BTreeSet<_>>()
                    .len(),
                source_groups: role_rows
                    .iter()
                    .map(|row| row.source_group_id.as_str())
                    .collect::<BTreeSet<_>>()
                    .len(),
                complete_modal_field_rows: role_rows
                    .iter()
                    .filter(|row| row.axes.complete_for_modal_field())
                    .count(),
            }
        })
        .collect()
}

fn build_capability_counts(rows: &[ProjectedRow]) -> CapabilityCounts {
    let mut counts = CapabilityCounts::default();
    for row in rows {
        counts.audio += 1;
        counts.recorded_impact_waveform +=
            usize::from(row.audio_semantics == "recorded_impact_waveform");
        counts.force_deconvolved_transfer_response +=
            usize::from(row.audio_semantics == "force_deconvolved_transfer_response");
        counts.material += usize::from(row.axes.material.is_some());
        counts.geometry += usize::from(row.axes.geometry.is_some());
        counts.support += usize::from(row.axes.support.is_some());
        counts.impact += usize::from(row.axes.impact.is_some());
        counts.impact_normal += usize::from(
            row.axes
                .impact
                .as_ref()
                .is_some_and(|impact| impact.outward_normal.is_some()),
        );
        counts.listener += usize::from(row.axes.listener.is_some());
        counts.excitation += usize::from(row.axes.excitation.is_some());
        counts.complete_modal_field += usize::from(row.axes.complete_for_modal_field());
    }
    counts
}

fn build_lane_capability_counts(rows: &[ProjectedRow]) -> LaneCapabilityCounts {
    let mut counts = LaneCapabilityCounts::default();
    for row in rows {
        match row.evidence_lane {
            Some("synthetic_teacher") => counts.synthetic_teacher += 1,
            Some("exact_real_transfer") => counts.exact_real_transfer += 1,
            Some("identified_real_recording") => counts.identified_real_recording += 1,
            _ => continue,
        }
        counts.teacher_target += usize::from(row.axes.teacher_target.is_some());
        counts.lane_contract_complete += 1;
    }
    counts
}
