#![forbid(unsafe_code)]

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn game_reports_invalid_cli_as_a_typed_diagnostic() {
    let result = Command::new(env!("CARGO_BIN_EXE_next_game"))
        .arg("--unsupported")
        .output()
        .expect("run game");

    assert!(!result.status.success());
    let report: next_application::DiagnosticReportV1 =
        serde_json::from_slice(&result.stdout).expect("typed diagnostic");
    assert_eq!(report.status, "ERROR");
    assert_eq!(report.code, "CLI_ARGUMENT_INVALID");
}

#[test]
fn game_activates_the_same_exact_cooked_project_as_headless() {
    let (output, lock) = cooked_store("exact-project");
    let state_root = output.join("state");
    let result = Command::new(env!("CARGO_BIN_EXE_next_game"))
        .args([
            "--project",
            output.to_str().expect("UTF-8 temporary path"),
            "--lock",
            &lock,
            "--state-root",
            state_root.to_str().expect("UTF-8 state path"),
        ])
        .output()
        .expect("run game");
    cleanup(&output);

    assert!(
        result.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: next_application::RunReportV1 =
        serde_json::from_slice(&result.stdout).expect("typed run report");
    assert_eq!(report.status, "PASS");
    assert_eq!(report.project_composition_lock_hash, lock);
}

#[test]
fn game_rejects_project_lock_mismatch_before_session_bootstrap() {
    let (output, _) = cooked_store("lock-mismatch");
    let state_root = output.join("state");
    let mismatch = "00".repeat(32);
    let result = Command::new(env!("CARGO_BIN_EXE_next_game"))
        .args([
            "--project",
            output.to_str().expect("UTF-8 temporary path"),
            "--lock",
            &mismatch,
            "--state-root",
            state_root.to_str().expect("UTF-8 state path"),
        ])
        .output()
        .expect("run game");
    cleanup(&output);

    assert!(!result.status.success());
    let report: next_application::DiagnosticReportV1 =
        serde_json::from_slice(&result.stdout).expect("typed diagnostic");
    assert_eq!(report.code, "PROJECT_LOCK_MISMATCH");
}

fn cooked_store(test_name: &str) -> (std::path::PathBuf, String) {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let output = std::env::temp_dir().join(format!(
        "nextengine-game-{test_name}-{}-{counter}",
        std::process::id()
    ));
    let cooked =
        next_project::cook_project_v2(next_reference_game::project_source_v2().expect("source"))
            .expect("cook");
    let store = ContentStore::new(&output);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    (output, activated.project_lock.project_lock_sha256.to_hex())
}

fn cleanup(output: &std::path::Path) {
    if output.exists() {
        std::fs::remove_dir_all(output).expect("cleanup");
    }
}
