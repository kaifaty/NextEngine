use std::fs::{self, File};
use std::io::{Read, Take};
use std::path::{Component, Path};

use sha2::{Digest, Sha256};

use super::{PACKAGE_MANIFEST_FILE, PackageFileV2, package_error, validate_hash};

pub(super) const MAX_PACKAGE_DEPTH: usize = 32;
pub(super) const MAX_PACKAGE_ENTRIES: u64 = 32_768;
pub(super) const MAX_PACKAGE_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub(super) const MAX_PACKAGE_TOTAL_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub(super) fn validate_package_root(package_root: &Path) -> Result<(), String> {
    let root_metadata = checked_metadata(package_root)?;
    if !root_metadata.is_dir() {
        return package_error(format!(
            "{} is not a package directory",
            package_root.display()
        ));
    }
    let mut seen = Vec::new();
    for entry in read_directory(package_root)? {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "NATIVE_GATE_PACKAGE_INVALID: package path is not UTF-8".to_owned())?;
        let metadata = checked_metadata(&entry.path())?;
        match name.as_str() {
            "bin" | "project" | "source" if metadata.is_dir() => {}
            PACKAGE_MANIFEST_FILE
            | "ACCEPTANCE.md"
            | "LICENSE"
            | "NOTICE"
            | "REFERENCE_ALPHA_NOTICE"
            | "THIRD_PARTY_NOTICES.md"
            | "MIGRATION_PROVENANCE.md"
                if metadata.is_file() => {}
            _ => {
                return package_error(format!("top-level package object {name} is not allowed"));
            }
        }
        seen.push(name);
    }
    for required in [
        "bin",
        "project",
        "source",
        PACKAGE_MANIFEST_FILE,
        "LICENSE",
        "NOTICE",
        "THIRD_PARTY_NOTICES.md",
        "MIGRATION_PROVENANCE.md",
    ] {
        if !seen.iter().any(|name| name == required) {
            return package_error(format!("required package object {required} is missing"));
        }
    }
    Ok(())
}

pub(super) fn validate_project_store_layout(package_root: &Path) -> Result<(), String> {
    let project = package_root.join("project");
    if !checked_metadata(&project)?.is_dir() {
        return package_error("project is not a directory");
    }
    let mut current_seen = false;
    let mut generations_seen = false;
    for entry in read_directory(&project)? {
        let name = utf8_name(&entry)?;
        let metadata = checked_metadata(&entry.path())?;
        match name.as_str() {
            next_assets::CONTENT_CURRENT_FILE if metadata.is_file() => current_seen = true,
            next_assets::CONTENT_GENERATIONS_DIRECTORY if metadata.is_dir() => {
                generations_seen = true;
            }
            _ => {
                return package_error(format!(
                    "project content-store object {name} is not allowed"
                ));
            }
        }
    }
    if !current_seen || !generations_seen {
        return package_error("project content store requires exactly CURRENT and generations");
    }

    let current_path = project.join(next_assets::CONTENT_CURRENT_FILE);
    let current = read_bounded(&current_path, 65)?;
    if current.len() != 65 || current.last() != Some(&b'\n') {
        return package_error("project CURRENT is not a canonical generation pointer");
    }
    let generation = std::str::from_utf8(&current[..64])
        .map_err(|_| "NATIVE_GATE_PACKAGE_INVALID: project CURRENT is not valid UTF-8".to_owned())?
        .to_owned();
    validate_hash("project CURRENT generation", &generation)?;

    let generations = project.join(next_assets::CONTENT_GENERATIONS_DIRECTORY);
    let entries = read_directory(&generations)?;
    if entries.len() != 1 {
        return package_error("project generations must contain exactly the current generation");
    }
    let entry = &entries[0];
    let name = utf8_name(entry)?;
    if name != generation || !checked_metadata(&entry.path())?.is_dir() {
        return package_error("project generations do not exactly match CURRENT");
    }
    Ok(())
}

pub(super) fn collect_inventory(package_root: &Path) -> Result<Vec<PackageFileV2>, String> {
    let mut inventory = Vec::new();
    let mut budget = TraversalBudget::default();
    collect_inventory_from(package_root, package_root, 0, &mut budget, &mut inventory)?;
    inventory.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(inventory)
}

fn collect_inventory_from(
    package_root: &Path,
    directory: &Path,
    depth: usize,
    budget: &mut TraversalBudget,
    inventory: &mut Vec<PackageFileV2>,
) -> Result<(), String> {
    for entry in read_directory(directory)? {
        let path = entry.path();
        let metadata = checked_metadata(&path)?;
        let entry_depth = depth
            .checked_add(1)
            .ok_or_else(|| "NATIVE_GATE_PACKAGE_INVALID: package depth overflow".to_owned())?;
        budget.observe_entry(entry_depth, &metadata)?;
        let relative = path.strip_prefix(package_root).map_err(|_| {
            "NATIVE_GATE_PACKAGE_INVALID: package traversal escaped its root".to_owned()
        })?;
        let relative = portable_relative_path(relative)?;
        validate_relative_package_path(&relative)?;
        if metadata.is_dir() {
            collect_inventory_from(package_root, &path, entry_depth, budget, inventory)?;
        } else if metadata.is_file() {
            if relative != PACKAGE_MANIFEST_FILE {
                inventory.push(PackageFileV2 {
                    path: relative,
                    sha256: hash_file(&path)?,
                    size_bytes: metadata.len(),
                });
            }
        } else {
            return package_error(format!("{relative} is not a regular file or directory"));
        }
    }
    Ok(())
}

#[derive(Default)]
pub(super) struct TraversalBudget {
    entries: u64,
    total_file_bytes: u64,
}

impl TraversalBudget {
    pub(super) fn observe_entry(
        &mut self,
        depth: usize,
        metadata: &fs::Metadata,
    ) -> Result<(), String> {
        if depth > MAX_PACKAGE_DEPTH {
            return package_error(format!(
                "package depth {depth} exceeds limit {MAX_PACKAGE_DEPTH}"
            ));
        }
        self.entries = self.entries.checked_add(1).ok_or_else(|| {
            "NATIVE_GATE_PACKAGE_INVALID: package entry count overflow".to_owned()
        })?;
        if self.entries > MAX_PACKAGE_ENTRIES {
            return package_error(format!(
                "package entry count {} exceeds limit {MAX_PACKAGE_ENTRIES}",
                self.entries
            ));
        }
        if metadata.is_file() {
            let size = metadata.len();
            if size > MAX_PACKAGE_FILE_BYTES {
                return package_error(format!(
                    "package file size {size} exceeds limit {MAX_PACKAGE_FILE_BYTES}"
                ));
            }
            self.total_file_bytes = self.total_file_bytes.checked_add(size).ok_or_else(|| {
                "NATIVE_GATE_PACKAGE_INVALID: package total byte count overflow".to_owned()
            })?;
            if self.total_file_bytes > MAX_PACKAGE_TOTAL_BYTES {
                return package_error(format!(
                    "package byte count {} exceeds limit {MAX_PACKAGE_TOTAL_BYTES}",
                    self.total_file_bytes
                ));
            }
        }
        Ok(())
    }
}

pub(super) fn validate_relative_package_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path.contains("//")
    {
        return package_error(format!("invalid portable package path {path:?}"));
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return package_error(format!("package path escapes its root: {path}"));
    }
    for component in path.split('/') {
        let normalized = component.to_ascii_lowercase();
        let forbidden = forbidden_credential_component(&normalized)
            || normalized
                .split(|character: char| !character.is_ascii_alphanumeric())
                .filter(|token| !token.is_empty())
                .any(forbidden_operational_token);
        if component == "." || component == ".." || forbidden {
            return package_error(format!("forbidden package path component {component}"));
        }
    }
    Ok(())
}

fn forbidden_operational_token(token: &str) -> bool {
    matches!(
        token,
        "state"
            | "states"
            | "log"
            | "logs"
            | "save"
            | "saves"
            | "savegame"
            | "savegames"
            | "session"
            | "sessions"
            | "cache"
            | "caches"
            | "credential"
            | "credentials"
            | "secret"
            | "secrets"
            | "password"
            | "passwords"
            | "token"
            | "tokens"
            | "env"
            | "import"
            | "imports"
            | "imported"
            | "importer"
            | "importers"
    ) || token.starts_with("credential")
        || token.starts_with("save")
        || token.starts_with("session")
        || token.starts_with("cache")
        || token.starts_with("import")
        || token.ends_with("cache")
}

fn forbidden_credential_component(component: &str) -> bool {
    matches!(
        component,
        ".aws"
            | ".azure"
            | ".docker"
            | ".dockerconfigjson"
            | ".git-credentials"
            | ".kube"
            | ".kubeconfig"
            | ".netrc"
            | ".npmrc"
            | ".pypirc"
            | "_netrc"
            | "application_default_credentials.json"
            | "authorized_keys"
            | "id_dsa"
            | "id_ecdsa"
            | "id_ed25519"
            | "id_rsa"
            | "known_hosts"
            | "kubeconfig"
            | "service-account.json"
            | "service_account.json"
    ) || [".pem", ".key", ".p12", ".pfx"]
        .iter()
        .any(|extension| component.ends_with(extension))
        || [
            "api-key",
            "api_key",
            "apikey",
            "auth-token",
            "auth_token",
            "authtoken",
        ]
        .iter()
        .any(|pattern| component.contains(pattern))
}

pub(super) fn checked_metadata(path: &Path) -> Result<fs::Metadata, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to inspect {}: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return package_error(format!(
            "symlink or reparse point is forbidden: {}",
            path.display()
        ));
    }
    Ok(metadata)
}

pub(super) fn hash_file(path: &Path) -> Result<String, String> {
    let metadata = checked_metadata(path)?;
    if !metadata.is_file() {
        return package_error(format!("{} is not a regular file", path.display()));
    }
    let mut file = File::open(path).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to open {}: {error}",
            path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: failed to hash {}: {error}",
                path.display()
            )
        })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    Ok(next_contracts::ids::content_hash_from_bytes(digest).to_hex())
}

fn read_directory(path: &Path) -> Result<Vec<fs::DirEntry>, String> {
    fs::read_dir(path)
        .map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: failed to inspect {}: {error}",
                path.display()
            )
        })?
        .map(|entry| {
            entry.map_err(|error| {
                format!("NATIVE_GATE_PACKAGE_INVALID: failed to inspect package: {error}")
            })
        })
        .collect()
}

pub(super) fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = checked_metadata(path)?;
    if !metadata.is_file() || metadata.len() > limit {
        return package_error(format!("{} exceeds its {limit}-byte bound", path.display()));
    }
    let file = File::open(path).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to open {}: {error}",
            path.display()
        )
    })?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    let mut bounded: Take<File> = file.take(limit + 1);
    bounded.read_to_end(&mut bytes).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_INVALID: failed to read {}: {error}",
            path.display()
        )
    })?;
    if bytes.len() as u64 > limit {
        return package_error(format!("{} exceeds its {limit}-byte bound", path.display()));
    }
    Ok(bytes)
}

fn utf8_name(entry: &fs::DirEntry) -> Result<String, String> {
    entry
        .file_name()
        .into_string()
        .map_err(|_| "NATIVE_GATE_PACKAGE_INVALID: package path is not UTF-8".to_owned())
}

fn portable_relative_path(path: &Path) -> Result<String, String> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return package_error(format!("invalid package path {}", path.display()));
        };
        let component = component.to_str().ok_or_else(|| {
            "NATIVE_GATE_PACKAGE_INVALID: package path is not valid UTF-8".to_owned()
        })?;
        components.push(component);
    }
    Ok(components.join("/"))
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}
