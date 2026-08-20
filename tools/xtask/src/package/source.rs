use std::fs;
use std::path::Path;

use super::{PackageTargetNeutralRootsV3, checked_metadata, package_error};

pub(super) const REFERENCE_SOURCE_PATH: &str = "source/reference-alpha";
pub(super) const REFERENCE_SOURCE_FILES: [&str; 4] = [
    "ACCEPTANCE.md",
    "NOTICE",
    "assets/humanoid-cc0.catalog.json",
    "project.authoring.json",
];

pub(super) fn copy_reference_project_source(
    repository_root: &Path,
    package_root: &Path,
) -> Result<(), String> {
    let repository_source = repository_root.join("projects/reference-alpha");
    let package_source = package_root.join(REFERENCE_SOURCE_PATH);
    fs::create_dir_all(package_source.join("assets")).map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: failed to create packaged source: {error}")
    })?;
    for relative in REFERENCE_SOURCE_FILES {
        let source = repository_source.join(relative);
        if !checked_metadata(&source)?.is_file() {
            return package_error(format!(
                "reference source {} is not a regular file",
                source.display()
            ));
        }
        fs::copy(&source, package_source.join(relative)).map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: failed to freeze reference source {relative}: {error}"
            )
        })?;
    }
    Ok(())
}

pub(super) fn cook_packaged_reference_source(
    package_root: &Path,
) -> Result<next_project::CookedProjectV7, String> {
    validate_reference_source_layout(package_root)?;
    let source = next_project::load_project_authoring_v7(package_root.join(REFERENCE_SOURCE_PATH))
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    next_project::cook_project_v7(source)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))
}

pub(super) fn validate_reference_project_source(
    package_root: &Path,
    expected: &PackageTargetNeutralRootsV3,
) -> Result<(), String> {
    let cooked = cook_packaged_reference_source(package_root)?;
    let actual = PackageTargetNeutralRootsV3 {
        content_manifest_sha256: cooked.content_manifest.content_manifest_sha256.to_hex(),
        mechanics_lock_sha256: cooked
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
        project_lock_sha256: cooked.project_lock.project_lock_sha256.to_hex(),
        schema_registry_sha256: cooked
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        world_partition_sha256: cooked
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
    };
    if &actual != expected {
        return package_error("frozen reference source does not reproduce manifest roots");
    }
    Ok(())
}

fn validate_reference_source_layout(package_root: &Path) -> Result<(), String> {
    let source = package_root.join(REFERENCE_SOURCE_PATH);
    let expected = REFERENCE_SOURCE_FILES
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut actual = Vec::new();
    collect_files(&source, &source, &mut actual)?;
    actual.sort();
    if actual != expected {
        return package_error("frozen reference source layout is not exact");
    }
    Ok(())
}

fn collect_files(root: &Path, directory: &Path, output: &mut Vec<String>) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: failed to inspect frozen source {}: {error}",
                directory.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = checked_metadata(&path)?;
        if metadata.is_dir() {
            collect_files(root, &path, output)?;
        } else if metadata.is_file() {
            output.push(portable_relative(root, &path)?);
        } else {
            return package_error(format!(
                "frozen source object {} is not a regular file or directory",
                path.display()
            ));
        }
    }
    Ok(())
}

fn portable_relative(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        "NATIVE_GATE_PACKAGE_INVALID: frozen source traversal escaped its root".to_owned()
    })?;
    let components = relative
        .components()
        .map(|component| {
            component.as_os_str().to_str().ok_or_else(|| {
                "NATIVE_GATE_PACKAGE_INVALID: frozen source path is not UTF-8".to_owned()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(components.join("/"))
}
