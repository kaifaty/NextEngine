use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use super::inventory::{MAX_PACKAGE_FILE_BYTES, checked_metadata, hash_file, read_bounded};
use super::manifest::{canonical_json_bytes, hash_bytes};
use super::{PackageFileV2, package_error, validate_hash};

pub(super) const CARGO_LOCK_PATH: &str = "Cargo.lock";
pub(super) const DEPENDENCY_INVENTORY_PATH: &str = "DEPENDENCY_INVENTORY.jcs";
pub(super) const GETTING_STARTED_PATH: &str = "GETTING_STARTED.md";
pub(super) const THIRD_PARTY_LICENSE_DIRECTORY: &str = "THIRD_PARTY_LICENSES";
pub(super) const TROUBLESHOOTING_PATH: &str = "TROUBLESHOOTING.md";

const DEPENDENCY_INVENTORY_SCHEMA_VERSION: u32 = 1;
const DEPENDENCY_INVENTORY_SCOPE: &str = "non-dev Cargo release closure for next_game/desktop-sdl-ash, next_headless and next_cli; build dependencies are conservatively included";
const RELEASE_NAME: &str = "nextengine";
const PROTECTED_DATA_SCANNER_ID: &str = "nextengine-protected-data-v1";
const SELECTED_ROOTS: [&str; 3] = ["next_cli", "next_game", "next_headless"];
const CRATES_IO_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDistributionV1 {
    pub cargo_lock_path: String,
    pub cargo_lock_sha256: String,
    pub dependency_count: u32,
    pub dependency_inventory_path: String,
    pub dependency_inventory_sha256: String,
    pub getting_started_path: String,
    pub license_file_count: u32,
    pub protected_data_scan: PackageProtectedDataScanV1,
    pub release_name: String,
    pub release_version: String,
    pub troubleshooting_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageProtectedDataScanV1 {
    pub scanned_byte_count: u64,
    pub scanned_file_count: u32,
    pub scanner_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyInventoryV1 {
    pub cargo_lock_sha256: String,
    pub dependencies: Vec<DependencyRecordV1>,
    pub roots: Vec<DependencyRootV1>,
    pub schema_version: u32,
    pub scope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyRootV1 {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyRecordV1 {
    pub checksum: String,
    pub license: String,
    pub license_files: Vec<DependencyLicenseFileV1>,
    pub name: String,
    pub source: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyLicenseFileV1 {
    pub path: String,
    pub sha256: String,
}

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    resolve: Option<MetadataResolve>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    id: String,
    license: Option<String>,
    license_file: Option<String>,
    manifest_path: PathBuf,
    name: String,
    source: Option<String>,
    version: String,
}

#[derive(Deserialize)]
struct MetadataResolve {
    nodes: Vec<MetadataNode>,
}

#[derive(Deserialize)]
struct MetadataNode {
    dependencies: Vec<String>,
    deps: Vec<MetadataNodeDependency>,
    id: String,
}

#[derive(Deserialize)]
struct MetadataNodeDependency {
    dep_kinds: Vec<MetadataDependencyKind>,
    pkg: String,
}

#[derive(Deserialize)]
struct MetadataDependencyKind {
    kind: Option<String>,
}

#[derive(Deserialize)]
struct CargoLock {
    package: Vec<CargoLockPackage>,
}

#[derive(Deserialize)]
struct CargoLockPackage {
    checksum: Option<String>,
    name: String,
    source: Option<String>,
    version: String,
}

pub(super) fn build_distribution_materials(
    repository_root: &Path,
    package_root: &Path,
) -> Result<PackageDistributionV1, String> {
    for path in [GETTING_STARTED_PATH, TROUBLESHOOTING_PATH, CARGO_LOCK_PATH] {
        copy_regular_file(repository_root.join(path), package_root.join(path), path)?;
    }

    let cargo_lock_sha256 = hash_file(&package_root.join(CARGO_LOCK_PATH))?;
    let inventory = build_dependency_inventory(repository_root, package_root, &cargo_lock_sha256)?;
    let dependency_count = u32::try_from(inventory.dependencies.len())
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: dependency count: {error}"))?;
    let license_file_count =
        inventory
            .dependencies
            .iter()
            .try_fold(0_u32, |count, dependency| {
                let files = u32::try_from(dependency.license_files.len()).map_err(|error| {
                    format!("NATIVE_GATE_PACKAGE_INVALID: license file count: {error}")
                })?;
                count.checked_add(files).ok_or_else(|| {
                    "NATIVE_GATE_PACKAGE_INVALID: license file count overflow".to_owned()
                })
            })?;
    let inventory_bytes = canonical_json_bytes(&inventory)?;
    fs::write(
        package_root.join(DEPENDENCY_INVENTORY_PATH),
        &inventory_bytes,
    )
    .map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: failed to write dependency inventory: {error}")
    })?;

    Ok(PackageDistributionV1 {
        cargo_lock_path: CARGO_LOCK_PATH.to_owned(),
        cargo_lock_sha256,
        dependency_count,
        dependency_inventory_path: DEPENDENCY_INVENTORY_PATH.to_owned(),
        dependency_inventory_sha256: hash_bytes(&inventory_bytes),
        getting_started_path: GETTING_STARTED_PATH.to_owned(),
        license_file_count,
        protected_data_scan: PackageProtectedDataScanV1 {
            scanned_byte_count: 0,
            scanned_file_count: 0,
            scanner_id: PROTECTED_DATA_SCANNER_ID.to_owned(),
            status: "PENDING".to_owned(),
        },
        release_name: RELEASE_NAME.to_owned(),
        release_version: env!("CARGO_PKG_VERSION").to_owned(),
        troubleshooting_path: TROUBLESHOOTING_PATH.to_owned(),
    })
}

fn build_dependency_inventory(
    repository_root: &Path,
    package_root: &Path,
    cargo_lock_sha256: &str,
) -> Result<DependencyInventoryV1, String> {
    let metadata = cargo_metadata(repository_root)?;
    let resolved = metadata.resolve.as_ref().ok_or_else(|| {
        "NATIVE_GATE_PACKAGE_INVALID: Cargo metadata omitted the resolve graph".to_owned()
    })?;
    let packages = metadata
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let nodes = resolved
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    let mut roots = Vec::new();
    let mut queue = VecDeque::new();
    for root_name in SELECTED_ROOTS {
        let root = metadata
            .packages
            .iter()
            .find(|package| package.name == root_name && package.source.is_none())
            .ok_or_else(|| {
                format!("NATIVE_GATE_PACKAGE_INVALID: selected Cargo root {root_name} is missing")
            })?;
        roots.push(DependencyRootV1 {
            name: root.name.clone(),
            version: root.version.clone(),
        });
        queue.push_back(root.id.as_str());
    }
    roots.sort();

    let mut selected = BTreeSet::new();
    while let Some(id) = queue.pop_front() {
        if !selected.insert(id) {
            continue;
        }
        let node = nodes.get(id).ok_or_else(|| {
            format!("NATIVE_GATE_PACKAGE_INVALID: Cargo resolve node {id} is missing")
        })?;
        let typed_edges = node
            .deps
            .iter()
            .filter(|dependency| {
                dependency.dep_kinds.is_empty()
                    || dependency
                        .dep_kinds
                        .iter()
                        .any(|kind| kind.kind.as_deref() != Some("dev"))
            })
            .map(|dependency| dependency.pkg.as_str())
            .collect::<BTreeSet<_>>();
        let edges: BTreeSet<_> = if node.deps.is_empty() {
            node.dependencies.iter().map(String::as_str).collect()
        } else {
            typed_edges
        };
        queue.extend(edges);
    }

    let lock_bytes =
        fs::read_to_string(repository_root.join(CARGO_LOCK_PATH)).map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: failed to read Cargo.lock: {error}")
        })?;
    let cargo_lock: CargoLock = toml::from_str(&lock_bytes).map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: failed to decode Cargo.lock: {error}")
    })?;
    let mut dependencies = Vec::new();
    for id in selected {
        let package = packages
            .get(id)
            .ok_or_else(|| format!("NATIVE_GATE_PACKAGE_INVALID: Cargo package {id} is missing"))?;
        let Some(source) = package.source.as_deref() else {
            continue;
        };
        if source != CRATES_IO_SOURCE {
            return package_error(format!(
                "selected dependency {} {} uses unsupported source {source}",
                package.name, package.version
            ));
        }
        let license = package
            .license
            .as_deref()
            .filter(|license| !license.trim().is_empty())
            .ok_or_else(|| {
                format!(
                    "NATIVE_GATE_PACKAGE_INVALID: dependency {} {} has no declared license",
                    package.name, package.version
                )
            })?;
        let locked = cargo_lock
            .package
            .iter()
            .find(|locked| {
                locked.name == package.name
                    && locked.version == package.version
                    && locked.source.as_deref() == Some(source)
            })
            .ok_or_else(|| {
                format!(
                    "NATIVE_GATE_PACKAGE_INVALID: dependency {} {} is absent from Cargo.lock",
                    package.name, package.version
                )
            })?;
        let checksum = locked.checksum.as_deref().ok_or_else(|| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: dependency {} {} has no locked checksum",
                package.name, package.version
            )
        })?;
        validate_hash("dependency checksum", checksum)?;
        let license_files = copy_dependency_license_files(package, package_root)?;
        dependencies.push(DependencyRecordV1 {
            checksum: checksum.to_owned(),
            license: license.to_owned(),
            license_files,
            name: package.name.clone(),
            source: source.to_owned(),
            version: package.version.clone(),
        });
    }
    dependencies.sort_by(|left, right| {
        (&left.name, &left.version, &left.source).cmp(&(&right.name, &right.version, &right.source))
    });

    Ok(DependencyInventoryV1 {
        cargo_lock_sha256: cargo_lock_sha256.to_owned(),
        dependencies,
        roots,
        schema_version: DEPENDENCY_INVENTORY_SCHEMA_VERSION,
        scope: DEPENDENCY_INVENTORY_SCOPE.to_owned(),
    })
}

fn cargo_metadata(repository_root: &Path) -> Result<CargoMetadata, String> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--offline",
            "--format-version",
            "1",
            "--filter-platform",
            "x86_64-unknown-linux-gnu",
            "--features",
            "next_game/desktop-sdl-ash",
        ])
        .current_dir(repository_root)
        .output()
        .map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: failed to run cargo metadata: {error}")
        })?;
    if !output.status.success() {
        return package_error(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: invalid cargo metadata: {error}"))
}

fn copy_dependency_license_files(
    package: &MetadataPackage,
    package_root: &Path,
) -> Result<Vec<DependencyLicenseFileV1>, String> {
    let source_root = package.manifest_path.parent().ok_or_else(|| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: dependency {} manifest has no parent",
            package.name
        )
    })?;
    let mut sources = BTreeSet::new();
    for entry in fs::read_dir(source_root).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to inspect dependency {} licenses: {error}",
            package.name
        )
    })? {
        let entry = entry.map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: failed to inspect license entry: {error}")
        })?;
        let name = entry.file_name().into_string().map_err(|_| {
            "NATIVE_GATE_PACKAGE_INVALID: dependency license file name is not UTF-8".to_owned()
        })?;
        if is_license_file_name(&name) {
            sources.insert(entry.path());
        }
    }
    if sources.is_empty() && package.name.ends_with("-src") {
        collect_nested_license_files(source_root, source_root, 0, &mut sources)?;
    }
    if let Some(license_file) = package.license_file.as_deref() {
        sources.insert(source_root.join(license_file));
    }
    if sources.is_empty() {
        return package_error(format!(
            "dependency {} {} has no distributable license file",
            package.name, package.version
        ));
    }

    let component = dependency_component(&package.name, &package.version)?;
    let destination_root = package_root
        .join(THIRD_PARTY_LICENSE_DIRECTORY)
        .join(&component);
    fs::create_dir_all(&destination_root).map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: failed to create license directory: {error}")
    })?;
    let mut files = Vec::new();
    for source in sources {
        let metadata = checked_metadata(&source)?;
        if !metadata.is_file() || metadata.len() > MAX_PACKAGE_FILE_BYTES {
            return package_error(format!(
                "dependency license {} is not a bounded regular file",
                source.display()
            ));
        }
        let relative = source.strip_prefix(source_root).map_err(|_| {
            "NATIVE_GATE_PACKAGE_INVALID: dependency license escaped its source root".to_owned()
        })?;
        let relative = relative.to_str().ok_or_else(|| {
            "NATIVE_GATE_PACKAGE_INVALID: dependency license path is not UTF-8".to_owned()
        })?;
        if relative
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
        {
            return package_error("dependency license path is not portable");
        }
        let destination = destination_root.join(relative);
        let destination_parent = destination.parent().ok_or_else(|| {
            "NATIVE_GATE_PACKAGE_INVALID: dependency license destination has no parent".to_owned()
        })?;
        fs::create_dir_all(destination_parent).map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: failed to create license directory: {error}")
        })?;
        copy_regular_file(&source, &destination, "dependency license")?;
        let path = format!("{THIRD_PARTY_LICENSE_DIRECTORY}/{component}/{relative}");
        files.push(DependencyLicenseFileV1 {
            path,
            sha256: hash_file(&destination)?,
        });
    }
    files.sort();
    if files.windows(2).any(|pair| pair[0].path == pair[1].path) {
        return package_error(format!(
            "dependency {} {} has duplicate license file names",
            package.name, package.version
        ));
    }
    Ok(files)
}

fn collect_nested_license_files(
    source_root: &Path,
    directory: &Path,
    depth: usize,
    output: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    const MAX_LICENSE_SEARCH_DEPTH: usize = 4;
    if depth >= MAX_LICENSE_SEARCH_DEPTH {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to inspect nested dependency licenses: {error}"
        )
    })? {
        let entry = entry.map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: failed to inspect license entry: {error}")
        })?;
        let path = entry.path();
        let metadata = checked_metadata(&path)?;
        if metadata.is_dir() {
            collect_nested_license_files(source_root, &path, depth + 1, output)?;
        } else if metadata.is_file() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    "NATIVE_GATE_PACKAGE_INVALID: dependency license file name is not UTF-8"
                        .to_owned()
                })?;
            if is_license_file_name(name) && path.starts_with(source_root) {
                output.insert(path);
            }
        }
    }
    Ok(())
}

fn dependency_component(name: &str, version: &str) -> Result<String, String> {
    let component = format!("{name}-{version}");
    if component.is_empty()
        || !component
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+'))
    {
        return package_error("dependency name/version is not a portable license path component");
    }
    Ok(component)
}

fn is_license_file_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    ["license", "copying", "copyright", "notice"]
        .iter()
        .any(|prefix| {
            name == *prefix
                || name.starts_with(&format!("{prefix}."))
                || name.starts_with(&format!("{prefix}-"))
        })
}

fn copy_regular_file(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    label: &str,
) -> Result<(), String> {
    let source = source.as_ref();
    if !checked_metadata(source)?.is_file() {
        return package_error(format!(
            "{label} {} is not a regular file",
            source.display()
        ));
    }
    fs::copy(source, destination.as_ref())
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: failed to copy {label}: {error}"))?;
    Ok(())
}

pub(super) fn scan_protected_data(
    package_root: &Path,
    inventory: &[PackageFileV2],
) -> Result<PackageProtectedDataScanV1, String> {
    let mut scanned_byte_count = 0_u64;
    for file in inventory {
        let bytes = read_bounded(&package_root.join(&file.path), MAX_PACKAGE_FILE_BYTES)?;
        scanned_byte_count = scanned_byte_count
            .checked_add(file.size_bytes)
            .ok_or_else(|| {
                "NATIVE_GATE_PACKAGE_INVALID: protected-data scan byte count overflow".to_owned()
            })?;
        if let Some(marker) = protected_marker(&bytes) {
            return package_error(format!(
                "PROTECTED_DATA_DETECTED: {} contains {marker}",
                file.path
            ));
        }
    }
    Ok(PackageProtectedDataScanV1 {
        scanned_byte_count,
        scanned_file_count: u32::try_from(inventory.len()).map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: protected-data file count: {error}")
        })?,
        scanner_id: PROTECTED_DATA_SCANNER_ID.to_owned(),
        status: "PASS".to_owned(),
    })
}

fn protected_marker(bytes: &[u8]) -> Option<&'static str> {
    const FIXED_MARKERS: [(&str, &[u8]); 12] = [
        ("PEM private key", b"-----BEGIN PRIVATE KEY-----"),
        ("RSA private key", b"-----BEGIN RSA PRIVATE KEY-----"),
        ("EC private key", b"-----BEGIN EC PRIVATE KEY-----"),
        ("DSA private key", b"-----BEGIN DSA PRIVATE KEY-----"),
        (
            "OpenSSH private key",
            b"-----BEGIN OPENSSH PRIVATE KEY-----",
        ),
        ("Linux user path", b"/home/"),
        ("root user path", b"/root/"),
        ("macOS user path", b"/Users/"),
        ("Windows user path", b":\\Users\\"),
        ("AWS access-key assignment", b"aws_access_key_id="),
        ("AWS secret-key assignment", b"aws_secret_access_key="),
        ("Slack token", b"xoxb-"),
    ];
    FIXED_MARKERS
        .iter()
        .find_map(|(name, marker)| contains_bytes(bytes, marker).then_some(*name))
        .or_else(|| credential_prefix(bytes).map(|_| "credential token"))
}

fn credential_prefix(bytes: &[u8]) -> Option<()> {
    [
        (b"ghp_".as_slice(), 20_usize),
        (b"AKIA".as_slice(), 16_usize),
    ]
    .into_iter()
    .find_map(|(prefix, suffix)| {
        bytes.windows(prefix.len() + suffix).find_map(|window| {
            (window.starts_with(prefix)
                && window[prefix.len()..].iter().all(u8::is_ascii_alphanumeric))
            .then_some(())
        })
    })
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

pub(super) fn validate_distribution_materials(
    package_root: &Path,
    distribution: &PackageDistributionV1,
    inventory: &[PackageFileV2],
) -> Result<(), String> {
    if distribution.release_name != RELEASE_NAME
        || distribution.release_version != env!("CARGO_PKG_VERSION")
        || distribution.cargo_lock_path != CARGO_LOCK_PATH
        || distribution.dependency_inventory_path != DEPENDENCY_INVENTORY_PATH
        || distribution.getting_started_path != GETTING_STARTED_PATH
        || distribution.troubleshooting_path != TROUBLESHOOTING_PATH
    {
        return package_error("distribution paths, name or release version are not canonical");
    }
    validate_hash("Cargo.lock", &distribution.cargo_lock_sha256)?;
    validate_hash(
        "dependency inventory",
        &distribution.dependency_inventory_sha256,
    )?;
    require_inventory_hash(inventory, CARGO_LOCK_PATH, &distribution.cargo_lock_sha256)?;
    require_inventory_path(inventory, GETTING_STARTED_PATH)?;
    require_inventory_path(inventory, TROUBLESHOOTING_PATH)?;

    let bytes = read_bounded(
        &package_root.join(DEPENDENCY_INVENTORY_PATH),
        MAX_PACKAGE_FILE_BYTES,
    )?;
    if hash_bytes(&bytes) != distribution.dependency_inventory_sha256 {
        return package_error("dependency inventory hash does not match distribution receipt");
    }
    let decoded: DependencyInventoryV1 = serde_json::from_slice(&bytes).map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: invalid dependency inventory: {error}")
    })?;
    if canonical_json_bytes(&decoded)? != bytes {
        return package_error("dependency inventory is not canonical JSON");
    }
    if decoded.schema_version != DEPENDENCY_INVENTORY_SCHEMA_VERSION
        || decoded.scope != DEPENDENCY_INVENTORY_SCOPE
        || decoded.cargo_lock_sha256 != distribution.cargo_lock_sha256
        || decoded.roots
            != SELECTED_ROOTS
                .map(|name| DependencyRootV1 {
                    name: name.to_owned(),
                    version: distribution.release_version.clone(),
                })
                .to_vec()
    {
        return package_error("dependency inventory header or selected roots are invalid");
    }
    if u32::try_from(decoded.dependencies.len()).ok() != Some(distribution.dependency_count) {
        return package_error("dependency count does not match distribution receipt");
    }
    if decoded.dependencies.windows(2).any(|pair| {
        (&pair[0].name, &pair[0].version, &pair[0].source)
            >= (&pair[1].name, &pair[1].version, &pair[1].source)
    }) {
        return package_error("dependency records must be unique and strictly sorted");
    }
    let mut declared_license_paths = BTreeSet::new();
    let mut license_file_count = 0_u32;
    for dependency in &decoded.dependencies {
        if dependency.name.is_empty()
            || dependency.version.is_empty()
            || dependency.license.trim().is_empty()
            || dependency.source != CRATES_IO_SOURCE
        {
            return package_error("dependency identity, source or license is invalid");
        }
        validate_hash("dependency checksum", &dependency.checksum)?;
        if dependency.license_files.is_empty()
            || dependency
                .license_files
                .windows(2)
                .any(|pair| pair[0].path >= pair[1].path)
        {
            return package_error("dependency license files are missing or not strictly sorted");
        }
        let expected_prefix = format!(
            "{THIRD_PARTY_LICENSE_DIRECTORY}/{}/",
            dependency_component(&dependency.name, &dependency.version)?
        );
        for license in &dependency.license_files {
            if !license.path.starts_with(&expected_prefix)
                || !declared_license_paths.insert(license.path.clone())
            {
                return package_error("dependency license path is invalid or duplicated");
            }
            validate_hash("dependency license file", &license.sha256)?;
            require_inventory_hash(inventory, &license.path, &license.sha256)?;
            license_file_count = license_file_count.checked_add(1).ok_or_else(|| {
                "NATIVE_GATE_PACKAGE_INVALID: dependency license count overflow".to_owned()
            })?;
        }
    }
    let actual_license_paths = inventory
        .iter()
        .filter(|file| {
            file.path
                .starts_with(&format!("{THIRD_PARTY_LICENSE_DIRECTORY}/"))
        })
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    if declared_license_paths != actual_license_paths
        || license_file_count != distribution.license_file_count
    {
        return package_error("third-party license inventory is incomplete");
    }
    Ok(())
}

pub(super) fn validate_protected_data_scan(
    package_root: &Path,
    expected: &PackageProtectedDataScanV1,
    inventory: &[PackageFileV2],
) -> Result<(), String> {
    if expected.scanner_id != PROTECTED_DATA_SCANNER_ID || expected.status != "PASS" {
        return package_error("protected-data scan receipt is not a canonical PASS");
    }
    let actual = scan_protected_data(package_root, inventory)?;
    if &actual != expected {
        return package_error("protected-data scan receipt does not match package bytes");
    }
    Ok(())
}

fn require_inventory_path(inventory: &[PackageFileV2], path: &str) -> Result<(), String> {
    inventory
        .iter()
        .any(|file| file.path == path && file.size_bytes != 0)
        .then_some(())
        .ok_or_else(|| format!("NATIVE_GATE_PACKAGE_INVALID: required file {path} is missing"))
}

fn require_inventory_hash(
    inventory: &[PackageFileV2],
    path: &str,
    expected_hash: &str,
) -> Result<(), String> {
    inventory
        .iter()
        .any(|file| file.path == path && file.sha256 == expected_hash)
        .then_some(())
        .ok_or_else(|| format!("NATIVE_GATE_PACKAGE_INVALID: required file {path} hash is missing"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_data_markers_are_bounded_and_fail_closed() {
        assert_eq!(protected_marker(b"ordinary neutral bytes"), None);
        assert_eq!(
            protected_marker(b"prefix /home/alice/project suffix"),
            Some("Linux user path")
        );
        assert_eq!(
            protected_marker(b"-----BEGIN OPENSSH PRIVATE KEY-----"),
            Some("OpenSSH private key")
        );
        assert_eq!(
            protected_marker(b"ghp_abcdefghijklmnopqrst"),
            Some("credential token")
        );
    }

    #[test]
    fn dependency_components_are_portable() {
        assert_eq!(
            dependency_component("sdl3-sys", "0.6.7+SDL-3.4.12"),
            Ok("sdl3-sys-0.6.7+SDL-3.4.12".to_owned())
        );
        assert!(dependency_component("bad/name", "1.0.0").is_err());
    }
}
