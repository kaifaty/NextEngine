use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[cfg(not(windows))]
use std::fs::File;

use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::cognition::{AgentCognitionSnapshotV1, AgentMemorySnapshotV1};
use next_contracts::persistence::{
    ManifestCodecError, ManifestValidationError, SaveCompatibility, SaveManifestV2,
};
use next_contracts::physical_animation::PhysicalAnimationSnapshotV1;
use next_contracts::physics::PhysicsWorldCheckpointV1;
use next_contracts::rpg::RpgSnapshotV2;
use next_contracts::snapshot::{RuntimeSnapshotV3, WorldCheckpointV4};
use next_contracts::world::WorldStreamingSnapshotV1;
use next_contracts::world_activity::WorldActivitySnapshotV1;
use next_contracts::world_population::WorldPopulationSnapshotV1;
use next_contracts::world_routine::WorldRoutineSnapshotV1;

use super::error::{PreservedFile, RejectedGeneration, SaveStoreError};
use super::image::SaveImage;

pub(super) const SLOT_COUNT: u64 = 2;
pub(super) const MANIFEST_FILE: &str = "manifest.jcs";
pub(super) const SEGMENTS_DIRECTORY: &str = "segments";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedSave {
    pub image: SaveImage,
    pub checkpoint: WorldCheckpointV4,
    pub snapshot: RuntimeSnapshotV3,
    pub rpg_snapshot: RpgSnapshotV2,
    pub physics_checkpoint: PhysicsWorldCheckpointV1,
    pub world_streaming_snapshot: Option<WorldStreamingSnapshotV1>,
    pub world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    pub world_population_snapshot_or_none: Option<WorldPopulationSnapshotV1>,
    pub world_activity_snapshot_or_none: Option<WorldActivitySnapshotV1>,
    pub agent_cognition_snapshot_or_none: Option<AgentCognitionSnapshotV1>,
    pub agent_memory_snapshot_or_none: Option<AgentMemorySnapshotV1>,
    pub physical_animation_snapshot_or_none: Option<PhysicalAnimationSnapshotV1>,
    pub slot: u8,
    pub rejected_generations: Vec<RejectedGeneration>,
}

pub(super) fn read_generation_directory(
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
    let manifest =
        match SaveManifestV2::from_jcs_bytes(manifest_bytes, CanonicalDecodeLimits::default()) {
            Ok(manifest) => manifest,
            Err(ManifestCodecError::Validation(
                ManifestValidationError::UnsupportedSaveVersion(_),
            )) => return Err(reject("UNSUPPORTED_SAVE_MANIFEST_VERSION")),
            Err(_) => return Err(reject("SAVE_MANIFEST_INVALID")),
        };
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
    let validated = image
        .validate_world()
        .map_err(|_| reject("SAVE_IMAGE_INVALID"))?;
    Ok(LoadedSave {
        image,
        checkpoint: validated.checkpoint,
        snapshot: validated.runtime_snapshot,
        rpg_snapshot: validated.rpg_snapshot,
        physics_checkpoint: validated.physics_checkpoint,
        world_streaming_snapshot: validated.world_streaming_snapshot,
        world_routine_snapshot_or_none: validated.world_routine_snapshot_or_none,
        world_population_snapshot_or_none: validated.world_population_snapshot_or_none,
        world_activity_snapshot_or_none: validated.world_activity_snapshot_or_none,
        agent_cognition_snapshot_or_none: validated.agent_cognition_snapshot_or_none,
        agent_memory_snapshot_or_none: validated.agent_memory_snapshot_or_none,
        physical_animation_snapshot_or_none: validated.physical_animation_snapshot_or_none,
        slot,
        rejected_generations: vec![],
    })
}

/// Probes a generation directory for validity without materializing the
/// decoded world. Accept/reject behavior and preserved-file capture are
/// identical to `read_generation_directory`; only the payload differs, so
/// callers that need just validity and the image avoid checkpoint
/// reconstruction and state-root recomputation.
pub(super) fn probe_generation_directory(
    path: &Path,
    slot: u8,
) -> Result<SaveImage, RejectedGeneration> {
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
    let manifest =
        match SaveManifestV2::from_jcs_bytes(manifest_bytes, CanonicalDecodeLimits::default()) {
            Ok(manifest) => manifest,
            Err(ManifestCodecError::Validation(
                ManifestValidationError::UnsupportedSaveVersion(_),
            )) => return Err(reject("UNSUPPORTED_SAVE_MANIFEST_VERSION")),
            Err(_) => return Err(reject("SAVE_MANIFEST_INVALID")),
        };
    if manifest.generation % SLOT_COUNT != u64::from(slot) {
        return Err(reject("SAVE_GENERATION_SLOT_MISMATCH"));
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
    image
        .validate_world_light()
        .map_err(|_| reject("SAVE_IMAGE_INVALID"))?;
    Ok(image)
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

pub(super) fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), SaveStoreError> {
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

#[cfg(not(windows))]
pub(super) fn sync_directory(path: &Path) -> Result<(), SaveStoreError> {
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

#[cfg(windows)]
pub(super) fn sync_directory(_path: &Path) -> Result<(), SaveStoreError> {
    // Rust's portable filesystem API cannot open a Windows directory for
    // `sync_all`. Save files are still synced individually before each atomic
    // rename; SPEC-03 requires directory fsync only where the platform permits.
    Ok(())
}

pub(super) fn remove_directory_if_present(path: &Path) -> Result<(), SaveStoreError> {
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

pub(super) fn remove_file_if_present(path: &Path) -> Result<(), SaveStoreError> {
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

pub(super) fn segment_file_name(index: usize) -> String {
    format!("{index:08}.necb")
}
