use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::native_gate_runner::{
    NativeGateMatrixSchedule, complete_native_gate_failure_records, ensure_clean_worktree,
    native_gate_repository_identity, native_gate_state_root, native_shipping_target_for_host,
    publish_native_gate_failure, run_with_identity_verification, validate_named_target_slots,
    verify_native_gate_repository_identity, wsl_markers_present,
};
use crate::*;

static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "nextengine-xtask-{label}-{}-{sequence}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove stale test directory");
        }
        fs::create_dir(&path).expect("create test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn native_gate_cli_requires_exact_flags_and_rejects_duplicates() {
    let run = parse_native_gate_run_arguments(
        ["--output", "artifacts/native-gate/commit"]
            .into_iter()
            .map(str::to_owned),
    )
    .expect("run arguments");
    assert_eq!(run, PathBuf::from("artifacts/native-gate/commit"));
    assert!(
        parse_native_gate_run_arguments(
            ["--output", "one", "extra"].into_iter().map(str::to_owned)
        )
        .is_err()
    );

    let compare = parse_native_gate_compare_arguments(
        [
            "--linux",
            "linux/target-report.json",
            "--output",
            "cross-target-report.json",
            "--windows",
            "windows/target-report.json",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("compare arguments");
    assert_eq!(compare.windows, PathBuf::from("windows/target-report.json"));
    assert!(
        parse_native_gate_compare_arguments(
            [
                "--windows",
                "one",
                "--windows",
                "two",
                "--linux",
                "linux",
                "--output",
                "output",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap_err()
        .starts_with("duplicate argument")
    );
    assert!(
        parse_native_gate_compare_arguments(
            ["--windows", "one", "--linux", "two"]
                .into_iter()
                .map(str::to_owned),
        )
        .unwrap_err()
        .contains("--output")
    );
    assert!(
        parse_native_gate_compare_arguments(
            ["--linux", "two", "--output", "out"]
                .into_iter()
                .map(str::to_owned)
        )
        .unwrap_err()
        .contains("--windows")
    );
    assert!(
        parse_native_gate_compare_arguments(
            ["--windows", "one", "--output", "out"]
                .into_iter()
                .map(str::to_owned)
        )
        .unwrap_err()
        .contains("--linux")
    );
}

#[test]
fn comparator_named_slots_reject_swapped_target_reports() {
    let windows = slot_report(WINDOWS_TARGET_TRIPLE);
    let linux = slot_report(LINUX_TARGET_TRIPLE);
    validate_named_target_slots(&windows, &linux).expect("correct named slots");
    assert!(
        validate_named_target_slots(&linux, &windows)
            .expect_err("swapped slots")
            .starts_with("NATIVE_GATE_TARGET_SET_INVALID")
    );
}

fn slot_report(target: &str) -> NativeGateTargetReportV1 {
    NativeGateTargetReportV1 {
        schema_version: xtask::native_gate::LEGACY_NATIVE_GATE_SCHEMA_VERSION,
        status: NativeGateRunStatusV1::Fail,
        git_commit_sha: "1".repeat(40),
        cargo_lock_sha256: "2".repeat(64),
        rustc_release: "1.97.1".to_owned(),
        target_triple: target.to_owned(),
        checks: Vec::new(),
        closure_targets: None,
        comparable_roots: None,
        package: None,
    }
}

#[test]
fn native_gate_state_roots_are_check_local_and_under_staging() {
    let staging = Path::new("native-gate-staging");
    let roots = NativeGateCheckNameV1::ORDERED
        .into_iter()
        .map(|check| native_gate_state_root(staging, check))
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 8);
    for root in &roots {
        assert!(root.starts_with(staging.join(".state")));
        assert!(!root.is_absolute());
    }
    let unique = roots.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), roots.len());
}

#[test]
fn matrix_scheduler_runs_fixed_order_once_with_fresh_isolated_state_roots() {
    let directory = TestDirectory::new("matrix-schedule-pass");
    let staging = directory.path.join("staging");
    fs::create_dir(&staging).expect("staging");
    fs::create_dir(staging.join(".state")).expect("state staging");
    let mut schedule = NativeGateMatrixSchedule::new(&staging);
    let mut invocations = Vec::new();
    let mut roots = Vec::new();

    for check in NativeGateCheckNameV1::ORDERED {
        let result = schedule.run(check, |state_root, _started| {
            assert!(state_root.is_dir(), "scheduler creates state root first");
            assert!(
                fs::read_dir(state_root)
                    .expect("read fresh state root")
                    .next()
                    .is_none(),
                "new check receives an empty state root"
            );
            invocations.push(check);
            roots.push(state_root.to_path_buf());
            Ok(((), pass_record(check)))
        });
        assert!(result.is_ok(), "{} should pass", check.as_str());
    }

    let records = match schedule.finish() {
        Ok(records) => records,
        Err(_) => panic!("completed schedule should finish"),
    };
    assert_eq!(invocations, NativeGateCheckNameV1::ORDERED);
    assert_eq!(
        records
            .iter()
            .map(|record| record.check)
            .collect::<Vec<_>>(),
        NativeGateCheckNameV1::ORDERED
    );
    assert_eq!(
        roots
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        NativeGateCheckNameV1::ORDERED.len()
    );
    assert!(
        roots
            .iter()
            .all(|root| root.starts_with(staging.join(".state")))
    );
}

#[test]
fn matrix_scheduler_rejects_reordering_and_never_retries_after_failure() {
    let wrong_order_directory = TestDirectory::new("matrix-schedule-order");
    let wrong_order_staging = wrong_order_directory.path.join("staging");
    fs::create_dir(&wrong_order_staging).expect("staging");
    fs::create_dir(wrong_order_staging.join(".state")).expect("state staging");
    let wrong_order_invoked = std::cell::Cell::new(false);
    let mut wrong_order = NativeGateMatrixSchedule::new(&wrong_order_staging);
    let order_failure = wrong_order
        .run::<()>(NativeGateCheckNameV1::Play, |_state_root, _started| {
            wrong_order_invoked.set(true);
            Ok(((), pass_record(NativeGateCheckNameV1::Play)))
        })
        .expect_err("out-of-order check must fail before execution");
    assert!(!wrong_order_invoked.get());
    assert_eq!(order_failure.records.len(), 1);
    assert_eq!(
        order_failure.records[0].check,
        NativeGateCheckNameV1::HostCheck
    );
    assert_eq!(
        order_failure.records[0].status,
        NativeGateCheckStatusV1::Fail
    );

    let directory = TestDirectory::new("matrix-schedule-failure");
    let staging = directory.path.join("staging");
    fs::create_dir(&staging).expect("staging");
    fs::create_dir(staging.join(".state")).expect("state staging");
    let mut schedule = NativeGateMatrixSchedule::new(&staging);
    let mut invocations = Vec::new();
    assert!(
        schedule
            .run(NativeGateCheckNameV1::HostCheck, |state_root, _started| {
                assert!(state_root.is_dir());
                invocations.push(NativeGateCheckNameV1::HostCheck);
                Ok(((), pass_record(NativeGateCheckNameV1::HostCheck)))
            })
            .is_ok()
    );
    let failure = schedule
        .run::<()>(NativeGateCheckNameV1::Play, |state_root, _started| {
            assert!(state_root.is_dir());
            invocations.push(NativeGateCheckNameV1::Play);
            Err(Box::new(NativeGateCheckExecutionFailure {
                record: fail_record(NativeGateCheckNameV1::Play),
                error: "NATIVE_GATE_CHECK_FAILED: play failed".to_owned(),
            }))
        })
        .expect_err("play failure stops the schedule");
    assert_eq!(
        invocations,
        [
            NativeGateCheckNameV1::HostCheck,
            NativeGateCheckNameV1::Play
        ]
    );
    assert_eq!(failure.records.len(), 2);

    let retry_invoked = std::cell::Cell::new(false);
    let retry_failure = schedule
        .run::<()>(NativeGateCheckNameV1::Play, |_state_root, _started| {
            retry_invoked.set(true);
            Ok(((), pass_record(NativeGateCheckNameV1::Play)))
        })
        .expect_err("failed check must not be retried");
    let tail_invoked = std::cell::Cell::new(false);
    let tail_failure = schedule
        .run::<()>(
            NativeGateCheckNameV1::PersistenceReplay,
            |_state_root, _started| {
                tail_invoked.set(true);
                Ok(((), pass_record(NativeGateCheckNameV1::PersistenceReplay)))
            },
        )
        .expect_err("tail check must not execute after failure");
    assert!(!retry_invoked.get());
    assert!(!tail_invoked.get());
    assert_eq!(retry_failure.records, failure.records);
    assert_eq!(tail_failure.records, failure.records);
}

#[test]
fn host_check_preserves_standalone_arguments_and_enables_only_desktop_gate_features() {
    assert_eq!(
        host_check_clippy_arguments(None, false),
        vec![
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings"
        ]
    );
    assert_eq!(
        host_check_test_arguments(None, false),
        vec!["test", "--workspace"]
    );

    let state_root = Path::new("isolated-state");
    assert_eq!(
        host_check_clippy_arguments(Some(state_root), true),
        vec![
            "clippy",
            "--locked",
            "--workspace",
            "--all-targets",
            "--features",
            "xtask/desktop-sdl-ash,next_verification/desktop-sdl-ash,next_game/desktop-sdl-ash",
            "--",
            "-D",
            "warnings"
        ]
    );
    assert_eq!(
        host_check_test_arguments(Some(state_root), true),
        vec![
            "test",
            "--locked",
            "--workspace",
            "--features",
            "xtask/desktop-sdl-ash,next_verification/desktop-sdl-ash,next_game/desktop-sdl-ash"
        ]
    );
    assert!(!host_check_clippy_arguments(Some(state_root), true).contains(&"--all-features"));
    assert!(!host_check_test_arguments(Some(state_root), true).contains(&"--all-features"));
}

#[test]
fn package_failure_prefixes_route_to_stable_native_gate_diagnostics() {
    for code in [
        "NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID",
        "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING",
        "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED",
        "NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT",
    ] {
        assert_eq!(
            diagnostic_code(&format!("{code}: controlled package failure")),
            code
        );
    }
}

#[test]
fn controlled_failure_stops_the_matrix_and_marks_the_tail_not_run() {
    let records = complete_native_gate_failure_records(&[
        pass_record(NativeGateCheckNameV1::HostCheck),
        fail_record(NativeGateCheckNameV1::Play),
    ])
    .expect("complete failure records");
    assert_eq!(records.len(), NativeGateCheckNameV1::ORDERED.len());
    assert_eq!(records[0].status, NativeGateCheckStatusV1::Pass);
    assert_eq!(records[1].status, NativeGateCheckStatusV1::Fail);
    assert!(records[2..].iter().all(|record| {
        record.status == NativeGateCheckStatusV1::NotRun
            && record
                .diagnostic
                .as_ref()
                .is_some_and(|diagnostic| diagnostic.code == "PRIOR_CHECK_FAILED")
            && record.elapsed_milliseconds == 0
    }));
}

#[test]
fn controlled_failure_bundle_is_published_atomically_and_collision_is_rejected() {
    let directory = TestDirectory::new("atomic-failure");
    let target_output = directory.path.join("target");
    let staging = directory.path.join(".target.staging");
    fs::create_dir(&staging).expect("staging");
    fs::create_dir(staging.join("checks")).expect("checks staging");
    fs::create_dir(staging.join(".state")).expect("state staging");
    fs::create_dir(staging.join(".package.staging-123")).expect("package staging");
    fs::create_dir(staging.join(".package.smoke-123")).expect("package smoke");
    fs::write(staging.join(".package.publish.lock"), b"123").expect("package lock");
    let identity = test_identity();
    publish_native_gate_failure(
        &staging,
        &target_output,
        &identity,
        &[fail_record(NativeGateCheckNameV1::HostCheck)],
    )
    .expect("publish controlled failure");
    assert!(!staging.exists());
    assert!(target_output.join("target-report.json").is_file());
    let report = xtask::native_gate::validate_native_gate_linux_bundle(
        &target_output.join("target-report.json"),
    )
    .expect("validate published failure");
    assert_eq!(report.status, NativeGateRunStatusV1::Fail);

    let collision_staging = directory.path.join(".collision.staging");
    fs::create_dir(&collision_staging).expect("collision staging");
    let error = publish_native_gate_failure(
        &collision_staging,
        &target_output,
        &identity,
        &[fail_record(NativeGateCheckNameV1::HostCheck)],
    )
    .expect_err("existing output must not be replaced");
    assert!(error.starts_with("NATIVE_GATE_OUTPUT_EXISTS"));
    assert!(collision_staging.exists());
}

#[test]
fn failure_publication_rejects_unknown_staging_entries() {
    let directory = TestDirectory::new("failure-extra");
    let target_output = directory.path.join("target");
    let staging = directory.path.join(".target.staging");
    fs::create_dir(&staging).expect("staging");
    fs::create_dir(staging.join("checks")).expect("checks");
    fs::write(staging.join("unexpected.tmp"), b"unexpected").expect("unexpected file");

    let error = publish_native_gate_failure(
        &staging,
        &target_output,
        &test_identity(),
        &[fail_record(NativeGateCheckNameV1::HostCheck)],
    )
    .expect_err("unknown staging entry must fail");
    assert!(error.starts_with("NATIVE_GATE_REPORT_INVALID"));
    assert!(!target_output.exists());
}

#[test]
fn operation_failure_still_runs_post_identity_and_identity_error_wins() {
    let verification_calls = std::cell::Cell::new(0_u8);
    let error = run_with_identity_verification(
        || {
            let call = verification_calls.get();
            verification_calls.set(call + 1);
            if call == 0 {
                Ok(())
            } else {
                Err("NATIVE_GATE_HEAD_CHANGED: identity changed".to_owned())
            }
        },
        || Err::<(), _>("NATIVE_GATE_CHECK_FAILED: operation failed".to_owned()),
    )
    .expect_err("post identity must take precedence");
    assert_eq!(verification_calls.get(), 2);
    assert!(error.starts_with("NATIVE_GATE_HEAD_CHANGED"));
}

#[test]
fn dirty_and_untracked_worktrees_are_rejected() {
    let directory = TestDirectory::new("dirty");
    initialize_test_repository(&directory.path);
    ensure_clean_worktree(&directory.path).expect("initial clean tree");

    fs::write(directory.path.join("untracked.tmp"), b"untracked").expect("untracked file");
    let error = ensure_clean_worktree(&directory.path).expect_err("untracked file is dirty");
    assert!(error.starts_with("NATIVE_GATE_WORKTREE_DIRTY"));
    fs::remove_file(directory.path.join("untracked.tmp")).expect("remove untracked file");

    fs::write(directory.path.join("tracked.txt"), b"changed").expect("change tracked file");
    let error = ensure_clean_worktree(&directory.path).expect_err("tracked file is dirty");
    assert!(error.starts_with("NATIVE_GATE_WORKTREE_DIRTY"));
}

#[test]
fn changed_head_is_rejected_against_the_start_identity() {
    let directory = TestDirectory::new("head-change");
    initialize_test_repository(&directory.path);
    let identity =
        native_gate_repository_identity(&directory.path, false).expect("initial identity");

    fs::write(directory.path.join("tracked.txt"), b"second commit").expect("change file");
    run_git(&directory.path, &["add", "--all"]);
    run_git(&directory.path, &["commit", "--quiet", "-m", "second"]);
    let error = verify_native_gate_repository_identity(&directory.path, &identity, false)
        .expect_err("changed HEAD");
    assert!(error.starts_with("NATIVE_GATE_HEAD_CHANGED"));
}

#[test]
fn unsupported_native_host_and_disabled_desktop_feature_have_stable_diagnostics() {
    let error =
        native_shipping_target_for_host("aarch64-unknown-linux-gnu").expect_err("unsupported");
    assert!(error.starts_with("NATIVE_GATE_UNSUPPORTED_TARGET"));
    let error = native_shipping_target_for_host(WINDOWS_TARGET_TRIPLE)
        .expect_err("Windows is outside the current release policy");
    assert!(error.starts_with("NATIVE_GATE_UNSUPPORTED_TARGET"));

    #[cfg(not(feature = "desktop-sdl-ash"))]
    {
        let error =
            crate::native_gate_runner::native_gate_run(Path::new("unused"), Path::new("unused"))
                .expect_err("feature is disabled");
        assert!(error.starts_with("NATIVE_GATE_DESKTOP_ADAPTER_DISABLED"));
    }
}

#[test]
fn wsl_markers_are_not_accepted_as_native_linux_evidence() {
    assert!(wsl_markers_present(true, false, "6.6.0-generic"));
    assert!(wsl_markers_present(false, true, "6.6.0-generic"));
    assert!(wsl_markers_present(
        false,
        false,
        "5.15.153.1-microsoft-standard-WSL2"
    ));
    assert!(!wsl_markers_present(false, false, "6.8.0-31-generic"));
}

fn pass_record(check: NativeGateCheckNameV1) -> NativeGateCheckRecordV1 {
    NativeGateCheckRecordV1 {
        check,
        status: NativeGateCheckStatusV1::Pass,
        report_path: Some(check.report_path()),
        report_sha256: Some("a".repeat(64)),
        diagnostic: None,
        elapsed_milliseconds: 1,
    }
}

fn fail_record(check: NativeGateCheckNameV1) -> NativeGateCheckRecordV1 {
    NativeGateCheckRecordV1 {
        check,
        status: NativeGateCheckStatusV1::Fail,
        report_path: None,
        report_sha256: None,
        diagnostic: Some(NativeGateDiagnosticV1 {
            code: "NATIVE_GATE_CHECK_FAILED".to_owned(),
            message: format!("{} failed", check.as_str()),
        }),
        elapsed_milliseconds: 1,
    }
}

fn test_identity() -> NativeGateIdentity {
    NativeGateIdentity {
        git_commit: "1".repeat(40),
        git_object_format: "sha1".to_owned(),
        cargo_lock_sha256: "2".repeat(64),
        rustc_release: "1.97.1".to_owned(),
        rustc_host: LINUX_TARGET_TRIPLE.to_owned(),
        target_triple: LINUX_TARGET_TRIPLE.to_owned(),
    }
}

fn initialize_test_repository(path: &Path) {
    run_git(path, &["init", "--quiet", "--object-format=sha1"]);
    run_git(path, &["config", "user.name", "NextEngine Test"]);
    run_git(
        path,
        &["config", "user.email", "nextengine@example.invalid"],
    );
    run_git(path, &["config", "commit.gpgsign", "false"]);
    fs::write(path.join("Cargo.lock"), b"version = 4\n").expect("Cargo.lock");
    fs::write(path.join("tracked.txt"), b"initial").expect("tracked file");
    run_git(path, &["add", "--all"]);
    run_git(path, &["commit", "--quiet", "-m", "initial"]);
}

fn run_git(path: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(path)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}
