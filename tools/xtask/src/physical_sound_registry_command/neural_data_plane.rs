use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

mod row_projection;
#[cfg(test)]
mod tests;

use row_projection::{
    audit_leakage, project_row, resolve_cached_artifact, validate_lineage_report,
};

use super::{
    ArtifactReport, FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file,
    require_empty_output, resolve_artifact, resolve_cli_path, resolve_output_path, set_once,
    sha256_hex, validate_file_ref, validate_label, validate_sorted_labels,
};

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.manifest.v1";
const PROJECTION_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.projection.v1";
const COMMITMENT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.sealed-roles.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-neural-data-plane.report.v1";
const REPORT_CLAIM: &str =
    "HASH_CLOSED_NEURAL_DATA_PROJECTION_ONLY / NO_MODEL_TRAINING_QUALITY_OR_ADMISSION_AUTHORITY";
const MAX_ROWS: usize = 65_536;
const MAX_LINEAGE_REPORTS: usize = 256;
const MAX_LINEAGE_REPORT_BYTES: usize = 4 * 1024 * 1024;
const NORMAL_SQUARED_MINIMUM: f64 = 0.998_001;
const NORMAL_SQUARED_MAXIMUM: f64 = 1.002_001;
const SPLIT_ROLES: [SplitRole; 5] = [
    SplitRole::Train,
    SplitRole::Development,
    SplitRole::Calibration,
    SplitRole::MethodHoldout,
    SplitRole::AdmissionShadow,
];

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NeuralDataPlaneManifest {
    schema: String,
    projection_id: String,
    revision: String,
    task_scope: TaskScope,
    split_policy: SplitPolicy,
    lineage_reports: Vec<LineageReport>,
    rows: Vec<NeuralRow>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum TaskScope {
    ExactObjectFewShotImpactListenerField,
}

impl TaskScope {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ExactObjectFewShotImpactListenerField => {
                "exact_object_few_shot_impact_listener_field"
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SplitPolicy {
    object_groups_disjoint_across_roles: bool,
    source_groups_disjoint_across_roles: bool,
    recording_parents_disjoint_across_roles: bool,
    condition_groups_disjoint_across_roles: bool,
    mutation_parents_disjoint_across_roles: bool,
    identical_audio_disjoint_across_roles: bool,
    method_holdout_sealed_before_candidate_freeze: bool,
    admission_shadow_sealed_until_validator_release: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LineageReport {
    id: String,
    expected_schema: String,
    expected_claim: String,
    artifact: FileRef,
}

#[derive(Deserialize)]
struct LineageReportProbe {
    schema: String,
    status: String,
    claim: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NeuralRow {
    row_id: String,
    split_role: SplitRole,
    sample_role: SampleRole,
    corpus_role: CorpusRole,
    source_group_id: String,
    family_group_id: String,
    object_group_id: String,
    recording_parent_id: String,
    condition_group_id: String,
    #[serde(default)]
    mutation_parent_id: Option<String>,
    lineage_report_ids: Vec<String>,
    audio: FileRef,
    audio_provenance: FileRef,
    #[serde(default)]
    axes: AxisClaims,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum SplitRole {
    Train,
    Development,
    Calibration,
    MethodHoldout,
    AdmissionShadow,
}

impl SplitRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Development => "development",
            Self::Calibration => "calibration",
            Self::MethodHoldout => "method_holdout",
            Self::AdmissionShadow => "admission_shadow",
        }
    }

    const fn is_sealed(self) -> bool {
        matches!(self, Self::MethodHoldout | Self::AdmissionShadow)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SampleRole {
    Context,
    Query,
}

impl SampleRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Query => "query",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CorpusRole {
    Target,
    RejectParent,
}

impl CorpusRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Target => "target",
            Self::RejectParent => "reject_parent",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AxisClaims {
    #[serde(default)]
    material: Option<LabelClaim>,
    #[serde(default)]
    geometry: Option<GeometryClaim>,
    #[serde(default)]
    support: Option<LabelClaim>,
    #[serde(default)]
    impact: Option<ImpactClaim>,
    #[serde(default)]
    listener: Option<ListenerClaim>,
    #[serde(default)]
    excitation: Option<ExcitationClaim>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LabelClaim {
    value_id: String,
    evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GeometryClaim {
    geometry_id: String,
    feature_artifact: FileRef,
    evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ImpactClaim {
    coordinate_profile: String,
    point_metres: [f64; 3],
    outward_normal: [f64; 3],
    evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ListenerClaim {
    coordinate_profile: String,
    point_metres: [f64; 3],
    evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExcitationClaim {
    #[serde(default)]
    impulse_newton_seconds: Option<f64>,
    #[serde(default)]
    energy_joules: Option<f64>,
    #[serde(default)]
    force_profile: Option<FileRef>,
    evidence: FileRef,
}

#[derive(Clone, Debug, Serialize)]
struct VerifiedArtifact {
    sha256: String,
    byte_count: usize,
}

impl From<ArtifactReport> for VerifiedArtifact {
    fn from(report: ArtifactReport) -> Self {
        Self {
            sha256: report.sha256,
            byte_count: report.byte_count,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ProjectedLineageReport {
    id: String,
    schema: String,
    claim: String,
    artifact: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
struct ProjectedRow {
    row_id: String,
    split_role: &'static str,
    sample_role: &'static str,
    corpus_role: &'static str,
    source_group_id: String,
    family_group_id: String,
    object_group_id: String,
    recording_parent_id: String,
    condition_group_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mutation_parent_id: Option<String>,
    lineage_report_ids: Vec<String>,
    audio: VerifiedArtifact,
    audio_provenance: VerifiedArtifact,
    axes: ProjectedAxes,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ProjectedAxes {
    #[serde(skip_serializing_if = "Option::is_none")]
    material: Option<ProjectedLabelClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ProjectedGeometryClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    support: Option<ProjectedLabelClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    impact: Option<ProjectedImpactClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    listener: Option<ProjectedListenerClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excitation: Option<ProjectedExcitationClaim>,
}

impl ProjectedAxes {
    const fn complete_for_modal_field(&self) -> bool {
        self.material.is_some()
            && self.geometry.is_some()
            && self.support.is_some()
            && self.impact.is_some()
            && self.listener.is_some()
            && self.excitation.is_some()
    }
}

#[derive(Clone, Debug, Serialize)]
struct ProjectedLabelClaim {
    value_id: String,
    evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
struct ProjectedGeometryClaim {
    geometry_id: String,
    feature_artifact: VerifiedArtifact,
    evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
struct ProjectedImpactClaim {
    coordinate_profile: String,
    point_metres: [f64; 3],
    outward_normal: [f64; 3],
    evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
struct ProjectedListenerClaim {
    coordinate_profile: String,
    point_metres: [f64; 3],
    evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
struct ProjectedExcitationClaim {
    #[serde(skip_serializing_if = "Option::is_none")]
    impulse_newton_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    energy_joules: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    force_profile: Option<VerifiedArtifact>,
    evidence: VerifiedArtifact,
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
    material: usize,
    geometry: usize,
    support: usize,
    impact: usize,
    listener: usize,
    excitation: usize,
    complete_modal_field: usize,
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

    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    write_json(&output.join("fit-projection.json"), &built.fit_projection)?;
    write_json(
        &output.join("calibration-projection.json"),
        &built.calibration_projection,
    )?;
    write_json(
        &output.join("sealed-role-commitments.json"),
        &built.sealed_commitments,
    )?;
    let report_json = serde_json::to_vec_pretty(&built.report)
        .map_err(|error| format!("serialize neural data plane report: {error}"))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write neural data plane report: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?;
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn validate_manifest(manifest: &NeuralDataPlaneManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound neural data plane schema: {}",
            manifest.schema
        ));
    }
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
        validate_row(row, &lineage_ids)?;
        if previous_row.is_some_and(|previous| previous >= row.row_id.as_str()) {
            return Err("neural rows must be strictly sorted by row_id".to_owned());
        }
        previous_row = Some(&row.row_id);
    }
    validate_role_coverage(&manifest.rows)
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

fn validate_row(row: &NeuralRow, lineage_ids: &BTreeSet<&str>) -> Result<(), String> {
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
    validate_axis_claims(&row.axes)
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
        validate_unit_normal(claim.outward_normal)?;
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
        schema: PROJECTION_SCHEMA,
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
            schema: REPORT_SCHEMA,
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
        counts.material += usize::from(row.axes.material.is_some());
        counts.geometry += usize::from(row.axes.geometry.is_some());
        counts.support += usize::from(row.axes.support.is_some());
        counts.impact += usize::from(row.axes.impact.is_some());
        counts.listener += usize::from(row.axes.listener.is_some());
        counts.excitation += usize::from(row.axes.excitation.is_some());
        counts.complete_modal_field += usize::from(row.axes.complete_for_modal_field());
    }
    counts
}
