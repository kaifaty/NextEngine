use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use super::transfer_calibration::dsp as frozen_dsp;
use super::{require_empty_output, resolve_output_path, set_once, sha256_hex};

const REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-transfer-extractor-parity-fixture.report.v1";
const EXTRACTOR_ID: &str = "injective-modal-16-fft65536-v2";
const BOUND_DSP_SHA256: &str = "bae1eb6f459d930e32effd8f45d5788867553b1991d832e5f6dcdc9e677284d0";
const SAMPLE_RATE_HZ: u32 = 48_000;
const SAMPLE_COUNT: usize = SAMPLE_RATE_HZ as usize * 4;
const ONSET_SAMPLE: usize = 240;
const FREQUENCIES_HZ: [f64; 16] = [
    311.0, 433.0, 587.0, 751.0, 947.0, 1_187.0, 1_451.0, 1_783.0, 2_153.0, 2_591.0, 3_083.0,
    3_659.0, 4_327.0, 5_101.0, 6_011.0, 7_013.0,
];
static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn run_cli(
    root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => {
                return Err(format!(
                    "unexpected realimpact-transfer-fixture argument: {flag}"
                ));
            }
        }
    }
    let output = output.ok_or_else(|| {
        "physical-sound-registry realimpact-transfer-fixture requires --output <external-empty-directory>"
            .to_owned()
    })?;
    run(root, &output)
}

fn run(root: &Path, output_argument: &Path) -> Result<(), String> {
    validate_bound_dsp(root)?;
    let samples = fixture_samples();
    let sample_bytes = encode_samples(&samples);
    let analysis = frozen_dsp::analyze_v2(&samples, SAMPLE_RATE_HZ, frozen_dsp::CANDIDATES_V2[2])?;
    if analysis.onset_sample != ONSET_SAMPLE || analysis.selected_mode_count != 16 {
        return Err("frozen extractor did not recover the parity fixture contract".to_owned());
    }
    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "FrozenRustExtractorFixtureGenerated",
        claim: "SYNTHETIC_EXTRACTOR_PARITY_FIXTURE_ONLY / NO_NETWORK_OR_REALIMPACT_PAYLOAD_ACCESS",
        extractor_id: EXTRACTOR_ID,
        bound_dsp_sha256: BOUND_DSP_SHA256,
        sample_rate_hz: SAMPLE_RATE_HZ,
        sample_count: SAMPLE_COUNT,
        onset_seed_sample: ONSET_SAMPLE,
        source_frequencies_hz: FREQUENCIES_HZ,
        sample_format: "f64le",
        sample_path: "fixture.f64le",
        sample_sha256: sha256_hex(&sample_bytes),
        sample_bytes: sample_bytes.len(),
        analysis,
        network_requests: 0,
        realimpact_payload_bytes_read: 0,
    };
    let mut report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("serialize extractor fixture report: {error}"))?;
    report_bytes.push(b'\n');
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish(&output, &sample_bytes, &report_bytes)?;

    println!("REALIMPACT extractor parity fixture: {}", output.display());
    println!("sample sha256: {}", report.sample_sha256);
    println!("selected modes: {}", report.analysis.selected_mode_count);
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn validate_bound_dsp(root: &Path) -> Result<(), String> {
    let path =
        root.join("tools/xtask/src/physical_sound_registry_command/transfer_calibration/dsp.rs");
    let bytes = fs::read(&path)
        .map_err(|error| format!("read frozen transfer DSP {}: {error}", path.display()))?;
    let actual = sha256_hex(&bytes);
    if actual != BOUND_DSP_SHA256 {
        return Err(format!(
            "frozen transfer DSP hash changed: expected {BOUND_DSP_SHA256}, got {actual}"
        ));
    }
    Ok(())
}

fn fixture_samples() -> Vec<f64> {
    let mut samples = vec![0.0_f64; SAMPLE_COUNT];
    for (index, sample) in samples.iter_mut().enumerate().skip(ONSET_SAMPLE) {
        let time = (index - ONSET_SAMPLE) as f64 / f64::from(SAMPLE_RATE_HZ);
        *sample = FREQUENCIES_HZ
            .iter()
            .enumerate()
            .map(|(mode, frequency)| {
                let decay = (-time * (1.15 + mode as f64 * 0.09)).exp();
                let phase = mode as f64 * 0.173;
                decay * (2.0 * PI * frequency * time + phase).sin() / (1.0 + mode as f64 * 0.19)
            })
            .sum();
    }
    samples
}

fn encode_samples(samples: &[f64]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect()
}

fn publish(output: &Path, samples: &[u8], report: &[u8]) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "extractor fixture output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-transfer-fixture-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create extractor fixture staging: {error}"))?;
    let result = (|| {
        fs::write(staging.join("fixture.f64le"), samples)
            .map_err(|error| format!("write extractor fixture samples: {error}"))?;
        fs::write(staging.join("report.json"), report)
            .map_err(|error| format!("write extractor fixture report: {error}"))?;
        if output.exists() {
            fs::remove_dir(output)
                .map_err(|error| format!("remove confirmed-empty fixture output: {error}"))?;
        }
        fs::rename(&staging, output).map_err(|error| format!("publish extractor fixture: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    extractor_id: &'static str,
    bound_dsp_sha256: &'static str,
    sample_rate_hz: u32,
    sample_count: usize,
    onset_seed_sample: usize,
    source_frequencies_hz: [f64; 16],
    sample_format: &'static str,
    sample_path: &'static str,
    sample_sha256: String,
    sample_bytes: usize,
    analysis: frozen_dsp::TransferAnalysis,
    network_requests: usize,
    realimpact_payload_bytes_read: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_is_long_enough_for_frozen_v2_tail() {
        let samples = fixture_samples();
        assert!(samples.len() > ONSET_SAMPLE + 2_400 * 48 + 16_384);
        assert!(
            FREQUENCIES_HZ
                .windows(2)
                .all(|pair| pair[1] - pair[0] >= 12.0)
        );
    }

    #[test]
    fn fixture_generation_repeats_exactly() {
        assert_eq!(
            encode_samples(&fixture_samples()),
            encode_samples(&fixture_samples())
        );
    }
}
