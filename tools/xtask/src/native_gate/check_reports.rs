use std::path::Path;

use serde::Deserialize;

use crate::report::{
    CommandReportV1, ContentPackageDetailsV1, HostCheckDetailsV1, PackageDetailsV1,
    PerformanceDetailsV1, PersistenceReplayDetailsV1, PlatformDetailsV1, V1ClosureDetailsV1,
};

use super::{
    NATIVE_GATE_REPORT_INVALID, NativeGateCheckNameV1, NativeGateCheckStatusV1,
    NativeGateComparisonError, NativeGateRunStatusV1, NativeGateTargetExecutionStatusV1,
    NativeGateTargetReportV1, read_bounded_regular_file, report_invalid, sha256_hex,
    validate_lower_hex,
};

const MAX_CHECK_REPORT_BYTES: usize = 8 * 1024 * 1024;
const COMMAND_REPORT_SCHEMA_VERSION: u32 = 1;

pub(super) fn validate_target_report_json_shape(
    bytes: &[u8],
) -> Result<(), NativeGateComparisonError> {
    const FIELDS: [&str; 10] = [
        "schema_version",
        "status",
        "git_commit_sha",
        "cargo_lock_sha256",
        "rustc_release",
        "target_triple",
        "checks",
        "closure_targets",
        "comparable_roots",
        "package",
    ];
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| report_invalid(format!("invalid target report JSON: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| report_invalid("target report must be a JSON object"))?;
    if object.len() != FIELDS.len() || FIELDS.iter().any(|field| !object.contains_key(*field)) {
        return Err(report_invalid(
            "target report fields do not exactly match NativeGateTargetReportV1",
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatePlayReportV1 {
    schema_version: u32,
    status: String,
    composition_root: String,
    session_id: String,
    close_receipt_hash: String,
    close_result: String,
    final_save_generation_hash: Option<String>,
    project_composition_lock_hash: String,
    ticks: u64,
    events: u64,
    rpg_events: u64,
    authoritative_revision: u64,
    authoritative_state_root: String,
    command_archive_root: String,
    command_identity_index_root: String,
    command_ledger_hash: String,
    interactive_host_object_count: u64,
    presentation: Option<GatePresentationReportV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatePresentationReportV1 {
    target: String,
    snapshot_hash: String,
    object_count: u64,
}

enum ValidatedCheckReportV1 {
    Host(Box<CommandReportV1<HostCheckDetailsV1>>),
    Play(Box<GatePlayReportV1>),
    PersistenceReplay(Box<CommandReportV1<PersistenceReplayDetailsV1>>),
    ContentPackage(Box<CommandReportV1<ContentPackageDetailsV1>>),
    Platform(Box<CommandReportV1<PlatformDetailsV1>>),
    Performance(Box<CommandReportV1<PerformanceDetailsV1>>),
    V1Closure(Box<CommandReportV1<V1ClosureDetailsV1>>),
    V1Package(Box<CommandReportV1<PackageDetailsV1>>),
}

pub(super) fn validate_check_reports(
    bundle_root: &Path,
    target: &NativeGateTargetReportV1,
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
        reports.push(parse_check_report(record.check, &bytes)?);
    }

    if target.status == NativeGateRunStatusV1::Pass {
        validate_pass_bindings(target, &reports)?;
    }
    Ok(())
}

fn parse_check_report(
    check: NativeGateCheckNameV1,
    bytes: &[u8],
) -> Result<ValidatedCheckReportV1, NativeGateComparisonError> {
    match check {
        NativeGateCheckNameV1::HostCheck => {
            let report =
                parse_command_report::<HostCheckDetailsV1>(check, bytes, "host-check", "PASS")?;
            Ok(ValidatedCheckReportV1::Host(Box::new(report)))
        }
        NativeGateCheckNameV1::Play => {
            validate_play_object_shape(bytes)?;
            let report: GatePlayReportV1 = parse_json(check, bytes)?;
            if report.schema_version != COMMAND_REPORT_SCHEMA_VERSION
                || report.status != "PASS"
                || report.composition_root != "Tools"
            {
                return Err(report_invalid(
                    "play report must use schema 1, PASS and Tools composition root",
                ));
            }
            validate_play_hashes(&report)?;
            Ok(ValidatedCheckReportV1::Play(Box::new(report)))
        }
        NativeGateCheckNameV1::PersistenceReplay => {
            let report = parse_command_report::<PersistenceReplayDetailsV1>(
                check,
                bytes,
                "persistence-replay",
                "PASS",
            )?;
            validate_hash(
                "persistence-replay final_state_root",
                &report.details.final_state_root,
            )?;
            validate_hash(
                "persistence-replay final_ledger_root",
                &report.details.final_ledger_root,
            )?;
            Ok(ValidatedCheckReportV1::PersistenceReplay(Box::new(report)))
        }
        NativeGateCheckNameV1::ContentPackage => {
            let report = parse_command_report::<ContentPackageDetailsV1>(
                check,
                bytes,
                "content-package",
                "PASS",
            )?;
            for (field, value) in [
                ("schema_registry_hash", &report.details.schema_registry_hash),
                (
                    "content_manifest_hash",
                    &report.details.content_manifest_hash,
                ),
                ("mechanics_lock_hash", &report.details.mechanics_lock_hash),
                ("world_partition_hash", &report.details.world_partition_hash),
                (
                    "composition_lock_hash",
                    &report.details.composition_lock_hash,
                ),
            ] {
                validate_hash(field, value)?;
            }
            Ok(ValidatedCheckReportV1::ContentPackage(Box::new(report)))
        }
        NativeGateCheckNameV1::Platform => {
            let report =
                parse_command_report::<PlatformDetailsV1>(check, bytes, "platform", "PASS")?;
            if report.details.portable_contract != "PASS"
                || report.details.sdl_ash_candidate != "PASS"
            {
                return Err(report_invalid(
                    "platform gate report requires portable and SDL/Ash PASS",
                ));
            }
            validate_hash(
                "presentation_snapshot_hash",
                &report.details.presentation_snapshot_hash,
            )?;
            validate_hash("platform ledger_hash", &report.details.ledger_hash)?;
            validate_hash("platform state_root", &report.details.state_root)?;
            Ok(ValidatedCheckReportV1::Platform(Box::new(report)))
        }
        NativeGateCheckNameV1::Performance => {
            let report =
                parse_command_report::<PerformanceDetailsV1>(check, bytes, "performance", "PASS")?;
            let streaming =
                report.details.streaming.as_ref().ok_or_else(|| {
                    report_invalid("performance streaming smoke result is missing")
                })?;
            let agent_planning = report
                .details
                .agent_planning
                .as_ref()
                .ok_or_else(|| report_invalid("performance agent smoke result is missing"))?;
            validate_hash(
                "streaming final_world_state_hash",
                &streaming.final_world_state_hash,
            )?;
            validate_hash("agent final_plan_hash", &agent_planning.final_plan_hash)?;
            Ok(ValidatedCheckReportV1::Performance(Box::new(report)))
        }
        NativeGateCheckNameV1::V1Closure => {
            let report = parse_command_report::<V1ClosureDetailsV1>(
                check,
                bytes,
                "v1-closure",
                "LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN",
            )?;
            if report.details.shipping_ready {
                return Err(report_invalid(
                    "native v1-closure check must not claim shipping_ready",
                ));
            }
            validate_closure_hashes(&report.details)?;
            Ok(ValidatedCheckReportV1::V1Closure(Box::new(report)))
        }
        NativeGateCheckNameV1::V1Package => {
            let report =
                parse_command_report::<PackageDetailsV1>(check, bytes, "v1-package", "PASS")?;
            if report.details.output != "package"
                || report.details.game_launch != "PASS"
                || report.details.headless_launch != "PASS"
            {
                return Err(report_invalid(
                    "v1-package check requires package output and PASS launches",
                ));
            }
            for (field, value) in [
                (
                    "package_manifest_hash",
                    &report.details.package_manifest_hash,
                ),
                (
                    "composition_lock_hash",
                    &report.details.composition_lock_hash,
                ),
                ("game_binary_hash", &report.details.game_binary_hash),
                ("headless_binary_hash", &report.details.headless_binary_hash),
            ] {
                validate_hash(field, value)?;
            }
            Ok(ValidatedCheckReportV1::V1Package(Box::new(report)))
        }
    }
}

fn parse_command_report<T>(
    check: NativeGateCheckNameV1,
    bytes: &[u8],
    command: &str,
    status: &str,
) -> Result<CommandReportV1<T>, NativeGateComparisonError>
where
    T: for<'de> Deserialize<'de>,
{
    let report: CommandReportV1<T> = parse_json(check, bytes)?;
    if report.schema_version != COMMAND_REPORT_SCHEMA_VERSION
        || report.command != command
        || report.status != status
    {
        return Err(report_invalid(format!(
            "{} report must use schema 1, command {command} and status {status}",
            check.as_str()
        )));
    }
    Ok(report)
}

fn parse_json<T: for<'de> Deserialize<'de>>(
    check: NativeGateCheckNameV1,
    bytes: &[u8],
) -> Result<T, NativeGateComparisonError> {
    serde_json::from_slice(bytes).map_err(|error| {
        report_invalid(format!(
            "invalid {} typed report JSON: {error}",
            check.as_str()
        ))
    })
}

fn validate_play_object_shape(bytes: &[u8]) -> Result<(), NativeGateComparisonError> {
    const FIELDS: [&str; 18] = [
        "schema_version",
        "status",
        "composition_root",
        "session_id",
        "close_receipt_hash",
        "close_result",
        "final_save_generation_hash",
        "project_composition_lock_hash",
        "ticks",
        "events",
        "rpg_events",
        "authoritative_revision",
        "authoritative_state_root",
        "command_archive_root",
        "command_identity_index_root",
        "command_ledger_hash",
        "interactive_host_object_count",
        "presentation",
    ];
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| report_invalid(format!("invalid play report JSON: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| report_invalid("play report must be a JSON object"))?;
    if object.len() != FIELDS.len() || FIELDS.iter().any(|field| !object.contains_key(*field)) {
        return Err(report_invalid(
            "play report fields do not exactly match RunReportV1",
        ));
    }
    if let Some(presentation) = object
        .get("presentation")
        .and_then(serde_json::Value::as_object)
    {
        const PRESENTATION_FIELDS: [&str; 3] = ["target", "snapshot_hash", "object_count"];
        if presentation.len() != PRESENTATION_FIELDS.len()
            || PRESENTATION_FIELDS
                .iter()
                .any(|field| !presentation.contains_key(*field))
        {
            return Err(report_invalid(
                "play presentation fields do not exactly match PresentationReportV1",
            ));
        }
    }
    Ok(())
}

fn validate_play_hashes(report: &GatePlayReportV1) -> Result<(), NativeGateComparisonError> {
    for (field, value) in [
        (
            "play project_composition_lock_hash",
            &report.project_composition_lock_hash,
        ),
        (
            "play authoritative_state_root",
            &report.authoritative_state_root,
        ),
        ("play command_ledger_hash", &report.command_ledger_hash),
    ] {
        validate_hash(field, value)?;
    }
    if report.close_result != "Saved" {
        return Err(report_invalid("play report close_result must be Saved"));
    }
    let _volatile_fields = (
        &report.session_id,
        &report.close_receipt_hash,
        &report.final_save_generation_hash,
        report.ticks,
        report.events,
        report.rpg_events,
        report.authoritative_revision,
        &report.command_archive_root,
        &report.command_identity_index_root,
        report.interactive_host_object_count,
    );
    if let Some(presentation) = &report.presentation {
        let _volatile_presentation = (
            &presentation.target,
            &presentation.snapshot_hash,
            presentation.object_count,
        );
    }
    Ok(())
}

fn validate_closure_hashes(details: &V1ClosureDetailsV1) -> Result<(), NativeGateComparisonError> {
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
        (
            "windows_package_descriptor_hash",
            &details.windows.package_descriptor_hash,
        ),
        (
            "linux_package_descriptor_hash",
            &details.linux.package_descriptor_hash,
        ),
        ("closure_hash", &details.closure_hash),
    ] {
        validate_hash(field, value)?;
    }
    Ok(())
}

fn validate_hash(field: &str, value: &str) -> Result<(), NativeGateComparisonError> {
    validate_lower_hex(field, value, 64)
}

fn validate_pass_bindings(
    target: &NativeGateTargetReportV1,
    reports: &[ValidatedCheckReportV1],
) -> Result<(), NativeGateComparisonError> {
    if reports.len() != NativeGateCheckNameV1::ORDERED.len() {
        return Err(report_invalid(
            "PASS target bundle does not contain all typed check reports",
        ));
    }
    let roots = target
        .comparable_roots
        .as_ref()
        .ok_or_else(|| report_invalid("PASS target report has no comparable roots"))?;
    let package = target
        .package
        .as_ref()
        .ok_or_else(|| report_invalid("PASS target report has no package summary"))?;

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
    bind(
        "play project composition",
        &play.project_composition_lock_hash,
        &roots.project_composition_lock_hash,
    )?;
    bind(
        "play state root",
        &play.authoritative_state_root,
        &roots.play_state_root,
    )?;
    bind(
        "play ledger hash",
        &play.command_ledger_hash,
        &roots.play_ledger_hash,
    )?;

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
    bind(
        "platform state root",
        &platform.details.state_root,
        &roots.platform_state_root,
    )?;
    bind(
        "platform ledger hash",
        &platform.details.ledger_hash,
        &roots.platform_ledger_hash,
    )?;
    bind(
        "presentation snapshot",
        &platform.details.presentation_snapshot_hash,
        &roots.presentation_snapshot_hash,
    )?;

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

    let ValidatedCheckReportV1::V1Closure(closure) = &reports[6] else {
        return Err(report_invalid("v1-closure typed report is out of order"));
    };
    validate_closure_bindings(target, &closure.details)?;

    let ValidatedCheckReportV1::V1Package(package_report) = &reports[7] else {
        return Err(report_invalid("v1-package typed report is out of order"));
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
            package.composition_lock_sha256.as_str(),
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

fn validate_closure_bindings(
    target: &NativeGateTargetReportV1,
    details: &V1ClosureDetailsV1,
) -> Result<(), NativeGateComparisonError> {
    let roots = target
        .comparable_roots
        .as_ref()
        .ok_or_else(|| report_invalid("PASS target report has no comparable roots"))?;
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

    let summaries = target
        .closure_targets
        .as_ref()
        .ok_or_else(|| report_invalid("PASS target report has no closure targets"))?;
    validate_closure_target_binding(
        "Windows",
        &details.windows,
        &summaries.windows.target_triple,
        &summaries.windows.package_descriptor_hash,
        &summaries.windows.runtime_check,
        &summaries.windows.desktop_smoke,
    )?;
    validate_closure_target_binding(
        "Linux",
        &details.linux,
        &summaries.linux.target_triple,
        &summaries.linux.package_descriptor_hash,
        &summaries.linux.runtime_check,
        &summaries.linux.desktop_smoke,
    )
}

fn validate_closure_target_binding(
    label: &str,
    details: &crate::report::TargetGateDetailsV1,
    target: &str,
    descriptor: &str,
    runtime: &NativeGateTargetExecutionStatusV1,
    smoke: &NativeGateTargetExecutionStatusV1,
) -> Result<(), NativeGateComparisonError> {
    bind(&format!("{label} target"), &details.target, target)?;
    bind(
        &format!("{label} package descriptor"),
        &details.package_descriptor_hash,
        descriptor,
    )?;
    bind(
        &format!("{label} runtime status"),
        &details.runtime_check,
        &target_status(runtime),
    )?;
    bind(
        &format!("{label} desktop status"),
        &details.desktop_smoke,
        &target_status(smoke),
    )
}

fn target_status(status: &NativeGateTargetExecutionStatusV1) -> String {
    match status {
        NativeGateTargetExecutionStatusV1::Pass => "PASS".to_owned(),
        NativeGateTargetExecutionStatusV1::NotRun { reason } => {
            format!("NOT_RUN({reason})")
        }
    }
}

fn bind(field: &str, actual: &str, expected: &str) -> Result<(), NativeGateComparisonError> {
    if actual == expected {
        Ok(())
    } else {
        Err(report_invalid(format!(
            "{field} does not match target report"
        )))
    }
}
