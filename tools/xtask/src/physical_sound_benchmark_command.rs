use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use serde::Serialize;

use crate::physical_sound_eval_command::audio_analysis::{analyze_benchmark_wav, parse_wav};

mod evaluator;
mod manifest;
mod mutations;
mod selective_risk;
#[cfg(test)]
mod tests;

use evaluator::{
    FeatureProfile, GroupedIdentityCounts, ResolvedEntry, TaskReport, evaluate_tasks,
    grouped_identity_counts,
};
use manifest::{
    BenchmarkManifest, DistanceMetric, ExternalFeatureSet, FeatureMatrix, FileRef, Partition,
    validate_feature_matrix, validate_manifest,
};
use selective_risk::{TemporalSelectiveRiskReport, evaluate_temporal_selective_risk};

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-benchmark.manifest.v1";
const FEATURE_MATRIX_SCHEMA: &str =
    "nextengine.experimental-physical-sound-corpus-feature-matrix.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-benchmark.report.v3";
const EVALUATOR_PROFILE: &str =
    "nextengine.experimental-physical-sound-corpus-benchmark.av-p0c-selective-risk.v2";
const CLASSICAL_FEATURE_SET_ID: &str = "classical-av-p0b-v1";
const TEMPORAL_FEATURE_SET_ID: &str = "temporal-dynamics-av-p0c-v1";
const MAX_ENTRIES: usize = 8_192;
const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
const MAX_WAV_BYTES: usize = 256 * 1024 * 1024;
const MAX_LICENSE_RECORD_BYTES: usize = 1024 * 1024;
const MAX_FEATURE_MATRIX_BYTES: usize = 512 * 1024 * 1024;
const MAX_EXTERNAL_FEATURE_SETS: usize = 8;
const MAX_EXTERNAL_FEATURE_DIMENSIONS: usize = 4_096;

pub(super) struct Request {
    manifest: PathBuf,
    output: PathBuf,
}

pub(super) fn mutate(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    mutations::run_cli(root, arguments)
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-benchmark requires --manifest <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-benchmark requires --output <external-empty-directory>".to_owned()
        })?,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
struct BenchmarkReport {
    schema: &'static str,
    decision: &'static str,
    benchmark_status: &'static str,
    claim: &'static str,
    benchmark_id: String,
    manifest_sha256: String,
    evaluator_profile: EvaluatorProfileReport,
    evaluator_profile_sha256: String,
    corpus: CorpusReport,
    split_audit: SplitAuditReport,
    input_failures: Vec<InputFailureReport>,
    optional_feature_components: Vec<OptionalFeatureComponentReport>,
    temporal_selective_risk: Option<TemporalSelectiveRiskReport>,
    tasks: Vec<TaskReport>,
    entries: Vec<EntryAudioReport>,
}

#[derive(Clone, Debug, Serialize)]
struct EvaluatorProfileReport {
    id: &'static str,
    classifier: &'static str,
    tie_break: &'static str,
    built_in_feature_definitions: Vec<BuiltInFeatureDefinitionReport>,
    maximum_entries: usize,
    maximum_wav_bytes: usize,
    feature_profiles: Vec<FeatureProfileReport>,
}

#[derive(Clone, Debug, Serialize)]
struct BuiltInFeatureDefinitionReport {
    id: &'static str,
    definition: &'static str,
    authority: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct FeatureProfileReport {
    id: String,
    provenance: &'static str,
    model_revision: Option<String>,
    model_sha256: Option<String>,
    matrix_sha256: Option<String>,
    dimensions: usize,
    distance: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct CorpusReport {
    source_count: usize,
    entry_count: usize,
    identities: GroupedIdentityCounts,
    sources: Vec<CorpusSourceReport>,
    license_boundary: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct CorpusSourceReport {
    id: String,
    revision: String,
    source_url: String,
    attribution: String,
    measurement_scope: &'static str,
    spdx_id: String,
    review_status: &'static str,
    redistribution: &'static str,
    review_record_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
struct SplitAuditReport {
    status: &'static str,
    partition_counts: Vec<PartitionCountReport>,
    object_and_family_groups_are_partition_disjoint: bool,
    development_gallery: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct PartitionCountReport {
    partition: &'static str,
    entry_count: usize,
    real_entry_count: usize,
}

#[derive(Clone, Debug, Serialize)]
struct InputFailureReport {
    entry_id: String,
    failure_tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct OptionalFeatureComponentReport {
    id: &'static str,
    status: &'static str,
    authority: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct EntryAudioReport {
    id: String,
    wav_sha256: String,
    sample_rate_hz: u32,
    channel_count: u16,
    duration_ms: f64,
    peak_dbfs: f64,
    rms_dbfs: f64,
    hard_failure_tags: Vec<String>,
    temporal_dynamics: crate::physical_sound_eval_command::audio_analysis::TemporalDynamicsReport,
}

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path = canonical_external_file(&root, &manifest_path, "manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let manifest_bytes = read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "manifest")?;
    let manifest: BenchmarkManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "manifest has no parent directory".to_owned())?;

    let sources = resolve_corpus_sources(&root, manifest_directory, &manifest)?;
    let mut entries = resolve_entries(&root, manifest_directory, &manifest)?;
    let mut feature_profile_reports = [CLASSICAL_FEATURE_SET_ID, TEMPORAL_FEATURE_SET_ID]
        .into_iter()
        .map(|id| {
            Ok(FeatureProfileReport {
                id: id.to_owned(),
                provenance: "built_in_deterministic_descriptor",
                model_revision: None,
                model_sha256: None,
                matrix_sha256: None,
                dimensions: entries
                    .first()
                    .and_then(|entry| entry.features.get(id))
                    .map(Vec::len)
                    .ok_or_else(|| format!("resolved corpus has no {id} features"))?,
                distance: DistanceMetric::Euclidean.as_str(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    resolve_external_features(
        &root,
        manifest_directory,
        &manifest,
        &mut entries,
        &mut feature_profile_reports,
    )?;
    let feature_profiles = feature_profile_reports
        .iter()
        .zip(
            [DistanceMetric::Euclidean, DistanceMetric::Euclidean]
                .into_iter()
                .chain(
                    manifest
                        .external_feature_sets
                        .iter()
                        .map(|set| set.distance),
                ),
        )
        .map(|(report, distance)| FeatureProfile {
            id: report.id.clone(),
            distance,
            dimensions: report.dimensions,
        })
        .collect::<Vec<_>>();
    validate_resolved_feature_dimensions(&entries, &feature_profiles)?;

    let input_failures = entries
        .iter()
        .filter(|entry| !entry.audio.hard_failure_tags.is_empty())
        .map(|entry| InputFailureReport {
            entry_id: entry.manifest.id.clone(),
            failure_tags: entry.audio.hard_failure_tags.clone(),
        })
        .collect::<Vec<_>>();
    let tasks = if input_failures.is_empty() {
        evaluate_tasks(&entries, &feature_profiles)
    } else {
        Vec::new()
    };
    let temporal_selective_risk = input_failures
        .is_empty()
        .then(|| evaluate_temporal_selective_risk(&entries));
    let optional_feature_components = optional_feature_components(&manifest);
    let profile = EvaluatorProfileReport {
        id: EVALUATOR_PROFILE,
        classifier: "deterministic one-nearest-neighbor baseline; benchmark only",
        tie_break: "minimum distance, then lexicographically smallest gallery entry id",
        built_in_feature_definitions: vec![
            BuiltInFeatureDefinitionReport {
                id: CLASSICAL_FEATURE_SET_ID,
                definition: "three gain-normalized log spectra plus bounded spectrum/onset/modal-count/decay scalars",
                authority: "frozen AV-P0B diagnostic baseline",
            },
            BuiltInFeatureDefinitionReport {
                id: TEMPORAL_FEATURE_SET_ID,
                definition: "bounded STFT spectral flux, adjacent/early-late cosine distance, centroid/flatness motion and active-bin turnover",
                authority: "AV-P0C diagnostic substrate; no selective-risk threshold",
            },
        ],
        maximum_entries: MAX_ENTRIES,
        maximum_wav_bytes: MAX_WAV_BYTES,
        feature_profiles: feature_profile_reports,
    };
    let profile_bytes = serde_json::to_vec(&profile).map_err(|error| error.to_string())?;
    let split_audit = split_audit_report(&entries);
    let corpus = CorpusReport {
        source_count: sources.len(),
        entry_count: entries.len(),
        identities: grouped_identity_counts(&entries),
        sources,
        license_boundary: "manifest declares reviewed or explicitly unreviewed research provenance; the tool verifies bytes/hash, forbids repository inputs, and makes no independent legal judgment",
    };
    let report = BenchmarkReport {
        schema: REPORT_SCHEMA,
        decision: "NoAcceptanceAuthority",
        benchmark_status: if input_failures.is_empty() {
            "Measured"
        } else {
            "InputRejected"
        },
        claim: "GROUPED_CORPUS_BENCHMARK_ONLY / NO_PERCEPTUAL_PASS_OR_RUNTIME_PROMOTION",
        benchmark_id: manifest.benchmark_id,
        manifest_sha256: sha256_hex(&manifest_bytes),
        evaluator_profile: profile,
        evaluator_profile_sha256: sha256_hex(&profile_bytes),
        corpus,
        split_audit,
        input_failures,
        optional_feature_components,
        temporal_selective_risk,
        tasks,
        entries: entries.into_iter().map(|entry| entry.audio).collect(),
    };
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

fn resolve_corpus_sources(
    root: &Path,
    manifest_directory: &Path,
    manifest: &BenchmarkManifest,
) -> Result<Vec<CorpusSourceReport>, String> {
    manifest
        .corpus_sources
        .iter()
        .map(|source| {
            let bytes = read_file_ref(
                root,
                manifest_directory,
                &source.license.review_record,
                MAX_LICENSE_RECORD_BYTES,
                "license review record",
            )?;
            if bytes.iter().all(u8::is_ascii_whitespace) {
                return Err(format!(
                    "license review record for source {} is empty",
                    source.id
                ));
            }
            Ok(CorpusSourceReport {
                id: source.id.clone(),
                revision: source.revision.clone(),
                source_url: source.source_url.clone(),
                attribution: source.attribution.clone(),
                measurement_scope: source.measurement_scope.as_str(),
                spdx_id: source.license.spdx_id.clone(),
                review_status: source.license.review_status.as_str(),
                redistribution: source.license.redistribution.as_str(),
                review_record_sha256: source.license.review_record.sha256.clone(),
            })
        })
        .collect()
}

fn resolve_entries(
    root: &Path,
    manifest_directory: &Path,
    manifest: &BenchmarkManifest,
) -> Result<Vec<ResolvedEntry>, String> {
    manifest
        .entries
        .iter()
        .cloned()
        .map(|entry| {
            let bytes = read_file_ref(
                root,
                manifest_directory,
                &entry.audio,
                MAX_WAV_BYTES,
                "corpus WAV",
            )?;
            let wav = parse_wav(&bytes)
                .map_err(|error| format!("parse corpus WAV {}: {error}", entry.audio.path))?;
            let analysis = analyze_benchmark_wav(&entry.audio.path, &entry.audio.sha256, wav)?;
            let mut features = BTreeMap::new();
            features.insert(CLASSICAL_FEATURE_SET_ID.to_owned(), analysis.feature_values);
            features.insert(
                TEMPORAL_FEATURE_SET_ID.to_owned(),
                analysis.temporal_feature_values,
            );
            let audio = EntryAudioReport {
                id: entry.id.clone(),
                wav_sha256: entry.audio.sha256.clone(),
                sample_rate_hz: analysis.sample_rate_hz,
                channel_count: analysis.channel_count,
                duration_ms: analysis.duration_ms,
                peak_dbfs: analysis.peak_dbfs,
                rms_dbfs: analysis.rms_dbfs,
                hard_failure_tags: analysis
                    .hard_failure_tags
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
                temporal_dynamics: analysis.temporal_dynamics,
            };
            Ok(ResolvedEntry {
                manifest: entry,
                audio,
                features,
            })
        })
        .collect()
}

fn resolve_external_features(
    root: &Path,
    manifest_directory: &Path,
    manifest: &BenchmarkManifest,
    entries: &mut [ResolvedEntry],
    profile_reports: &mut Vec<FeatureProfileReport>,
) -> Result<(), String> {
    for declaration in &manifest.external_feature_sets {
        let bytes = read_file_ref(
            root,
            manifest_directory,
            &declaration.matrix,
            MAX_FEATURE_MATRIX_BYTES,
            "external feature matrix",
        )?;
        let matrix: FeatureMatrix = serde_json::from_slice(&bytes).map_err(|error| {
            format!("parse external feature matrix {}: {error}", declaration.id)
        })?;
        validate_feature_matrix(declaration, &matrix, &manifest.entries)?;
        for (entry, feature) in entries.iter_mut().zip(matrix.entries) {
            entry
                .features
                .insert(declaration.id.clone(), feature.values);
        }
        profile_reports.push(external_feature_profile_report(declaration));
    }
    Ok(())
}

fn external_feature_profile_report(declaration: &ExternalFeatureSet) -> FeatureProfileReport {
    FeatureProfileReport {
        id: declaration.id.clone(),
        provenance: "external_hash_closed_feature_matrix",
        model_revision: Some(declaration.model_revision.clone()),
        model_sha256: Some(declaration.model_sha256.clone()),
        matrix_sha256: Some(declaration.matrix.sha256.clone()),
        dimensions: declaration.dimensions,
        distance: declaration.distance.as_str(),
    }
}

fn validate_resolved_feature_dimensions(
    entries: &[ResolvedEntry],
    profiles: &[FeatureProfile],
) -> Result<(), String> {
    for profile in profiles {
        for entry in entries {
            let values = entry.features.get(&profile.id).ok_or_else(|| {
                format!(
                    "entry {} is missing feature set {}",
                    entry.manifest.id, profile.id
                )
            })?;
            if values.len() != profile.dimensions {
                return Err(format!(
                    "entry {} feature set {} has {} dimensions, expected {}",
                    entry.manifest.id,
                    profile.id,
                    values.len(),
                    profile.dimensions
                ));
            }
        }
    }
    Ok(())
}

fn optional_feature_components(
    manifest: &BenchmarkManifest,
) -> Vec<OptionalFeatureComponentReport> {
    ["beats", "human-clap", "audiobox-aesthetics"]
        .into_iter()
        .map(|id| OptionalFeatureComponentReport {
            id,
            status: if manifest
                .external_feature_sets
                .iter()
                .any(|feature| feature.id == id)
            {
                "ProvidedAsFrozenMatrix"
            } else {
                "NotProvided"
            },
            authority: "diagnostic benchmark feature only",
        })
        .collect()
}

fn split_audit_report(entries: &[ResolvedEntry]) -> SplitAuditReport {
    SplitAuditReport {
        status: "Pass",
        partition_counts: Partition::ALL
            .into_iter()
            .map(|partition| PartitionCountReport {
                partition: partition.as_str(),
                entry_count: entries
                    .iter()
                    .filter(|entry| entry.manifest.partition == partition)
                    .count(),
                real_entry_count: entries
                    .iter()
                    .filter(|entry| {
                        entry.manifest.partition == partition
                            && matches!(&entry.manifest.origin, manifest::EntryOrigin::Real)
                    })
                    .count(),
            })
            .collect(),
        object_and_family_groups_are_partition_disjoint: true,
        development_gallery: "real entries only",
    }
}

fn resolve_cli_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn resolve_output_path(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let unresolved = resolve_cli_path(root, path);
    if unresolved.exists() {
        let resolved = fs::canonicalize(&unresolved)
            .map_err(|error| format!("canonicalize {}: {error}", unresolved.display()))?;
        if resolved.starts_with(root) {
            return Err(format!(
                "physical-sound-benchmark output must stay outside the repository: {}",
                resolved.display()
            ));
        }
        return Ok(resolved);
    }
    let parent = unresolved
        .parent()
        .ok_or_else(|| "output path has no parent directory".to_owned())?;
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("canonicalize output parent {}: {error}", parent.display()))?;
    let name = unresolved
        .file_name()
        .ok_or_else(|| "output path has no directory name".to_owned())?;
    let resolved = parent.join(name);
    if resolved.starts_with(root) {
        return Err(format!(
            "physical-sound-benchmark output must stay outside the repository: {}",
            resolved.display()
        ));
    }
    Ok(resolved)
}

fn canonical_external_file(root: &Path, path: &Path, role: &str) -> Result<PathBuf, String> {
    let resolved = fs::canonicalize(path)
        .map_err(|error| format!("canonicalize {role} {}: {error}", path.display()))?;
    if resolved.starts_with(root) {
        return Err(format!(
            "physical-sound-benchmark {role} must stay outside the repository: {}",
            resolved.display()
        ));
    }
    let metadata = fs::metadata(&resolved)
        .map_err(|error| format!("stat {role} {}: {error}", resolved.display()))?;
    if !metadata.is_file() {
        return Err(format!("{role} is not a file: {}", resolved.display()));
    }
    Ok(resolved)
}

fn require_empty_output(output: &Path) -> Result<(), String> {
    if !output.exists() {
        return Ok(());
    }
    if !output.is_dir() {
        return Err(format!("output is not a directory: {}", output.display()));
    }
    if fs::read_dir(output)
        .map_err(|error| format!("read {}: {error}", output.display()))?
        .next()
        .is_some()
    {
        return Err(format!(
            "physical-sound-benchmark output directory must be empty: {}",
            output.display()
        ));
    }
    Ok(())
}

fn read_file_ref(
    root: &Path,
    manifest_directory: &Path,
    file_ref: &FileRef,
    maximum_bytes: usize,
    role: &str,
) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(root, &manifest_directory.join(&file_ref.path), role)?;
    let bytes = read_bounded_file(&path, maximum_bytes, role)?;
    let actual_hash = sha256_hex(&bytes);
    if actual_hash != file_ref.sha256 {
        return Err(format!(
            "{role} hash mismatch for {}: expected {}, got {actual_hash}",
            path.display(),
            file_ref.sha256
        ));
    }
    Ok(bytes)
}

fn read_bounded_file(path: &Path, maximum_bytes: usize, role: &str) -> Result<Vec<u8>, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("stat {role} {}: {error}", path.display()))?;
    if metadata.len() > maximum_bytes as u64 {
        return Err(format!(
            "{role} exceeds {maximum_bytes} bytes: {}",
            path.display()
        ));
    }
    fs::read(path).map_err(|error| format!("read {role} {}: {error}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    ContentHash::from_bytes(sha256(bytes)).to_hex()
}
