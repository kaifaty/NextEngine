use super::*;

#[test]
fn package_failure_diagnostics_are_valid_native_gate_codes() {
    for code in [
        NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID,
        NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING,
        NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED,
        NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT,
    ] {
        validate_diagnostic(&NativeGateDiagnosticV1 {
            code: code.to_owned(),
            message: "controlled package failure".to_owned(),
        })
        .expect("package diagnostic is allowlisted");
    }
}

#[test]
fn bundle_validation_rejects_tampered_package_before_accepting_pass_report() {
    let bundle = TempBundle::new();
    let mut report = report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&bundle, &mut report);
    let package_root = bundle.root.join("package");
    fs::create_dir(&package_root).expect("package directory");
    fs::write(
        package_root.join(crate::package::PACKAGE_MANIFEST_FILE),
        b"{\"schema_version\":2,\"tampered\":true}",
    )
    .expect("tampered package manifest");
    write_target_report(&bundle, &report);

    let error =
        validate_native_gate_target_bundle(&bundle.report_path()).expect_err("tampered package");
    assert_eq!(error.code(), NATIVE_GATE_PACKAGE_INVALID);
}

#[test]
fn package_manifest_and_target_summary_must_match_exactly() {
    let report = report(WINDOWS_TARGET_TRIPLE);
    let summary = report.package.as_ref().expect("summary");
    let manifest = package_manifest_from_summary(&report);
    validate_package_manifest_summary(&report.target_triple, summary, &manifest)
        .expect("matching manifest and summary");

    let mut tampered_manifest = manifest.clone();
    tampered_manifest.binaries.game.binary_sha256 = hash('f');
    let error =
        validate_package_manifest_summary(&report.target_triple, summary, &tampered_manifest)
            .expect_err("tampered binary summary");
    assert_eq!(error.code(), NATIVE_GATE_PACKAGE_INVALID);
    assert!(error.detail().contains("game.binary_sha256"));

    let mut tampered_summary = summary.clone();
    tampered_summary.schema_registry_sha256 = hash('f');
    let error =
        validate_package_manifest_summary(&report.target_triple, &tampered_summary, &manifest)
            .expect_err("tampered roots summary");
    assert_eq!(error.code(), NATIVE_GATE_PACKAGE_INVALID);
    assert!(error.detail().contains("schema_registry_sha256"));
}
