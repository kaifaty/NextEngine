//! Backend-neutral skeletal-animation source records from SPEC-24/SPEC-28.
//!
//! These immutable records are content authority only. They contain no graph
//! cursor, body handle, runtime pose or direct transform mutation path.

mod codec;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeError, CanonicalDecodeLimits,
    CanonicalError, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash, IdentifierError, SchemaId};
use crate::project::{AssetRevisionRefV1, domain_hash};

use self::codec::{
    Reader, append_len, append_string, decode_keys, decode_markers, encode_keys, encode_markers,
    field, read_fixed, read_string, read_u32, read_u64, require_envelope,
};

pub const NEUTRAL_SKELETON_SCHEMA_ID: &str = "nextengine.content.skeleton";
pub const NEUTRAL_ANIMATION_SCHEMA_ID: &str = "nextengine.content.animation";
pub const NEUTRAL_ANIMATION_OWNER_ID: &str = "nextengine.assets";
pub const NEUTRAL_SKELETON_SEGMENT_ID: &str = "nextengine.skeleton.v1";
pub const NEUTRAL_ANIMATION_SEGMENT_ID: &str = "nextengine.animation.v1";
pub const NEUTRAL_ANIMATION_SCHEMA_VERSION: u32 = 1;
pub const MAX_SKELETON_JOINTS_V1: usize = 1_024;
pub const MAX_SKELETON_ROOTS_V1: usize = 64;
pub const MAX_JOINT_SEMANTIC_ROLES_V1: usize = 32;
pub const MAX_ANIMATION_CHANNELS_V1: usize = 8_192;
pub const MAX_ANIMATION_KEYS_V1: usize = 16_777_216;
pub const MAX_ANIMATION_MARKERS_V1: usize = 65_535;
pub const MAX_ANIMATION_DURATION_US_V1: u64 = 86_400_000_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum AnimationPropertyV1 {
    Translation = 1,
    Rotation = 2,
    Scale = 3,
    MorphWeight = 4,
}

impl AnimationPropertyV1 {
    fn from_tag(tag: u32) -> Result<Self, NeutralAnimationContentErrorV1> {
        match tag {
            1 => Ok(Self::Translation),
            2 => Ok(Self::Rotation),
            3 => Ok(Self::Scale),
            4 => Ok(Self::MorphWeight),
            _ => Err(NeutralAnimationContentErrorV1::UnknownTag),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum AnimationInterpolationV1 {
    Step = 1,
    Linear = 2,
    CubicHermite = 3,
}

impl AnimationInterpolationV1 {
    fn from_tag(tag: u32) -> Result<Self, NeutralAnimationContentErrorV1> {
        match tag {
            1 => Ok(Self::Step),
            2 => Ok(Self::Linear),
            3 => Ok(Self::CubicHermite),
            _ => Err(NeutralAnimationContentErrorV1::UnknownTag),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum AnimationWrapModeV1 {
    Clamp = 1,
    Loop = 2,
    PingPong = 3,
}

impl AnimationWrapModeV1 {
    fn from_tag(tag: u32) -> Result<Self, NeutralAnimationContentErrorV1> {
        match tag {
            1 => Ok(Self::Clamp),
            2 => Ok(Self::Loop),
            3 => Ok(Self::PingPong),
            _ => Err(NeutralAnimationContentErrorV1::UnknownTag),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralTransformV1 {
    pub translation_micrometres: [i64; 3],
    pub rotation_q1_30: [i32; 4],
    pub scale_q16_16: [u32; 3],
}

impl NeutralTransformV1 {
    #[must_use]
    pub const fn translated(translation_micrometres: [i64; 3]) -> Self {
        Self {
            translation_micrometres,
            rotation_q1_30: [0, 0, 0, 1 << 30],
            scale_q16_16: [1 << 16; 3],
        }
    }

    fn validate(self) -> Result<(), NeutralAnimationContentErrorV1> {
        if self.rotation_q1_30 == [0; 4] || self.scale_q16_16.contains(&0) {
            Err(NeutralAnimationContentErrorV1::InvalidValue)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralSkeletonJointV1 {
    pub joint_key: SchemaId,
    pub parent_joint_key: Option<SchemaId>,
    pub bind_transform: NeutralTransformV1,
    pub semantic_roles: Vec<SchemaId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralSkeletonV1 {
    pub schema_version: u32,
    pub asset_id: AssetId,
    pub record_revision: u64,
    pub coordinate_profile_id: SchemaId,
    pub declared_root_joint_keys: Vec<SchemaId>,
    pub joints: Vec<NeutralSkeletonJointV1>,
    pub content_hash: ContentHash,
}

impl NeutralSkeletonV1 {
    pub fn new(
        asset_id: AssetId,
        record_revision: u64,
        coordinate_profile_id: SchemaId,
        mut declared_root_joint_keys: Vec<SchemaId>,
        joints: Vec<NeutralSkeletonJointV1>,
    ) -> Result<Self, NeutralAnimationContentErrorV1> {
        if record_revision == 0
            || joints.is_empty()
            || joints.len() > MAX_SKELETON_JOINTS_V1
            || declared_root_joint_keys.len() > MAX_SKELETON_ROOTS_V1
        {
            return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
        }
        declared_root_joint_keys.sort();
        ensure_strict(&declared_root_joint_keys)?;
        let mut by_key = BTreeMap::new();
        for joint in joints {
            joint.bind_transform.validate()?;
            if joint.semantic_roles.len() > MAX_JOINT_SEMANTIC_ROLES_V1 {
                return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
            }
            ensure_strict(&joint.semantic_roles)?;
            if by_key.insert(joint.joint_key.clone(), joint).is_some() {
                return Err(NeutralAnimationContentErrorV1::DuplicateIdentity);
            }
        }
        let mut depths = BTreeMap::new();
        for key in by_key.keys() {
            let depth = joint_depth(key, &by_key, &mut BTreeSet::new())?;
            depths.insert(key.clone(), depth);
        }
        let actual_roots = by_key
            .values()
            .filter(|joint| joint.parent_joint_key.is_none())
            .map(|joint| joint.joint_key.clone())
            .collect::<Vec<_>>();
        if actual_roots != declared_root_joint_keys {
            return Err(NeutralAnimationContentErrorV1::MissingReference);
        }
        let mut joints = by_key.into_values().collect::<Vec<_>>();
        joints.sort_by(|left, right| {
            depths[&left.joint_key]
                .cmp(&depths[&right.joint_key])
                .then_with(|| left.joint_key.cmp(&right.joint_key))
        });
        let mut value = Self {
            schema_version: NEUTRAL_ANIMATION_SCHEMA_VERSION,
            asset_id,
            record_revision,
            coordinate_profile_id,
            declared_root_joint_keys,
            joints,
            content_hash: ContentHash::default(),
        };
        value.content_hash = value.compute_content_hash()?;
        Ok(value)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, NeutralAnimationContentErrorV1> {
        let mut fields = self.body_fields()?;
        fields.push(CanonicalField::new(
            7,
            CANONICAL_TYPE_HASH256,
            self.content_hash.as_bytes().to_vec(),
        ));
        Ok(encode_canonical_segment(
            NEUTRAL_ANIMATION_OWNER_ID,
            NEUTRAL_SKELETON_SCHEMA_ID,
            NEUTRAL_SKELETON_SEGMENT_ID,
            fields,
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, NeutralAnimationContentErrorV1> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment,
            NEUTRAL_SKELETON_SCHEMA_ID,
            NEUTRAL_SKELETON_SEGMENT_ID,
            7,
        )?;
        let version = read_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?;
        if version != NEUTRAL_ANIMATION_SCHEMA_VERSION {
            return Err(NeutralAnimationContentErrorV1::UnsupportedVersion);
        }
        let asset_id = AssetId::from_bytes(read_fixed(field(&segment, 2, CANONICAL_TYPE_ID128)?)?);
        let revision = read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?;
        let coordinate = SchemaId::new(read_string(field(&segment, 4, CANONICAL_TYPE_UTF8_NFC)?)?)?;
        let roots = decode_ids(field(&segment, 5, CANONICAL_TYPE_BYTES)?)?;
        let joints = decode_joints(field(&segment, 6, CANONICAL_TYPE_BYTES)?)?;
        let expected =
            ContentHash::from_bytes(read_fixed(field(&segment, 7, CANONICAL_TYPE_HASH256)?)?);
        let value = Self::new(asset_id, revision, coordinate, roots, joints)?;
        if value.content_hash != expected || value.canonical_bytes()? != bytes {
            return Err(NeutralAnimationContentErrorV1::HashOrCanonicalMismatch);
        }
        Ok(value)
    }

    pub fn record_sha256(&self) -> Result<ContentHash, NeutralAnimationContentErrorV1> {
        Ok(domain_hash(
            NEUTRAL_SKELETON_SEGMENT_ID,
            &self.canonical_bytes()?,
        ))
    }

    pub fn asset_revision(&self) -> Result<AssetRevisionRefV1, NeutralAnimationContentErrorV1> {
        Ok(AssetRevisionRefV1 {
            asset_id: self.asset_id,
            record_sha256: self.record_sha256()?,
        })
    }

    fn body_fields(&self) -> Result<Vec<CanonicalField>, NeutralAnimationContentErrorV1> {
        Ok(vec![
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                self.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_ID128, self.asset_id.as_bytes().to_vec()),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_U64,
                self.record_revision.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_UTF8_NFC,
                self.coordinate_profile_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_BYTES,
                encode_ids(&self.declared_root_joint_keys)?,
            ),
            CanonicalField::new(6, CANONICAL_TYPE_BYTES, encode_joints(&self.joints)?),
        ])
    }

    fn compute_content_hash(&self) -> Result<ContentHash, NeutralAnimationContentErrorV1> {
        Ok(domain_hash(
            "nextengine.neutral-skeleton.content.v1",
            &encode_canonical_segment(
                NEUTRAL_ANIMATION_OWNER_ID,
                NEUTRAL_SKELETON_SCHEMA_ID,
                "nextengine.skeleton-body.v1",
                self.body_fields()?,
            )?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NeutralAnimationValueV1 {
    Translation([i64; 3]),
    Rotation([i32; 4]),
    Scale([u32; 3]),
    MorphWeight(u16),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralAnimationKeyV1 {
    pub time_microseconds: u64,
    pub value: NeutralAnimationValueV1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralAnimationChannelV1 {
    pub joint_key: SchemaId,
    pub property: AnimationPropertyV1,
    pub interpolation: AnimationInterpolationV1,
    pub keys: Vec<NeutralAnimationKeyV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralAnimationMarkerV1 {
    pub time_microseconds: u64,
    pub marker_id: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralAnimationV1 {
    pub schema_version: u32,
    pub asset_id: AssetId,
    pub record_revision: u64,
    pub clip_id: SchemaId,
    pub skeleton_revision: AssetRevisionRefV1,
    pub duration_microseconds: u64,
    pub wrap_mode: AnimationWrapModeV1,
    pub channels: Vec<NeutralAnimationChannelV1>,
    pub markers: Vec<NeutralAnimationMarkerV1>,
    pub root_motion_intent: Vec<NeutralAnimationKeyV1>,
    pub content_hash: ContentHash,
}

impl NeutralAnimationV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "neutral clip fields are explicit"
    )]
    pub fn new(
        asset_id: AssetId,
        record_revision: u64,
        clip_id: SchemaId,
        skeleton_revision: AssetRevisionRefV1,
        duration_microseconds: u64,
        wrap_mode: AnimationWrapModeV1,
        mut channels: Vec<NeutralAnimationChannelV1>,
        mut markers: Vec<NeutralAnimationMarkerV1>,
        root_motion_intent: Vec<NeutralAnimationKeyV1>,
    ) -> Result<Self, NeutralAnimationContentErrorV1> {
        if record_revision == 0
            || duration_microseconds == 0
            || duration_microseconds > MAX_ANIMATION_DURATION_US_V1
            || channels.len() > MAX_ANIMATION_CHANNELS_V1
            || markers.len() > MAX_ANIMATION_MARKERS_V1
        {
            return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
        }
        let mut total_keys = root_motion_intent.len();
        for channel in &channels {
            total_keys = total_keys
                .checked_add(channel.keys.len())
                .ok_or(NeutralAnimationContentErrorV1::LimitOrRevision)?;
            validate_keys(
                &channel.keys,
                channel.property,
                channel.interpolation,
                duration_microseconds,
            )?;
        }
        if total_keys > MAX_ANIMATION_KEYS_V1 {
            return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
        }
        if !root_motion_intent.is_empty() {
            validate_keys(
                &root_motion_intent,
                AnimationPropertyV1::Translation,
                AnimationInterpolationV1::Linear,
                duration_microseconds,
            )?;
        }
        channels.sort_by(|left, right| {
            left.joint_key
                .cmp(&right.joint_key)
                .then_with(|| left.property.cmp(&right.property))
        });
        if channels.windows(2).any(|pair| {
            pair[0].joint_key == pair[1].joint_key && pair[0].property == pair[1].property
        }) {
            return Err(NeutralAnimationContentErrorV1::DuplicateIdentity);
        }
        markers.sort();
        if markers.windows(2).any(|pair| pair[0] == pair[1])
            || markers
                .last()
                .is_some_and(|marker| marker.time_microseconds > duration_microseconds)
        {
            return Err(NeutralAnimationContentErrorV1::InvalidValue);
        }
        let mut value = Self {
            schema_version: NEUTRAL_ANIMATION_SCHEMA_VERSION,
            asset_id,
            record_revision,
            clip_id,
            skeleton_revision,
            duration_microseconds,
            wrap_mode,
            channels,
            markers,
            root_motion_intent,
            content_hash: ContentHash::default(),
        };
        value.content_hash = value.compute_content_hash()?;
        Ok(value)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, NeutralAnimationContentErrorV1> {
        let mut fields = self.body_fields()?;
        fields.push(CanonicalField::new(
            12,
            CANONICAL_TYPE_HASH256,
            self.content_hash.as_bytes().to_vec(),
        ));
        Ok(encode_canonical_segment(
            NEUTRAL_ANIMATION_OWNER_ID,
            NEUTRAL_ANIMATION_SCHEMA_ID,
            NEUTRAL_ANIMATION_SEGMENT_ID,
            fields,
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, NeutralAnimationContentErrorV1> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment,
            NEUTRAL_ANIMATION_SCHEMA_ID,
            NEUTRAL_ANIMATION_SEGMENT_ID,
            12,
        )?;
        if read_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)? != NEUTRAL_ANIMATION_SCHEMA_VERSION {
            return Err(NeutralAnimationContentErrorV1::UnsupportedVersion);
        }
        let asset_id = AssetId::from_bytes(read_fixed(field(&segment, 2, CANONICAL_TYPE_ID128)?)?);
        let revision = read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?;
        let clip_id = SchemaId::new(read_string(field(&segment, 4, CANONICAL_TYPE_UTF8_NFC)?)?)?;
        let skeleton_revision = AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes(read_fixed(field(&segment, 5, CANONICAL_TYPE_ID128)?)?),
            record_sha256: ContentHash::from_bytes(read_fixed(field(
                &segment,
                6,
                CANONICAL_TYPE_HASH256,
            )?)?),
        };
        let duration = read_u64(field(&segment, 7, CANONICAL_TYPE_U64)?)?;
        let wrap =
            AnimationWrapModeV1::from_tag(read_u32(field(&segment, 8, CANONICAL_TYPE_U32)?)?)?;
        let channels = decode_channels(field(&segment, 9, CANONICAL_TYPE_BYTES)?)?;
        let markers = decode_markers(field(&segment, 10, CANONICAL_TYPE_BYTES)?)?;
        let root_motion = decode_keys(field(&segment, 11, CANONICAL_TYPE_BYTES)?)?;
        let expected =
            ContentHash::from_bytes(read_fixed(field(&segment, 12, CANONICAL_TYPE_HASH256)?)?);
        let value = Self::new(
            asset_id,
            revision,
            clip_id,
            skeleton_revision,
            duration,
            wrap,
            channels,
            markers,
            root_motion,
        )?;
        if value.content_hash != expected || value.canonical_bytes()? != bytes {
            return Err(NeutralAnimationContentErrorV1::HashOrCanonicalMismatch);
        }
        Ok(value)
    }

    pub fn record_sha256(&self) -> Result<ContentHash, NeutralAnimationContentErrorV1> {
        Ok(domain_hash(
            NEUTRAL_ANIMATION_SEGMENT_ID,
            &self.canonical_bytes()?,
        ))
    }

    pub fn asset_revision(&self) -> Result<AssetRevisionRefV1, NeutralAnimationContentErrorV1> {
        Ok(AssetRevisionRefV1 {
            asset_id: self.asset_id,
            record_sha256: self.record_sha256()?,
        })
    }

    fn body_fields(&self) -> Result<Vec<CanonicalField>, NeutralAnimationContentErrorV1> {
        Ok(vec![
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                self.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_ID128, self.asset_id.as_bytes().to_vec()),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_U64,
                self.record_revision.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_UTF8_NFC,
                self.clip_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_ID128,
                self.skeleton_revision.asset_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_HASH256,
                self.skeleton_revision.record_sha256.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_U64,
                self.duration_microseconds.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_U32,
                (self.wrap_mode as u32).to_le_bytes().to_vec(),
            ),
            CanonicalField::new(9, CANONICAL_TYPE_BYTES, encode_channels(&self.channels)?),
            CanonicalField::new(10, CANONICAL_TYPE_BYTES, encode_markers(&self.markers)?),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_BYTES,
                encode_keys(&self.root_motion_intent)?,
            ),
        ])
    }

    fn compute_content_hash(&self) -> Result<ContentHash, NeutralAnimationContentErrorV1> {
        Ok(domain_hash(
            "nextengine.neutral-animation.content.v1",
            &encode_canonical_segment(
                NEUTRAL_ANIMATION_OWNER_ID,
                NEUTRAL_ANIMATION_SCHEMA_ID,
                "nextengine.animation-body.v1",
                self.body_fields()?,
            )?,
        ))
    }
}

fn validate_keys(
    keys: &[NeutralAnimationKeyV1],
    property: AnimationPropertyV1,
    interpolation: AnimationInterpolationV1,
    duration: u64,
) -> Result<(), NeutralAnimationContentErrorV1> {
    if keys.is_empty()
        || keys
            .windows(2)
            .any(|pair| pair[0].time_microseconds >= pair[1].time_microseconds)
        || keys
            .last()
            .is_some_and(|key| key.time_microseconds > duration)
        || (property == AnimationPropertyV1::Rotation
            && interpolation == AnimationInterpolationV1::CubicHermite)
    {
        return Err(NeutralAnimationContentErrorV1::InvalidValue);
    }
    for key in keys {
        let valid = match (property, key.value) {
            (AnimationPropertyV1::Translation, NeutralAnimationValueV1::Translation(_)) => true,
            (AnimationPropertyV1::Rotation, NeutralAnimationValueV1::Rotation(value)) => {
                value != [0; 4]
            }
            (AnimationPropertyV1::Scale, NeutralAnimationValueV1::Scale(value)) => {
                !value.contains(&0)
            }
            (AnimationPropertyV1::MorphWeight, NeutralAnimationValueV1::MorphWeight(_)) => true,
            _ => false,
        };
        if !valid {
            return Err(NeutralAnimationContentErrorV1::InvalidValue);
        }
    }
    Ok(())
}

fn joint_depth(
    key: &SchemaId,
    joints: &BTreeMap<SchemaId, NeutralSkeletonJointV1>,
    visiting: &mut BTreeSet<SchemaId>,
) -> Result<u16, NeutralAnimationContentErrorV1> {
    if !visiting.insert(key.clone()) {
        return Err(NeutralAnimationContentErrorV1::Cycle);
    }
    let joint = joints
        .get(key)
        .ok_or(NeutralAnimationContentErrorV1::MissingReference)?;
    let depth = match &joint.parent_joint_key {
        Some(parent) => joint_depth(parent, joints, visiting)?
            .checked_add(1)
            .ok_or(NeutralAnimationContentErrorV1::LimitOrRevision)?,
        None => 0,
    };
    visiting.remove(key);
    Ok(depth)
}

fn ensure_strict<T: Ord>(values: &[T]) -> Result<(), NeutralAnimationContentErrorV1> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(NeutralAnimationContentErrorV1::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn encode_ids(values: &[SchemaId]) -> Result<Vec<u8>, NeutralAnimationContentErrorV1> {
    let mut bytes = Vec::new();
    append_len(&mut bytes, values.len())?;
    for value in values {
        append_string(&mut bytes, value.as_str())?;
    }
    Ok(bytes)
}

fn decode_ids(bytes: &[u8]) -> Result<Vec<SchemaId>, NeutralAnimationContentErrorV1> {
    let mut reader = Reader::new(bytes);
    let count = reader.len()?;
    if count > MAX_SKELETON_JOINTS_V1 {
        return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
    }
    let values = (0..count)
        .map(|_| Ok(SchemaId::new(reader.string()?)?))
        .collect::<Result<Vec<_>, NeutralAnimationContentErrorV1>>()?;
    reader.finish()?;
    Ok(values)
}

fn encode_joints(
    values: &[NeutralSkeletonJointV1],
) -> Result<Vec<u8>, NeutralAnimationContentErrorV1> {
    let mut bytes = Vec::new();
    append_len(&mut bytes, values.len())?;
    for joint in values {
        append_string(&mut bytes, joint.joint_key.as_str())?;
        bytes.push(u8::from(joint.parent_joint_key.is_some()));
        if let Some(parent) = &joint.parent_joint_key {
            append_string(&mut bytes, parent.as_str())?;
        }
        encode_transform(&mut bytes, joint.bind_transform);
        bytes.extend_from_slice(&encode_ids(&joint.semantic_roles)?);
    }
    Ok(bytes)
}

fn decode_joints(
    bytes: &[u8],
) -> Result<Vec<NeutralSkeletonJointV1>, NeutralAnimationContentErrorV1> {
    let mut reader = Reader::new(bytes);
    let count = reader.len()?;
    if count > MAX_SKELETON_JOINTS_V1 {
        return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
    }
    let mut joints = Vec::with_capacity(count);
    for _ in 0..count {
        let joint_key = SchemaId::new(reader.string()?)?;
        let parent_joint_key = match reader.u8()? {
            0 => None,
            1 => Some(SchemaId::new(reader.string()?)?),
            _ => return Err(NeutralAnimationContentErrorV1::InvalidValue),
        };
        let bind_transform = decode_transform(&mut reader)?;
        let role_count = reader.len()?;
        if role_count > MAX_JOINT_SEMANTIC_ROLES_V1 {
            return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
        }
        let mut semantic_roles = Vec::with_capacity(role_count);
        for _ in 0..role_count {
            semantic_roles.push(SchemaId::new(reader.string()?)?);
        }
        joints.push(NeutralSkeletonJointV1 {
            joint_key,
            parent_joint_key,
            bind_transform,
            semantic_roles,
        });
    }
    reader.finish()?;
    Ok(joints)
}

fn encode_transform(bytes: &mut Vec<u8>, value: NeutralTransformV1) {
    for item in value.translation_micrometres {
        bytes.extend_from_slice(&item.to_le_bytes());
    }
    for item in value.rotation_q1_30 {
        bytes.extend_from_slice(&item.to_le_bytes());
    }
    for item in value.scale_q16_16 {
        bytes.extend_from_slice(&item.to_le_bytes());
    }
}

fn decode_transform(
    reader: &mut Reader<'_>,
) -> Result<NeutralTransformV1, NeutralAnimationContentErrorV1> {
    Ok(NeutralTransformV1 {
        translation_micrometres: [reader.i64()?, reader.i64()?, reader.i64()?],
        rotation_q1_30: [reader.i32()?, reader.i32()?, reader.i32()?, reader.i32()?],
        scale_q16_16: [reader.u32()?, reader.u32()?, reader.u32()?],
    })
}

fn encode_channels(
    values: &[NeutralAnimationChannelV1],
) -> Result<Vec<u8>, NeutralAnimationContentErrorV1> {
    let mut bytes = Vec::new();
    append_len(&mut bytes, values.len())?;
    for value in values {
        append_string(&mut bytes, value.joint_key.as_str())?;
        bytes.extend_from_slice(&(value.property as u32).to_le_bytes());
        bytes.extend_from_slice(&(value.interpolation as u32).to_le_bytes());
        let keys = encode_keys(&value.keys)?;
        append_len(&mut bytes, keys.len())?;
        bytes.extend_from_slice(&keys);
    }
    Ok(bytes)
}

fn decode_channels(
    bytes: &[u8],
) -> Result<Vec<NeutralAnimationChannelV1>, NeutralAnimationContentErrorV1> {
    let mut reader = Reader::new(bytes);
    let count = reader.len()?;
    if count > MAX_ANIMATION_CHANNELS_V1 {
        return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let joint_key = SchemaId::new(reader.string()?)?;
        let property = AnimationPropertyV1::from_tag(reader.u32()?)?;
        let interpolation = AnimationInterpolationV1::from_tag(reader.u32()?)?;
        let payload_len = reader.len()?;
        let payload = reader.bytes(payload_len)?;
        values.push(NeutralAnimationChannelV1 {
            joint_key,
            property,
            interpolation,
            keys: decode_keys(payload)?,
        });
    }
    reader.finish()?;
    Ok(values)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NeutralAnimationContentErrorV1 {
    Canonical(CanonicalError),
    DecodeCanonical,
    Identifier(IdentifierError),
    EnvelopeMismatch,
    UnsupportedVersion,
    LimitOrRevision,
    DuplicateIdentity,
    MissingReference,
    Cycle,
    InvalidValue,
    UnknownTag,
    HashOrCanonicalMismatch,
    Decode,
}

impl Display for NeutralAnimationContentErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "neutral animation content invalid: {self:?}")
    }
}
impl Error for NeutralAnimationContentErrorV1 {}
impl From<CanonicalError> for NeutralAnimationContentErrorV1 {
    fn from(value: CanonicalError) -> Self {
        Self::Canonical(value)
    }
}
impl From<CanonicalDecodeError> for NeutralAnimationContentErrorV1 {
    fn from(_: CanonicalDecodeError) -> Self {
        Self::DecodeCanonical
    }
}
impl From<IdentifierError> for NeutralAnimationContentErrorV1 {
    fn from(value: IdentifierError) -> Self {
        Self::Identifier(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joint(key: &str, parent: Option<&str>) -> NeutralSkeletonJointV1 {
        NeutralSkeletonJointV1 {
            joint_key: SchemaId::new(key).expect("joint key"),
            parent_joint_key: parent.map(|value| SchemaId::new(value).expect("parent")),
            bind_transform: NeutralTransformV1::translated([0, 0, 0]),
            semantic_roles: Vec::new(),
        }
    }

    #[test]
    fn skeleton_and_clip_round_trip_with_exact_dependency() {
        let skeleton = NeutralSkeletonV1::new(
            AssetId::from_bytes([1; 16]),
            1,
            SchemaId::new("nextengine.coordinates.reference").expect("profile"),
            vec![SchemaId::new("nextengine.joint.root").expect("root")],
            vec![
                joint("nextengine.joint.child", Some("nextengine.joint.root")),
                joint("nextengine.joint.root", None),
            ],
        )
        .expect("skeleton");
        let decoded = NeutralSkeletonV1::from_canonical_bytes(
            &skeleton.canonical_bytes().expect("encode"),
            CanonicalDecodeLimits::default(),
        )
        .expect("decode");
        assert_eq!(decoded, skeleton);
        assert_eq!(
            decoded.joints[0].joint_key.as_str(),
            "nextengine.joint.root"
        );

        let clip = NeutralAnimationV1::new(
            AssetId::from_bytes([2; 16]),
            1,
            SchemaId::new("nextengine.animation.idle").expect("clip id"),
            skeleton.asset_revision().expect("revision"),
            1_000_000,
            AnimationWrapModeV1::Loop,
            vec![NeutralAnimationChannelV1 {
                joint_key: SchemaId::new("nextengine.joint.child").expect("joint"),
                property: AnimationPropertyV1::Translation,
                interpolation: AnimationInterpolationV1::Linear,
                keys: vec![
                    NeutralAnimationKeyV1 {
                        time_microseconds: 0,
                        value: NeutralAnimationValueV1::Translation([0, 0, 0]),
                    },
                    NeutralAnimationKeyV1 {
                        time_microseconds: 1_000_000,
                        value: NeutralAnimationValueV1::Translation([0, 1, 0]),
                    },
                ],
            }],
            Vec::new(),
            Vec::new(),
        )
        .expect("clip");
        assert_eq!(
            NeutralAnimationV1::from_canonical_bytes(
                &clip.canonical_bytes().expect("encode"),
                CanonicalDecodeLimits::default(),
            )
            .expect("decode"),
            clip
        );
    }

    #[test]
    fn skeleton_cycle_and_duplicate_clip_time_fail_closed() {
        let cyclic = NeutralSkeletonV1::new(
            AssetId::from_bytes([1; 16]),
            1,
            SchemaId::new("nextengine.coordinates.reference").expect("profile"),
            Vec::new(),
            vec![
                joint("nextengine.joint.a", Some("nextengine.joint.b")),
                joint("nextengine.joint.b", Some("nextengine.joint.a")),
            ],
        );
        assert_eq!(cyclic, Err(NeutralAnimationContentErrorV1::Cycle));
    }
}
