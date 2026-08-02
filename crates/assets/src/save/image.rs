use next_contracts::canonical::{CanonicalDecodeLimits, CanonicalError};
use next_contracts::ids::SchemaId;
use next_contracts::persistence::{
    CommandLedgerDescriptorV2, SaveCompatibility, SaveManifestV2, SaveSegmentDescriptor,
};
use next_contracts::physics::{
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
    PhysicsWorldCheckpointV1,
};
use next_contracts::rpg::{
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION, RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID, RpgSnapshotV2,
};
use next_contracts::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID,
    RuntimeSnapshotV3, WorldCheckpointV4,
};
use next_contracts::world::{
    WORLD_STREAMING_SNAPSHOT_OWNER_ID, WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
    WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION, WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
    WorldStreamingSnapshotV1,
};

use super::error::SaveStoreError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveImage {
    pub manifest: SaveManifestV2,
    pub segments: Vec<Vec<u8>>,
}

impl SaveImage {
    /// Builds a save image from an already-constructed checkpoint.
    ///
    /// `WorldCheckpointV4::new` performs full component validation, encoding
    /// and closure checks at construction, so a `&WorldCheckpointV4` reference
    /// is proof of a valid checkpoint; this constructor must not repeat that
    /// work. On-disk validity is re-probed after staging before publication
    /// (see `SaveStore` commit), so nothing here is the last line of defense.
    pub fn from_world_checkpoint(
        generation: u64,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
    ) -> Result<Self, SaveStoreError> {
        let runtime_snapshot = &checkpoint.runtime_snapshot;
        let rpg_snapshot = &checkpoint.rpg_snapshot;
        let physics_checkpoint = &checkpoint.physics_checkpoint;
        let runtime_bytes = runtime_snapshot.canonical_bytes()?;
        let rpg_bytes = rpg_snapshot.canonical_bytes()?;
        let physics_bytes = physics_checkpoint.canonical_bytes()?;
        let mut manifest = SaveManifestV2::for_runtime_snapshot(
            generation,
            compatibility,
            runtime_snapshot,
            &runtime_bytes,
        )?;
        let runtime_descriptor = manifest.segments.pop().ok_or(SaveStoreError::InvalidImage(
            "SAVE_RUNTIME_SNAPSHOT_MISSING",
        ))?;
        let rpg_descriptor = SaveSegmentDescriptor::for_bytes(
            SchemaId::new(RPG_AGGREGATE_SNAPSHOT_OWNER_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
            &rpg_bytes,
        )?;
        let physics_descriptor = SaveSegmentDescriptor::for_bytes(
            SchemaId::new(PHYSICS_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION),
            &physics_bytes,
        )?;
        let mut segments = vec![
            (runtime_descriptor, runtime_bytes),
            (rpg_descriptor, rpg_bytes),
            (physics_descriptor, physics_bytes),
        ];
        segments.sort_by(|left, right| {
            (&left.0.owner_id, &left.0.schema_id, &left.0.segment_id).cmp(&(
                &right.0.owner_id,
                &right.0.schema_id,
                &right.0.segment_id,
            ))
        });
        manifest.segments = segments
            .iter()
            .map(|(descriptor, _)| descriptor.clone())
            .collect();
        manifest.validate()?;
        Ok(Self {
            manifest,
            segments: segments.into_iter().map(|(_, bytes)| bytes).collect(),
        })
    }

    pub fn from_world_checkpoint_with_streaming(
        generation: u64,
        compatibility: SaveCompatibility,
        checkpoint: &WorldCheckpointV4,
        world_streaming_snapshot: &WorldStreamingSnapshotV1,
    ) -> Result<Self, SaveStoreError> {
        world_streaming_snapshot.validate()?;
        let mut image = Self::from_world_checkpoint(generation, compatibility, checkpoint)?;
        let bytes = world_streaming_snapshot.canonical_bytes()?;
        let descriptor = SaveSegmentDescriptor::for_bytes(
            SchemaId::new(WORLD_STREAMING_SNAPSHOT_OWNER_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(WORLD_STREAMING_SNAPSHOT_SCHEMA_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(WORLD_STREAMING_SNAPSHOT_SEGMENT_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION,
            &bytes,
        )?;
        let mut segments = image
            .manifest
            .segments
            .into_iter()
            .zip(image.segments)
            .collect::<Vec<_>>();
        segments.push((descriptor, bytes));
        segments.sort_by(|left, right| {
            (&left.0.owner_id, &left.0.schema_id, &left.0.segment_id).cmp(&(
                &right.0.owner_id,
                &right.0.schema_id,
                &right.0.segment_id,
            ))
        });
        image.manifest.segments = segments
            .iter()
            .map(|(descriptor, _)| descriptor.clone())
            .collect();
        image.segments = segments.into_iter().map(|(_, bytes)| bytes).collect();
        image.manifest.validate()?;
        // The streaming snapshot was validated above and its canonical bytes
        // are embedded verbatim with a matching descriptor; the staged
        // generation probe re-verifies bytes before publication.
        Ok(image)
    }

    pub fn validate_world(&self) -> Result<ValidatedSaveImage, SaveStoreError> {
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
        if self.manifest.segments[runtime_index].schema_version
            != next_contracts::snapshot::RUNTIME_SNAPSHOT_SCHEMA_VERSION
        {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_RUNTIME_SNAPSHOT_VERSION_MISMATCH",
            ));
        }
        let runtime_snapshot = RuntimeSnapshotV3::from_canonical_bytes(
            &self.segments[runtime_index],
            CanonicalDecodeLimits::default(),
        )?;
        if runtime_snapshot.authoritative_revision != self.manifest.world_revision {
            return Err(SaveStoreError::InvalidImage("SAVE_WORLD_REVISION_MISMATCH"));
        }
        let expected_ledger = CommandLedgerDescriptorV2 {
            world_namespace: runtime_snapshot.world_identity.world_namespace,
            stream_count: u64::try_from(runtime_snapshot.command_ledger.streams.len())
                .map_err(|_| SaveStoreError::InvalidImage("SAVE_STREAM_COUNT_OVERFLOW"))?,
            archive_root: runtime_snapshot.command_ledger.body_archive.archive_root,
            identity_index_root: runtime_snapshot.command_ledger.identity_index.index_root,
            runtime_snapshot_segment_hash: self.manifest.segments[runtime_index].content_hash,
        };
        if expected_ledger != self.manifest.command_ledger {
            return Err(SaveStoreError::InvalidImage("SAVE_COMMAND_LEDGER_MISMATCH"));
        }
        let rpg_index = self.manifest.segments.iter().position(|segment| {
            segment.owner_id.as_str() == RPG_AGGREGATE_SNAPSHOT_OWNER_ID
                && segment.schema_id.as_str() == RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID
                && segment.segment_id.as_str() == RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID
        });
        let rpg_index =
            rpg_index.ok_or(SaveStoreError::InvalidImage("SAVE_RPG_SNAPSHOT_MISSING"))?;
        if self.manifest.segments[rpg_index].schema_version != RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION
        {
            return Err(SaveStoreError::InvalidImage("RPG_SCHEMA_UNSUPPORTED"));
        }
        let rpg_snapshot = RpgSnapshotV2::from_canonical_bytes(
            &self.segments[rpg_index],
            CanonicalDecodeLimits::default(),
        )?;
        let physics_index = self
            .manifest
            .segments
            .iter()
            .position(|segment| {
                segment.owner_id.as_str() == PHYSICS_SNAPSHOT_OWNER_ID
                    && segment.schema_id.as_str() == PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID
                    && segment.segment_id.as_str() == PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID
            })
            .ok_or(SaveStoreError::InvalidImage(
                "SAVE_PHYSICS_SNAPSHOT_MISSING",
            ))?;
        if self.manifest.segments[physics_index].schema_version
            != u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION)
        {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_PHYSICS_SNAPSHOT_VERSION_MISMATCH",
            ));
        }
        let physics_checkpoint = PhysicsWorldCheckpointV1::from_canonical_bytes(
            &self.segments[physics_index],
            CanonicalDecodeLimits::default(),
        )?;
        let checkpoint = WorldCheckpointV4::new(
            runtime_snapshot.clone(),
            rpg_snapshot.clone(),
            physics_checkpoint.clone(),
        )?;
        let world_streaming_indices = self
            .manifest
            .segments
            .iter()
            .enumerate()
            .filter(|(_, segment)| segment.owner_id.as_str() == WORLD_STREAMING_SNAPSHOT_OWNER_ID)
            .collect::<Vec<_>>();
        if world_streaming_indices.len() > 1 {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_WORLD_STREAMING_SEGMENT_DUPLICATE",
            ));
        }
        let world_streaming_snapshot =
            if let Some((world_index, descriptor)) = world_streaming_indices.first().copied() {
                if descriptor.schema_id.as_str() != WORLD_STREAMING_SNAPSHOT_SCHEMA_ID
                    || descriptor.segment_id.as_str() != WORLD_STREAMING_SNAPSHOT_SEGMENT_ID
                    || descriptor.schema_version != WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION
                {
                    return Err(SaveStoreError::InvalidImage(
                        "WORLD_STREAM_SCHEMA_UNSUPPORTED",
                    ));
                }
                Some(WorldStreamingSnapshotV1::from_canonical_bytes(
                    &self.segments[world_index],
                    CanonicalDecodeLimits::default(),
                )?)
            } else {
                None
            };
        let tick = &self.manifest.compatibility.tick_settings;
        if tick.gameplay_hz != runtime_snapshot.tick_rate_profile.gameplay_hz
            || tick.physics_hz != runtime_snapshot.tick_rate_profile.physics_hz()
            || tick.motor_hz
                != tick.physics_hz
                    / runtime_snapshot
                        .tick_rate_profile
                        .motor_period_physics_substeps
        {
            return Err(SaveStoreError::InvalidImage("SAVE_TICK_PROFILE_MISMATCH"));
        }
        Ok(ValidatedSaveImage {
            checkpoint,
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            world_streaming_snapshot,
        })
    }

    /// Probes on-disk validity without rebuilding validated values.
    ///
    /// Performs exactly the same accept/reject checks as `validate_world` —
    /// manifest, segment hashes, full canonical decodes (which validate every
    /// component), descriptor/revision/tick closures and the cross-component
    /// checkpoint closures — but skips checkpoint reconstruction, component
    /// re-encoding and state-root recomputation. The state root is not stored
    /// in the image, so recomputing it cannot change the verdict.
    pub fn validate_world_light(&self) -> Result<(), SaveStoreError> {
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
        if self.manifest.segments[runtime_index].schema_version
            != next_contracts::snapshot::RUNTIME_SNAPSHOT_SCHEMA_VERSION
        {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_RUNTIME_SNAPSHOT_VERSION_MISMATCH",
            ));
        }
        let runtime_snapshot = RuntimeSnapshotV3::from_canonical_bytes(
            &self.segments[runtime_index],
            CanonicalDecodeLimits::default(),
        )?;
        if runtime_snapshot.authoritative_revision != self.manifest.world_revision {
            return Err(SaveStoreError::InvalidImage("SAVE_WORLD_REVISION_MISMATCH"));
        }
        let expected_ledger = CommandLedgerDescriptorV2 {
            world_namespace: runtime_snapshot.world_identity.world_namespace,
            stream_count: u64::try_from(runtime_snapshot.command_ledger.streams.len())
                .map_err(|_| SaveStoreError::InvalidImage("SAVE_STREAM_COUNT_OVERFLOW"))?,
            archive_root: runtime_snapshot.command_ledger.body_archive.archive_root,
            identity_index_root: runtime_snapshot.command_ledger.identity_index.index_root,
            runtime_snapshot_segment_hash: self.manifest.segments[runtime_index].content_hash,
        };
        if expected_ledger != self.manifest.command_ledger {
            return Err(SaveStoreError::InvalidImage("SAVE_COMMAND_LEDGER_MISMATCH"));
        }
        let rpg_index = self.manifest.segments.iter().position(|segment| {
            segment.owner_id.as_str() == RPG_AGGREGATE_SNAPSHOT_OWNER_ID
                && segment.schema_id.as_str() == RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID
                && segment.segment_id.as_str() == RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID
        });
        let rpg_index =
            rpg_index.ok_or(SaveStoreError::InvalidImage("SAVE_RPG_SNAPSHOT_MISSING"))?;
        if self.manifest.segments[rpg_index].schema_version != RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION
        {
            return Err(SaveStoreError::InvalidImage("RPG_SCHEMA_UNSUPPORTED"));
        }
        let rpg_snapshot = RpgSnapshotV2::from_canonical_bytes(
            &self.segments[rpg_index],
            CanonicalDecodeLimits::default(),
        )?;
        let physics_index = self
            .manifest
            .segments
            .iter()
            .position(|segment| {
                segment.owner_id.as_str() == PHYSICS_SNAPSHOT_OWNER_ID
                    && segment.schema_id.as_str() == PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID
                    && segment.segment_id.as_str() == PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID
            })
            .ok_or(SaveStoreError::InvalidImage(
                "SAVE_PHYSICS_SNAPSHOT_MISSING",
            ))?;
        if self.manifest.segments[physics_index].schema_version
            != u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION)
        {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_PHYSICS_SNAPSHOT_VERSION_MISMATCH",
            ));
        }
        let physics_checkpoint = PhysicsWorldCheckpointV1::from_canonical_bytes(
            &self.segments[physics_index],
            CanonicalDecodeLimits::default(),
        )?;
        next_contracts::snapshot::validate_world_checkpoint_component_closures(
            &runtime_snapshot,
            &rpg_snapshot,
            &physics_checkpoint,
        )?;
        let world_streaming_indices = self
            .manifest
            .segments
            .iter()
            .enumerate()
            .filter(|(_, segment)| segment.owner_id.as_str() == WORLD_STREAMING_SNAPSHOT_OWNER_ID)
            .collect::<Vec<_>>();
        if world_streaming_indices.len() > 1 {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_WORLD_STREAMING_SEGMENT_DUPLICATE",
            ));
        }
        if let Some((world_index, descriptor)) = world_streaming_indices.first().copied() {
            if descriptor.schema_id.as_str() != WORLD_STREAMING_SNAPSHOT_SCHEMA_ID
                || descriptor.segment_id.as_str() != WORLD_STREAMING_SNAPSHOT_SEGMENT_ID
                || descriptor.schema_version != WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION
            {
                return Err(SaveStoreError::InvalidImage(
                    "WORLD_STREAM_SCHEMA_UNSUPPORTED",
                ));
            }
            let _ = WorldStreamingSnapshotV1::from_canonical_bytes(
                &self.segments[world_index],
                CanonicalDecodeLimits::default(),
            )?;
        }
        let tick = &self.manifest.compatibility.tick_settings;
        if tick.gameplay_hz != runtime_snapshot.tick_rate_profile.gameplay_hz
            || tick.physics_hz != runtime_snapshot.tick_rate_profile.physics_hz()
            || tick.motor_hz
                != tick.physics_hz
                    / runtime_snapshot
                        .tick_rate_profile
                        .motor_period_physics_substeps
        {
            return Err(SaveStoreError::InvalidImage("SAVE_TICK_PROFILE_MISMATCH"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedSaveImage {
    pub checkpoint: WorldCheckpointV4,
    pub runtime_snapshot: RuntimeSnapshotV3,
    pub rpg_snapshot: RpgSnapshotV2,
    pub physics_checkpoint: PhysicsWorldCheckpointV1,
    pub world_streaming_snapshot: Option<WorldStreamingSnapshotV1>,
}
