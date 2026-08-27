use std::collections::BTreeMap;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};
use serde_json::Value;

use super::evaluator::{FeatureProfile, ResolvedEntry, evaluate_tasks};
use super::manifest::{
    BenchmarkManifest, CorpusEntry, CorpusSource, DistanceMetric, EntryOrigin, ExternalFeatureSet,
    FeatureMatrix, FeatureMatrixEntry, FileRef, LicenseDeclaration, LicenseReviewStatus,
    MeasurementScope, Partition, RedistributionPolicy, validate_feature_matrix, validate_manifest,
};
use super::*;

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-physical-sound-benchmark-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated physical sound benchmark directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path).expect("remove physical sound benchmark directory");
        }
    }
}

#[test]
fn arguments_require_manifest_and_output() {
    let request = parse_arguments(
        [
            "--manifest".to_owned(),
            "/tmp/corpus.json".to_owned(),
            "--output".to_owned(),
            "/tmp/corpus-report".to_owned(),
        ]
        .into_iter(),
    )
    .expect("arguments parse");
    assert_eq!(request.manifest, PathBuf::from("/tmp/corpus.json"));
    assert!(parse_arguments(std::iter::empty()).is_err());
    assert!(parse_arguments(["--unknown".to_owned()].into_iter()).is_err());
}

#[test]
fn manifest_rejects_partition_leakage_and_generated_training() {
    let mut manifest = test_manifest();
    manifest.entries[4].object_id = manifest.entries[0].object_id.clone();
    assert!(
        validate_manifest(&manifest)
            .expect_err("object leakage rejects")
            .contains("leaks across partitions")
    );

    let mut manifest = test_manifest();
    let development_index = manifest
        .entries
        .iter()
        .position(|entry| entry.partition == Partition::Development)
        .expect("development entry");
    manifest.entries[development_index].origin = EntryOrigin::Generated {
        generator_revision: "generator.v1".to_owned(),
        generator_sha256: "a".repeat(64),
    };
    assert!(
        validate_manifest(&manifest)
            .expect_err("generated development entry rejects")
            .contains("real-only development gallery")
    );

    let mut manifest = test_manifest();
    let steel_index = manifest
        .entries
        .iter()
        .position(|entry| entry.partition == Partition::Development && entry.material == "steel")
        .expect("development steel entry");
    manifest.entries[steel_index].object_family_id = "glass-family-a".to_owned();
    assert!(
        validate_manifest(&manifest)
            .expect_err("mixed-material family rejects")
            .contains("changes material")
    );
}

#[test]
fn material_identity_source_needs_no_fake_spatial_repeats() {
    let mut manifest = test_manifest_without_external_features();
    manifest.corpus_sources[0].measurement_scope = MeasurementScope::MaterialIdentityOnly;
    for entry in &mut manifest.entries {
        entry.impact_position_id = "unspecified".to_owned();
        entry.listener_position_id = "unspecified".to_owned();
        entry.force_band = "unspecified".to_owned();
    }
    validate_manifest(&manifest).expect("material-only corpus validates without fake repeats");

    manifest.entries[0].impact_position_id = "invented-position".to_owned();
    assert!(
        validate_manifest(&manifest)
            .expect_err("material-only corpus rejects invented spatial labels")
            .contains("requires unspecified")
    );
}

#[test]
fn unreviewed_public_source_cannot_declare_external_redistribution() {
    let mut manifest = test_manifest_without_external_features();
    manifest.corpus_sources[0].license.review_status =
        LicenseReviewStatus::UnreviewedPublicResearchSource;
    assert!(
        validate_manifest(&manifest)
            .expect_err("unreviewed public source needs the non-distribution boundary")
            .contains("requires no_repository_or_distribution")
    );

    manifest.corpus_sources[0].license.redistribution =
        RedistributionPolicy::NoRepositoryOrDistribution;
    validate_manifest(&manifest).expect("explicit non-distribution boundary validates");
}

#[test]
fn external_matrix_must_match_every_sorted_entry() {
    let manifest = test_manifest();
    let declaration = manifest
        .external_feature_sets
        .first()
        .expect("feature declaration");
    let mut matrix = FeatureMatrix {
        schema: FEATURE_MATRIX_SCHEMA.to_owned(),
        feature_set_id: declaration.id.clone(),
        entries: manifest
            .entries
            .iter()
            .map(|entry| FeatureMatrixEntry {
                id: entry.id.clone(),
                values: vec![0.0, 1.0],
            })
            .collect(),
    };
    validate_feature_matrix(declaration, &matrix, &manifest.entries)
        .expect("complete matrix validates");
    matrix.entries[0].id = "wrong".to_owned();
    assert!(validate_feature_matrix(declaration, &matrix, &manifest.entries).is_err());
}

#[test]
fn grouped_material_tasks_never_train_on_held_out_entries() {
    let manifest = test_manifest_without_external_features();
    let entries = manifest
        .entries
        .iter()
        .cloned()
        .map(|entry| {
            let material_value = if entry.material == "glass" { 0.0 } else { 10.0 };
            ResolvedEntry {
                audio: dummy_audio(&entry.id),
                manifest: entry,
                features: BTreeMap::from([(
                    CLASSICAL_FEATURE_SET_ID.to_owned(),
                    vec![material_value, material_value],
                )]),
            }
        })
        .collect::<Vec<_>>();
    let reports = evaluate_tasks(
        &entries,
        &[FeatureProfile {
            id: CLASSICAL_FEATURE_SET_ID.to_owned(),
            distance: DistanceMetric::Euclidean,
            dimensions: 2,
        }],
    );
    let holdout = reports
        .iter()
        .find(|report| report.id == "holdout_material_from_real_development")
        .expect("holdout report");
    assert_eq!(holdout.status, "Measured");
    assert_eq!(holdout.accuracy, Some(1.0));
    assert!(holdout.leakage_guard.contains("never train"));
    assert_eq!(holdout.predictions.len(), holdout.evaluated_count);
    assert!(
        holdout
            .predictions
            .iter()
            .all(|prediction| prediction.nearest_entry_id.starts_with("dev-"))
    );
    assert!(holdout.predictions.iter().all(|prediction| {
        prediction
            .expected_label_nearest_entry_id
            .starts_with("dev-")
            && prediction.nearest_competing_entry_id.is_some()
            && prediction
                .expected_label_margin
                .is_some_and(|margin| margin > 0.0)
    }));
}

#[test]
fn end_to_end_report_has_no_acceptance_authority() {
    let directory = TestDirectory::new();
    let license_bytes = b"Reviewed for isolated test corpus use only.";
    fs::write(directory.path.join("license-review.txt"), license_bytes)
        .expect("write license review");

    let mut manifest = test_manifest_without_external_features();
    manifest.corpus_sources[0].license.review_record.sha256 = sha256_hex(license_bytes);
    manifest.corpus_sources[0].license.review_status =
        LicenseReviewStatus::UnreviewedPublicResearchSource;
    manifest.corpus_sources[0].license.redistribution =
        RedistributionPolicy::NoRepositoryOrDistribution;
    for entry in &mut manifest.entries {
        let frequency = if entry.material == "glass" {
            2_400.0
        } else {
            540.0
        };
        let wav = test_wav(frequency);
        let file_name = format!("{}.wav", entry.id);
        fs::write(directory.path.join(&file_name), &wav).expect("write test WAV");
        entry.audio.path = file_name;
        entry.audio.sha256 = sha256_hex(&wav);
    }
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).expect("serialize manifest");
    let manifest_path = directory.path.join("manifest.json");
    fs::write(&manifest_path, manifest_bytes).expect("write manifest");
    let output = directory.path.join("report");
    run(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root"),
        &Request {
            manifest: manifest_path,
            output: output.clone(),
        },
    )
    .expect("benchmark runs");

    let report: Value = serde_json::from_slice(
        &fs::read(output.join("report.json")).expect("read benchmark report"),
    )
    .expect("parse benchmark report");
    assert_eq!(report["decision"], "NoAcceptanceAuthority");
    assert_eq!(report["benchmark_status"], "Measured");
    assert_eq!(report["split_audit"]["status"], "Pass");
    assert_eq!(
        report["corpus"]["sources"][0]["review_status"],
        "unreviewed_public_research_source"
    );
    assert_eq!(
        report["corpus"]["sources"][0]["redistribution"],
        "no_repository_or_distribution"
    );
    assert!(
        report["tasks"]
            .as_array()
            .expect("tasks")
            .iter()
            .any(|task| task["id"] == "shadow_material_from_real_development")
    );

    let repeated_output = directory.path.join("report-repeated");
    run(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root"),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: repeated_output.clone(),
        },
    )
    .expect("repeated benchmark runs");
    assert_eq!(
        fs::read(output.join("report.json")).expect("read first report bytes"),
        fs::read(repeated_output.join("report.json")).expect("read repeated report bytes")
    );
}

fn test_manifest() -> BenchmarkManifest {
    let mut manifest = test_manifest_without_external_features();
    manifest.external_feature_sets.push(ExternalFeatureSet {
        id: "beats".to_owned(),
        model_revision: "beats-iter3-plus-as2m".to_owned(),
        model_sha256: "b".repeat(64),
        dimensions: 2,
        distance: DistanceMetric::Cosine,
        matrix: FileRef {
            path: "beats.json".to_owned(),
            sha256: "c".repeat(64),
        },
    });
    manifest
}

fn test_manifest_without_external_features() -> BenchmarkManifest {
    let mut entries = vec![
        entry(
            "dev-glass-a-p0",
            Partition::Development,
            "glass-a",
            "glass-family-a",
            "glass",
            "p0",
        ),
        entry(
            "dev-glass-a-p1",
            Partition::Development,
            "glass-a",
            "glass-family-a",
            "glass",
            "p1",
        ),
        entry(
            "dev-glass-b-p0",
            Partition::Development,
            "glass-b",
            "glass-family-b",
            "glass",
            "p0",
        ),
        entry(
            "dev-glass-b-p1",
            Partition::Development,
            "glass-b",
            "glass-family-b",
            "glass",
            "p1",
        ),
        entry(
            "dev-steel-a-p0",
            Partition::Development,
            "steel-a",
            "steel-family-a",
            "steel",
            "p0",
        ),
        entry(
            "dev-steel-a-p1",
            Partition::Development,
            "steel-a",
            "steel-family-a",
            "steel",
            "p1",
        ),
        entry(
            "dev-steel-b-p0",
            Partition::Development,
            "steel-b",
            "steel-family-b",
            "steel",
            "p0",
        ),
        entry(
            "dev-steel-b-p1",
            Partition::Development,
            "steel-b",
            "steel-family-b",
            "steel",
            "p1",
        ),
        entry(
            "cal-glass",
            Partition::Calibration,
            "glass-cal",
            "glass-family-cal",
            "glass",
            "p0",
        ),
        entry(
            "cal-steel",
            Partition::Calibration,
            "steel-cal",
            "steel-family-cal",
            "steel",
            "p0",
        ),
        entry(
            "hold-glass",
            Partition::Holdout,
            "glass-hold",
            "glass-family-hold",
            "glass",
            "p0",
        ),
        entry(
            "hold-steel",
            Partition::Holdout,
            "steel-hold",
            "steel-family-hold",
            "steel",
            "p0",
        ),
        entry(
            "shadow-glass",
            Partition::Shadow,
            "glass-shadow",
            "glass-family-shadow",
            "glass",
            "p0",
        ),
        entry(
            "shadow-steel",
            Partition::Shadow,
            "steel-shadow",
            "steel-family-shadow",
            "steel",
            "p0",
        ),
    ];
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    BenchmarkManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        benchmark_id: "test-rigid-impact-corpus-v1".to_owned(),
        corpus_sources: vec![CorpusSource {
            id: "test-source".to_owned(),
            revision: "test-source-v1".to_owned(),
            source_url: "https://example.invalid/test-source".to_owned(),
            attribution: "Next Engine test fixture".to_owned(),
            measurement_scope: MeasurementScope::ControlledImpact,
            license: LicenseDeclaration {
                spdx_id: "CC0-1.0".to_owned(),
                review_status: LicenseReviewStatus::ApprovedExternalBenchmarkOnly,
                redistribution: RedistributionPolicy::ExternalOnly,
                review_record: FileRef {
                    path: "license-review.txt".to_owned(),
                    sha256: "a".repeat(64),
                },
            },
        }],
        external_feature_sets: Vec::new(),
        entries,
    }
}

fn entry(
    id: &str,
    partition: Partition,
    object_id: &str,
    object_family_id: &str,
    material: &str,
    impact_position_id: &str,
) -> CorpusEntry {
    CorpusEntry {
        id: id.to_owned(),
        source_id: "test-source".to_owned(),
        partition,
        object_id: object_id.to_owned(),
        object_family_id: object_family_id.to_owned(),
        material: material.to_owned(),
        impact_position_id: impact_position_id.to_owned(),
        listener_position_id: "listener-0".to_owned(),
        force_band: "medium".to_owned(),
        origin: EntryOrigin::Real,
        audio: FileRef {
            path: format!("{id}.wav"),
            sha256: "0".repeat(64),
        },
    }
}

fn dummy_audio(id: &str) -> EntryAudioReport {
    EntryAudioReport {
        id: id.to_owned(),
        wav_sha256: "0".repeat(64),
        sample_rate_hz: 48_000,
        channel_count: 2,
        duration_ms: 250.0,
        peak_dbfs: -6.0,
        rms_dbfs: -18.0,
        hard_failure_tags: Vec::new(),
    }
}

fn test_wav(frequency_hz: f64) -> Vec<u8> {
    let profile = AudioMixProfileV1::stereo_baseline_v1().expect("audio profile");
    let mut samples = Vec::with_capacity(24_000);
    for index in 0..12_000 {
        let time = index as f64 / 48_000.0;
        let sample = (2.0 * PI * frequency_hz * time).sin() * (-8.0 * time).exp() * 0.35;
        let sample = (sample * f64::from(i16::MAX)).round() as i16;
        samples.extend([sample, sample]);
    }
    encode_canonical_wav(&profile, &samples)
}
