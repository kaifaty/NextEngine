use std::fs;
use std::path::Path;

use super::{
    PackageTargetNeutralRootsV3, PackagedToolValidationV1, checked_metadata, package_error,
};

pub(super) const REFERENCE_SOURCE_PATH: &str = "source/reference-alpha";
pub(super) const REFERENCE_SOURCE_FILES: [&str; 13] = [
    "ACCEPTANCE.md",
    "NOTICE",
    "assets/humanoid-cc0.catalog.json",
    // Scene look L5 (plan `look/05`): the procedural texture sets the
    // authoring manifest references.
    "assets/textures/concrete_albedo.png",
    "assets/textures/concrete_metallic_roughness.png",
    "assets/textures/concrete_normal.png",
    "assets/textures/ground_albedo.png",
    "assets/textures/ground_metallic_roughness.png",
    "assets/textures/ground_normal.png",
    "assets/textures/pad_albedo.png",
    "assets/textures/pad_metallic_roughness.png",
    "assets/textures/pad_normal.png",
    "project.authoring.json",
];

pub(super) fn copy_reference_project_source(
    repository_root: &Path,
    package_root: &Path,
) -> Result<(), String> {
    let repository_source = repository_root.join("projects/reference-alpha");
    let package_source = package_root.join(REFERENCE_SOURCE_PATH);
    fs::create_dir_all(package_source.join("assets/textures")).map_err(|error| {
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
) -> Result<(next_project::CookedProjectV7, u32), String> {
    validate_reference_source_layout(package_root)?;
    let source = next_project::load_project_authoring_v7(package_root.join(REFERENCE_SOURCE_PATH))
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    let neutral_record_count = u32::try_from(source.records.len())
        .map_err(|_| "NATIVE_GATE_PACKAGE_INVALID: neutral record count overflow".to_owned())?;
    let cooked = next_project::cook_project_v7(source)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    Ok((cooked, neutral_record_count))
}

pub(super) fn validate_reference_project_source(
    package_root: &Path,
    expected: &PackageTargetNeutralRootsV3,
    tools: &PackagedToolValidationV1,
) -> Result<(), String> {
    let (cooked, neutral_record_count) = cook_packaged_reference_source(package_root)?;
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
    let render = &cooked.render_content_catalog;
    let render_asset_count = 1_usize
        .checked_add(render.meshes().len())
        .and_then(|count| count.checked_add(render.materials().len()))
        .and_then(|count| count.checked_add(render.textures().len()))
        .and_then(|count| count.checked_add(render.base_skinning_profiles().len()))
        .ok_or_else(|| "NATIVE_GATE_PACKAGE_INVALID: render asset count overflow".to_owned())?;
    let publication_file_count = cooked
        .publication()
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?
        .files
        .len();
    let expected_counts = [
        (
            "root asset",
            tools.root_asset_count,
            count(cooked.content_manifest.body.root_assets.len())?,
        ),
        (
            "content entry",
            tools.content_entry_count,
            count(cooked.content_manifest.body.asset_entries.len())?,
        ),
        (
            "neutral record",
            tools.neutral_record_count,
            neutral_record_count,
        ),
        (
            "render asset",
            tools.render_asset_count,
            count(render_asset_count)?,
        ),
        (
            "world chunk",
            tools.world_chunk_count,
            count(cooked.world_partition.body.chunk_bindings.len())?,
        ),
        (
            "publication file",
            tools.publication_file_count,
            count(publication_file_count)?,
        ),
    ];
    if tools.project_id != cooked.project_lock.project_id.as_str()
        || tools.project_revision != cooked.project_lock.project_revision
        || tools.authoring_sha256 != cooked.project_lock.authoring_sha256.to_hex()
        || tools.project_composition_lock_hash != cooked.project_lock.project_lock_sha256.to_hex()
        || tools.publication_state != "validated-not-written"
        || expected_counts
            .into_iter()
            .any(|(_, reported, actual)| reported != actual)
    {
        return package_error(
            "tools validation receipt does not match the frozen reference source",
        );
    }
    Ok(())
}

fn count(value: usize) -> Result<u32, String> {
    u32::try_from(value)
        .map_err(|_| "NATIVE_GATE_PACKAGE_INVALID: source count overflow".to_owned())
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
