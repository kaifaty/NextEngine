use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateReleaseRootsV2 {
    pub project_composition_lock_hash: String,
    pub schema_registry_hash: String,
    pub content_manifest_hash: String,
    pub mechanics_lock_hash: String,
    pub world_partition_hash: String,
    pub luau_manifest_hash: String,
    pub wasm_manifest_hash: String,
    pub wit_v2_hash: String,
    pub wit_v3_hash: String,
    pub extension_compatibility_hash: String,
    pub play_state_root: String,
    pub play_ledger_hash: String,
    pub replay_state_root: String,
    pub replay_ledger_hash: String,
    pub platform_state_root: String,
    pub platform_ledger_hash: String,
    pub presentation_snapshot_hash: String,
    pub streaming_performance_hash: String,
    pub agent_performance_hash: String,
    pub audio_scene_pcm_digest: String,
    pub packaged_game_state_root: String,
    pub packaged_game_ledger_hash: String,
    pub packaged_headless_state_root: String,
    pub packaged_headless_ledger_hash: String,
    pub closure_hash: String,
    pub package_descriptor_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateLinuxReportV2 {
    pub schema_version: u32,
    pub status: NativeGateRunStatusV1,
    pub release_ready: bool,
    pub git_commit_sha: String,
    pub cargo_lock_sha256: String,
    pub rustc_release: String,
    pub target_triple: String,
    pub checks: Vec<NativeGateCheckRecordV1>,
    pub release_target: Option<NativeGateClosureTargetSummaryV1>,
    pub release_roots: Option<NativeGateReleaseRootsV2>,
    pub package: Option<NativeGatePackageSummaryV1>,
}

pub fn validate_native_gate_linux_bundle(
    target_report_path: &Path,
) -> Result<NativeGateLinuxReportV2, NativeGateComparisonError> {
    if target_report_path
        .file_name()
        .and_then(|name| name.to_str())
        != Some(TARGET_REPORT_FILE)
    {
        return Err(report_invalid(format!(
            "target report path must end with {TARGET_REPORT_FILE}"
        )));
    }
    let target_report_bytes = read_bounded_regular_file(
        target_report_path,
        MAX_TARGET_REPORT_BYTES,
        NATIVE_GATE_REPORT_INVALID,
        "target report",
    )?;
    check_reports::validate_linux_target_report_json_shape(&target_report_bytes)?;
    let report: NativeGateLinuxReportV2 = serde_json::from_slice(&target_report_bytes)
        .map_err(|error| report_invalid(format!("invalid target report JSON: {error}")))?;
    validate_native_gate_linux_report(&report)?;

    let bundle_root = target_report_path
        .parent()
        .ok_or_else(|| report_invalid("target report has no bundle directory"))?;
    bundle::validate_bundle_tree(bundle_root, report.status, &report.checks)?;
    check_reports::validate_linux_check_reports(bundle_root, &report)?;

    if report.status == NativeGateRunStatusV1::Pass {
        let package_summary = report.package.as_ref().ok_or_else(|| {
            package_invalid("PASS Linux report does not contain a package summary")
        })?;
        let package_root = bundle_root.join(&package_summary.relative_path);
        let manifest =
            crate::package::validate_v1_package(&package_root).map_err(package_validation_error)?;
        let manifest_path = package_root.join(crate::package::PACKAGE_MANIFEST_FILE);
        let manifest_bytes = read_bounded_regular_file(
            &manifest_path,
            MAX_PACKAGE_MANIFEST_BYTES,
            NATIVE_GATE_PACKAGE_INVALID,
            "package manifest",
        )?;
        if sha256_hex(&manifest_bytes) != package_summary.package_manifest_sha256 {
            return Err(package_invalid(
                "package manifest bytes do not match package summary hash",
            ));
        }
        validate_package_manifest_summary(&report.target_triple, package_summary, &manifest)?;
    }

    Ok(report)
}

pub fn validate_native_gate_linux_report(
    report: &NativeGateLinuxReportV2,
) -> Result<(), NativeGateComparisonError> {
    if report.schema_version != NATIVE_GATE_SCHEMA_VERSION {
        return Err(report_invalid(format!(
            "unsupported Linux target report schema version: {}",
            report.schema_version
        )));
    }
    validate_lower_hex("git_commit_sha", &report.git_commit_sha, 40)?;
    validate_lower_hex("cargo_lock_sha256", &report.cargo_lock_sha256, 64)?;
    if report.rustc_release != NATIVE_GATE_RUSTC_RELEASE {
        return Err(report_invalid(format!(
            "rustc_release must be {NATIVE_GATE_RUSTC_RELEASE}"
        )));
    }
    if report.target_triple != LINUX_TARGET_TRIPLE {
        return Err(NativeGateComparisonError::new(
            NATIVE_GATE_TARGET_SET_INVALID,
            format!(
                "Linux release report target must be {LINUX_TARGET_TRIPLE}, got {}",
                report.target_triple
            ),
        ));
    }
    validate_check_records(report.status, &report.checks)?;

    if let Some(target) = &report.release_target {
        validate_release_target(target)?;
    }
    if let Some(roots) = &report.release_roots {
        validate_release_roots(roots)?;
        let target = report.release_target.as_ref().ok_or_else(|| {
            report_invalid("release roots require a Linux release target summary")
        })?;
        if target.package_descriptor_hash != roots.package_descriptor_hash {
            return Err(report_invalid(
                "Linux release target descriptor does not match release roots",
            ));
        }
        validate_derived_release_roots(roots)?;
    }

    match report.status {
        NativeGateRunStatusV1::Pass => {
            if !report.release_ready {
                return Err(report_invalid(
                    "PASS Linux target report must claim release_ready",
                ));
            }
            let roots = report
                .release_roots
                .as_ref()
                .ok_or_else(|| report_invalid("PASS Linux report requires release roots"))?;
            if report.release_target.is_none() {
                return Err(report_invalid(
                    "PASS Linux report requires a release target summary",
                ));
            }
            let package = report
                .package
                .as_ref()
                .ok_or_else(|| package_invalid("PASS Linux report requires a package summary"))?;
            validate_package(package, roots)?;
        }
        NativeGateRunStatusV1::Fail => {
            if report.release_ready {
                return Err(report_invalid("FAIL Linux report cannot be release_ready"));
            }
            if report.release_target.is_some() || report.release_roots.is_some() {
                return Err(report_invalid(
                    "FAIL Linux report must not publish release target or roots",
                ));
            }
            if report.package.is_some() {
                return Err(package_invalid(
                    "FAIL Linux report must not publish a package summary",
                ));
            }
        }
    }
    Ok(())
}

fn validate_check_records(
    status: NativeGateRunStatusV1,
    checks: &[NativeGateCheckRecordV1],
) -> Result<(), NativeGateComparisonError> {
    if checks.len() != NativeGateCheckNameV1::ORDERED.len() {
        return Err(report_invalid(format!(
            "expected {} check records, found {}",
            NativeGateCheckNameV1::ORDERED.len(),
            checks.len()
        )));
    }
    for (index, (record, expected)) in checks
        .iter()
        .zip(NativeGateCheckNameV1::ORDERED)
        .enumerate()
    {
        if record.check != expected {
            return Err(report_invalid(format!(
                "check record {index} must be {}",
                expected.as_str()
            )));
        }
        super::validate_check_record(record)?;
    }
    match status {
        NativeGateRunStatusV1::Pass => {
            if let Some(record) = checks
                .iter()
                .find(|record| record.status != NativeGateCheckStatusV1::Pass)
            {
                return Err(report_invalid(format!(
                    "PASS Linux report contains non-PASS check {}",
                    record.check.as_str()
                )));
            }
        }
        NativeGateRunStatusV1::Fail => {
            let failed_index = checks
                .iter()
                .position(|record| record.status == NativeGateCheckStatusV1::Fail)
                .ok_or_else(|| report_invalid("FAIL Linux report has no failed check"))?;
            if checks.iter().enumerate().any(|(index, record)| {
                let expected = if index < failed_index {
                    NativeGateCheckStatusV1::Pass
                } else if index == failed_index {
                    NativeGateCheckStatusV1::Fail
                } else {
                    NativeGateCheckStatusV1::NotRun
                };
                record.status != expected
            }) {
                return Err(report_invalid(
                    "FAIL Linux check records must follow PASS* -> FAIL -> NOT_RUN*",
                ));
            }
        }
    }
    Ok(())
}

fn validate_release_target(
    target: &NativeGateClosureTargetSummaryV1,
) -> Result<(), NativeGateComparisonError> {
    if target.target_triple != LINUX_TARGET_TRIPLE {
        return Err(NativeGateComparisonError::new(
            NATIVE_GATE_TARGET_SET_INVALID,
            format!(
                "release target must be {LINUX_TARGET_TRIPLE}, got {}",
                target.target_triple
            ),
        ));
    }
    validate_lower_hex(
        "package_descriptor_hash",
        &target.package_descriptor_hash,
        64,
    )?;
    if target.runtime_check != NativeGateTargetExecutionStatusV1::Pass
        || target.desktop_smoke != NativeGateTargetExecutionStatusV1::Pass
    {
        return Err(report_invalid(
            "Linux release target runtime and desktop checks must PASS",
        ));
    }
    Ok(())
}

fn validate_release_roots(
    roots: &NativeGateReleaseRootsV2,
) -> Result<(), NativeGateComparisonError> {
    for (field, value) in release_root_fields_v2(roots) {
        validate_lower_hex(field, value, 64)?;
    }
    Ok(())
}

fn validate_derived_release_roots(
    roots: &NativeGateReleaseRootsV2,
) -> Result<(), NativeGateComparisonError> {
    if roots.package_descriptor_hash != linux_package_descriptor_hash_v2(roots) {
        return Err(package_invalid(
            "Linux package descriptor does not match its package-bound roots",
        ));
    }
    if roots.closure_hash != linux_closure_hash_v2(roots) {
        return Err(NativeGateComparisonError::new(
            NATIVE_GATE_ROOT_MISMATCH,
            "Linux closure_hash does not match its declared roots",
        ));
    }
    Ok(())
}

fn validate_package(
    package: &NativeGatePackageSummaryV1,
    roots: &NativeGateReleaseRootsV2,
) -> Result<(), NativeGateComparisonError> {
    if package.status != NativeGateRunStatusV1::Pass {
        return Err(package_invalid("package summary status must be PASS"));
    }
    if package.target_triple != LINUX_TARGET_TRIPLE {
        return Err(package_invalid(format!(
            "package target must be {LINUX_TARGET_TRIPLE}, got {}",
            package.target_triple
        )));
    }
    if package.relative_path != "package" || !is_portable_relative_path(&package.relative_path) {
        return Err(package_invalid(
            "package relative_path must be the portable path package",
        ));
    }
    for (field, value) in [
        (
            "package_manifest_sha256",
            package.package_manifest_sha256.as_str(),
        ),
        ("project_lock_sha256", package.project_lock_sha256.as_str()),
        (
            "schema_registry_sha256",
            package.schema_registry_sha256.as_str(),
        ),
        (
            "content_manifest_sha256",
            package.content_manifest_sha256.as_str(),
        ),
        (
            "mechanics_lock_sha256",
            package.mechanics_lock_sha256.as_str(),
        ),
        (
            "world_partition_sha256",
            package.world_partition_sha256.as_str(),
        ),
        ("game_binary_sha256", package.game_binary_sha256.as_str()),
        (
            "headless_binary_sha256",
            package.headless_binary_sha256.as_str(),
        ),
    ] {
        validate_package_hash(field, value)?;
    }
    validate_packaged_launch("game", &package.game)?;
    validate_packaged_launch("headless", &package.headless)?;
    for (field, actual, expected) in [
        (
            "project_lock_sha256",
            package.project_lock_sha256.as_str(),
            roots.project_composition_lock_hash.as_str(),
        ),
        (
            "schema_registry_sha256",
            package.schema_registry_sha256.as_str(),
            roots.schema_registry_hash.as_str(),
        ),
        (
            "content_manifest_sha256",
            package.content_manifest_sha256.as_str(),
            roots.content_manifest_hash.as_str(),
        ),
        (
            "mechanics_lock_sha256",
            package.mechanics_lock_sha256.as_str(),
            roots.mechanics_lock_hash.as_str(),
        ),
        (
            "world_partition_sha256",
            package.world_partition_sha256.as_str(),
            roots.world_partition_hash.as_str(),
        ),
        (
            "game.state_root",
            package.game.state_root.as_str(),
            roots.packaged_game_state_root.as_str(),
        ),
        (
            "game.ledger_hash",
            package.game.ledger_hash.as_str(),
            roots.packaged_game_ledger_hash.as_str(),
        ),
        (
            "headless.state_root",
            package.headless.state_root.as_str(),
            roots.packaged_headless_state_root.as_str(),
        ),
        (
            "headless.ledger_hash",
            package.headless.ledger_hash.as_str(),
            roots.packaged_headless_ledger_hash.as_str(),
        ),
    ] {
        compare_package_root(field, actual, expected)?;
    }
    Ok(())
}
