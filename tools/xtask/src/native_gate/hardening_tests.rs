use std::fs;
use std::path::Path;

use super::tests::{
    TempBundle, controlled_fail_report, hash, materialize_check_reports, report,
    write_target_report,
};
use super::*;

#[test]
fn typed_check_validation_rejects_arbitrary_fail_and_wrong_command_json() {
    for (label, mutation) in [
        ("arbitrary", CheckMutation::Arbitrary),
        ("fail", CheckMutation::FailStatus),
        ("wrong-command", CheckMutation::WrongCommand),
    ] {
        let bundle = TempBundle::new();
        let mut target = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
        materialize_check_reports(&bundle, &mut target);
        let host_path = bundle.root.join(
            target.checks[0]
                .report_path
                .as_ref()
                .expect("host report path"),
        );
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&host_path).expect("read host report"))
                .expect("host report JSON");
        match mutation {
            CheckMutation::Arbitrary => value = serde_json::json!({"schema_version": 1}),
            CheckMutation::FailStatus => value["status"] = serde_json::json!("FAIL"),
            CheckMutation::WrongCommand => value["command"] = serde_json::json!("play"),
        }
        let bytes = serde_json::to_vec(&value).expect("serialize mutation");
        fs::write(&host_path, &bytes).expect("write mutation");
        target.checks[0].report_sha256 = Some(sha256_hex(&bytes));
        write_target_report(&bundle, &target);

        let error = validate_native_gate_target_bundle(&bundle.report_path())
            .expect_err("typed check mutation must fail");
        assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID, "{label}");
        assert!(
            error.detail().contains("host-check") || error.detail().contains("host"),
            "{label}: {error}"
        );
    }
}

#[derive(Clone, Copy)]
enum CheckMutation {
    Arbitrary,
    FailStatus,
    WrongCommand,
}

#[test]
fn comparable_roots_are_bound_to_typed_check_reports() {
    let bundle = TempBundle::new();
    let mut target = report(LINUX_TARGET_TRIPLE);
    materialize_check_reports(&bundle, &mut target);
    fs::create_dir(bundle.root.join("package")).expect("package placeholder");
    {
        let roots = target.comparable_roots.as_mut().expect("comparable roots");
        roots.streaming_performance_hash = hash('f');
        roots.windows_package_descriptor_hash =
            target_package_descriptor_hash(WINDOWS_TARGET_TRIPLE, roots);
        roots.linux_package_descriptor_hash =
            target_package_descriptor_hash(LINUX_TARGET_TRIPLE, roots);
        roots.closure_hash = closure_hash(roots);
    }
    let roots = target.comparable_roots.as_ref().expect("comparable roots");
    let targets = target.closure_targets.as_mut().expect("closure targets");
    targets.windows.package_descriptor_hash = roots.windows_package_descriptor_hash.clone();
    targets.linux.package_descriptor_hash = roots.linux_package_descriptor_hash.clone();
    write_target_report(&bundle, &target);

    let error = validate_native_gate_target_bundle(&bundle.report_path())
        .expect_err("unrelated target roots must not be trusted");
    assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID, "{error}");
    assert!(error.detail().contains("streaming performance root"));
}

#[test]
fn fail_bundle_rejects_package_state_and_arbitrary_top_level_entries() {
    for extra in ["package", ".state", "arbitrary"] {
        let bundle = TempBundle::new();
        let mut target = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
        materialize_check_reports(&bundle, &mut target);
        let path = bundle.root.join(extra);
        if extra == "arbitrary" {
            fs::write(&path, b"extra").expect("extra file");
        } else {
            fs::create_dir(&path).expect("extra directory");
        }
        write_target_report(&bundle, &target);

        let error = validate_native_gate_target_bundle(&bundle.report_path())
            .expect_err("FAIL inventory must be exact");
        assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID, "{extra}");
        assert!(error.detail().contains("top-level"), "{extra}: {error}");
    }
}

#[test]
fn checks_directory_symlink_or_reparse_point_is_rejected() {
    let bundle = TempBundle::new();
    let target_directory = TempBundle::new();
    let target = fail_at_host();
    create_directory_link(&bundle.root.join("checks"), &target_directory.root);
    write_target_report(&bundle, &target);

    let error = validate_native_gate_target_bundle(&bundle.report_path())
        .expect_err("checks link must be rejected");
    assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID);
    assert!(
        error.detail().contains("symlink") || error.detail().contains("reparse"),
        "{error}"
    );
}

fn fail_at_host() -> NativeGateTargetReportV1 {
    let mut target = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    target.checks[0].status = NativeGateCheckStatusV1::Fail;
    target.checks[0].report_path = None;
    target.checks[0].report_sha256 = None;
    target.checks[0].diagnostic = Some(NativeGateDiagnosticV1 {
        code: "NATIVE_GATE_CHECK_FAILED".to_owned(),
        message: "host failed".to_owned(),
    });
    for record in &mut target.checks[1..] {
        record.status = NativeGateCheckStatusV1::NotRun;
        record.report_path = None;
        record.report_sha256 = None;
        record.diagnostic = Some(NativeGateDiagnosticV1 {
            code: "PRIOR_CHECK_FAILED".to_owned(),
            message: "host-check failed".to_owned(),
        });
        record.elapsed_milliseconds = 0;
    }
    target
}

#[cfg(unix)]
fn create_directory_link(link: &Path, target: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create checks symlink");
}

#[cfg(windows)]
fn create_directory_link(link: &Path, target: &Path) {
    let output = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .expect("create checks junction");
    assert!(
        output.status.success(),
        "mklink failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
