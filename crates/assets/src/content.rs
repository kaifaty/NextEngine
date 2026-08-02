use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};

pub const CONTENT_GENERATIONS_DIRECTORY: &str = "generations";
pub const CONTENT_CURRENT_FILE: &str = "CURRENT";
pub const CONTENT_INDEX_FILE: &str = "INDEX.v1";
pub const CONTENT_MAX_FILES: usize = 16_384;
pub const CONTENT_MAX_FILE_BYTES: usize = 256 * 1024 * 1024;
const CONTENT_MAX_REMOVAL_TREE_DEPTH: usize = 64;
const CONTENT_MAX_REMOVAL_TREE_ENTRIES: usize = CONTENT_MAX_FILES * 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationFileV1 {
    relative_path: String,
    bytes: Vec<u8>,
    bytes_sha256: ContentHash,
}

impl PublicationFileV1 {
    pub fn new(
        relative_path: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Result<Self, ContentStoreError> {
        let relative_path = relative_path.into();
        validate_relative_path(&relative_path)?;
        if bytes.len() > CONTENT_MAX_FILE_BYTES {
            return Err(ContentStoreError::LimitExceeded {
                actual: bytes.len(),
                limit: CONTENT_MAX_FILE_BYTES,
            });
        }
        let bytes_sha256 = content_hash_from_bytes(sha256(&bytes));
        Ok(Self {
            relative_path,
            bytes,
            bytes_sha256,
        })
    }

    #[must_use]
    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn bytes_sha256(&self) -> ContentHash {
        self.bytes_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPublicationV1 {
    pub generation_id: ContentHash,
    pub files: Vec<PublicationFileV1>,
}

impl ContentPublicationV1 {
    pub fn new(
        generation_id: ContentHash,
        mut files: Vec<PublicationFileV1>,
    ) -> Result<Self, ContentStoreError> {
        if files.len() > CONTENT_MAX_FILES {
            return Err(ContentStoreError::LimitExceeded {
                actual: files.len(),
                limit: CONTENT_MAX_FILES,
            });
        }
        files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        if files
            .windows(2)
            .any(|pair| pair[0].relative_path == pair[1].relative_path)
        {
            return Err(ContentStoreError::DuplicatePath);
        }
        Ok(Self {
            generation_id,
            files,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedContentGenerationV1 {
    pub generation_id: ContentHash,
    pub files: BTreeMap<String, Vec<u8>>,
}

impl PublishedContentGenerationV1 {
    #[must_use]
    pub fn file(&self, relative_path: &str) -> Option<&[u8]> {
        self.files.get(relative_path).map(Vec::as_slice)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContentPublishFault {
    None,
    BeforeGenerationCommit,
    BeforeCurrentSwitch,
}

#[derive(Clone, Debug)]
pub struct ContentStore {
    root: PathBuf,
}

impl ContentStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn publish(&self, publication: &ContentPublicationV1) -> Result<(), ContentStoreError> {
        self.publish_inner(publication, ContentPublishFault::None)
    }

    #[cfg(test)]
    pub(crate) fn publish_with_fault(
        &self,
        publication: &ContentPublicationV1,
        fault: ContentPublishFault,
    ) -> Result<(), ContentStoreError> {
        self.publish_inner(publication, fault)
    }

    pub fn load_current(&self) -> Result<PublishedContentGenerationV1, ContentStoreError> {
        let generation_id = self.current_generation_id()?;
        self.load_generation_by_id(generation_id)
    }

    pub(crate) fn current_generation_id(&self) -> Result<ContentHash, ContentStoreError> {
        let current_path = self.root.join(CONTENT_CURRENT_FILE);
        let current = read_bounded(&current_path, 65)?;
        let current = std::str::from_utf8(&current).map_err(|_| ContentStoreError::InvalidIndex)?;
        parse_hash(current.trim_end_matches('\n'))
    }

    pub(crate) fn load_generation_by_id(
        &self,
        generation_id: ContentHash,
    ) -> Result<PublishedContentGenerationV1, ContentStoreError> {
        let generation_path = self
            .root
            .join(CONTENT_GENERATIONS_DIRECTORY)
            .join(generation_id.to_hex());
        load_generation(&generation_path, generation_id)
    }

    pub(crate) fn prune_generations_except(
        &self,
        retained_generations: &[ContentHash],
    ) -> Result<(), ContentStoreError> {
        let generations_path = self.root.join(CONTENT_GENERATIONS_DIRECTORY);
        let generations_metadata = match fs::symlink_metadata(&generations_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        validate_plain_directory(&generations_metadata)?;

        let mut entries: Vec<_> = fs::read_dir(&generations_path)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let file_name = entry.file_name();
            let file_name = file_name.to_str().ok_or(ContentStoreError::InvalidPath)?;
            let generation_path = entry.path();
            if generation_path.parent() != Some(generations_path.as_path()) {
                return Err(ContentStoreError::InvalidPath);
            }
            if is_staging_generation_name(file_name) {
                validate_generation_tree_for_removal(&generation_path, &generation_path)?;
                fs::remove_dir_all(&generation_path)?;
                continue;
            }
            let generation_id =
                parse_hash(file_name).map_err(|_| ContentStoreError::InvalidPath)?;
            validate_plain_directory(&fs::symlink_metadata(&generation_path)?)?;

            if retained_generations.contains(&generation_id) {
                continue;
            }

            validate_generation_tree_for_removal(&generation_path, &generation_path)?;
            fs::remove_dir_all(&generation_path)?;
        }
        Ok(())
    }

    fn publish_inner(
        &self,
        publication: &ContentPublicationV1,
        fault: ContentPublishFault,
    ) -> Result<(), ContentStoreError> {
        // Publications assembled by `ContentPublicationV1::new` are already
        // count-bounded, sorted and duplicate-free, so the common path
        // validates and publishes the borrowed files without a defensive
        // clone of every payload. Unsorted input falls back to the
        // canonicalizing constructor, preserving its exact behavior.
        let canonical_owned;
        let canonical: &ContentPublicationV1 = if publication
            .files
            .windows(2)
            .any(|pair| pair[0].relative_path > pair[1].relative_path)
        {
            canonical_owned =
                ContentPublicationV1::new(publication.generation_id, publication.files.clone())?;
            &canonical_owned
        } else {
            if publication.files.len() > CONTENT_MAX_FILES {
                return Err(ContentStoreError::LimitExceeded {
                    actual: publication.files.len(),
                    limit: CONTENT_MAX_FILES,
                });
            }
            if publication
                .files
                .windows(2)
                .any(|pair| pair[0].relative_path == pair[1].relative_path)
            {
                return Err(ContentStoreError::DuplicatePath);
            }
            publication
        };
        fs::create_dir_all(self.root.join(CONTENT_GENERATIONS_DIRECTORY))?;
        let generation_hex = canonical.generation_id.to_hex();
        let generation_path = self
            .root
            .join(CONTENT_GENERATIONS_DIRECTORY)
            .join(&generation_hex);
        if generation_path.exists() {
            let loaded = load_generation(&generation_path, canonical.generation_id)?;
            ensure_matches(&loaded, canonical)?;
        } else {
            let staging_path = self
                .root
                .join(CONTENT_GENERATIONS_DIRECTORY)
                .join(format!(".{generation_hex}.staging.{}", std::process::id()));
            if staging_path.exists() {
                return Err(ContentStoreError::StagingConflict);
            }
            fs::create_dir(&staging_path)?;
            let staging_result = (|| {
                for file in &canonical.files {
                    let output_path = staging_path.join(&file.relative_path);
                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    write_synced(&output_path, &file.bytes)?;
                }
                write_synced(
                    &staging_path.join(CONTENT_INDEX_FILE),
                    &index_bytes(canonical),
                )?;
                verify_staged_generation_bytes(&staging_path, canonical)?;
                if fault == ContentPublishFault::BeforeGenerationCommit {
                    return Err(ContentStoreError::InjectedFault);
                }
                fs::rename(&staging_path, &generation_path)?;
                Ok(())
            })();
            if staging_result.is_err() && staging_path.exists() {
                fs::remove_dir_all(&staging_path)?;
            }
            staging_result?;
        }

        if fault == ContentPublishFault::BeforeCurrentSwitch {
            return Err(ContentStoreError::InjectedFault);
        }
        let current_tmp = self
            .root
            .join(format!(".{CONTENT_CURRENT_FILE}.{}", std::process::id()));
        write_synced(&current_tmp, format!("{generation_hex}\n").as_bytes())?;
        fs::rename(current_tmp, self.root.join(CONTENT_CURRENT_FILE))?;
        Ok(())
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ContentStoreError {
    Io(std::io::Error),
    InvalidPath,
    DuplicatePath,
    LimitExceeded { actual: usize, limit: usize },
    InvalidIndex,
    HashMismatch,
    MissingFile(String),
    UnexpectedFile(String),
    StagingConflict,
    InjectedFault,
}

impl Display for ContentStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "content store I/O failed: {error}"),
            Self::InvalidPath => formatter.write_str("content path must be safe and relative"),
            Self::DuplicatePath => formatter.write_str("content publication path is duplicated"),
            Self::LimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "content size/count {actual} exceeds limit {limit}"
                )
            }
            Self::InvalidIndex => formatter.write_str("content generation index is invalid"),
            Self::HashMismatch => formatter.write_str("content file hash mismatch"),
            Self::MissingFile(path) => write!(formatter, "content file is missing: {path}"),
            Self::UnexpectedFile(path) => write!(formatter, "unexpected content file: {path}"),
            Self::StagingConflict => formatter.write_str("content staging generation exists"),
            Self::InjectedFault => formatter.write_str("injected content publication fault"),
        }
    }
}

impl Error for ContentStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ContentStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

fn validate_relative_path(path: &str) -> Result<(), ContentStoreError> {
    if path.is_empty() || path.contains('\\') {
        return Err(ContentStoreError::InvalidPath);
    }
    let path = Path::new(path);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ContentStoreError::InvalidPath);
    }
    if path
        .components()
        .any(|component| matches!(component, Component::CurDir))
    {
        return Err(ContentStoreError::InvalidPath);
    }
    Ok(())
}

fn index_bytes(publication: &ContentPublicationV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.content-index.v1\n");
    bytes.extend_from_slice(publication.generation_id.to_hex().as_bytes());
    bytes.push(b'\n');
    for file in &publication.files {
        bytes.extend_from_slice(file.bytes_sha256.to_hex().as_bytes());
        bytes.push(b' ');
        bytes.extend_from_slice(file.relative_path.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

fn load_generation(
    generation_path: &Path,
    expected_generation: ContentHash,
) -> Result<PublishedContentGenerationV1, ContentStoreError> {
    let index = read_bounded(&generation_path.join(CONTENT_INDEX_FILE), 4 * 1024 * 1024)?;
    let index = std::str::from_utf8(&index).map_err(|_| ContentStoreError::InvalidIndex)?;
    let mut lines = index.lines();
    if lines.next() != Some("nextengine.content-index.v1") {
        return Err(ContentStoreError::InvalidIndex);
    }
    if parse_hash(lines.next().ok_or(ContentStoreError::InvalidIndex)?)? != expected_generation {
        return Err(ContentStoreError::HashMismatch);
    }
    let mut descriptors = Vec::new();
    let mut previous_path: Option<String> = None;
    for line in lines {
        let Some((hash, path)) = line.split_once(' ') else {
            return Err(ContentStoreError::InvalidIndex);
        };
        validate_relative_path(path)?;
        if previous_path
            .as_deref()
            .is_some_and(|previous| previous >= path)
        {
            return Err(ContentStoreError::InvalidIndex);
        }
        previous_path = Some(path.to_owned());
        descriptors.push((path.to_owned(), parse_hash(hash)?));
    }
    if descriptors.len() > CONTENT_MAX_FILES {
        return Err(ContentStoreError::LimitExceeded {
            actual: descriptors.len(),
            limit: CONTENT_MAX_FILES,
        });
    }

    let expected_paths: BTreeSet<_> = descriptors.iter().map(|(path, _)| path.as_str()).collect();
    let mut actual_paths = BTreeSet::new();
    collect_files(generation_path, generation_path, &mut actual_paths)?;
    actual_paths.remove(CONTENT_INDEX_FILE);
    for path in &actual_paths {
        if !expected_paths.contains(path.as_str()) {
            return Err(ContentStoreError::UnexpectedFile(path.clone()));
        }
    }

    let mut files = BTreeMap::new();
    for (relative_path, expected_hash) in descriptors {
        if !actual_paths.contains(&relative_path) {
            return Err(ContentStoreError::MissingFile(relative_path));
        }
        let bytes = read_bounded(
            &generation_path.join(&relative_path),
            CONTENT_MAX_FILE_BYTES,
        )?;
        if content_hash_from_bytes(sha256(&bytes)) != expected_hash {
            return Err(ContentStoreError::HashMismatch);
        }
        files.insert(relative_path, bytes);
    }
    Ok(PublishedContentGenerationV1 {
        generation_id: expected_generation,
        files,
    })
}

fn collect_files(
    root: &Path,
    directory: &Path,
    output: &mut BTreeSet<String>,
) -> Result<(), ContentStoreError> {
    let mut entries: Vec<_> = fs::read_dir(directory)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(ContentStoreError::InvalidPath);
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_files(root, &path, output)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| ContentStoreError::InvalidPath)?
                .to_str()
                .ok_or(ContentStoreError::InvalidPath)?
                .replace(std::path::MAIN_SEPARATOR, "/");
            validate_relative_path(&relative)?;
            output.insert(relative);
        } else {
            return Err(ContentStoreError::InvalidPath);
        }
    }
    Ok(())
}

fn validate_generation_tree_for_removal(
    root: &Path,
    directory: &Path,
) -> Result<(), ContentStoreError> {
    let mut visited_entries = 0;
    validate_generation_tree_for_removal_inner(root, directory, 0, &mut visited_entries)
}

fn validate_generation_tree_for_removal_inner(
    root: &Path,
    directory: &Path,
    depth: usize,
    visited_entries: &mut usize,
) -> Result<(), ContentStoreError> {
    if depth > CONTENT_MAX_REMOVAL_TREE_DEPTH {
        return Err(ContentStoreError::LimitExceeded {
            actual: depth,
            limit: CONTENT_MAX_REMOVAL_TREE_DEPTH,
        });
    }
    let metadata = fs::symlink_metadata(directory)?;
    validate_plain_directory(&metadata)?;

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        *visited_entries =
            visited_entries
                .checked_add(1)
                .ok_or(ContentStoreError::LimitExceeded {
                    actual: usize::MAX,
                    limit: CONTENT_MAX_REMOVAL_TREE_ENTRIES,
                })?;
        if *visited_entries > CONTENT_MAX_REMOVAL_TREE_ENTRIES {
            return Err(ContentStoreError::LimitExceeded {
                actual: *visited_entries,
                limit: CONTENT_MAX_REMOVAL_TREE_ENTRIES,
            });
        }
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| ContentStoreError::InvalidPath)?
            .to_str()
            .ok_or(ContentStoreError::InvalidPath)?
            .replace(std::path::MAIN_SEPARATOR, "/");
        validate_relative_path(&relative)?;

        let metadata = fs::symlink_metadata(&path)?;
        if is_reparse_point(&metadata) {
            return Err(ContentStoreError::InvalidPath);
        }
        if metadata.is_dir() {
            validate_generation_tree_for_removal_inner(root, &path, depth + 1, visited_entries)?;
        } else if !metadata.is_file() {
            return Err(ContentStoreError::InvalidPath);
        }
    }
    Ok(())
}

fn validate_plain_directory(metadata: &fs::Metadata) -> Result<(), ContentStoreError> {
    if !metadata.is_dir() || is_reparse_point(metadata) {
        return Err(ContentStoreError::InvalidPath);
    }
    Ok(())
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn ensure_matches(
    loaded: &PublishedContentGenerationV1,
    publication: &ContentPublicationV1,
) -> Result<(), ContentStoreError> {
    if loaded.generation_id != publication.generation_id
        || loaded.files.len() != publication.files.len()
    {
        return Err(ContentStoreError::HashMismatch);
    }
    for file in &publication.files {
        if loaded.file(&file.relative_path) != Some(file.bytes.as_slice()) {
            return Err(ContentStoreError::HashMismatch);
        }
    }
    Ok(())
}

/// Byte-exact verification of a freshly staged generation against the
/// canonical in-memory publication.
///
/// This replaces a decode-and-rehash round trip on the write path: staged
/// bytes are compared directly with the bytes intended to be written, which
/// is strictly stronger than hash-consistency checking and detects on-disk
/// corruption with the same failure modes at a fraction of the cost.
fn verify_staged_generation_bytes(
    staging_path: &Path,
    publication: &ContentPublicationV1,
) -> Result<(), ContentStoreError> {
    let staged_index = read_bounded(&staging_path.join(CONTENT_INDEX_FILE), 4 * 1024 * 1024)?;
    if staged_index != index_bytes(publication) {
        return Err(ContentStoreError::HashMismatch);
    }
    let mut actual_paths = BTreeSet::new();
    collect_files(staging_path, staging_path, &mut actual_paths)?;
    actual_paths.remove(CONTENT_INDEX_FILE);
    let expected_paths: BTreeSet<_> = publication
        .files
        .iter()
        .map(|file| file.relative_path.as_str())
        .collect();
    for path in &actual_paths {
        if !expected_paths.contains(path.as_str()) {
            return Err(ContentStoreError::UnexpectedFile(path.clone()));
        }
    }
    for file in &publication.files {
        if !actual_paths.contains(file.relative_path.as_str()) {
            return Err(ContentStoreError::MissingFile(file.relative_path.clone()));
        }
        let staged = read_bounded(
            &staging_path.join(&file.relative_path),
            CONTENT_MAX_FILE_BYTES,
        )?;
        if staged != file.bytes {
            return Err(ContentStoreError::HashMismatch);
        }
    }
    Ok(())
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), ContentStoreError> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, ContentStoreError> {
    let mut file = File::open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ContentStoreError::MissingFile(path.display().to_string())
        } else {
            ContentStoreError::Io(error)
        }
    })?;
    let length =
        usize::try_from(file.metadata()?.len()).map_err(|_| ContentStoreError::LimitExceeded {
            actual: usize::MAX,
            limit,
        })?;
    if length > limit {
        return Err(ContentStoreError::LimitExceeded {
            actual: length,
            limit,
        });
    }
    let mut bytes = Vec::with_capacity(length);
    file.read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(ContentStoreError::LimitExceeded {
            actual: bytes.len(),
            limit,
        });
    }
    Ok(bytes)
}

fn parse_hash(value: &str) -> Result<ContentHash, ContentStoreError> {
    if !is_lower_hex_digest(value) {
        return Err(ContentStoreError::InvalidIndex);
    }
    let mut output = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    Ok(content_hash_from_bytes(output))
}

fn is_staging_generation_name(value: &str) -> bool {
    let Some(value) = value.strip_prefix('.') else {
        return false;
    };
    let Some((generation, process_id)) = value.split_once(".staging.") else {
        return false;
    };
    let Ok(parsed_process_id) = process_id.parse::<u32>() else {
        return false;
    };
    is_lower_hex_digest(generation)
        && parsed_process_id != 0
        && process_id == parsed_process_id.to_string()
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{
        CONTENT_GENERATIONS_DIRECTORY, ContentPublicationV1, ContentPublishFault, ContentStore,
        ContentStoreError, PublicationFileV1,
    };
    use next_contracts::ids::content_hash_from_bytes;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn path_escape_is_rejected() {
        assert!(matches!(
            PublicationFileV1::new("../escape", Vec::new()),
            Err(ContentStoreError::InvalidPath)
        ));
        assert!(matches!(
            PublicationFileV1::new("/absolute", Vec::new()),
            Err(ContentStoreError::InvalidPath)
        ));
    }

    #[test]
    fn fault_does_not_switch_current_generation() {
        let root = test_root("fault");
        let store = ContentStore::new(&root);
        let first = publication(1, b"first");
        store.publish(&first).expect("first publication");
        let second = publication(2, b"second");
        assert!(matches!(
            store.publish_inner(&second, ContentPublishFault::BeforeCurrentSwitch),
            Err(ContentStoreError::InjectedFault)
        ));
        let loaded = store
            .load_current()
            .expect("previous generation remains active");
        assert_eq!(loaded.generation_id, first.generation_id);
        assert_eq!(
            loaded.file("manifests/value.bin"),
            Some(b"first".as_slice())
        );
        std::fs::remove_dir_all(root).expect("remove test store");
    }

    #[test]
    fn repeated_publication_is_byte_identical() {
        let root = test_root("repeat");
        let store = ContentStore::new(&root);
        let publication = publication(7, b"same");
        store.publish(&publication).expect("first publication");
        store.publish(&publication).expect("idempotent publication");
        assert_eq!(
            store
                .load_current()
                .expect("load")
                .file("manifests/value.bin"),
            Some(b"same".as_slice())
        );
        std::fs::remove_dir_all(root).expect("remove test store");
    }

    #[test]
    fn generic_content_store_keeps_all_published_generations() {
        let root = test_root("no-global-retention");
        let store = ContentStore::new(&root);
        for (id, bytes) in [(1, b"one".as_slice()), (2, b"two"), (3, b"three")] {
            store
                .publish(&publication(id, bytes))
                .expect("content publication");
        }
        let generations = std::fs::read_dir(root.join(CONTENT_GENERATIONS_DIRECTORY))
            .expect("generation directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("generation entries");
        assert_eq!(generations.len(), 3);
        std::fs::remove_dir_all(root).expect("remove test store");
    }

    #[test]
    fn staged_byte_verification_detects_corruption() {
        let root = test_root("staged-verify");
        let publication = publication(9, b"staged");
        let staging = root.join("staging");
        std::fs::create_dir_all(staging.join("manifests")).expect("create staging");
        std::fs::write(staging.join("manifests/value.bin"), b"staged").expect("write file");
        std::fs::write(
            staging.join(super::CONTENT_INDEX_FILE),
            super::index_bytes(&publication),
        )
        .expect("write index");
        super::verify_staged_generation_bytes(&staging, &publication)
            .expect("byte-identical staging verifies");

        let value_path = staging.join("manifests/value.bin");
        let mut corrupted = std::fs::read(&value_path).expect("read staged file");
        corrupted[0] ^= 0xff;
        std::fs::write(&value_path, corrupted).expect("corrupt staged file");
        assert!(matches!(
            super::verify_staged_generation_bytes(&staging, &publication),
            Err(ContentStoreError::HashMismatch)
        ));

        std::fs::write(&value_path, b"staged").expect("restore staged file");
        std::fs::write(staging.join(super::CONTENT_INDEX_FILE), b"tampered-index")
            .expect("corrupt staged index");
        assert!(matches!(
            super::verify_staged_generation_bytes(&staging, &publication),
            Err(ContentStoreError::HashMismatch)
        ));

        std::fs::write(
            staging.join(super::CONTENT_INDEX_FILE),
            super::index_bytes(&publication),
        )
        .expect("restore staged index");
        std::fs::write(staging.join("unexpected.bin"), b"extra").expect("add unexpected file");
        assert!(matches!(
            super::verify_staged_generation_bytes(&staging, &publication),
            Err(ContentStoreError::UnexpectedFile(_))
        ));

        std::fs::remove_file(staging.join("unexpected.bin")).expect("remove unexpected file");
        std::fs::remove_file(&value_path).expect("remove staged file");
        assert!(matches!(
            super::verify_staged_generation_bytes(&staging, &publication),
            Err(ContentStoreError::MissingFile(_))
        ));
        std::fs::remove_dir_all(root).expect("remove test store");
    }

    fn publication(byte: u8, bytes: &[u8]) -> ContentPublicationV1 {
        ContentPublicationV1::new(
            content_hash_from_bytes([byte; 32]),
            vec![
                PublicationFileV1::new("manifests/value.bin", bytes.to_vec()).expect("valid file"),
            ],
        )
        .expect("valid publication")
    }

    fn test_root(label: &str) -> std::path::PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "nextengine-content-{label}-{}-{counter}",
            std::process::id()
        ))
    }
}
