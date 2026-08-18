use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::canonical::sha256;
use next_contracts::ids::content_hash_from_bytes;
use next_project::{CookedProjectV7, activate_project};
use serde::{Deserialize, Serialize};

use crate::runtime::{self, RuntimeExecutionError};
use crate::{
    CreatorPackageDetailsV1, CreatorProjectIdentityV1, CreatorRuntimeProofV1,
    project_identity_from_activated, project_identity_from_cooked,
};

pub(super) const CREATOR_PACKAGE_FORMAT_V1: &str = "nextengine.creator-project-package.v1";
pub(super) const CREATOR_PACKAGE_MANIFEST_FILE: &str = "creator-package.manifest.jcs";

const CREATOR_PACKAGE_SCHEMA_VERSION: u32 = 1;
const MAXIMUM_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAXIMUM_NOTICE_BYTES: u64 = 1024 * 1024;
const MAXIMUM_PACKAGE_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAXIMUM_PACKAGE_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;
const MAXIMUM_PACKAGE_FILES: usize = 4_096;
const REQUIRED_NOTICE_PATH: &str = "NOTICE";

static STAGING_ORDINAL: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(super) enum CreatorPackageError {
    Activation,
    Invalid,
    NoticeInvalid,
    OutputInvalid,
    Runtime(RuntimeExecutionError),
    Storage,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatorProjectPackageManifestV1 {
    files: Vec<CreatorPackageFileV1>,
    format: String,
    project: CreatorProjectIdentityV1,
    required_notices: Vec<String>,
    run: CreatorRuntimeProofV1,
    schema_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatorPackageFileV1 {
    path: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Deserialize)]
struct CreatorPackageVersionProbe {
    format: String,
    schema_version: u32,
}

pub(super) struct ValidatedCreatorPackageV1 {
    pub(super) project: CreatorProjectIdentityV1,
    pub(super) runtime: CreatorRuntimeProofV1,
}

pub(super) fn build_project_package(
    project_root: &Path,
    cooked: &CookedProjectV7,
    requested_output: &Path,
) -> Result<CreatorPackageDetailsV1, CreatorPackageError> {
    let staging = StagingDirectory::new(requested_output)?;
    let result = (|| {
        let project_store_root = staging.path().join("project");
        ContentStore::new(&project_store_root)
            .publish(
                &cooked
                    .publication()
                    .map_err(|_| CreatorPackageError::Invalid)?,
            )
            .map_err(|_| CreatorPackageError::Storage)?;
        copy_notice(project_root, staging.path())?;
        let run = runtime::run_store(&project_store_root, cooked.project_lock.project_lock_sha256)
            .map_err(CreatorPackageError::Runtime)?;
        let files = collect_inventory(staging.path())?;
        let manifest = CreatorProjectPackageManifestV1 {
            files,
            format: CREATOR_PACKAGE_FORMAT_V1.to_owned(),
            project: project_identity_from_cooked(cooked),
            required_notices: vec![REQUIRED_NOTICE_PATH.to_owned()],
            run,
            schema_version: CREATOR_PACKAGE_SCHEMA_VERSION,
        };
        let manifest_bytes = canonical_json_bytes(&manifest)?;
        fs::write(
            staging.path().join(CREATOR_PACKAGE_MANIFEST_FILE),
            &manifest_bytes,
        )
        .map_err(|_| CreatorPackageError::Storage)?;
        let validated = validate_and_run(staging.path())?;
        if validated.project != manifest.project || validated.runtime != manifest.run {
            return Err(CreatorPackageError::Invalid);
        }
        let packaged_size_bytes = manifest.files.iter().try_fold(
            u64::try_from(manifest_bytes.len()).map_err(|_| CreatorPackageError::Invalid)?,
            |total, file| {
                total
                    .checked_add(file.size_bytes)
                    .ok_or(CreatorPackageError::Invalid)
            },
        )?;
        let details = CreatorPackageDetailsV1 {
            project: manifest.project.clone(),
            package_format: manifest.format.clone(),
            package_manifest_sha256: hash_bytes(&manifest_bytes),
            packaged_file_count: u32::try_from(manifest.files.len())
                .map_err(|_| CreatorPackageError::Invalid)?,
            packaged_size_bytes,
            required_notices: manifest.required_notices.clone(),
            runtime: manifest.run.clone(),
        };
        Ok(details)
    })();
    match result {
        Ok(details) => {
            staging.publish()?;
            Ok(details)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn validate_and_run(
    package_root: &Path,
) -> Result<ValidatedCreatorPackageV1, CreatorPackageError> {
    let metadata = fs::symlink_metadata(package_root).map_err(|_| CreatorPackageError::Invalid)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(CreatorPackageError::Invalid);
    }
    let manifest_path = package_root.join(CREATOR_PACKAGE_MANIFEST_FILE);
    let manifest_metadata =
        fs::symlink_metadata(&manifest_path).map_err(|_| CreatorPackageError::Invalid)?;
    if !manifest_metadata.is_file()
        || manifest_metadata.file_type().is_symlink()
        || manifest_metadata.len() > MAXIMUM_MANIFEST_BYTES
    {
        return Err(CreatorPackageError::Invalid);
    }
    let manifest_bytes = fs::read(&manifest_path).map_err(|_| CreatorPackageError::Storage)?;
    let probe: CreatorPackageVersionProbe =
        serde_json::from_slice(&manifest_bytes).map_err(|_| CreatorPackageError::Invalid)?;
    if probe.schema_version != CREATOR_PACKAGE_SCHEMA_VERSION
        || probe.format != CREATOR_PACKAGE_FORMAT_V1
    {
        return Err(CreatorPackageError::Unsupported);
    }
    let manifest: CreatorProjectPackageManifestV1 =
        serde_json::from_slice(&manifest_bytes).map_err(|_| CreatorPackageError::Invalid)?;
    if manifest.required_notices != [REQUIRED_NOTICE_PATH]
        || canonical_json_bytes(&manifest)? != manifest_bytes
        || manifest.files.is_empty()
        || manifest
            .files
            .windows(2)
            .any(|pair| pair[0].path >= pair[1].path)
        || !manifest
            .files
            .iter()
            .any(|file| file.path == REQUIRED_NOTICE_PATH)
        || manifest.files.iter().any(|file| {
            validate_relative_path(&file.path).is_err()
                || (file.path != REQUIRED_NOTICE_PATH && !file.path.starts_with("project/"))
                || !valid_hash(&file.sha256)
        })
    {
        return Err(CreatorPackageError::Invalid);
    }
    let actual_files = collect_inventory(package_root)?;
    if actual_files != manifest.files {
        return Err(CreatorPackageError::Invalid);
    }
    let notice = manifest
        .files
        .iter()
        .find(|file| file.path == REQUIRED_NOTICE_PATH)
        .ok_or(CreatorPackageError::NoticeInvalid)?;
    if notice.size_bytes == 0 || notice.size_bytes > MAXIMUM_NOTICE_BYTES {
        return Err(CreatorPackageError::NoticeInvalid);
    }
    let activated = activate_project(&ContentStore::new(package_root.join("project")))
        .map_err(|_| CreatorPackageError::Activation)?;
    let actual_identity = project_identity_from_activated(&activated);
    if actual_identity != manifest.project {
        return Err(CreatorPackageError::Invalid);
    }
    let actual_run = runtime::run_store(
        &package_root.join("project"),
        activated.project_lock.project_lock_sha256,
    )
    .map_err(CreatorPackageError::Runtime)?;
    if actual_run != manifest.run {
        return Err(CreatorPackageError::Invalid);
    }
    Ok(ValidatedCreatorPackageV1 {
        project: actual_identity,
        runtime: actual_run,
    })
}

fn copy_notice(project_root: &Path, package_root: &Path) -> Result<(), CreatorPackageError> {
    let resolved_root =
        fs::canonicalize(project_root).map_err(|_| CreatorPackageError::NoticeInvalid)?;
    let notice_path = project_root.join(REQUIRED_NOTICE_PATH);
    let metadata =
        fs::symlink_metadata(&notice_path).map_err(|_| CreatorPackageError::NoticeInvalid)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > MAXIMUM_NOTICE_BYTES
    {
        return Err(CreatorPackageError::NoticeInvalid);
    }
    let resolved_notice =
        fs::canonicalize(&notice_path).map_err(|_| CreatorPackageError::NoticeInvalid)?;
    if !resolved_notice.starts_with(&resolved_root) {
        return Err(CreatorPackageError::NoticeInvalid);
    }
    fs::copy(resolved_notice, package_root.join(REQUIRED_NOTICE_PATH))
        .map_err(|_| CreatorPackageError::Storage)?;
    Ok(())
}

fn collect_inventory(root: &Path) -> Result<Vec<CreatorPackageFileV1>, CreatorPackageError> {
    let mut directories = vec![root.to_path_buf()];
    let mut files = Vec::new();
    let mut total_bytes = 0_u64;
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).map_err(|_| CreatorPackageError::Storage)? {
            let entry = entry.map_err(|_| CreatorPackageError::Storage)?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|_| CreatorPackageError::Storage)?;
            if metadata.file_type().is_symlink() {
                return Err(CreatorPackageError::Invalid);
            }
            if metadata.is_dir() {
                directories.push(path);
                continue;
            }
            if !metadata.is_file() {
                return Err(CreatorPackageError::Invalid);
            }
            let relative = normalized_relative_path(root, &path)?;
            if relative == CREATOR_PACKAGE_MANIFEST_FILE {
                continue;
            }
            if metadata.len() > MAXIMUM_PACKAGE_FILE_BYTES {
                return Err(CreatorPackageError::Invalid);
            }
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or(CreatorPackageError::Invalid)?;
            if total_bytes > MAXIMUM_PACKAGE_TOTAL_BYTES || files.len() >= MAXIMUM_PACKAGE_FILES {
                return Err(CreatorPackageError::Invalid);
            }
            let bytes = fs::read(&path).map_err(|_| CreatorPackageError::Storage)?;
            if u64::try_from(bytes.len()).map_err(|_| CreatorPackageError::Invalid)?
                != metadata.len()
            {
                return Err(CreatorPackageError::Invalid);
            }
            files.push(CreatorPackageFileV1 {
                path: relative,
                sha256: hash_bytes(&bytes),
                size_bytes: metadata.len(),
            });
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    if files.windows(2).any(|pair| pair[0].path == pair[1].path) {
        return Err(CreatorPackageError::Invalid);
    }
    Ok(files)
}

fn normalized_relative_path(root: &Path, path: &Path) -> Result<String, CreatorPackageError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| CreatorPackageError::Invalid)?;
    let mut parts = Vec::new();
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(CreatorPackageError::Invalid);
        };
        parts.push(
            value
                .to_str()
                .ok_or(CreatorPackageError::Invalid)?
                .to_owned(),
        );
    }
    if parts.is_empty() {
        return Err(CreatorPackageError::Invalid);
    }
    let value = parts.join("/");
    validate_relative_path(&value)?;
    Ok(value)
}

fn validate_relative_path(value: &str) -> Result<(), CreatorPackageError> {
    if value.is_empty() || value.len() > 1_024 || value.contains('\\') {
        return Err(CreatorPackageError::Invalid);
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CreatorPackageError::Invalid);
    }
    Ok(())
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CreatorPackageError> {
    let value = serde_json::to_value(value).map_err(|_| CreatorPackageError::Invalid)?;
    let mut bytes = Vec::new();
    write_canonical_value(&value, &mut bytes)?;
    Ok(bytes)
}

fn write_canonical_value(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> Result<(), CreatorPackageError> {
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(value) => {
            output.extend_from_slice(if *value { b"true" } else { b"false" });
        }
        serde_json::Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::String(value) => {
            serde_json::to_writer(output, value).map_err(|_| CreatorPackageError::Invalid)?;
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
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| *key);
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key)
                    .map_err(|_| CreatorPackageError::Invalid)?;
                output.push(b':');
                write_canonical_value(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> String {
    content_hash_from_bytes(sha256(bytes)).to_hex()
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

struct StagingDirectory {
    output: PathBuf,
    path: PathBuf,
}

impl StagingDirectory {
    fn new(requested_output: &Path) -> Result<Self, CreatorPackageError> {
        let absolute = if requested_output.is_absolute() {
            requested_output.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|_| CreatorPackageError::Storage)?
                .join(requested_output)
        };
        if !matches!(
            fs::symlink_metadata(&absolute),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ) {
            return Err(CreatorPackageError::OutputInvalid);
        }
        let parent = absolute
            .parent()
            .ok_or(CreatorPackageError::OutputInvalid)?;
        let resolved_parent =
            fs::canonicalize(parent).map_err(|_| CreatorPackageError::OutputInvalid)?;
        let parent_metadata = fs::symlink_metadata(&resolved_parent)
            .map_err(|_| CreatorPackageError::OutputInvalid)?;
        if !parent_metadata.is_dir() || parent_metadata.file_type().is_symlink() {
            return Err(CreatorPackageError::OutputInvalid);
        }
        let output_name = absolute
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(CreatorPackageError::OutputInvalid)?;
        let output = resolved_parent.join(output_name);
        for _ in 0..1_024 {
            let ordinal = STAGING_ORDINAL.fetch_add(1, Ordering::Relaxed);
            let path = resolved_parent.join(format!(
                ".{output_name}.creator-staging-{}-{ordinal}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { output, path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(CreatorPackageError::Storage),
            }
        }
        Err(CreatorPackageError::Storage)
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn publish(self) -> Result<(), CreatorPackageError> {
        if !matches!(
            fs::symlink_metadata(&self.output),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ) {
            return Err(CreatorPackageError::OutputInvalid);
        }
        fs::rename(&self.path, &self.output).map_err(|_| CreatorPackageError::Storage)?;
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
