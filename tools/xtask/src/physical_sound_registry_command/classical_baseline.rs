use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};
use next_presentation::physical_sound_lab::{
    render_selected_glass_q30_impact, selected_glass_q30_profile_snapshot,
};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

use crate::physical_sound_eval_command::audio_analysis::{
    BenchmarkAudioAnalysis, WavAudio, analyze_benchmark_wav, parse_wav,
};

use super::{
    FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file, require_empty_output,
    resolve_artifact, resolve_cli_path, resolve_output_path, set_once, sha256_hex,
    validate_file_ref, validate_label,
};

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-classical-baseline.manifest.v1";
const PROJECTION_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.projection.v1";
const PROFILE_SCHEMA: &str = "nextengine.experimental-physical-sound-classical-q30-profile.v1";
const RECORDS_SCHEMA: &str = "nextengine.experimental-physical-sound-classical-baseline.records.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-classical-baseline.report.v1";
const CLAIM: &str =
    "DETERMINISTIC_CLASSICAL_BENCHMARK_ONLY / NO_QUALITY_DOMAIN_OR_ADMISSION_AUTHORITY";
const TASK_SCOPE: &str = "exact_object_few_shot_impact_listener_field";
const MAX_BINDINGS: usize = 65_536;
const FROZEN_SELECTED_Q30_WAV_SHA256: &str =
    "c912806ccbc24e9b2af04b80f9a030f1ec66a6f16321bd3addb3cabc1db9c823";
const FROZEN_PROFILE_SHA256: &str =
    "19b051fe85be66847e0b8178c2f602cfec7d0f39bf6b6c2f893e0326f5fef181";
const DCT_PREREGISTRATION_PATH: &str =
    "lab/scripts/physical_sound_realimpact_colored_residual_preregistration.py";
const DCT_PREREGISTRATION_SHA256: &str =
    "fe8a12f516ee8f85dc515c23cf2b7f67cc6bcc4d12a2b7f20c7f69d0626cebf5";
const DCT_PIPELINE_PATH: &str =
    "lab/scripts/physical_sound_realimpact_colored_residual_pipeline.py";
const DCT_PIPELINE_SHA256: &str =
    "34c16185790fac7dbccd00fcc00fd332a0917f6d071cbd70100aaeb4f7081e51";

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
            _ => return Err(format!("unexpected classical-baseline argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry classical-baseline requires --manifest <external-json>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry classical-baseline requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ClassicalBaselineManifest {
    schema: String,
    benchmark_id: String,
    revision: String,
    projection: FileRef,
    bindings: Vec<RowBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RowBinding {
    row_id: String,
    profile: BaselineProfile,
    energy_q16: u32,
    reference_audio: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum BaselineProfile {
    SelectedThinContainerQ30V1,
    FrozenColoredDctResidualV1,
}

impl BaselineProfile {
    const fn as_str(self) -> &'static str {
        match self {
            Self::SelectedThinContainerQ30V1 => "selected_thin_container_q30_v1",
            Self::FrozenColoredDctResidualV1 => "frozen_colored_dct_residual_v1",
        }
    }
}

#[derive(Deserialize)]
struct DataProjection {
    schema: String,
    projection_id: String,
    revision: String,
    task_scope: String,
    manifest_sha256: String,
    role_scope: Vec<String>,
    rows: Vec<ProjectionRow>,
}

#[derive(Deserialize, Serialize)]
struct ProjectionRow {
    row_id: String,
    split_role: String,
    sample_role: String,
    corpus_role: String,
    audio: ProjectedArtifact,
    axes: ProjectionAxes,
}

#[derive(Deserialize, Serialize)]
struct ProjectedArtifact {
    sha256: String,
    byte_count: usize,
}

#[derive(Default, Deserialize, Serialize)]
struct ProjectionAxes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    material: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    geometry: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    support: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    impact: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    listener: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excitation: Option<serde_json::Value>,
}

impl ProjectionAxes {
    const fn complete_for_modal_field(&self) -> bool {
        self.material.is_some()
            && self.geometry.is_some()
            && self.support.is_some()
            && self.impact.is_some()
            && self.listener.is_some()
            && self.excitation.is_some()
    }
}

#[derive(Serialize)]
struct ClassicalProfileRecord {
    schema: &'static str,
    revision: &'static str,
    source_profile_sha256: &'static str,
    numeric_profile: &'static str,
    sample_rate_hz: u32,
    channel_count: u32,
    duration_frames: u32,
    canonical_mode_order: &'static str,
    modes: Vec<ModeRecord>,
    residual: ResidualRecord,
    scaling: ScalingRecord,
    colored_dct_boundary: ColoredDctBoundary,
}

#[derive(Serialize)]
struct ModeRecord {
    canonical_index: usize,
    coefficient_a_q30: i64,
    coefficient_b_q30: i64,
    initial_sample_q30: i64,
}

#[derive(Serialize)]
struct ResidualRecord {
    kind: &'static str,
    sample_count: usize,
    samples_q30: Vec<i64>,
}

#[derive(Serialize)]
struct ScalingRecord {
    raw_peak_q30: String,
    center_channel_peak: String,
    energy_profile: &'static str,
    stereo_projection: &'static str,
}

#[derive(Clone, Serialize)]
struct CoverageStatus {
    decision: &'static str,
    reason: &'static str,
}

#[derive(Serialize)]
struct ColoredDctBoundary {
    candidate_id: &'static str,
    preregistration_source: FrozenSource,
    boundary_pipeline_source: FrozenSource,
    maximum_coefficients_per_band: usize,
    decision: &'static str,
    reason: &'static str,
}

#[derive(Serialize)]
struct FrozenSource {
    path: &'static str,
    sha256: &'static str,
}

#[derive(Serialize)]
struct BaselineRecords {
    schema: &'static str,
    claim: &'static str,
    benchmark_id: String,
    revision: String,
    manifest_sha256: String,
    projection_sha256: String,
    projection_id: String,
    projection_revision: String,
    projection_manifest_sha256: String,
    profile_sha256: String,
    rows: Vec<RowRecord>,
}

#[derive(Serialize)]
struct RowRecord {
    row_id: String,
    split_role: String,
    sample_role: String,
    corpus_role: String,
    requested_profile: &'static str,
    decision: &'static str,
    coverage: CoverageStatus,
    reference_audio: ArtifactSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction: Option<PredictionRecord>,
}

#[derive(Serialize)]
struct ArtifactSummary {
    sha256: String,
    byte_count: usize,
}

#[derive(Serialize)]
struct PredictionRecord {
    profile_sha256: String,
    energy_q16: u32,
    wav_file: String,
    wav_sha256: String,
    pcm_s16le_sha256: String,
    stereo_frame_count: usize,
    features: AcousticFeatures,
    comparison: ComparisonReport,
}

#[derive(Serialize)]
struct AcousticFeatures {
    sample_rate_hz: u32,
    channel_count: u16,
    frame_count: usize,
    duration_microseconds: u64,
    peak_dbfs: f64,
    rms_dbfs: f64,
    hard_failure_tags: Vec<&'static str>,
    validator_feature_values: Vec<f64>,
    temporal_feature_values: Vec<f64>,
    amplitude_envelope_feature_values: Vec<f64>,
}

#[derive(Serialize)]
struct ComparisonReport {
    reference_wav_sha256: String,
    prediction_wav_sha256: String,
    exact_wav_match: bool,
    comparable_mono_pcm: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum_absolute_error: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rms_error: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation: Option<f64>,
}

#[derive(Serialize)]
struct BaselineReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    benchmark_id: String,
    revision: String,
    manifest_sha256: String,
    projection_sha256: String,
    profile_sha256: String,
    row_count: usize,
    baseline_available_rows: usize,
    fallback_out_of_domain_rows: usize,
    exact_wav_match_rows: usize,
    emitted_files: Vec<String>,
    model_training_authorized: bool,
    quality_or_admission_authorized: bool,
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.manifest),
        "classical baseline manifest",
    )?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "classical baseline manifest",
    )?;
    let manifest: ClassicalBaselineManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "classical baseline manifest has no parent directory".to_owned())?;
    let projection_summary = resolve_artifact(
        &root,
        manifest_directory,
        &manifest.projection,
        "neural fit projection",
    )?;
    let projection_path = canonical_external_file(
        &root,
        &manifest_directory.join(&manifest.projection.path),
        "neural fit projection",
    )?;
    let projection_bytes = read_bounded_file(
        &projection_path,
        MAX_MANIFEST_BYTES,
        "neural fit projection",
    )?;
    let projection: DataProjection = serde_json::from_slice(&projection_bytes)
        .map_err(|error| format!("parse {}: {error}", projection_path.display()))?;
    validate_projection(&projection, &manifest.bindings)?;
    validate_frozen_sources(&root)?;

    let profile = build_profile();
    let profile_bytes = serde_json::to_vec_pretty(&profile)
        .map_err(|error| format!("serialize classical profile: {error}"))?;
    let profile_sha256 = sha256_hex(&profile_bytes);
    if profile_sha256 != FROZEN_PROFILE_SHA256 {
        return Err(format!(
            "frozen classical profile hash changed: expected {FROZEN_PROFILE_SHA256}, got {profile_sha256}"
        ));
    }
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let bindings = manifest
        .bindings
        .iter()
        .map(|binding| (binding.row_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    let audio_profile =
        AudioMixProfileV1::stereo_baseline_v1().map_err(|error| error.to_string())?;
    let mut row_records = Vec::with_capacity(projection.rows.len());
    let mut emitted_files = vec![
        "classical-profile.json".to_owned(),
        "baseline-records.json".to_owned(),
        "report.json".to_owned(),
    ];
    let mut generated_wavs = Vec::new();
    for row in &projection.rows {
        let binding = bindings
            .get(row.row_id.as_str())
            .ok_or_else(|| format!("missing binding for projection row {}", row.row_id))?;
        let reference_summary = resolve_artifact(
            &root,
            manifest_directory,
            &binding.reference_audio,
            "classical baseline reference audio",
        )?;
        if reference_summary.sha256 != row.audio.sha256
            || reference_summary.byte_count != row.audio.byte_count
        {
            return Err(format!(
                "reference audio for {} does not match projected audio identity",
                row.row_id
            ));
        }
        let reference = ArtifactSummary {
            sha256: reference_summary.sha256.clone(),
            byte_count: reference_summary.byte_count,
        };
        let unsupported = fallback_reason(row, binding.profile);
        if let Some(coverage) = unsupported {
            row_records.push(RowRecord {
                row_id: row.row_id.clone(),
                split_role: row.split_role.clone(),
                sample_role: row.sample_role.clone(),
                corpus_role: row.corpus_role.clone(),
                requested_profile: binding.profile.as_str(),
                decision: "FallbackOutOfDomain",
                coverage,
                reference_audio: reference,
                prediction: None,
            });
            continue;
        }

        let reference_path = canonical_external_file(
            &root,
            &manifest_directory.join(&binding.reference_audio.path),
            "classical baseline reference audio",
        )?;
        let reference_bytes = read_bounded_file(
            &reference_path,
            super::MAX_REFERENCED_FILE_BYTES,
            "classical baseline reference audio",
        )?;
        let reference_wav = parse_wav(&reference_bytes)
            .map_err(|error| format!("parse reference WAV for {}: {error}", row.row_id))?;
        let pcm = render_selected_glass_q30_impact(binding.energy_q16);
        let wav = encode_canonical_wav(&audio_profile, &pcm);
        let wav_sha256 = sha256_hex(&wav);
        if binding.energy_q16 == 65_536 && wav_sha256 != FROZEN_SELECTED_Q30_WAV_SHA256 {
            return Err(format!(
                "frozen selected Q30 WAV hash changed: expected {FROZEN_SELECTED_Q30_WAV_SHA256}, got {wav_sha256}"
            ));
        }
        let wav_file = format!("{}.classical.wav", row.row_id);
        emitted_files.push(wav_file.clone());
        let prediction_wav = parse_wav(&wav)
            .map_err(|error| format!("parse rendered WAV for {}: {error}", row.row_id))?;
        let comparison = compare_audio(
            &reference_wav,
            &prediction_wav,
            &reference_summary.sha256,
            &wav_sha256,
            reference_bytes == wav,
        );
        let features = acoustic_features(&wav_file, &wav_sha256, prediction_wav)?;
        let pcm_bytes = pcm
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect::<Vec<_>>();
        generated_wavs.push((wav_file.clone(), wav));
        row_records.push(RowRecord {
            row_id: row.row_id.clone(),
            split_role: row.split_role.clone(),
            sample_role: row.sample_role.clone(),
            corpus_role: row.corpus_role.clone(),
            requested_profile: binding.profile.as_str(),
            decision: "BaselineAvailable",
            coverage: CoverageStatus {
                decision: "ExactRowOnly",
                reason: "explicit_hash_closed_row_binding",
            },
            reference_audio: reference,
            prediction: Some(PredictionRecord {
                profile_sha256: profile_sha256.clone(),
                energy_q16: binding.energy_q16,
                wav_file,
                wav_sha256,
                pcm_s16le_sha256: sha256_hex(&pcm_bytes),
                stereo_frame_count: pcm.len() / 2,
                features,
                comparison,
            }),
        });
    }
    if !output.exists() {
        fs::create_dir_all(&output)
            .map_err(|error| format!("create {}: {error}", output.display()))?;
    }
    for (file, wav) in generated_wavs {
        fs::write(output.join(&file), wav).map_err(|error| format!("write {file}: {error}"))?;
    }
    fs::write(output.join("classical-profile.json"), &profile_bytes)
        .map_err(|error| format!("write classical profile: {error}"))?;
    let records = BaselineRecords {
        schema: RECORDS_SCHEMA,
        claim: CLAIM,
        benchmark_id: manifest.benchmark_id.clone(),
        revision: manifest.revision.clone(),
        manifest_sha256: manifest_sha256.clone(),
        projection_sha256: projection_summary.sha256.clone(),
        projection_id: projection.projection_id,
        projection_revision: projection.revision,
        projection_manifest_sha256: projection.manifest_sha256,
        profile_sha256: profile_sha256.clone(),
        rows: row_records,
    };
    let records_bytes = serde_json::to_vec_pretty(&records)
        .map_err(|error| format!("serialize baseline records: {error}"))?;
    fs::write(output.join("baseline-records.json"), records_bytes)
        .map_err(|error| format!("write baseline records: {error}"))?;

    let available = records
        .rows
        .iter()
        .filter(|row| row.prediction.is_some())
        .count();
    let exact = records
        .rows
        .iter()
        .filter_map(|row| row.prediction.as_ref())
        .filter(|prediction| prediction.comparison.exact_wav_match)
        .count();
    emitted_files.sort();
    let report = BaselineReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: match available {
            0 => "ClassicalBaselineUnavailable",
            count if count == records.rows.len() => "ClassicalBaselineAvailable",
            _ => "PartialClassicalBaselineCoverage",
        },
        claim: CLAIM,
        benchmark_id: manifest.benchmark_id,
        revision: manifest.revision,
        manifest_sha256,
        projection_sha256: projection_summary.sha256,
        profile_sha256,
        row_count: records.rows.len(),
        baseline_available_rows: available,
        fallback_out_of_domain_rows: records.rows.len() - available,
        exact_wav_match_rows: exact,
        emitted_files,
        model_training_authorized: false,
        quality_or_admission_authorized: false,
    };
    let report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("serialize classical baseline report: {error}"))?;
    fs::write(output.join("report.json"), &report_bytes)
        .map_err(|error| format!("write classical baseline report: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_bytes).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn validate_manifest(manifest: &ClassicalBaselineManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported classical baseline schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.benchmark_id, "classical benchmark id")?;
    validate_label(&manifest.revision, "classical benchmark revision")?;
    validate_file_ref(&manifest.projection, "neural fit projection")?;
    if manifest.bindings.is_empty() || manifest.bindings.len() > MAX_BINDINGS {
        return Err(format!(
            "classical binding count must be 1..={MAX_BINDINGS}"
        ));
    }
    let mut previous: Option<&str> = None;
    for binding in &manifest.bindings {
        validate_label(&binding.row_id, "classical binding row id")?;
        validate_file_ref(&binding.reference_audio, "classical reference audio")?;
        if binding.energy_q16 > 65_536 {
            return Err("classical binding energy_q16 must be at most 65536".to_owned());
        }
        if previous.is_some_and(|prior| prior >= binding.row_id.as_str()) {
            return Err("classical bindings must be strictly sorted by row_id".to_owned());
        }
        previous = Some(&binding.row_id);
    }
    Ok(())
}

fn validate_projection(projection: &DataProjection, bindings: &[RowBinding]) -> Result<(), String> {
    if projection.schema != PROJECTION_SCHEMA || projection.task_scope != TASK_SCOPE {
        return Err(
            "classical baseline requires the exact PS-2N0 fit projection schema and task scope"
                .to_owned(),
        );
    }
    if projection.role_scope != ["train", "development"] {
        return Err(
            "classical baseline accepts only the unsealed train/development projection".to_owned(),
        );
    }
    validate_label(&projection.projection_id, "classical projection id")?;
    validate_label(&projection.revision, "classical projection revision")?;
    validate_sha256(
        &projection.manifest_sha256,
        "classical projection manifest sha256",
    )?;
    if projection.rows.len() != bindings.len() {
        return Err(
            "classical baseline requires exactly one binding for every projected row".to_owned(),
        );
    }
    let mut previous: Option<&str> = None;
    for (row, binding) in projection.rows.iter().zip(bindings) {
        validate_label(&row.row_id, "projected classical row id")?;
        if !matches!(row.split_role.as_str(), "train" | "development") {
            return Err(format!(
                "sealed or unsupported row role: {}",
                row.split_role
            ));
        }
        if !matches!(row.sample_role.as_str(), "context" | "query")
            || !matches!(row.corpus_role.as_str(), "target" | "reject_parent")
        {
            return Err(format!("invalid projected row role for {}", row.row_id));
        }
        if row.audio.byte_count == 0 {
            return Err(format!(
                "invalid projected audio identity for {}",
                row.row_id
            ));
        }
        validate_sha256(&row.audio.sha256, "projected classical audio sha256")?;
        if previous.is_some_and(|prior| prior >= row.row_id.as_str()) {
            return Err("projected classical rows must be strictly sorted by row_id".to_owned());
        }
        if row.row_id != binding.row_id {
            return Err(
                "classical bindings must match projected row order and identity".to_owned(),
            );
        }
        previous = Some(&row.row_id);
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

fn build_profile() -> ClassicalProfileRecord {
    let snapshot = selected_glass_q30_profile_snapshot();
    ClassicalProfileRecord {
        schema: PROFILE_SCHEMA,
        revision: snapshot.revision,
        source_profile_sha256: snapshot.source_profile_sha256,
        numeric_profile: "signed_q30_second_order_recurrence_stereo_center_v1",
        sample_rate_hz: snapshot.sample_rate_hz,
        channel_count: snapshot.channel_count,
        duration_frames: snapshot.duration_frames,
        canonical_mode_order: "frozen_source_profile_index_ascending",
        modes: snapshot
            .modes
            .into_iter()
            .enumerate()
            .map(|(canonical_index, mode)| ModeRecord {
                canonical_index,
                coefficient_a_q30: mode.coefficient_a_q30,
                coefficient_b_q30: mode.coefficient_b_q30,
                initial_sample_q30: mode.initial_sample_q30,
            })
            .collect(),
        residual: ResidualRecord {
            kind: "bounded_onset_transient_q30",
            sample_count: snapshot.transient_q30.len(),
            samples_q30: snapshot.transient_q30.to_vec(),
        },
        scaling: ScalingRecord {
            raw_peak_q30: snapshot.raw_peak_q30.to_string(),
            center_channel_peak: snapshot.center_channel_peak.to_string(),
            energy_profile: "explicit_q16_16_clamped_0_to_1",
            stereo_projection: "centered_mono_equal_power_by_integer_half_sum",
        },
        colored_dct_boundary: ColoredDctBoundary {
            candidate_id: "modal_plus_seeded_colored_subband_residual_v1",
            preregistration_source: FrozenSource {
                path: DCT_PREREGISTRATION_PATH,
                sha256: DCT_PREREGISTRATION_SHA256,
            },
            boundary_pipeline_source: FrozenSource {
                path: DCT_PIPELINE_PATH,
                sha256: DCT_PIPELINE_SHA256,
            },
            maximum_coefficients_per_band: 8,
            decision: "FallbackOutOfDomain",
            reason: "frozen_dct_protocol_has_no_hash_closed_per_row_fit_or_exact_cooked_renderer",
        },
    }
}

fn validate_frozen_sources(root: &Path) -> Result<(), String> {
    for (path, expected) in [
        (DCT_PREREGISTRATION_PATH, DCT_PREREGISTRATION_SHA256),
        (DCT_PIPELINE_PATH, DCT_PIPELINE_SHA256),
    ] {
        let bytes = fs::read(root.join(path))
            .map_err(|error| format!("read frozen classical source {path}: {error}"))?;
        let actual = sha256_hex(&bytes);
        if actual != expected {
            return Err(format!(
                "frozen classical source changed for {path}: expected {expected}, got {actual}"
            ));
        }
    }
    Ok(())
}

fn fallback_reason(row: &ProjectionRow, profile: BaselineProfile) -> Option<CoverageStatus> {
    if row.corpus_role != "target" {
        return Some(CoverageStatus {
            decision: "FallbackOutOfDomain",
            reason: "reject_parent_is_validation_evidence_not_a_target_prediction",
        });
    }
    if !row.axes.complete_for_modal_field() {
        return Some(CoverageStatus {
            decision: "FallbackOutOfDomain",
            reason: "declared_modal_field_axis_coverage_incomplete",
        });
    }
    if profile == BaselineProfile::FrozenColoredDctResidualV1 {
        return Some(CoverageStatus {
            decision: "FallbackOutOfDomain",
            reason: "frozen_dct_protocol_has_no_hash_closed_per_row_fit_or_exact_cooked_renderer",
        });
    }
    None
}

fn acoustic_features(
    label: &str,
    wav_sha256: &str,
    wav: WavAudio,
) -> Result<AcousticFeatures, String> {
    let frame_count = wav.mono_samples.len();
    let sample_rate_hz = wav.sample_rate_hz;
    let channel_count = wav.channel_count;
    let analysis = analyze_benchmark_wav(label, wav_sha256, wav)?;
    Ok(features_from_analysis(
        frame_count,
        sample_rate_hz,
        channel_count,
        analysis,
    ))
}

fn features_from_analysis(
    frame_count: usize,
    sample_rate_hz: u32,
    channel_count: u16,
    analysis: BenchmarkAudioAnalysis,
) -> AcousticFeatures {
    AcousticFeatures {
        sample_rate_hz,
        channel_count,
        frame_count,
        duration_microseconds: u64::try_from(frame_count)
            .unwrap_or(u64::MAX)
            .saturating_mul(1_000_000)
            / u64::from(sample_rate_hz),
        peak_dbfs: analysis.peak_dbfs,
        rms_dbfs: analysis.rms_dbfs,
        hard_failure_tags: analysis.hard_failure_tags,
        validator_feature_values: analysis.feature_values,
        temporal_feature_values: analysis.temporal_feature_values,
        amplitude_envelope_feature_values: analysis.amplitude_envelope_feature_values,
    }
}

fn compare_audio(
    reference: &WavAudio,
    prediction: &WavAudio,
    reference_sha256: &str,
    prediction_sha256: &str,
    exact_wav_match: bool,
) -> ComparisonReport {
    let comparable = reference.sample_rate_hz == prediction.sample_rate_hz
        && reference.mono_samples.len() == prediction.mono_samples.len();
    let (maximum_absolute_error, rms_error, correlation) = if comparable {
        let mut maximum = 0.0_f64;
        let mut squared_error = 0.0_f64;
        let reference_mean =
            reference.mono_samples.iter().sum::<f64>() / reference.mono_samples.len() as f64;
        let prediction_mean =
            prediction.mono_samples.iter().sum::<f64>() / prediction.mono_samples.len() as f64;
        let mut covariance = 0.0_f64;
        let mut reference_energy = 0.0_f64;
        let mut prediction_energy = 0.0_f64;
        for (left, right) in reference.mono_samples.iter().zip(&prediction.mono_samples) {
            let error = left - right;
            maximum = maximum.max(error.abs());
            squared_error += error * error;
            let left_centered = left - reference_mean;
            let right_centered = right - prediction_mean;
            covariance += left_centered * right_centered;
            reference_energy += left_centered * left_centered;
            prediction_energy += right_centered * right_centered;
        }
        let denominator = (reference_energy * prediction_energy).sqrt();
        (
            Some(maximum),
            Some((squared_error / reference.mono_samples.len() as f64).sqrt()),
            Some(if denominator > 0.0 {
                covariance / denominator
            } else if exact_wav_match {
                1.0
            } else {
                0.0
            }),
        )
    } else {
        (None, None, None)
    };
    ComparisonReport {
        reference_wav_sha256: reference_sha256.to_owned(),
        prediction_wav_sha256: prediction_sha256.to_owned(),
        exact_wav_match,
        comparable_mono_pcm: comparable,
        maximum_absolute_error,
        rms_error,
        correlation,
    }
}
