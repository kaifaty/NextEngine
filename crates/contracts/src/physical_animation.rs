//! Canonical state for the bounded R5a physical-animation owner.
//!
//! Physics remains the sole pose/contact authority. These records persist
//! only the graph state and phase needed to reconstruct a presentation pose
//! from committed physics plus immutable neutral animation content.

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::animation_content::{
    AnimationInterpolationV1, AnimationPropertyV1, NeutralAnimationContentErrorV1,
    NeutralAnimationV1, NeutralSkeletonV1,
};
use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_U16, CANONICAL_TYPE_U64,
    CANONICAL_TYPE_UTF8_NFC, CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits,
    CanonicalError, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{ContentHash, IdentifierError, PersistentId, SchemaId};
use crate::physics::PhysicsBodyIdV1;
use crate::project::{AssetRevisionRefV1, domain_hash};

pub const PHYSICAL_ANIMATION_SCHEMA_VERSION: u16 = 1;
pub const PHYSICAL_ANIMATION_PROFILE_OWNER_ID: &str = "nextengine.physical-embodiment";
pub const PHYSICAL_ANIMATION_PROFILE_SCHEMA_ID: &str = "nextengine.physical-animation-profile";
pub const PHYSICAL_ANIMATION_PROFILE_SEGMENT_ID: &str = "nextengine.physical-animation-profile.v1";
pub const PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID: &str = "nextengine.physical-embodiment";
pub const PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID: &str = "nextengine.physical-animation-snapshot";
pub const PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID: &str = "physical-animation";
pub const MAX_PHYSICAL_ANIMATION_INSTANCES_V1: usize = 64;
pub const MAX_PHYSICAL_ANIMATION_RETARGET_JOINTS_V1: usize = 1_024;
pub const MAX_PRESENTATION_ROOT_OFFSET_MICROMETRES_V1: u64 = 5_000_000;
pub const MAX_FOOT_IK_CORRECTION_MICROMETRES_V1: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicalAnimationGraphStateV1 {
    Idle = 1,
    Locomotion = 2,
}

impl PhysicalAnimationGraphStateV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicalAnimationContractErrorV1> {
        match tag {
            1 => Ok(Self::Idle),
            2 => Ok(Self::Locomotion),
            value => Err(PhysicalAnimationContractErrorV1::UnknownGraphState(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicalAnimationRetargetJointV1 {
    pub source_joint_key: SchemaId,
    pub target_joint_key: SchemaId,
}

/// One immutable engine-owned profile used by all instances of the bounded
/// R5a humanoid archetype.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub skeleton_revision: AssetRevisionRefV1,
    pub idle_clip_revision: AssetRevisionRefV1,
    pub locomotion_clip_revision: AssetRevisionRefV1,
    pub gameplay_hz: u32,
    pub locomotion_threshold_micrometres_per_tick: u64,
    pub presentation_root_offset_micrometres: [i64; 3],
    pub presentation_motion_joint_key: SchemaId,
    pub foot_joint_keys: [SchemaId; 2],
    pub max_foot_ik_correction_micrometres: u64,
    pub retarget_joints: Vec<PhysicalAnimationRetargetJointV1>,
}

impl PhysicalAnimationProfileV1 {
    pub fn validate(&self) -> Result<(), PhysicalAnimationContractErrorV1> {
        let revisions_valid = [
            self.skeleton_revision,
            self.idle_clip_revision,
            self.locomotion_clip_revision,
        ]
        .iter()
        .all(|revision| {
            revision.record_sha256 != ContentHash::default()
                && revision.asset_id != crate::ids::AssetId::default()
        });
        if self.schema_version != PHYSICAL_ANIMATION_SCHEMA_VERSION
            || !revisions_valid
            || self.idle_clip_revision == self.locomotion_clip_revision
            || !matches!(self.gameplay_hz, 20 | 30 | 60)
            || self.locomotion_threshold_micrometres_per_tick == 0
            || self.max_foot_ik_correction_micrometres == 0
            || self.max_foot_ik_correction_micrometres > MAX_FOOT_IK_CORRECTION_MICROMETRES_V1
            || self.foot_joint_keys[0] == self.foot_joint_keys[1]
            || self.retarget_joints.is_empty()
            || self.retarget_joints.len() > MAX_PHYSICAL_ANIMATION_RETARGET_JOINTS_V1
            || self
                .presentation_root_offset_micrometres
                .iter()
                .any(|value| value.unsigned_abs() > MAX_PRESENTATION_ROOT_OFFSET_MICROMETRES_V1)
            || self.retarget_joints.windows(2).any(|pair| {
                pair[0].source_joint_key >= pair[1].source_joint_key
                    || pair[0].target_joint_key >= pair[1].target_joint_key
            })
            || self
                .retarget_joints
                .iter()
                .any(|joint| joint.source_joint_key != joint.target_joint_key)
            || self
                .retarget_joints
                .binary_search_by(|joint| {
                    joint
                        .target_joint_key
                        .cmp(&self.presentation_motion_joint_key)
                })
                .is_err()
            || self.foot_joint_keys.iter().any(|foot| {
                self.retarget_joints
                    .binary_search_by(|joint| joint.target_joint_key.cmp(foot))
                    .is_err()
            })
        {
            return Err(PhysicalAnimationContractErrorV1::ProfileInvalid);
        }
        Ok(())
    }

    /// Closes the bounded R5a profile over exact neutral content. Non-identity
    /// retarget, cubic/morph channels and root motion are deliberately not
    /// admitted by this first production consumer.
    pub fn validate_against_content(
        &self,
        skeleton: &NeutralSkeletonV1,
        idle: &NeutralAnimationV1,
        locomotion: &NeutralAnimationV1,
    ) -> Result<(), PhysicalAnimationContractErrorV1> {
        self.validate()?;
        let mut skeleton_joint_keys = skeleton
            .joints
            .iter()
            .map(|joint| joint.joint_key.clone())
            .collect::<Vec<_>>();
        skeleton_joint_keys.sort();
        let mapped_joint_keys = self
            .retarget_joints
            .iter()
            .map(|joint| joint.source_joint_key.clone())
            .collect::<Vec<_>>();
        let supported_clip = |clip: &NeutralAnimationV1| {
            clip.skeleton_revision == self.skeleton_revision
                && clip.root_motion_intent.is_empty()
                && clip.channels.iter().all(|channel| {
                    channel.property == AnimationPropertyV1::Translation
                        && matches!(
                            channel.interpolation,
                            AnimationInterpolationV1::Step | AnimationInterpolationV1::Linear
                        )
                })
        };
        let supported_bind_pose = skeleton.joints.iter().all(|joint| {
            joint.bind_transform.rotation_q1_30 == [0, 0, 0, 1 << 30]
                && joint.bind_transform.scale_q16_16 == [1 << 16; 3]
        });
        if skeleton.asset_revision()? != self.skeleton_revision
            || idle.asset_revision()? != self.idle_clip_revision
            || locomotion.asset_revision()? != self.locomotion_clip_revision
            || idle.clip_id == locomotion.clip_id
            || !supported_clip(idle)
            || !supported_clip(locomotion)
            || !supported_bind_pose
            || mapped_joint_keys != skeleton_joint_keys
        {
            return Err(PhysicalAnimationContractErrorV1::ContentClosureInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, PhysicalAnimationContractErrorV1> {
        self.validate()?;
        let mut payload = Writer::default();
        payload.asset_revision(self.skeleton_revision);
        payload.asset_revision(self.idle_clip_revision);
        payload.asset_revision(self.locomotion_clip_revision);
        payload.u32(self.gameplay_hz);
        payload.u64(self.locomotion_threshold_micrometres_per_tick);
        for value in self.presentation_root_offset_micrometres {
            payload.i64(value);
        }
        payload.schema_id(&self.presentation_motion_joint_key)?;
        payload.schema_id(&self.foot_joint_keys[0])?;
        payload.schema_id(&self.foot_joint_keys[1])?;
        payload.u64(self.max_foot_ik_correction_micrometres);
        payload.count(self.retarget_joints.len())?;
        for joint in &self.retarget_joints {
            payload.schema_id(&joint.source_joint_key)?;
            payload.schema_id(&joint.target_joint_key)?;
        }
        Ok(encode_canonical_segment(
            PHYSICAL_ANIMATION_PROFILE_OWNER_ID,
            PHYSICAL_ANIMATION_PROFILE_SCHEMA_ID,
            PHYSICAL_ANIMATION_PROFILE_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.profile_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, payload.finish()),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicalAnimationContractErrorV1> {
        let fields = decode_contract(
            bytes,
            limits,
            PHYSICAL_ANIMATION_PROFILE_OWNER_ID,
            PHYSICAL_ANIMATION_PROFILE_SCHEMA_ID,
            PHYSICAL_ANIMATION_PROFILE_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let mut payload = Reader::new(field(&fields, 3)?, limits);
        let skeleton_revision = payload.asset_revision()?;
        let idle_clip_revision = payload.asset_revision()?;
        let locomotion_clip_revision = payload.asset_revision()?;
        let gameplay_hz = payload.u32()?;
        let locomotion_threshold_micrometres_per_tick = payload.u64()?;
        let presentation_root_offset_micrometres = [payload.i64()?, payload.i64()?, payload.i64()?];
        let presentation_motion_joint_key = payload.schema_id()?;
        let foot_joint_keys = [payload.schema_id()?, payload.schema_id()?];
        let max_foot_ik_correction_micrometres = payload.u64()?;
        let count = payload.count(MAX_PHYSICAL_ANIMATION_RETARGET_JOINTS_V1)?;
        let mut retarget_joints = Vec::with_capacity(count);
        for _ in 0..count {
            retarget_joints.push(PhysicalAnimationRetargetJointV1 {
                source_joint_key: payload.schema_id()?,
                target_joint_key: payload.schema_id()?,
            });
        }
        payload.finish()?;
        let value = Self {
            schema_version: read_u16(field(&fields, 1)?)?,
            profile_id: SchemaId::new(
                std::str::from_utf8(field(&fields, 2)?)
                    .map_err(|_| PhysicalAnimationContractErrorV1::ProfileInvalid)?,
            )?,
            skeleton_revision,
            idle_clip_revision,
            locomotion_clip_revision,
            gameplay_hz,
            locomotion_threshold_micrometres_per_tick,
            presentation_root_offset_micrometres,
            presentation_motion_joint_key,
            foot_joint_keys,
            max_foot_ik_correction_micrometres,
            retarget_joints,
        };
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(PhysicalAnimationContractErrorV1::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn revision(&self) -> Result<ContentHash, PhysicalAnimationContractErrorV1> {
        Ok(domain_hash(
            PHYSICAL_ANIMATION_PROFILE_SEGMENT_ID,
            &self.canonical_bytes()?,
        ))
    }

    #[must_use]
    pub const fn clip_revision_for_state(
        &self,
        state: PhysicalAnimationGraphStateV1,
    ) -> AssetRevisionRefV1 {
        match state {
            PhysicalAnimationGraphStateV1::Idle => self.idle_clip_revision,
            PhysicalAnimationGraphStateV1::Locomotion => self.locomotion_clip_revision,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicalAnimationBindingV1 {
    pub subject_id: PersistentId,
    pub body_id: PhysicsBodyIdV1,
}

impl PhysicalAnimationBindingV1 {
    pub fn validate(self) -> Result<(), PhysicalAnimationContractErrorV1> {
        if self.subject_id == PersistentId::default() || self.body_id.subject_id != self.subject_id
        {
            return Err(PhysicalAnimationContractErrorV1::BindingInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicalAnimationRecordV1 {
    pub subject_id: PersistentId,
    pub body_id: PhysicsBodyIdV1,
    pub graph_state: PhysicalAnimationGraphStateV1,
    pub phase_ticks: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationSnapshotV1 {
    pub schema_version: u16,
    pub profile_revision: ContentHash,
    pub next_simulation_tick: u64,
    pub records: Vec<PhysicalAnimationRecordV1>,
}

impl PhysicalAnimationSnapshotV1 {
    pub fn initial(
        profile: &PhysicalAnimationProfileV1,
        bindings: &[PhysicalAnimationBindingV1],
    ) -> Result<Self, PhysicalAnimationContractErrorV1> {
        validate_bindings(bindings)?;
        let value = Self {
            schema_version: PHYSICAL_ANIMATION_SCHEMA_VERSION,
            profile_revision: profile.revision()?,
            next_simulation_tick: 0,
            records: bindings
                .iter()
                .map(|binding| PhysicalAnimationRecordV1 {
                    subject_id: binding.subject_id,
                    body_id: binding.body_id,
                    graph_state: PhysicalAnimationGraphStateV1::Idle,
                    phase_ticks: 0,
                })
                .collect(),
        };
        value.validate_against(profile, bindings)?;
        Ok(value)
    }

    pub fn validate_structure(&self) -> Result<(), PhysicalAnimationContractErrorV1> {
        if self.schema_version != PHYSICAL_ANIMATION_SCHEMA_VERSION
            || self.profile_revision == ContentHash::default()
            || self.records.is_empty()
            || self.records.len() > MAX_PHYSICAL_ANIMATION_INSTANCES_V1
            || self.records.windows(2).any(|pair| {
                (pair[0].subject_id, pair[0].body_id) >= (pair[1].subject_id, pair[1].body_id)
            })
            || self.records.iter().any(|record| {
                record.subject_id == PersistentId::default()
                    || record.body_id.subject_id != record.subject_id
                    || record.phase_ticks > self.next_simulation_tick
            })
        {
            return Err(PhysicalAnimationContractErrorV1::SnapshotClosureInvalid);
        }
        Ok(())
    }

    pub fn validate_against(
        &self,
        profile: &PhysicalAnimationProfileV1,
        bindings: &[PhysicalAnimationBindingV1],
    ) -> Result<(), PhysicalAnimationContractErrorV1> {
        self.validate_structure()?;
        validate_bindings(bindings)?;
        if self.profile_revision != profile.revision()?
            || self.records.len() != bindings.len()
            || self.records.iter().zip(bindings).any(|(record, binding)| {
                record.subject_id != binding.subject_id || record.body_id != binding.body_id
            })
        {
            return Err(PhysicalAnimationContractErrorV1::SnapshotClosureInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, PhysicalAnimationContractErrorV1> {
        self.validate_structure()?;
        let mut records = Writer::default();
        records.count(self.records.len())?;
        for record in &self.records {
            records.id(record.subject_id);
            records.id(record.body_id.subject_id);
            records.u32(record.body_id.body_slot);
            records.u8(record.graph_state as u8);
            records.u64(record.phase_ticks);
        }
        Ok(encode_canonical_segment(
            PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID,
            PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID,
            PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                field_hash(2, self.profile_revision),
                field_u64(3, self.next_simulation_tick),
                CanonicalField::new(4, CANONICAL_TYPE_BYTES, records.finish()),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicalAnimationContractErrorV1> {
        let fields = decode_contract(
            bytes,
            limits,
            PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID,
            PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID,
            PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_HASH256),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let mut records_reader = Reader::new(field(&fields, 4)?, limits);
        let count = records_reader.count(MAX_PHYSICAL_ANIMATION_INSTANCES_V1)?;
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            let subject_id = records_reader.id()?;
            records.push(PhysicalAnimationRecordV1 {
                subject_id,
                body_id: PhysicsBodyIdV1 {
                    subject_id: records_reader.id()?,
                    body_slot: records_reader.u32()?,
                },
                graph_state: PhysicalAnimationGraphStateV1::from_tag(records_reader.u8()?)?,
                phase_ticks: records_reader.u64()?,
            });
        }
        records_reader.finish()?;
        let value = Self {
            schema_version: read_u16(field(&fields, 1)?)?,
            profile_revision: ContentHash::from_bytes(read_exact(field(&fields, 2)?)?),
            next_simulation_tick: read_u64(field(&fields, 3)?)?,
            records,
        };
        value.validate_structure()?;
        if value.canonical_bytes()? != bytes {
            return Err(PhysicalAnimationContractErrorV1::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

fn validate_bindings(
    bindings: &[PhysicalAnimationBindingV1],
) -> Result<(), PhysicalAnimationContractErrorV1> {
    if bindings.is_empty()
        || bindings.len() > MAX_PHYSICAL_ANIMATION_INSTANCES_V1
        || bindings.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(PhysicalAnimationContractErrorV1::BindingInvalid);
    }
    for binding in bindings {
        binding.validate()?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicalAnimationContractErrorV1 {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    Content(NeutralAnimationContentErrorV1),
    UnsupportedVersion(u16),
    WrongEnvelope,
    FieldSetInvalid,
    FieldLength,
    ProfileInvalid,
    ContentClosureInvalid,
    BindingInvalid,
    SnapshotClosureInvalid,
    UnknownGraphState(u8),
    NonCanonicalEncoding,
}

impl PhysicalAnimationContractErrorV1 {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Canonical(_) | Self::Decode(_) | Self::FieldLength => {
                "PHYSICAL_ANIMATION_CANONICAL_INVALID"
            }
            Self::Identifier(_) | Self::ProfileInvalid => "PHYSICAL_ANIMATION_PROFILE_INVALID",
            Self::Content(_) | Self::ContentClosureInvalid => {
                "PHYSICAL_ANIMATION_CONTENT_CLOSURE_INVALID"
            }
            Self::UnsupportedVersion(_) => "UNSUPPORTED_PHYSICAL_ANIMATION_VERSION",
            Self::WrongEnvelope | Self::FieldSetInvalid | Self::NonCanonicalEncoding => {
                "PHYSICAL_ANIMATION_CANONICAL_INVALID"
            }
            Self::BindingInvalid => "PHYSICAL_ANIMATION_BINDING_INVALID",
            Self::SnapshotClosureInvalid | Self::UnknownGraphState(_) => {
                "PHYSICAL_ANIMATION_SNAPSHOT_CLOSURE_INVALID"
            }
        }
    }
}

impl Display for PhysicalAnimationContractErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for PhysicalAnimationContractErrorV1 {}

impl From<CanonicalError> for PhysicalAnimationContractErrorV1 {
    fn from(value: CanonicalError) -> Self {
        Self::Canonical(value)
    }
}

impl From<CanonicalDecodeError> for PhysicalAnimationContractErrorV1 {
    fn from(value: CanonicalDecodeError) -> Self {
        Self::Decode(value)
    }
}

impl From<IdentifierError> for PhysicalAnimationContractErrorV1 {
    fn from(value: IdentifierError) -> Self {
        Self::Identifier(value)
    }
}

impl From<NeutralAnimationContentErrorV1> for PhysicalAnimationContractErrorV1 {
    fn from(value: NeutralAnimationContentErrorV1) -> Self {
        Self::Content(value)
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn id(&mut self, value: PersistentId) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn asset_revision(&mut self, value: AssetRevisionRefV1) {
        self.bytes.extend_from_slice(value.asset_id.as_bytes());
        self.bytes.extend_from_slice(value.record_sha256.as_bytes());
    }

    fn schema_id(&mut self, value: &SchemaId) -> Result<(), PhysicalAnimationContractErrorV1> {
        self.bytes.extend_from_slice(
            &u32::try_from(value.as_str().len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        self.bytes.extend_from_slice(value.as_str().as_bytes());
        Ok(())
    }

    fn count(&mut self, value: usize) -> Result<(), PhysicalAnimationContractErrorV1> {
        self.u32(u32::try_from(value).map_err(|_| CanonicalError::LengthOverflow)?);
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

struct Reader<'a> {
    cursor: CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], limits: CanonicalDecodeLimits) -> Self {
        Self {
            cursor: CanonicalCursor::new(bytes),
            limits,
        }
    }

    fn u8(&mut self) -> Result<u8, PhysicalAnimationContractErrorV1> {
        Ok(self.cursor.read_u8()?)
    }

    fn u32(&mut self) -> Result<u32, PhysicalAnimationContractErrorV1> {
        Ok(self.cursor.read_u32()?)
    }

    fn u64(&mut self) -> Result<u64, PhysicalAnimationContractErrorV1> {
        Ok(self.cursor.read_u64()?)
    }

    fn i64(&mut self) -> Result<i64, PhysicalAnimationContractErrorV1> {
        Ok(i64::from_le_bytes(read_array(&mut self.cursor)?))
    }

    fn id(&mut self) -> Result<PersistentId, PhysicalAnimationContractErrorV1> {
        Ok(PersistentId::from_bytes(read_array(&mut self.cursor)?))
    }

    fn asset_revision(&mut self) -> Result<AssetRevisionRefV1, PhysicalAnimationContractErrorV1> {
        Ok(AssetRevisionRefV1 {
            asset_id: crate::ids::AssetId::from_bytes(read_array(&mut self.cursor)?),
            record_sha256: ContentHash::from_bytes(read_array(&mut self.cursor)?),
        })
    }

    fn schema_id(&mut self) -> Result<SchemaId, PhysicalAnimationContractErrorV1> {
        let bytes = self
            .cursor
            .read_u32_length_prefixed(self.limits.max_field_payload_bytes)?;
        Ok(SchemaId::new(std::str::from_utf8(bytes).map_err(
            |_| PhysicalAnimationContractErrorV1::ProfileInvalid,
        )?)?)
    }

    fn count(&mut self, maximum: usize) -> Result<usize, PhysicalAnimationContractErrorV1> {
        let value = usize::try_from(self.u32()?)
            .map_err(|_| PhysicalAnimationContractErrorV1::FieldLength)?;
        if value == 0 || value > maximum {
            return Err(PhysicalAnimationContractErrorV1::FieldLength);
        }
        Ok(value)
    }

    fn finish(self) -> Result<(), PhysicalAnimationContractErrorV1> {
        Ok(self.cursor.finish()?)
    }
}

fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment: &str,
    expected: &[(u32, u8)],
) -> Result<Vec<CanonicalField>, PhysicalAnimationContractErrorV1> {
    let decoded = decode_canonical_segment(bytes, limits)?;
    if decoded.owner_id != owner || decoded.schema_id != schema || decoded.segment_id != segment {
        return Err(PhysicalAnimationContractErrorV1::WrongEnvelope);
    }
    if decoded.fields.len() != expected.len()
        || decoded
            .fields
            .iter()
            .zip(expected)
            .any(|(actual, expected)| (actual.field_id, actual.type_tag) != *expected)
    {
        return Err(PhysicalAnimationContractErrorV1::FieldSetInvalid);
    }
    Ok(decoded.fields)
}

fn field(fields: &[CanonicalField], id: u32) -> Result<&[u8], PhysicalAnimationContractErrorV1> {
    Ok(&fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(PhysicalAnimationContractErrorV1::FieldSetInvalid)?
        .payload)
}

fn field_u16(id: u32, value: u16) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U16, value.to_le_bytes().to_vec())
}

fn field_u64(id: u32, value: u64) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U64, value.to_le_bytes().to_vec())
}

fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

fn read_u16(bytes: &[u8]) -> Result<u16, PhysicalAnimationContractErrorV1> {
    Ok(u16::from_le_bytes(read_exact(bytes)?))
}

fn read_u64(bytes: &[u8]) -> Result<u64, PhysicalAnimationContractErrorV1> {
    Ok(u64::from_le_bytes(read_exact(bytes)?))
}

fn read_exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], PhysicalAnimationContractErrorV1> {
    bytes
        .try_into()
        .map_err(|_| PhysicalAnimationContractErrorV1::FieldLength)
}

fn read_array<const N: usize>(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<[u8; N], PhysicalAnimationContractErrorV1> {
    cursor
        .read_exact(N)?
        .try_into()
        .map_err(|_| PhysicalAnimationContractErrorV1::FieldLength)
}

#[cfg(test)]
mod tests;
