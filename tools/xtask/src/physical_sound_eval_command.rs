use std::fs;
use std::path::{Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use serde::{Deserialize, Serialize};

pub(super) mod audio_analysis;
#[cfg(test)]
mod tests;

use audio_analysis::{Analysis, analyze_wav, parse_wav};

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-quality.manifest.v0";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-quality.report.v0";
const EVALUATOR_PROFILE: &str = "nextengine.experimental-physical-sound-quality.classical.v0";
const MAX_ENTRIES: usize = 512;
const MAX_WAV_BYTES: usize = 256 * 1024 * 1024;
const MAX_DURATION_SECONDS: usize = 30;
const LOG_SPECTRUM_BINS: usize = 96;
const FFT_SIZES: [usize; 3] = [2_048, 8_192, 32_768];

pub(super) struct Request {
    manifest: PathBuf,
    output: PathBuf,
    blind_seed: u64,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let mut manifest = None;
    let mut output = None;
    let mut blind_seed = 0x51a7_2026_0826_u64;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            "--blind-seed" => {
                blind_seed = value
                    .parse::<u64>()
                    .map_err(|error| format!("invalid --blind-seed {value}: {error}"))?;
            }
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest
            .ok_or_else(|| "physical-sound-eval requires --manifest <external-json>".to_owned())?,
        output: output.ok_or_else(|| {
            "physical-sound-eval requires --output <external-empty-directory>".to_owned()
        })?,
        blind_seed,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct QualityManifest {
    schema: String,
    split: String,
    entries: Vec<ManifestEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ManifestEntry {
    id: String,
    object_id: String,
    material: String,
    impact_position: String,
    force_band: String,
    candidate: AudioFileRef,
    reference: Option<AudioFileRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AudioFileRef {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug)]
pub(super) struct Q0Candidate {
    pub id: String,
    pub object_id: String,
    pub material: String,
    pub impact_position: String,
    pub force_band: String,
    pub file: String,
    pub sha256: String,
}

pub(super) fn write_q0_manifest(
    output_directory: &Path,
    candidates: &[Q0Candidate],
) -> Result<(), String> {
    let mut candidates = candidates.to_vec();
    candidates.sort_by(|left, right| left.id.cmp(&right.id));
    let manifest = QualityManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        split: "q0-negative-baseline".to_owned(),
        entries: candidates
            .iter()
            .map(|candidate| ManifestEntry {
                id: candidate.id.clone(),
                object_id: candidate.object_id.clone(),
                material: candidate.material.clone(),
                impact_position: candidate.impact_position.clone(),
                force_band: candidate.force_band.clone(),
                candidate: AudioFileRef {
                    path: candidate.file.clone(),
                    sha256: candidate.sha256.clone(),
                },
                reference: None,
            })
            .collect(),
    };
    let json = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    fs::write(output_directory.join("quality-manifest.json"), json)
        .map_err(|error| format!("write quality-manifest.json: {error}"))
}

#[derive(Clone, Debug, Serialize)]
struct EvaluatorProfileReport {
    id: &'static str,
    onset_rule: &'static str,
    gain_match_rule: &'static str,
    fft_sizes: [usize; 3],
    log_spectrum_bins: usize,
    modal_peak_limit: usize,
    decay_window_frames: usize,
    decay_hop_frames: usize,
    maximum_entries: usize,
    maximum_wav_bytes: usize,
    maximum_duration_seconds: usize,
}

impl EvaluatorProfileReport {
    const fn current() -> Self {
        Self {
            id: EVALUATOR_PROFILE,
            onset_rule: "first absolute sample >= max(2% clip peak, -80 dBFS)",
            gain_match_rule: "separate raw RMS delta; spectra normalized to each clip maximum",
            fft_sizes: FFT_SIZES,
            log_spectrum_bins: LOG_SPECTRUM_BINS,
            modal_peak_limit: 12,
            decay_window_frames: 1_024,
            decay_hop_frames: 256,
            maximum_entries: MAX_ENTRIES,
            maximum_wav_bytes: MAX_WAV_BYTES,
            maximum_duration_seconds: MAX_DURATION_SECONDS,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct QualityReport {
    schema: &'static str,
    status: String,
    claim: &'static str,
    split: String,
    manifest_sha256: String,
    evaluator_profile: EvaluatorProfileReport,
    evaluator_profile_sha256: String,
    entry_count: usize,
    matched_reference_count: usize,
    blind_bundle: Option<BlindBundleReport>,
    entries: Vec<EntryReport>,
}

#[derive(Clone, Debug, Serialize)]
struct EntryReport {
    id: String,
    object_id: String,
    material: String,
    impact_position: String,
    force_band: String,
    candidate: FileAnalysisReport,
    reference: Option<FileAnalysisReport>,
    matched: Option<MatchedReport>,
    failure_tags: Vec<String>,
    decision: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct FileAnalysisReport {
    manifest_path: String,
    wav_sha256: String,
    sample_format: String,
    sample_rate_hz: u32,
    channel_count: u16,
    frame_count: usize,
    duration_ms: f64,
    signal: SignalReport,
    spectrum: SpectrumReport,
    modal_peaks: Vec<ModalPeakReport>,
    decay: Vec<DecayBandReport>,
}

#[derive(Clone, Debug, Serialize)]
struct SignalReport {
    peak_dbfs: f64,
    rms_dbfs: f64,
    dc_offset: f64,
    crest_db: f64,
    clipped_sample_count: usize,
    onset_frame: Option<usize>,
    attack_ms: Option<f64>,
    temporal_centroid_ms: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct SpectrumReport {
    centroid_hz: f64,
    bandwidth_hz: f64,
    flatness_db: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ModalPeakReport {
    frequency_hz: f64,
    relative_level_db: f64,
}

#[derive(Clone, Debug, Serialize)]
struct DecayBandReport {
    band: &'static str,
    lower_hz: f64,
    upper_hz: f64,
    slope_db_per_second: Option<f64>,
    t20_ms: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct MatchedReport {
    raw_rms_delta_db: f64,
    gain_matched_multiresolution_log_spectrum_rmse_db: f64,
    modal_assignment_cost: f64,
    attack_delta_ms: Option<f64>,
    temporal_centroid_delta_ms: Option<f64>,
    spectral_centroid_delta_hz: f64,
    mean_t20_delta_ms: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct BlindBundleReport {
    directory: &'static str,
    pair_count: usize,
    blind_seed: u64,
    browser: &'static str,
    hidden_answers: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct BlindAnswer {
    pair_id: String,
    side_a_role: &'static str,
    side_b_role: &'static str,
    candidate_sha256: String,
    reference_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
struct BlindBrowserPair {
    pair_id: String,
    side_a_file: String,
    side_b_file: String,
    material: String,
    impact_position: String,
    force_band: String,
}

struct ResolvedEntry {
    manifest: ManifestEntry,
    candidate_bytes: Vec<u8>,
    candidate: Analysis,
    reference_bytes: Option<Vec<u8>>,
    reference: Option<Analysis>,
}

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path = canonical_external_file(&root, &manifest_path, "manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|error| format!("read {}: {error}", manifest_path.display()))?;
    let manifest: QualityManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "manifest has no parent directory".to_owned())?;

    let mut resolved = Vec::with_capacity(manifest.entries.len());
    for entry in manifest.entries.iter().cloned() {
        let (candidate_bytes, candidate) =
            read_analysis(&root, manifest_directory, &entry.candidate, "candidate")?;
        let (reference_bytes, reference) = if let Some(reference) = &entry.reference {
            let (bytes, analysis) =
                read_analysis(&root, manifest_directory, reference, "reference")?;
            (Some(bytes), Some(analysis))
        } else {
            (None, None)
        };
        resolved.push(ResolvedEntry {
            manifest: entry,
            candidate_bytes,
            candidate,
            reference_bytes,
            reference,
        });
    }

    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    let blind_bundle = write_blind_bundle(&output, request.blind_seed, &resolved)?;
    let matched_reference_count = resolved
        .iter()
        .filter(|entry| entry.reference.is_some())
        .count();
    let entries = resolved
        .into_iter()
        .map(build_entry_report)
        .collect::<Vec<_>>();
    let status = report_status(&entries, matched_reference_count);
    let profile = EvaluatorProfileReport::current();
    let profile_bytes = serde_json::to_vec(&profile).map_err(|error| error.to_string())?;
    let report = QualityReport {
        schema: REPORT_SCHEMA,
        status,
        claim: "OFFLINE_EXPERIMENT_ONLY / UNCALIBRATED / NO_P1_OR_SHIPPING_PROMOTION",
        split: manifest.split,
        manifest_sha256: sha256_hex(&manifest_bytes),
        evaluator_profile: profile,
        evaluator_profile_sha256: sha256_hex(&profile_bytes),
        entry_count: entries.len(),
        matched_reference_count,
        blind_bundle,
        entries,
    };
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
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
                "physical-sound-eval output must stay outside the repository: {}",
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
            "physical-sound-eval output must stay outside the repository: {}",
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
            "physical-sound-eval {role} must stay outside the repository: {}",
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
            "physical-sound-eval output directory must be empty: {}",
            output.display()
        ));
    }
    Ok(())
}

fn validate_manifest(manifest: &QualityManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound quality manifest schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.split, "split")?;
    if manifest.entries.is_empty() || manifest.entries.len() > MAX_ENTRIES {
        return Err(format!(
            "manifest entry count must be 1..={MAX_ENTRIES}, got {}",
            manifest.entries.len()
        ));
    }
    let mut previous = None;
    for entry in &manifest.entries {
        validate_label(&entry.id, "entry id")?;
        validate_label(&entry.object_id, "object_id")?;
        validate_label(&entry.material, "material")?;
        validate_label(&entry.impact_position, "impact_position")?;
        validate_label(&entry.force_band, "force_band")?;
        if previous.is_some_and(|value: &String| value >= &entry.id) {
            return Err(format!(
                "manifest entries must be strictly sorted by id; offending id {}",
                entry.id
            ));
        }
        previous = Some(&entry.id);
        validate_audio_ref(&entry.candidate, "candidate")?;
        if let Some(reference) = &entry.reference {
            validate_audio_ref(reference, "reference")?;
        }
    }
    Ok(())
}

fn validate_label(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(format!(
            "{field} must be 1..=96 ASCII [A-Za-z0-9._-] characters: {value:?}"
        ));
    }
    Ok(())
}

fn validate_audio_ref(value: &AudioFileRef, role: &str) -> Result<(), String> {
    if value.path.is_empty() || value.path.len() > 4_096 {
        return Err(format!("{role} path length is invalid"));
    }
    if value.sha256.len() != 64
        || !value
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{role} sha256 must be 64 lowercase hex digits"));
    }
    Ok(())
}

fn read_analysis(
    root: &Path,
    manifest_directory: &Path,
    file_ref: &AudioFileRef,
    role: &str,
) -> Result<(Vec<u8>, Analysis), String> {
    let unresolved = PathBuf::from(&file_ref.path);
    let unresolved = if unresolved.is_absolute() {
        unresolved
    } else {
        manifest_directory.join(unresolved)
    };
    let path = canonical_external_file(root, &unresolved, role)?;
    let metadata =
        fs::metadata(&path).map_err(|error| format!("stat {role} {}: {error}", path.display()))?;
    let byte_len = usize::try_from(metadata.len())
        .map_err(|_| format!("{role} is too large: {}", path.display()))?;
    if byte_len > MAX_WAV_BYTES {
        return Err(format!(
            "{role} exceeds {MAX_WAV_BYTES} bytes: {}",
            path.display()
        ));
    }
    let bytes =
        fs::read(&path).map_err(|error| format!("read {role} {}: {error}", path.display()))?;
    let actual_hash = sha256_hex(&bytes);
    if actual_hash != file_ref.sha256 {
        return Err(format!(
            "{role} hash mismatch for {}: expected {}, got {actual_hash}",
            path.display(),
            file_ref.sha256
        ));
    }
    let wav = parse_wav(&bytes).map_err(|error| format!("parse {role} WAV: {error}"))?;
    let analysis = analyze_wav(&file_ref.path, &actual_hash, wav)?;
    Ok((bytes, analysis))
}

fn build_entry_report(entry: ResolvedEntry) -> EntryReport {
    let mut failure_tags = signal_failure_tags(&entry.candidate.report);
    if let Some(reference) = &entry.reference {
        for tag in signal_failure_tags(&reference.report) {
            push_unique(&mut failure_tags, &format!("REFERENCE_{tag}"));
        }
    }
    let matched = entry
        .reference
        .as_ref()
        .map(|reference| matched_report(&entry.candidate, reference));
    if let (Some(reference), Some(comparison)) = (&entry.reference, &matched) {
        add_matched_failure_tags(
            &entry.candidate.report,
            &reference.report,
            comparison,
            &mut failure_tags,
        );
    }
    let decision = if failure_tags.iter().any(|tag| is_hard_signal_failure(tag)) {
        "Reject"
    } else if entry.reference.is_none() {
        "NeedsReference"
    } else {
        "NeedsHumanAudit"
    };
    EntryReport {
        id: entry.manifest.id,
        object_id: entry.manifest.object_id,
        material: entry.manifest.material,
        impact_position: entry.manifest.impact_position,
        force_band: entry.manifest.force_band,
        candidate: entry.candidate.report,
        reference: entry.reference.map(|analysis| analysis.report),
        matched,
        failure_tags,
        decision,
    }
}

fn signal_failure_tags(report: &FileAnalysisReport) -> Vec<String> {
    let mut tags = Vec::new();
    if report.signal.peak_dbfs <= -100.0 {
        tags.push("SILENCE".to_owned());
    }
    if report.signal.clipped_sample_count * 100 > report.frame_count {
        tags.push("CLIPPING".to_owned());
    }
    if report.signal.dc_offset.abs() > 0.02 {
        tags.push("EXCESSIVE_DC".to_owned());
    }
    if report.duration_ms < 50.0 {
        tags.push("TOO_SHORT".to_owned());
    }
    if report.signal.onset_frame.is_none() {
        tags.push("ONSET_MISSING".to_owned());
    }
    if report.spectrum.flatness_db < -48.0 && report.modal_peaks.len() <= 3 {
        tags.push("EXCESSIVE_NARROW_RINGING".to_owned());
    }
    if report
        .decay
        .iter()
        .filter_map(|band| band.t20_ms)
        .any(|t20_ms| t20_ms > report.duration_ms)
    {
        tags.push("DECAY_WINDOW_CENSORED".to_owned());
    }
    tags
}

fn is_hard_signal_failure(tag: &str) -> bool {
    let tag = tag.strip_prefix("REFERENCE_").unwrap_or(tag);
    matches!(
        tag,
        "SILENCE" | "CLIPPING" | "EXCESSIVE_DC" | "TOO_SHORT" | "ONSET_MISSING"
    )
}

fn matched_report(candidate: &Analysis, reference: &Analysis) -> MatchedReport {
    let spectral_squared = candidate
        .log_spectra
        .iter()
        .zip(&reference.log_spectra)
        .flat_map(|(candidate, reference)| candidate.iter().zip(reference))
        .map(|(candidate, reference)| (candidate - reference).powi(2))
        .sum::<f64>();
    let spectral_count = candidate.log_spectra.len() * LOG_SPECTRUM_BINS;
    MatchedReport {
        raw_rms_delta_db: candidate.report.signal.rms_dbfs - reference.report.signal.rms_dbfs,
        gain_matched_multiresolution_log_spectrum_rmse_db: (spectral_squared
            / spectral_count as f64)
            .sqrt(),
        modal_assignment_cost: modal_assignment_cost(
            &candidate.report.modal_peaks,
            &reference.report.modal_peaks,
        ),
        attack_delta_ms: optional_delta(
            candidate.report.signal.attack_ms,
            reference.report.signal.attack_ms,
        ),
        temporal_centroid_delta_ms: optional_delta(
            candidate.report.signal.temporal_centroid_ms,
            reference.report.signal.temporal_centroid_ms,
        ),
        spectral_centroid_delta_hz: candidate.report.spectrum.centroid_hz
            - reference.report.spectrum.centroid_hz,
        mean_t20_delta_ms: mean_decay_delta(&candidate.report.decay, &reference.report.decay),
    }
}

fn optional_delta(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    left.zip(right).map(|(left, right)| left - right)
}

fn mean_decay_delta(left: &[DecayBandReport], right: &[DecayBandReport]) -> Option<f64> {
    let values = left
        .iter()
        .zip(right)
        .filter_map(|(left, right)| optional_delta(left.t20_ms, right.t20_ms))
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn modal_assignment_cost(left: &[ModalPeakReport], right: &[ModalPeakReport]) -> f64 {
    let size = left.len().max(right.len());
    if size == 0 {
        return 1.0;
    }
    let mut costs = vec![vec![1.0_f64; size]; size];
    for (left_index, left_peak) in left.iter().enumerate() {
        for (right_index, right_peak) in right.iter().enumerate() {
            let octave_distance = (left_peak.frequency_hz / right_peak.frequency_hz)
                .log2()
                .abs();
            let level_distance =
                (left_peak.relative_level_db - right_peak.relative_level_db).abs() / 40.0;
            costs[left_index][right_index] = octave_distance + 0.25 * level_distance;
        }
    }
    hungarian_minimum(&costs) / size as f64
}

fn hungarian_minimum(costs: &[Vec<f64>]) -> f64 {
    let size = costs.len();
    let mut row_potential = vec![0.0_f64; size + 1];
    let mut column_potential = vec![0.0_f64; size + 1];
    let mut matching = vec![0_usize; size + 1];
    let mut predecessor = vec![0_usize; size + 1];
    for row in 1..=size {
        matching[0] = row;
        let mut column = 0_usize;
        let mut minimum = vec![f64::INFINITY; size + 1];
        let mut used = vec![false; size + 1];
        loop {
            used[column] = true;
            let current_row = matching[column];
            let mut delta = f64::INFINITY;
            let mut next_column = 0_usize;
            for candidate_column in 1..=size {
                if used[candidate_column] {
                    continue;
                }
                let reduced = costs[current_row - 1][candidate_column - 1]
                    - row_potential[current_row]
                    - column_potential[candidate_column];
                if reduced < minimum[candidate_column] {
                    minimum[candidate_column] = reduced;
                    predecessor[candidate_column] = column;
                }
                if minimum[candidate_column] < delta {
                    delta = minimum[candidate_column];
                    next_column = candidate_column;
                }
            }
            for candidate_column in 0..=size {
                if used[candidate_column] {
                    row_potential[matching[candidate_column]] += delta;
                    column_potential[candidate_column] -= delta;
                } else {
                    minimum[candidate_column] -= delta;
                }
            }
            column = next_column;
            if matching[column] == 0 {
                break;
            }
        }
        loop {
            let previous = predecessor[column];
            matching[column] = matching[previous];
            column = previous;
            if column == 0 {
                break;
            }
        }
    }
    (-column_potential[0]).max(0.0)
}

fn add_matched_failure_tags(
    candidate: &FileAnalysisReport,
    reference: &FileAnalysisReport,
    comparison: &MatchedReport,
    tags: &mut Vec<String>,
) {
    if comparison.gain_matched_multiresolution_log_spectrum_rmse_db > 12.0 {
        push_unique(tags, "SPECTRAL_MISMATCH_UNCALIBRATED");
    }
    if comparison.modal_assignment_cost > 0.6 {
        push_unique(tags, "MODAL_STRUCTURE_MISMATCH_UNCALIBRATED");
    }
    if comparison
        .attack_delta_ms
        .is_some_and(|delta| delta.abs() > 15.0)
    {
        push_unique(tags, "ATTACK_MISMATCH_UNCALIBRATED");
    }
    let candidate_high = candidate
        .decay
        .iter()
        .find(|band| band.band == "high")
        .and_then(|band| band.t20_ms);
    let reference_high = reference
        .decay
        .iter()
        .find(|band| band.band == "high")
        .and_then(|band| band.t20_ms);
    if candidate_high
        .zip(reference_high)
        .is_some_and(|(candidate, reference)| candidate > reference * 1.4 + 10.0)
    {
        push_unique(tags, "DECAY_TOO_LONG_HIGH_BANDS");
    }
}

fn push_unique(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|existing| existing == tag) {
        tags.push(tag.to_owned());
    }
}

fn report_status(entries: &[EntryReport], matched_reference_count: usize) -> String {
    if entries.iter().any(|entry| entry.decision == "Reject") {
        "ANALYZED_WITH_SIGNAL_REJECTS".to_owned()
    } else if matched_reference_count == 0 {
        "Q0_BASELINE_FROZEN / REFERENCES_REQUIRED".to_owned()
    } else {
        "Q1_MATCHED_ANALYSIS / HUMAN_CALIBRATION_REQUIRED".to_owned()
    }
}

fn write_blind_bundle(
    output: &Path,
    blind_seed: u64,
    entries: &[ResolvedEntry],
) -> Result<Option<BlindBundleReport>, String> {
    let matched = entries
        .iter()
        .filter(|entry| entry.reference.is_some())
        .collect::<Vec<_>>();
    if matched.is_empty() {
        return Ok(None);
    }
    let blind_directory = output.join("blind");
    fs::create_dir_all(&blind_directory)
        .map_err(|error| format!("create {}: {error}", blind_directory.display()))?;
    let mut answers = Vec::with_capacity(matched.len());
    let mut browser_pairs = Vec::with_capacity(matched.len());
    for (index, entry) in matched.into_iter().enumerate() {
        let pair_number = index + 1;
        let side_a_file = format!("pair-{pair_number:03}-A.wav");
        let side_b_file = format!("pair-{pair_number:03}-B.wav");
        let candidate_is_a = blind_candidate_is_a(blind_seed, &entry.manifest.id);
        let reference_bytes = entry
            .reference_bytes
            .as_ref()
            .ok_or_else(|| format!("matched entry {} has no reference bytes", entry.manifest.id))?;
        let (side_a, side_b, side_a_role, side_b_role) = if candidate_is_a {
            (
                &entry.candidate_bytes,
                reference_bytes,
                "candidate",
                "reference",
            )
        } else {
            (
                reference_bytes,
                &entry.candidate_bytes,
                "reference",
                "candidate",
            )
        };
        fs::write(blind_directory.join(&side_a_file), side_a)
            .map_err(|error| format!("write {side_a_file}: {error}"))?;
        fs::write(blind_directory.join(&side_b_file), side_b)
            .map_err(|error| format!("write {side_b_file}: {error}"))?;
        answers.push(BlindAnswer {
            pair_id: entry.manifest.id.clone(),
            side_a_role,
            side_b_role,
            candidate_sha256: entry.manifest.candidate.sha256.clone(),
            reference_sha256: entry
                .manifest
                .reference
                .as_ref()
                .ok_or_else(|| {
                    format!(
                        "matched entry {} has no reference manifest",
                        entry.manifest.id
                    )
                })?
                .sha256
                .clone(),
        });
        browser_pairs.push(BlindBrowserPair {
            pair_id: entry.manifest.id.clone(),
            side_a_file,
            side_b_file,
            material: entry.manifest.material.clone(),
            impact_position: entry.manifest.impact_position.clone(),
            force_band: entry.manifest.force_band.clone(),
        });
    }
    let answers_json = serde_json::to_vec_pretty(&answers).map_err(|error| error.to_string())?;
    fs::write(
        blind_directory.join("DO_NOT_OPEN_UNTIL_DONE-answers.json"),
        answers_json,
    )
    .map_err(|error| format!("write blind answers: {error}"))?;
    let pair_json = serde_json::to_string(&browser_pairs).map_err(|error| error.to_string())?;
    let html = BLIND_BROWSER_HTML.replace("__PAIR_JSON__", &pair_json);
    fs::write(blind_directory.join("index.html"), html)
        .map_err(|error| format!("write blind index.html: {error}"))?;
    Ok(Some(BlindBundleReport {
        directory: "blind",
        pair_count: browser_pairs.len(),
        blind_seed,
        browser: "blind/index.html",
        hidden_answers: "blind/DO_NOT_OPEN_UNTIL_DONE-answers.json",
    }))
}

fn blind_candidate_is_a(seed: u64, pair_id: &str) -> bool {
    let mut bytes = seed.to_le_bytes().to_vec();
    bytes.extend_from_slice(pair_id.as_bytes());
    sha256(&bytes)[0] & 1 == 0
}

fn sha256_hex(bytes: &[u8]) -> String {
    ContentHash::from_bytes(sha256(bytes)).to_hex()
}

const BLIND_BROWSER_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Physical sound blind comparison</title>
<style>
body { font: 16px system-ui, sans-serif; max-width: 900px; margin: 2rem auto; padding: 0 1rem; background: #111; color: #eee; }
.pair { border: 1px solid #444; border-radius: 12px; padding: 1rem; margin: 1rem 0; }
.sides { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
audio { width: 100%; }
label { display: block; margin: .5rem 0; }
button { padding: .7rem 1rem; font-size: 1rem; }
small { color: #aaa; }
</style>
</head>
<body>
<h1>Physical sound blind comparison</h1>
<p>Use the same headphones and level. Judge realism first; loudness is intentionally not hidden. Do not open the answers file until results are saved.</p>
<div id="pairs"></div>
<button id="download">Download ratings JSON</button>
<script>
const pairs = __PAIR_JSON__;
const root = document.getElementById('pairs');
for (const [index, pair] of pairs.entries()) {
  const section = document.createElement('section');
  section.className = 'pair';
  section.dataset.pair = pair.pair_id;
  section.innerHTML = `<h2>Pair ${index + 1}</h2><small>Material ${pair.material}; position ${pair.impact_position}; force ${pair.force_band}</small>
    <div class="sides"><div><h3>A</h3><audio controls preload="metadata" src="${pair.side_a_file}"></audio>
    <label>A realism (1–5) <input name="a_realism" type="number" min="1" max="5"></label></div>
    <div><h3>B</h3><audio controls preload="metadata" src="${pair.side_b_file}"></audio>
    <label>B realism (1–5) <input name="b_realism" type="number" min="1" max="5"></label></div></div>
    <label>Preference <select name="preference"><option value="">Choose</option><option>A</option><option>B</option><option>NoPreference</option></select></label>
    <label>Material plausibility winner <select name="material"><option value="">Choose</option><option>A</option><option>B</option><option>Equal</option></select></label>
    <label>Force plausibility winner <select name="force"><option value="">Choose</option><option>A</option><option>B</option><option>Equal</option></select></label>
    <label>Artifacts / notes <input name="notes" maxlength="500"></label>`;
  root.appendChild(section);
}
document.getElementById('download').addEventListener('click', () => {
  const ratings = [...document.querySelectorAll('.pair')].map(section => ({
    pair_id: section.dataset.pair,
    a_realism: section.querySelector('[name=a_realism]').value || null,
    b_realism: section.querySelector('[name=b_realism]').value || null,
    preference: section.querySelector('[name=preference]').value || null,
    material_winner: section.querySelector('[name=material]').value || null,
    force_winner: section.querySelector('[name=force]').value || null,
    notes: section.querySelector('[name=notes]').value
  }));
  const blob = new Blob([JSON.stringify({schema: 'nextengine.experimental-physical-sound-preferences.v0', ratings}, null, 2)], {type: 'application/json'});
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = 'physical-sound-preferences.json';
  link.click();
  URL.revokeObjectURL(link.href);
});
</script>
</body>
</html>
"#;
