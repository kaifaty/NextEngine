use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use super::{
    LINUX_TARGET_TRIPLE, NATIVE_GATE_COMMIT_MISMATCH, NATIVE_GATE_PACKAGE_INVALID,
    NATIVE_GATE_REPORT_INVALID, NATIVE_GATE_ROOT_MISMATCH, NATIVE_GATE_TARGET_SET_INVALID,
    NativeGateCheckStatusV1, NativeGateComparableRootsV1, NativeGateComparedTargetSummaryV1,
    NativeGateComparisonError, NativeGatePackageSummaryV1, NativeGatePackagedLaunchSummaryV1,
    NativeGateTargetReportV1, WINDOWS_TARGET_TRIPLE,
};

pub(super) fn validate_package_manifest_summary(
    report: &NativeGateTargetReportV1,
    summary: &NativeGatePackageSummaryV1,
    manifest: &crate::package::PackageManifestV3,
) -> Result<(), NativeGateComparisonError> {
    if manifest.target_triple != report.target_triple
        || manifest.target_triple != summary.target_triple
    {
        return Err(package_invalid(
            "package manifest target does not match target report and package summary",
        ));
    }

    let manifest_roots = &manifest.target_neutral_roots;
    for (field, manifest_value, summary_value) in [
        (
            "project_composition_lock_sha256",
            manifest_roots.project_composition_lock_sha256.as_str(),
            summary.composition_lock_sha256.as_str(),
        ),
        (
            "schema_registry_sha256",
            manifest_roots.schema_registry_sha256.as_str(),
            summary.schema_registry_sha256.as_str(),
        ),
        (
            "content_manifest_sha256",
            manifest_roots.content_manifest_sha256.as_str(),
            summary.content_manifest_sha256.as_str(),
        ),
        (
            "mechanics_lock_sha256",
            manifest_roots.mechanics_lock_sha256.as_str(),
            summary.mechanics_lock_sha256.as_str(),
        ),
        (
            "world_partition_sha256",
            manifest_roots.world_partition_sha256.as_str(),
            summary.world_partition_sha256.as_str(),
        ),
    ] {
        compare_package_summary_field(field, manifest_value, summary_value)?;
    }

    validate_packaged_run_summary(
        "game",
        &manifest.binaries.game,
        &summary.game_binary_sha256,
        &summary.game,
    )?;
    validate_packaged_run_summary(
        "headless",
        &manifest.binaries.headless,
        &summary.headless_binary_sha256,
        &summary.headless,
    )
}

pub(super) fn validate_packaged_run_summary(
    name: &str,
    manifest: &crate::package::PackagedRunV2,
    summary_binary_hash: &str,
    summary: &NativeGatePackagedLaunchSummaryV1,
) -> Result<(), NativeGateComparisonError> {
    compare_package_summary_field(
        &format!("{name}.binary_sha256"),
        &manifest.binary_sha256,
        summary_binary_hash,
    )?;
    compare_package_summary_field(
        &format!("{name}.state_root"),
        &manifest.authoritative_state_root,
        &summary.state_root,
    )?;
    compare_package_summary_field(
        &format!("{name}.ledger_hash"),
        &manifest.command_ledger_hash,
        &summary.ledger_hash,
    )?;
    if manifest.launch_status != "PASS" || summary.status != NativeGateCheckStatusV1::Pass {
        return Err(package_invalid(format!(
            "{name} launch status does not match PASS package summary"
        )));
    }
    Ok(())
}

pub(super) fn compare_package_summary_field(
    field: &str,
    manifest_value: &str,
    summary_value: &str,
) -> Result<(), NativeGateComparisonError> {
    if manifest_value == summary_value {
        Ok(())
    } else {
        Err(package_invalid(format!(
            "package manifest {field} does not match package summary"
        )))
    }
}

pub(super) fn validate_packaged_launch(
    name: &str,
    launch: &NativeGatePackagedLaunchSummaryV1,
) -> Result<(), NativeGateComparisonError> {
    if launch.status != NativeGateCheckStatusV1::Pass {
        return Err(package_invalid(format!("packaged {name} launch must PASS")));
    }
    validate_package_hash(&format!("{name}.state_root"), &launch.state_root)?;
    validate_package_hash(&format!("{name}.ledger_hash"), &launch.ledger_hash)
}

pub(super) fn compare_package_root(
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), NativeGateComparisonError> {
    if actual == expected {
        Ok(())
    } else {
        Err(package_invalid(format!(
            "{field} does not match comparable roots"
        )))
    }
}

pub(super) fn validate_package_hash(
    field: &str,
    value: &str,
) -> Result<(), NativeGateComparisonError> {
    if is_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(package_invalid(format!(
            "{field} must be 64 lowercase hexadecimal characters"
        )))
    }
}

pub(super) fn order_target_reports<'a>(
    first: &'a NativeGateTargetReportV1,
    second: &'a NativeGateTargetReportV1,
) -> Result<(&'a NativeGateTargetReportV1, &'a NativeGateTargetReportV1), NativeGateComparisonError>
{
    match (first.target_triple.as_str(), second.target_triple.as_str()) {
        (WINDOWS_TARGET_TRIPLE, LINUX_TARGET_TRIPLE) => Ok((first, second)),
        (LINUX_TARGET_TRIPLE, WINDOWS_TARGET_TRIPLE) => Ok((second, first)),
        _ => Err(NativeGateComparisonError::new(
            NATIVE_GATE_TARGET_SET_INVALID,
            format!(
                "expected one {WINDOWS_TARGET_TRIPLE} and one {LINUX_TARGET_TRIPLE}; found {} and {}",
                first.target_triple, second.target_triple
            ),
        )),
    }
}

pub(super) fn compare_environment_field(
    field: &str,
    windows: &str,
    linux: &str,
) -> Result<(), NativeGateComparisonError> {
    if windows == linux {
        Ok(())
    } else {
        Err(NativeGateComparisonError::new(
            NATIVE_GATE_COMMIT_MISMATCH,
            format!("{field} differs between Windows and Linux reports"),
        ))
    }
}

pub(super) fn compare_roots(
    windows: &NativeGateComparableRootsV1,
    linux: &NativeGateComparableRootsV1,
) -> Result<(), NativeGateComparisonError> {
    for ((windows_field, windows_value), (linux_field, linux_value)) in
        comparable_root_fields(windows)
            .into_iter()
            .zip(comparable_root_fields(linux))
    {
        debug_assert_eq!(windows_field, linux_field);
        if windows_value != linux_value {
            return Err(NativeGateComparisonError::new(
                NATIVE_GATE_ROOT_MISMATCH,
                format!("{windows_field} differs: Windows={windows_value}, Linux={linux_value}"),
            ));
        }
    }
    Ok(())
}

pub(super) fn comparable_root_fields(
    roots: &NativeGateComparableRootsV1,
) -> [(&'static str, &str); 26] {
    [
        (
            "project_composition_lock_hash",
            &roots.project_composition_lock_hash,
        ),
        ("schema_registry_hash", &roots.schema_registry_hash),
        ("content_manifest_hash", &roots.content_manifest_hash),
        ("mechanics_lock_hash", &roots.mechanics_lock_hash),
        ("world_partition_hash", &roots.world_partition_hash),
        ("luau_manifest_hash", &roots.luau_manifest_hash),
        ("wasm_manifest_hash", &roots.wasm_manifest_hash),
        ("wit_v2_hash", &roots.wit_v2_hash),
        ("wit_v3_hash", &roots.wit_v3_hash),
        (
            "extension_compatibility_hash",
            &roots.extension_compatibility_hash,
        ),
        ("play_state_root", &roots.play_state_root),
        ("play_ledger_hash", &roots.play_ledger_hash),
        ("replay_state_root", &roots.replay_state_root),
        ("replay_ledger_hash", &roots.replay_ledger_hash),
        ("platform_state_root", &roots.platform_state_root),
        ("platform_ledger_hash", &roots.platform_ledger_hash),
        (
            "presentation_snapshot_hash",
            &roots.presentation_snapshot_hash,
        ),
        (
            "streaming_performance_hash",
            &roots.streaming_performance_hash,
        ),
        ("agent_performance_hash", &roots.agent_performance_hash),
        ("packaged_game_state_root", &roots.packaged_game_state_root),
        (
            "packaged_game_ledger_hash",
            &roots.packaged_game_ledger_hash,
        ),
        (
            "packaged_headless_state_root",
            &roots.packaged_headless_state_root,
        ),
        (
            "packaged_headless_ledger_hash",
            &roots.packaged_headless_ledger_hash,
        ),
        ("closure_hash", &roots.closure_hash),
        (
            "windows_package_descriptor_hash",
            &roots.windows_package_descriptor_hash,
        ),
        (
            "linux_package_descriptor_hash",
            &roots.linux_package_descriptor_hash,
        ),
    ]
}

pub(super) fn compared_target_summary(
    report: &NativeGateTargetReportV1,
) -> Result<NativeGateComparedTargetSummaryV1, NativeGateComparisonError> {
    let package = report.package.as_ref().ok_or_else(|| {
        NativeGateComparisonError::new(
            NATIVE_GATE_PACKAGE_INVALID,
            format!("{} report has no package summary", report.target_triple),
        )
    })?;
    Ok(NativeGateComparedTargetSummaryV1 {
        target_triple: report.target_triple.clone(),
        package_manifest_sha256: package.package_manifest_sha256.clone(),
        game_binary_sha256: package.game_binary_sha256.clone(),
        headless_binary_sha256: package.headless_binary_sha256.clone(),
        total_elapsed_milliseconds: report.checks.iter().fold(0_u64, |total, record| {
            total.saturating_add(record.elapsed_milliseconds)
        }),
    })
}

pub(super) fn validate_lower_hex(
    field: &str,
    value: &str,
    expected_length: usize,
) -> Result<(), NativeGateComparisonError> {
    if is_lower_hex(value, expected_length) {
        Ok(())
    } else {
        Err(report_invalid(format!(
            "{field} must be {expected_length} lowercase hexadecimal characters"
        )))
    }
}

pub(super) fn is_lower_hex(value: &str, expected_length: usize) -> bool {
    value.len() == expected_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn is_shipping_target(target: &str) -> bool {
    matches!(target, WINDOWS_TARGET_TRIPLE | LINUX_TARGET_TRIPLE)
}

pub(super) fn is_portable_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

pub(super) fn read_bounded_regular_file(
    path: &Path,
    maximum_bytes: usize,
    code: &'static str,
    label: &str,
) -> Result<Vec<u8>, NativeGateComparisonError> {
    let metadata = checked_bundle_metadata(path, code, label)?;
    if !metadata.is_file() {
        return Err(NativeGateComparisonError::new(
            code,
            format!("{label} is not a regular file: {}", path.display()),
        ));
    }
    let length = usize::try_from(metadata.len()).map_err(|_| {
        NativeGateComparisonError::new(code, format!("{label} length does not fit usize"))
    })?;
    if length > maximum_bytes {
        return Err(NativeGateComparisonError::new(
            code,
            format!("{label} has {length} bytes; limit is {maximum_bytes}"),
        ));
    }
    let file = File::open(path).map_err(|error| {
        NativeGateComparisonError::new(
            code,
            format!("failed to read {label} {}: {error}", path.display()),
        )
    })?;
    let read_limit = u64::try_from(maximum_bytes)
        .unwrap_or(u64::MAX - 1)
        .saturating_add(1);
    let mut bytes = Vec::with_capacity(length.min(maximum_bytes));
    file.take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            NativeGateComparisonError::new(
                code,
                format!("failed to read {label} {}: {error}", path.display()),
            )
        })?;
    if bytes.len() > maximum_bytes {
        return Err(NativeGateComparisonError::new(
            code,
            format!("{label} grew beyond the {maximum_bytes}-byte limit"),
        ));
    }
    Ok(bytes)
}

pub(super) fn checked_bundle_metadata(
    path: &Path,
    code: &'static str,
    label: &str,
) -> Result<fs::Metadata, NativeGateComparisonError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        NativeGateComparisonError::new(
            code,
            format!("failed to inspect {label} {}: {error}", path.display()),
        )
    })?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err(NativeGateComparisonError::new(
            code,
            format!("{label} must not be a symlink or reparse point"),
        ));
    }
    Ok(metadata)
}

#[cfg(windows)]
pub(super) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    next_contracts::ids::content_hash_from_bytes(next_contracts::canonical::sha256(bytes)).to_hex()
}

pub(super) fn target_package_descriptor_hash(
    target_triple: &str,
    roots: &NativeGateComparableRootsV1,
) -> String {
    let mut bytes = b"nextengine.v1-target-package-descriptor.v1\0".to_vec();
    extend_hash_text(&mut bytes, target_triple);
    extend_hash_text(&mut bytes, env!("CARGO_PKG_VERSION"));
    for root in [
        &roots.project_composition_lock_hash,
        &roots.schema_registry_hash,
        &roots.content_manifest_hash,
        &roots.mechanics_lock_hash,
        &roots.world_partition_hash,
        &roots.extension_compatibility_hash,
    ] {
        bytes.extend_from_slice(
            &decode_hash(root).expect("validated comparable root must decode as SHA-256"),
        );
    }
    extend_hash_text(&mut bytes, "next_game");
    extend_hash_text(&mut bytes, "next_headless");
    sha256_hex(&bytes)
}

pub(super) fn closure_hash(roots: &NativeGateComparableRootsV1) -> String {
    let mut bytes = b"nextengine.v1-closure.v1\0".to_vec();
    for root in [
        &roots.project_composition_lock_hash,
        &roots.schema_registry_hash,
        &roots.content_manifest_hash,
        &roots.mechanics_lock_hash,
        &roots.world_partition_hash,
        &roots.play_state_root,
        &roots.play_ledger_hash,
        &roots.replay_state_root,
        &roots.replay_ledger_hash,
        &roots.streaming_performance_hash,
        &roots.agent_performance_hash,
        &roots.extension_compatibility_hash,
        &roots.windows_package_descriptor_hash,
        &roots.linux_package_descriptor_hash,
    ] {
        bytes.extend_from_slice(
            &decode_hash(root).expect("validated comparable root must decode as SHA-256"),
        );
    }
    sha256_hex(&bytes)
}

fn extend_hash_text(bytes: &mut Vec<u8>, value: &str) {
    let length = u32::try_from(value.len()).expect("static native-gate label fits u32");
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn decode_hash(value: &str) -> Option<[u8; 32]> {
    if !is_lower_hex(value, 64) {
        return None;
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Some(bytes)
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

pub(super) fn package_validation_error(error: String) -> NativeGateComparisonError {
    let detail = error
        .strip_prefix("NATIVE_GATE_PACKAGE_INVALID: ")
        .unwrap_or(&error)
        .to_owned();
    package_invalid(detail)
}

pub(super) fn report_invalid(detail: impl Into<String>) -> NativeGateComparisonError {
    NativeGateComparisonError::new(NATIVE_GATE_REPORT_INVALID, detail)
}

pub(super) fn package_invalid(detail: impl Into<String>) -> NativeGateComparisonError {
    NativeGateComparisonError::new(NATIVE_GATE_PACKAGE_INVALID, detail)
}
