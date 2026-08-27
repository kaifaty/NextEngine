use std::f64::consts::PI;

use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};

use super::audio_analysis::WavAudio;
use super::manifest::ImpactControl;
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
            "--write-blind-bundle".to_owned(),
        ]
        .into_iter(),
    )
    .expect("arguments parse");
    assert_eq!(request.blind_seed, 7);
    assert!(request.write_blind_bundle);
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

    let mut retired = test_manifest();
    retired.schema = "nextengine.experimental-physical-sound-quality.manifest.v0".to_owned();
    assert!(validate_manifest(&retired).is_err());

    let manifest_with_unknown_relation_field = serde_json::json!({
        "schema": MANIFEST_SCHEMA,
        "split": "test",
        "relations": [{
            "kind": "exact_wav_repeat",
            "id": "repeat",
            "left": "a",
            "right": "b",
            "generator_controlled_threshold": 999
        }],
        "entries": []
    });
    assert!(
        serde_json::from_value::<QualityManifest>(manifest_with_unknown_relation_field).is_err()
    );
}

#[test]
fn incomplete_control_matrix_falls_back_out_of_domain() {
    let manifest = test_manifest();
    let relation_reports = Vec::new();
    let coverage = evaluate_coverage(&manifest, &relation_reports);
    let mutation_suite = evaluate_mutation_suite().expect("mutation controls run");
    assert_eq!(coverage.status, "FallbackOutOfDomain");
    assert_eq!(mutation_suite.status, "Pass");
    assert!(
        coverage
            .missing_controls
            .iter()
            .any(|failure| failure == "VALIDATOR_DECLARATION_MISSING")
    );
}

#[test]
fn force_relation_rejects_energy_inversion() {
    let low = resolved_test_entry("low", "low", 880.0, 0.45, b"low");
    let high = resolved_test_entry("high", "high", 880.0, 0.15, b"high");
    let relation = RelationSpec::ForceResponse {
        id: "force".to_owned(),
        ordered_entries: vec!["low".to_owned(), "high".to_owned()],
    };
    let reports = evaluate_relations(&[relation], &[high, low]);
    assert_eq!(reports[0].status, "Reject");
    assert!(
        reports[0]
            .failure_tags
            .iter()
            .any(|tag| tag == "FORCE_ENERGY_NOT_MONOTONIC")
    );
}

#[test]
fn complete_control_matrix_can_pass_without_a_learned_judge() {
    let entries = complete_control_entries();
    let mut manifest = complete_control_manifest(&entries);
    validate_manifest(&manifest).expect("complete manifest validates");
    let relation_reports = evaluate_relations(&manifest.relations, &entries);
    let coverage = evaluate_coverage(&manifest, &relation_reports);
    let mutation_suite = evaluate_mutation_suite().expect("mutation controls run");
    let entry_reports = entries
        .into_iter()
        .map(build_entry_report)
        .collect::<Vec<_>>();
    let fallback = analyze_test_signal_with_gain(330.0, 6.0, 0.35);
    let declaration = manifest.validator.take().expect("validator declaration");
    let domain = build_domain_report(Some((&declaration, &fallback)))
        .expect("fallback validates")
        .expect("domain report");
    let decision = validator_decision(ValidatorDecisionInputs {
        entries: &entry_reports,
        domain: Some(&domain),
        mutation_suite: &mutation_suite,
        coverage: &coverage,
        relations: &relation_reports,
    });
    assert_eq!(coverage.status, "Pass");
    assert!(
        relation_reports
            .iter()
            .all(|report| report.status == "Pass")
    );
    assert_eq!(decision, "Pass");
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
    analyze_test_signal_with_gain(frequency_hz, decay_per_second, 0.5)
}

fn analyze_test_signal_with_gain(frequency_hz: f64, decay_per_second: f64, gain: f64) -> Analysis {
    let sample_rate_hz = 48_000_u32;
    let mono_samples = (0..24_000)
        .map(|index| {
            let time = index as f64 / f64::from(sample_rate_hz);
            (2.0 * PI * frequency_hz * time).sin() * (-decay_per_second * time).exp() * gain
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
        validator: None,
        relations: Vec::new(),
        entries: vec![ManifestEntry {
            id: "test-entry".to_owned(),
            object_id: "test-object".to_owned(),
            material: "test-material".to_owned(),
            impact_position: "center".to_owned(),
            force_band: "medium".to_owned(),
            expected_signal: ExpectedSignal::Impact,
            control: None,
            candidate: AudioFileRef {
                path: "test.wav".to_owned(),
                sha256: "0".repeat(64),
            },
            reference: None,
        }],
    }
}

fn resolved_test_entry(
    id: &str,
    force_band: &str,
    frequency_hz: f64,
    gain: f64,
    bytes: &[u8],
) -> ResolvedEntry {
    let impulse_micronewton_seconds = match force_band {
        "zero" => 0,
        "low" => 1_000_000,
        "medium" => 2_000_000,
        "high" => 4_000_000,
        _ => 2_000_000,
    };
    ResolvedEntry {
        manifest: ManifestEntry {
            id: id.to_owned(),
            object_id: "glass-vessel".to_owned(),
            material: "glass".to_owned(),
            impact_position: "center".to_owned(),
            force_band: force_band.to_owned(),
            expected_signal: ExpectedSignal::Impact,
            control: Some(ImpactControl {
                impact_position_micrometres: [0, 0, 0],
                impulse_micronewton_seconds,
            }),
            candidate: AudioFileRef {
                path: format!("{id}.wav"),
                sha256: "0".repeat(64),
            },
            reference: None,
        },
        candidate_bytes: bytes.to_vec(),
        candidate: analyze_test_signal_with_gain(frequency_hz, 6.0, gain),
        reference_bytes: None,
        reference: None,
    }
}

fn complete_control_entries() -> Vec<ResolvedEntry> {
    let repeat_bytes = b"repeat";
    let mut position_left =
        resolved_test_entry("position-left", "medium", 880.0, 0.3, b"position-left");
    position_left.manifest.impact_position = "left".to_owned();
    position_left
        .manifest
        .control
        .as_mut()
        .expect("position control")
        .impact_position_micrometres = [-10_000, 0, 0];
    let mut position_right =
        resolved_test_entry("position-right", "medium", 880.0, 0.27, b"position-right");
    position_right.manifest.impact_position = "right".to_owned();
    position_right
        .manifest
        .control
        .as_mut()
        .expect("position control")
        .impact_position_micrometres = [10_000, 0, 0];
    let mut entries = vec![
        resolved_test_entry("force-high", "high", 880.0, 0.48, b"force-high"),
        resolved_test_entry("force-low", "low", 880.0, 0.12, b"force-low"),
        position_left,
        position_right,
        resolved_test_entry("repeat-a", "medium", 880.0, 0.3, repeat_bytes),
        resolved_test_entry("repeat-b", "medium", 880.0, 0.3, repeat_bytes),
    ];
    let mut zero = resolved_test_entry("zero", "zero", 880.0, 0.0, b"zero");
    zero.manifest.expected_signal = ExpectedSignal::Silence;
    zero.candidate = analyze_wav(
        "zero.wav",
        &"0".repeat(64),
        WavAudio {
            sample_format: "test-f64".to_owned(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            mono_samples: vec![0.0; 24_000],
        },
    )
    .expect("silence analysis");
    entries.push(zero);
    entries.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    entries
}

fn complete_control_manifest(entries: &[ResolvedEntry]) -> QualityManifest {
    QualityManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        split: "complete-controls".to_owned(),
        validator: Some(ValidatorDeclaration {
            source_family: "rigid-impact".to_owned(),
            generator_revision: "test-generator.v1".to_owned(),
            generator_sha256: "1".repeat(64),
            deterministic_probe_seed: 7,
            fallback: AudioFileRef {
                path: "fallback.wav".to_owned(),
                sha256: "0".repeat(64),
            },
        }),
        relations: vec![
            RelationSpec::ExactWavRepeat {
                id: "repeat".to_owned(),
                left: "repeat-a".to_owned(),
                right: "repeat-b".to_owned(),
            },
            RelationSpec::ForceResponse {
                id: "response-force".to_owned(),
                ordered_entries: vec!["force-low".to_owned(), "force-high".to_owned()],
            },
            RelationSpec::PositionContinuity {
                id: "response-position".to_owned(),
                ordered_entries: vec!["position-left".to_owned(), "position-right".to_owned()],
            },
        ],
        entries: entries.iter().map(|entry| entry.manifest.clone()).collect(),
    }
}
