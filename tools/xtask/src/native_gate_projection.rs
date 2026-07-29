use crate::*;

pub(crate) fn validate_native_gate_closure(
    identity: &NativeGateIdentity,
    closure: &CommandReportV1<V1ClosureDetailsV1>,
    play: &next_application::RunReportV1,
    replay: &CommandReportV1<PersistenceReplayDetailsV1>,
    content: &CommandReportV1<ContentPackageDetailsV1>,
    platform: &CommandReportV1<PlatformDetailsV1>,
) -> Result<NativeGateClosureTargetSetV1, String> {
    if closure.status != "LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN" || closure.details.shipping_ready {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: v1-closure must report LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN, got {}",
            closure.status
        ));
    }
    ensure_gate_root(
        "project_composition_lock_hash",
        &closure.details.project_composition_lock_hash,
        &content.details.composition_lock_hash,
    )?;
    ensure_gate_root(
        "schema_registry_hash",
        &closure.details.schema_registry_hash,
        &content.details.schema_registry_hash,
    )?;
    ensure_gate_root(
        "content_manifest_hash",
        &closure.details.content_manifest_hash,
        &content.details.content_manifest_hash,
    )?;
    ensure_gate_root(
        "mechanics_lock_hash",
        &closure.details.mechanics_lock_hash,
        &content.details.mechanics_lock_hash,
    )?;
    ensure_gate_root(
        "world_partition_hash",
        &closure.details.world_partition_hash,
        &content.details.world_partition_hash,
    )?;
    ensure_gate_root(
        "play.project_composition_lock_hash",
        &closure.details.project_composition_lock_hash,
        &play.project_composition_lock_hash,
    )?;
    ensure_gate_root(
        "play_state_root",
        &closure.details.play_state_root,
        &play.authoritative_state_root,
    )?;
    ensure_gate_root(
        "play_ledger_hash",
        &closure.details.play_ledger_hash,
        &play.command_ledger_hash,
    )?;
    ensure_gate_root(
        "replay_state_root",
        &closure.details.replay_state_root,
        &replay.details.final_state_root,
    )?;
    ensure_gate_root(
        "replay_ledger_hash",
        &closure.details.replay_ledger_hash,
        &replay.details.final_ledger_root,
    )?;
    ensure_gate_root(
        "platform_state_root",
        &closure.details.play_state_root,
        &platform.details.state_root,
    )?;
    ensure_gate_root(
        "platform_ledger_hash",
        &closure.details.play_ledger_hash,
        &platform.details.ledger_hash,
    )?;

    let targets = NativeGateClosureTargetSetV1 {
        windows: native_gate_closure_target(&closure.details.windows, WINDOWS_TARGET_TRIPLE)?,
        linux: native_gate_closure_target(&closure.details.linux, LINUX_TARGET_TRIPLE)?,
    };
    let (native, remote) = if identity.target_triple == WINDOWS_TARGET_TRIPLE {
        (&targets.windows, &targets.linux)
    } else {
        (&targets.linux, &targets.windows)
    };
    if native.runtime_check != NativeGateTargetExecutionStatusV1::Pass
        || native.desktop_smoke != NativeGateTargetExecutionStatusV1::Pass
    {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: native closure target {} did not PASS",
            identity.target_triple
        ));
    }
    let (
        NativeGateTargetExecutionStatusV1::NotRun {
            reason: runtime_reason,
        },
        NativeGateTargetExecutionStatusV1::NotRun {
            reason: desktop_reason,
        },
    ) = (&remote.runtime_check, &remote.desktop_smoke)
    else {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: remote closure target {} must be NOT_RUN",
            remote.target_triple
        ));
    };
    if runtime_reason != desktop_reason {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: remote closure target {} reasons differ",
            remote.target_triple
        ));
    }
    Ok(targets)
}

fn native_gate_closure_target(
    details: &TargetGateDetailsV1,
    expected_target: &str,
) -> Result<NativeGateClosureTargetSummaryV1, String> {
    if details.target != expected_target {
        return Err(format!(
            "NATIVE_GATE_TARGET_SET_INVALID: expected {expected_target}, got {}",
            details.target
        ));
    }
    Ok(NativeGateClosureTargetSummaryV1 {
        target_triple: details.target.clone(),
        package_descriptor_hash: details.package_descriptor_hash.clone(),
        runtime_check: parse_native_gate_target_status(&details.runtime_check)?,
        desktop_smoke: parse_native_gate_target_status(&details.desktop_smoke)?,
    })
}

fn parse_native_gate_target_status(
    status: &str,
) -> Result<NativeGateTargetExecutionStatusV1, String> {
    if status == "PASS" {
        return Ok(NativeGateTargetExecutionStatusV1::Pass);
    }
    let reason = status
        .strip_prefix("NOT_RUN(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| {
            format!("NATIVE_GATE_REPORT_INVALID: invalid target gate status {status}")
        })?;
    if reason.is_empty()
        || !reason
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: invalid target gate reason {reason}"
        ));
    }
    Ok(NativeGateTargetExecutionStatusV1::NotRun {
        reason: reason.to_owned(),
    })
}

fn ensure_gate_root(field: &str, expected: &str, actual: &str) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "NATIVE_GATE_ROOT_MISMATCH: {field} differs: expected {expected}, got {actual}"
        ))
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_native_gate_package_result(
    identity: &NativeGateIdentity,
    build: &xtask::package::PackageBuildResult,
    play: &next_application::RunReportV1,
    replay: &CommandReportV1<PersistenceReplayDetailsV1>,
    content: &CommandReportV1<ContentPackageDetailsV1>,
    platform: &CommandReportV1<PlatformDetailsV1>,
    performance: &CommandReportV1<PerformanceDetailsV1>,
    closure: &NativeGateClosureCheckResult,
) -> Result<NativeGatePackageCheckResult, String> {
    let manifest = &build.manifest;
    if manifest.target_triple != identity.target_triple {
        return Err(format!(
            "NATIVE_GATE_PACKAGE_INVALID: package target {} does not match {}",
            manifest.target_triple, identity.target_triple
        ));
    }
    let neutral = &manifest.target_neutral_roots;
    for (field, packaged, expected) in [
        (
            "project_composition_lock_hash",
            neutral.project_composition_lock_sha256.as_str(),
            closure
                .report
                .details
                .project_composition_lock_hash
                .as_str(),
        ),
        (
            "schema_registry_hash",
            neutral.schema_registry_sha256.as_str(),
            closure.report.details.schema_registry_hash.as_str(),
        ),
        (
            "content_manifest_hash",
            neutral.content_manifest_sha256.as_str(),
            closure.report.details.content_manifest_hash.as_str(),
        ),
        (
            "mechanics_lock_hash",
            neutral.mechanics_lock_sha256.as_str(),
            closure.report.details.mechanics_lock_hash.as_str(),
        ),
        (
            "world_partition_hash",
            neutral.world_partition_sha256.as_str(),
            closure.report.details.world_partition_hash.as_str(),
        ),
    ] {
        if packaged != expected {
            return Err(format!(
                "NATIVE_GATE_PACKAGE_INVALID: packaged {field} does not match checked root"
            ));
        }
    }
    if manifest.binaries.game.launch_status != "PASS"
        || manifest.binaries.headless.launch_status != "PASS"
    {
        return Err(
            "NATIVE_GATE_PACKAGE_INVALID: packaged game and headless launches must PASS".to_owned(),
        );
    }
    ensure_gate_root(
        "package/content composition",
        &content.details.composition_lock_hash,
        &neutral.project_composition_lock_sha256,
    )?;
    ensure_gate_root(
        "package/play composition",
        &play.project_composition_lock_hash,
        &neutral.project_composition_lock_sha256,
    )?;

    let package = NativeGatePackageSummaryV1 {
        status: NativeGateRunStatusV1::Pass,
        target_triple: manifest.target_triple.clone(),
        relative_path: "package".to_owned(),
        package_manifest_sha256: build.package_manifest_sha256.clone(),
        composition_lock_sha256: neutral.project_composition_lock_sha256.clone(),
        schema_registry_sha256: neutral.schema_registry_sha256.clone(),
        content_manifest_sha256: neutral.content_manifest_sha256.clone(),
        mechanics_lock_sha256: neutral.mechanics_lock_sha256.clone(),
        world_partition_sha256: neutral.world_partition_sha256.clone(),
        game_binary_sha256: manifest.binaries.game.binary_sha256.clone(),
        headless_binary_sha256: manifest.binaries.headless.binary_sha256.clone(),
        game: NativeGatePackagedLaunchSummaryV1 {
            status: NativeGateCheckStatusV1::Pass,
            state_root: manifest.binaries.game.authoritative_state_root.clone(),
            ledger_hash: manifest.binaries.game.command_ledger_hash.clone(),
        },
        headless: NativeGatePackagedLaunchSummaryV1 {
            status: NativeGateCheckStatusV1::Pass,
            state_root: manifest.binaries.headless.authoritative_state_root.clone(),
            ledger_hash: manifest.binaries.headless.command_ledger_hash.clone(),
        },
    };
    let comparable_roots = NativeGateComparableRootsV1 {
        project_composition_lock_hash: closure.report.details.project_composition_lock_hash.clone(),
        schema_registry_hash: closure.report.details.schema_registry_hash.clone(),
        content_manifest_hash: closure.report.details.content_manifest_hash.clone(),
        mechanics_lock_hash: closure.report.details.mechanics_lock_hash.clone(),
        world_partition_hash: closure.report.details.world_partition_hash.clone(),
        luau_manifest_hash: closure.report.details.luau_manifest_hash.clone(),
        wasm_manifest_hash: closure.report.details.wasm_manifest_hash.clone(),
        wit_v2_hash: closure.report.details.wit_v2_hash.clone(),
        wit_v3_hash: closure.report.details.wit_v3_hash.clone(),
        extension_compatibility_hash: closure.report.details.extension_compatibility_hash.clone(),
        play_state_root: play.authoritative_state_root.clone(),
        play_ledger_hash: play.command_ledger_hash.clone(),
        replay_state_root: replay.details.final_state_root.clone(),
        replay_ledger_hash: replay.details.final_ledger_root.clone(),
        platform_state_root: platform.details.state_root.clone(),
        platform_ledger_hash: platform.details.ledger_hash.clone(),
        presentation_snapshot_hash: platform.details.presentation_snapshot_hash.clone(),
        streaming_performance_hash: performance.details.streaming.final_world_state_hash.clone(),
        agent_performance_hash: performance.details.agent_planning.final_plan_hash.clone(),
        packaged_game_state_root: package.game.state_root.clone(),
        packaged_game_ledger_hash: package.game.ledger_hash.clone(),
        packaged_headless_state_root: package.headless.state_root.clone(),
        packaged_headless_ledger_hash: package.headless.ledger_hash.clone(),
        closure_hash: closure.report.details.closure_hash.clone(),
        windows_package_descriptor_hash: closure.targets.windows.package_descriptor_hash.clone(),
        linux_package_descriptor_hash: closure.targets.linux.package_descriptor_hash.clone(),
    };
    let report = package_command_report(build, "package".to_owned());
    Ok(NativeGatePackageCheckResult {
        report,
        closure_targets: closure.targets.clone(),
        comparable_roots,
        package,
    })
}
