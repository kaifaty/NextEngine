use std::fs;
use std::path::{Path, PathBuf};

use next_assets::SaveStore;
use next_contracts::{
    PHYSICS_SNAPSHOT_OWNER_ID, PersistentId, RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
    RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1, SaveCompatibility,
    SaveSegmentDescriptor,
};

use super::PersistenceReplayCheckError;

pub(super) fn corrupt_rpg_segment(
    store: &SaveStore,
    compatibility: &SaveCompatibility,
    slot: u8,
) -> Result<(PathBuf, Vec<u8>), PersistenceReplayCheckError> {
    let latest = store.load_latest(compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("inspect newest RPG generation", error.to_string())
    })?;
    let segment_index = latest
        .image
        .manifest
        .segments
        .iter()
        .position(|descriptor| descriptor.owner_id.as_str() == RPG_AGGREGATE_SNAPSHOT_OWNER_ID)
        .ok_or_else(|| PersistenceReplayCheckError::condition("newest RPG segment exists"))?;
    let mut corrupt_snapshot = latest.checkpoint.rpg_snapshot;
    let dialogue = corrupt_snapshot
        .aggregates
        .iter_mut()
        .find(|aggregate| aggregate.aggregate_kind == RpgAggregateKindV1::Dialogue)
        .ok_or_else(|| PersistenceReplayCheckError::condition("newest core dialogue exists"))?;
    let RpgAggregatePayloadV1::Dialogue(payload) = &dialogue.payload else {
        return Err(PersistenceReplayCheckError::condition(
            "newest core dialogue payload matches its kind",
        ));
    };
    let replacement = RpgAggregateEnvelopeV1::new(
        dialogue.persistent_id,
        dialogue.schema_version,
        dialogue.revision,
        dialogue.definition_ref.clone(),
        dialogue.provenance.clone(),
        RpgAggregatePayloadV1::Dialogue(next_contracts::DialoguePayloadV1 {
            speaker_id: payload.speaker_id,
            listener_id: PersistentId::from_bytes([0xee; 16]),
            node_id: payload.node_id.clone(),
        }),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("construct structural RPG corruption", error.to_string())
    })?;
    *dialogue = replacement;
    let bytes = corrupt_snapshot.canonical_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode structural RPG corruption", error.to_string())
    })?;
    let segment_path = segment_path(store.root(), slot, segment_index);
    fs::write(&segment_path, &bytes).map_err(|error| {
        PersistenceReplayCheckError::new("corrupt RPG segment", error.to_string())
    })?;
    Ok((segment_path, bytes))
}

pub(super) fn corrupt_physics_segment(
    store: &SaveStore,
    compatibility: &SaveCompatibility,
    slot: u8,
) -> Result<(PathBuf, Vec<u8>), PersistenceReplayCheckError> {
    let latest = store.load_latest(compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("inspect newest generation", error.to_string())
    })?;
    let segment_index = latest
        .image
        .manifest
        .segments
        .iter()
        .position(|descriptor| descriptor.owner_id.as_str() == PHYSICS_SNAPSHOT_OWNER_ID)
        .ok_or_else(|| PersistenceReplayCheckError::condition("newest physics segment exists"))?;
    let mut corrupt_checkpoint = latest.checkpoint.physics_checkpoint;
    let static_body_id = corrupt_checkpoint
        .catalog
        .bodies
        .iter()
        .find_map(|(body_id, descriptor)| {
            (descriptor.motion_kind == next_contracts::PhysicsMotionKindV1::Static)
                .then_some(*body_id)
        })
        .ok_or_else(|| {
            PersistenceReplayCheckError::condition("newest physics catalog has a static body")
        })?;
    corrupt_checkpoint
        .snapshot
        .sorted_body_states
        .get_mut(&static_body_id)
        .ok_or_else(|| {
            PersistenceReplayCheckError::condition("newest physics snapshot has static state")
        })?
        .linear_velocity_micrometres_per_second[0] = 1;
    let bytes = corrupt_checkpoint.canonical_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode structural physics corruption", error.to_string())
    })?;
    let segment_path = segment_path(store.root(), slot, segment_index);
    fs::write(&segment_path, &bytes).map_err(|error| {
        PersistenceReplayCheckError::new("corrupt physics segment", error.to_string())
    })?;
    let mut manifest = latest.image.manifest;
    let descriptor = manifest.segments[segment_index].clone();
    manifest.segments[segment_index] = SaveSegmentDescriptor::for_bytes(
        descriptor.owner_id,
        descriptor.schema_id,
        descriptor.segment_id,
        descriptor.schema_version,
        &bytes,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("bind structural physics corruption", error.to_string())
    })?;
    let manifest_path = store
        .root()
        .join(format!("slot-{slot}"))
        .join("manifest.jcs");
    fs::write(
        &manifest_path,
        manifest.to_jcs_bytes().map_err(|error| {
            PersistenceReplayCheckError::new(
                "encode corrupt generation manifest",
                error.to_string(),
            )
        })?,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("write corrupt generation manifest", error.to_string())
    })?;
    Ok((segment_path, bytes))
}

fn segment_path(root: &Path, slot: u8, segment_index: usize) -> PathBuf {
    root.join(format!("slot-{slot}"))
        .join("segments")
        .join(format!("{segment_index:08}.necb"))
}
