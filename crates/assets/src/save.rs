use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use next_contracts::{
    CanonicalDecodeLimits, CanonicalError, CommandLedgerDescriptor, ManifestCodecError,
    ManifestValidationError, RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID,
    RUNTIME_SNAPSHOT_SEGMENT_ID, RuntimeSnapshot, SaveCompatibility, SaveManifestV1,
    SnapshotDecodeError,
};

const SLOT_COUNT: u64 = 2;
const MANIFEST_FILE: &str = "manifest.jcs";
const SEGMENTS_DIRECTORY: &str = "segments";
const CURRENT_FILE: &str = "CURRENT";
const CURRENT_STAGING_FILE: &str = "CURRENT.new";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveImage {
    pub manifest: SaveManifestV1,
    pub segments: Vec<Vec<u8>>,
}

impl SaveImage {
    pub fn from_runtime_snapshot(
        generation: u64,
        compatibility: SaveCompatibility,
        snapshot: &RuntimeSnapshot,
    ) -> Result<Self, SaveStoreError> {
        let snapshot_bytes = snapshot.canonical_bytes()?;
        let manifest = SaveManifestV1::for_runtime_snapshot(
            generation,
            compatibility,
            snapshot,
            &snapshot_bytes,
        )?;
        let image = Self {
            manifest,
            segments: vec![snapshot_bytes],
        };
        let _ = image.validate()?;
        Ok(image)
    }

    pub fn validate(&self) -> Result<RuntimeSnapshot, SaveStoreError> {
        self.manifest.validate()?;
        if self.manifest.segments.len() != self.segments.len() {
            return Err(SaveStoreError::InvalidImage("SAVE_SEGMENT_COUNT_MISMATCH"));
        }
        for (descriptor, bytes) in self.manifest.segments.iter().zip(&self.segments) {
            if !descriptor.matches_bytes(bytes) {
                return Err(SaveStoreError::InvalidImage("SAVE_SEGMENT_HASH_MISMATCH"));
            }
        }

        let runtime_index = self
            .manifest
            .segments
            .iter()
            .position(|segment| {
                segment.owner_id.as_str() == RUNTIME_SNAPSHOT_OWNER_ID
                    && segment.schema_id.as_str() == RUNTIME_SNAPSHOT_SCHEMA_ID
                    && segment.segment_id.as_str() == RUNTIME_SNAPSHOT_SEGMENT_ID
            })
            .ok_or(SaveStoreError::InvalidImage(
                "SAVE_RUNTIME_SNAPSHOT_MISSING",
            ))?;
        let snapshot = RuntimeSnapshot::from_canonical_bytes(
            &self.segments[runtime_index],
            CanonicalDecodeLimits::default(),
        )?;
        if snapshot.authoritative_revision != self.manifest.world_revision {
            return Err(SaveStoreError::InvalidImage("SAVE_WORLD_REVISION_MISMATCH"));
        }
        let expected_ledgers = snapshot
            .command_ledgers
            .iter()
            .map(CommandLedgerDescriptor::from)
            .collect::<Vec<_>>();
        if expected_ledgers != self.manifest.command_ledgers {
            return Err(SaveStoreError::InvalidImage("SAVE_COMMAND_LEDGER_MISMATCH"));
        }
        Ok(snapshot)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveCommitReceipt {
    pub generation: u64,
    pub slot: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedSave {
    pub image: SaveImage,
    pub snapshot: RuntimeSnapshot,
    pub slot: u8,
    pub rejected_generations: Vec<RejectedGeneration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreservedFile {
    pub relative_path: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectedGeneration {
    pub slot: u8,
    pub stable_code: &'static str,
    pub original_files: Vec<PreservedFile>,
}

#[derive(Clone, Debug)]
pub struct SaveStore {
    root: PathBuf,
}

impl SaveStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn commit_runtime_snapshot(
        &self,
        compatibility: SaveCompatibility,
        snapshot: &RuntimeSnapshot,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        self.commit_runtime_snapshot_inner(compatibility, snapshot, None)
    }

    pub fn load_latest(
        &self,
        expected_compatibility: &SaveCompatibility,
    ) -> Result<LoadedSave, SaveLoadError> {
        let (mut candidates, rejected) = self.load_candidates(Some(expected_compatibility));
        candidates.sort_by_key(|candidate| candidate.image.manifest.generation);
        let Some(mut loaded) = candidates.pop() else {
            return Err(SaveLoadError::NoValidGeneration { rejected });
        };
        loaded.rejected_generations = rejected;
        Ok(loaded)
    }

    fn commit_runtime_snapshot_inner(
        &self,
        compatibility: SaveCompatibility,
        snapshot: &RuntimeSnapshot,
        fault: Option<CommitBoundary>,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        fs::create_dir_all(&self.root)
            .map_err(|source| SaveStoreError::io("create save root", &self.root, source))?;
        let (candidates, _) = self.load_candidates(None);
        let next_generation = candidates
            .iter()
            .map(|candidate| candidate.image.manifest.generation)
            .max()
            .map_or(Ok(0), |generation| {
                generation
                    .checked_add(1)
                    .ok_or(SaveStoreError::GenerationExhausted)
            })?;
        let image = SaveImage::from_runtime_snapshot(next_generation, compatibility, snapshot)?;
        let slot = u8::try_from(next_generation % SLOT_COUNT)
            .map_err(|_| SaveStoreError::GenerationExhausted)?;
        let staging = self.staging_path(slot);
        remove_directory_if_present(&staging)?;
        fs::create_dir_all(staging.join(SEGMENTS_DIRECTORY))
            .map_err(|source| SaveStoreError::io("create save staging", &staging, source))?;
        maybe_inject(fault, CommitBoundary::StagingCreated)?;

        for (index, bytes) in image.segments.iter().enumerate() {
            let path = staging
                .join(SEGMENTS_DIRECTORY)
                .join(segment_file_name(index));
            write_new_synced(&path, bytes)?;
        }
        sync_directory(&staging.join(SEGMENTS_DIRECTORY))?;
        maybe_inject(fault, CommitBoundary::SegmentsSynced)?;

        let manifest_bytes = image.manifest.to_jcs_bytes()?;
        write_new_synced(&staging.join(MANIFEST_FILE), &manifest_bytes)?;
        sync_directory(&staging)?;
        maybe_inject(fault, CommitBoundary::ManifestSynced)?;

        let staged = read_generation_directory(&staging, slot, None)
            .map_err(|rejected| SaveStoreError::InvalidStaging(rejected.stable_code))?;
        if staged.image != image {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_STAGING_ROUND_TRIP_MISMATCH",
            ));
        }
        maybe_inject(fault, CommitBoundary::StagingValidated)?;

        let slot_path = self.slot_path(slot);
        remove_directory_if_present(&slot_path)?;
        maybe_inject(fault, CommitBoundary::InactiveSlotRemoved)?;
        fs::rename(&staging, &slot_path)
            .map_err(|source| SaveStoreError::io("publish save generation", &slot_path, source))?;
        sync_directory(&self.root)?;
        maybe_inject(fault, CommitBoundary::GenerationPublished)?;

        let pointer_staging = self.root.join(CURRENT_STAGING_FILE);
        remove_file_if_present(&pointer_staging)?;
        write_new_synced(
            &pointer_staging,
            format!("{next_generation} {slot}\n").as_bytes(),
        )?;
        maybe_inject(fault, CommitBoundary::PointerStaged)?;
        remove_file_if_present(&self.root.join(CURRENT_FILE))?;
        maybe_inject(fault, CommitBoundary::PriorPointerRemoved)?;
        fs::rename(&pointer_staging, self.root.join(CURRENT_FILE)).map_err(|source| {
            SaveStoreError::io("publish save pointer", self.root.join(CURRENT_FILE), source)
        })?;
        sync_directory(&self.root)?;
        maybe_inject(fault, CommitBoundary::PointerPublished)?;

        Ok(SaveCommitReceipt {
            generation: next_generation,
            slot,
        })
    }

    fn load_candidates(
        &self,
        expected_compatibility: Option<&SaveCompatibility>,
    ) -> (Vec<LoadedSave>, Vec<RejectedGeneration>) {
        let mut candidates = Vec::new();
        let mut rejected = Vec::new();
        for slot in 0..2_u8 {
            let path = self.slot_path(slot);
            match fs::symlink_metadata(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => {
                    rejected.push(RejectedGeneration {
                        slot,
                        stable_code: "SAVE_SLOT_UNREADABLE",
                        original_files: vec![],
                    });
                    continue;
                }
                Ok(metadata) if !metadata.file_type().is_dir() => {
                    rejected.push(RejectedGeneration {
                        slot,
                        stable_code: "SAVE_SLOT_NOT_REGULAR_DIRECTORY",
                        original_files: vec![],
                    });
                    continue;
                }
                Ok(_) => {}
            }
            match read_generation_directory(&path, slot, expected_compatibility) {
                Ok(candidate) => candidates.push(candidate),
                Err(rejection) => rejected.push(rejection),
            }
        }
        (candidates, rejected)
    }

    fn slot_path(&self, slot: u8) -> PathBuf {
        self.root.join(format!("slot-{slot}"))
    }

    fn staging_path(&self, slot: u8) -> PathBuf {
        self.root.join(format!("slot-{slot}.staging"))
    }
}

fn read_generation_directory(
    path: &Path,
    slot: u8,
    expected_compatibility: Option<&SaveCompatibility>,
) -> Result<LoadedSave, RejectedGeneration> {
    let original_files = preserve_generation_files(path);
    let reject = |stable_code| RejectedGeneration {
        slot,
        stable_code,
        original_files: original_files.clone(),
    };
    let files = original_files
        .iter()
        .map(|file| (file.relative_path.as_str(), file.bytes.as_slice()))
        .collect::<BTreeMap<_, _>>();
    if !generation_layout_is_regular(path) {
        return Err(reject("SAVE_FILE_SET_MISMATCH"));
    }
    let manifest_bytes = files
        .get(MANIFEST_FILE)
        .copied()
        .ok_or_else(|| reject("SAVE_MANIFEST_MISSING"))?;
    let manifest = SaveManifestV1::from_jcs_bytes(manifest_bytes, CanonicalDecodeLimits::default())
        .map_err(|_| reject("SAVE_MANIFEST_INVALID"))?;
    if manifest.generation % SLOT_COUNT != u64::from(slot) {
        return Err(reject("SAVE_GENERATION_SLOT_MISMATCH"));
    }
    if expected_compatibility.is_some_and(|expected| &manifest.compatibility != expected) {
        return Err(reject("SAVE_COMPATIBILITY_MISMATCH"));
    }
    let expected_file_count = manifest.segments.len().saturating_add(1);
    if files.len() != expected_file_count {
        return Err(reject("SAVE_FILE_SET_MISMATCH"));
    }
    let mut segments = Vec::with_capacity(manifest.segments.len());
    for index in 0..manifest.segments.len() {
        let name = format!("{SEGMENTS_DIRECTORY}/{}", segment_file_name(index));
        let bytes = files
            .get(name.as_str())
            .copied()
            .ok_or_else(|| reject("SAVE_SEGMENT_MISSING"))?;
        segments.push(bytes.to_vec());
    }
    let image = SaveImage { manifest, segments };
    let snapshot = image.validate().map_err(|_| reject("SAVE_IMAGE_INVALID"))?;
    Ok(LoadedSave {
        image,
        snapshot,
        slot,
        rejected_generations: vec![],
    })
}

fn preserve_generation_files(path: &Path) -> Vec<PreservedFile> {
    let mut files = Vec::new();
    if let Some(bytes) = read_regular_file(&path.join(MANIFEST_FILE)) {
        files.push(PreservedFile {
            relative_path: MANIFEST_FILE.to_owned(),
            bytes,
        });
    }
    let segments = path.join(SEGMENTS_DIRECTORY);
    if let Ok(entries) = fs::read_dir(segments) {
        let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }
            if let Ok(bytes) = fs::read(entry.path()) {
                files.push(PreservedFile {
                    relative_path: format!(
                        "{SEGMENTS_DIRECTORY}/{}",
                        entry.file_name().to_string_lossy()
                    ),
                    bytes,
                });
            }
        }
    }
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    files
}

fn read_regular_file(path: &Path) -> Option<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    fs::read(path).ok()
}

fn generation_layout_is_regular(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    let mut manifest_seen = false;
    let mut segments_seen = false;
    for entry in entries {
        let Ok(entry) = entry else {
            return false;
        };
        let Ok(file_type) = entry.file_type() else {
            return false;
        };
        match entry.file_name().to_str() {
            Some(MANIFEST_FILE) if file_type.is_file() => manifest_seen = true,
            Some(SEGMENTS_DIRECTORY) if file_type.is_dir() => segments_seen = true,
            _ => return false,
        }
    }
    if !manifest_seen || !segments_seen {
        return false;
    }
    let Ok(entries) = fs::read_dir(path.join(SEGMENTS_DIRECTORY)) else {
        return false;
    };
    entries.into_iter().all(|entry| {
        entry
            .ok()
            .and_then(|entry| entry.file_type().ok())
            .is_some_and(|file_type| file_type.is_file())
    })
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), SaveStoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| SaveStoreError::io("create staged save file", path, source))?;
    file.write_all(bytes)
        .map_err(|source| SaveStoreError::io("write staged save file", path, source))?;
    file.sync_all()
        .map_err(|source| SaveStoreError::io("sync staged save file", path, source))
}

fn sync_directory(path: &Path) -> Result<(), SaveStoreError> {
    match File::open(path).and_then(|directory| directory.sync_all()) {
        Ok(()) => Ok(()),
        Err(source)
            if matches!(
                source.kind(),
                std::io::ErrorKind::InvalidInput | std::io::ErrorKind::Unsupported
            ) =>
        {
            Ok(())
        }
        Err(source) => Err(SaveStoreError::io("sync save directory", path, source)),
    }
}

fn remove_directory_if_present(path: &Path) -> Result<(), SaveStoreError> {
    match fs::symlink_metadata(path) {
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SaveStoreError::io(
            "inspect inactive save directory",
            path,
            source,
        )),
        Ok(metadata) if !metadata.file_type().is_dir() => Err(SaveStoreError::io(
            "inspect inactive save directory",
            path,
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "save generation path is not a regular directory",
            ),
        )),
        Ok(_) => fs::remove_dir_all(path)
            .map_err(|source| SaveStoreError::io("remove inactive save directory", path, source)),
    }
}

fn remove_file_if_present(path: &Path) -> Result<(), SaveStoreError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SaveStoreError::io(
            "remove prior save pointer",
            path,
            source,
        )),
    }
}

fn segment_file_name(index: usize) -> String {
    format!("{index:08}.necb")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommitBoundary {
    StagingCreated,
    SegmentsSynced,
    ManifestSynced,
    StagingValidated,
    InactiveSlotRemoved,
    GenerationPublished,
    PointerStaged,
    PriorPointerRemoved,
    PointerPublished,
}

fn maybe_inject(
    requested: Option<CommitBoundary>,
    boundary: CommitBoundary,
) -> Result<(), SaveStoreError> {
    if requested == Some(boundary) {
        Err(SaveStoreError::InjectedFault)
    } else {
        Ok(())
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SaveStoreError {
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    Canonicalization(CanonicalError),
    ManifestValidation(ManifestValidationError),
    ManifestCodec(ManifestCodecError),
    SnapshotDecode(SnapshotDecodeError),
    InvalidImage(&'static str),
    InvalidStaging(&'static str),
    GenerationExhausted,
    InjectedFault,
}

impl SaveStoreError {
    fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }

    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "SAVE_IO_FAILED",
            Self::Canonicalization(_) => "SAVE_CANONICALIZATION_FAILED",
            Self::ManifestValidation(_) => "SAVE_MANIFEST_INVALID",
            Self::ManifestCodec(_) => "SAVE_MANIFEST_CODEC_FAILED",
            Self::SnapshotDecode(_) => "SAVE_SNAPSHOT_INVALID",
            Self::InvalidImage(code) | Self::InvalidStaging(code) => code,
            Self::GenerationExhausted => "SAVE_GENERATION_EXHAUSTED",
            Self::InjectedFault => "SAVE_FAULT_INJECTED",
        }
    }
}

impl Display for SaveStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(formatter, "{operation} at {}: {source}", path.display()),
            Self::Canonicalization(error) => {
                write!(formatter, "save canonicalization failed: {error}")
            }
            Self::ManifestValidation(error) => {
                write!(formatter, "save manifest is invalid: {error}")
            }
            Self::ManifestCodec(error) => write!(formatter, "save manifest codec failed: {error}"),
            Self::SnapshotDecode(error) => write!(formatter, "save snapshot is invalid: {error}"),
            Self::InvalidImage(code) | Self::InvalidStaging(code) => formatter.write_str(code),
            Self::GenerationExhausted => formatter.write_str("SAVE_GENERATION_EXHAUSTED"),
            Self::InjectedFault => formatter.write_str("SAVE_FAULT_INJECTED"),
        }
    }
}

impl Error for SaveStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Canonicalization(error) => Some(error),
            Self::ManifestValidation(error) => Some(error),
            Self::ManifestCodec(error) => Some(error),
            Self::SnapshotDecode(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CanonicalError> for SaveStoreError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<ManifestValidationError> for SaveStoreError {
    fn from(error: ManifestValidationError) -> Self {
        Self::ManifestValidation(error)
    }
}

impl From<ManifestCodecError> for SaveStoreError {
    fn from(error: ManifestCodecError) -> Self {
        Self::ManifestCodec(error)
    }
}

impl From<SnapshotDecodeError> for SaveStoreError {
    fn from(error: SnapshotDecodeError) -> Self {
        Self::SnapshotDecode(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SaveLoadError {
    NoValidGeneration { rejected: Vec<RejectedGeneration> },
}

impl Display for SaveLoadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoValidGeneration { rejected } => {
                write!(
                    formatter,
                    "no valid save generation; {} rejected",
                    rejected.len()
                )
            }
        }
    }
}

impl Error for SaveLoadError {}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use next_contracts::{ContentHash, RuntimeSnapshot, SaveCompatibility, SchemaId, TickSettings};

    use super::{CommitBoundary, SaveStore};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: std::path::PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nextengine-save-test-{}-{sequence}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("test directory is created");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn compatibility(seed: u8) -> SaveCompatibility {
        SaveCompatibility {
            engine_build_hash: ContentHash::from_bytes([seed; 32]),
            game_build_hash: ContentHash::from_bytes([seed.wrapping_add(1); 32]),
            project_id: SchemaId::new("nextengine.save-test").expect("valid project"),
            schema_registry_hash: ContentHash::from_bytes([seed.wrapping_add(2); 32]),
            content_manifest_hash: ContentHash::from_bytes([seed.wrapping_add(3); 32]),
            mechanics_lock_hash: ContentHash::from_bytes([seed.wrapping_add(4); 32]),
            tick_settings: TickSettings {
                gameplay_hz: 30,
                physics_hz: 120,
                motor_hz: 60,
            },
            loaded_chunk_revisions: vec![],
            rng_stream_states: vec![],
            physical_bindings: vec![],
            policy_state_schemas: vec![],
            plugin_script_bindings: vec![],
        }
    }

    fn snapshot(revision: u64) -> RuntimeSnapshot {
        RuntimeSnapshot {
            next_tick: revision,
            committed_event_count: revision,
            authoritative_revision: revision,
            command_ledgers: vec![],
        }
    }

    #[test]
    fn two_generations_commit_and_latest_loads() {
        let directory = TestDirectory::new();
        let store = SaveStore::new(&directory.path);
        let compatibility = compatibility(1);
        let first = store
            .commit_runtime_snapshot(compatibility.clone(), &snapshot(1))
            .expect("first generation commits");
        let second = store
            .commit_runtime_snapshot(compatibility.clone(), &snapshot(2))
            .expect("second generation commits");

        assert_eq!(first.generation, 0);
        assert_eq!(second.generation, 1);
        let loaded = store
            .load_latest(&compatibility)
            .expect("latest generation loads");
        assert_eq!(loaded.snapshot, snapshot(2));
        assert!(loaded.rejected_generations.is_empty());
    }

    #[test]
    fn corrupt_latest_generation_falls_back_and_preserves_original_bytes() {
        let directory = TestDirectory::new();
        let store = SaveStore::new(&directory.path);
        let compatibility = compatibility(1);
        store
            .commit_runtime_snapshot(compatibility.clone(), &snapshot(1))
            .expect("first generation commits");
        store
            .commit_runtime_snapshot(compatibility.clone(), &snapshot(2))
            .expect("second generation commits");
        let corrupt_path = store
            .slot_path(1)
            .join(super::SEGMENTS_DIRECTORY)
            .join(super::segment_file_name(0));
        let mut corrupt_bytes = std::fs::read(&corrupt_path).expect("segment exists");
        corrupt_bytes[0] ^= 1;
        std::fs::write(&corrupt_path, &corrupt_bytes).expect("test corrupts latest segment");

        let loaded = store
            .load_latest(&compatibility)
            .expect("prior valid generation loads");
        assert_eq!(loaded.snapshot, snapshot(1));
        assert_eq!(loaded.rejected_generations.len(), 1);
        assert!(
            loaded.rejected_generations[0]
                .original_files
                .iter()
                .any(|file| file.bytes == corrupt_bytes)
        );
        assert_eq!(
            std::fs::read(&corrupt_path).expect("load does not rewrite corrupt source"),
            corrupt_bytes
        );
    }

    #[test]
    fn every_commit_boundary_retains_a_valid_generation() {
        let boundaries = [
            CommitBoundary::StagingCreated,
            CommitBoundary::SegmentsSynced,
            CommitBoundary::ManifestSynced,
            CommitBoundary::StagingValidated,
            CommitBoundary::InactiveSlotRemoved,
            CommitBoundary::GenerationPublished,
            CommitBoundary::PointerStaged,
            CommitBoundary::PriorPointerRemoved,
            CommitBoundary::PointerPublished,
        ];
        for boundary in boundaries {
            let directory = TestDirectory::new();
            let store = SaveStore::new(&directory.path);
            let compatibility = compatibility(1);
            store
                .commit_runtime_snapshot(compatibility.clone(), &snapshot(1))
                .expect("baseline generation commits");
            let result = store.commit_runtime_snapshot_inner(
                compatibility.clone(),
                &snapshot(2),
                Some(boundary),
            );
            assert!(result.is_err(), "boundary {boundary:?} must inject a fault");
            let loaded = store
                .load_latest(&compatibility)
                .expect("a valid generation must remain");
            assert!(
                loaded.snapshot == snapshot(1) || loaded.snapshot == snapshot(2),
                "boundary {boundary:?} loaded an unexpected state"
            );
        }
    }

    #[test]
    fn incompatible_save_fails_closed_with_original_files() {
        let directory = TestDirectory::new();
        let store = SaveStore::new(&directory.path);
        let written = compatibility(1);
        store
            .commit_runtime_snapshot(written, &snapshot(1))
            .expect("generation commits");

        let error = store
            .load_latest(&compatibility(9))
            .expect_err("incompatible build must not load");
        let super::SaveLoadError::NoValidGeneration { rejected } = error;
        assert_eq!(rejected.len(), 1);
        assert!(!rejected[0].original_files.is_empty());
        assert_eq!(rejected[0].stable_code, "SAVE_COMPATIBILITY_MISMATCH");
    }
}
