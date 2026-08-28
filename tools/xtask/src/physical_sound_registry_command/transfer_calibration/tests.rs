use super::*;

fn file_ref(path: &str, sha256: &str) -> FileRef {
    FileRef {
        path: path.to_owned(),
        sha256: sha256.to_owned(),
    }
}

fn manifest() -> CalibrationManifest {
    CalibrationManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        study_id: STUDY_ID.to_owned(),
        revision: STUDY_REVISION.to_owned(),
        source_feasibility_report: file_ref("source-report.json", SOURCE_REPORT_SHA256),
        rows: EXPECTED_ROWS
            .iter()
            .map(|row| RowDeclaration {
                profile_row_id: row.profile_row_id.to_owned(),
                partition: row.partition,
                inventory_report: file_ref("inventory.json", row.inventory_report_sha256),
                audio_payload: file_ref("transfer.f32le", row.audio_sha256),
            })
            .collect(),
        allowed_capabilities: ALLOWED_CAPABILITIES
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        prohibited_claims: PROHIBITED_CLAIMS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

#[test]
fn preregistration_requires_exact_object_disjoint_split_and_prohibitions() {
    validate_manifest(&manifest(), CalibrationVersion::V1).expect("frozen manifest validates");
    let mut changed = manifest();
    changed.rows.swap(2, 3);
    assert!(
        validate_manifest(&changed, CalibrationVersion::V1)
            .expect_err("partition swap rejects")
            .contains("frozen row declaration mismatch")
    );
    let mut changed = manifest();
    changed.prohibited_claims.pop();
    assert!(
        validate_manifest(&changed, CalibrationVersion::V1)
            .expect_err("missing prohibition rejects")
            .contains("prohibited claims mismatch")
    );
}

#[test]
fn v2_moves_only_the_fresh_holdout_boundary() {
    let mut manifest = manifest();
    manifest.schema = MANIFEST_SCHEMA_V2.to_owned();
    manifest.revision = STUDY_REVISION_V2.to_owned();
    for (row, expected) in manifest
        .rows
        .iter_mut()
        .zip(expected_rows(CalibrationVersion::V2))
    {
        row.partition = expected.partition;
    }
    assert_eq!(
        manifest_version(&manifest).expect("V2 version"),
        CalibrationVersion::V2
    );
    validate_manifest(&manifest, CalibrationVersion::V2).expect("V2 manifest validates");
    assert_eq!(manifest.rows[4].partition, Partition::Holdout);
}

#[test]
fn candidate_selection_uses_calibration_loss_after_dev_sanity() {
    let candidates = vec![
        CandidateSelectionReport {
            profile_id: CANDIDATES[0].id,
            dev_mean_calibration_loss: 1.0,
            calibration_loss: 3.0,
            dev_sanity_pass: true,
        },
        CandidateSelectionReport {
            profile_id: CANDIDATES[1].id,
            dev_mean_calibration_loss: 9.0,
            calibration_loss: 2.0,
            dev_sanity_pass: true,
        },
        CandidateSelectionReport {
            profile_id: CANDIDATES[2].id,
            dev_mean_calibration_loss: 0.1,
            calibration_loss: 1.0,
            dev_sanity_pass: false,
        },
    ];
    assert_eq!(
        select_candidate(&candidates).expect("candidate selected"),
        CANDIDATES[1].id
    );
}

#[test]
fn holdout_gate_is_conjunctive_and_does_not_grant_spatial_credit() {
    let passing = TransferAnalysis {
        onset_sample: 0,
        selected_mode_count: 8,
        persistent_mode_count: 6,
        persistent_mode_recall: 0.75,
        median_frequency_error_cents: 20.0,
        decaying_mode_fraction: 0.75,
        median_tail_prediction_rmse_db: 12.0,
        calibration_loss: 1.0,
        modes: Vec::new(),
    };
    assert!(evaluate_holdout(CalibrationVersion::V1, &passing).passed);
    let mut failing = passing;
    failing.median_tail_prediction_rmse_db = 25.0;
    assert!(!evaluate_holdout(CalibrationVersion::V1, &failing).passed);
    assert_eq!(
        expected_rows(CalibrationVersion::V1)
            .last()
            .expect("reserved row")
            .partition,
        Partition::Reserved
    );
}

#[test]
fn arguments_require_manifest_and_output() {
    let request = parse_arguments(
        [
            "--manifest".to_owned(),
            "/tmp/transfer-calibration.json".to_owned(),
            "--output".to_owned(),
            "/tmp/transfer-calibration-report".to_owned(),
        ]
        .into_iter(),
    )
    .expect("arguments parse");
    assert_eq!(
        request.manifest,
        PathBuf::from("/tmp/transfer-calibration.json")
    );
    assert!(parse_arguments(std::iter::empty()).is_err());
}
