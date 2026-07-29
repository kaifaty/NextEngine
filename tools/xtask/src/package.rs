use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

mod inventory;
mod runtime;
mod smoke;

use inventory::{
    checked_metadata, collect_inventory, hash_file, read_bounded, validate_package_root,
    validate_project_store_layout, validate_relative_package_path,
};
use smoke::run_packaged_binary;
#[cfg(test)]
use smoke::{
    LINUX_DYNAMIC_LOADER_FAILURE_EXIT_CODE, WINDOWS_STATUS_DLL_NOT_FOUND,
    WINDOWS_STATUS_ENTRYPOINT_NOT_FOUND, WINDOWS_STATUS_INVALID_IMAGE_FORMAT,
    WINDOWS_STATUS_INVALID_IMAGE_LE_FORMAT, WINDOWS_STATUS_INVALID_IMAGE_NOT_MZ,
    WINDOWS_STATUS_INVALID_IMAGE_WIN_16, WINDOWS_STATUS_ORDINAL_NOT_FOUND,
    configure_smoke_environment, run_packaged_binary_with_timeout,
    runtime_prerequisite_failure_code,
};

pub const PACKAGE_MANIFEST_FILE: &str = "package.manifest.jcs";
pub const PACKAGE_MANIFEST_SCHEMA_VERSION: u32 = 3;

const MAX_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
const REQUIRED_NOTICE_PATHS: [&str; 4] = [
    "LICENSE",
    "MIGRATION_PROVENANCE.md",
    "NOTICE",
    "THIRD_PARTY_NOTICES.md",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageManifestV3 {
    pub binaries: PackageBinariesV2,
    pub file_inventory: Vec<PackageFileV2>,
    pub required_notices: Vec<String>,
    pub runtime_profile: PackageRuntimeProfileV3,
    pub schema_version: u32,
    pub target_neutral_roots: PackageTargetNeutralRootsV2,
    pub target_triple: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageRuntimeProfileV3 {
    pub abi: PackageRuntimeAbiV3,
    pub binaries: Vec<PackageBinaryRuntimeV3>,
    pub external_prerequisites: Vec<PackageExternalPrerequisiteV3>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PackageRuntimeAbiV3 {
    WindowsMsvcX64 { crt: PackageWindowsCrtV3 },
    LinuxGnuX64 { minimum_glibc: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageWindowsCrtV3 {
    DynamicSystem,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageBinaryRuntimeV3 {
    pub binary_path: String,
    pub direct_libraries: Vec<String>,
    pub maximum_required_glibc: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageExternalPrerequisiteV3 {
    pub diagnostic_code: String,
    pub id: String,
    pub locator: String,
    pub requirement: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageBinariesV2 {
    pub game: PackagedRunV2,
    pub headless: PackagedRunV2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackagedRunV2 {
    pub authoritative_state_root: String,
    pub binary_path: String,
    pub binary_sha256: String,
    pub command_ledger_hash: String,
    pub composition_root: String,
    pub launch_status: String,
    pub project_composition_lock_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageFileV2 {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageTargetNeutralRootsV2 {
    pub content_manifest_sha256: String,
    pub mechanics_lock_sha256: String,
    pub project_composition_lock_sha256: String,
    pub schema_registry_sha256: String,
    pub world_partition_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageBuildResult {
    pub manifest: PackageManifestV3,
    pub output: PathBuf,
    pub package_manifest_sha256: String,
}

pub fn build_v1_package(
    repository_root: &Path,
    requested_output: &Path,
) -> Result<PackageBuildResult, String> {
    build_v1_package_with_binary_sources(
        repository_root,
        requested_output,
        prepare_release_binary_sources,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PackageBinarySources {
    game: PathBuf,
    headless: PathBuf,
}

fn build_v1_package_with_binary_sources<F>(
    repository_root: &Path,
    requested_output: &Path,
    prepare_binary_sources: F,
) -> Result<PackageBuildResult, String>
where
    F: FnOnce(&Path, &str) -> Result<PackageBinarySources, String>,
{
    let repository_root = fs::canonicalize(repository_root).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to resolve repository root {}: {error}",
            repository_root.display()
        )
    })?;
    let target_triple = native_shipping_target_triple()?;
    let output = absolute_from(&repository_root, requested_output);
    ensure_publish_destination_absent(&output)?;
    let parent = output
        .parent()
        .ok_or_else(|| "NATIVE_GATE_PACKAGE_INVALID: package output has no parent".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to create package parent {}: {error}",
            parent.display()
        )
    })?;
    let _publish_lock = PublishLock::acquire(&output)?;
    ensure_publish_destination_absent(&output)?;
    let output_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            "NATIVE_GATE_PACKAGE_INVALID: package output needs a UTF-8 file name".to_owned()
        })?;
    let staging = parent.join(format!(".{output_name}.staging-{}", std::process::id()));
    let smoke_root = parent.join(format!(".{output_name}.smoke-{}", std::process::id()));
    if path_exists_without_following(&staging)? || path_exists_without_following(&smoke_root)? {
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: package staging or smoke path already exists for {}",
            output.display()
        ));
    }

    fs::create_dir(&staging).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to create package staging {}: {error}",
            staging.display()
        )
    })?;

    let staged_result =
        prepare_binary_sources(&repository_root, target_triple).and_then(|binary_sources| {
            build_staged_package(
                &repository_root,
                &staging,
                &smoke_root,
                target_triple,
                &binary_sources,
            )
        });
    let smoke_cleanup = remove_directory_if_present(&smoke_root);

    let (manifest, manifest_bytes) = match (staged_result, smoke_cleanup) {
        (Ok(value), Ok(())) => value,
        (Err(error), Ok(())) | (Ok(_), Err(error)) => {
            return cleanup_failed_build(&staging, error);
        }
        (Err(error), Err(cleanup_error)) => {
            return cleanup_failed_build(
                &staging,
                format!("{error}; smoke cleanup also failed: {cleanup_error}"),
            );
        }
    };

    ensure_publish_destination_absent(&output)?;
    if let Err(error) = fs::rename(&staging, &output) {
        return cleanup_failed_build(
            &staging,
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: failed to atomically publish {}: {error}",
                output.display()
            ),
        );
    }

    Ok(PackageBuildResult {
        manifest,
        output,
        package_manifest_sha256: hash_bytes(&manifest_bytes),
    })
}

pub fn validate_v1_package(package_root: &Path) -> Result<PackageManifestV3, String> {
    validate_package_root(package_root)?;
    let manifest_path = package_root.join(PACKAGE_MANIFEST_FILE);
    let manifest_metadata = checked_metadata(&manifest_path)?;
    if !manifest_metadata.is_file() {
        return package_error(format!("{} is not a regular file", manifest_path.display()));
    }
    let manifest_length = usize::try_from(manifest_metadata.len())
        .map_err(|_| "NATIVE_GATE_PACKAGE_INVALID: manifest is too large".to_owned())?;
    if manifest_length > MAX_MANIFEST_BYTES {
        return package_error(format!(
            "manifest has {manifest_length} bytes; limit is {MAX_MANIFEST_BYTES}"
        ));
    }
    let manifest_bytes = read_bounded(&manifest_path, MAX_MANIFEST_BYTES as u64)?;
    let manifest: PackageManifestV3 = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: invalid manifest JSON: {error}"))?;
    let canonical = canonical_json_bytes(&manifest)?;
    if manifest_bytes != canonical {
        return package_error("package manifest is not canonical JSON");
    }

    validate_manifest_fields(&manifest)?;
    validate_project_store_layout(package_root)?;
    let actual_inventory = collect_inventory(package_root)?;
    if manifest.file_inventory != actual_inventory {
        return package_error("package file inventory does not exactly match package files");
    }
    validate_binary_inventory(&manifest)?;
    runtime::validate_runtime_profile(
        package_root,
        &manifest.target_triple,
        &manifest.runtime_profile,
        &[
            manifest.binaries.game.binary_path.as_str(),
            manifest.binaries.headless.binary_path.as_str(),
        ],
    )?;
    validate_activated_project(package_root, &manifest.target_neutral_roots)?;
    Ok(manifest)
}

fn build_staged_package(
    repository_root: &Path,
    staging: &Path,
    smoke_root: &Path,
    target_triple: &str,
    binary_sources: &PackageBinarySources,
) -> Result<(PackageManifestV3, Vec<u8>), String> {
    let bin_directory = staging.join("bin");
    fs::create_dir(&bin_directory).map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: failed to create package bin: {error}")
    })?;
    let project_directory = staging.join("project");
    let source = next_reference_game::project_source_v2()
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    let cooked = next_project::cook_project_v1(source)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    let project_store = next_assets::ContentStore::new(&project_directory);
    let publication = cooked
        .publication()
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    project_store
        .publish(&publication)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    let activated = next_project::activate_project(&project_store)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    if activated.composition_lock.composition_lock_sha256
        != cooked.composition_lock.composition_lock_sha256
    {
        return package_error("packaged project activation lock mismatch");
    }

    copy_required_notices(repository_root, staging)?;

    let executable_suffix = if target_triple == "x86_64-pc-windows-msvc" {
        ".exe"
    } else {
        ""
    };
    let game_name = format!("next_game{executable_suffix}");
    let headless_name = format!("next_headless{executable_suffix}");
    let game_destination = bin_directory.join(&game_name);
    let headless_destination = bin_directory.join(&headless_name);
    copy_binary(&binary_sources.game, &game_destination)?;
    copy_binary(&binary_sources.headless, &headless_destination)?;
    let game_binary_path = format!("bin/{game_name}");
    let headless_binary_path = format!("bin/{headless_name}");
    let runtime_profile = runtime::build_runtime_profile(
        staging,
        target_triple,
        &[game_binary_path.as_str(), headless_binary_path.as_str()],
    )?;
    let inventory_before_smoke = collect_inventory(staging)?;

    fs::create_dir(smoke_root).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to create disposable smoke root {}: {error}",
            smoke_root.display()
        )
    })?;
    let project_lock = cooked.composition_lock.composition_lock_sha256.to_hex();
    let headless_report = run_packaged_binary(
        &headless_destination,
        &["--project", "project", "--lock", &project_lock],
        &smoke_root.join("headless"),
        staging,
        "Headless",
        &project_lock,
    )?;
    let game_report = run_packaged_binary(
        &game_destination,
        &[
            "--interactive",
            "--maximum-frames",
            "1",
            "--project",
            "project",
            "--lock",
            &project_lock,
        ],
        &smoke_root.join("game"),
        staging,
        "Game",
        &project_lock,
    )?;
    if game_report.authoritative_state_root != headless_report.authoritative_state_root
        || game_report.command_ledger_hash != headless_report.command_ledger_hash
    {
        return package_error("packaged game/headless state or ledger parity mismatch");
    }
    let inventory_after_smoke = collect_inventory(staging)?;
    ensure_inventory_unchanged(&inventory_before_smoke, &inventory_after_smoke)?;

    let roots = PackageTargetNeutralRootsV2 {
        content_manifest_sha256: cooked.content_manifest.content_manifest_sha256.to_hex(),
        mechanics_lock_sha256: cooked
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
        project_composition_lock_sha256: project_lock,
        schema_registry_sha256: cooked
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        world_partition_sha256: cooked
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
    };
    let game = packaged_run("bin", &game_name, &game_destination, game_report)?;
    let headless = packaged_run(
        "bin",
        &headless_name,
        &headless_destination,
        headless_report,
    )?;
    let manifest = PackageManifestV3 {
        binaries: PackageBinariesV2 { game, headless },
        file_inventory: inventory_after_smoke,
        required_notices: required_notice_paths(),
        runtime_profile,
        schema_version: PACKAGE_MANIFEST_SCHEMA_VERSION,
        target_neutral_roots: roots,
        target_triple: target_triple.to_owned(),
    };
    let bytes = canonical_json_bytes(&manifest)?;
    fs::write(staging.join(PACKAGE_MANIFEST_FILE), &bytes).map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: failed to write package manifest: {error}")
    })?;
    let validated = validate_v1_package(staging)?;
    if validated != manifest {
        return package_error("package changed while it was being validated");
    }
    Ok((manifest, bytes))
}

fn prepare_release_binary_sources(
    repository_root: &Path,
    target_triple: &str,
) -> Result<PackageBinarySources, String> {
    run_checked(
        repository_root,
        "cargo",
        &[
            "build",
            "--locked",
            "--release",
            "--target",
            target_triple,
            "-p",
            "next_game",
            "-p",
            "next_headless",
            "--features",
            "next_game/desktop-sdl-ash",
        ],
    )?;

    let executable_suffix = if target_triple == "x86_64-pc-windows-msvc" {
        ".exe"
    } else {
        ""
    };
    let game_name = format!("next_game{executable_suffix}");
    let headless_name = format!("next_headless{executable_suffix}");
    let release_directory =
        release_binary_directory(&cargo_target_directory(repository_root), target_triple);
    Ok(PackageBinarySources {
        game: release_directory.join(game_name),
        headless: release_directory.join(headless_name),
    })
}

fn packaged_run(
    binary_directory: &str,
    binary_name: &str,
    binary: &Path,
    report: next_application::RunReportV1,
) -> Result<PackagedRunV2, String> {
    Ok(PackagedRunV2 {
        authoritative_state_root: report.authoritative_state_root,
        binary_path: format!("{binary_directory}/{binary_name}"),
        binary_sha256: hash_file(binary)?,
        command_ledger_hash: report.command_ledger_hash,
        composition_root: report.composition_root,
        launch_status: report.status,
        project_composition_lock_hash: report.project_composition_lock_hash,
    })
}

fn validate_manifest_fields(manifest: &PackageManifestV3) -> Result<(), String> {
    if manifest.schema_version != PACKAGE_MANIFEST_SCHEMA_VERSION {
        return package_error(format!(
            "unsupported package schema version {}",
            manifest.schema_version
        ));
    }
    if !matches!(
        manifest.target_triple.as_str(),
        "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu"
    ) {
        return package_error(format!(
            "unsupported package target {}",
            manifest.target_triple
        ));
    }
    if manifest.required_notices != required_notice_paths() {
        return package_error("required notice list is incomplete or not canonical");
    }

    let roots = &manifest.target_neutral_roots;
    for (name, hash) in [
        ("content manifest", roots.content_manifest_sha256.as_str()),
        ("mechanics lock", roots.mechanics_lock_sha256.as_str()),
        (
            "project composition lock",
            roots.project_composition_lock_sha256.as_str(),
        ),
        ("schema registry", roots.schema_registry_sha256.as_str()),
        ("world partition", roots.world_partition_sha256.as_str()),
    ] {
        validate_hash(name, hash)?;
    }

    validate_packaged_run(
        &manifest.binaries.game,
        "Game",
        &roots.project_composition_lock_sha256,
    )?;
    validate_packaged_run(
        &manifest.binaries.headless,
        "Headless",
        &roots.project_composition_lock_sha256,
    )?;
    if manifest.binaries.game.binary_path == manifest.binaries.headless.binary_path {
        return package_error("game and headless binary paths must differ");
    }
    if manifest.binaries.game.authoritative_state_root
        != manifest.binaries.headless.authoritative_state_root
        || manifest.binaries.game.command_ledger_hash
            != manifest.binaries.headless.command_ledger_hash
    {
        return package_error("packaged game/headless state or ledger roots differ");
    }
    let expected_suffix = if manifest.target_triple == "x86_64-pc-windows-msvc" {
        ".exe"
    } else {
        ""
    };
    if manifest.binaries.game.binary_path != format!("bin/next_game{expected_suffix}")
        || manifest.binaries.headless.binary_path != format!("bin/next_headless{expected_suffix}")
    {
        return package_error("binary paths do not match the package target");
    }

    if manifest
        .file_inventory
        .windows(2)
        .any(|pair| pair[0].path >= pair[1].path)
    {
        return package_error("file inventory paths must be unique and strictly sorted");
    }
    for file in &manifest.file_inventory {
        validate_relative_package_path(&file.path)?;
        validate_hash("inventory file", &file.sha256)?;
    }
    Ok(())
}

fn validate_packaged_run(
    run: &PackagedRunV2,
    expected_root: &str,
    expected_project_lock: &str,
) -> Result<(), String> {
    validate_relative_package_path(&run.binary_path)?;
    validate_hash("binary", &run.binary_sha256)?;
    validate_hash(
        "run authoritative state root",
        &run.authoritative_state_root,
    )?;
    validate_hash("run command ledger hash", &run.command_ledger_hash)?;
    validate_hash(
        "run project composition lock",
        &run.project_composition_lock_hash,
    )?;
    if run.launch_status != "PASS"
        || run.composition_root != expected_root
        || run.project_composition_lock_hash != expected_project_lock
    {
        return package_error(format!("{expected_root} packaged run summary is invalid"));
    }
    Ok(())
}

fn validate_binary_inventory(manifest: &PackageManifestV3) -> Result<(), String> {
    for run in [&manifest.binaries.game, &manifest.binaries.headless] {
        let entry = manifest
            .file_inventory
            .iter()
            .find(|entry| entry.path == run.binary_path)
            .ok_or_else(|| {
                format!(
                    "NATIVE_GATE_PACKAGE_INVALID: binary {} is absent from inventory",
                    run.binary_path
                )
            })?;
        if entry.sha256 != run.binary_sha256 {
            return package_error(format!(
                "binary hash for {} does not match inventory",
                run.binary_path
            ));
        }
    }
    Ok(())
}

fn ensure_inventory_unchanged(
    before: &[PackageFileV2],
    after: &[PackageFileV2],
) -> Result<(), String> {
    if before != after {
        return package_error("package bytes changed while copied binaries were smoke-tested");
    }
    Ok(())
}

fn validate_activated_project(
    package_root: &Path,
    roots: &PackageTargetNeutralRootsV2,
) -> Result<(), String> {
    let store = next_assets::ContentStore::new(package_root.join("project"));
    let activated = next_project::activate_project(&store)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    let actual = PackageTargetNeutralRootsV2 {
        content_manifest_sha256: activated.content_manifest.content_manifest_sha256.to_hex(),
        mechanics_lock_sha256: activated
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
        project_composition_lock_sha256: activated
            .composition_lock
            .composition_lock_sha256
            .to_hex(),
        schema_registry_sha256: activated
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        world_partition_sha256: activated
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
    };
    if &actual != roots {
        return package_error("manifest roots do not match the activated packaged project");
    }
    Ok(())
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let value = serde_json::to_value(value)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    let mut bytes = Vec::new();
    write_canonical_value(&value, &mut bytes)?;
    Ok(bytes)
}

fn write_canonical_value(value: &serde_json::Value, output: &mut Vec<u8>) -> Result<(), String> {
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(value) => {
            output.extend_from_slice(if *value { b"true" } else { b"false" });
        }
        serde_json::Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::String(value) => {
            serde_json::to_writer(output, value)
                .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_value(value, output)?;
            }
            output.push(b']');
        }
        serde_json::Value::Object(values) => {
            output.push(b'{');
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key)
                    .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
                output.push(b':');
                write_canonical_value(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn copy_required_notices(repository_root: &Path, staging: &Path) -> Result<(), String> {
    for notice in REQUIRED_NOTICE_PATHS {
        let source = repository_root.join(notice);
        checked_metadata(&source)?;
        fs::copy(&source, staging.join(notice)).map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: failed to package required notice {notice}: {error}"
            )
        })?;
    }
    Ok(())
}

fn copy_binary(source: &Path, destination: &Path) -> Result<(), String> {
    let metadata = checked_metadata(source)?;
    if !metadata.is_file() {
        return package_error(format!(
            "release binary {} is not a regular file",
            source.display()
        ));
    }
    fs::copy(source, destination).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to copy binary {}: {error}",
            source.display()
        )
    })?;
    Ok(())
}

fn run_checked(root: &Path, program: &str, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: failed to run {program}: {error}")
        })?;
    if !output.stdout.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if output.status.success() {
        Ok(())
    } else {
        package_error(format!(
            "{program} {} failed with {}",
            arguments.join(" "),
            output.status
        ))
    }
}

fn cargo_target_directory(repository_root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR").map_or_else(
        || repository_root.join("target"),
        |configured| {
            let configured = PathBuf::from(configured);
            if configured.is_absolute() {
                configured
            } else {
                repository_root.join(configured)
            }
        },
    )
}

fn release_binary_directory(target_directory: &Path, target_triple: &str) -> PathBuf {
    target_directory.join(target_triple).join("release")
}

fn absolute_from(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn native_shipping_target_triple() -> Result<&'static str, String> {
    if cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        Ok("x86_64-pc-windows-msvc")
    } else if cfg!(all(
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    )) {
        Ok("x86_64-unknown-linux-gnu")
    } else {
        Err(
            "TARGET_PACKAGE_REQUIRES_NATIVE_WINDOWS_OR_LINUX_X86_64: package requires native Windows or Linux x86_64"
                .to_owned(),
        )
    }
}

fn required_notice_paths() -> Vec<String> {
    REQUIRED_NOTICE_PATHS
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn hash_bytes(bytes: &[u8]) -> String {
    next_contracts::ids::content_hash_from_bytes(next_contracts::canonical::sha256(bytes)).to_hex()
}

fn validate_hash(name: &str, value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return package_error(format!("{name} is not a canonical lowercase SHA-256"));
    }
    Ok(())
}

fn trim_ascii_whitespace(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &bytes[start..end]
}

fn bounded_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).chars().take(1024).collect()
}

struct PublishLock {
    path: PathBuf,
    file: Option<fs::File>,
}

impl PublishLock {
    fn acquire(output: &Path) -> Result<Self, String> {
        let parent = output.parent().ok_or_else(|| {
            "NATIVE_GATE_PACKAGE_INVALID: package output has no parent".to_owned()
        })?;
        let output_name = output
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                "NATIVE_GATE_PACKAGE_INVALID: package output needs a UTF-8 file name".to_owned()
            })?;
        let path = parent.join(format!(".{output_name}.publish.lock"));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    format!(
                        "NATIVE_GATE_OUTPUT_EXISTS: package publish lock already exists: {}",
                        path.display()
                    )
                } else {
                    format!(
                        "NATIVE_GATE_PACKAGE_INVALID: failed to create package publish lock {}: {error}",
                        path.display()
                    )
                }
            })?;
        Ok(Self {
            path,
            file: Some(file),
        })
    }
}

impl Drop for PublishLock {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

fn ensure_publish_destination_absent(output: &Path) -> Result<(), String> {
    if path_exists_without_following(output)? {
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: v1 package output already exists: {}",
            output.display()
        ));
    }
    Ok(())
}

fn path_exists_without_following(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to inspect {}: {error}",
            path.display()
        )),
    }
}

fn remove_directory_if_present(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to inspect disposable directory {}: {error}",
            path.display()
        )),
        Ok(_) => {
            if !checked_metadata(path)?.is_dir() {
                return package_error(format!(
                    "refusing to recursively remove non-directory {}",
                    path.display()
                ));
            }
            fs::remove_dir_all(path).map_err(|error| {
                format!(
                    "NATIVE_GATE_PACKAGE_INVALID: failed to remove disposable directory {}: {error}",
                    path.display()
                )
            })
        }
    }
}

fn cleanup_failed_build<T>(staging: &Path, error: String) -> Result<T, String> {
    match remove_directory_if_present(staging) {
        Ok(()) => Err(error),
        Err(cleanup_error) => Err(format!(
            "{error}; staging cleanup also failed: {cleanup_error}"
        )),
    }
}

fn package_error<T>(message: impl std::fmt::Display) -> Result<T, String> {
    Err(format!("NATIVE_GATE_PACKAGE_INVALID: {message}"))
}

#[cfg(test)]
mod tests;
