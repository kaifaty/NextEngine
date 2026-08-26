use std::fs;
use std::path::{Component, Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};
use next_presentation::physical_sound_lab::{
    OfflineModalMode, cook_offline_q30_modal_bank, normalize_offline_q30_samples,
    render_offline_modal_recurrence, render_offline_q30_modal_recurrence,
};
use serde::{Deserialize, Serialize};

use crate::physical_sound_eval_command::audio_analysis::{WavAudio, parse_wav};

mod transfer_math;

use transfer_math::{
    calculate_residual, calculate_residual_f32, concatenate_audition_pair,
    normalized_mono_to_stereo_s16, resample_zero_extended_linear, scale_sample_count,
};

const PROFILE_SCHEMA: &str = "nextengine.external-diffsound-checkpoint.v0";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-reproduction.report.v1";
const QUALITY_MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-quality.manifest.v0";
const MAX_PROFILE_BYTES: u64 = 1024 * 1024;
const MAX_MODE_COUNT: usize = 128;
const MAX_TRANSIENT_SAMPLES: usize = 8_192;
const RMS_RESIDUAL_LIMIT: f64 = 1.0e-3;
const CORRELATION_MINIMUM: f64 = 0.999;
const Q30_SAMPLE_RATE_HZ: u32 = 48_000;
const Q30_RMS_RESIDUAL_LIMIT: f64 = 1.0e-4;
const Q30_MAXIMUM_ABSOLUTE_RESIDUAL_LIMIT: f64 = 1.0e-3;
const Q30_CORRELATION_MINIMUM: f64 = 0.999_99;
const AUDITION_SILENCE_MILLISECONDS: u32 = 250;

pub(super) struct Request {
    profile: PathBuf,
    output: PathBuf,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let mut profile = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--profile" => set_once(&mut profile, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(Request {
        profile: profile.ok_or_else(|| {
            "physical-sound-reproduce requires --profile <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-reproduce requires --output <external-empty-directory>".to_owned()
        })?,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ExternalProfile {
    schema: String,
    upstream_commit: String,
    step: u32,
    transient_samples: usize,
    transient_values: Vec<f64>,
    modes: Vec<ExternalMode>,
    audio: ExternalAudio,
}

#[derive(Debug, Deserialize)]
struct ExternalMode {
    index: usize,
    undamped_frequency_hz: f64,
    damped_frequency_hz: f64,
    damping_per_second: f64,
    t60_seconds: f64,
    runtime_decay_pole: f64,
    ridge_amplitude: f64,
}

#[derive(Debug, Deserialize)]
struct ExternalAudio {
    ridge_amplitude: ExternalAudioFile,
    ridge_amplitude_transient: ExternalAudioFile,
}

#[derive(Debug, Deserialize)]
struct ExternalAudioFile {
    file: String,
    sha256: String,
}

#[derive(Serialize)]
struct ReproductionReport {
    schema: &'static str,
    status: &'static str,
    claim: &'static str,
    fallback: &'static str,
    source_profile: String,
    source_profile_sha256: String,
    source_schema: String,
    source_upstream_commit: String,
    source_step: u32,
    renderer: &'static str,
    sample_rate_hz: u32,
    frame_count: usize,
    mode_count: usize,
    transient_sample_count: usize,
    repeated_render_identical: bool,
    acceptance: AcceptanceReport,
    modal_only: AudioReproductionReport,
    modal_with_transient: AudioReproductionReport,
    fixed_point_48khz: FixedPointTransferReport,
    quality_manifest_file: &'static str,
}

#[derive(Serialize)]
struct AcceptanceReport {
    rms_residual_at_most: f64,
    correlation_at_least: f64,
}

#[derive(Serialize)]
struct AudioReproductionReport {
    reference_file: String,
    reference_sha256: String,
    output_file: &'static str,
    output_sha256: String,
    exact_sample_count: usize,
    residual: ResidualReport,
    accepted: bool,
}

#[derive(Serialize)]
struct ResidualReport {
    maximum_absolute: f64,
    rms: f64,
    signal_to_noise_db: Option<f64>,
    correlation: f64,
}

#[derive(Serialize)]
struct FixedPointTransferReport {
    status: &'static str,
    claim: &'static str,
    renderer: &'static str,
    q_format: &'static str,
    sample_rate_hz: u32,
    frame_count: usize,
    mode_count: usize,
    source_transient_sample_count: usize,
    resampled_transient_sample_count: usize,
    repeated_render_identical: bool,
    acceptance: FixedPointAcceptanceReport,
    modal_only: FixedPointAudioReport,
    modal_with_transient: FixedPointAudioReport,
    audition_a_file: &'static str,
    audition_a_sha256: String,
    audition_b_file: &'static str,
    audition_b_sha256: String,
    audition_ab_file: &'static str,
    audition_ab_sha256: String,
}

#[derive(Serialize)]
struct FixedPointAcceptanceReport {
    maximum_absolute_residual_at_most: f64,
    rms_residual_at_most: f64,
    correlation_at_least: f64,
}

#[derive(Serialize)]
struct FixedPointAudioReport {
    reference_file: &'static str,
    reference_sha256: String,
    output_file: &'static str,
    output_sha256: String,
    residual: ResidualReport,
    accepted: bool,
}

#[derive(Serialize)]
struct QualityManifest {
    schema: &'static str,
    split: &'static str,
    entries: Vec<QualityManifestEntry>,
}

#[derive(Serialize)]
struct QualityManifestEntry {
    id: String,
    object_id: &'static str,
    material: &'static str,
    impact_position: &'static str,
    force_band: &'static str,
    candidate: QualityAudioRef,
    reference: Option<QualityAudioRef>,
}

#[derive(Serialize)]
struct QualityAudioRef {
    path: String,
    sha256: String,
}

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let profile_path = canonical_external_input(root, &request.profile, "modal profile")?;
    let profile_metadata = fs::metadata(&profile_path)
        .map_err(|error| format!("stat {}: {error}", profile_path.display()))?;
    if profile_metadata.len() == 0 || profile_metadata.len() > MAX_PROFILE_BYTES {
        return Err(format!(
            "modal profile must be 1..={MAX_PROFILE_BYTES} bytes: {}",
            profile_path.display()
        ));
    }
    let profile_bytes = fs::read(&profile_path)
        .map_err(|error| format!("read {}: {error}", profile_path.display()))?;
    let profile: ExternalProfile = serde_json::from_slice(&profile_bytes)
        .map_err(|error| format!("parse {}: {error}", profile_path.display()))?;
    validate_profile(&profile)?;

    let profile_parent = profile_path
        .parent()
        .ok_or_else(|| "modal profile has no parent directory".to_owned())?;
    let modal_reference_path =
        canonical_sibling(profile_parent, &profile.audio.ridge_amplitude.file)?;
    let transient_reference_path = canonical_sibling(
        profile_parent,
        &profile.audio.ridge_amplitude_transient.file,
    )?;
    let modal_reference = load_reference(
        &modal_reference_path,
        &profile.audio.ridge_amplitude.sha256,
        "ridge-amplitude reference",
    )?;
    let transient_reference = load_reference(
        &transient_reference_path,
        &profile.audio.ridge_amplitude_transient.sha256,
        "ridge-amplitude-transient reference",
    )?;
    validate_reference_pair(&modal_reference, &transient_reference)?;
    validate_profile_for_sample_rate(&profile, modal_reference.audio.sample_rate_hz)?;

    let modes = profile
        .modes
        .iter()
        .map(|mode| OfflineModalMode {
            damped_frequency_hz: mode.damped_frequency_hz,
            damping_per_second: mode.damping_per_second,
            amplitude: mode.ridge_amplitude,
        })
        .collect::<Vec<_>>();
    let frame_count = modal_reference.audio.mono_samples.len();
    let sample_rate_hz = modal_reference.audio.sample_rate_hz;
    let modal_samples = render_offline_modal_recurrence(sample_rate_hz, frame_count, &modes, &[])
        .map_err(|error| error.to_string())?;
    let transient_samples = render_offline_modal_recurrence(
        sample_rate_hz,
        frame_count,
        &modes,
        &profile.transient_values,
    )
    .map_err(|error| error.to_string())?;
    let repeated_render_identical = transient_samples
        == render_offline_modal_recurrence(
            sample_rate_hz,
            frame_count,
            &modes,
            &profile.transient_values,
        )
        .map_err(|error| error.to_string())?;

    let q30_frame_count = scale_sample_count(frame_count, sample_rate_hz, Q30_SAMPLE_RATE_HZ)?;
    let resampled_transient = resample_zero_extended_linear(
        &profile.transient_values,
        sample_rate_hz,
        Q30_SAMPLE_RATE_HZ,
    )?;
    let q30_modal_reference =
        render_offline_modal_recurrence(Q30_SAMPLE_RATE_HZ, q30_frame_count, &modes, &[])
            .map_err(|error| error.to_string())?;
    let q30_transient_reference = render_offline_modal_recurrence(
        Q30_SAMPLE_RATE_HZ,
        q30_frame_count,
        &modes,
        &resampled_transient,
    )
    .map_err(|error| error.to_string())?;
    let q30_modal_bank = cook_offline_q30_modal_bank(Q30_SAMPLE_RATE_HZ, &modes, &[])
        .map_err(|error| error.to_string())?;
    let q30_transient_bank =
        cook_offline_q30_modal_bank(Q30_SAMPLE_RATE_HZ, &modes, &resampled_transient)
            .map_err(|error| error.to_string())?;
    let q30_modal_raw = render_offline_q30_modal_recurrence(q30_frame_count, &q30_modal_bank)
        .map_err(|error| error.to_string())?;
    let q30_transient_raw =
        render_offline_q30_modal_recurrence(q30_frame_count, &q30_transient_bank)
            .map_err(|error| error.to_string())?;
    let q30_modal_samples =
        normalize_offline_q30_samples(&q30_modal_raw).map_err(|error| error.to_string())?;
    let q30_transient_samples =
        normalize_offline_q30_samples(&q30_transient_raw).map_err(|error| error.to_string())?;
    let repeated_q30_render_identical = q30_transient_raw
        == render_offline_q30_modal_recurrence(q30_frame_count, &q30_transient_bank)
            .map_err(|error| error.to_string())?;

    let output = resolve_external_output(root, &request.output)?;
    require_empty_output(&output)?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    let modal_output_file = "engine-ridge-amplitude.wav";
    let transient_output_file = "engine-ridge-amplitude-transient.wav";
    let q30_modal_reference_file = "reference-48khz-ridge-amplitude.wav";
    let q30_transient_reference_file = "reference-48khz-ridge-amplitude-transient.wav";
    let q30_modal_output_file = "engine-q30-48khz-ridge-amplitude.wav";
    let q30_transient_output_file = "engine-q30-48khz-ridge-amplitude-transient.wav";
    let audition_a_file = "audition-a-high-precision.wav";
    let audition_b_file = "audition-b-q30.wav";
    let audition_ab_file = "audition-ab-high-precision-then-q30.wav";
    let modal_wav = encode_float32_mono_wav(sample_rate_hz, &modal_samples)?;
    let transient_wav = encode_float32_mono_wav(sample_rate_hz, &transient_samples)?;
    let q30_modal_reference_wav =
        encode_float32_mono_wav(Q30_SAMPLE_RATE_HZ, &q30_modal_reference)?;
    let q30_transient_reference_wav =
        encode_float32_mono_wav(Q30_SAMPLE_RATE_HZ, &q30_transient_reference)?;
    let q30_modal_wav = encode_float32_mono_wav(Q30_SAMPLE_RATE_HZ, &q30_modal_samples)?;
    let q30_transient_wav = encode_float32_mono_wav(Q30_SAMPLE_RATE_HZ, &q30_transient_samples)?;
    let audio_profile =
        AudioMixProfileV1::stereo_baseline_v1().map_err(|error| error.to_string())?;
    let audition_a = normalized_mono_to_stereo_s16(&q30_transient_reference)?;
    let audition_b = normalized_mono_to_stereo_s16(&q30_transient_samples)?;
    let audition_ab = concatenate_audition_pair(
        &audition_a,
        &audition_b,
        Q30_SAMPLE_RATE_HZ,
        AUDITION_SILENCE_MILLISECONDS,
    )?;
    let audition_a_wav = encode_canonical_wav(&audio_profile, &audition_a);
    let audition_b_wav = encode_canonical_wav(&audio_profile, &audition_b);
    let audition_ab_wav = encode_canonical_wav(&audio_profile, &audition_ab);
    fs::write(output.join(modal_output_file), &modal_wav)
        .map_err(|error| format!("write {modal_output_file}: {error}"))?;
    fs::write(output.join(transient_output_file), &transient_wav)
        .map_err(|error| format!("write {transient_output_file}: {error}"))?;
    for (file, bytes) in [
        (q30_modal_reference_file, &q30_modal_reference_wav),
        (q30_transient_reference_file, &q30_transient_reference_wav),
        (q30_modal_output_file, &q30_modal_wav),
        (q30_transient_output_file, &q30_transient_wav),
        (audition_a_file, &audition_a_wav),
        (audition_b_file, &audition_b_wav),
        (audition_ab_file, &audition_ab_wav),
    ] {
        fs::write(output.join(file), bytes).map_err(|error| format!("write {file}: {error}"))?;
    }

    let modal_output_sha256 = sha256_hex(&modal_wav);
    let transient_output_sha256 = sha256_hex(&transient_wav);
    let q30_modal_reference_sha256 = sha256_hex(&q30_modal_reference_wav);
    let q30_transient_reference_sha256 = sha256_hex(&q30_transient_reference_wav);
    let q30_modal_output_sha256 = sha256_hex(&q30_modal_wav);
    let q30_transient_output_sha256 = sha256_hex(&q30_transient_wav);
    let audition_a_sha256 = sha256_hex(&audition_a_wav);
    let audition_b_sha256 = sha256_hex(&audition_b_wav);
    let audition_ab_sha256 = sha256_hex(&audition_ab_wav);
    let modal_report = compare_render(
        &modal_reference,
        &modal_samples,
        modal_output_file,
        modal_output_sha256.clone(),
    )?;
    let transient_report = compare_render(
        &transient_reference,
        &transient_samples,
        transient_output_file,
        transient_output_sha256.clone(),
    )?;
    let q30_modal_report = compare_fixed_point_render(
        &q30_modal_reference,
        &q30_modal_samples,
        q30_modal_reference_file,
        q30_modal_reference_sha256.clone(),
        q30_modal_output_file,
        q30_modal_output_sha256.clone(),
    )?;
    let q30_transient_report = compare_fixed_point_render(
        &q30_transient_reference,
        &q30_transient_samples,
        q30_transient_reference_file,
        q30_transient_reference_sha256.clone(),
        q30_transient_output_file,
        q30_transient_output_sha256.clone(),
    )?;
    let q30_status = if repeated_q30_render_identical
        && q30_modal_report.accepted
        && q30_transient_report.accepted
    {
        "PASS"
    } else {
        "RESIDUAL_TOO_LARGE"
    };
    let status = if repeated_render_identical
        && modal_report.accepted
        && transient_report.accepted
        && q30_status == "PASS"
    {
        "PASS"
    } else {
        "RESIDUAL_TOO_LARGE"
    };

    let mut manifest_entries = vec![
        QualityManifestEntry {
            id: format!("step-{:04}-ridge", profile.step),
            object_id: "external-diffsound-glass-proxy",
            material: "glass",
            impact_position: "proxy",
            force_band: "normalized",
            candidate: QualityAudioRef {
                path: output.join(modal_output_file).display().to_string(),
                sha256: modal_output_sha256,
            },
            reference: Some(QualityAudioRef {
                path: modal_reference.path.display().to_string(),
                sha256: modal_reference.sha256.clone(),
            }),
        },
        QualityManifestEntry {
            id: format!("step-{:04}-ridge-transient", profile.step),
            object_id: "external-diffsound-glass-proxy",
            material: "glass",
            impact_position: "proxy",
            force_band: "normalized",
            candidate: QualityAudioRef {
                path: output.join(transient_output_file).display().to_string(),
                sha256: transient_output_sha256,
            },
            reference: Some(QualityAudioRef {
                path: transient_reference.path.display().to_string(),
                sha256: transient_reference.sha256.clone(),
            }),
        },
        QualityManifestEntry {
            id: format!("step-{:04}-q30-48khz-ridge", profile.step),
            object_id: "external-diffsound-glass-proxy",
            material: "glass",
            impact_position: "proxy",
            force_band: "normalized",
            candidate: QualityAudioRef {
                path: output.join(q30_modal_output_file).display().to_string(),
                sha256: q30_modal_output_sha256.clone(),
            },
            reference: Some(QualityAudioRef {
                path: output.join(q30_modal_reference_file).display().to_string(),
                sha256: q30_modal_reference_sha256.clone(),
            }),
        },
        QualityManifestEntry {
            id: format!("step-{:04}-q30-48khz-ridge-transient", profile.step),
            object_id: "external-diffsound-glass-proxy",
            material: "glass",
            impact_position: "proxy",
            force_band: "normalized",
            candidate: QualityAudioRef {
                path: output.join(q30_transient_output_file).display().to_string(),
                sha256: q30_transient_output_sha256.clone(),
            },
            reference: Some(QualityAudioRef {
                path: output
                    .join(q30_transient_reference_file)
                    .display()
                    .to_string(),
                sha256: q30_transient_reference_sha256.clone(),
            }),
        },
    ];
    manifest_entries.sort_by(|left, right| left.id.cmp(&right.id));
    let manifest = QualityManifest {
        schema: QUALITY_MANIFEST_SCHEMA,
        split: "q1-diffsound-recurrence-reproduction",
        entries: manifest_entries,
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    fs::write(output.join("quality-manifest.json"), manifest_json)
        .map_err(|error| format!("write quality-manifest.json: {error}"))?;

    let report = ReproductionReport {
        schema: REPORT_SCHEMA,
        status,
        claim: "OFFLINE_PRESENTATION_EXPERIMENT_ONLY / NO_P1_OR_SHIPPING_PROMOTION",
        fallback: "ordinary authored clip path remains mandatory",
        source_profile: profile_path.display().to_string(),
        source_profile_sha256: sha256_hex(&profile_bytes),
        source_schema: profile.schema,
        source_upstream_commit: profile.upstream_commit,
        source_step: profile.step,
        renderer: "engine-owned f64 second-order damped modal recurrence; peak-normalized mono float32",
        sample_rate_hz,
        frame_count,
        mode_count: modes.len(),
        transient_sample_count: profile.transient_values.len(),
        repeated_render_identical,
        acceptance: AcceptanceReport {
            rms_residual_at_most: RMS_RESIDUAL_LIMIT,
            correlation_at_least: CORRELATION_MINIMUM,
        },
        modal_only: modal_report,
        modal_with_transient: transient_report,
        fixed_point_48khz: FixedPointTransferReport {
            status: q30_status,
            claim: "ISOLATED_FIXED_POINT_TRANSFER_ONLY / CURRENT_DEMO_GLASS_UNCHANGED",
            renderer: "engine-owned integer second-order recurrence with checked i128 multiply-accumulate; final audition normalization only uses float",
            q_format: "signed Q30 coefficients, per-mode state and resampled transient",
            sample_rate_hz: Q30_SAMPLE_RATE_HZ,
            frame_count: q30_frame_count,
            mode_count: q30_transient_bank.mode_count(),
            source_transient_sample_count: profile.transient_values.len(),
            resampled_transient_sample_count: q30_transient_bank.transient_sample_count(),
            repeated_render_identical: repeated_q30_render_identical,
            acceptance: FixedPointAcceptanceReport {
                maximum_absolute_residual_at_most: Q30_MAXIMUM_ABSOLUTE_RESIDUAL_LIMIT,
                rms_residual_at_most: Q30_RMS_RESIDUAL_LIMIT,
                correlation_at_least: Q30_CORRELATION_MINIMUM,
            },
            modal_only: q30_modal_report,
            modal_with_transient: q30_transient_report,
            audition_a_file,
            audition_a_sha256,
            audition_b_file,
            audition_b_sha256,
            audition_ab_file,
            audition_ab_sha256,
        },
        quality_manifest_file: "quality-manifest.json",
    };
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    if status != "PASS" {
        return Err(
            "physical-sound-reproduce high-precision or Q30 residual exceeded its bounds"
                .to_owned(),
        );
    }
    Ok(())
}

struct LoadedReference {
    path: PathBuf,
    sha256: String,
    audio: WavAudio,
}

fn load_reference(path: &Path, expected_hash: &str, role: &str) -> Result<LoadedReference, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("read {role} {}: {error}", path.display()))?;
    let actual_hash = sha256_hex(&bytes);
    if actual_hash != expected_hash {
        return Err(format!(
            "{role} hash mismatch for {}: expected {expected_hash}, got {actual_hash}",
            path.display()
        ));
    }
    let audio = parse_wav(&bytes).map_err(|error| format!("parse {role}: {error}"))?;
    if audio.sample_format != "ieee-f32" || audio.channel_count != 1 {
        return Err(format!(
            "{role} must be mono IEEE float32, got {} with {} channels",
            audio.sample_format, audio.channel_count
        ));
    }
    Ok(LoadedReference {
        path: path.to_owned(),
        sha256: actual_hash,
        audio,
    })
}

fn validate_reference_pair(left: &LoadedReference, right: &LoadedReference) -> Result<(), String> {
    if left.audio.sample_rate_hz != right.audio.sample_rate_hz
        || left.audio.mono_samples.len() != right.audio.mono_samples.len()
    {
        return Err(
            "modal and transient references must have identical rate and length".to_owned(),
        );
    }
    Ok(())
}

fn compare_render(
    reference: &LoadedReference,
    rendered: &[f32],
    output_file: &'static str,
    output_sha256: String,
) -> Result<AudioReproductionReport, String> {
    let (exact_sample_count, residual) =
        calculate_residual(&reference.audio.mono_samples, rendered)?;
    let accepted =
        residual.rms <= RMS_RESIDUAL_LIMIT && residual.correlation >= CORRELATION_MINIMUM;
    Ok(AudioReproductionReport {
        reference_file: reference.path.display().to_string(),
        reference_sha256: reference.sha256.clone(),
        output_file,
        output_sha256,
        exact_sample_count,
        residual,
        accepted,
    })
}

fn compare_fixed_point_render(
    reference: &[f32],
    rendered: &[f32],
    reference_file: &'static str,
    reference_sha256: String,
    output_file: &'static str,
    output_sha256: String,
) -> Result<FixedPointAudioReport, String> {
    let (_, residual) = calculate_residual_f32(reference, rendered)?;
    let accepted = residual.maximum_absolute <= Q30_MAXIMUM_ABSOLUTE_RESIDUAL_LIMIT
        && residual.rms <= Q30_RMS_RESIDUAL_LIMIT
        && residual.correlation >= Q30_CORRELATION_MINIMUM;
    Ok(FixedPointAudioReport {
        reference_file,
        reference_sha256,
        output_file,
        output_sha256,
        residual,
        accepted,
    })
}

fn validate_profile(profile: &ExternalProfile) -> Result<(), String> {
    if profile.schema != PROFILE_SCHEMA {
        return Err(format!(
            "unsupported external modal profile schema: {}",
            profile.schema
        ));
    }
    if !is_lower_hex(&profile.upstream_commit, 40) {
        return Err(
            "external modal profile upstream commit must be 40 lowercase hex digits".to_owned(),
        );
    }
    if profile.modes.is_empty() || profile.modes.len() > MAX_MODE_COUNT {
        return Err(format!(
            "external modal profile mode count must be 1..={MAX_MODE_COUNT}"
        ));
    }
    if profile.transient_samples != profile.transient_values.len()
        || profile.transient_samples > MAX_TRANSIENT_SAMPLES
        || profile
            .transient_values
            .iter()
            .any(|value| !value.is_finite())
    {
        return Err("external modal profile transient is invalid or unbounded".to_owned());
    }
    for (index, mode) in profile.modes.iter().enumerate() {
        let expected_t60 = 1000.0_f64.ln() / mode.damping_per_second;
        if mode.index != index
            || !mode.undamped_frequency_hz.is_finite()
            || !mode.damped_frequency_hz.is_finite()
            || mode.undamped_frequency_hz <= 0.0
            || mode.damped_frequency_hz <= 0.0
            || mode.damped_frequency_hz > mode.undamped_frequency_hz
            || !mode.damping_per_second.is_finite()
            || mode.damping_per_second <= 0.0
            || !mode.t60_seconds.is_finite()
            || (mode.t60_seconds - expected_t60).abs() > expected_t60 * 1.0e-5
            || !mode.runtime_decay_pole.is_finite()
            || !(0.0..1.0).contains(&mode.runtime_decay_pole)
            || !mode.ridge_amplitude.is_finite()
            || mode.ridge_amplitude.abs() > 1.000_001
        {
            return Err(format!(
                "external modal profile mode {index} is inconsistent"
            ));
        }
    }
    if profile
        .modes
        .iter()
        .all(|mode| mode.ridge_amplitude.abs() <= f64::EPSILON)
    {
        return Err("external modal profile ridge amplitudes are all zero".to_owned());
    }
    validate_audio_file(&profile.audio.ridge_amplitude)?;
    validate_audio_file(&profile.audio.ridge_amplitude_transient)?;
    if profile.audio.ridge_amplitude.file == profile.audio.ridge_amplitude_transient.file
        || profile.audio.ridge_amplitude.sha256 == profile.audio.ridge_amplitude_transient.sha256
    {
        return Err("modal and transient references must be distinct".to_owned());
    }
    Ok(())
}

fn validate_profile_for_sample_rate(
    profile: &ExternalProfile,
    sample_rate_hz: u32,
) -> Result<(), String> {
    let sample_rate = f64::from(sample_rate_hz);
    let nyquist = sample_rate * 0.5;
    for (index, mode) in profile.modes.iter().enumerate() {
        let expected_pole = (-mode.damping_per_second / sample_rate).exp();
        if mode.undamped_frequency_hz >= nyquist
            || mode.damped_frequency_hz >= nyquist
            || (mode.runtime_decay_pole - expected_pole).abs() > 1.0e-7
        {
            return Err(format!(
                "external modal profile mode {index} is inconsistent with {sample_rate_hz} Hz"
            ));
        }
    }
    Ok(())
}

fn validate_audio_file(audio: &ExternalAudioFile) -> Result<(), String> {
    let path = Path::new(&audio.file);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(format!(
            "external modal reference must be a sibling file: {}",
            audio.file
        ));
    }
    if !is_lower_hex(&audio.sha256, 64) {
        return Err(format!(
            "external modal reference has invalid SHA-256: {}",
            audio.sha256
        ));
    }
    Ok(())
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn canonical_external_input(root: &Path, path: &Path, role: &str) -> Result<PathBuf, String> {
    let unresolved = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let resolved = fs::canonicalize(&unresolved)
        .map_err(|error| format!("canonicalize {role} {}: {error}", unresolved.display()))?;
    if resolved.starts_with(root) {
        return Err(format!(
            "physical-sound-reproduce {role} must stay outside the repository: {}",
            resolved.display()
        ));
    }
    if !resolved.is_file() {
        return Err(format!("{role} is not a file: {}", resolved.display()));
    }
    Ok(resolved)
}

fn canonical_sibling(parent: &Path, file: &str) -> Result<PathBuf, String> {
    let resolved = fs::canonicalize(parent.join(file))
        .map_err(|error| format!("canonicalize modal reference {file}: {error}"))?;
    if resolved.parent() != Some(parent) || !resolved.is_file() {
        return Err(format!(
            "modal reference is not a regular sibling file: {file}"
        ));
    }
    Ok(resolved)
}

fn resolve_external_output(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let unresolved = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let resolved = if unresolved.exists() {
        fs::canonicalize(&unresolved)
            .map_err(|error| format!("canonicalize {}: {error}", unresolved.display()))?
    } else {
        let parent = unresolved
            .parent()
            .ok_or_else(|| "output path has no parent directory".to_owned())?;
        let parent = fs::canonicalize(parent)
            .map_err(|error| format!("canonicalize output parent {}: {error}", parent.display()))?;
        let name = unresolved
            .file_name()
            .ok_or_else(|| "output path has no directory name".to_owned())?;
        parent.join(name)
    };
    if resolved.starts_with(root) {
        return Err(format!(
            "physical-sound-reproduce output must stay outside the repository: {}",
            resolved.display()
        ));
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
            "output directory must be empty: {}",
            output.display()
        ));
    }
    Ok(())
}

fn encode_float32_mono_wav(sample_rate_hz: u32, samples: &[f32]) -> Result<Vec<u8>, String> {
    let data_bytes = u32::try_from(
        samples
            .len()
            .checked_mul(4)
            .ok_or_else(|| "float WAV byte length overflow".to_owned())?,
    )
    .map_err(|_| "float WAV exceeds RIFF size limit".to_owned())?;
    let mut bytes = Vec::with_capacity(44 + data_bytes as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&data_bytes.wrapping_add(36).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&3_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate_hz * 4).to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&32_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    ContentHash::from_bytes(sha256(bytes)).to_hex()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_require_external_profile_and_output() {
        assert!(
            parse_arguments(
                [
                    "--profile".to_owned(),
                    "/tmp/profile.json".to_owned(),
                    "--output".to_owned(),
                    "/tmp/output".to_owned(),
                ]
                .into_iter(),
            )
            .is_ok()
        );
        assert!(parse_arguments(std::iter::empty()).is_err());
        assert!(
            parse_arguments(
                [
                    "--profile".to_owned(),
                    "/tmp/profile.json".to_owned(),
                    "--profile".to_owned(),
                    "/tmp/other.json".to_owned(),
                    "--output".to_owned(),
                    "/tmp/output".to_owned(),
                ]
                .into_iter(),
            )
            .is_err()
        );
    }

    #[test]
    fn float_wav_round_trips_through_the_quality_decoder() {
        let samples = [0.0_f32, 0.25, -0.5, 1.0];
        let wav = encode_float32_mono_wav(32_000, &samples).expect("encode float WAV");
        let decoded = parse_wav(&wav).expect("decode float WAV");
        assert_eq!(decoded.sample_rate_hz, 32_000);
        assert_eq!(decoded.channel_count, 1);
        assert_eq!(decoded.sample_format, "ieee-f32");
        assert_eq!(decoded.mono_samples, samples.map(f64::from).to_vec());
    }

    #[test]
    fn profile_validation_checks_modal_relations_and_reference_identity() {
        let mut profile = valid_profile();
        validate_profile(&profile).expect("valid profile structure");
        validate_profile_for_sample_rate(&profile, 32_000).expect("valid profile rate");

        profile.modes[0].runtime_decay_pole = 0.5;
        assert!(validate_profile_for_sample_rate(&profile, 32_000).is_err());
        profile = valid_profile();
        profile.audio.ridge_amplitude_transient.sha256 =
            profile.audio.ridge_amplitude.sha256.clone();
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn transient_resampling_preserves_duration_and_uses_a_zero_boundary() {
        let resampled = resample_zero_extended_linear(&[0.0, 1.0, 0.0, 1.0], 32_000, 48_000)
            .expect("resample exact three-to-two duration");
        assert_eq!(resampled.len(), 6);
        let expected = [0.0, 2.0 / 3.0, 2.0 / 3.0, 0.0, 2.0 / 3.0, 2.0 / 3.0];
        for (actual, expected) in resampled.iter().zip(expected) {
            assert!((actual - expected).abs() <= f64::EPSILON);
        }
        assert_eq!(
            scale_sample_count(16_000, 32_000, 48_000).expect("exact half second"),
            24_000
        );
        assert!(scale_sample_count(1, 32_000, 48_000).is_err());
    }

    #[test]
    fn audition_pair_is_stereo_and_separates_a_from_b() {
        let a = normalized_mono_to_stereo_s16(&[1.0, -1.0]).expect("valid normalized A");
        let b = normalized_mono_to_stereo_s16(&[0.5]).expect("valid normalized B");
        let combined =
            concatenate_audition_pair(&a, &b, 48_000, 250).expect("valid stereo audition pair");
        assert_eq!(&combined[..4], a.as_slice());
        assert!(combined[4..4 + 24_000].iter().all(|sample| *sample == 0));
        assert_eq!(&combined[4 + 24_000..], b.as_slice());
    }

    fn valid_profile() -> ExternalProfile {
        let damping = 40.0;
        ExternalProfile {
            schema: PROFILE_SCHEMA.to_owned(),
            upstream_commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            step: 1,
            transient_samples: 1,
            transient_values: vec![0.01],
            modes: vec![ExternalMode {
                index: 0,
                undamped_frequency_hz: 1_000.1,
                damped_frequency_hz: 1_000.0,
                damping_per_second: damping,
                t60_seconds: 1000.0_f64.ln() / damping,
                runtime_decay_pole: (-damping / 32_000.0_f64).exp(),
                ridge_amplitude: 1.0,
            }],
            audio: ExternalAudio {
                ridge_amplitude: ExternalAudioFile {
                    file: "ridge.wav".to_owned(),
                    sha256: "a".repeat(64),
                },
                ridge_amplitude_transient: ExternalAudioFile {
                    file: "ridge-transient.wav".to_owned(),
                    sha256: "b".repeat(64),
                },
            },
        }
    }
}
