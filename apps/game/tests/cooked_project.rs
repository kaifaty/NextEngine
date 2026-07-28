#![forbid(unsafe_code)]

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn game_activates_the_same_exact_cooked_project_as_headless() {
    let (output, lock) = cooked_store("exact-project");
    let result = Command::new(env!("CARGO_BIN_EXE_next_game"))
        .args([
            "--project",
            output.to_str().expect("UTF-8 temporary path"),
            "--lock",
            &lock,
        ])
        .output()
        .expect("run game");
    cleanup(&output);

    assert!(
        result.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8(result.stdout).expect("UTF-8 output");
    assert!(stdout.contains("\"status\":\"PASS\""));
    assert!(stdout.contains(&format!("\"project_lock\":\"{lock}\"")));
}

#[test]
fn game_rejects_project_lock_mismatch_before_session_bootstrap() {
    let (output, _) = cooked_store("lock-mismatch");
    let result = Command::new(env!("CARGO_BIN_EXE_next_game"))
        .args([
            "--project",
            output.to_str().expect("UTF-8 temporary path"),
            "--lock",
            &"00".repeat(32),
        ])
        .output()
        .expect("run game");
    cleanup(&output);

    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("project lock mismatch"),
        "stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn cooked_store(test_name: &str) -> (std::path::PathBuf, String) {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let output = std::env::temp_dir().join(format!(
        "nextengine-game-{test_name}-{}-{counter}",
        std::process::id()
    ));
    let cooked = next_project::cook_project_v1(
        next_project::neutral_vertical_slice_source_v1().expect("source"),
    )
    .expect("cook");
    let store = ContentStore::new(&output);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    (
        output,
        activated.composition_lock.composition_lock_sha256.to_hex(),
    )
}

fn cleanup(output: &std::path::Path) {
    if output.exists() {
        std::fs::remove_dir_all(output).expect("cleanup");
    }
}
