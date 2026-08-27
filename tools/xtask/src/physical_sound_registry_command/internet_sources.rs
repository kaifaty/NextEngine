use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use next_contracts::ids::ContentHash;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    ArtifactReport, FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file,
    require_empty_output, resolve_artifact, resolve_cli_path, resolve_output_path, set_once,
    sha256_hex, validate_file_ref, validate_label,
};

mod adapters;
mod fetch;
mod identified_corpus;

pub(super) use fetch::resolve_public_https_endpoint;

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-internet-sources.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-internet-sources.report.v1";
const DEFAULT_MAXIMUM_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
const HARD_MAXIMUM_DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024 * 1024;
const MAX_SOURCES: usize = 4_096;
const MAX_ARTIFACTS_PER_SOURCE: usize = 256;
const MAX_CAPABILITIES_PER_SOURCE: usize = 64;
const MAX_URL_BYTES: usize = 4_096;
const DOWNLOAD_BUFFER_BYTES: usize = 128 * 1024;

pub(super) struct Request {
    manifest: PathBuf,
    cache: PathBuf,
    output: PathBuf,
    fetch_missing: bool,
    maximum_download_bytes: u64,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

pub(super) fn run_identified_corpus_cli(
    root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    identified_corpus::run_cli(root, arguments)
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut manifest = None;
    let mut cache = None;
    let mut output = None;
    let mut fetch_missing = false;
    let mut maximum_download_bytes = DEFAULT_MAXIMUM_DOWNLOAD_BYTES;
    let mut maximum_download_bytes_set = false;

    while let Some(flag) = arguments.next() {
        if flag == "--fetch-missing" {
            if fetch_missing {
                return Err("duplicate argument: --fetch-missing".to_owned());
            }
            fetch_missing = true;
            continue;
        }
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--cache" => set_once(&mut cache, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            "--maximum-download-bytes" => {
                if maximum_download_bytes_set {
                    return Err("duplicate argument: --maximum-download-bytes".to_owned());
                }
                maximum_download_bytes = value
                    .parse::<u64>()
                    .map_err(|error| format!("parse --maximum-download-bytes: {error}"))?;
                maximum_download_bytes_set = true;
            }
            _ => return Err(format!("unexpected internet-sources argument: {flag}")),
        }
    }
    if maximum_download_bytes == 0 || maximum_download_bytes > HARD_MAXIMUM_DOWNLOAD_BYTES {
        return Err(format!(
            "--maximum-download-bytes must be 1..={HARD_MAXIMUM_DOWNLOAD_BYTES}"
        ));
    }

    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry internet-sources requires --manifest <external-json>"
                .to_owned()
        })?,
        cache: cache.ok_or_else(|| {
            "physical-sound-registry internet-sources requires --cache <external-directory>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry internet-sources requires --output <external-empty-directory>"
                .to_owned()
        })?,
        fetch_missing,
        maximum_download_bytes,
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct InternetSourceManifest {
    schema: String,
    registry_id: String,
    revision: String,
    sources: Vec<InternetSource>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct InternetSource {
    id: String,
    publisher_id: String,
    project_id: String,
    declared_revision: String,
    review_date: String,
    landing_page_url: String,
    #[serde(default)]
    terms_url: Option<String>,
    adapter_id: String,
    #[serde(default)]
    adapter_profile: Option<adapters::AdapterProfile>,
    license_expression: String,
    redistribution_policy: RedistributionPolicy,
    provenance_review: FileRef,
    artifacts: Vec<RemoteArtifact>,
    capability_evidence: Vec<CapabilityEvidence>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum RedistributionPolicy {
    ExternalResearchOnly,
    Prohibited,
    RedistributableWithNotice,
}

impl RedistributionPolicy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalResearchOnly => "external_research_only",
            Self::Prohibited => "prohibited",
            Self::RedistributableWithNotice => "redistributable_with_notice",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RemoteArtifact {
    id: String,
    role: ArtifactRole,
    url: String,
    #[serde(default)]
    redirect_policy: Option<FetchRedirectPolicy>,
    #[serde(default)]
    normalization_policy: Option<FetchNormalizationPolicy>,
    maximum_bytes: u64,
    #[serde(default)]
    expected_byte_count: Option<u64>,
    #[serde(default)]
    expected_sha256: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum FetchRedirectPolicy {
    FigshareKiltHubV1,
    OsfStorageV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum FetchNormalizationPolicy {
    FreesoundPackIdentityV1,
}

impl FetchNormalizationPolicy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::FreesoundPackIdentityV1 => "freesound_pack_identity_v1",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ArtifactRole {
    AudioArchive,
    AudioPayload,
    ForceArchive,
    GeometryArchive,
    LicenseText,
    Metadata,
    ProjectDescription,
    SourceArchive,
    TransferArchive,
}

impl ArtifactRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AudioArchive => "audio_archive",
            Self::AudioPayload => "audio_payload",
            Self::ForceArchive => "force_archive",
            Self::GeometryArchive => "geometry_archive",
            Self::LicenseText => "license_text",
            Self::Metadata => "metadata",
            Self::ProjectDescription => "project_description",
            Self::SourceArchive => "source_archive",
            Self::TransferArchive => "transfer_archive",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CapabilityEvidence {
    capability: EvidenceCapability,
    artifact_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceCapability {
    CalibratedForce,
    ForceCalibration,
    ForceDeconvolvedTransfer,
    ForceProfile,
    Geometry,
    ImpactPosition,
    ListenerPosition,
    MaterialComposition,
    MaterialIdentity,
    MicrophoneCalibration,
    ObjectIdentity,
    RawMicrophone,
    RealRecording,
    RepeatIdentity,
    SupportCondition,
    SyntheticLineage,
}

impl EvidenceCapability {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CalibratedForce => "calibrated_force",
            Self::ForceCalibration => "force_calibration",
            Self::ForceDeconvolvedTransfer => "force_deconvolved_transfer",
            Self::ForceProfile => "force_profile",
            Self::Geometry => "geometry",
            Self::ImpactPosition => "impact_position",
            Self::ListenerPosition => "listener_position",
            Self::MaterialComposition => "material_composition",
            Self::MaterialIdentity => "material_identity",
            Self::MicrophoneCalibration => "microphone_calibration",
            Self::ObjectIdentity => "object_identity",
            Self::RawMicrophone => "raw_microphone",
            Self::RealRecording => "real_recording",
            Self::RepeatIdentity => "repeat_identity",
            Self::SupportCondition => "support_condition",
            Self::SyntheticLineage => "synthetic_lineage",
        }
    }
}

#[derive(Clone, Copy)]
enum EvidenceTier {
    E1SynchronizedResponse,
    E2TransferResponse,
    E3IdentifiedRecording,
    E4SyntheticGenerated,
}

impl EvidenceTier {
    const fn as_str(self) -> &'static str {
        match self {
            Self::E1SynchronizedResponse => "E1SynchronizedResponse",
            Self::E2TransferResponse => "E2TransferResponse",
            Self::E3IdentifiedRecording => "E3IdentifiedRecording",
            Self::E4SyntheticGenerated => "E4SyntheticGenerated",
        }
    }

    const fn required_capabilities(self) -> &'static [EvidenceCapability] {
        match self {
            Self::E1SynchronizedResponse => &[
                EvidenceCapability::CalibratedForce,
                EvidenceCapability::ForceCalibration,
                EvidenceCapability::Geometry,
                EvidenceCapability::ImpactPosition,
                EvidenceCapability::ListenerPosition,
                EvidenceCapability::MaterialComposition,
                EvidenceCapability::MicrophoneCalibration,
                EvidenceCapability::ObjectIdentity,
                EvidenceCapability::RawMicrophone,
                EvidenceCapability::RealRecording,
                EvidenceCapability::RepeatIdentity,
                EvidenceCapability::SupportCondition,
            ],
            Self::E2TransferResponse => &[
                EvidenceCapability::ForceDeconvolvedTransfer,
                EvidenceCapability::Geometry,
                EvidenceCapability::ImpactPosition,
                EvidenceCapability::ListenerPosition,
                EvidenceCapability::ObjectIdentity,
                EvidenceCapability::RealRecording,
            ],
            Self::E3IdentifiedRecording => &[
                EvidenceCapability::MaterialIdentity,
                EvidenceCapability::ObjectIdentity,
                EvidenceCapability::RealRecording,
                EvidenceCapability::RepeatIdentity,
            ],
            Self::E4SyntheticGenerated => &[EvidenceCapability::SyntheticLineage],
        }
    }
}

const EVIDENCE_TIERS: [EvidenceTier; 4] = [
    EvidenceTier::E1SynchronizedResponse,
    EvidenceTier::E2TransferResponse,
    EvidenceTier::E3IdentifiedRecording,
    EvidenceTier::E4SyntheticGenerated,
];

#[derive(Serialize)]
struct InternetSourceReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    registry_id: String,
    revision: String,
    manifest_sha256: String,
    source_count: usize,
    ready_source_count: usize,
    sources: Vec<SourceReport>,
}

#[derive(Serialize)]
struct SourceReport {
    id: String,
    publisher_id: String,
    project_id: String,
    declared_revision: String,
    review_date: String,
    landing_page_url: String,
    terms_url: Option<String>,
    adapter_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    adapter_evidence: Option<adapters::AdapterEvidenceReport>,
    license_expression: String,
    redistribution_policy: &'static str,
    provenance_review: ArtifactReport,
    artifacts: Vec<RemoteArtifactReport>,
    capabilities: Vec<CapabilityReport>,
    supported_tiers: Vec<&'static str>,
    source_status: &'static str,
}

#[derive(Serialize)]
struct RemoteArtifactReport {
    id: String,
    role: &'static str,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    normalization_policy: Option<&'static str>,
    maximum_bytes: u64,
    expected_byte_count: Option<u64>,
    expected_sha256: Option<String>,
    cache_status: &'static str,
}

#[derive(Serialize)]
struct CapabilityReport {
    capability: &'static str,
    artifact_ids: Vec<String>,
    bytes_available: bool,
    adapter_validated: bool,
    available: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CacheStatus {
    CachedVerified,
    DownloadLimitExceeded,
    FetchFailed,
    FetchToolUnavailable,
    MissingFromCache,
    MissingIntegrityMetadata,
}

impl CacheStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CachedVerified => "CachedVerified",
            Self::DownloadLimitExceeded => "DownloadLimitExceeded",
            Self::FetchFailed => "FetchFailed",
            Self::FetchToolUnavailable => "FetchToolUnavailable",
            Self::MissingFromCache => "MissingFromCache",
            Self::MissingIntegrityMetadata => "MissingIntegrityMetadata",
        }
    }
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path = canonical_external_file(&root, &manifest_path, "internet source manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;
    let cache = resolve_cache_root(&root, &request.cache)?;
    if output.starts_with(&cache) || cache.starts_with(&output) {
        return Err("internet source cache and output must not overlap".to_owned());
    }

    let manifest_bytes = read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "internet source")?;
    let manifest: InternetSourceManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "internet source manifest has no parent directory".to_owned())?;
    let report = build_report(
        &root,
        manifest_directory,
        &cache,
        manifest,
        sha256_hex(&manifest_bytes),
        request,
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

fn validate_manifest(manifest: &InternetSourceManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound internet source schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.registry_id, "internet source registry id")?;
    validate_label(&manifest.revision, "internet source registry revision")?;
    if manifest.sources.is_empty() || manifest.sources.len() > MAX_SOURCES {
        return Err(format!("internet source count must be 1..={MAX_SOURCES}"));
    }
    let mut previous_source: Option<&str> = None;
    for source in &manifest.sources {
        validate_source(source)?;
        if previous_source.is_some_and(|previous| previous >= source.id.as_str()) {
            return Err("internet sources must be strictly sorted by id".to_owned());
        }
        previous_source = Some(&source.id);
    }
    Ok(())
}

fn validate_source(source: &InternetSource) -> Result<(), String> {
    for (value, role) in [
        (&source.id, "internet source id"),
        (&source.publisher_id, "publisher id"),
        (&source.project_id, "project id"),
        (&source.declared_revision, "declared source revision"),
        (&source.adapter_id, "source adapter id"),
    ] {
        validate_label(value, role)?;
    }
    validate_date(&source.review_date)?;
    validate_https_url(&source.landing_page_url, "landing page URL")?;
    if let Some(terms_url) = &source.terms_url {
        validate_https_url(terms_url, "terms URL")?;
    }
    validate_bounded_text(&source.license_expression, "license expression")?;
    validate_file_ref(&source.provenance_review, "source provenance review")?;
    adapters::validate_profile_declaration(source)?;
    if source.artifacts.is_empty() || source.artifacts.len() > MAX_ARTIFACTS_PER_SOURCE {
        return Err(format!(
            "source {} artifact count must be 1..={MAX_ARTIFACTS_PER_SOURCE}",
            source.id
        ));
    }
    let mut artifact_ids = BTreeSet::new();
    let mut previous_artifact: Option<&str> = None;
    for artifact in &source.artifacts {
        validate_label(&artifact.id, "remote artifact id")?;
        validate_https_url(&artifact.url, "remote artifact URL")?;
        if artifact.maximum_bytes == 0 || artifact.maximum_bytes > HARD_MAXIMUM_DOWNLOAD_BYTES {
            return Err(format!(
                "artifact {} maximum_bytes must be 1..={HARD_MAXIMUM_DOWNLOAD_BYTES}",
                artifact.id
            ));
        }
        if artifact.redirect_policy.is_some() && artifact.normalization_policy.is_some() {
            return Err(format!(
                "artifact {} cannot combine redirect and normalization policies",
                artifact.id
            ));
        }
        if artifact
            .expected_byte_count
            .is_some_and(|bytes| bytes == 0 || bytes > artifact.maximum_bytes)
        {
            return Err(format!(
                "artifact {} expected byte count exceeds its bound",
                artifact.id
            ));
        }
        if let Some(hash) = &artifact.expected_sha256 {
            validate_sha256(hash, "remote artifact sha256")?;
        }
        if previous_artifact.is_some_and(|previous| previous >= artifact.id.as_str()) {
            return Err(format!(
                "source {} artifacts must be strictly sorted by id",
                source.id
            ));
        }
        previous_artifact = Some(&artifact.id);
        artifact_ids.insert(artifact.id.as_str());
    }
    if source.capability_evidence.is_empty()
        || source.capability_evidence.len() > MAX_CAPABILITIES_PER_SOURCE
    {
        return Err(format!(
            "source {} capability count must be 1..={MAX_CAPABILITIES_PER_SOURCE}",
            source.id
        ));
    }
    let mut previous_capability: Option<&str> = None;
    for evidence in &source.capability_evidence {
        let capability = evidence.capability.as_str();
        if previous_capability.is_some_and(|previous| previous >= capability) {
            return Err(format!(
                "source {} capabilities must be strictly sorted",
                source.id
            ));
        }
        previous_capability = Some(capability);
        validate_sorted_artifact_ids(&evidence.artifact_ids, &artifact_ids, &source.id)?;
    }
    Ok(())
}

fn validate_sorted_artifact_ids(
    values: &[String],
    artifact_ids: &BTreeSet<&str>,
    source_id: &str,
) -> Result<(), String> {
    if values.is_empty() || values.len() > MAX_ARTIFACTS_PER_SOURCE {
        return Err(format!(
            "source {source_id} capability artifact count must be 1..={MAX_ARTIFACTS_PER_SOURCE}"
        ));
    }
    let mut previous: Option<&str> = None;
    for value in values {
        validate_label(value, "capability artifact id")?;
        if previous.is_some_and(|prior| prior >= value.as_str()) {
            return Err(format!(
                "source {source_id} capability artifact ids must be strictly sorted"
            ));
        }
        if !artifact_ids.contains(value.as_str()) {
            return Err(format!(
                "source {source_id} capability references missing artifact {value}"
            ));
        }
        previous = Some(value);
    }
    Ok(())
}

fn validate_date(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return Err("source review date must use YYYY-MM-DD".to_owned());
    }
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|error| format!("parse source review month: {error}"))?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|error| format!("parse source review day: {error}"))?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err("source review date has invalid month or day".to_owned());
    }
    Ok(())
}

fn validate_https_url(value: &str, role: &str) -> Result<(), String> {
    if value.len() > MAX_URL_BYTES
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
        || value.contains(['@', '\\', '#', '?'])
    {
        return Err(format!(
            "{role} must be a bounded credential-free canonical HTTPS URL"
        ));
    }
    let (host, path) = canonical_https_host_and_path(value, role)?;
    if host.is_empty()
        || path.is_empty()
        || host.contains(':')
        || !host.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
    {
        return Err(format!("{role} has a non-canonical host or empty path"));
    }
    Ok(())
}

fn canonical_https_host_and_path<'a>(
    value: &'a str,
    role: &str,
) -> Result<(&'a str, &'a str), String> {
    let remainder = value
        .strip_prefix("https://")
        .ok_or_else(|| format!("{role} must use https"))?;
    remainder
        .split_once('/')
        .ok_or_else(|| format!("{role} must include a host and path"))
}

fn validate_bounded_text(value: &str, role: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(format!("{role} must be 1..=128 printable ASCII bytes"));
    }
    Ok(())
}

fn validate_sha256(value: &str, role: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{role} must be 64 lowercase hex digits"));
    }
    Ok(())
}

fn build_report(
    root: &Path,
    manifest_directory: &Path,
    cache: &Path,
    manifest: InternetSourceManifest,
    manifest_sha256: String,
    request: &Request,
) -> Result<InternetSourceReport, String> {
    let mut source_reports = Vec::with_capacity(manifest.sources.len());
    let mut ready_source_count = 0_usize;
    for source in manifest.sources {
        let provenance_review = resolve_artifact(
            root,
            manifest_directory,
            &source.provenance_review,
            "source provenance review",
        )?;
        let mut artifact_statuses = BTreeMap::<String, CacheStatus>::new();
        let mut artifact_reports = Vec::with_capacity(source.artifacts.len());
        for artifact in &source.artifacts {
            let status = audit_or_fetch_artifact(cache, artifact, request)?;
            artifact_statuses.insert(artifact.id.clone(), status);
            artifact_reports.push(RemoteArtifactReport {
                id: artifact.id.clone(),
                role: artifact.role.as_str(),
                url: artifact.url.clone(),
                normalization_policy: artifact.normalization_policy.map(|value| value.as_str()),
                maximum_bytes: artifact.maximum_bytes,
                expected_byte_count: artifact.expected_byte_count,
                expected_sha256: artifact.expected_sha256.clone(),
                cache_status: status.as_str(),
            });
        }

        let adapter_audit = adapters::audit(cache, &source, &artifact_statuses)?;

        let mut available_capabilities = BTreeSet::new();
        let mut capability_reports = Vec::with_capacity(source.capability_evidence.len());
        for evidence in source.capability_evidence {
            let bytes_available = evidence.artifact_ids.iter().all(|artifact_id| {
                artifact_statuses.get(artifact_id) == Some(&CacheStatus::CachedVerified)
            });
            let adapter_validated = bytes_available && adapter_audit.validates(evidence.capability);
            let available = bytes_available && adapter_validated;
            if available {
                available_capabilities.insert(evidence.capability);
            }
            capability_reports.push(CapabilityReport {
                capability: evidence.capability.as_str(),
                artifact_ids: evidence.artifact_ids,
                bytes_available,
                adapter_validated,
                available,
            });
        }
        let supported_tiers = EVIDENCE_TIERS
            .into_iter()
            .filter(|tier| {
                tier.required_capabilities()
                    .iter()
                    .all(|capability| available_capabilities.contains(capability))
            })
            .map(EvidenceTier::as_str)
            .collect::<Vec<_>>();
        let source_status = if supported_tiers.is_empty() {
            "DiscoveryOnly"
        } else {
            ready_source_count += 1;
            "EvidenceReady"
        };
        source_reports.push(SourceReport {
            id: source.id,
            publisher_id: source.publisher_id,
            project_id: source.project_id,
            declared_revision: source.declared_revision,
            review_date: source.review_date,
            landing_page_url: source.landing_page_url,
            terms_url: source.terms_url,
            adapter_id: source.adapter_id,
            adapter_evidence: adapter_audit.evidence,
            license_expression: source.license_expression,
            redistribution_policy: source.redistribution_policy.as_str(),
            provenance_review,
            artifacts: artifact_reports,
            capabilities: capability_reports,
            supported_tiers,
            source_status,
        });
    }

    Ok(InternetSourceReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: if ready_source_count == source_reports.len() {
            "SourceSetComplete"
        } else {
            "SourceSetIncomplete"
        },
        claim: "INTERNET_SOURCE_CACHE_AND_CAPABILITY_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY",
        registry_id: manifest.registry_id,
        revision: manifest.revision,
        manifest_sha256,
        source_count: source_reports.len(),
        ready_source_count,
        sources: source_reports,
    })
}

fn audit_or_fetch_artifact(
    cache: &Path,
    artifact: &RemoteArtifact,
    request: &Request,
) -> Result<CacheStatus, String> {
    let (Some(expected_sha256), Some(expected_byte_count)) =
        (&artifact.expected_sha256, artifact.expected_byte_count)
    else {
        return Ok(CacheStatus::MissingIntegrityMetadata);
    };
    let target = cache_artifact_path(cache, expected_sha256)?;
    if target.exists() {
        verify_cache_artifact(&target, expected_sha256, expected_byte_count)?;
        return Ok(CacheStatus::CachedVerified);
    }
    if !request.fetch_missing {
        return Ok(CacheStatus::MissingFromCache);
    }
    let download_bound = artifact.maximum_bytes.min(request.maximum_download_bytes);
    if expected_byte_count > download_bound {
        return Ok(CacheStatus::DownloadLimitExceeded);
    }
    fetch::fetch_exact_artifact(
        cache,
        &target,
        &artifact.url,
        fetch::FetchPolicies {
            redirect: artifact.redirect_policy,
            normalization: artifact.normalization_policy,
        },
        expected_sha256,
        expected_byte_count,
        download_bound,
    )
}

fn resolve_cache_root(root: &Path, requested: &Path) -> Result<PathBuf, String> {
    let unresolved = resolve_cli_path(root, requested);
    if unresolved.exists() {
        let metadata = fs::symlink_metadata(&unresolved)
            .map_err(|error| format!("stat cache {}: {error}", unresolved.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "internet source cache must be a real directory: {}",
                unresolved.display()
            ));
        }
    } else {
        let parent = unresolved
            .parent()
            .ok_or_else(|| "internet source cache has no parent".to_owned())?;
        let parent = fs::canonicalize(parent)
            .map_err(|error| format!("canonicalize cache parent {}: {error}", parent.display()))?;
        if parent.starts_with(root) {
            return Err(format!(
                "internet source cache must stay outside the repository: {}",
                unresolved.display()
            ));
        }
        let name = unresolved
            .file_name()
            .ok_or_else(|| "internet source cache has no directory name".to_owned())?;
        fs::create_dir(parent.join(name))
            .map_err(|error| format!("create cache {}: {error}", unresolved.display()))?;
    }
    let cache = fs::canonicalize(&unresolved)
        .map_err(|error| format!("canonicalize cache {}: {error}", unresolved.display()))?;
    if cache.starts_with(root) {
        return Err(format!(
            "internet source cache must stay outside the repository: {}",
            cache.display()
        ));
    }
    ensure_real_directory(&cache.join("objects"))?;
    ensure_real_directory(&cache.join("staging"))?;
    Ok(cache)
}

fn ensure_real_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| format!("stat cache directory {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "cache path must be a real directory: {}",
                path.display()
            ));
        }
    } else {
        fs::create_dir(path)
            .map_err(|error| format!("create cache directory {}: {error}", path.display()))?;
    }
    Ok(())
}

fn cache_artifact_path(cache: &Path, sha256: &str) -> Result<PathBuf, String> {
    let prefix = cache.join("objects").join(&sha256[..2]);
    ensure_real_directory(&prefix)?;
    Ok(prefix.join(sha256))
}

fn verify_cache_artifact(
    path: &Path,
    expected_sha256: &str,
    expected_bytes: u64,
) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("stat cached artifact {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "cached artifact must be a real file: {}",
            path.display()
        ));
    }
    if metadata.len() != expected_bytes {
        return Err(format!(
            "cached artifact byte count mismatch: expected {expected_bytes}, got {}",
            metadata.len()
        ));
    }
    let (actual_sha256, actual_bytes) = hash_file_bounded(path, expected_bytes)?;
    if actual_bytes != expected_bytes || actual_sha256 != expected_sha256 {
        return Err(format!(
            "cached artifact hash mismatch: expected {expected_sha256}, got {actual_sha256}"
        ));
    }
    Ok(())
}

fn hash_file_bounded(path: &Path, maximum_bytes: u64) -> Result<(String, u64), String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("open cached artifact {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; DOWNLOAD_BUFFER_BYTES];
    let mut byte_count = 0_u64;
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("read cached artifact {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        byte_count = byte_count
            .checked_add(count as u64)
            .ok_or_else(|| "cached artifact byte count overflow".to_owned())?;
        if byte_count > maximum_bytes {
            return Err(format!(
                "cached artifact exceeds {maximum_bytes} bytes: {}",
                path.display()
            ));
        }
        hasher.update(&buffer[..count]);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    Ok((ContentHash::from_bytes(digest).to_hex(), byte_count))
}

#[cfg(test)]
mod tests;
