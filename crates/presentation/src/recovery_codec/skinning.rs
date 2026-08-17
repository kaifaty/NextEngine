use next_contracts::animation_content::NeutralTransformV1;
use next_contracts::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U32, CanonicalDecodeLimits, CanonicalField, DecodedCanonicalSegment,
    decode_canonical_segment, encode_canonical_segment,
};
use next_contracts::ids::{AssetId, PersistentId, SchemaId};
use next_contracts::presentation::{
    BaseSkinningProjectionModeV1, CHARACTER_SKINNING_PRESENTATION_RECORD_SCHEMA_VERSION,
    CharacterSkinningPresentationRecordV1, PRESENTATION_MAX_RENDER_JOINT_POSES,
    PresentationObjectKeyV1, RenderJointPoseV1,
};
use next_contracts::project::AssetRevisionRefV1;

use super::{
    RECOVERY_OWNER, RecoveryCursor, decode_hash, decode_id, decode_presentation_role,
    decode_sequence, decode_u8, decode_u32, encode_sequence, ensure_segment, field, hash_field,
    id_field,
};

const CHARACTER_SKINNING_RECORD_SCHEMA: &str =
    "nextengine.character-skinning-presentation-recovery-record.v1";

pub(super) fn encode_character_skinning_record(
    index: usize,
    record: &CharacterSkinningPresentationRecordV1,
) -> Result<Vec<u8>, ()> {
    record.validate().map_err(|_| ())?;
    let joint_poses = record
        .ordered_local_joint_poses
        .iter()
        .map(encode_render_joint_pose)
        .collect::<Result<Vec<_>, _>>()?;
    encode_canonical_segment(
        RECOVERY_OWNER,
        CHARACTER_SKINNING_RECORD_SCHEMA,
        &format!("character-skinning-{index}"),
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                record.schema_version.to_le_bytes().to_vec(),
            ),
            hash_field(2, record.object_key.snapshot_epoch),
            id_field(3, record.object_key.persistent_id.as_bytes()),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_U8,
                vec![record.object_key.presentation_role as u8],
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U32,
                record.object_key.incarnation.to_le_bytes().to_vec(),
            ),
            id_field(6, record.mesh_revision.asset_id.as_bytes()),
            hash_field(7, record.mesh_revision.record_sha256),
            id_field(8, record.skinning_profile_revision.asset_id.as_bytes()),
            hash_field(9, record.skinning_profile_revision.record_sha256),
            id_field(10, record.source_skeleton_revision.asset_id.as_bytes()),
            hash_field(11, record.source_skeleton_revision.record_sha256),
            id_field(12, record.source_body_schema_revision.asset_id.as_bytes()),
            hash_field(13, record.source_body_schema_revision.record_sha256),
            hash_field(14, record.source_animation_profile_hash),
            CanonicalField::new(15, CANONICAL_TYPE_U8, vec![record.projection_mode as u8]),
            CanonicalField::new(16, CANONICAL_TYPE_SEQUENCE, encode_sequence(joint_poses)?),
            hash_field(17, record.canonical_hash),
        ],
    )
    .map_err(|_| ())
}

pub(super) fn decode_character_skinning_record(
    index: usize,
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<CharacterSkinningPresentationRecordV1, ()> {
    let segment = decode_canonical_segment(bytes, limits).map_err(|_| ())?;
    ensure_segment(
        &segment,
        RECOVERY_OWNER,
        CHARACTER_SKINNING_RECORD_SCHEMA,
        &format!("character-skinning-{index}"),
        17,
    )?;
    if decode_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?
        != CHARACTER_SKINNING_PRESENTATION_RECORD_SCHEMA_VERSION
    {
        return Err(());
    }
    let object_key = PresentationObjectKeyV1 {
        snapshot_epoch: decode_hash(field(&segment, 2, CANONICAL_TYPE_HASH256)?)?,
        persistent_id: PersistentId::from_bytes(decode_id(field(
            &segment,
            3,
            CANONICAL_TYPE_ID128,
        )?)?),
        presentation_role: decode_presentation_role(decode_u8(field(
            &segment,
            4,
            CANONICAL_TYPE_U8,
        )?)?)?,
        incarnation: decode_u32(field(&segment, 5, CANONICAL_TYPE_U32)?)?,
    };
    let mesh_revision = decode_asset_revision(&segment, 6, 7)?;
    let skinning_profile_revision = decode_asset_revision(&segment, 8, 9)?;
    let source_skeleton_revision = decode_asset_revision(&segment, 10, 11)?;
    let source_body_schema_revision = decode_asset_revision(&segment, 12, 13)?;
    let source_animation_profile_hash = decode_hash(field(&segment, 14, CANONICAL_TYPE_HASH256)?)?;
    let projection_mode =
        decode_skinning_projection_mode(decode_u8(field(&segment, 15, CANONICAL_TYPE_U8)?)?)?;
    let ordered_local_joint_poses = decode_sequence(
        field(&segment, 16, CANONICAL_TYPE_SEQUENCE)?,
        PRESENTATION_MAX_RENDER_JOINT_POSES,
        limits.max_field_payload_bytes,
    )?
    .into_iter()
    .map(decode_render_joint_pose)
    .collect::<Result<Vec<_>, _>>()?;
    let expected_hash = decode_hash(field(&segment, 17, CANONICAL_TYPE_HASH256)?)?;
    let record = CharacterSkinningPresentationRecordV1::new(
        object_key,
        mesh_revision,
        skinning_profile_revision,
        source_skeleton_revision,
        source_body_schema_revision,
        source_animation_profile_hash,
        projection_mode,
        ordered_local_joint_poses,
    )
    .map_err(|_| ())?;
    if record.canonical_hash != expected_hash
        || encode_character_skinning_record(index, &record)? != bytes
    {
        return Err(());
    }
    Ok(record)
}

fn decode_asset_revision(
    segment: &DecodedCanonicalSegment,
    asset_field_id: u32,
    hash_field_id: u32,
) -> Result<AssetRevisionRefV1, ()> {
    Ok(AssetRevisionRefV1 {
        asset_id: AssetId::from_bytes(decode_id(field(
            segment,
            asset_field_id,
            CANONICAL_TYPE_ID128,
        )?)?),
        record_sha256: decode_hash(field(segment, hash_field_id, CANONICAL_TYPE_HASH256)?)?,
    })
}

fn encode_render_joint_pose(pose: &RenderJointPoseV1) -> Result<Vec<u8>, ()> {
    let id = pose.render_joint_id.as_str().as_bytes();
    let mut bytes = Vec::with_capacity(id.len().checked_add(60).ok_or(())?);
    bytes.extend_from_slice(&u32::try_from(id.len()).map_err(|_| ())?.to_le_bytes());
    bytes.extend_from_slice(id);
    for value in pose.local_transform.translation_micrometres {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in pose.local_transform.rotation_q1_30 {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in pose.local_transform.scale_q16_16 {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}

fn decode_render_joint_pose(bytes: &[u8]) -> Result<RenderJointPoseV1, ()> {
    let mut cursor = RecoveryCursor::new(bytes);
    let id_length = usize::try_from(cursor.read_u32()?).map_err(|_| ())?;
    let id = std::str::from_utf8(cursor.read_exact(id_length)?).map_err(|_| ())?;
    let mut translation_micrometres = [0_i64; 3];
    for value in &mut translation_micrometres {
        *value = cursor.read_i64()?;
    }
    let mut rotation_q1_30 = [0_i32; 4];
    for value in &mut rotation_q1_30 {
        *value = cursor.read_i32()?;
    }
    let mut scale_q16_16 = [0_u32; 3];
    for value in &mut scale_q16_16 {
        *value = cursor.read_u32()?;
    }
    cursor.finish()?;
    Ok(RenderJointPoseV1 {
        render_joint_id: SchemaId::new(id).map_err(|_| ())?,
        local_transform: NeutralTransformV1 {
            translation_micrometres,
            rotation_q1_30,
            scale_q16_16,
        },
    })
}

fn decode_skinning_projection_mode(value: u8) -> Result<BaseSkinningProjectionModeV1, ()> {
    match value {
        1 => Ok(BaseSkinningProjectionModeV1::Sampled),
        2 => Ok(BaseSkinningProjectionModeV1::BindPoseFallback),
        _ => Err(()),
    }
}
