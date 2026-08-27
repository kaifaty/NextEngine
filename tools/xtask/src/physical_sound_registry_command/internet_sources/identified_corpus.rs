use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::super::{
    FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file, require_empty_output,
    resolve_artifact, resolve_cli_path, resolve_output_path, set_once, sha256_hex,
    validate_file_ref, validate_label,
};
use super::adapters::AdapterEvidenceReport;
use super::{InternetSourceManifest, InternetSourceReport, SourceReport};

mod heller;

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-identified-corpus.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-identified-corpus.report.v1";
const PLAN_REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-plan.report.v1";
const MAX_ASSIGNMENTS: usize = 4_096;
const REQUIRED_CAPABILITIES: [&str; 4] = [
    "material_identity",
    "object_identity",
    "real_recording",
    "repeat_identity",
];
const UNAVAILABLE_CLAIM_AXES: [&str; 7] = [
    "absolute_force",
    "geometry",
    "impact_position",
    "listener_position",
    "material_composition",
    "support_condition",
    "transfer_response",
];

pub(super) struct Request {
    manifest: PathBuf,
    cache: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut manifest = None;
    let mut cache = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--cache" => set_once(&mut cache, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected identified-corpus argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry identified-corpus requires --manifest <external-json>"
                .to_owned()
        })?,
        cache: cache.ok_or_else(|| {
            "physical-sound-registry identified-corpus requires --cache <external-directory>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry identified-corpus requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IdentifiedCorpusManifest {
    schema: String,
    corpus_id: String,
    revision: String,
    target_material_label: String,
    internet_source_manifest: FileRef,
    corpus_plan_report: FileRef,
    assignments: Vec<SourceAssignment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceAssignment {
    source_id: String,
    partition: Partition,
    #[serde(default)]
    corpus_role: CorpusRole,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum Partition {
    Dev,
    Calibration,
    Holdout,
    Shadow,
}

impl Partition {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Calibration => "calibration",
            Self::Holdout => "holdout",
            Self::Shadow => "shadow",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CorpusRole {
    #[default]
    Unassigned,
    Target,
    RejectParent,
}

impl CorpusRole {
    const fn is_unassigned(&self) -> bool {
        matches!(self, Self::Unassigned)
    }
}

#[derive(Deserialize)]
struct CorpusPlanProbe {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    risk_power_analysis: RiskPowerProbe,
}

#[derive(Clone, Copy, Deserialize)]
struct RiskPowerProbe {
    required_reject_parent_groups: usize,
    minimum_in_domain_groups_for_coverage: usize,
}

#[derive(Serialize)]
struct IdentifiedCorpusReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    corpus_id: String,
    revision: String,
    target_material_label: String,
    manifest_sha256: String,
    internet_source_manifest_sha256: String,
    internet_source_audit_report_sha256: String,
    corpus_plan_report_sha256: String,
    source_count: usize,
    recording_count: usize,
    independent_group_counts: IndependentGroupCounts,
    partition_counts: Vec<PartitionCount>,
    material_coverage: Vec<MaterialCoverage>,
    coverage_against_plan: CoverageAgainstPlan,
    unavailable_claim_axes: &'static [&'static str],
    entries: Vec<IdentifiedRecording>,
}

#[derive(Serialize)]
struct IndependentGroupCounts {
    publishers: usize,
    project_revisions: usize,
    objects: usize,
    recordings: usize,
    material_labels: usize,
}

#[derive(Serialize)]
struct PartitionCount {
    partition: &'static str,
    sources: usize,
    objects: usize,
    recordings: usize,
}

#[derive(Serialize)]
struct MaterialCoverage {
    material_label: String,
    object_groups: usize,
    recordings: usize,
}

#[derive(Serialize)]
struct CoverageAgainstPlan {
    target_object_groups: usize,
    target_recordings: usize,
    minimum_in_domain_groups_for_coverage: usize,
    missing_target_object_groups: usize,
    target_in_domain_coverage_sufficient: bool,
    reject_parent_groups_in_this_e3_corpus: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    reject_parent_partition_counts: Vec<RejectParentPartitionCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    missing_reject_parent_groups: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reject_parent_coverage_sufficient: Option<bool>,
    required_reject_parent_groups: usize,
}

#[derive(Serialize)]
struct RejectParentPartitionCount {
    partition: &'static str,
    object_groups: usize,
    recordings: usize,
}

#[derive(Serialize)]
struct IdentifiedRecording {
    entry_id: String,
    partition: &'static str,
    #[serde(skip_serializing_if = "CorpusRole::is_unassigned")]
    corpus_role: CorpusRole,
    evidence_tier: &'static str,
    publisher_id: String,
    project_id: String,
    declared_revision: String,
    source_id: String,
    source_group_id: String,
    object_group_id: String,
    object_id: String,
    material_label: String,
    recording_id: String,
    sample_encoding: &'static str,
    sample_rate_hz: u32,
    channel_count: u16,
    bits_per_sample: u16,
    sample_frames: u64,
    audio_file_bytes: u64,
    audio_file_sha256: String,
    license_expression: String,
    redistribution_policy: &'static str,
}

#[derive(Default)]
struct MutablePartitionCount {
    sources: BTreeSet<String>,
    objects: BTreeSet<String>,
    recordings: usize,
}

#[derive(Default)]
struct MutableMaterialCoverage {
    objects: BTreeSet<String>,
    recordings: usize,
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path =
        canonical_external_file(&root, &manifest_path, "identified corpus manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;
    let cache = super::resolve_cache_root(&root, &request.cache)?;
    if output.starts_with(&cache) || cache.starts_with(&output) {
        return Err("identified corpus cache and output must not overlap".to_owned());
    }

    let manifest_bytes =
        read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "identified corpus")?;
    let manifest: IdentifiedCorpusManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "identified corpus manifest has no parent directory".to_owned())?;

    let source_manifest_artifact = resolve_artifact(
        &root,
        manifest_directory,
        &manifest.internet_source_manifest,
        "internet source manifest",
    )?;
    let source_manifest_path = canonical_external_file(
        &root,
        &manifest_directory.join(&manifest.internet_source_manifest.path),
        "internet source manifest",
    )?;
    let source_manifest_bytes =
        read_bounded_file(&source_manifest_path, MAX_MANIFEST_BYTES, "internet source")?;
    let source_manifest: InternetSourceManifest = serde_json::from_slice(&source_manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", source_manifest_path.display()))?;
    super::validate_manifest(&source_manifest)?;
    let source_manifest_directory = source_manifest_path
        .parent()
        .ok_or_else(|| "internet source manifest has no parent directory".to_owned())?;
    let source_request = super::Request {
        manifest: source_manifest_path.clone(),
        cache: cache.clone(),
        output: PathBuf::new(),
        fetch_missing: false,
        maximum_download_bytes: super::DEFAULT_MAXIMUM_DOWNLOAD_BYTES,
    };
    let source_audit = super::build_report(
        &root,
        source_manifest_directory,
        &cache,
        source_manifest,
        source_manifest_artifact.sha256.clone(),
        &source_request,
    )?;
    let source_audit_json =
        serde_json::to_vec_pretty(&source_audit).map_err(|error| error.to_string())?;

    let plan_artifact = resolve_artifact(
        &root,
        manifest_directory,
        &manifest.corpus_plan_report,
        "corpus plan report",
    )?;
    let plan_path = canonical_external_file(
        &root,
        &manifest_directory.join(&manifest.corpus_plan_report.path),
        "corpus plan report",
    )?;
    let plan_bytes = read_bounded_file(&plan_path, MAX_MANIFEST_BYTES, "corpus plan report")?;
    let plan: CorpusPlanProbe = serde_json::from_slice(&plan_bytes)
        .map_err(|error| format!("parse {}: {error}", plan_path.display()))?;
    validate_plan(&plan)?;

    let report = build_report(
        manifest,
        sha256_hex(&manifest_bytes),
        source_manifest_artifact.sha256,
        sha256_hex(&source_audit_json),
        source_audit,
        plan_artifact.sha256,
        plan.risk_power_analysis,
    )?;
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn validate_manifest(manifest: &IdentifiedCorpusManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound identified corpus schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.corpus_id, "identified corpus id")?;
    validate_label(&manifest.revision, "identified corpus revision")?;
    validate_material_label(&manifest.target_material_label)?;
    validate_file_ref(
        &manifest.internet_source_manifest,
        "internet source manifest",
    )?;
    validate_file_ref(&manifest.corpus_plan_report, "corpus plan report")?;
    if manifest.assignments.is_empty() || manifest.assignments.len() > MAX_ASSIGNMENTS {
        return Err(format!(
            "identified corpus assignment count must be 1..={MAX_ASSIGNMENTS}"
        ));
    }
    let mut previous: Option<&str> = None;
    let mut assigned_role_count = 0usize;
    let mut target_role_count = 0usize;
    let mut reject_parent_role_count = 0usize;
    for assignment in &manifest.assignments {
        validate_label(&assignment.source_id, "identified corpus source id")?;
        if previous.is_some_and(|value| value >= assignment.source_id.as_str()) {
            return Err("identified corpus assignments must be strictly sorted".to_owned());
        }
        previous = Some(&assignment.source_id);
        match assignment.corpus_role {
            CorpusRole::Unassigned => {}
            CorpusRole::Target => {
                assigned_role_count += 1;
                target_role_count += 1;
            }
            CorpusRole::RejectParent => {
                assigned_role_count += 1;
                reject_parent_role_count += 1;
            }
        }
    }
    if assigned_role_count != 0 && assigned_role_count != manifest.assignments.len() {
        return Err(
            "identified corpus explicit role mode cannot mix assigned and unassigned sources"
                .to_owned(),
        );
    }
    if assigned_role_count != 0 && (target_role_count == 0 || reject_parent_role_count == 0) {
        return Err(
            "identified corpus explicit role mode requires target and reject-parent sources"
                .to_owned(),
        );
    }
    Ok(())
}

fn validate_material_label(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'-'))
    {
        return Err("target material label must be 1..=64 ASCII label bytes".to_owned());
    }
    Ok(())
}

fn validate_plan(plan: &CorpusPlanProbe) -> Result<(), String> {
    if plan.schema != PLAN_REPORT_SCHEMA
        || plan.status != "Validated"
        || plan.decision != "PlanPowerSufficient"
        || plan.claim != "PREREGISTRATION_AND_POWER_PLAN_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
    {
        return Err("corpus plan report is not the supported validated power plan".to_owned());
    }
    if plan.risk_power_analysis.required_reject_parent_groups == 0
        || plan.risk_power_analysis.required_reject_parent_groups > 1_000_000
        || plan
            .risk_power_analysis
            .minimum_in_domain_groups_for_coverage
            == 0
        || plan
            .risk_power_analysis
            .minimum_in_domain_groups_for_coverage
            > 1_000_000
    {
        return Err("corpus plan report has invalid group requirements".to_owned());
    }
    Ok(())
}

fn build_report(
    manifest: IdentifiedCorpusManifest,
    manifest_sha256: String,
    source_manifest_sha256: String,
    source_audit_report_sha256: String,
    source_audit: InternetSourceReport,
    corpus_plan_report_sha256: String,
    requirements: RiskPowerProbe,
) -> Result<IdentifiedCorpusReport, String> {
    validate_source_audit(&source_audit)?;
    if manifest.assignments.len() != source_audit.sources.len() {
        return Err(
            "identified corpus must assign every audited E3 source exactly once".to_owned(),
        );
    }
    let explicit_role_mode = manifest
        .assignments
        .iter()
        .any(|assignment| !assignment.corpus_role.is_unassigned());
    let assignments = manifest
        .assignments
        .iter()
        .map(|assignment| (assignment.source_id.as_str(), assignment))
        .collect::<BTreeMap<_, _>>();
    let mut source_group_partitions = BTreeMap::<String, Partition>::new();
    let mut object_group_partitions = BTreeMap::<String, Partition>::new();
    let mut publishers = BTreeSet::new();
    let mut source_groups = BTreeSet::new();
    let mut object_groups = BTreeSet::new();
    let mut recording_groups = BTreeSet::new();
    let mut partition_counts = BTreeMap::<Partition, MutablePartitionCount>::new();
    let mut material_coverage = BTreeMap::<String, MutableMaterialCoverage>::new();
    let mut declared_target_objects = BTreeSet::new();
    let mut declared_target_recordings = 0usize;
    let mut reject_parent_objects = BTreeSet::new();
    let mut reject_parent_partition_counts = BTreeMap::<Partition, MutableMaterialCoverage>::new();
    let mut entries = Vec::new();

    for source in &source_audit.sources {
        let assignment = assignments
            .get(source.id.as_str())
            .ok_or_else(|| format!("identified corpus is missing source {}", source.id))?;
        let partition = assignment.partition;
        let (source_group_id, object_group_id, mut normalized) =
            normalize_source(source, partition)?;
        insert_partition_group(
            &mut source_group_partitions,
            &source_group_id,
            partition,
            "source group",
        )?;
        if object_group_partitions
            .insert(object_group_id.clone(), partition)
            .is_some()
        {
            return Err(format!(
                "identified corpus contains duplicate object group {object_group_id}"
            ));
        }
        publishers.insert(source.publisher_id.clone());
        source_groups.insert(source_group_id.clone());
        object_groups.insert(object_group_id.clone());
        let partition_count = partition_counts.entry(partition).or_default();
        partition_count.sources.insert(source_group_id);
        partition_count.objects.insert(object_group_id.clone());
        partition_count.recordings += normalized.len();
        let material = normalized
            .first()
            .ok_or_else(|| format!("source {} has no identified recordings", source.id))?
            .material_label
            .clone();
        match assignment.corpus_role {
            CorpusRole::Unassigned => {}
            CorpusRole::Target => {
                if material != manifest.target_material_label {
                    return Err(format!(
                        "target source {} has material {material}, expected {}",
                        source.id, manifest.target_material_label
                    ));
                }
                declared_target_objects.insert(object_group_id.clone());
                declared_target_recordings += normalized.len();
            }
            CorpusRole::RejectParent => {
                if material == manifest.target_material_label {
                    return Err(format!(
                        "reject-parent source {} has target material {material}",
                        source.id
                    ));
                }
                reject_parent_objects.insert(object_group_id.clone());
                let reject_partition = reject_parent_partition_counts.entry(partition).or_default();
                reject_partition.objects.insert(object_group_id.clone());
                reject_partition.recordings += normalized.len();
            }
        }
        let material_count = material_coverage.entry(material).or_default();
        material_count.objects.insert(object_group_id);
        material_count.recordings += normalized.len();
        for mut entry in normalized.drain(..) {
            entry.corpus_role = assignment.corpus_role;
            let recording_group_id = format!("{}--{}", entry.object_group_id, entry.recording_id);
            if !recording_groups.insert(recording_group_id.clone()) {
                return Err(format!(
                    "identified corpus contains duplicate recording group {recording_group_id}"
                ));
            }
            entries.push(entry);
        }
    }
    if assignments.keys().any(|source_id| {
        !source_audit
            .sources
            .iter()
            .any(|source| source.id == *source_id)
    }) {
        return Err("identified corpus assignment references an unaudited source".to_owned());
    }

    let target = material_coverage.get(&manifest.target_material_label);
    let (target_object_groups, target_recordings) = if explicit_role_mode {
        (declared_target_objects.len(), declared_target_recordings)
    } else {
        (
            target.map_or(0, |coverage| coverage.objects.len()),
            target.map_or(0, |coverage| coverage.recordings),
        )
    };
    let minimum_in_domain_groups = requirements.minimum_in_domain_groups_for_coverage;
    Ok(IdentifiedCorpusReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "DevelopmentCoverageMeasured",
        claim: "GROUP_AND_COVERAGE_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY",
        corpus_id: manifest.corpus_id,
        revision: manifest.revision,
        target_material_label: manifest.target_material_label,
        manifest_sha256,
        internet_source_manifest_sha256: source_manifest_sha256,
        internet_source_audit_report_sha256: source_audit_report_sha256,
        corpus_plan_report_sha256,
        source_count: source_audit.source_count,
        recording_count: entries.len(),
        independent_group_counts: IndependentGroupCounts {
            publishers: publishers.len(),
            project_revisions: source_groups.len(),
            objects: object_groups.len(),
            recordings: recording_groups.len(),
            material_labels: material_coverage.len(),
        },
        partition_counts: [
            Partition::Dev,
            Partition::Calibration,
            Partition::Holdout,
            Partition::Shadow,
        ]
        .into_iter()
        .map(|partition| {
            let counts = partition_counts.get(&partition);
            PartitionCount {
                partition: partition.as_str(),
                sources: counts.map_or(0, |value| value.sources.len()),
                objects: counts.map_or(0, |value| value.objects.len()),
                recordings: counts.map_or(0, |value| value.recordings),
            }
        })
        .collect(),
        material_coverage: material_coverage
            .into_iter()
            .map(|(material_label, coverage)| MaterialCoverage {
                material_label,
                object_groups: coverage.objects.len(),
                recordings: coverage.recordings,
            })
            .collect(),
        coverage_against_plan: CoverageAgainstPlan {
            target_object_groups,
            target_recordings,
            minimum_in_domain_groups_for_coverage: minimum_in_domain_groups,
            missing_target_object_groups: minimum_in_domain_groups
                .saturating_sub(target_object_groups),
            target_in_domain_coverage_sufficient: target_object_groups >= minimum_in_domain_groups,
            reject_parent_groups_in_this_e3_corpus: reject_parent_objects.len(),
            reject_parent_partition_counts: if explicit_role_mode {
                [
                    Partition::Dev,
                    Partition::Calibration,
                    Partition::Holdout,
                    Partition::Shadow,
                ]
                .into_iter()
                .map(|partition| {
                    let counts = reject_parent_partition_counts.get(&partition);
                    RejectParentPartitionCount {
                        partition: partition.as_str(),
                        object_groups: counts.map_or(0, |value| value.objects.len()),
                        recordings: counts.map_or(0, |value| value.recordings),
                    }
                })
                .collect()
            } else {
                Vec::new()
            },
            missing_reject_parent_groups: explicit_role_mode.then(|| {
                requirements
                    .required_reject_parent_groups
                    .saturating_sub(reject_parent_objects.len())
            }),
            reject_parent_coverage_sufficient: explicit_role_mode.then_some(
                reject_parent_objects.len() >= requirements.required_reject_parent_groups,
            ),
            required_reject_parent_groups: requirements.required_reject_parent_groups,
        },
        unavailable_claim_axes: &UNAVAILABLE_CLAIM_AXES,
        entries,
    })
}

fn validate_source_audit(report: &InternetSourceReport) -> Result<(), String> {
    if report.schema != super::REPORT_SCHEMA
        || report.status != "Validated"
        || report.decision != "SourceSetComplete"
        || report.claim
            != "INTERNET_SOURCE_CACHE_AND_CAPABILITY_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
        || report.source_count == 0
        || report.source_count != report.sources.len()
        || report.ready_source_count != report.source_count
    {
        return Err("identified corpus requires one complete internet-source audit".to_owned());
    }
    Ok(())
}

fn normalize_source(
    source: &SourceReport,
    partition: Partition,
) -> Result<(String, String, Vec<IdentifiedRecording>), String> {
    if source.source_status != "EvidenceReady"
        || source.supported_tiers != ["E3IdentifiedRecording"]
        || source.capabilities.len() != REQUIRED_CAPABILITIES.len()
    {
        return Err(format!(
            "source {} is not an exact adapter-backed E3 source",
            source.id
        ));
    }
    for (capability, required) in source.capabilities.iter().zip(REQUIRED_CAPABILITIES) {
        if capability.capability != required
            || !capability.bytes_available
            || !capability.adapter_validated
            || !capability.available
        {
            return Err(format!(
                "source {} does not close required E3 capability {required}",
                source.id
            ));
        }
    }
    if source.adapter_id == "heller-impact-identified-recording-v1" {
        return heller::normalize_source(source, partition);
    }
    let (object_id, material_label, recordings) = match &source.adapter_evidence {
        Some(AdapterEvidenceReport::AvMsfIdentifiedRecordingV1 {
            object_id,
            material_label,
            recordings,
        }) if source.adapter_id == "av-msf-identified-recording-v1" => (
            object_id,
            material_label,
            recordings.iter().collect::<Vec<_>>(),
        ),
        Some(AdapterEvidenceReport::YcbImpactIdentifiedRecordingV1 {
            object_id,
            primary_material_label,
            recordings,
            ..
        }) if source.adapter_id == "ycb-impact-identified-recording-v1" => (
            object_id,
            primary_material_label,
            recordings
                .iter()
                .map(|recording| &recording.audio)
                .collect::<Vec<_>>(),
        ),
        Some(AdapterEvidenceReport::FreesoundGlassBowlIdentifiedRecordingV1 {
            object_id,
            material_label,
            recordings,
            ..
        }) if source.adapter_id == "freesound-glass-bowl-identified-recording-v1" => (
            object_id,
            material_label,
            recordings
                .iter()
                .map(|recording| &recording.audio)
                .collect::<Vec<_>>(),
        ),
        Some(AdapterEvidenceReport::FreesoundWineGlassIdentifiedRecordingV1 {
            object_id,
            material_label,
            recordings,
            ..
        }) if source.adapter_id == "freesound-wine-glass-identified-recording-v1" => (
            object_id,
            material_label,
            recordings
                .iter()
                .map(|recording| &recording.audio)
                .collect::<Vec<_>>(),
        ),
        _ => {
            return Err(format!(
                "source {} has no matching typed E3 adapter evidence",
                source.id
            ));
        }
    };
    if recordings.is_empty() {
        return Err(format!("source {} has no identified recordings", source.id));
    }
    let source_group_id = format!(
        "{}--{}--{}",
        source.publisher_id, source.project_id, source.declared_revision
    );
    let object_group_id = format!("{source_group_id}--object-{object_id}");
    let mut normalized = Vec::with_capacity(recordings.len());
    for recording in recordings {
        let artifact_id = format!("impact-{}", recording.recording_id);
        let artifact = source
            .artifacts
            .iter()
            .find(|artifact| artifact.id == artifact_id)
            .ok_or_else(|| {
                format!(
                    "source {} is missing recording artifact {artifact_id}",
                    source.id
                )
            })?;
        let (Some(audio_file_bytes), Some(audio_file_sha256)) = (
            artifact.expected_byte_count,
            artifact.expected_sha256.as_ref(),
        ) else {
            return Err(format!("artifact {artifact_id} is not hash closed"));
        };
        if artifact.role != "audio_payload" || artifact.cache_status != "CachedVerified" {
            return Err(format!(
                "artifact {artifact_id} is not cached verified audio"
            ));
        }
        normalized.push(IdentifiedRecording {
            entry_id: format!("{}--impact-{}", source.id, recording.recording_id),
            partition: partition.as_str(),
            corpus_role: CorpusRole::Unassigned,
            evidence_tier: "E3IdentifiedRecording",
            publisher_id: source.publisher_id.clone(),
            project_id: source.project_id.clone(),
            declared_revision: source.declared_revision.clone(),
            source_id: source.id.clone(),
            source_group_id: source_group_id.clone(),
            object_group_id: object_group_id.clone(),
            object_id: object_id.clone(),
            material_label: material_label.clone(),
            recording_id: recording.recording_id.clone(),
            sample_encoding: recording.sample_encoding,
            sample_rate_hz: recording.sample_rate_hz,
            channel_count: recording.channel_count,
            bits_per_sample: recording.bits_per_sample,
            sample_frames: recording.sample_frames,
            audio_file_bytes,
            audio_file_sha256: audio_file_sha256.clone(),
            license_expression: source.license_expression.clone(),
            redistribution_policy: source.redistribution_policy,
        });
    }
    Ok((source_group_id, object_group_id, normalized))
}

fn insert_partition_group(
    assignments: &mut BTreeMap<String, Partition>,
    group_id: &str,
    partition: Partition,
    role: &str,
) -> Result<(), String> {
    if let Some(previous) = assignments.insert(group_id.to_owned(), partition)
        && previous != partition
    {
        return Err(format!(
            "partition leakage for {role} {group_id}: {} versus {}",
            previous.as_str(),
            partition.as_str()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ycb_tests;
