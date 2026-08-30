use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use super::*;

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-physical-sound-neural-data-plane-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create neural data plane test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path).expect("remove neural data plane test directory");
        }
    }
}

#[test]
fn projection_is_hash_closed_repeatable_and_keeps_sealed_rows_private() {
    let directory = TestDirectory::new();
    let manifest = write_manifest(&directory.path);
    let manifest_path = directory.path.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize neural manifest"),
    )
    .expect("write neural manifest");

    let first = directory.path.join("first");
    run(
        workspace_root(),
        &Request {
            manifest: manifest_path.clone(),
            output: first.clone(),
        },
    )
    .expect("first neural projection validates");
    let second = directory.path.join("second");
    run(
        workspace_root(),
        &Request {
            manifest: manifest_path,
            output: second.clone(),
        },
    )
    .expect("second neural projection validates");

    for file in [
        "fit-projection.json",
        "calibration-projection.json",
        "sealed-role-commitments.json",
        "report.json",
    ] {
        assert_eq!(
            fs::read(first.join(file)).expect("read first output"),
            fs::read(second.join(file)).expect("read second output")
        );
    }
    let report: Value =
        serde_json::from_slice(&fs::read(first.join("report.json")).expect("read neural report"))
            .expect("parse neural report");
    assert_eq!(report["status"], "Validated");
    assert_eq!(report["decision"], "DeclaredAxisCoverageComplete");
    assert_eq!(report["row_count"], 10);
    assert_eq!(report["model_training_authorized"], false);
    assert_eq!(report["method_holdout_materialized"], false);
    assert_eq!(report["admission_shadow_materialized"], false);

    let sealed = fs::read_to_string(first.join("sealed-role-commitments.json"))
        .expect("read sealed commitments");
    assert!(!sealed.contains("row-method-holdout"));
    assert!(!sealed.contains("row-admission-shadow"));
    let fit = fs::read_to_string(first.join("fit-projection.json")).expect("read fit projection");
    let calibration = fs::read_to_string(first.join("calibration-projection.json"))
        .expect("read calibration projection");
    for row in manifest
        .rows
        .iter()
        .filter(|row| row.split_role.is_sealed())
    {
        assert!(!sealed.contains(&row.audio.sha256));
        assert!(!fit.contains(&row.row_id));
        assert!(!calibration.contains(&row.row_id));
    }
}

#[test]
fn every_forbidden_cross_role_parent_and_identical_audio_leak_rejects() {
    let directory = TestDirectory::new();
    let manifest = write_manifest(&directory.path);
    let manifest_directory = directory.path.as_path();

    for mutate in 0..6 {
        let mut candidate = manifest.clone();
        match mutate {
            0 => candidate.rows[2].object_group_id = candidate.rows[0].object_group_id.clone(),
            1 => candidate.rows[2].source_group_id = candidate.rows[0].source_group_id.clone(),
            2 => {
                candidate.rows[2].recording_parent_id =
                    candidate.rows[0].recording_parent_id.clone();
            }
            3 => candidate.rows[2].audio = candidate.rows[0].audio.clone(),
            4 => {
                candidate.rows[2].condition_group_id = candidate.rows[0].condition_group_id.clone();
            }
            5 => {
                let parent = "shared-mutation-parent".to_owned();
                candidate.rows[0].mutation_parent_id = Some(parent.clone());
                candidate.rows[2].mutation_parent_id = Some(parent);
            }
            _ => unreachable!(),
        }
        let bytes = serde_json::to_vec_pretty(&candidate).expect("serialize candidate");
        let path = directory.path.join(format!("candidate-{mutate}.json"));
        fs::write(&path, bytes).expect("write candidate");
        let parsed: NeuralDataPlaneManifest =
            serde_json::from_slice(&fs::read(&path).expect("read candidate"))
                .expect("parse candidate");
        validate_manifest(&parsed).expect("shape remains valid");
        let error = build_projection(
            workspace_root(),
            manifest_directory,
            &parsed,
            &sha256_hex(&fs::read(path).expect("read candidate bytes")),
        )
        .err()
        .expect("cross-role leakage rejects");
        assert!(error.contains("crosses split roles"));
    }
}

#[test]
fn missing_axis_is_reported_and_never_fabricated() {
    let directory = TestDirectory::new();
    let mut manifest = write_manifest(&directory.path);
    let train_row = manifest
        .rows
        .iter_mut()
        .find(|row| row.split_role == SplitRole::Train)
        .expect("train row");
    train_row.axes.geometry = None;
    let incomplete_row_id = train_row.row_id.clone();
    validate_manifest(&manifest).expect("missing published axis remains a valid incomplete row");
    let bytes = serde_json::to_vec_pretty(&manifest).expect("serialize incomplete manifest");
    let built = build_projection(
        workspace_root(),
        &directory.path,
        &manifest,
        &sha256_hex(&bytes),
    )
    .expect("incomplete projection is reported");
    assert_eq!(built.report.decision, "DeclaredAxisCoverageIncomplete");
    assert_eq!(built.report.capability_counts.geometry, 9);
    assert_eq!(built.report.capability_counts.complete_modal_field, 9);
    let projected = serde_json::to_value(&built.fit_projection).expect("serialize projection");
    let row = projected["rows"]
        .as_array()
        .expect("projected rows")
        .iter()
        .find(|row| row["row_id"] == incomplete_row_id)
        .expect("incomplete row is projected");
    assert!(row["axes"].get("geometry").is_none());
}

#[test]
fn invalid_normal_missing_excitation_and_unsealed_policy_reject() {
    let directory = TestDirectory::new();
    let manifest = write_manifest(&directory.path);

    let mut invalid_normal = manifest.clone();
    invalid_normal.rows[0]
        .axes
        .impact
        .as_mut()
        .expect("impact claim")
        .outward_normal = [0.0, 0.0, 0.0];
    assert!(
        validate_manifest(&invalid_normal)
            .expect_err("invalid normal rejects")
            .contains("squared norm")
    );

    let mut empty_excitation = manifest.clone();
    let excitation = empty_excitation.rows[0]
        .axes
        .excitation
        .as_mut()
        .expect("excitation claim");
    excitation.impulse_newton_seconds = None;
    excitation.energy_joules = None;
    excitation.force_profile = None;
    assert!(
        validate_manifest(&empty_excitation)
            .expect_err("empty excitation rejects")
            .contains("requires impulse")
    );

    let mut unsealed = manifest;
    unsealed
        .split_policy
        .admission_shadow_sealed_until_validator_release = false;
    assert!(
        validate_manifest(&unsealed)
            .expect_err("unsealed policy rejects")
            .contains("sealing rule")
    );

    let mut false_lineage = write_manifest(&directory.path);
    false_lineage.lineage_reports[0].expected_claim = "FALSE_CLAIM".to_owned();
    let bytes = serde_json::to_vec_pretty(&false_lineage).expect("serialize false lineage");
    let error = build_projection(
        workspace_root(),
        &directory.path,
        &false_lineage,
        &sha256_hex(&bytes),
    )
    .err()
    .expect("false lineage semantics reject");
    assert!(error.contains("does not match declared schema"));
}

fn write_manifest(directory: &Path) -> NeuralDataPlaneManifest {
    fs::write(
        directory.join("lineage.json"),
        br#"{"schema":"nextengine.experimental-test-registry.report.v1","status":"Validated","claim":"SYNTHETIC_TYPED_REGISTRY_FIXTURE_ONLY"}"#,
    )
    .expect("write lineage");
    fs::write(directory.join("evidence.json"), b"{\"fixture\":true}").expect("write evidence");
    fs::write(directory.join("provenance.txt"), b"synthetic test fixture")
        .expect("write provenance");
    let lineage = file_ref(directory, "lineage.json");
    let evidence = file_ref(directory, "evidence.json");
    let provenance = file_ref(directory, "provenance.txt");
    let mut rows = Vec::new();
    for (role_index, role) in SPLIT_ROLES.into_iter().enumerate() {
        let role_name = role.as_str();
        let geometry_name = format!("geometry-{role_name}.bin");
        fs::write(directory.join(&geometry_name), [role_index as u8, 1, 2, 3])
            .expect("write geometry fixture");
        let geometry = file_ref(directory, &geometry_name);
        for (sample_index, sample_role) in [SampleRole::Context, SampleRole::Query]
            .into_iter()
            .enumerate()
        {
            let audio_name = format!("audio-{role_name}-{sample_index}.pcm");
            fs::write(
                directory.join(&audio_name),
                [role_index as u8, sample_index as u8, 7, 11],
            )
            .expect("write audio fixture");
            rows.push(NeuralRow {
                row_id: format!("row-{role_name}-{sample_index}"),
                split_role: role,
                sample_role,
                corpus_role: CorpusRole::Target,
                source_group_id: format!("source-{role_name}"),
                family_group_id: "thin-glass-vessel".to_owned(),
                object_group_id: format!("object-{role_name}"),
                recording_parent_id: format!("recording-{role_name}-{sample_index}"),
                condition_group_id: format!("condition-{role_name}-{sample_index}"),
                mutation_parent_id: None,
                lineage_report_ids: vec!["typed-registry-report".to_owned()],
                audio: file_ref(directory, &audio_name),
                audio_provenance: provenance.clone(),
                axes: complete_axes(role_index, sample_index, geometry.clone(), evidence.clone()),
            });
        }
    }
    rows.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    NeuralDataPlaneManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        projection_id: "few-shot-fixture".to_owned(),
        revision: "v1".to_owned(),
        task_scope: TaskScope::ExactObjectFewShotImpactListenerField,
        split_policy: SplitPolicy {
            object_groups_disjoint_across_roles: true,
            source_groups_disjoint_across_roles: true,
            recording_parents_disjoint_across_roles: true,
            condition_groups_disjoint_across_roles: true,
            mutation_parents_disjoint_across_roles: true,
            identical_audio_disjoint_across_roles: true,
            method_holdout_sealed_before_candidate_freeze: true,
            admission_shadow_sealed_until_validator_release: true,
        },
        lineage_reports: vec![LineageReport {
            id: "typed-registry-report".to_owned(),
            expected_schema: "nextengine.experimental-test-registry.report.v1".to_owned(),
            expected_claim: "SYNTHETIC_TYPED_REGISTRY_FIXTURE_ONLY".to_owned(),
            artifact: lineage,
        }],
        rows,
    }
}

fn complete_axes(
    role_index: usize,
    sample_index: usize,
    geometry: FileRef,
    evidence: FileRef,
) -> AxisClaims {
    AxisClaims {
        material: Some(LabelClaim {
            value_id: "glass".to_owned(),
            evidence: evidence.clone(),
        }),
        geometry: Some(GeometryClaim {
            geometry_id: format!("geometry-{role_index}"),
            feature_artifact: geometry,
            evidence: evidence.clone(),
        }),
        support: Some(LabelClaim {
            value_id: "freely-supported".to_owned(),
            evidence: evidence.clone(),
        }),
        impact: Some(ImpactClaim {
            coordinate_profile: "right-handed-metres".to_owned(),
            point_metres: [sample_index as f64 * 0.01, 0.0, 0.0],
            outward_normal: [0.0, 0.0, 1.0],
            evidence: evidence.clone(),
        }),
        listener: Some(ListenerClaim {
            coordinate_profile: "right-handed-metres".to_owned(),
            point_metres: [0.0, role_index as f64 * 0.1, 1.0],
            evidence: evidence.clone(),
        }),
        excitation: Some(ExcitationClaim {
            impulse_newton_seconds: Some(0.1 + sample_index as f64 * 0.01),
            energy_joules: None,
            force_profile: None,
            evidence,
        }),
    }
}

fn file_ref(directory: &Path, name: &str) -> FileRef {
    let bytes = fs::read(directory.join(name)).expect("read fixture artifact");
    FileRef {
        path: name.to_owned(),
        sha256: sha256_hex(&bytes),
    }
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}
