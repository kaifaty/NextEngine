use std::collections::{BTreeMap, BTreeSet};

use next_assets::SessionObjectV1;
use next_contracts::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I32, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
};
use next_contracts::ids::{
    ApplicationSessionId, CommandLedgerHash, ContentHash, command_ledger_hash_from_bytes,
};
use next_contracts::physics::PhysicsWorldCheckpointV1;
use next_contracts::project::ActivatedProjectV2;
use next_contracts::rpg::RpgSnapshotV2;
use next_contracts::session::PresentationTargetKindV1;
use next_contracts::snapshot::{RuntimeSnapshotV3, WorldCheckpointV4};
use next_contracts::world::WorldStreamingSnapshotV1;
use next_player::PlayerInputSessionV1;
use next_reference_game::{
    ReferenceGameDriverV1, ReferenceLiveDriverRecoveryV1, ReferenceLiveStateV1,
};

use crate::ApplicationError;

use super::{PreparedRunV1, prepare_live_state};

const LIVE_RECOVERY_SCHEMA_VERSION: u32 = 1;
const LIVE_RECOVERY_OWNER_ID: &str = "nextengine.application";
const LIVE_RECOVERY_SCHEMA_ID: &str = "nextengine.application-live-run-recovery.v1";
const LIVE_RECOVERY_FIELD_COUNT: usize = 24;

#[derive(Clone, Debug, Eq, PartialEq)]
struct LiveRunPayloadHashesV1 {
    runtime_snapshot: ContentHash,
    rpg_snapshot: ContentHash,
    physics_checkpoint: ContentHash,
    world_streaming_snapshot: ContentHash,
    presentation_snapshot: ContentHash,
    input_session: ContentHash,
}

impl LiveRunPayloadHashesV1 {
    fn ordered(&self) -> [ContentHash; 6] {
        [
            self.runtime_snapshot,
            self.rpg_snapshot,
            self.physics_checkpoint,
            self.world_streaming_snapshot,
            self.presentation_snapshot,
            self.input_session,
        ]
    }

    fn validate(&self) -> Result<(), ApplicationError> {
        let unique = self.ordered().into_iter().collect::<BTreeSet<_>>();
        if unique.len() != self.ordered().len() {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LiveRunRecoveryManifestV1 {
    session_id: ApplicationSessionId,
    project_composition_lock_hash: ContentHash,
    payloads: LiveRunPayloadHashesV1,
    next_logical_frame_sequence: u64,
    events: u64,
    rpg_events: u64,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    camera_cut: bool,
    presentation_input_count: u64,
    authoritative_state_root: ContentHash,
    command_archive_root: ContentHash,
    command_identity_index_root: ContentHash,
    command_ledger_hash: CommandLedgerHash,
    authoritative_revision: u64,
    presentation_snapshot_hash: ContentHash,
    ticks: u64,
    content_manifest_hash: ContentHash,
}

impl LiveRunRecoveryManifestV1 {
    fn validate(&self) -> Result<(), ApplicationError> {
        self.payloads.validate()?;
        if self.next_logical_frame_sequence != self.ticks
            || self.rpg_events > self.events
            || self.camera_cut
            || self.presentation_input_count == 0
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        Ok(())
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, ApplicationError> {
        self.validate()?;
        Ok(encode_canonical_segment(
            LIVE_RECOVERY_OWNER_ID,
            LIVE_RECOVERY_SCHEMA_ID,
            &self.session_id.to_hex(),
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    LIVE_RECOVERY_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_ID128, self.session_id.as_bytes().to_vec()),
                hash_field(3, self.project_composition_lock_hash),
                hash_field(4, self.payloads.runtime_snapshot),
                hash_field(5, self.payloads.rpg_snapshot),
                hash_field(6, self.payloads.physics_checkpoint),
                hash_field(7, self.payloads.world_streaming_snapshot),
                hash_field(8, self.payloads.presentation_snapshot),
                u64_field(9, self.next_logical_frame_sequence),
                u64_field(10, self.events),
                u64_field(11, self.rpg_events),
                CanonicalField::new(
                    12,
                    CANONICAL_TYPE_I32,
                    self.camera_yaw_millidegrees.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    13,
                    CANONICAL_TYPE_I32,
                    self.camera_pitch_millidegrees.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(14, CANONICAL_TYPE_BOOL, vec![u8::from(self.camera_cut)]),
                u64_field(15, self.presentation_input_count),
                hash_field(16, self.authoritative_state_root),
                hash_field(17, self.command_archive_root),
                hash_field(18, self.command_identity_index_root),
                CanonicalField::new(
                    19,
                    CANONICAL_TYPE_HASH256,
                    self.command_ledger_hash.as_bytes().to_vec(),
                ),
                u64_field(20, self.authoritative_revision),
                hash_field(21, self.presentation_snapshot_hash),
                u64_field(22, self.ticks),
                hash_field(23, self.content_manifest_hash),
                hash_field(24, self.payloads.input_session),
            ],
        )?)
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ApplicationError> {
        let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())
            .map_err(|_| ApplicationError::RecoveryIncompatible)?;
        ensure_segment(&decoded)?;
        if decode_u32(field(&decoded, 1, CANONICAL_TYPE_U32)?)? != LIVE_RECOVERY_SCHEMA_VERSION {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        let value = Self {
            session_id: decode_session_id(field(&decoded, 2, CANONICAL_TYPE_ID128)?)?,
            project_composition_lock_hash: decode_hash(field(
                &decoded,
                3,
                CANONICAL_TYPE_HASH256,
            )?)?,
            payloads: LiveRunPayloadHashesV1 {
                runtime_snapshot: decode_hash(field(&decoded, 4, CANONICAL_TYPE_HASH256)?)?,
                rpg_snapshot: decode_hash(field(&decoded, 5, CANONICAL_TYPE_HASH256)?)?,
                physics_checkpoint: decode_hash(field(&decoded, 6, CANONICAL_TYPE_HASH256)?)?,
                world_streaming_snapshot: decode_hash(field(&decoded, 7, CANONICAL_TYPE_HASH256)?)?,
                presentation_snapshot: decode_hash(field(&decoded, 8, CANONICAL_TYPE_HASH256)?)?,
                input_session: decode_hash(field(&decoded, 24, CANONICAL_TYPE_HASH256)?)?,
            },
            next_logical_frame_sequence: decode_u64(field(&decoded, 9, CANONICAL_TYPE_U64)?)?,
            events: decode_u64(field(&decoded, 10, CANONICAL_TYPE_U64)?)?,
            rpg_events: decode_u64(field(&decoded, 11, CANONICAL_TYPE_U64)?)?,
            camera_yaw_millidegrees: decode_i32(field(&decoded, 12, CANONICAL_TYPE_I32)?)?,
            camera_pitch_millidegrees: decode_i32(field(&decoded, 13, CANONICAL_TYPE_I32)?)?,
            camera_cut: decode_bool(field(&decoded, 14, CANONICAL_TYPE_BOOL)?)?,
            presentation_input_count: decode_u64(field(&decoded, 15, CANONICAL_TYPE_U64)?)?,
            authoritative_state_root: decode_hash(field(&decoded, 16, CANONICAL_TYPE_HASH256)?)?,
            command_archive_root: decode_hash(field(&decoded, 17, CANONICAL_TYPE_HASH256)?)?,
            command_identity_index_root: decode_hash(field(&decoded, 18, CANONICAL_TYPE_HASH256)?)?,
            command_ledger_hash: decode_ledger_hash(field(&decoded, 19, CANONICAL_TYPE_HASH256)?)?,
            authoritative_revision: decode_u64(field(&decoded, 20, CANONICAL_TYPE_U64)?)?,
            presentation_snapshot_hash: decode_hash(field(&decoded, 21, CANONICAL_TYPE_HASH256)?)?,
            ticks: decode_u64(field(&decoded, 22, CANONICAL_TYPE_U64)?)?,
            content_manifest_hash: decode_hash(field(&decoded, 23, CANONICAL_TYPE_HASH256)?)?,
        };
        if decoded.segment_id != value.session_id.to_hex() {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        Ok(value)
    }
}

pub(super) struct LiveRunObjectClosureV1 {
    pub(super) manifest_hash: ContentHash,
    pub(super) objects: BTreeMap<ContentHash, Vec<u8>>,
}

pub(crate) fn validate_live_run_evidence_closure(
    objects: &BTreeMap<ContentHash, Vec<u8>>,
    manifest_hash: ContentHash,
    session_id: ApplicationSessionId,
    project_composition_lock_hash: ContentHash,
    content_manifest_hash: ContentHash,
    expected_authoritative_revision: u64,
    presentation_target: PresentationTargetKindV1,
) -> Result<(), ApplicationError> {
    validate_persisted_live_run_closure(
        objects,
        manifest_hash,
        session_id,
        project_composition_lock_hash,
        content_manifest_hash,
        expected_authoritative_revision,
        presentation_target,
    )?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn live_run_evidence_payload_hashes(
    objects: &BTreeMap<ContentHash, Vec<u8>>,
    manifest_hash: ContentHash,
) -> Result<[ContentHash; 6], ApplicationError> {
    let manifest_bytes = required_object(objects, manifest_hash)?;
    Ok(
        LiveRunRecoveryManifestV1::from_canonical_bytes(manifest_bytes)?
            .payloads
            .ordered(),
    )
}

#[cfg(test)]
pub(crate) fn replace_live_run_evidence_payload(
    objects: &mut BTreeMap<ContentHash, Vec<u8>>,
    manifest_hash: ContentHash,
    payload_index: usize,
    replacement_bytes: Vec<u8>,
) -> Result<ContentHash, ApplicationError> {
    let manifest_bytes = required_object(objects, manifest_hash)?;
    let mut manifest = LiveRunRecoveryManifestV1::from_canonical_bytes(manifest_bytes)?;
    let replacement = SessionObjectV1::new(replacement_bytes);
    match payload_index {
        0 => manifest.payloads.runtime_snapshot = replacement.content_hash,
        1 => manifest.payloads.rpg_snapshot = replacement.content_hash,
        2 => manifest.payloads.physics_checkpoint = replacement.content_hash,
        3 => manifest.payloads.world_streaming_snapshot = replacement.content_hash,
        4 => manifest.payloads.presentation_snapshot = replacement.content_hash,
        5 => manifest.payloads.input_session = replacement.content_hash,
        _ => return Err(ApplicationError::RecoveryIncompatible),
    }
    let replacement_manifest = SessionObjectV1::new(manifest.canonical_bytes()?);
    objects.remove(&manifest_hash);
    objects.insert(replacement.content_hash, replacement.bytes);
    objects.insert(
        replacement_manifest.content_hash,
        replacement_manifest.bytes,
    );
    Ok(replacement_manifest.content_hash)
}

pub(crate) struct RecoveredLiveRunV1 {
    pub(crate) lifecycle_objects: BTreeMap<ContentHash, Vec<u8>>,
    pub(crate) prepared_run_objects: BTreeMap<ContentHash, Vec<u8>>,
    pub(crate) live_run_recovery_manifest_hash: ContentHash,
    pub(crate) prepared_run: PreparedRunV1,
    pub(crate) live_run: ReferenceGameDriverV1,
}

pub(super) fn live_run_object_closure(
    session_id: ApplicationSessionId,
    prepared: &PreparedRunV1,
) -> Result<LiveRunObjectClosureV1, ApplicationError> {
    let recovery = prepared
        .driver_recovery
        .as_ref()
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    let runtime = SessionObjectV1::new(
        prepared
            .checkpoint
            .runtime_snapshot
            .canonical_bytes()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    );
    let rpg = SessionObjectV1::new(
        prepared
            .checkpoint
            .rpg_snapshot
            .canonical_bytes()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    );
    let physics = SessionObjectV1::new(
        prepared
            .checkpoint
            .physics_checkpoint
            .canonical_bytes()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    );
    let streaming = SessionObjectV1::new(
        prepared
            .streaming
            .canonical_bytes()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    );
    let presentation = SessionObjectV1::new(recovery.presentation_snapshot_bytes.clone());
    let input = SessionObjectV1::new(recovery.input_session_bytes.clone());
    let payloads = LiveRunPayloadHashesV1 {
        runtime_snapshot: runtime.content_hash,
        rpg_snapshot: rpg.content_hash,
        physics_checkpoint: physics.content_hash,
        world_streaming_snapshot: streaming.content_hash,
        presentation_snapshot: presentation.content_hash,
        input_session: input.content_hash,
    };
    payloads.validate()?;

    let extractor = next_presentation::PresentationExtractorV1::resume_from_recovery_bytes(
        &recovery.presentation_snapshot_bytes,
    )
    .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    let presentation_snapshot = extractor
        .accepted_snapshot()
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    if prepared
        .summary
        .presentation_snapshot
        .as_ref()
        .is_some_and(|published| published != presentation_snapshot)
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    let manifest = LiveRunRecoveryManifestV1 {
        session_id,
        project_composition_lock_hash: prepared.summary.project_composition_lock_hash,
        payloads,
        next_logical_frame_sequence: recovery.next_logical_frame_sequence,
        events: recovery.events,
        rpg_events: recovery.rpg_events,
        camera_yaw_millidegrees: recovery.camera_yaw_millidegrees,
        camera_pitch_millidegrees: recovery.camera_pitch_millidegrees,
        camera_cut: recovery.camera_cut,
        presentation_input_count: prepared.summary.presentation_input_count,
        authoritative_state_root: prepared.summary.authoritative_state_root,
        command_archive_root: prepared.summary.command_archive_root,
        command_identity_index_root: prepared.summary.command_identity_index_root,
        command_ledger_hash: prepared.summary.command_ledger_hash,
        authoritative_revision: prepared.summary.authoritative_revision,
        presentation_snapshot_hash: presentation_snapshot.canonical_hash,
        ticks: prepared.summary.ticks,
        content_manifest_hash: presentation_snapshot.content_manifest_hash,
    };
    let manifest = SessionObjectV1::new(manifest.canonical_bytes()?);
    if [
        runtime.content_hash,
        rpg.content_hash,
        physics.content_hash,
        streaming.content_hash,
        presentation.content_hash,
        input.content_hash,
    ]
    .contains(&manifest.content_hash)
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    let manifest_hash = manifest.content_hash;
    let objects = [
        runtime,
        rpg,
        physics,
        streaming,
        presentation,
        input,
        manifest,
    ]
    .into_iter()
    .map(|object| (object.content_hash, object.bytes))
    .collect();
    Ok(LiveRunObjectClosureV1 {
        manifest_hash,
        objects,
    })
}

struct ValidatedPersistedLiveRunV1 {
    checkpoint: WorldCheckpointV4,
    streaming: WorldStreamingSnapshotV1,
    recovery: ReferenceLiveDriverRecoveryV1,
    closure: LiveRunObjectClosureV1,
}

#[allow(
    clippy::too_many_arguments,
    reason = "historical evidence must bind the exact session, project, content, revision and target"
)]
fn validate_persisted_live_run_closure(
    published_objects: &BTreeMap<ContentHash, Vec<u8>>,
    manifest_hash: ContentHash,
    session_id: ApplicationSessionId,
    project_composition_lock_hash: ContentHash,
    content_manifest_hash: ContentHash,
    expected_authoritative_revision: u64,
    presentation_target: PresentationTargetKindV1,
) -> Result<ValidatedPersistedLiveRunV1, ApplicationError> {
    let manifest_bytes = required_object(published_objects, manifest_hash)?;
    let manifest = LiveRunRecoveryManifestV1::from_canonical_bytes(manifest_bytes)?;
    if manifest.session_id != session_id
        || manifest.project_composition_lock_hash != project_composition_lock_hash
        || manifest.content_manifest_hash != content_manifest_hash
        || manifest.authoritative_revision != expected_authoritative_revision
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }

    let runtime_bytes = required_object(published_objects, manifest.payloads.runtime_snapshot)?;
    let rpg_bytes = required_object(published_objects, manifest.payloads.rpg_snapshot)?;
    let physics_bytes = required_object(published_objects, manifest.payloads.physics_checkpoint)?;
    let streaming_bytes = required_object(
        published_objects,
        manifest.payloads.world_streaming_snapshot,
    )?;
    let presentation_bytes =
        required_object(published_objects, manifest.payloads.presentation_snapshot)?;
    let input_bytes = required_object(published_objects, manifest.payloads.input_session)?;

    let limits = CanonicalDecodeLimits::default();
    let checkpoint = WorldCheckpointV4::new(
        RuntimeSnapshotV3::from_canonical_bytes(runtime_bytes, limits)
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
        RpgSnapshotV2::from_canonical_bytes(rpg_bytes, limits)
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
        PhysicsWorldCheckpointV1::from_canonical_bytes(physics_bytes, limits)
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    )
    .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    let streaming = WorldStreamingSnapshotV1::from_canonical_bytes(streaming_bytes, limits)
        .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    let input_session = PlayerInputSessionV1::restore_from_recovery_bytes(input_bytes)
        .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    if input_session
        .recovery_bytes()
        .map_err(|_| ApplicationError::RecoveryIncompatible)?
        != input_bytes
        || input_session.last_logical_frame_sequence()
            != manifest.next_logical_frame_sequence.checked_sub(1)
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    let recovery = ReferenceLiveDriverRecoveryV1 {
        next_logical_frame_sequence: manifest.next_logical_frame_sequence,
        events: manifest.events,
        rpg_events: manifest.rpg_events,
        camera_yaw_millidegrees: manifest.camera_yaw_millidegrees,
        camera_pitch_millidegrees: manifest.camera_pitch_millidegrees,
        camera_cut: manifest.camera_cut,
        input_session_bytes: input_bytes.to_vec(),
        presentation_snapshot_bytes: presentation_bytes.to_vec(),
    };

    // First validate the exact persisted closure as evidence. Authoritative
    // presentation recovery deliberately creates a fresh epoch, so comparing a
    // post-restore driver closure with this persisted manifest would conflate
    // the old publication with the new recovery cut.
    let persisted_presentation_extractor =
        next_presentation::PresentationExtractorV1::resume_from_recovery_bytes(presentation_bytes)
            .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    let persisted_presentation_snapshot = persisted_presentation_extractor
        .accepted_snapshot()
        .cloned()
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    if checkpoint.runtime_snapshot.next_tick != manifest.next_logical_frame_sequence
        || checkpoint.runtime_snapshot.committed_event_count != manifest.events
        || persisted_presentation_snapshot.simulation_tick != checkpoint.runtime_snapshot.next_tick
        || persisted_presentation_snapshot.project_composition_lock_hash
            != project_composition_lock_hash
        || persisted_presentation_snapshot.content_manifest_hash != content_manifest_hash
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    let presentation_input_count = persisted_presentation_snapshot
        .scene_records()
        .count()
        .checked_add(persisted_presentation_snapshot.camera_records().count())
        .and_then(|count| u64::try_from(count).ok())
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    let persisted_prepared_run = prepare_live_state(
        session_id,
        ReferenceLiveStateV1 {
            checkpoint: checkpoint.clone(),
            world_streaming_snapshot: streaming.clone(),
            ticks: recovery.next_logical_frame_sequence,
            events: recovery.events,
            rpg_events: recovery.rpg_events,
            project_composition_lock_hash,
            content_manifest_hash,
            presentation_input_count,
            presentation_snapshot: persisted_presentation_snapshot,
            driver_recovery: recovery.clone(),
        },
        presentation_target,
    )?;
    let persisted_closure = live_run_object_closure(session_id, &persisted_prepared_run)?;
    if persisted_closure.manifest_hash != manifest_hash {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    for (hash, expected_bytes) in &persisted_closure.objects {
        if published_objects.get(hash) != Some(expected_bytes) {
            return Err(ApplicationError::RecoveryIncompatible);
        }
    }
    Ok(ValidatedPersistedLiveRunV1 {
        checkpoint,
        streaming,
        recovery,
        closure: persisted_closure,
    })
}

pub(crate) fn restore_live_run(
    mut published_objects: BTreeMap<ContentHash, Vec<u8>>,
    manifest_hash: ContentHash,
    session_id: ApplicationSessionId,
    activated_project: ActivatedProjectV2,
    expected_authoritative_revision: u64,
    presentation_target: PresentationTargetKindV1,
) -> Result<RecoveredLiveRunV1, ApplicationError> {
    let persisted = validate_persisted_live_run_closure(
        &published_objects,
        manifest_hash,
        session_id,
        activated_project.composition_lock.composition_lock_sha256,
        activated_project.content_manifest.content_manifest_sha256,
        expected_authoritative_revision,
        presentation_target,
    )?;

    for hash in persisted.closure.objects.keys() {
        published_objects
            .remove(hash)
            .ok_or(ApplicationError::RecoveryIncompatible)?;
    }

    let mut live_run = ReferenceGameDriverV1::restore(
        activated_project,
        persisted.checkpoint,
        persisted.streaming,
        persisted.recovery,
    )
    .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    live_run
        .cancel_recovered_controls()
        .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    let prepared_run = prepare_live_state(session_id, live_run.state()?, presentation_target)?;
    let recovered_closure = live_run_object_closure(session_id, &prepared_run)?;
    Ok(RecoveredLiveRunV1 {
        lifecycle_objects: published_objects,
        prepared_run_objects: recovered_closure.objects,
        live_run_recovery_manifest_hash: recovered_closure.manifest_hash,
        prepared_run,
        live_run,
    })
}

fn required_object(
    objects: &BTreeMap<ContentHash, Vec<u8>>,
    hash: ContentHash,
) -> Result<&[u8], ApplicationError> {
    let bytes = objects
        .get(&hash)
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    if SessionObjectV1::new(bytes.clone()).content_hash != hash {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    Ok(bytes)
}

fn hash_field(field_id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(field_id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

fn u64_field(field_id: u32, value: u64) -> CanonicalField {
    CanonicalField::new(field_id, CANONICAL_TYPE_U64, value.to_le_bytes().to_vec())
}

fn ensure_segment(decoded: &DecodedCanonicalSegment) -> Result<(), ApplicationError> {
    if decoded.owner_id != LIVE_RECOVERY_OWNER_ID
        || decoded.schema_id != LIVE_RECOVERY_SCHEMA_ID
        || decoded.fields.len() != LIVE_RECOVERY_FIELD_COUNT
        || decoded
            .fields
            .iter()
            .enumerate()
            .any(|(index, field)| field.field_id != u32::try_from(index + 1).unwrap_or(u32::MAX))
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    Ok(())
}

fn field(
    decoded: &DecodedCanonicalSegment,
    field_id: u32,
    type_tag: u8,
) -> Result<&[u8], ApplicationError> {
    let field = decoded
        .field(field_id)
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    if field.type_tag != type_tag {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    Ok(&field.payload)
}

fn decode_u32(bytes: &[u8]) -> Result<u32, ApplicationError> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    ))
}

fn decode_u64(bytes: &[u8]) -> Result<u64, ApplicationError> {
    Ok(u64::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    ))
}

fn decode_i32(bytes: &[u8]) -> Result<i32, ApplicationError> {
    Ok(i32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    ))
}

fn decode_bool(bytes: &[u8]) -> Result<bool, ApplicationError> {
    match bytes {
        [0] => Ok(false),
        [1] => Ok(true),
        _ => Err(ApplicationError::RecoveryIncompatible),
    }
}

fn decode_hash(bytes: &[u8]) -> Result<ContentHash, ApplicationError> {
    Ok(ContentHash::from_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    ))
}

fn decode_ledger_hash(bytes: &[u8]) -> Result<CommandLedgerHash, ApplicationError> {
    Ok(command_ledger_hash_from_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    ))
}

fn decode_session_id(bytes: &[u8]) -> Result<ApplicationSessionId, ApplicationError> {
    Ok(ApplicationSessionId::from_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::RecoveryIncompatible)?,
    ))
}
