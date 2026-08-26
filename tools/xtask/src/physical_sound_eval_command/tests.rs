use std::f64::consts::PI;

use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};

use super::audio_analysis::WavAudio;
use super::*;

#[test]
fn arguments_require_manifest_and_output() {
    let request = parse_arguments(
        [
            "--manifest".to_owned(),
            "/tmp/manifest.json".to_owned(),
            "--output".to_owned(),
            "/tmp/report".to_owned(),
            "--blind-seed".to_owned(),
            "7".to_owned(),
        ]
        .into_iter(),
    )
    .expect("arguments parse");
    assert_eq!(request.blind_seed, 7);
    assert!(parse_arguments(std::iter::empty()).is_err());
    assert!(parse_arguments(["--unknown".to_owned()].into_iter()).is_err());
}

#[test]
fn canonical_wav_downmixes_and_parses() {
    let profile = AudioMixProfileV1::stereo_baseline_v1().expect("profile");
    let wav = encode_canonical_wav(&profile, &[1_000, -1_000, 2_000, 2_000]);
    let parsed = parse_wav(&wav).expect("WAV parses");
    assert_eq!(parsed.sample_format, "pcm-s16");
    assert_eq!(parsed.sample_rate_hz, 48_000);
    assert_eq!(parsed.channel_count, 2);
    assert_eq!(parsed.mono_samples.len(), 2);
    assert_eq!(parsed.mono_samples[0], 0.0);
}

#[test]
fn malformed_wav_and_unfrozen_manifest_reject() {
    assert!(parse_wav(b"not a wave").is_err());

    let mut manifest = test_manifest();
    manifest.entries[0].candidate.sha256 = "not-a-hash".to_owned();
    assert!(validate_manifest(&manifest).is_err());

    let mut manifest = test_manifest();
    let duplicate = manifest.entries[0].clone();
    manifest.entries.push(duplicate);
    assert!(validate_manifest(&manifest).is_err());
}

#[test]
fn identical_signal_scores_better_than_octave_mismatch() {
    let reference = analyze_test_signal(440.0, 6.0);
    let identical = analyze_test_signal(440.0, 6.0);
    let octave = analyze_test_signal(880.0, 2.0);
    let identical_report = matched_report(&identical, &reference);
    let octave_report = matched_report(&octave, &reference);
    assert!(identical_report.gain_matched_multiresolution_log_spectrum_rmse_db < 1.0e-9);
    assert!(identical_report.modal_assignment_cost < 1.0e-9);
    assert!(
        octave_report.gain_matched_multiresolution_log_spectrum_rmse_db
            > identical_report.gain_matched_multiresolution_log_spectrum_rmse_db + 5.0
    );
    assert!(octave_report.modal_assignment_cost > identical_report.modal_assignment_cost);
}

#[test]
fn blind_side_is_stable_and_pair_specific() {
    assert_eq!(
        blind_candidate_is_a(42, "steel-center"),
        blind_candidate_is_a(42, "steel-center")
    );
    let sides = (0..32)
        .map(|index| blind_candidate_is_a(42, &format!("pair-{index}")))
        .collect::<Vec<_>>();
    assert!(sides.iter().any(|side| *side));
    assert!(sides.iter().any(|side| !*side));
}

fn analyze_test_signal(frequency_hz: f64, decay_per_second: f64) -> Analysis {
    let sample_rate_hz = 48_000_u32;
    let mono_samples = (0..24_000)
        .map(|index| {
            let time = index as f64 / f64::from(sample_rate_hz);
            (2.0 * PI * frequency_hz * time).sin() * (-decay_per_second * time).exp() * 0.5
        })
        .collect();
    analyze_wav(
        "test.wav",
        &"0".repeat(64),
        WavAudio {
            sample_format: "test-f64".to_owned(),
            sample_rate_hz,
            channel_count: 1,
            mono_samples,
        },
    )
    .expect("analysis")
}

fn test_manifest() -> QualityManifest {
    QualityManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        split: "test".to_owned(),
        entries: vec![ManifestEntry {
            id: "test-entry".to_owned(),
            object_id: "test-object".to_owned(),
            material: "test-material".to_owned(),
            impact_position: "center".to_owned(),
            force_band: "medium".to_owned(),
            candidate: AudioFileRef {
                path: "test.wav".to_owned(),
                sha256: "0".repeat(64),
            },
            reference: None,
        }],
    }
}
