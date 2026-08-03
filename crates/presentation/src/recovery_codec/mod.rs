use next_contracts::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
};
use next_contracts::ids::{AssetId, ContentHash, PersistentId, SchemaId};
use next_contracts::presentation::{
    CAMERA_PRESENTATION_RECORD_SCHEMA_VERSION, CameraInterpolationPolicyV1,
    CameraPresentationRecordV2, CameraProjectionProfileV1, CameraResultSampleV1, CameraRoleV1,
    CameraViewportV1, PRESENTATION_MAX_CAMERA_RECORDS, PRESENTATION_MAX_SCENE_RECORDS,
    PRESENTATION_MAX_SEMANTIC_UI_RECORDS, PRESENTATION_SCENE_RECORD_SCHEMA_VERSION,
    PresentationObjectKeyV1, PresentationRoleV1, PresentationSnapshotV2,
    QuantizedPresentationTransformV1, SEMANTIC_UI_PRESENTATION_RECORD_SCHEMA_VERSION,
    ScenePresentationFlagsV1, ScenePresentationRecordV2, SemanticUiPresentationRecordV1,
    ThirdPersonCameraIntentSampleV1, UI_MAX_AFFORDANCES_PER_ELEMENT, UI_MAX_TEXT_ARGUMENTS,
    UiAccessibilityRoleV1, UiActionAffordanceV1, UiElementRoleV1, UiElementValueV1,
    UiSemanticElementV1, UiStyleRoleV1, UiTextArgumentV1, UiTextRefV1,
};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::render_content::AabbI64V1;

const RECOVERY_CODEC_SCHEMA_VERSION: u32 = 2;
const RECOVERY_OWNER: &str = "nextengine.presentation";
const RECOVERY_SCHEMA: &str = "nextengine.presentation-snapshot-recovery.v2";
const SCENE_RECORD_SCHEMA: &str = "nextengine.scene-presentation-recovery-record.v1";
const CAMERA_RECORD_SCHEMA: &str = "nextengine.camera-presentation-recovery-record.v1";
const SEMANTIC_UI_RECORD_SCHEMA: &str = "nextengine.semantic-ui-presentation-recovery-record.v1";
const UI_TEXT_REF_SCHEMA: &str = "nextengine.ui-text-ref-recovery.v1";

pub(super) fn encode_snapshot(
    snapshot: &PresentationSnapshotV2,
    max_scene_records_per_batch: usize,
    max_camera_records_per_batch: usize,
    max_semantic_ui_records_per_batch: usize,
) -> Result<Vec<u8>, ()> {
    snapshot.validate().map_err(|_| ())?;
    validate_batch_profile(
        snapshot,
        max_scene_records_per_batch,
        max_camera_records_per_batch,
        max_semantic_ui_records_per_batch,
    )?;
    let scene_records = snapshot
        .scene_records()
        .enumerate()
        .map(|(index, record)| encode_scene_record(index, record))
        .collect::<Result<Vec<_>, _>>()?;
    let camera_records = snapshot
        .camera_records()
        .enumerate()
        .map(|(index, record)| encode_camera_record(index, record))
        .collect::<Result<Vec<_>, _>>()?;
    let semantic_ui_records = snapshot
        .semantic_ui_records()
        .enumerate()
        .map(|(index, record)| encode_semantic_ui_record(index, record))
        .collect::<Result<Vec<_>, _>>()?;
    encode_canonical_segment(
        RECOVERY_OWNER,
        RECOVERY_SCHEMA,
        "snapshot",
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                RECOVERY_CODEC_SCHEMA_VERSION.to_le_bytes().to_vec(),
            ),
            hash_field(2, snapshot.snapshot_epoch),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_U64,
                snapshot.snapshot_sequence.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_U64,
                snapshot.simulation_tick.to_le_bytes().to_vec(),
            ),
            hash_field(5, snapshot.project_composition_lock_hash),
            hash_field(6, snapshot.content_manifest_hash),
            hash_field(7, snapshot.presentation_profile_hash),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_U32,
                u32::try_from(max_scene_records_per_batch)
                    .map_err(|_| ())?
                    .to_le_bytes()
                    .to_vec(),
            ),
            CanonicalField::new(
                9,
                CANONICAL_TYPE_U32,
                u32::try_from(max_camera_records_per_batch)
                    .map_err(|_| ())?
                    .to_le_bytes()
                    .to_vec(),
            ),
            CanonicalField::new(10, CANONICAL_TYPE_SEQUENCE, encode_sequence(scene_records)?),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(camera_records)?,
            ),
            CanonicalField::new(
                12,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(semantic_ui_records)?,
            ),
            CanonicalField::new(
                13,
                CANONICAL_TYPE_SEQUENCE,
                encode_hash_sequence(&snapshot.cue_batches)?,
            ),
            hash_field(14, snapshot.environment_batch),
            hash_field(15, snapshot.canonical_hash),
            CanonicalField::new(
                16,
                CANONICAL_TYPE_U32,
                u32::try_from(max_semantic_ui_records_per_batch)
                    .map_err(|_| ())?
                    .to_le_bytes()
                    .to_vec(),
            ),
        ],
    )
    .map_err(|_| ())
}

pub(super) fn decode_snapshot(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<(PresentationSnapshotV2, usize, usize, usize), ()> {
    let segment = decode_canonical_segment(bytes, limits).map_err(|_| ())?;
    ensure_segment(&segment, RECOVERY_OWNER, RECOVERY_SCHEMA, "snapshot", 16)?;
    if decode_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)? != RECOVERY_CODEC_SCHEMA_VERSION {
        return Err(());
    }
    let snapshot_epoch = decode_hash(field(&segment, 2, CANONICAL_TYPE_HASH256)?)?;
    let snapshot_sequence = decode_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?;
    let simulation_tick = decode_u64(field(&segment, 4, CANONICAL_TYPE_U64)?)?;
    let project_composition_lock_hash = decode_hash(field(&segment, 5, CANONICAL_TYPE_HASH256)?)?;
    let content_manifest_hash = decode_hash(field(&segment, 6, CANONICAL_TYPE_HASH256)?)?;
    let presentation_profile_hash = decode_hash(field(&segment, 7, CANONICAL_TYPE_HASH256)?)?;
    let max_scene_records_per_batch =
        usize::try_from(decode_u32(field(&segment, 8, CANONICAL_TYPE_U32)?)?).map_err(|_| ())?;
    let max_camera_records_per_batch =
        usize::try_from(decode_u32(field(&segment, 9, CANONICAL_TYPE_U32)?)?).map_err(|_| ())?;
    let max_semantic_ui_records_per_batch =
        usize::try_from(decode_u32(field(&segment, 16, CANONICAL_TYPE_U32)?)?).map_err(|_| ())?;
    if max_scene_records_per_batch == 0
        || max_scene_records_per_batch > PRESENTATION_MAX_SCENE_RECORDS
        || max_camera_records_per_batch == 0
        || max_camera_records_per_batch > PRESENTATION_MAX_CAMERA_RECORDS
        || max_semantic_ui_records_per_batch == 0
        || max_semantic_ui_records_per_batch > PRESENTATION_MAX_SEMANTIC_UI_RECORDS
    {
        return Err(());
    }
    let scene_records = decode_sequence(
        field(&segment, 10, CANONICAL_TYPE_SEQUENCE)?,
        PRESENTATION_MAX_SCENE_RECORDS,
        limits.max_field_payload_bytes,
    )?
    .into_iter()
    .enumerate()
    .map(|(index, bytes)| decode_scene_record(index, bytes, limits))
    .collect::<Result<Vec<_>, _>>()?;
    let camera_records = decode_sequence(
        field(&segment, 11, CANONICAL_TYPE_SEQUENCE)?,
        PRESENTATION_MAX_CAMERA_RECORDS,
        limits.max_field_payload_bytes,
    )?
    .into_iter()
    .enumerate()
    .map(|(index, bytes)| decode_camera_record(index, bytes, limits))
    .collect::<Result<Vec<_>, _>>()?;
    let semantic_ui_records = decode_sequence(
        field(&segment, 12, CANONICAL_TYPE_SEQUENCE)?,
        PRESENTATION_MAX_SEMANTIC_UI_RECORDS,
        limits.max_field_payload_bytes,
    )?
    .into_iter()
    .enumerate()
    .map(|(index, bytes)| decode_semantic_ui_record(index, bytes, limits))
    .collect::<Result<Vec<_>, _>>()?;
    let cue_batches = decode_hash_sequence(
        field(&segment, 13, CANONICAL_TYPE_SEQUENCE)?,
        limits.max_sequence_items,
        limits.max_field_payload_bytes,
    )?;
    let environment_batch = decode_hash(field(&segment, 14, CANONICAL_TYPE_HASH256)?)?;
    let canonical_hash = decode_hash(field(&segment, 15, CANONICAL_TYPE_HASH256)?)?;
    let mut snapshot = PresentationSnapshotV2::new_with_camera_and_semantic_ui_records(
        snapshot_epoch,
        snapshot_sequence,
        simulation_tick,
        project_composition_lock_hash,
        content_manifest_hash,
        presentation_profile_hash,
        scene_records,
        camera_records,
        semantic_ui_records,
        max_scene_records_per_batch,
        max_camera_records_per_batch,
        max_semantic_ui_records_per_batch,
        environment_batch,
    )
    .map_err(|_| ())?;
    snapshot.cue_batches = cue_batches;
    snapshot.canonical_hash = canonical_hash;
    snapshot.validate().map_err(|_| ())?;
    if encode_snapshot(
        &snapshot,
        max_scene_records_per_batch,
        max_camera_records_per_batch,
        max_semantic_ui_records_per_batch,
    )? != bytes
    {
        return Err(());
    }
    Ok((
        snapshot,
        max_scene_records_per_batch,
        max_camera_records_per_batch,
        max_semantic_ui_records_per_batch,
    ))
}

fn validate_batch_profile(
    snapshot: &PresentationSnapshotV2,
    max_scene_records_per_batch: usize,
    max_camera_records_per_batch: usize,
    max_semantic_ui_records_per_batch: usize,
) -> Result<(), ()> {
    if max_scene_records_per_batch == 0
        || max_scene_records_per_batch > PRESENTATION_MAX_SCENE_RECORDS
        || max_camera_records_per_batch == 0
        || max_camera_records_per_batch > PRESENTATION_MAX_CAMERA_RECORDS
        || max_semantic_ui_records_per_batch == 0
        || max_semantic_ui_records_per_batch > PRESENTATION_MAX_SEMANTIC_UI_RECORDS
    {
        return Err(());
    }
    let mut rebuilt = PresentationSnapshotV2::new_with_camera_and_semantic_ui_records(
        snapshot.snapshot_epoch,
        snapshot.snapshot_sequence,
        snapshot.simulation_tick,
        snapshot.project_composition_lock_hash,
        snapshot.content_manifest_hash,
        snapshot.presentation_profile_hash,
        snapshot.scene_records().cloned().collect(),
        snapshot.camera_records().cloned().collect(),
        snapshot.semantic_ui_records().cloned().collect(),
        max_scene_records_per_batch,
        max_camera_records_per_batch,
        max_semantic_ui_records_per_batch,
        snapshot.environment_batch,
    )
    .map_err(|_| ())?;
    rebuilt.cue_batches = snapshot.cue_batches.clone();
    rebuilt.canonical_hash = snapshot.canonical_hash;
    rebuilt.validate().map_err(|_| ())?;
    if &rebuilt != snapshot {
        return Err(());
    }
    Ok(())
}

fn encode_scene_record(index: usize, record: &ScenePresentationRecordV2) -> Result<Vec<u8>, ()> {
    record.validate().map_err(|_| ())?;
    encode_canonical_segment(
        RECOVERY_OWNER,
        SCENE_RECORD_SCHEMA,
        &format!("scene-{index}"),
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                record.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_U16,
                record.presentation_layer.to_le_bytes().to_vec(),
            ),
            hash_field(3, record.object_key.snapshot_epoch),
            id_field(4, record.object_key.persistent_id.as_bytes()),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U8,
                vec![record.object_key.presentation_role as u8],
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_U32,
                record.object_key.incarnation.to_le_bytes().to_vec(),
            ),
            id_field(7, record.mesh_revision.asset_id.as_bytes()),
            hash_field(8, record.mesh_revision.record_sha256),
            id_field(9, record.material_revision.asset_id.as_bytes()),
            hash_field(10, record.material_revision.record_sha256),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_U32,
                record.instance_ordinal.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(12, CANONICAL_TYPE_BYTES, encode_bounds(record.local_bounds)),
            CanonicalField::new(
                13,
                CANONICAL_TYPE_U32,
                record.feature_flags.bits().to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                14,
                CANONICAL_TYPE_BYTES,
                encode_transform(record.previous_transform),
            ),
            CanonicalField::new(
                15,
                CANONICAL_TYPE_BYTES,
                encode_transform(record.current_transform),
            ),
            CanonicalField::new(16, CANONICAL_TYPE_BOOL, vec![u8::from(record.visible)]),
            hash_field(17, record.canonical_hash),
        ],
    )
    .map_err(|_| ())
}

fn decode_scene_record(
    index: usize,
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<ScenePresentationRecordV2, ()> {
    let segment = decode_canonical_segment(bytes, limits).map_err(|_| ())?;
    ensure_segment(
        &segment,
        RECOVERY_OWNER,
        SCENE_RECORD_SCHEMA,
        &format!("scene-{index}"),
        17,
    )?;
    let record = ScenePresentationRecordV2 {
        schema_version: decode_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?,
        presentation_layer: decode_u16(field(&segment, 2, CANONICAL_TYPE_U16)?)?,
        object_key: PresentationObjectKeyV1 {
            snapshot_epoch: decode_hash(field(&segment, 3, CANONICAL_TYPE_HASH256)?)?,
            persistent_id: PersistentId::from_bytes(decode_id(field(
                &segment,
                4,
                CANONICAL_TYPE_ID128,
            )?)?),
            presentation_role: decode_presentation_role(decode_u8(field(
                &segment,
                5,
                CANONICAL_TYPE_U8,
            )?)?)?,
            incarnation: decode_u32(field(&segment, 6, CANONICAL_TYPE_U32)?)?,
        },
        mesh_revision: AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes(decode_id(field(&segment, 7, CANONICAL_TYPE_ID128)?)?),
            record_sha256: decode_hash(field(&segment, 8, CANONICAL_TYPE_HASH256)?)?,
        },
        material_revision: AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes(decode_id(field(&segment, 9, CANONICAL_TYPE_ID128)?)?),
            record_sha256: decode_hash(field(&segment, 10, CANONICAL_TYPE_HASH256)?)?,
        },
        instance_ordinal: decode_u32(field(&segment, 11, CANONICAL_TYPE_U32)?)?,
        local_bounds: decode_bounds(field(&segment, 12, CANONICAL_TYPE_BYTES)?)?,
        feature_flags: ScenePresentationFlagsV1::from_bits(decode_u32(field(
            &segment,
            13,
            CANONICAL_TYPE_U32,
        )?)?)
        .map_err(|_| ())?,
        previous_transform: decode_transform(field(&segment, 14, CANONICAL_TYPE_BYTES)?)?,
        current_transform: decode_transform(field(&segment, 15, CANONICAL_TYPE_BYTES)?)?,
        visible: decode_bool(field(&segment, 16, CANONICAL_TYPE_BOOL)?)?,
        canonical_hash: decode_hash(field(&segment, 17, CANONICAL_TYPE_HASH256)?)?,
    };
    if record.schema_version != PRESENTATION_SCENE_RECORD_SCHEMA_VERSION {
        return Err(());
    }
    record.validate().map_err(|_| ())?;
    if encode_scene_record(index, &record)? != bytes {
        return Err(());
    }
    Ok(record)
}

fn encode_camera_record(index: usize, record: &CameraPresentationRecordV2) -> Result<Vec<u8>, ()> {
    record.validate().map_err(|_| ())?;
    encode_canonical_segment(
        RECOVERY_OWNER,
        CAMERA_RECORD_SCHEMA,
        &format!("camera-{index}"),
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                record.schema_version.to_le_bytes().to_vec(),
            ),
            hash_field(2, record.snapshot_epoch),
            id_field(3, record.camera_id.as_bytes()),
            CanonicalField::new(4, CANONICAL_TYPE_U8, vec![record.camera_role as u8]),
            CanonicalField::new(5, CANONICAL_TYPE_BYTES, encode_viewport(record.viewport)),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_BYTES,
                encode_projection(record.projection_profile),
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_OPTIONAL,
                record
                    .intent_sample
                    .focus_subject_id
                    .map_or_else(Vec::new, |id| id.as_bytes().to_vec()),
            ),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_BYTES,
                encode_i64_array(record.intent_sample.focus_point_micrometres),
            ),
            CanonicalField::new(
                9,
                CANONICAL_TYPE_BYTES,
                record
                    .intent_sample
                    .orbit_yaw_millidegrees
                    .to_le_bytes()
                    .to_vec(),
            ),
            CanonicalField::new(
                10,
                CANONICAL_TYPE_BYTES,
                record
                    .intent_sample
                    .orbit_pitch_millidegrees
                    .to_le_bytes()
                    .to_vec(),
            ),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_U64,
                record
                    .intent_sample
                    .distance_micrometres
                    .to_le_bytes()
                    .to_vec(),
            ),
            CanonicalField::new(
                12,
                CANONICAL_TYPE_BYTES,
                encode_i64_array(record.intent_sample.shoulder_offset_micrometres),
            ),
            CanonicalField::new(
                13,
                CANONICAL_TYPE_BYTES,
                encode_transform(record.previous_result_sample.pose),
            ),
            CanonicalField::new(
                14,
                CANONICAL_TYPE_BYTES,
                encode_i64_array(record.previous_result_sample.focus_point_micrometres),
            ),
            CanonicalField::new(
                15,
                CANONICAL_TYPE_BYTES,
                encode_transform(record.current_result_sample.pose),
            ),
            CanonicalField::new(
                16,
                CANONICAL_TYPE_BYTES,
                encode_i64_array(record.current_result_sample.focus_point_micrometres),
            ),
            id_field(17, record.exposure_profile_revision.asset_id.as_bytes()),
            hash_field(18, record.exposure_profile_revision.record_sha256),
            CanonicalField::new(19, CANONICAL_TYPE_BOOL, vec![u8::from(record.cut)]),
            CanonicalField::new(
                20,
                CANONICAL_TYPE_U8,
                vec![record.interpolation_policy as u8],
            ),
            hash_field(21, record.canonical_hash),
        ],
    )
    .map_err(|_| ())
}

fn decode_camera_record(
    index: usize,
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<CameraPresentationRecordV2, ()> {
    let segment = decode_canonical_segment(bytes, limits).map_err(|_| ())?;
    ensure_segment(
        &segment,
        RECOVERY_OWNER,
        CAMERA_RECORD_SCHEMA,
        &format!("camera-{index}"),
        21,
    )?;
    let record = CameraPresentationRecordV2 {
        schema_version: decode_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?,
        snapshot_epoch: decode_hash(field(&segment, 2, CANONICAL_TYPE_HASH256)?)?,
        camera_id: PersistentId::from_bytes(decode_id(field(&segment, 3, CANONICAL_TYPE_ID128)?)?),
        camera_role: decode_camera_role(decode_u8(field(&segment, 4, CANONICAL_TYPE_U8)?)?)?,
        viewport: decode_viewport(field(&segment, 5, CANONICAL_TYPE_BYTES)?)?,
        projection_profile: decode_projection(field(&segment, 6, CANONICAL_TYPE_BYTES)?)?,
        intent_sample: ThirdPersonCameraIntentSampleV1 {
            focus_subject_id: decode_optional_id(field(&segment, 7, CANONICAL_TYPE_OPTIONAL)?)?
                .map(PersistentId::from_bytes),
            focus_point_micrometres: decode_i64_array(field(&segment, 8, CANONICAL_TYPE_BYTES)?)?,
            orbit_yaw_millidegrees: decode_i32(field(&segment, 9, CANONICAL_TYPE_BYTES)?)?,
            orbit_pitch_millidegrees: decode_i32(field(&segment, 10, CANONICAL_TYPE_BYTES)?)?,
            distance_micrometres: decode_u64(field(&segment, 11, CANONICAL_TYPE_U64)?)?,
            shoulder_offset_micrometres: decode_i64_array(field(
                &segment,
                12,
                CANONICAL_TYPE_BYTES,
            )?)?,
        },
        previous_result_sample: CameraResultSampleV1 {
            pose: decode_transform(field(&segment, 13, CANONICAL_TYPE_BYTES)?)?,
            focus_point_micrometres: decode_i64_array(field(&segment, 14, CANONICAL_TYPE_BYTES)?)?,
        },
        current_result_sample: CameraResultSampleV1 {
            pose: decode_transform(field(&segment, 15, CANONICAL_TYPE_BYTES)?)?,
            focus_point_micrometres: decode_i64_array(field(&segment, 16, CANONICAL_TYPE_BYTES)?)?,
        },
        exposure_profile_revision: AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes(decode_id(field(&segment, 17, CANONICAL_TYPE_ID128)?)?),
            record_sha256: decode_hash(field(&segment, 18, CANONICAL_TYPE_HASH256)?)?,
        },
        cut: decode_bool(field(&segment, 19, CANONICAL_TYPE_BOOL)?)?,
        interpolation_policy: decode_interpolation_policy(decode_u8(field(
            &segment,
            20,
            CANONICAL_TYPE_U8,
        )?)?)?,
        canonical_hash: decode_hash(field(&segment, 21, CANONICAL_TYPE_HASH256)?)?,
    };
    if record.schema_version != CAMERA_PRESENTATION_RECORD_SCHEMA_VERSION {
        return Err(());
    }
    record.validate().map_err(|_| ())?;
    if encode_camera_record(index, &record)? != bytes {
        return Err(());
    }
    Ok(record)
}

mod semantic_ui;
use semantic_ui::{decode_semantic_ui_record, encode_semantic_ui_record};

fn encode_sequence(items: Vec<Vec<u8>>) -> Result<Vec<u8>, ()> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&u32::try_from(items.len()).map_err(|_| ())?.to_le_bytes());
    for item in items {
        bytes.extend_from_slice(&u32::try_from(item.len()).map_err(|_| ())?.to_le_bytes());
        bytes.extend_from_slice(&item);
    }
    Ok(bytes)
}

fn decode_sequence(
    bytes: &[u8],
    max_items: usize,
    max_item_bytes: usize,
) -> Result<Vec<&[u8]>, ()> {
    let mut cursor = RecoveryCursor::new(bytes);
    let count = usize::try_from(cursor.read_u32()?).map_err(|_| ())?;
    if count > max_items {
        return Err(());
    }
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let length = usize::try_from(cursor.read_u32()?).map_err(|_| ())?;
        if length > max_item_bytes {
            return Err(());
        }
        items.push(cursor.read_exact(length)?);
    }
    cursor.finish()?;
    Ok(items)
}

fn encode_hash_sequence(values: &[ContentHash]) -> Result<Vec<u8>, ()> {
    encode_sequence(values.iter().map(|hash| hash.as_bytes().to_vec()).collect())
}

fn decode_hash_sequence(
    bytes: &[u8],
    max_items: usize,
    max_item_bytes: usize,
) -> Result<Vec<ContentHash>, ()> {
    decode_sequence(bytes, max_items, max_item_bytes)?
        .into_iter()
        .map(decode_hash)
        .collect()
}

fn hash_field(field_id: u32, hash: ContentHash) -> CanonicalField {
    CanonicalField::new(field_id, CANONICAL_TYPE_HASH256, hash.as_bytes().to_vec())
}

fn id_field(field_id: u32, bytes: &[u8; 16]) -> CanonicalField {
    CanonicalField::new(field_id, CANONICAL_TYPE_ID128, bytes.to_vec())
}

fn ensure_segment(
    segment: &DecodedCanonicalSegment,
    owner: &str,
    schema: &str,
    id: &str,
    field_count: usize,
) -> Result<(), ()> {
    if segment.owner_id != owner
        || segment.schema_id != schema
        || segment.segment_id != id
        || segment.fields.len() != field_count
    {
        return Err(());
    }
    Ok(())
}

fn field(segment: &DecodedCanonicalSegment, id: u32, tag: u8) -> Result<&[u8], ()> {
    let field = segment.field(id).ok_or(())?;
    if field.type_tag != tag {
        return Err(());
    }
    Ok(&field.payload)
}

fn decode_u8(bytes: &[u8]) -> Result<u8, ()> {
    bytes
        .first()
        .copied()
        .filter(|_| bytes.len() == 1)
        .ok_or(())
}

fn decode_bool(bytes: &[u8]) -> Result<bool, ()> {
    match decode_u8(bytes)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(()),
    }
}

fn decode_u16(bytes: &[u8]) -> Result<u16, ()> {
    Ok(u16::from_le_bytes(bytes.try_into().map_err(|_| ())?))
}

fn decode_u32(bytes: &[u8]) -> Result<u32, ()> {
    Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| ())?))
}

fn decode_i32(bytes: &[u8]) -> Result<i32, ()> {
    Ok(i32::from_le_bytes(bytes.try_into().map_err(|_| ())?))
}

fn decode_u64(bytes: &[u8]) -> Result<u64, ()> {
    Ok(u64::from_le_bytes(bytes.try_into().map_err(|_| ())?))
}

fn decode_hash(bytes: &[u8]) -> Result<ContentHash, ()> {
    Ok(ContentHash::from_bytes(bytes.try_into().map_err(|_| ())?))
}

fn decode_id(bytes: &[u8]) -> Result<[u8; 16], ()> {
    bytes.try_into().map_err(|_| ())
}

fn decode_optional_id(bytes: &[u8]) -> Result<Option<[u8; 16]>, ()> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode_id(bytes).map(Some)
    }
}

fn encode_bounds(bounds: AabbI64V1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(48);
    for value in bounds.min().into_iter().chain(bounds.max()) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_bounds(bytes: &[u8]) -> Result<AabbI64V1, ()> {
    let mut cursor = RecoveryCursor::new(bytes);
    let mut values = [0_i64; 6];
    for value in &mut values {
        *value = cursor.read_i64()?;
    }
    cursor.finish()?;
    AabbI64V1::new(
        [values[0], values[1], values[2]],
        [values[3], values[4], values[5]],
    )
    .map_err(|_| ())
}

fn encode_transform(transform: QuantizedPresentationTransformV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(40);
    for value in transform.translation_micrometres {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in transform.orientation_q30 {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_transform(bytes: &[u8]) -> Result<QuantizedPresentationTransformV1, ()> {
    let mut cursor = RecoveryCursor::new(bytes);
    let mut translation_micrometres = [0_i64; 3];
    for value in &mut translation_micrometres {
        *value = cursor.read_i64()?;
    }
    let mut orientation_q30 = [0_i32; 4];
    for value in &mut orientation_q30 {
        *value = cursor.read_i32()?;
    }
    cursor.finish()?;
    Ok(QuantizedPresentationTransformV1 {
        translation_micrometres,
        orientation_q30,
    })
}

fn encode_i64_array(values: [i64; 3]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(24);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_i64_array(bytes: &[u8]) -> Result<[i64; 3], ()> {
    let mut cursor = RecoveryCursor::new(bytes);
    let values = [cursor.read_i64()?, cursor.read_i64()?, cursor.read_i64()?];
    cursor.finish()?;
    Ok(values)
}

fn encode_viewport(viewport: CameraViewportV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(10);
    bytes.extend_from_slice(&viewport.viewport_id.to_le_bytes());
    for value in viewport
        .origin_unorm16
        .into_iter()
        .chain(viewport.extent_unorm16)
    {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_viewport(bytes: &[u8]) -> Result<CameraViewportV1, ()> {
    let mut cursor = RecoveryCursor::new(bytes);
    let viewport = CameraViewportV1::new(
        cursor.read_u16()?,
        [cursor.read_u16()?, cursor.read_u16()?],
        [cursor.read_u16()?, cursor.read_u16()?],
    )
    .map_err(|_| ())?;
    cursor.finish()?;
    Ok(viewport)
}

fn encode_projection(profile: CameraProjectionProfileV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(20);
    bytes.extend_from_slice(&profile.vertical_fov_millidegrees.to_le_bytes());
    bytes.extend_from_slice(&profile.near_plane_micrometres.to_le_bytes());
    bytes.extend_from_slice(&profile.far_plane_micrometres.to_le_bytes());
    bytes
}

fn decode_projection(bytes: &[u8]) -> Result<CameraProjectionProfileV1, ()> {
    let mut cursor = RecoveryCursor::new(bytes);
    let profile =
        CameraProjectionProfileV1::new(cursor.read_u32()?, cursor.read_u64()?, cursor.read_u64()?)
            .map_err(|_| ())?;
    cursor.finish()?;
    Ok(profile)
}

fn decode_presentation_role(value: u8) -> Result<PresentationRoleV1, ()> {
    match value {
        0 => Ok(PresentationRoleV1::Environment),
        1 => Ok(PresentationRoleV1::PlayerAvatar),
        2 => Ok(PresentationRoleV1::InteractiveObject),
        3 => Ok(PresentationRoleV1::Item),
        4 => Ok(PresentationRoleV1::Character),
        _ => Err(()),
    }
}

fn decode_camera_role(value: u8) -> Result<CameraRoleV1, ()> {
    match value {
        0 => Ok(CameraRoleV1::PrimaryThirdPerson),
        1 => Ok(CameraRoleV1::DeveloperCapture),
        _ => Err(()),
    }
}

fn decode_interpolation_policy(value: u8) -> Result<CameraInterpolationPolicyV1, ()> {
    match value {
        0 => Ok(CameraInterpolationPolicyV1::Hold),
        1 => Ok(CameraInterpolationPolicyV1::LinearPose),
        _ => Err(()),
    }
}

struct RecoveryCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> RecoveryCursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], ()> {
        let end = self.position.checked_add(length).ok_or(())?;
        let value = self.bytes.get(self.position..end).ok_or(())?;
        self.position = end;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, ()> {
        Ok(u16::from_le_bytes(
            self.read_exact(2)?.try_into().map_err(|_| ())?,
        ))
    }

    fn read_u32(&mut self) -> Result<u32, ()> {
        Ok(u32::from_le_bytes(
            self.read_exact(4)?.try_into().map_err(|_| ())?,
        ))
    }

    fn read_i32(&mut self) -> Result<i32, ()> {
        Ok(i32::from_le_bytes(
            self.read_exact(4)?.try_into().map_err(|_| ())?,
        ))
    }

    fn read_u64(&mut self) -> Result<u64, ()> {
        Ok(u64::from_le_bytes(
            self.read_exact(8)?.try_into().map_err(|_| ())?,
        ))
    }

    fn read_i64(&mut self) -> Result<i64, ()> {
        Ok(i64::from_le_bytes(
            self.read_exact(8)?.try_into().map_err(|_| ())?,
        ))
    }

    fn finish(self) -> Result<(), ()> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(())
        }
    }
}
