#[cfg(test)]
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
use next_contracts::ids::PhysicsWorldId;
use next_contracts::persistence::SaveCompatibility;
#[cfg(test)]
use next_contracts::physics::{
    PhysicsCanonicalSnapshotV2, PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1,
    PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1,
};
#[cfg(test)]
use next_contracts::rpg::RpgSnapshotV2;
#[cfg(test)]
use next_contracts::snapshot::RuntimeSnapshotV3;
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_contracts::world_routine::WorldRoutineSnapshotV1;

use super::error::{RejectedGeneration, SaveLoadError, SaveStoreError};
use super::generation::{
    LoadedSave, MANIFEST_FILE, SEGMENTS_DIRECTORY, SLOT_COUNT, probe_generation_directory,
    read_generation_directory, remove_directory_if_present, remove_file_if_present,
    segment_file_name, sync_directory, write_new_synced,
};
use super::image::SaveImage;

const CURRENT_FILE: &str = "CURRENT";
const CURRENT_STAGING_FILE: &str = "CURRENT.new";

#[cfg(test)]
pub(super) fn synthetic_empty_checkpoint(
    runtime_snapshot: RuntimeSnapshotV3,
    rpg_snapshot: RpgSnapshotV2,
) -> Result<WorldCheckpointV4, SaveStoreError> {
    if !runtime_snapshot
        .player_controller_registry
        .bindings
        .is_empty()
    {
        return Err(SaveStoreError::InvalidImage(
            "SAVE_PHYSICS_SNAPSHOT_REQUIRED",
        ));
    }
    let physics_tick = runtime_snapshot
        .next_tick
        .checked_mul(u64::from(
            runtime_snapshot
                .tick_rate_profile
                .physics_substeps_per_gameplay_tick,
        ))
        .ok_or(SaveStoreError::InvalidImage("SAVE_PHYSICS_TICK_OVERFLOW"))?;
    let catalog = PhysicsWorldCatalogV1::new(
        PhysicsWorldId::from_bytes(*runtime_snapshot.world_identity.world_namespace.as_bytes()),
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1()
                .expect("built-in coordinate profile"),
            limits: PhysicsLimitsProfileV1::reference_v1().expect("built-in limits profile"),
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1()
                .expect("built-in solver profile"),
            tick_rate_hash: runtime_snapshot.tick_rate_profile.profile_hash()?,
            authoritative_numeric_hash: runtime_snapshot
                .authoritative_numeric_profile
                .profile_hash()?,
            quantization_hash: runtime_snapshot
                .physics_quantization_profile
                .profile_hash()?,
        },
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )?;
    let mut physics_snapshot = PhysicsCanonicalSnapshotV2::genesis(
        &catalog,
        &runtime_snapshot.tick_rate_profile,
        &runtime_snapshot.authoritative_numeric_profile,
        &runtime_snapshot.physics_quantization_profile,
    )?;
    physics_snapshot.checkpoint_revision = runtime_snapshot.authoritative_revision;
    physics_snapshot.physics_tick = physics_tick;
    physics_snapshot.world_revision = physics_tick;
    let physics_checkpoint = PhysicsWorldCheckpointV1::new(catalog, physics_snapshot)?;
    runtime_snapshot
        .canonical_bytes()
        .map_err(|_| SaveStoreError::InvalidImage("SAVE_RUNTIME_SNAPSHOT_INVALID"))?;
    rpg_snapshot
        .canonical_bytes()
        .map_err(|_| SaveStoreError::InvalidImage("SAVE_RPG_SNAPSHOT_INVALID"))?;
    physics_checkpoint
        .canonical_bytes()
        .map_err(|_| SaveStoreError::InvalidImage("SAVE_PHYSICS_SNAPSHOT_INVALID"))?;
    Ok(WorldCheckpointV4::new(
        runtime_snapshot,
        rpg_snapshot,
        physics_checkpoint,
    )?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveCommitReceipt {
    pub generation: u64,
    pub slot: u8,
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

    #[cfg(test)]
    pub(super) fn commit_runtime_snapshot(
        &self,
        compatibility: SaveCompatibility,
        snapshot: &RuntimeSnapshotV3,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        let checkpoint = synthetic_empty_checkpoint(snapshot.clone(), RpgSnapshotV2::default())?;
        self.commit_world_checkpoint_inner(compatibility, &checkpoint, None)
    }

    #[cfg(test)]
    pub(super) fn commit_world_snapshot(
        &self,
        compatibility: SaveCompatibility,
        runtime_snapshot: &RuntimeSnapshotV3,
        rpg_snapshot: &RpgSnapshotV2,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        let checkpoint =
            synthetic_empty_checkpoint(runtime_snapshot.clone(), rpg_snapshot.clone())?;
        self.commit_world_checkpoint_inner(compatibility, &checkpoint, None)
    }

    pub fn commit_world_checkpoint(
        &self,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        self.commit_world_checkpoint_inner(compatibility, checkpoint, None)
    }

    pub fn commit_world_checkpoint_with_streaming(
        &self,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
        world_streaming_snapshot: &WorldStreamingSnapshotV1,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        self.commit_world_checkpoint_inner_with_streaming(
            compatibility,
            checkpoint,
            Some(world_streaming_snapshot),
            None,
            None,
        )
    }

    pub fn commit_world_checkpoint_with_streaming_and_routine(
        &self,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
        world_streaming_snapshot: &WorldStreamingSnapshotV1,
        world_routine_snapshot: &WorldRoutineSnapshotV1,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        self.commit_world_checkpoint_inner_with_streaming(
            compatibility,
            checkpoint,
            Some(world_streaming_snapshot),
            Some(world_routine_snapshot),
            None,
        )
    }

    pub fn prepare_world_checkpoint_with_streaming(
        &self,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
        world_streaming_snapshot: &WorldStreamingSnapshotV1,
    ) -> Result<SaveImage, SaveStoreError> {
        let next_generation = self
            .probe_candidates()
            .iter()
            .map(|candidate| candidate.manifest.generation)
            .max()
            .map_or(Ok(0), |generation| {
                generation
                    .checked_add(1)
                    .ok_or(SaveStoreError::GenerationExhausted)
            })?;
        SaveImage::from_world_checkpoint_with_streaming(
            next_generation,
            compatibility,
            checkpoint,
            world_streaming_snapshot,
        )
    }

    pub fn prepare_world_checkpoint_with_streaming_and_routine(
        &self,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
        world_streaming_snapshot: &WorldStreamingSnapshotV1,
        world_routine_snapshot: &WorldRoutineSnapshotV1,
    ) -> Result<SaveImage, SaveStoreError> {
        let next_generation = self
            .probe_candidates()
            .iter()
            .map(|candidate| candidate.manifest.generation)
            .max()
            .map_or(Ok(0), |generation| {
                generation
                    .checked_add(1)
                    .ok_or(SaveStoreError::GenerationExhausted)
            })?;
        SaveImage::from_world_checkpoint_with_streaming_and_routine(
            next_generation,
            compatibility,
            checkpoint,
            world_streaming_snapshot,
            world_routine_snapshot,
        )
    }

    /// Publishes an immutable image prepared by the close journal. Repeating
    /// the call after a crash is idempotent when the exact generation already
    /// occupies its canonical slot.
    pub fn commit_prepared_image(
        &self,
        image: &SaveImage,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        image.validate_world_light()?;
        self.commit_image_inner(image, None)
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

    #[cfg(test)]
    pub(super) fn commit_runtime_snapshot_inner(
        &self,
        compatibility: SaveCompatibility,
        snapshot: &RuntimeSnapshotV3,
        fault: Option<CommitBoundary>,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        let checkpoint = synthetic_empty_checkpoint(snapshot.clone(), RpgSnapshotV2::default())?;
        self.commit_world_checkpoint_inner(compatibility, &checkpoint, fault)
    }

    fn commit_world_checkpoint_inner(
        &self,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
        fault: Option<CommitBoundary>,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        self.commit_world_checkpoint_inner_with_streaming(
            compatibility,
            checkpoint,
            None,
            None,
            fault,
        )
    }

    fn commit_world_checkpoint_inner_with_streaming(
        &self,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
        world_streaming_snapshot: Option<&WorldStreamingSnapshotV1>,
        world_routine_snapshot: Option<&WorldRoutineSnapshotV1>,
        fault: Option<CommitBoundary>,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        fs::create_dir_all(&self.root)
            .map_err(|source| SaveStoreError::io("create save root", &self.root, source))?;
        // Generation numbering only needs the highest valid generation, not
        // the decoded world; probe instead of full validation.
        let candidates = self.probe_candidates();
        let next_generation = candidates
            .iter()
            .map(|candidate| candidate.manifest.generation)
            .max()
            .map_or(Ok(0), |generation| {
                generation
                    .checked_add(1)
                    .ok_or(SaveStoreError::GenerationExhausted)
            })?;
        let image = match (world_streaming_snapshot, world_routine_snapshot) {
            (Some(streaming), Some(routine)) => {
                SaveImage::from_world_checkpoint_with_streaming_and_routine(
                    next_generation,
                    compatibility,
                    checkpoint,
                    streaming,
                    routine,
                )?
            }
            (Some(snapshot), None) => SaveImage::from_world_checkpoint_with_streaming(
                next_generation,
                compatibility,
                checkpoint,
                snapshot,
            )?,
            (None, None) => {
                SaveImage::from_world_checkpoint(next_generation, compatibility, checkpoint)?
            }
            (None, Some(_)) => {
                return Err(SaveStoreError::InvalidImage(
                    "SAVE_WORLD_ROUTINE_STREAMING_MISSING",
                ));
            }
        };
        self.commit_image_inner(&image, fault)
    }

    fn commit_image_inner(
        &self,
        image: &SaveImage,
        fault: Option<CommitBoundary>,
    ) -> Result<SaveCommitReceipt, SaveStoreError> {
        fs::create_dir_all(&self.root)
            .map_err(|source| SaveStoreError::io("create save root", &self.root, source))?;
        image.validate_world_light()?;
        let next_generation = image.manifest.generation;
        let slot = u8::try_from(next_generation % SLOT_COUNT)
            .map_err(|_| SaveStoreError::GenerationExhausted)?;
        let slot_path = self.slot_path(slot);
        if slot_path.exists()
            && probe_generation_directory(&slot_path, slot).is_ok_and(|existing| existing == *image)
        {
            self.publish_pointer(next_generation, slot, fault)?;
            return Ok(SaveCommitReceipt {
                generation: next_generation,
                slot,
            });
        }
        let highest = self
            .probe_candidates()
            .into_iter()
            .map(|candidate| candidate.manifest.generation)
            .max();
        let expected = highest.map_or(0, |generation| generation.saturating_add(1));
        if next_generation != expected {
            return Err(SaveStoreError::InvalidImage("SAVE_GENERATION_NOT_NEXT"));
        }
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

        let staged_image = probe_generation_directory(&staging, slot)
            .map_err(|rejected| SaveStoreError::InvalidStaging(rejected.stable_code))?;
        if staged_image != *image {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_STAGING_ROUND_TRIP_MISMATCH",
            ));
        }
        maybe_inject(fault, CommitBoundary::StagingValidated)?;

        remove_directory_if_present(&slot_path)?;
        maybe_inject(fault, CommitBoundary::InactiveSlotRemoved)?;
        fs::rename(&staging, &slot_path)
            .map_err(|source| SaveStoreError::io("publish save generation", &slot_path, source))?;
        sync_directory(&self.root)?;
        maybe_inject(fault, CommitBoundary::GenerationPublished)?;

        self.publish_pointer(next_generation, slot, fault)?;

        Ok(SaveCommitReceipt {
            generation: next_generation,
            slot,
        })
    }

    fn publish_pointer(
        &self,
        generation: u64,
        slot: u8,
        fault: Option<CommitBoundary>,
    ) -> Result<(), SaveStoreError> {
        let pointer_staging = self.root.join(CURRENT_STAGING_FILE);
        remove_file_if_present(&pointer_staging)?;
        write_new_synced(
            &pointer_staging,
            format!("{generation} {slot}\n").as_bytes(),
        )?;
        maybe_inject(fault, CommitBoundary::PointerStaged)?;
        remove_file_if_present(&self.root.join(CURRENT_FILE))?;
        maybe_inject(fault, CommitBoundary::PriorPointerRemoved)?;
        fs::rename(&pointer_staging, self.root.join(CURRENT_FILE)).map_err(|source| {
            SaveStoreError::io("publish save pointer", self.root.join(CURRENT_FILE), source)
        })?;
        sync_directory(&self.root)?;
        maybe_inject(fault, CommitBoundary::PointerPublished)?;
        Ok(())
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

    /// Returns the valid on-disk generation images without decoding their
    /// worlds. Rejected generations are ignored exactly as the commit path
    /// always ignored them when computing the next generation number.
    fn probe_candidates(&self) -> Vec<SaveImage> {
        let mut candidates = Vec::new();
        for slot in 0..2_u8 {
            let path = self.slot_path(slot);
            match fs::symlink_metadata(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Ok(metadata) if metadata.file_type().is_dir() => {}
                _ => continue,
            }
            if let Ok(image) = probe_generation_directory(&path, slot) {
                candidates.push(image);
            }
        }
        candidates
    }

    pub(super) fn slot_path(&self, slot: u8) -> PathBuf {
        self.root.join(format!("slot-{slot}"))
    }

    fn staging_path(&self, slot: u8) -> PathBuf {
        self.root.join(format!("slot-{slot}.staging"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CommitBoundary {
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
