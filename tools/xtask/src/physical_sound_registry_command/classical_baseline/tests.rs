use std::sync::atomic::{AtomicU64, Ordering};

use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};
use serde_json::{Value, json};

use super::*;

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-physical-sound-classical-baseline-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create classical baseline test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path).expect("remove classical baseline test directory");
        }
    }
}

#[test]
fn exact_q30_export_repeats_and_preserves_frozen_pcm() {
    let directory = TestDirectory::new();
    let wav = selected_wav(65_536);
    fs::write(directory.path.join("reference.wav"), &wav).expect("write reference WAV");
    write_projection(
        &directory.path,
        &[projection_row(
            "row-development-query",
            &wav,
            true,
            "target",
        )],
    );
    write_manifest(
        &directory.path,
        vec![binding(
            &directory.path,
            "row-development-query",
            BaselineProfile::SelectedThinContainerQ30V1,
        )],
    );

    let first = directory.path.join("first");
    run(
        workspace_root(),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: first.clone(),
        },
    )
    .expect("first classical baseline export validates");
    let second = directory.path.join("second");
    run(
        workspace_root(),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: second.clone(),
        },
    )
    .expect("second classical baseline export validates");

    for file in [
        "classical-profile.json",
        "baseline-records.json",
        "report.json",
        "row-development-query.classical.wav",
    ] {
        assert_eq!(
            fs::read(first.join(file)).expect("read first baseline output"),
            fs::read(second.join(file)).expect("read second baseline output")
        );
    }
    assert_eq!(
        fs::read(first.join("row-development-query.classical.wav"))
            .expect("read rendered baseline"),
        wav
    );
    let report: Value = serde_json::from_slice(
        &fs::read(first.join("report.json")).expect("read classical report"),
    )
    .expect("parse classical report");
    assert_eq!(report["status"], "Validated");
    assert_eq!(report["decision"], "ClassicalBaselineAvailable");
    assert_eq!(report["baseline_available_rows"], 1);
    assert_eq!(report["exact_wav_match_rows"], 1);
    assert_eq!(report["model_training_authorized"], false);
    assert_eq!(report["quality_or_admission_authorized"], false);
    let profile: Value = serde_json::from_slice(
        &fs::read(first.join("classical-profile.json")).expect("read classical profile"),
    )
    .expect("parse classical profile");
    assert_eq!(profile["modes"].as_array().expect("mode array").len(), 16);
    assert_eq!(profile["residual"]["sample_count"], 144);
    assert_eq!(
        profile["colored_dct_boundary"]["decision"],
        "FallbackOutOfDomain"
    );
    assert_eq!(sha256_hex(&wav), FROZEN_SELECTED_Q30_WAV_SHA256);
}

#[test]
fn incomplete_axis_dct_and_reject_parent_rows_fall_back_without_partial_pcm() {
    let directory = TestDirectory::new();
    let wav = selected_wav(65_536);
    fs::write(directory.path.join("reference.wav"), &wav).expect("write reference WAV");
    let mut transfer_row = projection_row("row-development-transfer", &wav, true, "target");
    transfer_row["audio_semantics"] = json!("force_deconvolved_transfer_response");
    let rows = [
        projection_row("row-development-dct", &wav, true, "target"),
        projection_row("row-development-incomplete", &wav, false, "target"),
        projection_row("row-development-reject-parent", &wav, true, "reject_parent"),
        transfer_row,
    ];
    write_projection(&directory.path, &rows);
    write_manifest(
        &directory.path,
        vec![
            binding(
                &directory.path,
                "row-development-dct",
                BaselineProfile::FrozenColoredDctResidualV1,
            ),
            binding(
                &directory.path,
                "row-development-incomplete",
                BaselineProfile::SelectedThinContainerQ30V1,
            ),
            binding(
                &directory.path,
                "row-development-reject-parent",
                BaselineProfile::SelectedThinContainerQ30V1,
            ),
            binding(
                &directory.path,
                "row-development-transfer",
                BaselineProfile::SelectedThinContainerQ30V1,
            ),
        ],
    );
    let output = directory.path.join("output");
    run(
        workspace_root(),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: output.clone(),
        },
    )
    .expect("fallback-only baseline export validates");

    let report: Value = serde_json::from_slice(
        &fs::read(output.join("report.json")).expect("read fallback report"),
    )
    .expect("parse fallback report");
    assert_eq!(report["decision"], "ClassicalBaselineUnavailable");
    assert_eq!(report["baseline_available_rows"], 0);
    assert_eq!(report["fallback_out_of_domain_rows"], 4);
    let records: Value = serde_json::from_slice(
        &fs::read(output.join("baseline-records.json")).expect("read fallback records"),
    )
    .expect("parse fallback records");
    for row in records["rows"].as_array().expect("record rows") {
        assert_eq!(row["decision"], "FallbackOutOfDomain");
        assert!(row.get("prediction").is_none());
    }
    assert!(
        fs::read_dir(&output)
            .expect("read fallback output")
            .all(|entry| !entry
                .expect("read output entry")
                .file_name()
                .to_string_lossy()
                .ends_with(".wav"))
    );
}

#[test]
fn stale_projection_audio_and_sealed_roles_fail_closed() {
    let directory = TestDirectory::new();
    let wav = selected_wav(65_536);
    fs::write(directory.path.join("reference.wav"), &wav).expect("write reference WAV");
    write_projection(
        &directory.path,
        &[projection_row(
            "row-development-query",
            &wav,
            true,
            "target",
        )],
    );
    write_manifest(
        &directory.path,
        vec![binding(
            &directory.path,
            "row-development-query",
            BaselineProfile::SelectedThinContainerQ30V1,
        )],
    );
    fs::write(directory.path.join("projection.json"), b"stale projection")
        .expect("replace projection");
    let error = run(
        workspace_root(),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: directory.path.join("stale-output"),
        },
    )
    .expect_err("stale projection rejects");
    assert!(error.contains("hash mismatch"));

    write_projection_with_roles(
        &directory.path,
        &[projection_row(
            "row-development-query",
            &wav,
            true,
            "target",
        )],
        &["method_holdout"],
    );
    write_manifest(
        &directory.path,
        vec![binding(
            &directory.path,
            "row-development-query",
            BaselineProfile::SelectedThinContainerQ30V1,
        )],
    );
    let error = run(
        workspace_root(),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: directory.path.join("sealed-output"),
        },
    )
    .expect_err("sealed role projection rejects");
    assert!(error.contains("unsealed train/development"));
}

fn selected_wav(energy_q16: u32) -> Vec<u8> {
    let profile = AudioMixProfileV1::stereo_baseline_v1().expect("baseline audio profile");
    encode_canonical_wav(&profile, &render_selected_glass_q30_impact(energy_q16))
}

fn projection_row(row_id: &str, audio: &[u8], complete_axes: bool, corpus_role: &str) -> Value {
    let mut axes = json!({
        "material": {"value_id":"glass"},
        "geometry": {"geometry_id":"fixture"},
        "support": {"value_id":"fixture"},
        "impact": {"point_metres":[0.0,0.0,0.0]},
        "listener": {"point_metres":[0.0,0.0,1.0]},
        "excitation": {"impulse_newton_seconds":0.1}
    });
    if !complete_axes {
        axes.as_object_mut()
            .expect("axes object")
            .remove("geometry");
    }
    json!({
        "row_id": row_id,
        "split_role": "development",
        "sample_role": "query",
        "corpus_role": corpus_role,
        "audio_semantics": "recorded_impact_waveform",
        "audio": {"sha256":sha256_hex(audio),"byte_count":audio.len()},
        "axes": axes
    })
}

fn write_projection(directory: &Path, rows: &[Value]) {
    write_projection_with_roles(directory, rows, &["train", "development"]);
}

fn write_projection_with_roles(directory: &Path, rows: &[Value], roles: &[&str]) {
    let projection = json!({
        "schema": PROJECTION_SCHEMA,
        "projection_id": "classical-fixture",
        "revision": "v1",
        "task_scope": TASK_SCOPE,
        "manifest_sha256": "a".repeat(64),
        "role_scope": roles,
        "lineage_reports": [],
        "rows": rows
    });
    fs::write(
        directory.join("projection.json"),
        serde_json::to_vec_pretty(&projection).expect("serialize projection"),
    )
    .expect("write projection");
}

fn write_manifest(directory: &Path, bindings: Vec<RowBinding>) {
    let projection_bytes = fs::read(directory.join("projection.json")).expect("read projection");
    let manifest = ClassicalBaselineManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        benchmark_id: "classical-fixture".to_owned(),
        revision: "v1".to_owned(),
        projection: FileRef {
            path: "projection.json".to_owned(),
            sha256: sha256_hex(&projection_bytes),
        },
        bindings,
    };
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize classical manifest"),
    )
    .expect("write classical manifest");
}

fn binding(directory: &Path, row_id: &str, profile: BaselineProfile) -> RowBinding {
    RowBinding {
        row_id: row_id.to_owned(),
        profile,
        energy_q16: 65_536,
        reference_audio: FileRef {
            path: "reference.wav".to_owned(),
            sha256: sha256_hex(
                &fs::read(directory.join("reference.wav")).expect("read reference WAV"),
            ),
        },
    }
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}
