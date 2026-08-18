use crate::animation_content::NeutralTransformV1;
use crate::ids::{ContentHash, SchemaId};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::physics::PhysicsPoseV1;
use crate::project::{AssetRevisionRefV1, domain_hash};

use super::{
    PresentationContractError, PresentationObjectKeyV1, asset_revision_value, number, object,
    object_key_value, string,
};

pub const CHARACTER_SKINNING_PRESENTATION_RECORD_SCHEMA_VERSION: u32 = 1;
pub const PRESENTATION_MAX_CHARACTER_SKINNING_RECORDS: usize = 4_096;
pub const PRESENTATION_MAX_RENDER_JOINT_POSES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BaseSkinningProjectionModeV1 {
    Sampled = 1,
    BindPoseFallback = 2,
    HeldPresentationPose = 3,
}

impl BaseSkinningProjectionModeV1 {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Sampled => "Sampled",
            Self::BindPoseFallback => "BindPoseFallback",
            Self::HeldPresentationPose => "HeldPresentationPose",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CharacterDeformationLodV1 {
    FullCorrectives = 1,
    ReducedCorrectives = 2,
    BaseSkinningOnly = 3,
    Culled = 4,
}

impl CharacterDeformationLodV1 {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::FullCorrectives => "FullCorrectives",
            Self::ReducedCorrectives => "ReducedCorrectives",
            Self::BaseSkinningOnly => "BaseSkinningOnly",
            Self::Culled => "Culled",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RenderJointPoseV1 {
    pub render_joint_id: SchemaId,
    pub local_transform: NeutralTransformV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterSkinningPresentationRecordV1 {
    pub schema_version: u32,
    pub object_key: PresentationObjectKeyV1,
    pub mesh_revision: AssetRevisionRefV1,
    pub skinning_profile_revision: AssetRevisionRefV1,
    pub source_skeleton_revision: AssetRevisionRefV1,
    pub source_body_schema_revision: AssetRevisionRefV1,
    pub source_animation_profile_hash: ContentHash,
    pub projection_mode: BaseSkinningProjectionModeV1,
    pub deformation_lod: CharacterDeformationLodV1,
    pub ordered_local_joint_poses: Vec<RenderJointPoseV1>,
    pub canonical_hash: ContentHash,
}

impl CharacterSkinningPresentationRecordV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the skinning publication binds every immutable source revision explicitly"
    )]
    pub fn new(
        object_key: PresentationObjectKeyV1,
        mesh_revision: AssetRevisionRefV1,
        skinning_profile_revision: AssetRevisionRefV1,
        source_skeleton_revision: AssetRevisionRefV1,
        source_body_schema_revision: AssetRevisionRefV1,
        source_animation_profile_hash: ContentHash,
        projection_mode: BaseSkinningProjectionModeV1,
        deformation_lod: CharacterDeformationLodV1,
        mut ordered_local_joint_poses: Vec<RenderJointPoseV1>,
    ) -> Result<Self, PresentationContractError> {
        ordered_local_joint_poses
            .sort_by(|left, right| left.render_joint_id.cmp(&right.render_joint_id));
        let mut value = Self {
            schema_version: CHARACTER_SKINNING_PRESENTATION_RECORD_SCHEMA_VERSION,
            object_key,
            mesh_revision,
            skinning_profile_revision,
            source_skeleton_revision,
            source_body_schema_revision,
            source_animation_profile_hash,
            projection_mode,
            deformation_lod,
            ordered_local_joint_poses,
            canonical_hash: ContentHash::default(),
        };
        value.validate_body()?;
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        self.validate_body()?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), PresentationContractError> {
        if self.schema_version != CHARACTER_SKINNING_PRESENTATION_RECORD_SCHEMA_VERSION
            || self.mesh_revision.record_sha256 == ContentHash::default()
            || self.skinning_profile_revision.record_sha256 == ContentHash::default()
            || self.source_skeleton_revision.record_sha256 == ContentHash::default()
            || self.source_body_schema_revision.record_sha256 == ContentHash::default()
            || self.source_animation_profile_hash == ContentHash::default()
            || (self.projection_mode == BaseSkinningProjectionModeV1::BindPoseFallback
                && self.deformation_lod != CharacterDeformationLodV1::BaseSkinningOnly)
            || self.ordered_local_joint_poses.is_empty()
            || self.ordered_local_joint_poses.len() > PRESENTATION_MAX_RENDER_JOINT_POSES
            || self
                .ordered_local_joint_poses
                .windows(2)
                .any(|pair| pair[0].render_joint_id >= pair[1].render_joint_id)
        {
            return Err(PresentationContractError::InvalidSkinningPose);
        }
        for pose in &self.ordered_local_joint_poses {
            if pose.local_transform.scale_q16_16 != [1 << 16; 3]
                || (PhysicsPoseV1 {
                    translation_micrometres: pose.local_transform.translation_micrometres,
                    rotation_q1_30: pose.local_transform.rotation_q1_30,
                })
                .validate()
                .is_err()
            {
                return Err(PresentationContractError::InvalidSkinningPose);
            }
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.character-skinning-presentation-record.v1",
            &encode_canonical_jcs(&object([
                ("mesh_revision", asset_revision_value(self.mesh_revision)),
                ("object_key", object_key_value(self.object_key)),
                (
                    "ordered_local_joint_poses",
                    JcsValue::Array(
                        self.ordered_local_joint_poses
                            .iter()
                            .map(|pose| {
                                object([
                                    (
                                        "local_transform",
                                        local_transform_value(pose.local_transform),
                                    ),
                                    (
                                        "render_joint_id",
                                        string(pose.render_joint_id.as_str().to_owned()),
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
                ("projection_mode", string(self.projection_mode.token())),
                ("deformation_lod", string(self.deformation_lod.token())),
                ("schema_version", number(self.schema_version)),
                (
                    "skinning_profile_revision",
                    asset_revision_value(self.skinning_profile_revision),
                ),
                (
                    "source_animation_profile_hash",
                    string(self.source_animation_profile_hash.to_hex()),
                ),
                (
                    "source_body_schema_revision",
                    asset_revision_value(self.source_body_schema_revision),
                ),
                (
                    "source_skeleton_revision",
                    asset_revision_value(self.source_skeleton_revision),
                ),
            ])),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterSkinningPresentationBatchV1 {
    pub batch_index: u32,
    pub first_global_ordinal: u32,
    pub records: Vec<CharacterSkinningPresentationRecordV1>,
    pub records_root: ContentHash,
}

impl CharacterSkinningPresentationBatchV1 {
    fn new(
        batch_index: u32,
        first_global_ordinal: u32,
        records: Vec<CharacterSkinningPresentationRecordV1>,
    ) -> Result<Self, PresentationContractError> {
        if records.is_empty() {
            return Err(PresentationContractError::EmptyBatch);
        }
        let mut value = Self {
            batch_index,
            first_global_ordinal,
            records,
            records_root: ContentHash::default(),
        };
        value.records_root = value.computed_root();
        Ok(value)
    }

    fn validate(&self) -> Result<(), PresentationContractError> {
        if self.records.is_empty() {
            return Err(PresentationContractError::EmptyBatch);
        }
        for record in &self.records {
            record.validate()?;
        }
        if self.computed_root() != self.records_root {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_root(&self) -> ContentHash {
        domain_hash(
            "nextengine.presentation-batch.v1",
            &encode_canonical_jcs(&object([
                ("batch_index", number(self.batch_index)),
                ("batch_kind", string("CharacterSkinning")),
                ("first_global_ordinal", number(self.first_global_ordinal)),
                (
                    "record_count",
                    number(u32::try_from(self.records.len()).unwrap_or(u32::MAX)),
                ),
                (
                    "ordered_record_hashes",
                    JcsValue::Array(
                        self.records
                            .iter()
                            .map(|record| string(record.canonical_hash.to_hex()))
                            .collect(),
                    ),
                ),
            ])),
        )
    }
}

pub(super) fn build_character_skinning_batches(
    snapshot_epoch: ContentHash,
    mut records: Vec<CharacterSkinningPresentationRecordV1>,
    max_records_per_batch: usize,
) -> Result<Vec<CharacterSkinningPresentationBatchV1>, PresentationContractError> {
    if max_records_per_batch == 0 || records.len() > PRESENTATION_MAX_CHARACTER_SKINNING_RECORDS {
        return Err(PresentationContractError::InvalidBatchProfile);
    }
    for record in &records {
        record.validate()?;
        if record.object_key.snapshot_epoch != snapshot_epoch {
            return Err(PresentationContractError::SnapshotEpochMismatch);
        }
    }
    records.sort_by_key(|record| record.object_key);
    if records
        .windows(2)
        .any(|pair| pair[0].object_key == pair[1].object_key)
    {
        return Err(PresentationContractError::DuplicateSkinningKey);
    }
    records
        .chunks(max_records_per_batch)
        .enumerate()
        .map(|(batch_index, records)| {
            let first = batch_index
                .checked_mul(max_records_per_batch)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or(PresentationContractError::LimitExceeded)?;
            CharacterSkinningPresentationBatchV1::new(
                u32::try_from(batch_index).map_err(|_| PresentationContractError::LimitExceeded)?,
                first,
                records.to_vec(),
            )
        })
        .collect()
}

pub(super) fn validate_character_skinning_batches(
    snapshot_epoch: ContentHash,
    batches: &[CharacterSkinningPresentationBatchV1],
) -> Result<(), PresentationContractError> {
    let mut expected_batch = 0_u32;
    let mut expected_ordinal = 0_u32;
    let mut prior_key = None;
    let mut count = 0_usize;
    for batch in batches {
        if batch.batch_index != expected_batch || batch.first_global_ordinal != expected_ordinal {
            return Err(PresentationContractError::InvalidBatchBoundary);
        }
        batch.validate()?;
        for record in &batch.records {
            if record.object_key.snapshot_epoch != snapshot_epoch
                || prior_key.is_some_and(|prior| prior >= record.object_key)
            {
                return Err(PresentationContractError::NonCanonicalOrder);
            }
            prior_key = Some(record.object_key);
        }
        count = count
            .checked_add(batch.records.len())
            .ok_or(PresentationContractError::LimitExceeded)?;
        expected_batch = expected_batch
            .checked_add(1)
            .ok_or(PresentationContractError::LimitExceeded)?;
        expected_ordinal = expected_ordinal
            .checked_add(
                u32::try_from(batch.records.len())
                    .map_err(|_| PresentationContractError::LimitExceeded)?,
            )
            .ok_or(PresentationContractError::LimitExceeded)?;
    }
    if count > PRESENTATION_MAX_CHARACTER_SKINNING_RECORDS {
        return Err(PresentationContractError::LimitExceeded);
    }
    Ok(())
}

pub(super) fn character_skinning_batches_value(
    batches: &[CharacterSkinningPresentationBatchV1],
) -> JcsValue {
    JcsValue::Array(
        batches
            .iter()
            .map(|batch| {
                object([
                    ("batch_index", number(batch.batch_index)),
                    ("first_global_ordinal", number(batch.first_global_ordinal)),
                    ("records_root", string(batch.records_root.to_hex())),
                ])
            })
            .collect(),
    )
}

fn local_transform_value(transform: NeutralTransformV1) -> JcsValue {
    object([
        (
            "rotation_q1_30",
            JcsValue::Array(
                transform
                    .rotation_q1_30
                    .into_iter()
                    .map(|value| string(format!("{:08x}", value as u32)))
                    .collect(),
            ),
        ),
        (
            "scale_q16_16",
            JcsValue::Array(transform.scale_q16_16.into_iter().map(number).collect()),
        ),
        (
            "translation_micrometres",
            JcsValue::Array(
                transform
                    .translation_micrometres
                    .into_iter()
                    .map(|value| string(format!("{:016x}", value as u64)))
                    .collect(),
            ),
        ),
    ])
}
