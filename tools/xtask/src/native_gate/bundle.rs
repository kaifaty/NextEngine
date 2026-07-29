use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::{
    NATIVE_GATE_REPORT_INVALID, NativeGateComparisonError, NativeGateRunStatusV1,
    NativeGateTargetReportV1, checked_bundle_metadata, report_invalid,
};

const MAX_BUNDLE_DEPTH: usize = 16;
const MAX_BUNDLE_ENTRIES: usize = 50_000;
const MAX_BUNDLE_FILE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_BUNDLE_TOTAL_BYTES: u64 = 8 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BundleEntryKind {
    Directory,
    File,
}

pub(super) fn validate_bundle_tree(
    bundle_root: &Path,
    report: &NativeGateTargetReportV1,
) -> Result<(), NativeGateComparisonError> {
    let root_metadata =
        checked_bundle_metadata(bundle_root, NATIVE_GATE_REPORT_INVALID, "bundle directory")?;
    if !root_metadata.is_dir() {
        return Err(report_invalid(format!(
            "target bundle root is not a directory: {}",
            bundle_root.display()
        )));
    }

    let mut inventory = BTreeMap::new();
    let mut total_bytes = 0_u64;
    walk_bundle(
        bundle_root,
        bundle_root,
        0,
        &mut inventory,
        &mut total_bytes,
    )?;
    validate_top_level_shape(&inventory, report.status)?;
    validate_checks_shape(&inventory, report)?;
    Ok(())
}

fn walk_bundle(
    bundle_root: &Path,
    directory: &Path,
    depth: usize,
    inventory: &mut BTreeMap<String, BundleEntryKind>,
    total_bytes: &mut u64,
) -> Result<(), NativeGateComparisonError> {
    if depth >= MAX_BUNDLE_DEPTH {
        return Err(report_invalid(format!(
            "bundle directory depth exceeds {MAX_BUNDLE_DEPTH}"
        )));
    }
    let entries = fs::read_dir(directory).map_err(|error| {
        report_invalid(format!(
            "failed to enumerate bundle directory {}: {error}",
            directory.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            report_invalid(format!(
                "failed to enumerate bundle directory {}: {error}",
                directory.display()
            ))
        })?;
        let path = entry.path();
        let metadata = checked_bundle_metadata(&path, NATIVE_GATE_REPORT_INVALID, "bundle entry")?;
        let relative = portable_relative_path(bundle_root, &path)?;
        let kind = if metadata.is_dir() {
            BundleEntryKind::Directory
        } else if metadata.is_file() {
            if metadata.len() > MAX_BUNDLE_FILE_BYTES {
                return Err(report_invalid(format!(
                    "bundle file {relative} exceeds {MAX_BUNDLE_FILE_BYTES} bytes"
                )));
            }
            *total_bytes = total_bytes.checked_add(metadata.len()).ok_or_else(|| {
                report_invalid("bundle file sizes overflow the total byte counter")
            })?;
            if *total_bytes > MAX_BUNDLE_TOTAL_BYTES {
                return Err(report_invalid(format!(
                    "bundle exceeds {MAX_BUNDLE_TOTAL_BYTES} total bytes"
                )));
            }
            BundleEntryKind::File
        } else {
            return Err(report_invalid(format!(
                "bundle entry {relative} is neither a regular file nor a directory"
            )));
        };
        if inventory.insert(relative.clone(), kind).is_some() {
            return Err(report_invalid(format!(
                "bundle contains duplicate path {relative}"
            )));
        }
        if inventory.len() > MAX_BUNDLE_ENTRIES {
            return Err(report_invalid(format!(
                "bundle contains more than {MAX_BUNDLE_ENTRIES} entries"
            )));
        }
        if kind == BundleEntryKind::Directory {
            walk_bundle(bundle_root, &path, depth + 1, inventory, total_bytes)?;
        }
    }
    Ok(())
}

fn portable_relative_path(
    bundle_root: &Path,
    path: &Path,
) -> Result<String, NativeGateComparisonError> {
    let relative = path
        .strip_prefix(bundle_root)
        .map_err(|_| report_invalid(format!("bundle entry escapes root: {}", path.display())))?;
    let mut components = Vec::new();
    for component in relative.components() {
        let std::path::Component::Normal(component) = component else {
            return Err(report_invalid(format!(
                "bundle entry is not a portable relative path: {}",
                path.display()
            )));
        };
        components.push(
            component
                .to_str()
                .ok_or_else(|| report_invalid("bundle path is not valid UTF-8"))?,
        );
    }
    if components.is_empty() {
        return Err(report_invalid("bundle inventory contains its root"));
    }
    Ok(components.join("/"))
}

fn validate_top_level_shape(
    inventory: &BTreeMap<String, BundleEntryKind>,
    status: NativeGateRunStatusV1,
) -> Result<(), NativeGateComparisonError> {
    let mut expected = BTreeMap::from([
        ("checks", BundleEntryKind::Directory),
        ("target-report.json", BundleEntryKind::File),
    ]);
    if status == NativeGateRunStatusV1::Pass {
        expected.insert("package", BundleEntryKind::Directory);
    }

    let actual = inventory
        .iter()
        .filter(|(path, _)| !path.contains('/'))
        .map(|(path, kind)| (path.as_str(), *kind))
        .collect::<BTreeMap<_, _>>();
    if actual != expected {
        return Err(report_invalid(format!(
            "{} target bundle has invalid top-level inventory",
            match status {
                NativeGateRunStatusV1::Pass => "PASS",
                NativeGateRunStatusV1::Fail => "FAIL",
            }
        )));
    }
    Ok(())
}

fn validate_checks_shape(
    inventory: &BTreeMap<String, BundleEntryKind>,
    report: &NativeGateTargetReportV1,
) -> Result<(), NativeGateComparisonError> {
    let expected = report
        .checks
        .iter()
        .filter_map(|record| record.report_path.clone())
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    for (path, kind) in inventory {
        let Some(remainder) = path.strip_prefix("checks/") else {
            continue;
        };
        if remainder.contains('/') || *kind != BundleEntryKind::File {
            return Err(report_invalid(format!(
                "checks inventory contains an unexpected entry: {path}"
            )));
        }
        actual.insert(path.clone());
    }
    if actual != expected {
        if let Some(missing) = expected.difference(&actual).next() {
            return Err(report_invalid(format!(
                "checks inventory is missing {missing}"
            )));
        }
        if let Some(extra) = actual.difference(&expected).next() {
            return Err(report_invalid(format!(
                "checks inventory contains unexpected file {extra}"
            )));
        }
        return Err(report_invalid("checks inventory mismatch"));
    }
    Ok(())
}
