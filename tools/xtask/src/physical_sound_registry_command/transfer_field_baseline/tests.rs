use std::sync::atomic::{AtomicU64, Ordering};

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
            "nextengine-transfer-field-baseline-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create transfer baseline test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path).expect("remove transfer baseline test directory");
        }
    }
}

#[test]
fn interpolation_reconstructs_midpoint_and_repeats() {
    let directory = TestDirectory::new();
    write_fixture(&directory.path, true);
    let first = directory.path.join("first");
    run(
        workspace_root(),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: first.clone(),
        },
    )
    .expect("first transfer baseline validates");
    let second = directory.path.join("second");
    run(
        workspace_root(),
        &Request {
            manifest: directory.path.join("manifest.json"),
            output: second.clone(),
        },
    )
    .expect("second transfer baseline validates");

    for file in [
        "report.json",
        "predictions/query.nearest-listener.wav",
        "predictions/query.linear-listener-segment.wav",
    ] {
        assert_eq!(
            fs::read(first.join(file)).expect("read first output"),
            fs::read(second.join(file)).expect("read second output")
        );
    }
    let report: Value =
        serde_json::from_slice(&fs::read(first.join("report.json")).expect("read transfer report"))
            .expect("parse transfer report");
    assert_eq!(report["decision"], "FrozenTransferControlsAvailable");
    assert_eq!(report["context_row_count"], 2);
    assert_eq!(report["query_row_count"], 1);
    assert_eq!(
        report["rows"][0]["linear_segment"]["normalized_waveform_rmse_db"],
        DB_FLOOR
    );
    assert_eq!(report["model_training_authorized"], false);
    assert_eq!(report["admission_shadow_opened"], false);
}

#[test]
fn stale_binding_and_unbracketed_query_fail_before_output() {
    let directory = TestDirectory::new();
    write_fixture(&directory.path, true);
    let manifest_path = directory.path.join("manifest.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read transfer manifest"))
            .expect("parse transfer manifest");
    manifest["bindings"][0]["audio"]["sha256"] = json!("a".repeat(64));
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize stale manifest"),
    )
    .expect("write stale manifest");
    let output = directory.path.join("stale-output");
    let error = run(
        workspace_root(),
        &Request {
            manifest: manifest_path.clone(),
            output: output.clone(),
        },
    )
    .expect_err("stale binding rejects");
    assert!(error.contains("hash mismatch"));
    assert!(!output.exists());

    write_fixture(&directory.path, false);
    let output = directory.path.join("unbracketed-output");
    let error = run(
        workspace_root(),
        &Request {
            manifest: manifest_path,
            output: output.clone(),
        },
    )
    .expect_err("unbracketed query rejects");
    assert!(error.contains("not bracketed"));
    assert!(!output.exists());
}

fn write_fixture(directory: &Path, bracketed: bool) {
    for child in ["first", "second", "stale-output", "unbracketed-output"] {
        let path = directory.join(child);
        if path.is_dir() {
            fs::remove_dir_all(path).expect("remove old output");
        }
    }
    let left = encode_pcm16_mono_wav(&[0.125, 0.25, -0.125, -0.25], 48_000).expect("encode left");
    let right = encode_pcm16_mono_wav(&[0.375, 0.5, -0.375, -0.5], 48_000).expect("encode right");
    let query_samples = if bracketed {
        [0.25, 0.375, -0.25, -0.375]
    } else {
        [0.5, 0.625, -0.5, -0.625]
    };
    let query = encode_pcm16_mono_wav(&query_samples, 48_000).expect("encode query");
    for (name, bytes) in [
        ("left.wav", &left),
        ("query.wav", &query),
        ("right.wav", &right),
    ] {
        fs::write(directory.join(name), bytes).expect("write fixture WAV");
    }
    let query_z = if bracketed { 0.0 } else { 2.0 };
    let rows = [
        row("left", CONTEXT_ROLE, -1.0, &left),
        row("query", QUERY_ROLE, query_z, &query),
        row("right", CONTEXT_ROLE, 1.0, &right),
    ];
    let projection = json!({
        "schema": PROJECTION_SCHEMA,
        "projection_id": "fixture-projection",
        "revision": "v1",
        "task_scope": TASK_SCOPE,
        "manifest_sha256": "fixture-source-manifest",
        "role_scope": ["train", DEVELOPMENT_ROLE],
        "lineage_reports": [],
        "rows": rows
    });
    let projection_bytes = serde_json::to_vec_pretty(&projection).expect("serialize projection");
    fs::write(directory.join("projection.json"), &projection_bytes).expect("write projection");
    let bindings = [
        binding("left", "left.wav", &left),
        binding("query", "query.wav", &query),
        binding("right", "right.wav", &right),
    ];
    let manifest = json!({
        "schema": MANIFEST_SCHEMA,
        "baseline_id": "fixture-transfer-baseline",
        "revision": "v1",
        "projection": {
            "path": "projection.json",
            "sha256": sha256_hex(&projection_bytes)
        },
        "bindings": bindings
    });
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
}

fn row(id: &str, sample_role: &str, listener_z: f64, wav: &[u8]) -> Value {
    json!({
        "row_id": id,
        "split_role": DEVELOPMENT_ROLE,
        "sample_role": sample_role,
        "corpus_role": TARGET_ROLE,
        "audio_semantics": TRANSFER_SEMANTICS,
        "source_group_id": "fixture-source",
        "family_group_id": "fixture-family",
        "object_group_id": "fixture-object",
        "recording_parent_id": "fixture-impact",
        "condition_group_id": format!("fixture-{id}"),
        "lineage_report_ids": ["fixture-lineage"],
        "audio": {
            "sha256": sha256_hex(wav),
            "byte_count": wav.len()
        },
        "audio_provenance": {
            "sha256": "b".repeat(64),
            "byte_count": 1
        },
        "axes": {
            "impact": {
                "coordinate_profile": "fixture-coordinate-v1",
                "point_metres": [0.0, 0.0, 0.0],
                "evidence": {"sha256": "c".repeat(64), "byte_count": 1}
            },
            "listener": {
                "coordinate_profile": "fixture-coordinate-v1",
                "point_metres": [0.0, 0.0, listener_z],
                "evidence": {"sha256": "c".repeat(64), "byte_count": 1}
            }
        }
    })
}

fn binding(id: &str, path: &str, wav: &[u8]) -> Value {
    json!({
        "row_id": id,
        "audio": {"path": path, "sha256": sha256_hex(wav)}
    })
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}
