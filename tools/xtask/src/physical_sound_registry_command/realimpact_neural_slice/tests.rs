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
            "nextengine-realimpact-neural-slice-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create neural-slice test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path).expect("remove neural-slice test directory");
        }
    }
}

#[test]
fn shared_scale_preserves_relative_listener_amplitude_and_repeats() {
    let directory = TestDirectory::new();
    write_fixture(&directory.path);
    let first = directory.path.join("first");
    run(
        workspace_root(),
        &Request {
            listener_manifest: directory.path.join("manifest.json"),
            acquisition_report: directory.path.join("acquisition-report.json"),
            output: first.clone(),
        },
    )
    .expect("first neural slice validates");
    let second = directory.path.join("second");
    run(
        workspace_root(),
        &Request {
            listener_manifest: directory.path.join("manifest.json"),
            acquisition_report: directory.path.join("acquisition-report.json"),
            output: second.clone(),
        },
    )
    .expect("second neural slice validates");

    for file in [
        "report.json",
        "slice.json",
        "row-0000-mic-00.relative-transfer.wav",
        "row-0001-mic-01.relative-transfer.wav",
    ] {
        assert_eq!(
            fs::read(first.join(file)).expect("read first output"),
            fs::read(second.join(file)).expect("read second output")
        );
    }
    let quiet =
        fs::read(first.join("row-0000-mic-00.relative-transfer.wav")).expect("read quiet WAV");
    let loud =
        fs::read(first.join("row-0001-mic-01.relative-transfer.wav")).expect("read loud WAV");
    assert_eq!(i16::from_le_bytes([quiet[44], quiet[45]]), 7_782);
    assert_eq!(i16::from_le_bytes([loud[44], loud[45]]), 15_564);

    let report: Value =
        serde_json::from_slice(&fs::read(first.join("report.json")).expect("read slice report"))
            .expect("parse slice report");
    assert_eq!(report["decision"], "RelativeTransferSliceAvailable");
    assert_eq!(report["row_count"], 2);
    assert_eq!(report["shared_raw_peak_abs"], 2.0);
    assert_eq!(report["relative_amplitude_preserved"], true);
    assert_eq!(report["model_training_authorized"], false);
}

#[test]
fn stale_row_and_nonfinite_payload_fail_before_output() {
    let directory = TestDirectory::new();
    write_fixture(&directory.path);
    let manifest_path = directory.path.join("manifest.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read listener manifest"))
            .expect("parse listener manifest");
    manifest["rows"][0]["raw_f32le_sha256"] = json!("a".repeat(64));
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize stale manifest"),
    )
    .expect("write stale manifest");
    rewrite_report_hash(&directory.path);
    let output = directory.path.join("stale-output");
    let error = run(
        workspace_root(),
        &Request {
            listener_manifest: manifest_path.clone(),
            acquisition_report: directory.path.join("acquisition-report.json"),
            output: output.clone(),
        },
    )
    .expect_err("stale row rejects");
    assert!(error.contains("row 0 hash changed"));
    assert!(!output.exists());

    write_fixture(&directory.path);
    let mut block = fs::read(directory.path.join("block.f32le")).expect("read block");
    block[..4].copy_from_slice(&f32::NAN.to_le_bytes());
    fs::write(directory.path.join("block.f32le"), &block).expect("write nonfinite block");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read refreshed manifest"))
            .expect("parse refreshed manifest");
    manifest["block_payload"]["sha256"] = json!(sha256_hex(&block));
    manifest["rows"][0]["raw_f32le_sha256"] = json!(sha256_hex(&block[..16]));
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize nonfinite manifest"),
    )
    .expect("write nonfinite manifest");
    rewrite_report_hash(&directory.path);
    let output = directory.path.join("nonfinite-output");
    let error = run(
        workspace_root(),
        &Request {
            listener_manifest: manifest_path,
            acquisition_report: directory.path.join("acquisition-report.json"),
            output: output.clone(),
        },
    )
    .expect_err("nonfinite sample rejects");
    assert!(error.contains("non-finite sample"));
    assert!(!output.exists());
}

fn write_fixture(directory: &Path) {
    for child in ["first", "second", "stale-output", "nonfinite-output"] {
        let path = directory.join(child);
        if path.is_dir() {
            fs::remove_dir_all(path).expect("remove prior output");
        }
    }
    let rows = [[0.5_f32, -0.25, 1.0, 0.0], [1.0_f32, -2.0, 0.5, 0.0]];
    let block = rows
        .iter()
        .flatten()
        .flat_map(|sample| sample.to_le_bytes())
        .collect::<Vec<_>>();
    fs::write(directory.join("block.f32le"), &block).expect("write listener block");
    let manifest = json!({
        "schema": SOURCE_MANIFEST_SCHEMA,
        "status": "development_pilot_partial_source",
        "profile": "fixture-listener-block-v1",
        "source_repository_revision": "commit-fixture",
        "dataset_object_id": "fixture-object",
        "object_id": "fixture-object",
        "geometry_revision": "fixture-geometry-v1",
        "impact_position_id": "fixture-point",
        "impact_position_metres": [0.0, 0.0, 0.0],
        "sample_rate_hz": 48000,
        "sample_count_per_row": 4,
        "row_count": 2,
        "block_payload": {
            "path": "block.f32le",
            "sha256": sha256_hex(&block),
            "byte_count": block.len()
        },
        "rows": [
            {
                "row_index": 0,
                "microphone_id": 0,
                "listener_condition_id": "mic-00",
                "listener_position_metres": [0.0, 0.0, -1.0],
                "payload_offset_bytes": 0,
                "sample_count": 4,
                "raw_f32le_sha256": sha256_hex(&block[..16]),
                "peak_abs": 1.0
            },
            {
                "row_index": 1,
                "microphone_id": 1,
                "listener_condition_id": "mic-01",
                "listener_position_metres": [0.0, 0.0, 1.0],
                "payload_offset_bytes": 16,
                "sample_count": 4,
                "raw_f32le_sha256": sha256_hex(&block[16..]),
                "peak_abs": 2.0
            }
        ]
    });
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize listener manifest"),
    )
    .expect("write listener manifest");
    rewrite_report_hash(directory);
}

fn rewrite_report_hash(directory: &Path) {
    let manifest = fs::read(directory.join("manifest.json")).expect("read listener manifest");
    let report = json!({
        "schema": SOURCE_REPORT_SCHEMA,
        "status": "Validated",
        "claim": SOURCE_REPORT_CLAIM,
        "profile": "fixture-listener-block-v1",
        "manifest_sha256": sha256_hex(&manifest),
        "row_count": 2,
        "same_impact_vertex": true
    });
    fs::write(
        directory.join("acquisition-report.json"),
        serde_json::to_vec_pretty(&report).expect("serialize acquisition report"),
    )
    .expect("write acquisition report");
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}
