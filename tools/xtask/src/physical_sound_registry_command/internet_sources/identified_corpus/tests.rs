use serde_json::Value;

use super::super::adapters::RecordingReport;
use super::super::{CapabilityReport, RemoteArtifactReport};
use super::*;
use crate::physical_sound_registry_command::ArtifactReport;

#[test]
fn arguments_require_manifest_cache_and_output() {
    let request = parse_arguments(
        [
            "--manifest",
            "/tmp/identified.json",
            "--cache",
            "/tmp/source-cache",
            "--output",
            "/tmp/identified-report",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("arguments parse");
    assert_eq!(request.manifest, PathBuf::from("/tmp/identified.json"));
    assert!(parse_arguments(std::iter::empty()).is_err());
    assert!(
        parse_arguments(
            [
                "--manifest",
                "/tmp/identified.json",
                "--cache",
                "/tmp/source-cache",
                "--output",
                "/tmp/identified-report",
                "--fetch-missing",
            ]
            .into_iter()
            .map(str::to_owned)
        )
        .is_err()
    );
}

#[test]
fn normalizes_e3_objects_and_measures_target_coverage_repeatably() {
    let manifest = test_manifest(Partition::Dev, Partition::Dev);
    let first = build_report(
        manifest,
        "11".repeat(32),
        "22".repeat(32),
        "33".repeat(32),
        test_source_audit(),
        "44".repeat(32),
        test_requirements(),
    )
    .expect("identified corpus validates");
    let second = build_report(
        test_manifest(Partition::Dev, Partition::Dev),
        "11".repeat(32),
        "22".repeat(32),
        "33".repeat(32),
        test_source_audit(),
        "44".repeat(32),
        test_requirements(),
    )
    .expect("identified corpus repeats");
    let first_json = serde_json::to_vec_pretty(&first).expect("serialize report");
    assert_eq!(
        first_json,
        serde_json::to_vec_pretty(&second).expect("serialize repeated report")
    );
    let report: Value = serde_json::from_slice(&first_json).expect("parse report");
    assert_eq!(report["decision"], "DevelopmentCoverageMeasured");
    assert_eq!(report["source_count"], 2);
    assert_eq!(report["recording_count"], 4);
    assert_eq!(report["independent_group_counts"]["project_revisions"], 1);
    assert_eq!(report["independent_group_counts"]["objects"], 2);
    assert_eq!(report["coverage_against_plan"]["target_object_groups"], 1);
    assert_eq!(
        report["coverage_against_plan"]["missing_target_object_groups"],
        15
    );
    assert_eq!(
        report["coverage_against_plan"]["reject_parent_groups_in_this_e3_corpus"],
        0
    );
    assert!(
        report["coverage_against_plan"]
            .get("reject_parent_partition_counts")
            .is_none()
    );
    assert!(
        report["coverage_against_plan"]
            .get("missing_reject_parent_groups")
            .is_none()
    );
    assert!(report["entries"][0].get("corpus_role").is_none());
    assert_eq!(
        report["claim"],
        "GROUP_AND_COVERAGE_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
    );
}

#[test]
fn explicit_roles_count_only_declared_non_target_parent_groups() {
    let report = build_report(
        explicit_test_manifest(),
        "11".repeat(32),
        "22".repeat(32),
        "33".repeat(32),
        test_source_audit(),
        "44".repeat(32),
        test_requirements(),
    )
    .expect("explicit target and reject-parent roles validate");
    let json = serde_json::to_value(report).expect("serialize report");
    assert_eq!(json["coverage_against_plan"]["target_object_groups"], 1);
    assert_eq!(
        json["coverage_against_plan"]["reject_parent_groups_in_this_e3_corpus"],
        1
    );
    assert_eq!(
        json["coverage_against_plan"]["missing_reject_parent_groups"],
        34
    );
    assert_eq!(
        json["coverage_against_plan"]["reject_parent_coverage_sufficient"],
        false
    );
    assert_eq!(
        json["coverage_against_plan"]["reject_parent_partition_counts"][0],
        serde_json::json!({
            "partition": "dev",
            "object_groups": 1,
            "recordings": 2
        })
    );
    assert_eq!(json["entries"][0]["corpus_role"], "target");
    assert_eq!(json["entries"][2]["corpus_role"], "reject_parent");
}

#[test]
fn explicit_roles_fail_closed_on_partial_or_material_mismatch() {
    let mut partial = test_manifest(Partition::Dev, Partition::Dev);
    partial.assignments[0].corpus_role = CorpusRole::Target;
    let error = validate_manifest(&partial).expect_err("partial role declarations reject");
    assert!(error.contains("cannot mix assigned and unassigned"));

    let mut mismatched = explicit_test_manifest();
    mismatched.assignments[0].corpus_role = CorpusRole::RejectParent;
    mismatched.assignments[1].corpus_role = CorpusRole::Target;
    let result = build_report(
        mismatched,
        "11".repeat(32),
        "22".repeat(32),
        "33".repeat(32),
        test_source_audit(),
        "44".repeat(32),
        test_requirements(),
    );
    let error = match result {
        Ok(_) => panic!("material-role mismatch must reject"),
        Err(error) => error,
    };
    assert!(error.contains("reject-parent source object-006 has target material Glass"));
}

#[test]
fn rejects_partition_leakage_within_one_project_revision() {
    let result = build_report(
        test_manifest(Partition::Dev, Partition::Shadow),
        "11".repeat(32),
        "22".repeat(32),
        "33".repeat(32),
        test_source_audit(),
        "44".repeat(32),
        test_requirements(),
    );
    let error = match result {
        Ok(_) => panic!("shared source group must not cross partitions"),
        Err(error) => error,
    };
    assert!(error.contains("partition leakage for source group"));
}

#[test]
fn rejects_non_exact_e3_source() {
    let mut audit = test_source_audit();
    audit.sources[0].capabilities[0].adapter_validated = false;
    let result = build_report(
        test_manifest(Partition::Dev, Partition::Dev),
        "11".repeat(32),
        "22".repeat(32),
        "33".repeat(32),
        audit,
        "44".repeat(32),
        test_requirements(),
    );
    let error = match result {
        Ok(_) => panic!("unvalidated capability rejects"),
        Err(error) => error,
    };
    assert!(error.contains("does not close required E3 capability"));
}

fn test_manifest(first: Partition, second: Partition) -> IdentifiedCorpusManifest {
    IdentifiedCorpusManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        corpus_id: "av-msf-test".to_owned(),
        revision: "r1".to_owned(),
        target_material_label: "Glass".to_owned(),
        internet_source_manifest: test_file_ref("sources.json", "aa"),
        corpus_plan_report: test_file_ref("plan.json", "bb"),
        assignments: vec![
            SourceAssignment {
                source_id: "object-006".to_owned(),
                partition: first,
                corpus_role: CorpusRole::Unassigned,
            },
            SourceAssignment {
                source_id: "object-012".to_owned(),
                partition: second,
                corpus_role: CorpusRole::Unassigned,
            },
        ],
    }
}

fn explicit_test_manifest() -> IdentifiedCorpusManifest {
    let mut manifest = test_manifest(Partition::Dev, Partition::Dev);
    manifest.assignments[0].corpus_role = CorpusRole::Target;
    manifest.assignments[1].corpus_role = CorpusRole::RejectParent;
    manifest
}

fn test_file_ref(path: &str, byte: &str) -> FileRef {
    FileRef {
        path: path.to_owned(),
        sha256: byte.repeat(32),
    }
}

fn test_requirements() -> RiskPowerProbe {
    RiskPowerProbe {
        required_reject_parent_groups: 35,
        minimum_in_domain_groups_for_coverage: 16,
    }
}

fn test_source_audit() -> InternetSourceReport {
    InternetSourceReport {
        schema: super::super::REPORT_SCHEMA,
        status: "Validated",
        decision: "SourceSetComplete",
        claim: "INTERNET_SOURCE_CACHE_AND_CAPABILITY_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY",
        registry_id: "av-msf-test".to_owned(),
        revision: "r1".to_owned(),
        manifest_sha256: "aa".repeat(32),
        source_count: 2,
        ready_source_count: 2,
        sources: vec![
            test_source("object-006", "6", "Glass", &["017", "027"]),
            test_source("object-012", "12", "Wood", &["016", "019"]),
        ],
    }
}

fn test_source(
    source_id: &str,
    object_id: &str,
    material_label: &str,
    recording_ids: &[&str],
) -> SourceReport {
    let mut artifacts = recording_ids
        .iter()
        .map(|recording_id| RemoteArtifactReport {
            id: format!("impact-{recording_id}"),
            role: "audio_payload",
            url: format!("https://example.invalid/{object_id}/{recording_id}.wav"),
            normalization_policy: None,
            maximum_bytes: 1_000_000,
            expected_byte_count: Some(529_258),
            expected_sha256: Some(recording_id.repeat(21).chars().take(64).collect()),
            cache_status: "CachedVerified",
        })
        .collect::<Vec<_>>();
    artifacts.push(RemoteArtifactReport {
        id: "project-page".to_owned(),
        role: "project_page",
        url: "https://example.invalid/index.html".to_owned(),
        normalization_policy: None,
        maximum_bytes: 1_000_000,
        expected_byte_count: Some(1_024),
        expected_sha256: Some("cc".repeat(32)),
        cache_status: "CachedVerified",
    });
    SourceReport {
        id: source_id.to_owned(),
        publisher_id: "zisen-shao".to_owned(),
        project_id: "av-msf".to_owned(),
        declared_revision: "commit-723df64a94480fc8f8e592c66c0d916e8b0054d1".to_owned(),
        review_date: "2026-08-27".to_owned(),
        landing_page_url: "https://zisenshao.github.io/AV-MSF/".to_owned(),
        terms_url: None,
        adapter_id: "av-msf-identified-recording-v1".to_owned(),
        adapter_evidence: Some(AdapterEvidenceReport::AvMsfIdentifiedRecordingV1 {
            object_id: object_id.to_owned(),
            material_label: material_label.to_owned(),
            recordings: recording_ids
                .iter()
                .map(|recording_id| RecordingReport {
                    recording_id: (*recording_id).to_owned(),
                    sample_encoding: "pcm_integer",
                    sample_rate_hz: 44_100,
                    channel_count: 1,
                    bits_per_sample: 16,
                    sample_frames: 264_600,
                })
                .collect(),
        }),
        license_expression: "NOASSERTION".to_owned(),
        redistribution_policy: "external_research_only",
        provenance_review: ArtifactReport {
            sha256: "dd".repeat(32),
            byte_count: 1_024,
        },
        artifacts,
        capabilities: REQUIRED_CAPABILITIES
            .iter()
            .map(|capability| CapabilityReport {
                capability,
                artifact_ids: vec!["project-page".to_owned()],
                bytes_available: true,
                adapter_validated: true,
                available: true,
            })
            .collect(),
        supported_tiers: vec!["E3IdentifiedRecording"],
        source_status: "EvidenceReady",
    }
}
