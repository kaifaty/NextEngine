use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

use crate::physical_sound_eval_command::audio_analysis::{WavAudio, parse_wav};
use crate::physical_sound_eval_command::compare_benchmark_wav;

use super::{
    FileRef, MAX_MANIFEST_BYTES, MAX_REFERENCED_FILE_BYTES, canonical_external_file,
    read_bounded_file, require_empty_output, resolve_artifact, resolve_cli_path,
    resolve_output_path, set_once, sha256_hex, validate_file_ref, validate_label,
};

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-transfer-field-baseline.manifest.v1";
const PROJECTION_SCHEMA: &str =
    "nextengine.experimental-physical-sound-neural-data-plane.projection.v2";
const REPORT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-transfer-field-baseline.report.v1";
const CLAIM: &str = "FIXED_IMPACT_RELATIVE_TRANSFER_FIELD_CLASSICAL_CONTROLS_ONLY / NO_RECORDED_WAVEFORM_MODEL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY";
const TASK_SCOPE: &str = "exact_object_few_shot_impact_listener_field";
const DEVELOPMENT_ROLE: &str = "development";
const TARGET_ROLE: &str = "target";
const CONTEXT_ROLE: &str = "context";
const QUERY_ROLE: &str = "query";
const TRANSFER_SEMANTICS: &str = "force_deconvolved_transfer_response";
const MAX_BINDINGS: usize = 65_536;
const SEGMENT_RESIDUAL_SQUARED_LIMIT: f64 = 1.0e-18;
const DB_FLOOR: f64 = -240.0;

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
            _ => {
                return Err(format!(
                    "unexpected transfer-field-baseline argument: {flag}"
                ));
            }
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "transfer-field-baseline requires --manifest <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "transfer-field-baseline requires --output <external-empty-directory>".to_owned()
        })?,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    baseline_id: String,
    revision: String,
    projection: FileRef,
    bindings: Vec<AudioBinding>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AudioBinding {
    row_id: String,
    audio: FileRef,
}

#[derive(Deserialize)]
struct Projection {
    schema: String,
    projection_id: String,
    revision: String,
    task_scope: String,
    manifest_sha256: String,
    role_scope: Vec<String>,
    rows: Vec<ProjectionRow>,
}

#[derive(Clone, Deserialize)]
struct ProjectionRow {
    row_id: String,
    split_role: String,
    sample_role: String,
    corpus_role: String,
    audio_semantics: String,
    object_group_id: String,
    recording_parent_id: String,
    audio: ProjectedArtifact,
    axes: ProjectionAxes,
}

#[derive(Clone, Deserialize)]
struct ProjectedArtifact {
    sha256: String,
    byte_count: usize,
}

#[derive(Clone, Deserialize)]
struct ProjectionAxes {
    impact: Option<PointClaim>,
    listener: Option<PointClaim>,
}

#[derive(Clone, Deserialize, PartialEq)]
struct PointClaim {
    coordinate_profile: String,
    point_metres: [f64; 3],
}

struct LoadedRow {
    projection: ProjectionRow,
    audio: WavAudio,
}

#[derive(Clone, Copy)]
struct Segment<'a> {
    left: &'a LoadedRow,
    right: &'a LoadedRow,
    right_weight: f64,
    span_squared: f64,
}

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    baseline_id: String,
    revision: String,
    manifest_sha256: String,
    projection_sha256: String,
    projection_id: String,
    projection_revision: String,
    projection_manifest_sha256: String,
    task_scope: String,
    metric_profile: MetricProfile,
    context_row_count: usize,
    query_row_count: usize,
    controls: Vec<ControlAggregate>,
    rows: Vec<QueryReport>,
    exact_repeat_required: bool,
    model_training_authorized: bool,
    admission_shadow_opened: bool,
    emitted_files: Vec<String>,
}

#[derive(Serialize)]
struct MetricProfile {
    id: &'static str,
    primary_endpoints: [&'static str; 5],
    success_rule_for_r2: &'static str,
    aggregation: &'static str,
    waveform_alignment: &'static str,
    spectrum_gain_match: &'static str,
    db_floor: f64,
    segment_residual_squared_limit: f64,
}

impl MetricProfile {
    const fn frozen() -> Self {
        Self {
            id: "transfer-listener-field-r1-v1",
            primary_endpoints: [
                "mean_absolute_rms_level_error_db",
                "p95_absolute_rms_level_error_db",
                "mean_gain_matched_multiresolution_log_spectrum_rmse_db",
                "p95_gain_matched_multiresolution_log_spectrum_rmse_db",
                "mean_normalized_waveform_rmse_db",
            ],
            success_rule_for_r2: "candidate_must_be_strictly_lower_than_each_frozen_control_on_every_primary_endpoint_without_changing_rows_preprocessing_or_aggregation",
            aggregation: "unweighted_per_query_mean_and_nearest_rank_p95_with_full_per_query_distribution",
            waveform_alignment: "sample_synchronous_no_shift",
            spectrum_gain_match: "separate_raw_rms_delta_and_normalize_each_clip_spectrum_to_its_own_maximum",
            db_floor: DB_FLOOR,
            segment_residual_squared_limit: SEGMENT_RESIDUAL_SQUARED_LIMIT,
        }
    }
}

#[derive(Serialize)]
struct ControlAggregate {
    control: &'static str,
    query_count: usize,
    mean_absolute_rms_level_error_db: f64,
    p95_absolute_rms_level_error_db: f64,
    maximum_absolute_rms_level_error_db: f64,
    mean_gain_matched_multiresolution_log_spectrum_rmse_db: f64,
    p95_gain_matched_multiresolution_log_spectrum_rmse_db: f64,
    maximum_gain_matched_multiresolution_log_spectrum_rmse_db: f64,
    mean_normalized_waveform_rmse_db: f64,
    p95_normalized_waveform_rmse_db: f64,
    maximum_normalized_waveform_rmse_db: f64,
}

#[derive(Serialize)]
struct QueryReport {
    row_id: String,
    listener_point_metres: [f64; 3],
    reference_audio_sha256: String,
    nearest: PredictionReport,
    linear_segment: PredictionReport,
}

#[derive(Serialize)]
struct PredictionReport {
    control: &'static str,
    context_row_ids: Vec<String>,
    context_weights: Vec<f64>,
    prediction_file: String,
    prediction_sha256: String,
    prediction_byte_count: usize,
    raw_rms_delta_db: f64,
    absolute_rms_level_error_db: f64,
    gain_matched_multiresolution_log_spectrum_rmse_db: f64,
    normalized_waveform_rmse_db: f64,
    waveform_correlation: f64,
    modal_assignment_cost: f64,
    mean_t20_delta_ms: Option<f64>,
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, &request.manifest),
        "transfer field baseline manifest",
    )?;
    let output = resolve_output_path(root, &request.output)?;
    require_empty_output(&output)?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "transfer field baseline manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "transfer field baseline manifest has no parent".to_owned())?;
    let projection_report = resolve_artifact(
        root,
        manifest_directory,
        &manifest.projection,
        "transfer field projection",
    )?;
    let projection_path = canonical_external_file(
        root,
        &manifest_directory.join(&manifest.projection.path),
        "transfer field projection",
    )?;
    let projection_bytes = read_bounded_file(
        &projection_path,
        MAX_REFERENCED_FILE_BYTES,
        "transfer field projection",
    )?;
    let projection: Projection = serde_json::from_slice(&projection_bytes)
        .map_err(|error| format!("parse {}: {error}", projection_path.display()))?;
    validate_projection(&projection)?;
    let loaded = load_rows(root, manifest_directory, &manifest, &projection)?;
    let context = loaded
        .iter()
        .filter(|row| row.projection.sample_role == CONTEXT_ROLE)
        .collect::<Vec<_>>();
    let query = loaded
        .iter()
        .filter(|row| row.projection.sample_role == QUERY_ROLE)
        .collect::<Vec<_>>();
    validate_audio_grid(&context, &query)?;

    let mut rows = Vec::with_capacity(query.len());
    let mut emitted_files = Vec::with_capacity(query.len() * 2 + 1);
    let mut prediction_payloads = Vec::with_capacity(query.len() * 2);
    for target in query {
        let compatible = compatible_context(&context, target)?;
        let nearest = nearest_context(&compatible, target)?;
        let segment = bracketing_segment(&compatible, target)?;
        let nearest_samples = nearest.audio.mono_samples.clone();
        let linear_samples = interpolate_samples(
            &segment.left.audio.mono_samples,
            &segment.right.audio.mono_samples,
            segment.right_weight,
        )?;
        let nearest_name = format!(
            "predictions/{}.nearest-listener.wav",
            target.projection.row_id
        );
        let linear_name = format!(
            "predictions/{}.linear-listener-segment.wav",
            target.projection.row_id
        );
        let nearest_wav = encode_pcm16_mono_wav(&nearest_samples, target.audio.sample_rate_hz)?;
        let linear_wav = encode_pcm16_mono_wav(&linear_samples, target.audio.sample_rate_hz)?;
        let nearest_report = prediction_report(
            "nearest_listener",
            vec![nearest.projection.row_id.clone()],
            vec![1.0],
            nearest_name.clone(),
            &nearest_wav,
            nearest_samples,
            &target.audio,
        )?;
        let linear_report = prediction_report(
            "linear_listener_segment",
            vec![
                segment.left.projection.row_id.clone(),
                segment.right.projection.row_id.clone(),
            ],
            vec![1.0 - segment.right_weight, segment.right_weight],
            linear_name.clone(),
            &linear_wav,
            linear_samples,
            &target.audio,
        )?;
        prediction_payloads.push((nearest_name.clone(), nearest_wav));
        prediction_payloads.push((linear_name.clone(), linear_wav));
        emitted_files.push(nearest_name);
        emitted_files.push(linear_name);
        rows.push(QueryReport {
            row_id: target.projection.row_id.clone(),
            listener_point_metres: listener_point(target)?,
            reference_audio_sha256: target.projection.audio.sha256.clone(),
            nearest: nearest_report,
            linear_segment: linear_report,
        });
    }
    rows.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let controls = vec![
        aggregate("nearest_listener", rows.iter().map(|row| &row.nearest)),
        aggregate(
            "linear_listener_segment",
            rows.iter().map(|row| &row.linear_segment),
        ),
    ];
    emitted_files.sort();
    emitted_files.push("report.json".to_owned());
    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "FrozenTransferControlsAvailable",
        claim: CLAIM,
        baseline_id: manifest.baseline_id,
        revision: manifest.revision,
        manifest_sha256: sha256_hex(&manifest_bytes),
        projection_sha256: projection_report.sha256,
        projection_id: projection.projection_id,
        projection_revision: projection.revision,
        projection_manifest_sha256: projection.manifest_sha256,
        task_scope: projection.task_scope,
        metric_profile: MetricProfile::frozen(),
        context_row_count: context.len(),
        query_row_count: rows.len(),
        controls,
        rows,
        exact_repeat_required: true,
        model_training_authorized: false,
        admission_shadow_opened: false,
        emitted_files,
    };
    let report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("serialize transfer field baseline report: {error}"))?;
    fs::create_dir_all(output.join("predictions"))
        .map_err(|error| format!("create {}: {error}", output.display()))?;
    for (name, bytes) in prediction_payloads {
        fs::write(output.join(name), bytes)
            .map_err(|error| format!("write transfer prediction: {error}"))?;
    }
    fs::write(output.join("report.json"), &report_bytes)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_bytes).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported transfer field baseline schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.baseline_id, "transfer baseline id")?;
    validate_label(&manifest.revision, "transfer baseline revision")?;
    validate_file_ref(&manifest.projection, "transfer baseline projection")?;
    if manifest.bindings.is_empty() || manifest.bindings.len() > MAX_BINDINGS {
        return Err(format!("binding count must be 1..={MAX_BINDINGS}"));
    }
    let mut previous = None;
    for binding in &manifest.bindings {
        validate_label(&binding.row_id, "transfer binding row id")?;
        validate_file_ref(&binding.audio, "transfer binding audio")?;
        if previous.is_some_and(|value: &str| value >= binding.row_id.as_str()) {
            return Err("transfer bindings must be strictly sorted by row_id".to_owned());
        }
        previous = Some(binding.row_id.as_str());
    }
    Ok(())
}

fn validate_projection(projection: &Projection) -> Result<(), String> {
    if projection.schema != PROJECTION_SCHEMA {
        return Err(format!(
            "unsupported transfer field projection schema: {}",
            projection.schema
        ));
    }
    if projection.task_scope != TASK_SCOPE {
        return Err(format!(
            "transfer field projection task scope must be {TASK_SCOPE}"
        ));
    }
    if !projection
        .role_scope
        .iter()
        .any(|role| role == DEVELOPMENT_ROLE)
    {
        return Err("transfer field projection does not include development role".to_owned());
    }
    Ok(())
}

fn load_rows(
    root: &Path,
    manifest_directory: &Path,
    manifest: &Manifest,
    projection: &Projection,
) -> Result<Vec<LoadedRow>, String> {
    let eligible = projection
        .rows
        .iter()
        .filter(|row| {
            row.split_role == DEVELOPMENT_ROLE
                && row.corpus_role == TARGET_ROLE
                && row.audio_semantics == TRANSFER_SEMANTICS
        })
        .collect::<Vec<_>>();
    let eligible_ids = eligible
        .iter()
        .map(|row| row.row_id.as_str())
        .collect::<BTreeSet<_>>();
    let binding_ids = manifest
        .bindings
        .iter()
        .map(|binding| binding.row_id.as_str())
        .collect::<BTreeSet<_>>();
    if eligible_ids != binding_ids {
        return Err(
            "transfer bindings must exactly cover eligible development transfer rows".to_owned(),
        );
    }
    let binding_map = manifest
        .bindings
        .iter()
        .map(|binding| (binding.row_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut loaded = Vec::with_capacity(eligible.len());
    for row in eligible {
        let binding = binding_map
            .get(row.row_id.as_str())
            .ok_or_else(|| format!("missing binding for {}", row.row_id))?;
        let report = resolve_artifact(
            root,
            manifest_directory,
            &binding.audio,
            "transfer baseline WAV",
        )?;
        if report.sha256 != row.audio.sha256 || report.byte_count != row.audio.byte_count {
            return Err(format!(
                "binding for {} does not match projected audio identity",
                row.row_id
            ));
        }
        let path = canonical_external_file(
            root,
            &manifest_directory.join(&binding.audio.path),
            "transfer baseline WAV",
        )?;
        let bytes = read_bounded_file(&path, MAX_REFERENCED_FILE_BYTES, "transfer baseline WAV")?;
        let audio = parse_wav(&bytes)
            .map_err(|error| format!("parse transfer WAV {}: {error}", path.display()))?;
        if audio.sample_format != "pcm-s16" || audio.channel_count != 1 {
            return Err(format!("transfer WAV {} must be mono PCM16", row.row_id));
        }
        if row.axes.impact.is_none() || row.axes.listener.is_none() {
            return Err(format!(
                "transfer row {} requires impact and listener coordinates",
                row.row_id
            ));
        }
        loaded.push(LoadedRow {
            projection: row.clone(),
            audio,
        });
    }
    Ok(loaded)
}

fn validate_audio_grid(context: &[&LoadedRow], query: &[&LoadedRow]) -> Result<(), String> {
    if context.len() < 2 || query.is_empty() {
        return Err("transfer baseline requires at least two context and one query row".to_owned());
    }
    let first = &context[0].audio;
    for row in context.iter().chain(query) {
        if row.audio.sample_rate_hz != first.sample_rate_hz
            || row.audio.channel_count != first.channel_count
            || row.audio.mono_samples.len() != first.mono_samples.len()
        {
            return Err("transfer baseline WAV grid must share format and sample count".to_owned());
        }
    }
    Ok(())
}

fn compatible_context<'a>(
    context: &'a [&LoadedRow],
    query: &LoadedRow,
) -> Result<Vec<&'a LoadedRow>, String> {
    let values = context
        .iter()
        .copied()
        .filter(|row| {
            row.projection.object_group_id == query.projection.object_group_id
                && row.projection.recording_parent_id == query.projection.recording_parent_id
                && row.projection.axes.impact == query.projection.axes.impact
                && row
                    .projection
                    .axes
                    .listener
                    .as_ref()
                    .map(|claim| &claim.coordinate_profile)
                    == query
                        .projection
                        .axes
                        .listener
                        .as_ref()
                        .map(|claim| &claim.coordinate_profile)
        })
        .collect::<Vec<_>>();
    if values.len() < 2 {
        return Err(format!(
            "query {} has fewer than two compatible context rows",
            query.projection.row_id
        ));
    }
    Ok(values)
}

fn listener_point(row: &LoadedRow) -> Result<[f64; 3], String> {
    row.projection
        .axes
        .listener
        .as_ref()
        .map(|claim| claim.point_metres)
        .ok_or_else(|| format!("row {} has no listener point", row.projection.row_id))
}

fn squared_distance(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum()
}

fn nearest_context<'a>(
    context: &[&'a LoadedRow],
    query: &LoadedRow,
) -> Result<&'a LoadedRow, String> {
    let point = listener_point(query)?;
    context
        .iter()
        .copied()
        .min_by(|left, right| {
            let left_distance = squared_distance(listener_point(left).unwrap_or(point), point);
            let right_distance = squared_distance(listener_point(right).unwrap_or(point), point);
            left_distance
                .total_cmp(&right_distance)
                .then_with(|| left.projection.row_id.cmp(&right.projection.row_id))
        })
        .ok_or_else(|| format!("query {} has no context row", query.projection.row_id))
}

fn bracketing_segment<'a>(
    context: &[&'a LoadedRow],
    query: &LoadedRow,
) -> Result<Segment<'a>, String> {
    let point = listener_point(query)?;
    let mut best = None;
    for left_index in 0..context.len() {
        for right_index in left_index + 1..context.len() {
            let left = context[left_index];
            let right = context[right_index];
            let left_point = listener_point(left)?;
            let right_point = listener_point(right)?;
            let direction = [
                right_point[0] - left_point[0],
                right_point[1] - left_point[1],
                right_point[2] - left_point[2],
            ];
            let span_squared = direction.iter().map(|value| value * value).sum::<f64>();
            if span_squared <= f64::EPSILON {
                continue;
            }
            let query_offset = [
                point[0] - left_point[0],
                point[1] - left_point[1],
                point[2] - left_point[2],
            ];
            let right_weight = query_offset
                .into_iter()
                .zip(direction)
                .map(|(offset, direction)| offset * direction)
                .sum::<f64>()
                / span_squared;
            if !(0.0..=1.0).contains(&right_weight) {
                continue;
            }
            let projected = [
                left_point[0] + right_weight * direction[0],
                left_point[1] + right_weight * direction[1],
                left_point[2] + right_weight * direction[2],
            ];
            if squared_distance(projected, point) > SEGMENT_RESIDUAL_SQUARED_LIMIT {
                continue;
            }
            let candidate = Segment {
                left,
                right,
                right_weight,
                span_squared,
            };
            let ordering = best.as_ref().map(|current: &Segment<'_>| {
                candidate
                    .span_squared
                    .total_cmp(&current.span_squared)
                    .then_with(|| {
                        candidate
                            .left
                            .projection
                            .row_id
                            .cmp(&current.left.projection.row_id)
                    })
                    .then_with(|| {
                        candidate
                            .right
                            .projection
                            .row_id
                            .cmp(&current.right.projection.row_id)
                    })
            });
            if ordering.is_none_or(|ordering| ordering == Ordering::Less) {
                best = Some(candidate);
            }
        }
    }
    best.ok_or_else(|| {
        format!(
            "query {} is not bracketed by a compatible listener segment",
            query.projection.row_id
        )
    })
}

fn interpolate_samples(left: &[f64], right: &[f64], weight: f64) -> Result<Vec<f64>, String> {
    if left.len() != right.len() {
        return Err("transfer interpolation sample length mismatch".to_owned());
    }
    Ok(left
        .iter()
        .zip(right)
        .map(|(left, right)| left.mul_add(1.0 - weight, right * weight))
        .collect())
}

fn prediction_report(
    control: &'static str,
    context_row_ids: Vec<String>,
    context_weights: Vec<f64>,
    prediction_file: String,
    wav: &[u8],
    prediction_samples: Vec<f64>,
    reference: &WavAudio,
) -> Result<PredictionReport, String> {
    if prediction_samples.len() != reference.mono_samples.len() {
        return Err("prediction sample count changed before PCM cooking".to_owned());
    }
    let prediction =
        parse_wav(wav).map_err(|error| format!("parse cooked transfer prediction: {error}"))?;
    let normalized_waveform_rmse_db =
        normalized_waveform_rmse_db(&prediction.mono_samples, &reference.mono_samples)?;
    let waveform_correlation =
        waveform_correlation(&prediction.mono_samples, &reference.mono_samples)?;
    let matched = compare_benchmark_wav(prediction, clone_audio(reference))?;
    Ok(PredictionReport {
        control,
        context_row_ids,
        context_weights,
        prediction_file,
        prediction_sha256: sha256_hex(wav),
        prediction_byte_count: wav.len(),
        raw_rms_delta_db: matched.raw_rms_delta_db,
        absolute_rms_level_error_db: matched.raw_rms_delta_db.abs(),
        gain_matched_multiresolution_log_spectrum_rmse_db: matched
            .gain_matched_multiresolution_log_spectrum_rmse_db,
        normalized_waveform_rmse_db,
        waveform_correlation,
        modal_assignment_cost: matched.modal_assignment_cost,
        mean_t20_delta_ms: matched.mean_t20_delta_ms,
    })
}

fn clone_audio(audio: &WavAudio) -> WavAudio {
    WavAudio {
        sample_format: audio.sample_format.clone(),
        sample_rate_hz: audio.sample_rate_hz,
        channel_count: audio.channel_count,
        mono_samples: audio.mono_samples.clone(),
    }
}

fn normalized_waveform_rmse_db(candidate: &[f64], reference: &[f64]) -> Result<f64, String> {
    if candidate.len() != reference.len() || candidate.is_empty() {
        return Err("waveform metric requires equal non-empty sample grids".to_owned());
    }
    let error_energy = candidate
        .iter()
        .zip(reference)
        .map(|(candidate, reference)| (candidate - reference).powi(2))
        .sum::<f64>();
    let reference_energy = reference.iter().map(|sample| sample * sample).sum::<f64>();
    if reference_energy <= 0.0 {
        return Err("waveform metric reference is silent".to_owned());
    }
    let ratio = (error_energy / reference_energy).sqrt();
    Ok(if ratio <= 1.0e-12 {
        DB_FLOOR
    } else {
        (20.0 * ratio.log10()).max(DB_FLOOR)
    })
}

fn waveform_correlation(candidate: &[f64], reference: &[f64]) -> Result<f64, String> {
    if candidate.len() != reference.len() || candidate.is_empty() {
        return Err("correlation requires equal non-empty sample grids".to_owned());
    }
    let dot = candidate
        .iter()
        .zip(reference)
        .map(|(candidate, reference)| candidate * reference)
        .sum::<f64>();
    let candidate_energy = candidate.iter().map(|sample| sample * sample).sum::<f64>();
    let reference_energy = reference.iter().map(|sample| sample * sample).sum::<f64>();
    let denominator = (candidate_energy * reference_energy).sqrt();
    if denominator <= 0.0 {
        return Err("correlation requires non-silent signals".to_owned());
    }
    Ok((dot / denominator).clamp(-1.0, 1.0))
}

fn encode_pcm16_mono_wav(samples: &[f64], sample_rate_hz: u32) -> Result<Vec<u8>, String> {
    let data_bytes = samples
        .len()
        .checked_mul(2)
        .ok_or_else(|| "PCM16 WAV length overflow".to_owned())?;
    let riff_size = 36_usize
        .checked_add(data_bytes)
        .ok_or_else(|| "PCM16 RIFF length overflow".to_owned())?;
    let mut wav = Vec::with_capacity(44 + data_bytes);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(
        &u32::try_from(riff_size)
            .map_err(|_| "PCM16 RIFF length exceeds u32".to_owned())?
            .to_le_bytes(),
    );
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate_hz.to_le_bytes());
    wav.extend_from_slice(&(sample_rate_hz * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(
        &u32::try_from(data_bytes)
            .map_err(|_| "PCM16 data length exceeds u32".to_owned())?
            .to_le_bytes(),
    );
    for sample in samples {
        if !sample.is_finite() {
            return Err("PCM16 encoder received non-finite sample".to_owned());
        }
        let quantized = (sample.clamp(-1.0, 1.0) * 32_768.0)
            .round()
            .clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16;
        wav.extend_from_slice(&quantized.to_le_bytes());
    }
    Ok(wav)
}

fn aggregate<'a>(
    control: &'static str,
    rows: impl Iterator<Item = &'a PredictionReport>,
) -> ControlAggregate {
    let rows = rows.collect::<Vec<_>>();
    let rms = rows
        .iter()
        .map(|row| row.absolute_rms_level_error_db)
        .collect::<Vec<_>>();
    let spectrum = rows
        .iter()
        .map(|row| row.gain_matched_multiresolution_log_spectrum_rmse_db)
        .collect::<Vec<_>>();
    let waveform = rows
        .iter()
        .map(|row| row.normalized_waveform_rmse_db)
        .collect::<Vec<_>>();
    ControlAggregate {
        control,
        query_count: rows.len(),
        mean_absolute_rms_level_error_db: mean(&rms),
        p95_absolute_rms_level_error_db: nearest_rank_percentile(&rms, 95),
        maximum_absolute_rms_level_error_db: maximum(&rms),
        mean_gain_matched_multiresolution_log_spectrum_rmse_db: mean(&spectrum),
        p95_gain_matched_multiresolution_log_spectrum_rmse_db: nearest_rank_percentile(
            &spectrum, 95,
        ),
        maximum_gain_matched_multiresolution_log_spectrum_rmse_db: maximum(&spectrum),
        mean_normalized_waveform_rmse_db: mean(&waveform),
        p95_normalized_waveform_rmse_db: nearest_rank_percentile(&waveform, 95),
        maximum_normalized_waveform_rmse_db: maximum(&waveform),
    }
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn maximum(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn nearest_rank_percentile(values: &[f64], percentile: usize) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (percentile * sorted.len()).div_ceil(100).max(1);
    sorted[rank - 1]
}
