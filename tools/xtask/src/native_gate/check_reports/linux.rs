use std::path::Path;

use super::*;

const CLOSURE_REPORT_SCHEMA_VERSION: u32 = 2;
const PACKAGE_REPORT_SCHEMA_VERSION: u32 = 3;

pub(in crate::native_gate) fn validate_linux_target_report_json_shape(
    bytes: &[u8],
) -> Result<(), NativeGateComparisonError> {
    const FIELDS: [&str; 11] = [
        "schema_version",
        "status",
        "release_ready",
        "git_commit_sha",
        "cargo_lock_sha256",
        "rustc_release",
        "target_triple",
        "checks",
        "release_target",
        "release_roots",
        "package",
    ];
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| report_invalid(format!("invalid target report JSON: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| report_invalid("target report must be a JSON object"))?;
    if object.len() != FIELDS.len() || FIELDS.iter().any(|field| !object.contains_key(*field)) {
        return Err(report_invalid(
            "target report fields do not exactly match NativeGateLinuxReportV2",
        ));
    }
    let checks = object
        .get("checks")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| report_invalid("target report checks must be an array"))?;
    const CHECK_FIELDS: [&str; 6] = [
        "check",
        "status",
        "report_path",
        "report_sha256",
        "diagnostic",
        "elapsed_milliseconds",
    ];
    if checks.iter().any(|check| {
        check.as_object().is_none_or(|record| {
            record.len() != CHECK_FIELDS.len()
                || CHECK_FIELDS
                    .iter()
                    .any(|field| !record.contains_key(*field))
        })
    }) {
        return Err(report_invalid(
            "target check record fields do not exactly match NativeGateCheckRecordV1",
        ));
    }
    Ok(())
}

pub(in crate::native_gate) fn validate_linux_check_reports(
    bundle_root: &Path,
    target: &NativeGateLinuxReportV2,
) -> Result<(), NativeGateComparisonError> {
    let mut reports = Vec::new();
    for record in &target.checks {
        let (Some(relative_path), Some(expected_hash)) =
            (&record.report_path, &record.report_sha256)
        else {
            continue;
        };
        if record.status != NativeGateCheckStatusV1::Pass {
            return Err(report_invalid(format!(
                "{} check JSON may only accompany a PASS record",
                record.check.as_str()
            )));
        }
        let path = bundle_root.join(relative_path);
        let bytes = read_bounded_regular_file(
            &path,
            MAX_CHECK_REPORT_BYTES,
            NATIVE_GATE_REPORT_INVALID,
            record.check.as_str(),
        )?;
        if sha256_hex(&bytes) != *expected_hash {
            return Err(report_invalid(format!(
                "{} report hash mismatch",
                record.check.as_str()
            )));
        }
        let parsed = match record.check {
            NativeGateCheckNameV1::V1Closure => parse_linux_closure_report(&bytes)?,
            NativeGateCheckNameV1::V1Package => parse_linux_package_report(&bytes)?,
            _ => parse_check_report(
                record.check,
                &bytes,
                &target.git_commit_sha,
                &target.target_triple,
                &target.rustc_release,
            )?,
        };
        reports.push(parsed);
    }

    if target.status == NativeGateRunStatusV1::Pass {
        validate_linux_pass_bindings(target, &reports)?;
    }
    Ok(())
}

fn parse_linux_package_report(
    bytes: &[u8],
) -> Result<ValidatedCheckReportV1, NativeGateComparisonError> {
    let report: CommandReportV3<PackageDetailsV3> =
        parse_json(NativeGateCheckNameV1::V1Package, bytes)?;
    let details = &report.details;
    if report.schema_version != PACKAGE_REPORT_SCHEMA_VERSION
        || report.command != "v1-package"
        || report.status != "PASS"
        || details.output != "package"
        || details.game_launch != "PASS"
        || details.headless_launch != "PASS"
        || details.tool_launch != "PASS"
        || details.source_project != "source/reference-alpha"
        || details.tool_neutral_record_count == 0
        || details.tool_publication_file_count == 0
        || details.release_version != env!("CARGO_PKG_VERSION")
        || details.dependency_count == 0
        || details.license_file_count == 0
        || details.protected_data_scan != "PASS"
        || details.getting_started_path != "GETTING_STARTED.md"
        || details.troubleshooting_path != "TROUBLESHOOTING.md"
    {
        return Err(report_invalid(
            "v1-package report must use schema 3 and bind package output, distribution evidence and all clean-install launches",
        ));
    }
    for (field, value) in [
        (
            "package_manifest_hash",
            details.package_manifest_hash.as_str(),
        ),
        (
            "composition_lock_hash",
            details.composition_lock_hash.as_str(),
        ),
        ("game_binary_hash", details.game_binary_hash.as_str()),
        (
            "headless_binary_hash",
            details.headless_binary_hash.as_str(),
        ),
        ("tool_binary_hash", details.tool_binary_hash.as_str()),
        ("tool_authoring_hash", details.tool_authoring_hash.as_str()),
        ("cargo_lock_hash", details.cargo_lock_hash.as_str()),
        (
            "dependency_inventory_hash",
            details.dependency_inventory_hash.as_str(),
        ),
    ] {
        validate_hash(field, value)?;
    }
    Ok(ValidatedCheckReportV1::V1PackageV3(Box::new(report)))
}

fn parse_linux_closure_report(
    bytes: &[u8],
) -> Result<ValidatedCheckReportV1, NativeGateComparisonError> {
    let report: CommandReportV2<V1ClosureDetailsV2> =
        parse_json(NativeGateCheckNameV1::V1Closure, bytes)?;
    if report.schema_version != CLOSURE_REPORT_SCHEMA_VERSION
        || report.command != "v1-closure"
        || report.status != "PASS"
        || !report.details.release_ready
    {
        return Err(report_invalid(
            "v1-closure report must use schema 2, command v1-closure, PASS and release_ready",
        ));
    }
    validate_linux_closure_hashes(&report.details)?;
    Ok(ValidatedCheckReportV1::V1ClosureV2(Box::new(report)))
}

fn validate_linux_closure_hashes(
    details: &V1ClosureDetailsV2,
) -> Result<(), NativeGateComparisonError> {
    for (field, value) in [
        (
            "project_composition_lock_hash",
            &details.project_composition_lock_hash,
        ),
        ("schema_registry_hash", &details.schema_registry_hash),
        ("content_manifest_hash", &details.content_manifest_hash),
        ("mechanics_lock_hash", &details.mechanics_lock_hash),
        ("world_partition_hash", &details.world_partition_hash),
        ("luau_manifest_hash", &details.luau_manifest_hash),
        ("wasm_manifest_hash", &details.wasm_manifest_hash),
        ("wit_v2_hash", &details.wit_v2_hash),
        ("wit_v3_hash", &details.wit_v3_hash),
        (
            "extension_compatibility_hash",
            &details.extension_compatibility_hash,
        ),
        ("play_state_root", &details.play_state_root),
        ("play_ledger_hash", &details.play_ledger_hash),
        ("replay_state_root", &details.replay_state_root),
        ("replay_ledger_hash", &details.replay_ledger_hash),
        ("audio_scene_pcm_digest", &details.audio_scene_pcm_digest),
        (
            "package_descriptor_hash",
            &details.release_target.package_descriptor_hash,
        ),
        ("closure_hash", &details.closure_hash),
    ] {
        validate_hash(field, value)?;
    }
    Ok(())
}

fn validate_linux_pass_bindings(
    target: &NativeGateLinuxReportV2,
    reports: &[ValidatedCheckReportV1],
) -> Result<(), NativeGateComparisonError> {
    if reports.len() != NativeGateCheckNameV1::ORDERED.len() {
        return Err(report_invalid(
            "PASS Linux bundle does not contain all typed check reports",
        ));
    }
    let roots = target
        .release_roots
        .as_ref()
        .ok_or_else(|| report_invalid("PASS Linux report has no release roots"))?;
    let package = target
        .package
        .as_ref()
        .ok_or_else(|| report_invalid("PASS Linux report has no package summary"))?;

    let ValidatedCheckReportV1::Host(host) = &reports[0] else {
        return Err(report_invalid("host-check typed report is out of order"));
    };
    bind("host target", &host.details.host, &target.target_triple)?;
    bind(
        "host rustc release",
        &host.details.rustc_release,
        &target.rustc_release,
    )?;

    let ValidatedCheckReportV1::Play(play) = &reports[1] else {
        return Err(report_invalid("play typed report is out of order"));
    };
    for (field, actual, expected) in [
        (
            "play project composition",
            play.project_composition_lock_hash.as_str(),
            roots.project_composition_lock_hash.as_str(),
        ),
        (
            "play state root",
            play.authoritative_state_root.as_str(),
            roots.play_state_root.as_str(),
        ),
        (
            "play ledger hash",
            play.command_ledger_hash.as_str(),
            roots.play_ledger_hash.as_str(),
        ),
    ] {
        bind(field, actual, expected)?;
    }

    let ValidatedCheckReportV1::PersistenceReplay(replay) = &reports[2] else {
        return Err(report_invalid(
            "persistence-replay typed report is out of order",
        ));
    };
    bind(
        "replay state root",
        &replay.details.final_state_root,
        &roots.replay_state_root,
    )?;
    bind(
        "replay ledger hash",
        &replay.details.final_ledger_root,
        &roots.replay_ledger_hash,
    )?;

    let ValidatedCheckReportV1::ContentPackage(content) = &reports[3] else {
        return Err(report_invalid(
            "content-package typed report is out of order",
        ));
    };
    for (field, actual, expected) in [
        (
            "content project composition",
            content.details.composition_lock_hash.as_str(),
            roots.project_composition_lock_hash.as_str(),
        ),
        (
            "content schema registry",
            content.details.schema_registry_hash.as_str(),
            roots.schema_registry_hash.as_str(),
        ),
        (
            "content manifest",
            content.details.content_manifest_hash.as_str(),
            roots.content_manifest_hash.as_str(),
        ),
        (
            "content mechanics lock",
            content.details.mechanics_lock_hash.as_str(),
            roots.mechanics_lock_hash.as_str(),
        ),
        (
            "content world partition",
            content.details.world_partition_hash.as_str(),
            roots.world_partition_hash.as_str(),
        ),
    ] {
        bind(field, actual, expected)?;
    }

    let ValidatedCheckReportV1::Platform(platform) = &reports[4] else {
        return Err(report_invalid("platform typed report is out of order"));
    };
    for (field, actual, expected) in [
        (
            "platform state root",
            platform.details.state_root.as_str(),
            roots.platform_state_root.as_str(),
        ),
        (
            "platform ledger hash",
            platform.details.ledger_hash.as_str(),
            roots.platform_ledger_hash.as_str(),
        ),
        (
            "presentation snapshot",
            platform.details.presentation_snapshot_hash.as_str(),
            roots.presentation_snapshot_hash.as_str(),
        ),
    ] {
        bind(field, actual, expected)?;
    }

    let ValidatedCheckReportV1::Performance(performance) = &reports[5] else {
        return Err(report_invalid("performance typed report is out of order"));
    };
    let streaming = performance
        .details
        .streaming
        .as_ref()
        .ok_or_else(|| report_invalid("performance streaming smoke result is missing"))?;
    let agent_planning = performance
        .details
        .agent_planning
        .as_ref()
        .ok_or_else(|| report_invalid("performance agent smoke result is missing"))?;
    bind(
        "streaming performance root",
        &streaming.final_world_state_hash,
        &roots.streaming_performance_hash,
    )?;
    bind(
        "agent performance root",
        &agent_planning.final_plan_hash,
        &roots.agent_performance_hash,
    )?;

    let ValidatedCheckReportV1::V1ClosureV2(closure) = &reports[6] else {
        return Err(report_invalid(
            "v1-closure schema 2 typed report is out of order",
        ));
    };
    validate_linux_closure_bindings(target, &closure.details)?;

    let ValidatedCheckReportV1::V1PackageV3(package_report) = &reports[7] else {
        return Err(report_invalid(
            "v1-package schema 3 typed report is out of order",
        ));
    };
    for (field, actual, expected) in [
        (
            "package target",
            package_report.details.target.as_str(),
            package.target_triple.as_str(),
        ),
        (
            "package manifest hash",
            package_report.details.package_manifest_hash.as_str(),
            package.package_manifest_sha256.as_str(),
        ),
        (
            "package composition root",
            package_report.details.composition_lock_hash.as_str(),
            package.project_lock_sha256.as_str(),
        ),
        (
            "package game binary",
            package_report.details.game_binary_hash.as_str(),
            package.game_binary_sha256.as_str(),
        ),
        (
            "package headless binary",
            package_report.details.headless_binary_hash.as_str(),
            package.headless_binary_sha256.as_str(),
        ),
    ] {
        bind(field, actual, expected)?;
    }
    Ok(())
}

fn validate_linux_closure_bindings(
    target: &NativeGateLinuxReportV2,
    details: &V1ClosureDetailsV2,
) -> Result<(), NativeGateComparisonError> {
    let roots = target
        .release_roots
        .as_ref()
        .ok_or_else(|| report_invalid("PASS Linux report has no release roots"))?;
    for (field, actual, expected) in [
        (
            "closure project composition",
            details.project_composition_lock_hash.as_str(),
            roots.project_composition_lock_hash.as_str(),
        ),
        (
            "closure schema registry",
            details.schema_registry_hash.as_str(),
            roots.schema_registry_hash.as_str(),
        ),
        (
            "closure content manifest",
            details.content_manifest_hash.as_str(),
            roots.content_manifest_hash.as_str(),
        ),
        (
            "closure mechanics lock",
            details.mechanics_lock_hash.as_str(),
            roots.mechanics_lock_hash.as_str(),
        ),
        (
            "closure world partition",
            details.world_partition_hash.as_str(),
            roots.world_partition_hash.as_str(),
        ),
        (
            "closure Luau manifest",
            details.luau_manifest_hash.as_str(),
            roots.luau_manifest_hash.as_str(),
        ),
        (
            "closure Wasm manifest",
            details.wasm_manifest_hash.as_str(),
            roots.wasm_manifest_hash.as_str(),
        ),
        (
            "closure WIT v2",
            details.wit_v2_hash.as_str(),
            roots.wit_v2_hash.as_str(),
        ),
        (
            "closure WIT v3",
            details.wit_v3_hash.as_str(),
            roots.wit_v3_hash.as_str(),
        ),
        (
            "closure extension compatibility",
            details.extension_compatibility_hash.as_str(),
            roots.extension_compatibility_hash.as_str(),
        ),
        (
            "closure play state",
            details.play_state_root.as_str(),
            roots.play_state_root.as_str(),
        ),
        (
            "closure play ledger",
            details.play_ledger_hash.as_str(),
            roots.play_ledger_hash.as_str(),
        ),
        (
            "closure replay state",
            details.replay_state_root.as_str(),
            roots.replay_state_root.as_str(),
        ),
        (
            "closure replay ledger",
            details.replay_ledger_hash.as_str(),
            roots.replay_ledger_hash.as_str(),
        ),
        (
            "closure hash",
            details.closure_hash.as_str(),
            roots.closure_hash.as_str(),
        ),
    ] {
        bind(field, actual, expected)?;
    }
    let summary = target
        .release_target
        .as_ref()
        .ok_or_else(|| report_invalid("PASS Linux report has no release target"))?;
    validate_closure_target_binding(
        "Linux release",
        &details.release_target,
        &summary.target_triple,
        &summary.package_descriptor_hash,
        &summary.runtime_check,
        &summary.desktop_smoke,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_gate::LINUX_TARGET_TRIPLE;

    fn package_report() -> serde_json::Value {
        let hash = "a".repeat(64);
        serde_json::json!({
            "schema_version": 3,
            "status": "PASS",
            "command": "v1-package",
            "details": {
                "target": LINUX_TARGET_TRIPLE,
                "output": "package",
                "package_manifest_hash": hash,
                "composition_lock_hash": hash,
                "game_binary_hash": hash,
                "headless_binary_hash": hash,
                "tool_binary_hash": hash,
                "game_launch": "PASS",
                "headless_launch": "PASS",
                "tool_launch": "PASS",
                "source_project": "source/reference-alpha",
                "tool_authoring_hash": hash,
                "tool_neutral_record_count": 1,
                "tool_publication_file_count": 1,
                "release_version": env!("CARGO_PKG_VERSION"),
                "cargo_lock_hash": hash,
                "dependency_inventory_hash": hash,
                "dependency_count": 1,
                "license_file_count": 1,
                "protected_data_scan": "PASS",
                "getting_started_path": "GETTING_STARTED.md",
                "troubleshooting_path": "TROUBLESHOOTING.md"
            }
        })
    }

    #[test]
    fn current_linux_package_report_requires_schema_three_distribution_receipt() {
        let bytes = serde_json::to_vec(&package_report()).expect("package report");
        assert!(matches!(
            parse_linux_package_report(&bytes).expect("schema three package report"),
            ValidatedCheckReportV1::V1PackageV3(_)
        ));

        let mut schema_one = package_report();
        schema_one["schema_version"] = serde_json::json!(1);
        assert!(
            parse_linux_package_report(
                &serde_json::to_vec(&schema_one).expect("schema one package report")
            )
            .is_err()
        );

        let mut missing_tool = package_report();
        missing_tool["details"]
            .as_object_mut()
            .expect("details")
            .remove("tool_binary_hash");
        assert!(
            parse_linux_package_report(
                &serde_json::to_vec(&missing_tool).expect("missing tool package report")
            )
            .is_err()
        );

        let mut failed_tool = package_report();
        failed_tool["details"]["tool_launch"] = serde_json::json!("FAIL");
        assert!(
            parse_linux_package_report(
                &serde_json::to_vec(&failed_tool).expect("failed tool package report")
            )
            .is_err()
        );
    }
}
