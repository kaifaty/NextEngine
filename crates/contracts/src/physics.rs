use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I64,
    CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_STRUCT,
    CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalCursor, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, DecodedCanonicalSegment,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::{
    AssetId, CommandId, ContentHash, PersistentId, PhysicsContactId, PhysicsWorldId, SchemaId,
    TickRateProfileV1, content_hash_from_bytes,
};

pub const PHYSICAL_COMMAND_SCHEMA_ID: &str = "nextengine.command.physical";
pub const PHYSICAL_COMMAND_CAPABILITY_ID: &str = "nextengine.capability.physical-avatar-intent";
pub const PHYSICAL_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const PHYSICS_SNAPSHOT_SCHEMA_VERSION: u16 = 2;
pub const LEGACY_PHYSICS_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
pub const AUTHORITATIVE_NUMERIC_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_QUANTIZATION_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND: i64 = 3_000_000;
pub const PHYSICS_SWEEP_DISTANCE_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.sweep-distance";
pub const PHYSICS_CONTACT_NORMAL_X_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.contact-normal-x";
pub const PHYSICS_CONTACT_NORMAL_Y_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.contact-normal-y";
pub const PHYSICS_CONTACT_NORMAL_Z_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.contact-normal-z";
pub const PHYSICS_METRES_UNIT_ID: &str = "nextengine.unit.metre";
pub const PHYSICS_MICROMETRES_UNIT_ID: &str = "nextengine.unit.micrometre";
pub const PHYSICS_SCALAR_UNIT_ID: &str = "nextengine.unit.scalar";
pub const PHYSICS_MICROMETRES_FIXED_POINT_ID: &str = "nextengine.fixed.physics-micrometres-i64";
pub const PHYSICS_Q1_30_FIXED_POINT_ID: &str = "nextengine.fixed.q1-30";

pub const PHYSICS_SNAPSHOT_OWNER_ID: &str = "nextengine.physics";
pub const PHYSICS_SNAPSHOT_SCHEMA_ID: &str = "nextengine.physics-canonical-snapshot";
pub const PHYSICS_SNAPSHOT_SEGMENT_ID: &str = "v2";
pub const LEGACY_PHYSICS_SNAPSHOT_SEGMENT_ID: &str = "v1";
pub const PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID: &str = "nextengine.physics-world-checkpoint";
pub const PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID: &str = "v1";
pub const PHYSICS_STEP_INPUT_SCHEMA_VERSION: u16 = 2;
pub const CLOSED_PHYSICS_CONTACT_BATCH_SCHEMA_VERSION: u16 = 1;
pub const REFERENCE_GRAVITY_MICROMETRES_PER_SECOND_SQUARED: i64 = -9_792_000;
const PHYSICS_OWNER_ID: &str = "nextengine.physics";
const NUMERIC_OWNER_ID: &str = "nextengine.runtime";
const NUMERIC_PROFILE_SCHEMA_ID: &str = "nextengine.authoritative-numeric-profile";
const QUANTIZATION_PROFILE_SCHEMA_ID: &str = "nextengine.physics-quantization-profile";
const PHYSICAL_EVENT_SCHEMA_ID: &str = "nextengine.event.capsule-step-applied";
const SEGMENT_V1: &str = "v1";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicalCommandV1 {
    SetCapsuleLocomotionIntent { direction_q15: [i16; 2] },
}

impl PhysicalCommandV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        match self {
            Self::SetCapsuleLocomotionIntent { direction_q15 } => {
                let [x, z] = *direction_q15;
                if !matches!(
                    (x, z),
                    (0, 0) | (32_767, 0) | (-32_767, 0) | (0, 32_767) | (0, -32_767)
                ) {
                    return Err(PhysicsContractError::DirectionOutOfProfile);
                }
            }
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut tagged = vec![1];
        let mut direction = Vec::with_capacity(4);
        let Self::SetCapsuleLocomotionIntent { direction_q15 } = self;
        direction.extend_from_slice(&direction_q15[0].to_le_bytes());
        direction.extend_from_slice(&direction_q15[1].to_le_bytes());
        tagged.extend_from_slice(&nested(CANONICAL_TYPE_BYTES, &direction)?);
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            PHYSICAL_COMMAND_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, 1),
                CanonicalField::new(2, CANONICAL_TYPE_TAGGED_UNION, tagged),
            ],
        )
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            PHYSICAL_COMMAND_SCHEMA_ID,
            SEGMENT_V1,
            &[(1, CANONICAL_TYPE_U16), (2, CANONICAL_TYPE_TAGGED_UNION)],
        )?;
        if read_u16(&segment, 1)? != 1 {
            return Err(PhysicsContractError::UnsupportedVersion(1));
        }
        let mut cursor = CanonicalCursor::new(field(&segment, 2)?);
        let tag = cursor.read_u8()?;
        let (nested_tag, payload) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        if tag != 1 || nested_tag != CANONICAL_TYPE_BYTES || payload.len() != 4 {
            return Err(PhysicsContractError::UnknownTag(tag));
        }
        let value = Self::SetCapsuleLocomotionIntent {
            direction_q15: [
                i16::from_le_bytes(exact(&payload[..2])?),
                i16::from_le_bytes(exact(&payload[2..])?),
            ],
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_payload_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsPoseV1 {
    pub translation_micrometres: [i64; 3],
    pub rotation_q1_30: [i32; 4],
}

impl Default for PhysicsPoseV1 {
    fn default() -> Self {
        Self {
            translation_micrometres: [0; 3],
            rotation_q1_30: [0, 0, 0, 1 << 30],
        }
    }
}

impl PhysicsPoseV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        let norm = self.rotation_q1_30.iter().try_fold(0_i128, |sum, value| {
            let value = i128::from(*value);
            sum.checked_add(value * value)
        });
        if norm != Some(1_i128 << 60) {
            return Err(PhysicsContractError::InvalidRotation);
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut translation = Vec::with_capacity(24);
        for value in self.translation_micrometres {
            translation.extend_from_slice(&value.to_le_bytes());
        }
        let mut rotation = Vec::with_capacity(16);
        for value in self.rotation_q1_30 {
            rotation.extend_from_slice(&value.to_le_bytes());
        }
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_BYTES, translation),
            CanonicalField::new(2, CANONICAL_TYPE_BYTES, rotation),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[(1, CANONICAL_TYPE_BYTES), (2, CANONICAL_TYPE_BYTES)],
        )?;
        let translation = &field_from(&fields, 1)?.payload;
        let rotation = &field_from(&fields, 2)?.payload;
        if translation.len() != 24 || rotation.len() != 16 {
            return Err(PhysicsContractError::FieldLength);
        }
        let value = Self {
            translation_micrometres: [
                i64::from_le_bytes(exact(&translation[0..8])?),
                i64::from_le_bytes(exact(&translation[8..16])?),
                i64::from_le_bytes(exact(&translation[16..24])?),
            ],
            rotation_q1_30: [
                i32::from_le_bytes(exact(&rotation[0..4])?),
                i32::from_le_bytes(exact(&rotation[4..8])?),
                i32::from_le_bytes(exact(&rotation[8..12])?),
                i32::from_le_bytes(exact(&rotation[12..16])?),
            ],
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FixedPointDescriptorV1 {
    pub descriptor_id: SchemaId,
    pub storage_bits: u8,
    pub signed: bool,
    pub fractional_bits: u8,
    pub minimum_raw: i64,
    pub maximum_raw: i64,
    pub rounding: u8,
    pub overflow: u8,
}

impl FixedPointDescriptorV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if !matches!(self.storage_bits, 16 | 32 | 64)
            || self.fractional_bits >= self.storage_bits
            || self.minimum_raw > self.maximum_raw
            || (!self.signed && self.storage_bits == 64)
            || self.rounding != 1
            || self.overflow != 1
            || !self.bounds_fit_storage()
        {
            return Err(PhysicsContractError::InvalidProfile);
        }
        Ok(())
    }

    fn bounds_fit_storage(&self) -> bool {
        if self.signed {
            let maximum = if self.storage_bits == 64 {
                i128::from(i64::MAX)
            } else {
                (1_i128 << (self.storage_bits - 1)) - 1
            };
            let minimum = if self.storage_bits == 64 {
                i128::from(i64::MIN)
            } else {
                -(1_i128 << (self.storage_bits - 1))
            };
            i128::from(self.minimum_raw) >= minimum && i128::from(self.maximum_raw) <= maximum
        } else {
            let maximum = (1_i128 << self.storage_bits) - 1;
            self.minimum_raw >= 0 && i128::from(self.maximum_raw) <= maximum
        }
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.descriptor_id.as_str().as_bytes().to_vec(),
            ),
            field_u8(2, self.storage_bits),
            field_bool(3, self.signed),
            field_u8(4, self.fractional_bits),
            field_i64(5, self.minimum_raw),
            field_i64(6, self.maximum_raw),
            field_u8(7, self.rounding),
            field_u8(8, self.overflow),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_BOOL),
                (4, CANONICAL_TYPE_U8),
                (5, CANONICAL_TYPE_I64),
                (6, CANONICAL_TYPE_I64),
                (7, CANONICAL_TYPE_U8),
                (8, CANONICAL_TYPE_U8),
            ],
        )?;
        let value = Self {
            descriptor_id: SchemaId::new(read_utf8_fields(&fields, 1)?)?,
            storage_bits: read_u8_fields(&fields, 2)?,
            signed: read_bool_fields(&fields, 3)?,
            fractional_bits: read_u8_fields(&fields, 4)?,
            minimum_raw: read_i64_fields(&fields, 5)?,
            maximum_raw: read_i64_fields(&fields, 6)?,
            rounding: read_u8_fields(&fields, 7)?,
            overflow: read_u8_fields(&fields, 8)?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicsSourceFormatV1 {
    Ieee754Binary32,
    Ieee754Binary64,
}

impl PhysicsSourceFormatV1 {
    const fn tag(self) -> u8 {
        match self {
            Self::Ieee754Binary32 => 1,
            Self::Ieee754Binary64 => 2,
        }
    }

    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Ieee754Binary32),
            2 => Ok(Self::Ieee754Binary64),
            _ => Err(PhysicsContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsQuantizationRuleV1 {
    pub field_id: SchemaId,
    pub source_format: PhysicsSourceFormatV1,
    pub source_unit: SchemaId,
    pub destination_unit: SchemaId,
    pub destination_fixed_point: SchemaId,
    pub scale_numerator: i64,
    pub scale_denominator: u64,
    pub offset_raw: i64,
    pub minimum_raw: i64,
    pub maximum_raw: i64,
}

impl PhysicsQuantizationRuleV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.scale_numerator == 0
            || self.scale_denominator == 0
            || self.minimum_raw > self.maximum_raw
        {
            return Err(PhysicsContractError::InvalidProfile);
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.field_id.as_str().as_bytes().to_vec(),
            ),
            field_u8(2, self.source_format.tag()),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_UTF8_NFC,
                self.source_unit.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_UTF8_NFC,
                self.destination_unit.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_UTF8_NFC,
                self.destination_fixed_point.as_str().as_bytes().to_vec(),
            ),
            field_i64(6, self.scale_numerator),
            field_u64(7, self.scale_denominator),
            field_i64(8, self.offset_raw),
            field_i64(9, self.minimum_raw),
            field_i64(10, self.maximum_raw),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_UTF8_NFC),
                (4, CANONICAL_TYPE_UTF8_NFC),
                (5, CANONICAL_TYPE_UTF8_NFC),
                (6, CANONICAL_TYPE_I64),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_I64),
                (9, CANONICAL_TYPE_I64),
                (10, CANONICAL_TYPE_I64),
            ],
        )?;
        let value = Self {
            field_id: SchemaId::new(read_utf8_fields(&fields, 1)?)?,
            source_format: PhysicsSourceFormatV1::from_tag(read_u8_fields(&fields, 2)?)?,
            source_unit: SchemaId::new(read_utf8_fields(&fields, 3)?)?,
            destination_unit: SchemaId::new(read_utf8_fields(&fields, 4)?)?,
            destination_fixed_point: SchemaId::new(read_utf8_fields(&fields, 5)?)?,
            scale_numerator: read_i64_fields(&fields, 6)?,
            scale_denominator: read_u64_fields(&fields, 7)?,
            offset_raw: read_i64_fields(&fields, 8)?,
            minimum_raw: read_i64_fields(&fields, 9)?,
            maximum_raw: read_i64_fields(&fields, 10)?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsQuantizationProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub rules: BTreeMap<SchemaId, PhysicsQuantizationRuleV1>,
}

impl PhysicsQuantizationProfileV1 {
    pub fn capsule_reference_v1() -> Result<Self, crate::IdentifierError> {
        Ok(Self {
            schema_version: PHYSICS_QUANTIZATION_PROFILE_SCHEMA_VERSION,
            profile_id: SchemaId::new("nextengine.physics.quantization.capsule-reference-v1")?,
            rules: BTreeMap::new(),
        })
    }

    pub fn grounded_capsule_v2() -> Result<Self, crate::IdentifierError> {
        let distance = PhysicsQuantizationRuleV1 {
            field_id: SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID)?,
            source_format: PhysicsSourceFormatV1::Ieee754Binary32,
            source_unit: SchemaId::new(PHYSICS_METRES_UNIT_ID)?,
            destination_unit: SchemaId::new(PHYSICS_MICROMETRES_UNIT_ID)?,
            destination_fixed_point: SchemaId::new(PHYSICS_MICROMETRES_FIXED_POINT_ID)?,
            scale_numerator: 1_000_000,
            scale_denominator: 1,
            offset_raw: 0,
            minimum_raw: 0,
            maximum_raw: 8_388_608_000_000,
        };
        let normal_rule = |field_id| -> Result<_, crate::IdentifierError> {
            Ok(PhysicsQuantizationRuleV1 {
                field_id: SchemaId::new(field_id)?,
                source_format: PhysicsSourceFormatV1::Ieee754Binary32,
                source_unit: SchemaId::new(PHYSICS_SCALAR_UNIT_ID)?,
                destination_unit: SchemaId::new(PHYSICS_SCALAR_UNIT_ID)?,
                destination_fixed_point: SchemaId::new(PHYSICS_Q1_30_FIXED_POINT_ID)?,
                scale_numerator: 1,
                scale_denominator: 1,
                offset_raw: 0,
                minimum_raw: -(1_i64 << 30),
                maximum_raw: 1_i64 << 30,
            })
        };
        let normal_x = normal_rule(PHYSICS_CONTACT_NORMAL_X_FIELD_ID)?;
        let normal_y = normal_rule(PHYSICS_CONTACT_NORMAL_Y_FIELD_ID)?;
        let normal_z = normal_rule(PHYSICS_CONTACT_NORMAL_Z_FIELD_ID)?;
        Ok(Self {
            schema_version: PHYSICS_QUANTIZATION_PROFILE_SCHEMA_VERSION,
            profile_id: SchemaId::new("nextengine.physics.quantization.grounded-capsule-v2")?,
            rules: BTreeMap::from([
                (distance.field_id.clone(), distance),
                (normal_x.field_id.clone(), normal_x),
                (normal_y.field_id.clone(), normal_y),
                (normal_z.field_id.clone(), normal_z),
            ]),
        })
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_QUANTIZATION_PROFILE_SCHEMA_VERSION {
            return Err(PhysicsContractError::InvalidProfile);
        }
        let reference_id = "nextengine.physics.quantization.capsule-reference-v1";
        if self.profile_id.as_str() == reference_id {
            if !self.rules.is_empty() {
                return Err(PhysicsContractError::InvalidProfile);
            }
            return Ok(());
        }
        if self.rules.is_empty() {
            return Err(PhysicsContractError::InvalidProfile);
        }
        for (id, rule) in &self.rules {
            if id != &rule.field_id {
                return Err(PhysicsContractError::DuplicateKey);
            }
            rule.validate()?;
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let rules = self
            .rules
            .iter()
            .map(|(id, rule)| {
                encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_UTF8_NFC,
                        id.as_str().as_bytes().to_vec(),
                    ),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, rule.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            QUANTIZATION_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.profile_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_sequence(rules)?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            QUANTIZATION_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut rules = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 3)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_UTF8_NFC), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let id = SchemaId::new(read_utf8_fields(&fields, 1)?)?;
            let rule =
                PhysicsQuantizationRuleV1::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if rules.insert(id, rule).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            profile_id: SchemaId::new(read_utf8(&segment, 2)?)?,
            rules,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        profile_hash(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeNumericProfileV1 {
    pub schema_version: u16,
    pub integer_overflow: u8,
    pub fixed_points: BTreeMap<SchemaId, FixedPointDescriptorV1>,
    pub authoritative_float_reduction: bool,
    pub fast_math_allowed: bool,
    pub physics_quantization_profile_hash: ContentHash,
}

impl AuthoritativeNumericProfileV1 {
    pub fn capsule_reference_v1(
        quantization: &PhysicsQuantizationProfileV1,
    ) -> Result<Self, CanonicalError> {
        let descriptor = FixedPointDescriptorV1 {
            descriptor_id: SchemaId::new("nextengine.fixed.q1-30")?,
            storage_bits: 32,
            signed: true,
            fractional_bits: 30,
            minimum_raw: -(1_i64 << 30),
            maximum_raw: 1_i64 << 30,
            rounding: 1,
            overflow: 1,
        };
        Ok(Self {
            schema_version: AUTHORITATIVE_NUMERIC_PROFILE_SCHEMA_VERSION,
            integer_overflow: 1,
            fixed_points: BTreeMap::from([(descriptor.descriptor_id.clone(), descriptor)]),
            authoritative_float_reduction: false,
            fast_math_allowed: false,
            physics_quantization_profile_hash: quantization.profile_hash()?,
        })
    }

    pub fn grounded_capsule_v2(
        quantization: &PhysicsQuantizationProfileV1,
    ) -> Result<Self, CanonicalError> {
        let q1_30 = FixedPointDescriptorV1 {
            descriptor_id: SchemaId::new(PHYSICS_Q1_30_FIXED_POINT_ID)?,
            storage_bits: 32,
            signed: true,
            fractional_bits: 30,
            minimum_raw: -(1_i64 << 30),
            maximum_raw: 1_i64 << 30,
            rounding: 1,
            overflow: 1,
        };
        let micrometres = FixedPointDescriptorV1 {
            descriptor_id: SchemaId::new(PHYSICS_MICROMETRES_FIXED_POINT_ID)?,
            storage_bits: 64,
            signed: true,
            fractional_bits: 0,
            minimum_raw: -8_388_608_000_000,
            maximum_raw: 8_388_608_000_000,
            rounding: 1,
            overflow: 1,
        };
        Ok(Self {
            schema_version: AUTHORITATIVE_NUMERIC_PROFILE_SCHEMA_VERSION,
            integer_overflow: 1,
            fixed_points: BTreeMap::from([
                (q1_30.descriptor_id.clone(), q1_30),
                (micrometres.descriptor_id.clone(), micrometres),
            ]),
            authoritative_float_reduction: false,
            fast_math_allowed: false,
            physics_quantization_profile_hash: quantization.profile_hash()?,
        })
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != AUTHORITATIVE_NUMERIC_PROFILE_SCHEMA_VERSION
            || self.integer_overflow != 1
            || self.authoritative_float_reduction
            || self.fast_math_allowed
        {
            return Err(PhysicsContractError::InvalidProfile);
        }
        for (id, descriptor) in &self.fixed_points {
            if id != &descriptor.descriptor_id {
                return Err(PhysicsContractError::DuplicateKey);
            }
            descriptor.validate()?;
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let entries = self
            .fixed_points
            .iter()
            .map(|(id, descriptor)| {
                encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_UTF8_NFC,
                        id.as_str().as_bytes().to_vec(),
                    ),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, descriptor.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            NUMERIC_OWNER_ID,
            NUMERIC_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u8(2, self.integer_overflow),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_sequence(entries)?),
                field_bool(4, self.authoritative_float_reduction),
                field_bool(5, self.fast_math_allowed),
                field_hash(6, self.physics_quantization_profile_hash),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            NUMERIC_OWNER_ID,
            NUMERIC_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_MAP),
                (4, CANONICAL_TYPE_BOOL),
                (5, CANONICAL_TYPE_BOOL),
                (6, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let mut fixed_points = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 3)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_UTF8_NFC), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let id = SchemaId::new(read_utf8_fields(&fields, 1)?)?;
            let descriptor =
                FixedPointDescriptorV1::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if fixed_points.insert(id, descriptor).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            integer_overflow: read_u8(&segment, 2)?,
            fixed_points,
            authoritative_float_reduction: read_bool(&segment, 4)?,
            fast_math_allowed: read_bool(&segment, 5)?,
            physics_quantization_profile_hash: read_hash(&segment, 6)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        profile_hash(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsBodyIdV1 {
    pub subject_id: PersistentId,
    pub body_slot: u32,
}

impl PhysicsBodyIdV1 {
    #[must_use]
    pub fn canonical_key_bytes(self) -> [u8; 20] {
        let mut bytes = [0; 20];
        bytes[..16].copy_from_slice(self.subject_id.as_bytes());
        bytes[16..].copy_from_slice(&self.body_slot.to_le_bytes());
        bytes
    }

    fn canonical_record(self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_id(1, self.subject_id.as_bytes()),
            field_u32(2, self.body_slot),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_U32)],
        )?;
        Ok(Self {
            subject_id: PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            body_slot: u32::from_le_bytes(exact(&field_from(&fields, 2)?.payload)?),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsShapeIdV1 {
    pub body_id: PhysicsBodyIdV1,
    pub shape_slot: u32,
}

impl PhysicsShapeIdV1 {
    #[must_use]
    pub fn canonical_key_bytes(self) -> [u8; 24] {
        let mut bytes = [0; 24];
        bytes[..20].copy_from_slice(&self.body_id.canonical_key_bytes());
        bytes[20..].copy_from_slice(&self.shape_slot.to_le_bytes());
        bytes
    }

    fn canonical_record(self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            field_u32(2, self.shape_slot),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[(1, CANONICAL_TYPE_STRUCT), (2, CANONICAL_TYPE_U32)],
        )?;
        Ok(Self {
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?,
            shape_slot: u32::from_le_bytes(exact(&field_from(&fields, 2)?.payload)?),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsMotionKindV1 {
    Static = 1,
    Kinematic = 2,
    Dynamic = 3,
}

impl PhysicsMotionKindV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Static),
            2 => Ok(Self::Kinematic),
            3 => Ok(Self::Dynamic),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsParticipationV1 {
    Solid = 1,
    Sensor = 2,
    QueryOnly = 3,
}

impl PhysicsParticipationV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Solid),
            2 => Ok(Self::Sensor),
            3 => Ok(Self::QueryOnly),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsContactReportingV1 {
    Disabled = 1,
    BeginEnd = 2,
    BeginPersistEnd = 3,
}

impl PhysicsContactReportingV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Disabled),
            2 => Ok(Self::BeginEnd),
            3 => Ok(Self::BeginPersistEnd),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicsGeometryV1 {
    Box {
        half_extents_micrometres: [i64; 3],
    },
    Sphere {
        radius_micrometres: i64,
    },
    Capsule {
        radius_micrometres: i64,
        half_segment_micrometres: i64,
    },
    ConvexHull {
        collision_asset_id: AssetId,
        collision_asset_revision: u32,
        shape_index: u32,
        feature_table_hash: ContentHash,
    },
    TriangleMesh {
        collision_asset_id: AssetId,
        collision_asset_revision: u32,
        shape_index: u32,
        feature_table_hash: ContentHash,
    },
    HeightField {
        collision_asset_id: AssetId,
        collision_asset_revision: u32,
        shape_index: u32,
        feature_table_hash: ContentHash,
    },
}

impl PhysicsGeometryV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        match self {
            Self::Box {
                half_extents_micrometres,
            } => {
                validate_positive_vec3(*half_extents_micrometres)?;
            }
            Self::Sphere { radius_micrometres } => validate_positive(*radius_micrometres)?,
            Self::Capsule {
                radius_micrometres,
                half_segment_micrometres,
            } => {
                validate_positive(*radius_micrometres)?;
                if *half_segment_micrometres < 0 {
                    return Err(PhysicsContractError::InvalidDescriptor);
                }
            }
            Self::ConvexHull {
                collision_asset_revision,
                ..
            }
            | Self::TriangleMesh {
                collision_asset_revision,
                ..
            }
            | Self::HeightField {
                collision_asset_revision,
                ..
            } if *collision_asset_revision == 0 => {
                return Err(PhysicsContractError::InvalidDescriptor);
            }
            Self::ConvexHull { .. } | Self::TriangleMesh { .. } | Self::HeightField { .. } => {}
        }
        Ok(())
    }

    fn canonical_tagged_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, record) = match self {
            Self::Box {
                half_extents_micrometres,
            } => (
                1,
                encode_struct([CanonicalField::new(
                    1,
                    CANONICAL_TYPE_BYTES,
                    encode_i64_vec3(*half_extents_micrometres),
                )])?,
            ),
            Self::Sphere { radius_micrometres } => {
                (2, encode_struct([field_i64(1, *radius_micrometres)])?)
            }
            Self::Capsule {
                radius_micrometres,
                half_segment_micrometres,
            } => (
                3,
                encode_struct([
                    field_i64(1, *radius_micrometres),
                    field_i64(2, *half_segment_micrometres),
                ])?,
            ),
            Self::ConvexHull {
                collision_asset_id,
                collision_asset_revision,
                shape_index,
                feature_table_hash,
            } => (
                4,
                encode_asset_geometry(
                    *collision_asset_id,
                    *collision_asset_revision,
                    *shape_index,
                    *feature_table_hash,
                )?,
            ),
            Self::TriangleMesh {
                collision_asset_id,
                collision_asset_revision,
                shape_index,
                feature_table_hash,
            } => (
                5,
                encode_asset_geometry(
                    *collision_asset_id,
                    *collision_asset_revision,
                    *shape_index,
                    *feature_table_hash,
                )?,
            ),
            Self::HeightField {
                collision_asset_id,
                collision_asset_revision,
                shape_index,
                feature_table_hash,
            } => (
                6,
                encode_asset_geometry(
                    *collision_asset_id,
                    *collision_asset_revision,
                    *shape_index,
                    *feature_table_hash,
                )?,
            ),
        };
        let mut tagged = vec![tag];
        tagged.extend_from_slice(&nested(CANONICAL_TYPE_STRUCT, &record)?);
        Ok(tagged)
    }

    fn from_tagged_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        let tag = cursor.read_u8()?;
        let (nested_tag, record) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        if nested_tag != CANONICAL_TYPE_STRUCT {
            return Err(PhysicsContractError::FieldType);
        }
        let fields = decode_struct(record, limits)?;
        let geometry = match tag {
            1 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_BYTES)])?;
                Self::Box {
                    half_extents_micrometres: decode_i64_vec3(&field_from(&fields, 1)?.payload)?,
                }
            }
            2 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_I64)])?;
                Self::Sphere {
                    radius_micrometres: read_i64_fields(&fields, 1)?,
                }
            }
            3 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_I64), (2, CANONICAL_TYPE_I64)])?;
                Self::Capsule {
                    radius_micrometres: read_i64_fields(&fields, 1)?,
                    half_segment_micrometres: read_i64_fields(&fields, 2)?,
                }
            }
            4..=6 => {
                let (asset, revision, shape_index, feature_hash) = decode_asset_geometry(&fields)?;
                match tag {
                    4 => Self::ConvexHull {
                        collision_asset_id: asset,
                        collision_asset_revision: revision,
                        shape_index,
                        feature_table_hash: feature_hash,
                    },
                    5 => Self::TriangleMesh {
                        collision_asset_id: asset,
                        collision_asset_revision: revision,
                        shape_index,
                        feature_table_hash: feature_hash,
                    },
                    _ => Self::HeightField {
                        collision_asset_id: asset,
                        collision_asset_revision: revision,
                        shape_index,
                        feature_table_hash: feature_hash,
                    },
                }
            }
            other => return Err(PhysicsContractError::UnknownTag(other)),
        };
        geometry.validate()?;
        Ok(geometry)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsMaterialDescriptorV1 {
    pub material_id: SchemaId,
    pub descriptor_revision: u32,
    pub static_friction_q16: u32,
    pub dynamic_friction_q16: u32,
    pub restitution_q16: u32,
    pub canonical_material_tags: Vec<SchemaId>,
}

impl PhysicsMaterialDescriptorV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.descriptor_revision == 0
            || self.dynamic_friction_q16 > self.static_friction_q16
            || self.restitution_q16 > 65_536
            || self
                .canonical_material_tags
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.material_id.as_str().as_bytes().to_vec(),
            ),
            field_u32(2, self.descriptor_revision),
            field_u32(3, self.static_friction_q16),
            field_u32(4, self.dynamic_friction_q16),
            field_u32(5, self.restitution_q16),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.canonical_material_tags
                        .iter()
                        .map(|tag| tag.as_str().as_bytes().to_vec())
                        .collect(),
                )?,
            ),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_U32),
                (6, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            material_id: SchemaId::new(read_utf8_fields(&fields, 1)?)?,
            descriptor_revision: read_u32_fields(&fields, 2)?,
            static_friction_q16: read_u32_fields(&fields, 3)?,
            dynamic_friction_q16: read_u32_fields(&fields, 4)?,
            restitution_q16: read_u32_fields(&fields, 5)?,
            canonical_material_tags: decode_sequence(&field_from(&fields, 6)?.payload, limits)?
                .into_iter()
                .map(|bytes| {
                    let value = std::str::from_utf8(&bytes).map_err(|_| {
                        PhysicsContractError::Canonical(CanonicalDecodeError::InvalidUtf8)
                    })?;
                    Ok(SchemaId::new(value)?)
                })
                .collect::<Result<_, PhysicsContractError>>()?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsShapeDescriptorV1 {
    pub shape_id: PhysicsShapeIdV1,
    pub descriptor_revision: u32,
    pub local_pose: PhysicsPoseV1,
    pub geometry: PhysicsGeometryV1,
    pub material_id: SchemaId,
    pub collision_layer: u8,
    pub collision_mask: u64,
    pub participation: PhysicsParticipationV1,
    pub contact_reporting: PhysicsContactReportingV1,
}

impl PhysicsShapeDescriptorV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.descriptor_revision == 0 || self.collision_layer > 63 {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        self.local_pose.validate()?;
        self.geometry.validate()
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_STRUCT, self.shape_id.canonical_record()?),
            field_u32(2, self.descriptor_revision),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_STRUCT,
                self.local_pose.canonical_record()?,
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_TAGGED_UNION,
                self.geometry.canonical_tagged_bytes()?,
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_UTF8_NFC,
                self.material_id.as_str().as_bytes().to_vec(),
            ),
            field_u8(6, self.collision_layer),
            field_u64(7, self.collision_mask),
            field_u8(8, self.participation as u8),
            field_u8(9, self.contact_reporting as u8),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_STRUCT),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_STRUCT),
                (4, CANONICAL_TYPE_TAGGED_UNION),
                (5, CANONICAL_TYPE_UTF8_NFC),
                (6, CANONICAL_TYPE_U8),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_U8),
                (9, CANONICAL_TYPE_U8),
            ],
        )?;
        let value = Self {
            shape_id: PhysicsShapeIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?,
            descriptor_revision: read_u32_fields(&fields, 2)?,
            local_pose: PhysicsPoseV1::from_record(&field_from(&fields, 3)?.payload, limits)?,
            geometry: PhysicsGeometryV1::from_tagged_bytes(
                &field_from(&fields, 4)?.payload,
                limits,
            )?,
            material_id: SchemaId::new(read_utf8_fields(&fields, 5)?)?,
            collision_layer: read_u8_fields(&fields, 6)?,
            collision_mask: read_u64_fields(&fields, 7)?,
            participation: PhysicsParticipationV1::from_tag(read_u8_fields(&fields, 8)?)?,
            contact_reporting: PhysicsContactReportingV1::from_tag(read_u8_fields(&fields, 9)?)?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsBodyDescriptorV1 {
    pub body_id: PhysicsBodyIdV1,
    pub descriptor_revision: u32,
    pub motion_kind: PhysicsMotionKindV1,
    pub initial_pose: PhysicsPoseV1,
    pub initial_linear_velocity_micrometres_per_second: [i64; 3],
    pub initial_angular_velocity_q16: [i64; 3],
    pub active: bool,
    pub shapes: BTreeMap<PhysicsShapeIdV1, PhysicsShapeDescriptorV1>,
}

impl PhysicsBodyDescriptorV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.descriptor_revision == 0 || self.shapes.is_empty() {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        self.initial_pose.validate()?;
        for (id, shape) in &self.shapes {
            if id != &shape.shape_id || shape.shape_id.body_id != self.body_id {
                return Err(PhysicsContractError::DuplicateKey);
            }
            shape.validate()?;
        }
        if self.motion_kind == PhysicsMotionKindV1::Static
            && (self.initial_linear_velocity_micrometres_per_second != [0; 3]
                || self.initial_angular_velocity_q16 != [0; 3])
        {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let shapes = self
            .shapes
            .iter()
            .map(|(id, shape)| {
                encode_struct([
                    CanonicalField::new(1, CANONICAL_TYPE_STRUCT, id.canonical_record()?),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, shape.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            field_u32(2, self.descriptor_revision),
            field_u8(3, self.motion_kind as u8),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_STRUCT,
                self.initial_pose.canonical_record()?,
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.initial_linear_velocity_micrometres_per_second),
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.initial_angular_velocity_q16),
            ),
            field_bool(7, self.active),
            CanonicalField::new(8, CANONICAL_TYPE_MAP, encode_sequence(shapes)?),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_STRUCT),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_U8),
                (4, CANONICAL_TYPE_STRUCT),
                (5, CANONICAL_TYPE_BYTES),
                (6, CANONICAL_TYPE_BYTES),
                (7, CANONICAL_TYPE_BOOL),
                (8, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut shapes = BTreeMap::new();
        for entry in decode_sequence(&field_from(&fields, 8)?.payload, limits)? {
            let entry_fields = decode_struct(&entry, limits)?;
            require_fields(
                &entry_fields,
                &[(1, CANONICAL_TYPE_STRUCT), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let id = PhysicsShapeIdV1::from_record(&field_from(&entry_fields, 1)?.payload, limits)?;
            let shape = PhysicsShapeDescriptorV1::from_record(
                &field_from(&entry_fields, 2)?.payload,
                limits,
            )?;
            if shapes.insert(id, shape).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let value = Self {
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?,
            descriptor_revision: read_u32_fields(&fields, 2)?,
            motion_kind: PhysicsMotionKindV1::from_tag(read_u8_fields(&fields, 3)?)?,
            initial_pose: PhysicsPoseV1::from_record(&field_from(&fields, 4)?.payload, limits)?,
            initial_linear_velocity_micrometres_per_second: decode_i64_vec3(
                &field_from(&fields, 5)?.payload,
            )?,
            initial_angular_velocity_q16: decode_i64_vec3(&field_from(&fields, 6)?.payload)?,
            active: read_bool_fields(&fields, 7)?,
            shapes,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsCoordinateProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
}

impl PhysicsCoordinateProfileV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.physics-coordinate-profile",
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.profile_id.as_str().as_bytes().to_vec(),
                ),
            ],
        )
    }

    pub fn reference_v1() -> Result<Self, crate::IdentifierError> {
        Ok(Self {
            schema_version: 1,
            profile_id: SchemaId::new("nextengine.physics.coordinate.right-handed-y-up-v1")?,
        })
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        profile_hash(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsLimitsProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub maximum_bodies: u32,
    pub maximum_shapes: u32,
    pub maximum_contacts_per_substep: u32,
}

impl PhysicsLimitsProfileV1 {
    pub fn reference_v1() -> Result<Self, crate::IdentifierError> {
        Ok(Self {
            schema_version: 1,
            profile_id: SchemaId::new("nextengine.physics.limits.grounded-capsule-v1")?,
            maximum_bodies: 1_025,
            maximum_shapes: 4_096,
            maximum_contacts_per_substep: 4_096,
        })
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != 1
            || self.maximum_bodies == 0
            || self.maximum_bodies > 1_048_576
            || self.maximum_shapes == 0
            || self.maximum_shapes > 4_194_304
            || self.maximum_contacts_per_substep == 0
            || self.maximum_contacts_per_substep > 4_194_304
        {
            return Err(PhysicsContractError::LimitExceeded);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.physics-limits-profile",
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.profile_id.as_str().as_bytes().to_vec(),
                ),
                field_u32(3, self.maximum_bodies),
                field_u32(4, self.maximum_shapes),
                field_u32(5, self.maximum_contacts_per_substep),
            ],
        )
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        profile_hash(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsSolverSemanticsProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub gravity_micrometres_per_second_squared: [i64; 3],
}

impl PhysicsSolverSemanticsProfileV1 {
    pub fn grounded_capsule_v1() -> Result<Self, crate::IdentifierError> {
        Ok(Self {
            schema_version: 1,
            profile_id: SchemaId::new("nextengine.physics.solver.grounded-capsule-v1")?,
            gravity_micrometres_per_second_squared: [
                0,
                REFERENCE_GRAVITY_MICROMETRES_PER_SECOND_SQUARED,
                0,
            ],
        })
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != 1
            || self.gravity_micrometres_per_second_squared[0] != 0
            || self.gravity_micrometres_per_second_squared[2] != 0
            || self.gravity_micrometres_per_second_squared[1] > 0
        {
            return Err(PhysicsContractError::InvalidProfile);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.physics-solver-semantics-profile",
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.profile_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_BYTES,
                    encode_i64_vec3(self.gravity_micrometres_per_second_squared),
                ),
            ],
        )
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        profile_hash(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsWorldDescriptorV1 {
    pub schema_version: u16,
    pub world_id: PhysicsWorldId,
    pub descriptor_revision: u32,
    pub coordinate_profile_hash: ContentHash,
    pub tick_rate_profile_hash: ContentHash,
    pub authoritative_numeric_profile_hash: ContentHash,
    pub physics_quantization_profile_hash: ContentHash,
    pub physics_limits_profile_hash: ContentHash,
    pub solver_semantics_profile_hash: ContentHash,
    pub gravity_micrometres_per_second_squared: [i64; 3],
    pub material_catalog_hash: ContentHash,
    pub body_catalog_hash: ContentHash,
}

impl PhysicsWorldDescriptorV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u16(1, self.schema_version),
            field_id(2, self.world_id.as_bytes()),
            field_u32(3, self.descriptor_revision),
            field_hash(4, self.coordinate_profile_hash),
            field_hash(5, self.tick_rate_profile_hash),
            field_hash(6, self.authoritative_numeric_profile_hash),
            field_hash(7, self.physics_quantization_profile_hash),
            field_hash(8, self.physics_limits_profile_hash),
            field_hash(9, self.solver_semantics_profile_hash),
            CanonicalField::new(
                10,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.gravity_micrometres_per_second_squared),
            ),
            field_hash(11, self.material_catalog_hash),
            field_hash(12, self.body_catalog_hash),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_HASH256),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_HASH256),
                (8, CANONICAL_TYPE_HASH256),
                (9, CANONICAL_TYPE_HASH256),
                (10, CANONICAL_TYPE_BYTES),
                (11, CANONICAL_TYPE_HASH256),
                (12, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16_fields(&fields, 1)?,
            world_id: PhysicsWorldId::from_bytes(exact(&field_from(&fields, 2)?.payload)?),
            descriptor_revision: read_u32_fields(&fields, 3)?,
            coordinate_profile_hash: read_hash_fields(&fields, 4)?,
            tick_rate_profile_hash: read_hash_fields(&fields, 5)?,
            authoritative_numeric_profile_hash: read_hash_fields(&fields, 6)?,
            physics_quantization_profile_hash: read_hash_fields(&fields, 7)?,
            physics_limits_profile_hash: read_hash_fields(&fields, 8)?,
            solver_semantics_profile_hash: read_hash_fields(&fields, 9)?,
            gravity_micrometres_per_second_squared: decode_i64_vec3(
                &field_from(&fields, 10)?.payload,
            )?,
            material_catalog_hash: read_hash_fields(&fields, 11)?,
            body_catalog_hash: read_hash_fields(&fields, 12)?,
        };
        if value.schema_version != 1 || value.descriptor_revision == 0 {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsWorldCatalogV1 {
    pub schema_version: u16,
    pub coordinate_profile: PhysicsCoordinateProfileV1,
    pub limits_profile: PhysicsLimitsProfileV1,
    pub solver_profile: PhysicsSolverSemanticsProfileV1,
    pub world_descriptor: PhysicsWorldDescriptorV1,
    pub materials: BTreeMap<SchemaId, PhysicsMaterialDescriptorV1>,
    pub bodies: BTreeMap<PhysicsBodyIdV1, PhysicsBodyDescriptorV1>,
    pub avatar_bindings: BTreeMap<PersistentId, PhysicsBodyIdV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsWorldCatalogProfilesV1 {
    pub coordinate: PhysicsCoordinateProfileV1,
    pub limits: PhysicsLimitsProfileV1,
    pub solver: PhysicsSolverSemanticsProfileV1,
    pub tick_rate_hash: ContentHash,
    pub authoritative_numeric_hash: ContentHash,
    pub quantization_hash: ContentHash,
}

impl PhysicsWorldCatalogV1 {
    pub fn new(
        world_id: PhysicsWorldId,
        profiles: PhysicsWorldCatalogProfilesV1,
        materials: BTreeMap<SchemaId, PhysicsMaterialDescriptorV1>,
        bodies: BTreeMap<PhysicsBodyIdV1, PhysicsBodyDescriptorV1>,
        avatar_bindings: BTreeMap<PersistentId, PhysicsBodyIdV1>,
    ) -> Result<Self, PhysicsContractError> {
        let PhysicsWorldCatalogProfilesV1 {
            coordinate: coordinate_profile,
            limits: limits_profile,
            solver: solver_profile,
            tick_rate_hash: tick_rate_profile_hash,
            authoritative_numeric_hash: authoritative_numeric_profile_hash,
            quantization_hash: physics_quantization_profile_hash,
        } = profiles;
        let material_catalog_hash = descriptor_collection_hash(
            b"nextengine.physics-material-catalog.v1\0",
            materials
                .values()
                .map(PhysicsMaterialDescriptorV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let body_catalog_hash = descriptor_collection_hash(
            b"nextengine.physics-body-descriptor-catalog.v1\0",
            bodies
                .values()
                .map(PhysicsBodyDescriptorV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let world_descriptor = PhysicsWorldDescriptorV1 {
            schema_version: 1,
            world_id,
            descriptor_revision: 1,
            coordinate_profile_hash: coordinate_profile.profile_hash()?,
            tick_rate_profile_hash,
            authoritative_numeric_profile_hash,
            physics_quantization_profile_hash,
            physics_limits_profile_hash: limits_profile.profile_hash()?,
            solver_semantics_profile_hash: solver_profile.profile_hash()?,
            gravity_micrometres_per_second_squared: solver_profile
                .gravity_micrometres_per_second_squared,
            material_catalog_hash,
            body_catalog_hash,
        };
        let value = Self {
            schema_version: 1,
            coordinate_profile,
            limits_profile,
            solver_profile,
            world_descriptor,
            materials,
            bodies,
            avatar_bindings,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != 1 {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                self.schema_version,
            )));
        }
        self.limits_profile.validate()?;
        self.solver_profile.validate()?;
        if self.bodies.len()
            > usize::try_from(self.limits_profile.maximum_bodies)
                .map_err(|_| PhysicsContractError::LimitExceeded)?
        {
            return Err(PhysicsContractError::LimitExceeded);
        }
        let mut shape_count = 0_usize;
        for (id, material) in &self.materials {
            if id != &material.material_id {
                return Err(PhysicsContractError::DuplicateKey);
            }
            material.validate()?;
        }
        for (id, body) in &self.bodies {
            if id != &body.body_id {
                return Err(PhysicsContractError::DuplicateKey);
            }
            body.validate()?;
            shape_count = shape_count
                .checked_add(body.shapes.len())
                .ok_or(PhysicsContractError::LimitExceeded)?;
            for shape in body.shapes.values() {
                if !self.materials.contains_key(&shape.material_id) {
                    return Err(PhysicsContractError::ReferenceInvalid);
                }
            }
        }
        if shape_count
            > usize::try_from(self.limits_profile.maximum_shapes)
                .map_err(|_| PhysicsContractError::LimitExceeded)?
        {
            return Err(PhysicsContractError::LimitExceeded);
        }
        for (target, body) in &self.avatar_bindings {
            if target != &body.subject_id || !self.bodies.contains_key(body) {
                return Err(PhysicsContractError::ReferenceInvalid);
            }
        }
        let material_hash = descriptor_collection_hash(
            b"nextengine.physics-material-catalog.v1\0",
            self.materials
                .values()
                .map(PhysicsMaterialDescriptorV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let body_hash = descriptor_collection_hash(
            b"nextengine.physics-body-descriptor-catalog.v1\0",
            self.bodies
                .values()
                .map(PhysicsBodyDescriptorV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        if self.world_descriptor.schema_version != 1
            || self.world_descriptor.coordinate_profile_hash
                != self.coordinate_profile.profile_hash()?
            || self.world_descriptor.physics_limits_profile_hash
                != self.limits_profile.profile_hash()?
            || self.world_descriptor.solver_semantics_profile_hash
                != self.solver_profile.profile_hash()?
            || self.world_descriptor.gravity_micrometres_per_second_squared
                != self.solver_profile.gravity_micrometres_per_second_squared
            || self.world_descriptor.material_catalog_hash != material_hash
            || self.world_descriptor.body_catalog_hash != body_hash
        {
            return Err(PhysicsContractError::ProfileMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let materials = self
            .materials
            .iter()
            .map(|(id, descriptor)| {
                encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_UTF8_NFC,
                        id.as_str().as_bytes().to_vec(),
                    ),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, descriptor.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bodies = self
            .bodies
            .iter()
            .map(|(id, descriptor)| {
                encode_struct([
                    CanonicalField::new(1, CANONICAL_TYPE_STRUCT, id.canonical_record()?),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, descriptor.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bindings = self
            .avatar_bindings
            .iter()
            .map(|(target, body)| {
                encode_struct([
                    field_id(1, target.as_bytes()),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, body.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.physics-world-catalog",
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_BYTES,
                    self.coordinate_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_BYTES,
                    self.limits_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_BYTES,
                    self.solver_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_STRUCT,
                    self.world_descriptor.canonical_record()?,
                ),
                CanonicalField::new(6, CANONICAL_TYPE_MAP, encode_sequence(materials)?),
                CanonicalField::new(7, CANONICAL_TYPE_MAP, encode_sequence(bodies)?),
                CanonicalField::new(8, CANONICAL_TYPE_MAP, encode_sequence(bindings)?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            "nextengine.physics-world-catalog",
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_BYTES),
                (5, CANONICAL_TYPE_STRUCT),
                (6, CANONICAL_TYPE_MAP),
                (7, CANONICAL_TYPE_MAP),
                (8, CANONICAL_TYPE_MAP),
            ],
        )?;
        let coordinate_profile = decode_coordinate_profile(field(&segment, 2)?, limits)?;
        let limits_profile = decode_limits_profile(field(&segment, 3)?, limits)?;
        let solver_profile = decode_solver_profile(field(&segment, 4)?, limits)?;
        let world_descriptor = PhysicsWorldDescriptorV1::from_record(field(&segment, 5)?, limits)?;
        let mut materials = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 6)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_UTF8_NFC), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let id = SchemaId::new(read_utf8_fields(&fields, 1)?)?;
            let descriptor =
                PhysicsMaterialDescriptorV1::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if materials.insert(id, descriptor).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let mut bodies = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 7)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_STRUCT), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let id = PhysicsBodyIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?;
            let descriptor =
                PhysicsBodyDescriptorV1::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if bodies.insert(id, descriptor).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let mut avatar_bindings = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 8)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let target = PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?);
            let body = PhysicsBodyIdV1::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if avatar_bindings.insert(target, body).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            coordinate_profile,
            limits_profile,
            solver_profile,
            world_descriptor,
            materials,
            bodies,
            avatar_bindings,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn catalog_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-world-catalog-hash.v1\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsBodyStateV2 {
    pub body_id: PhysicsBodyIdV1,
    pub body_revision: u64,
    pub pose: PhysicsPoseV1,
    pub linear_velocity_micrometres_per_second: [i64; 3],
    pub angular_velocity_q16: [i64; 3],
    pub active: bool,
    pub sleep_counter: u32,
}

impl PhysicsBodyStateV2 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            field_u64(2, self.body_revision),
            CanonicalField::new(3, CANONICAL_TYPE_STRUCT, self.pose.canonical_record()?),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.linear_velocity_micrometres_per_second),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.angular_velocity_q16),
            ),
            field_bool(6, self.active),
            field_u32(7, self.sleep_counter),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_STRUCT),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_STRUCT),
                (4, CANONICAL_TYPE_BYTES),
                (5, CANONICAL_TYPE_BYTES),
                (6, CANONICAL_TYPE_BOOL),
                (7, CANONICAL_TYPE_U32),
            ],
        )?;
        let value = Self {
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?,
            body_revision: read_u64_fields(&fields, 2)?,
            pose: PhysicsPoseV1::from_record(&field_from(&fields, 3)?.payload, limits)?,
            linear_velocity_micrometres_per_second: decode_i64_vec3(
                &field_from(&fields, 4)?.payload,
            )?,
            angular_velocity_q16: decode_i64_vec3(&field_from(&fields, 5)?.payload)?,
            active: read_bool_fields(&fields, 6)?,
            sleep_counter: read_u32_fields(&fields, 7)?,
        };
        value.pose.validate()?;
        Ok(value)
    }

    pub fn state_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-body-state.v2\0",
            &self.canonical_record()?,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ContactPhaseV1 {
    Begin = 1,
    Persist = 2,
    End = 3,
}

impl ContactPhaseV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Begin),
            2 => Ok(Self::Persist),
            3 => Ok(Self::End),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsContactContinuityStateV1 {
    pub contact_id: PhysicsContactId,
    pub participant_low: PhysicsShapeIdV1,
    pub participant_high: PhysicsShapeIdV1,
    pub feature_low: u8,
    pub feature_high: u8,
    pub point_micrometres: [i64; 3],
    pub normal_low_to_high_q1_30: [i32; 3],
    pub last_seen_physics_tick: u64,
}

impl PhysicsContactContinuityStateV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_id(1, self.contact_id.as_bytes()),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_STRUCT,
                self.participant_low.canonical_record()?,
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_STRUCT,
                self.participant_high.canonical_record()?,
            ),
            field_u8(4, self.feature_low),
            field_u8(5, self.feature_high),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.point_micrometres),
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_BYTES,
                encode_i32_vec3(self.normal_low_to_high_q1_30),
            ),
            field_u64(8, self.last_seen_physics_tick),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_ID128),
                (2, CANONICAL_TYPE_STRUCT),
                (3, CANONICAL_TYPE_STRUCT),
                (4, CANONICAL_TYPE_U8),
                (5, CANONICAL_TYPE_U8),
                (6, CANONICAL_TYPE_BYTES),
                (7, CANONICAL_TYPE_BYTES),
                (8, CANONICAL_TYPE_U64),
            ],
        )?;
        let value = Self {
            contact_id: PhysicsContactId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            participant_low: PhysicsShapeIdV1::from_record(
                &field_from(&fields, 2)?.payload,
                limits,
            )?,
            participant_high: PhysicsShapeIdV1::from_record(
                &field_from(&fields, 3)?.payload,
                limits,
            )?,
            feature_low: read_u8_fields(&fields, 4)?,
            feature_high: read_u8_fields(&fields, 5)?,
            point_micrometres: decode_i64_vec3(&field_from(&fields, 6)?.payload)?,
            normal_low_to_high_q1_30: decode_i32_vec3(&field_from(&fields, 7)?.payload)?,
            last_seen_physics_tick: read_u64_fields(&fields, 8)?,
        };
        if value.participant_low >= value.participant_high
            || derive_physics_contact_id(
                value.participant_low,
                value.participant_high,
                value.feature_low,
                value.feature_high,
            ) != value.contact_id
        {
            return Err(PhysicsContractError::ContactIdentityMismatch);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsCanonicalSnapshotV2 {
    pub schema_version: u16,
    pub world_id: PhysicsWorldId,
    pub world_revision: u64,
    pub checkpoint_revision: u64,
    pub physics_tick: u64,
    pub world_descriptor_hash: ContentHash,
    pub catalog_hash: ContentHash,
    pub tick_rate_profile_hash: ContentHash,
    pub authoritative_numeric_profile_hash: ContentHash,
    pub physics_quantization_profile_hash: ContentHash,
    pub physics_limits_profile_hash: ContentHash,
    pub sorted_body_states: BTreeMap<PhysicsBodyIdV1, PhysicsBodyStateV2>,
    pub sorted_contact_continuity_states:
        BTreeMap<PhysicsContactId, PhysicsContactContinuityStateV1>,
    pub sorted_solver_continuation_states: BTreeMap<PhysicsContactId, [i64; 3]>,
}

impl PhysicsCanonicalSnapshotV2 {
    pub fn genesis(
        catalog: &PhysicsWorldCatalogV1,
        tick_rate: &TickRateProfileV1,
        numeric: &AuthoritativeNumericProfileV1,
        quantization: &PhysicsQuantizationProfileV1,
    ) -> Result<Self, PhysicsContractError> {
        catalog.validate()?;
        let sorted_body_states = catalog
            .bodies
            .values()
            .map(|body| {
                (
                    body.body_id,
                    PhysicsBodyStateV2 {
                        body_id: body.body_id,
                        body_revision: 0,
                        pose: body.initial_pose,
                        linear_velocity_micrometres_per_second: body
                            .initial_linear_velocity_micrometres_per_second,
                        angular_velocity_q16: body.initial_angular_velocity_q16,
                        active: body.active,
                        sleep_counter: 0,
                    },
                )
            })
            .collect();
        let value = Self {
            schema_version: PHYSICS_SNAPSHOT_SCHEMA_VERSION,
            world_id: catalog.world_descriptor.world_id,
            world_revision: 0,
            checkpoint_revision: 0,
            physics_tick: 0,
            world_descriptor_hash: physics_contract_hash(
                b"nextengine.physics-world-descriptor.v1\0",
                &catalog.world_descriptor.canonical_record()?,
            )?,
            catalog_hash: catalog.catalog_hash()?,
            tick_rate_profile_hash: tick_rate.profile_hash()?,
            authoritative_numeric_profile_hash: numeric.profile_hash()?,
            physics_quantization_profile_hash: quantization.profile_hash()?,
            physics_limits_profile_hash: catalog.limits_profile.profile_hash()?,
            sorted_body_states,
            sorted_contact_continuity_states: BTreeMap::new(),
            sorted_solver_continuation_states: BTreeMap::new(),
        };
        value.validate_profile_closure(catalog, tick_rate, numeric, quantization)?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_SNAPSHOT_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                self.schema_version,
            )));
        }
        for (id, body) in &self.sorted_body_states {
            if id != &body.body_id {
                return Err(PhysicsContractError::DuplicateKey);
            }
            body.pose.validate()?;
        }
        for (id, contact) in &self.sorted_contact_continuity_states {
            if id != &contact.contact_id {
                return Err(PhysicsContractError::DuplicateKey);
            }
            if contact.participant_low >= contact.participant_high
                || contact.last_seen_physics_tick != self.physics_tick
                || derive_physics_contact_id(
                    contact.participant_low,
                    contact.participant_high,
                    contact.feature_low,
                    contact.feature_high,
                ) != *id
            {
                return Err(PhysicsContractError::ContactIdentityMismatch);
            }
        }
        if !self
            .sorted_solver_continuation_states
            .keys()
            .eq(self.sorted_contact_continuity_states.keys())
        {
            return Err(PhysicsContractError::ReferenceInvalid);
        }
        Ok(())
    }

    fn validate_catalog_closure(
        &self,
        catalog: &PhysicsWorldCatalogV1,
    ) -> Result<(), PhysicsContractError> {
        if !self.sorted_body_states.keys().eq(catalog.bodies.keys()) {
            return Err(PhysicsContractError::ReferenceInvalid);
        }
        for (body_id, descriptor) in &catalog.bodies {
            let state = self
                .sorted_body_states
                .get(body_id)
                .ok_or(PhysicsContractError::ReferenceInvalid)?;
            if state.active != descriptor.active {
                return Err(PhysicsContractError::ReferenceInvalid);
            }
            match descriptor.motion_kind {
                PhysicsMotionKindV1::Static => {
                    if state.body_revision != 0
                        || state.pose != descriptor.initial_pose
                        || state.linear_velocity_micrometres_per_second != [0; 3]
                        || state.angular_velocity_q16 != [0; 3]
                        || state.sleep_counter != 0
                    {
                        return Err(PhysicsContractError::ReferenceInvalid);
                    }
                }
                PhysicsMotionKindV1::Kinematic | PhysicsMotionKindV1::Dynamic => {
                    let has_capsule = descriptor
                        .shapes
                        .values()
                        .any(|shape| matches!(shape.geometry, PhysicsGeometryV1::Capsule { .. }));
                    if has_capsule
                        && (state.pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
                            || state.angular_velocity_q16 != [0; 3])
                    {
                        return Err(PhysicsContractError::ReferenceInvalid);
                    }
                }
            }
        }
        for contact in self.sorted_contact_continuity_states.values() {
            let low = catalog_shape(catalog, contact.participant_low)
                .ok_or(PhysicsContractError::ReferenceInvalid)?;
            let high = catalog_shape(catalog, contact.participant_high)
                .ok_or(PhysicsContractError::ReferenceInvalid)?;
            if !primitive_feature_is_valid(&low.geometry, contact.feature_low)
                || !primitive_feature_is_valid(&high.geometry, contact.feature_high)
            {
                return Err(PhysicsContractError::ReferenceInvalid);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let bodies = self
            .sorted_body_states
            .iter()
            .map(|(id, body)| {
                encode_struct([
                    CanonicalField::new(1, CANONICAL_TYPE_STRUCT, id.canonical_record()?),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, body.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let contacts = self
            .sorted_contact_continuity_states
            .iter()
            .map(|(id, contact)| {
                encode_struct([
                    field_id(1, id.as_bytes()),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, contact.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let solver = self
            .sorted_solver_continuation_states
            .iter()
            .map(|(id, impulses)| {
                encode_struct([
                    field_id(1, id.as_bytes()),
                    CanonicalField::new(2, CANONICAL_TYPE_BYTES, encode_i64_vec3(*impulses)),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_SNAPSHOT_SCHEMA_ID,
            PHYSICS_SNAPSHOT_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.world_id.as_bytes()),
                field_u64(3, self.world_revision),
                field_u64(4, self.checkpoint_revision),
                field_u64(5, self.physics_tick),
                field_hash(6, self.world_descriptor_hash),
                field_hash(7, self.catalog_hash),
                field_hash(8, self.tick_rate_profile_hash),
                field_hash(9, self.authoritative_numeric_profile_hash),
                field_hash(10, self.physics_quantization_profile_hash),
                field_hash(11, self.physics_limits_profile_hash),
                CanonicalField::new(12, CANONICAL_TYPE_MAP, encode_sequence(bodies)?),
                CanonicalField::new(13, CANONICAL_TYPE_MAP, encode_sequence(contacts)?),
                CanonicalField::new(14, CANONICAL_TYPE_MAP, encode_sequence(solver)?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let header = decode_canonical_segment(bytes, limits)?;
        if header.owner_id == PHYSICS_SNAPSHOT_OWNER_ID
            && header.schema_id == PHYSICS_SNAPSHOT_SCHEMA_ID
        {
            let version = read_u16(&header, 1)?;
            if version != PHYSICS_SNAPSHOT_SCHEMA_VERSION
                || header.segment_id != PHYSICS_SNAPSHOT_SEGMENT_ID
            {
                return Err(PhysicsContractError::UnsupportedVersion(u32::from(version)));
            }
        }
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_SNAPSHOT_SCHEMA_ID,
            PHYSICS_SNAPSHOT_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_U64),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_HASH256),
                (8, CANONICAL_TYPE_HASH256),
                (9, CANONICAL_TYPE_HASH256),
                (10, CANONICAL_TYPE_HASH256),
                (11, CANONICAL_TYPE_HASH256),
                (12, CANONICAL_TYPE_MAP),
                (13, CANONICAL_TYPE_MAP),
                (14, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut bodies = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 12)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_STRUCT), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let id = PhysicsBodyIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?;
            let body = PhysicsBodyStateV2::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if bodies.insert(id, body).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let mut contacts = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 13)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let id = PhysicsContactId::from_bytes(exact(&field_from(&fields, 1)?.payload)?);
            let contact = PhysicsContactContinuityStateV1::from_record(
                &field_from(&fields, 2)?.payload,
                limits,
            )?;
            if contacts.insert(id, contact).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let mut solver = BTreeMap::new();
        for entry in decode_sequence(field(&segment, 14)?, limits)? {
            let fields = decode_struct(&entry, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_BYTES)],
            )?;
            let id = PhysicsContactId::from_bytes(exact(&field_from(&fields, 1)?.payload)?);
            let impulses = decode_i64_vec3(&field_from(&fields, 2)?.payload)?;
            if solver.insert(id, impulses).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            world_id: PhysicsWorldId::from_bytes(exact(field(&segment, 2)?)?),
            world_revision: read_u64(&segment, 3)?,
            checkpoint_revision: read_u64(&segment, 4)?,
            physics_tick: read_u64(&segment, 5)?,
            world_descriptor_hash: read_hash(&segment, 6)?,
            catalog_hash: read_hash(&segment, 7)?,
            tick_rate_profile_hash: read_hash(&segment, 8)?,
            authoritative_numeric_profile_hash: read_hash(&segment, 9)?,
            physics_quantization_profile_hash: read_hash(&segment, 10)?,
            physics_limits_profile_hash: read_hash(&segment, 11)?,
            sorted_body_states: bodies,
            sorted_contact_continuity_states: contacts,
            sorted_solver_continuation_states: solver,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn snapshot_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-snapshot.v2\0",
            &self.canonical_bytes()?,
        )
    }

    pub fn validate_profile_closure(
        &self,
        catalog: &PhysicsWorldCatalogV1,
        tick_rate: &TickRateProfileV1,
        numeric: &AuthoritativeNumericProfileV1,
        quantization: &PhysicsQuantizationProfileV1,
    ) -> Result<(), PhysicsContractError> {
        self.validate_catalog_closure(catalog)?;
        if self.world_id != catalog.world_descriptor.world_id
            || self.world_descriptor_hash
                != physics_contract_hash(
                    b"nextengine.physics-world-descriptor.v1\0",
                    &catalog.world_descriptor.canonical_record()?,
                )?
            || self.catalog_hash != catalog.catalog_hash()?
            || self.tick_rate_profile_hash != tick_rate.profile_hash()?
            || self.authoritative_numeric_profile_hash != numeric.profile_hash()?
            || self.physics_quantization_profile_hash != quantization.profile_hash()?
            || self.physics_limits_profile_hash != catalog.limits_profile.profile_hash()?
        {
            return Err(PhysicsContractError::ProfileMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsWorldCheckpointV1 {
    pub schema_version: u16,
    pub catalog: PhysicsWorldCatalogV1,
    pub snapshot: PhysicsCanonicalSnapshotV2,
}

impl PhysicsWorldCheckpointV1 {
    pub fn new(
        catalog: PhysicsWorldCatalogV1,
        snapshot: PhysicsCanonicalSnapshotV2,
    ) -> Result<Self, PhysicsContractError> {
        let value = Self {
            schema_version: PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION,
            catalog,
            snapshot,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                self.schema_version,
            )));
        }
        self.catalog.validate()?;
        self.snapshot.validate()?;
        self.snapshot.validate_catalog_closure(&self.catalog)?;
        if self.catalog.world_descriptor.world_id != self.snapshot.world_id
            || self.catalog.catalog_hash()? != self.snapshot.catalog_hash
        {
            return Err(PhysicsContractError::ProfileMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_BYTES, self.catalog.canonical_bytes()?),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, self.snapshot.canonical_bytes()?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            catalog: PhysicsWorldCatalogV1::from_canonical_bytes(field(&segment, 2)?, limits)?,
            snapshot: PhysicsCanonicalSnapshotV2::from_canonical_bytes(
                field(&segment, 3)?,
                limits,
            )?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn checkpoint_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-world-checkpoint.v1\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AcceptedLocomotionIntentV2 {
    pub causal_command_id: CommandId,
    pub controlled_target_id: PersistentId,
    pub body_id: PhysicsBodyIdV1,
    pub target_gameplay_tick: u64,
    pub direction_q15: [i16; 2],
}

impl AcceptedLocomotionIntentV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.controlled_target_id != self.body_id.subject_id {
            return Err(PhysicsContractError::ReferenceInvalid);
        }
        PhysicalCommandV1::SetCapsuleLocomotionIntent {
            direction_q15: self.direction_q15,
        }
        .validate()
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut direction = Vec::with_capacity(4);
        direction.extend_from_slice(&self.direction_q15[0].to_le_bytes());
        direction.extend_from_slice(&self.direction_q15[1].to_le_bytes());
        encode_struct([
            field_id(1, self.causal_command_id.as_bytes()),
            field_id(2, self.controlled_target_id.as_bytes()),
            CanonicalField::new(3, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            field_u64(4, self.target_gameplay_tick),
            CanonicalField::new(5, CANONICAL_TYPE_BYTES, direction),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_ID128),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_STRUCT),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let direction = &field_from(&fields, 5)?.payload;
        if direction.len() != 4 {
            return Err(PhysicsContractError::FieldLength);
        }
        let value = Self {
            causal_command_id: CommandId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            controlled_target_id: PersistentId::from_bytes(exact(
                &field_from(&fields, 2)?.payload,
            )?),
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 3)?.payload, limits)?,
            target_gameplay_tick: read_u64_fields(&fields, 4)?,
            direction_q15: [
                i16::from_le_bytes(exact(&direction[..2])?),
                i16::from_le_bytes(exact(&direction[2..])?),
            ],
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsStepInputV2 {
    pub schema_version: u16,
    pub world_id: PhysicsWorldId,
    pub expected_world_revision: u64,
    pub expected_snapshot_hash: ContentHash,
    pub expected_catalog_hash: ContentHash,
    pub gameplay_tick: u64,
    pub first_physics_tick: u64,
    pub physics_substeps: u32,
    pub accepted_intents: Vec<AcceptedLocomotionIntentV2>,
}

impl PhysicsStepInputV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_STEP_INPUT_SCHEMA_VERSION || self.physics_substeps == 0 {
            return Err(PhysicsContractError::InvalidProfile);
        }
        if self.accepted_intents.windows(2).any(|pair| {
            (pair[0].body_id, pair[0].causal_command_id)
                >= (pair[1].body_id, pair[1].causal_command_id)
        }) {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        for intent in &self.accepted_intents {
            intent.validate()?;
            if intent.target_gameplay_tick != self.gameplay_tick {
                return Err(PhysicsContractError::InvalidProfile);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.physics-step-input",
            "v2",
            [
                field_u16(1, self.schema_version),
                field_id(2, self.world_id.as_bytes()),
                field_u64(3, self.expected_world_revision),
                field_hash(4, self.expected_snapshot_hash),
                field_hash(5, self.expected_catalog_hash),
                field_u64(6, self.gameplay_tick),
                field_u64(7, self.first_physics_tick),
                field_u32(8, self.physics_substeps),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.accepted_intents
                            .iter()
                            .map(AcceptedLocomotionIntentV2::canonical_record)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            "nextengine.physics-step-input",
            "v2",
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_HASH256),
                (6, CANONICAL_TYPE_U64),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_U32),
                (9, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            world_id: PhysicsWorldId::from_bytes(exact(field(&segment, 2)?)?),
            expected_world_revision: read_u64(&segment, 3)?,
            expected_snapshot_hash: read_hash(&segment, 4)?,
            expected_catalog_hash: read_hash(&segment, 5)?,
            gameplay_tick: read_u64(&segment, 6)?,
            first_physics_tick: read_u64(&segment, 7)?,
            physics_substeps: u32::from_le_bytes(exact(field(&segment, 8)?)?),
            accepted_intents: decode_sequence(field(&segment, 9)?, limits)?
                .into_iter()
                .map(|record| AcceptedLocomotionIntentV2::from_record(&record, limits))
                .collect::<Result<Vec<_>, _>>()?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn input_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-step-input.v2\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContactEventV1 {
    pub gameplay_tick: u64,
    pub physics_tick: u64,
    pub substep: u32,
    pub participant_low: PhysicsShapeIdV1,
    pub participant_high: PhysicsShapeIdV1,
    pub feature_low: u8,
    pub feature_high: u8,
    pub point_micrometres: [i64; 3],
    pub normal_low_to_high_q1_30: [i32; 3],
    pub phase: ContactPhaseV1,
    pub contact_id: PhysicsContactId,
    pub source_snapshot_hash: ContentHash,
}

impl ContactEventV1 {
    fn validate_identity(&self) -> Result<(), PhysicsContractError> {
        if self.participant_low >= self.participant_high
            || derive_physics_contact_id(
                self.participant_low,
                self.participant_high,
                self.feature_low,
                self.feature_high,
            ) != self.contact_id
        {
            return Err(PhysicsContractError::ContactIdentityMismatch);
        }
        Ok(())
    }

    fn validate_against_catalog(
        &self,
        catalog: &PhysicsWorldCatalogV1,
    ) -> Result<(), PhysicsContractError> {
        self.validate_identity()?;
        let low = catalog_shape(catalog, self.participant_low)
            .ok_or(PhysicsContractError::ReferenceInvalid)?;
        let high = catalog_shape(catalog, self.participant_high)
            .ok_or(PhysicsContractError::ReferenceInvalid)?;
        if !primitive_feature_is_valid(&low.geometry, self.feature_low)
            || !primitive_feature_is_valid(&high.geometry, self.feature_high)
        {
            return Err(PhysicsContractError::ReferenceInvalid);
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_id(1, self.contact_id.as_bytes()),
            field_u64(2, self.gameplay_tick),
            field_u64(3, self.physics_tick),
            field_u32(4, self.substep),
            field_u8(5, self.phase as u8),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_STRUCT,
                self.participant_low.canonical_record()?,
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_STRUCT,
                self.participant_high.canonical_record()?,
            ),
            field_u8(8, self.feature_low),
            field_u8(9, self.feature_high),
            CanonicalField::new(
                10,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.point_micrometres),
            ),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_BYTES,
                encode_i32_vec3(self.normal_low_to_high_q1_30),
            ),
            field_hash(12, self.source_snapshot_hash),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_ID128),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_U8),
                (6, CANONICAL_TYPE_STRUCT),
                (7, CANONICAL_TYPE_STRUCT),
                (8, CANONICAL_TYPE_U8),
                (9, CANONICAL_TYPE_U8),
                (10, CANONICAL_TYPE_BYTES),
                (11, CANONICAL_TYPE_BYTES),
                (12, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            contact_id: PhysicsContactId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            gameplay_tick: read_u64_fields(&fields, 2)?,
            physics_tick: read_u64_fields(&fields, 3)?,
            substep: read_u32_fields(&fields, 4)?,
            phase: ContactPhaseV1::from_tag(read_u8_fields(&fields, 5)?)?,
            participant_low: PhysicsShapeIdV1::from_record(
                &field_from(&fields, 6)?.payload,
                limits,
            )?,
            participant_high: PhysicsShapeIdV1::from_record(
                &field_from(&fields, 7)?.payload,
                limits,
            )?,
            feature_low: read_u8_fields(&fields, 8)?,
            feature_high: read_u8_fields(&fields, 9)?,
            point_micrometres: decode_i64_vec3(&field_from(&fields, 10)?.payload)?,
            normal_low_to_high_q1_30: decode_i32_vec3(&field_from(&fields, 11)?.payload)?,
            source_snapshot_hash: read_hash_fields(&fields, 12)?,
        };
        value.validate_identity()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedPhysicsContactBatchV1 {
    pub schema_version: u16,
    pub gameplay_tick: u64,
    pub first_physics_tick: u64,
    pub substep_count: u32,
    pub events: Vec<ContactEventV1>,
    pub source_snapshot_hash: ContentHash,
    pub batch_hash: ContentHash,
}

impl ClosedPhysicsContactBatchV1 {
    pub fn new(
        gameplay_tick: u64,
        first_physics_tick: u64,
        substep_count: u32,
        events: Vec<ContactEventV1>,
        source_snapshot_hash: ContentHash,
    ) -> Result<Self, PhysicsContractError> {
        let mut value = Self {
            schema_version: CLOSED_PHYSICS_CONTACT_BATCH_SCHEMA_VERSION,
            gameplay_tick,
            first_physics_tick,
            substep_count,
            events,
            source_snapshot_hash,
            batch_hash: ContentHash::default(),
        };
        value.batch_hash = value.compute_batch_hash()?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != CLOSED_PHYSICS_CONTACT_BATCH_SCHEMA_VERSION
            || self.substep_count == 0
            || self.events.windows(2).any(|pair| pair[0] >= pair[1])
            || self.events.iter().any(|event| {
                event.gameplay_tick != self.gameplay_tick
                    || event.substep >= self.substep_count
                    || self
                        .first_physics_tick
                        .checked_add(u64::from(event.substep))
                        != Some(event.physics_tick)
                    || event.validate_identity().is_err()
            })
            || self.compute_batch_hash()? != self.batch_hash
        {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn validate_against_catalog(
        &self,
        catalog: &PhysicsWorldCatalogV1,
    ) -> Result<(), PhysicsContractError> {
        self.validate()?;
        if self.events.len()
            > usize::try_from(catalog.limits_profile.maximum_contacts_per_substep)
                .map_err(|_| PhysicsContractError::LimitExceeded)?
                .checked_mul(
                    usize::try_from(self.substep_count)
                        .map_err(|_| PhysicsContractError::LimitExceeded)?,
                )
                .ok_or(PhysicsContractError::LimitExceeded)?
        {
            return Err(PhysicsContractError::LimitExceeded);
        }
        for event in &self.events {
            event.validate_against_catalog(catalog)?;
        }
        Ok(())
    }

    fn body_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u16(1, self.schema_version),
            field_u64(2, self.gameplay_tick),
            field_u64(3, self.first_physics_tick),
            field_u32(4, self.substep_count),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.events
                        .iter()
                        .map(ContactEventV1::canonical_record)
                        .collect::<Result<Vec<_>, _>>()?,
                )?,
            ),
            field_hash(6, self.source_snapshot_hash),
        ])
    }

    fn compute_batch_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-contact-batch.v1\0",
            &self.body_bytes()?,
        )
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.closed-physics-contact-batch",
            SEGMENT_V1,
            [
                CanonicalField::new(1, CANONICAL_TYPE_STRUCT, self.body_bytes()?),
                field_hash(2, self.batch_hash),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            "nextengine.closed-physics-contact-batch",
            SEGMENT_V1,
            &[(1, CANONICAL_TYPE_STRUCT), (2, CANONICAL_TYPE_HASH256)],
        )?;
        let fields = decode_struct(field(&segment, 1)?, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16_fields(&fields, 1)?,
            gameplay_tick: read_u64_fields(&fields, 2)?,
            first_physics_tick: read_u64_fields(&fields, 3)?,
            substep_count: read_u32_fields(&fields, 4)?,
            events: decode_sequence(&field_from(&fields, 5)?.payload, limits)?
                .into_iter()
                .map(|record| ContactEventV1::from_record(&record, limits))
                .collect::<Result<Vec<_>, _>>()?,
            source_snapshot_hash: read_hash_fields(&fields, 6)?,
            batch_hash: read_hash(&segment, 2)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedLocomotionResultV1 {
    pub causal_command_id: CommandId,
    pub body_id: PhysicsBodyIdV1,
    pub requested_direction_q15: [i16; 2],
    pub before_state_hash: ContentHash,
    pub after_state_hash: ContentHash,
    pub applied_displacement_micrometres: [i64; 3],
    pub related_contact_ids: Vec<PhysicsContactId>,
}

impl AppliedLocomotionResultV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut direction = Vec::with_capacity(4);
        direction.extend_from_slice(&self.requested_direction_q15[0].to_le_bytes());
        direction.extend_from_slice(&self.requested_direction_q15[1].to_le_bytes());
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.applied-locomotion-result",
            SEGMENT_V1,
            [
                field_id(1, self.causal_command_id.as_bytes()),
                CanonicalField::new(2, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, direction),
                field_hash(4, self.before_state_hash),
                field_hash(5, self.after_state_hash),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_BYTES,
                    encode_i64_vec3(self.applied_displacement_micrometres),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.related_contact_ids
                            .iter()
                            .map(|id| id.as_bytes().to_vec())
                            .collect(),
                    )?,
                ),
            ],
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsStepResultV1 {
    pub step_input_hash: ContentHash,
    pub before_snapshot_hash: ContentHash,
    pub after_snapshot_hash: ContentHash,
    pub applied_locomotion: Vec<AppliedLocomotionResultV1>,
    pub contact_batch: ClosedPhysicsContactBatchV1,
}

#[must_use]
pub fn derive_physics_contact_id(
    participant_low: PhysicsShapeIdV1,
    participant_high: PhysicsShapeIdV1,
    feature_low: u8,
    feature_high: u8,
) -> PhysicsContactId {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.physics-contact-id.v1\0");
    preimage.extend_from_slice(&participant_low.canonical_key_bytes());
    preimage.extend_from_slice(&participant_high.canonical_key_bytes());
    preimage.push(feature_low);
    preimage.push(feature_high);
    let hash = sha256(&preimage);
    let mut id = [0; 16];
    id.copy_from_slice(&hash[..16]);
    PhysicsContactId::from_bytes(id)
}

fn catalog_shape(
    catalog: &PhysicsWorldCatalogV1,
    shape_id: PhysicsShapeIdV1,
) -> Option<&PhysicsShapeDescriptorV1> {
    catalog.bodies.get(&shape_id.body_id)?.shapes.get(&shape_id)
}

fn primitive_feature_is_valid(geometry: &PhysicsGeometryV1, feature: u8) -> bool {
    match geometry {
        PhysicsGeometryV1::Box { .. } => (1..=26).contains(&feature),
        PhysicsGeometryV1::Sphere { .. } | PhysicsGeometryV1::Capsule { .. } => feature == 1,
        PhysicsGeometryV1::ConvexHull { .. }
        | PhysicsGeometryV1::TriangleMesh { .. }
        | PhysicsGeometryV1::HeightField { .. } => feature != 0,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhysicalEventV1 {
    CapsuleStepApplied {
        body_id: PersistentId,
        physics_tick: u64,
        before: PhysicsPoseV1,
        after: PhysicsPoseV1,
    },
}

impl PhysicalEventV1 {
    #[must_use]
    pub const fn schema_id(&self) -> &'static str {
        PHYSICAL_EVENT_SCHEMA_ID
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        match self {
            Self::CapsuleStepApplied {
                body_id,
                physics_tick,
                before,
                after,
            } => encode_canonical_segment(
                PHYSICS_OWNER_ID,
                PHYSICAL_EVENT_SCHEMA_ID,
                SEGMENT_V1,
                [
                    field_id(1, body_id.as_bytes()),
                    field_u64(2, *physics_tick),
                    CanonicalField::new(3, CANONICAL_TYPE_STRUCT, before.canonical_record()?),
                    CanonicalField::new(4, CANONICAL_TYPE_STRUCT, after.canonical_record()?),
                ],
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicsContractError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identifier(crate::IdentifierError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType,
    FieldLength,
    UnknownTag(u8),
    UnsupportedVersion(u32),
    InvalidProfile,
    InvalidRotation,
    DirectionOutOfProfile,
    NonCanonicalOrder,
    DuplicateKey,
    ProfileMismatch,
    UnsupportedPhysicalState,
    NonCanonicalEncoding,
    InvalidDescriptor,
    ReferenceInvalid,
    LimitExceeded,
    ContactIdentityMismatch,
    ReferenceProfileUnsupported,
}

impl PhysicsContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion(_) => "UNSUPPORTED_PHYSICS_VERSION",
            Self::ProfileMismatch => "PHYSICS_PROFILE_MISMATCH",
            Self::DirectionOutOfProfile => "INPUT_VALUE_OUT_OF_PROFILE",
            Self::InvalidDescriptor => "PHYS_DESCRIPTOR_INVALID",
            Self::ReferenceInvalid => "PHYS_REFERENCE_INVALID",
            Self::LimitExceeded => "PHYS_LIMIT_EXCEEDED",
            Self::ContactIdentityMismatch => "PHYS_CONTACT_IDENTITY_INVALID",
            Self::ReferenceProfileUnsupported => "PHYS_REFERENCE_PROFILE_UNSUPPORTED",
            _ => "PHYSICS_CONTRACT_INVALID",
        }
    }
}

impl Display for PhysicsContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "physics encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "physics canonicalization failed: {error}")
            }
            Self::Identifier(error) => write!(formatter, "physics identifier is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("physics envelope does not match"),
            Self::UnknownField(id) => write!(formatter, "unknown physics field {id}"),
            Self::MissingField(id) => write!(formatter, "missing physics field {id}"),
            Self::FieldType => formatter.write_str("physics field has the wrong type"),
            Self::FieldLength => formatter.write_str("physics field has the wrong length"),
            Self::UnknownTag(tag) => write!(formatter, "unknown physics tag {tag}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported physics version {version}")
            }
            Self::InvalidProfile => formatter.write_str("physics profile is invalid"),
            Self::InvalidRotation => {
                formatter.write_str("physics rotation is not exact Q1.30 unit")
            }
            Self::DirectionOutOfProfile => {
                formatter.write_str("locomotion direction is outside the axial profile")
            }
            Self::NonCanonicalOrder => formatter.write_str("physics collection order is invalid"),
            Self::DuplicateKey => formatter.write_str("physics map contains a duplicate key"),
            Self::ProfileMismatch => formatter.write_str("physics profile closure does not match"),
            Self::UnsupportedPhysicalState => {
                formatter.write_str("physical state is outside the capsule reference slice")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("physics value does not re-encode byte-exactly")
            }
            Self::InvalidDescriptor => formatter.write_str("physics descriptor is invalid"),
            Self::ReferenceInvalid => formatter.write_str("physics reference is invalid"),
            Self::LimitExceeded => formatter.write_str("physics limit is exceeded"),
            Self::ContactIdentityMismatch => {
                formatter.write_str("physics contact identity does not match its participants")
            }
            Self::ReferenceProfileUnsupported => {
                formatter.write_str("physics descriptor is outside the reference profile")
            }
        }
    }
}

impl Error for PhysicsContractError {}

impl From<CanonicalDecodeError> for PhysicsContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for PhysicsContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<crate::IdentifierError> for PhysicsContractError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::InputContractError> for PhysicsContractError {
    fn from(error: crate::InputContractError) -> Self {
        match error {
            crate::InputContractError::Canonical(error) => Self::Canonical(error),
            crate::InputContractError::Canonicalization(error) => Self::Canonicalization(error),
            _ => Self::InvalidProfile,
        }
    }
}

fn validate_positive(value: i64) -> Result<(), PhysicsContractError> {
    if !(1..=8_388_608_000_000).contains(&value) {
        return Err(PhysicsContractError::InvalidDescriptor);
    }
    Ok(())
}

fn validate_positive_vec3(values: [i64; 3]) -> Result<(), PhysicsContractError> {
    for value in values {
        validate_positive(value)?;
    }
    Ok(())
}

fn encode_i64_vec3(values: [i64; 3]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(24);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_i64_vec3(bytes: &[u8]) -> Result<[i64; 3], PhysicsContractError> {
    if bytes.len() != 24 {
        return Err(PhysicsContractError::FieldLength);
    }
    Ok([
        i64::from_le_bytes(exact(&bytes[0..8])?),
        i64::from_le_bytes(exact(&bytes[8..16])?),
        i64::from_le_bytes(exact(&bytes[16..24])?),
    ])
}

fn encode_i32_vec3(values: [i32; 3]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(12);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_i32_vec3(bytes: &[u8]) -> Result<[i32; 3], PhysicsContractError> {
    if bytes.len() != 12 {
        return Err(PhysicsContractError::FieldLength);
    }
    Ok([
        i32::from_le_bytes(exact(&bytes[0..4])?),
        i32::from_le_bytes(exact(&bytes[4..8])?),
        i32::from_le_bytes(exact(&bytes[8..12])?),
    ])
}

fn encode_asset_geometry(
    asset: AssetId,
    revision: u32,
    shape_index: u32,
    feature_hash: ContentHash,
) -> Result<Vec<u8>, CanonicalError> {
    encode_struct([
        field_id(1, asset.as_bytes()),
        field_u32(2, revision),
        field_u32(3, shape_index),
        field_hash(4, feature_hash),
    ])
}

fn decode_asset_geometry(
    fields: &[CanonicalField],
) -> Result<(AssetId, u32, u32, ContentHash), PhysicsContractError> {
    require_fields(
        fields,
        &[
            (1, CANONICAL_TYPE_ID128),
            (2, CANONICAL_TYPE_U32),
            (3, CANONICAL_TYPE_U32),
            (4, CANONICAL_TYPE_HASH256),
        ],
    )?;
    Ok((
        AssetId::from_bytes(exact(&field_from(fields, 1)?.payload)?),
        read_u32_fields(fields, 2)?,
        read_u32_fields(fields, 3)?,
        read_hash_fields(fields, 4)?,
    ))
}

fn descriptor_collection_hash(
    domain: &[u8],
    records: Vec<Vec<u8>>,
) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(
        &u64::try_from(records.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for record in records {
        preimage.extend_from_slice(
            &u64::try_from(record.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&record);
    }
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn physics_contract_hash(domain: &[u8], bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn decode_coordinate_profile(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<PhysicsCoordinateProfileV1, PhysicsContractError> {
    let segment = decode_contract(
        bytes,
        limits,
        PHYSICS_OWNER_ID,
        "nextengine.physics-coordinate-profile",
        SEGMENT_V1,
        &[(1, CANONICAL_TYPE_U16), (2, CANONICAL_TYPE_UTF8_NFC)],
    )?;
    let value = PhysicsCoordinateProfileV1 {
        schema_version: read_u16(&segment, 1)?,
        profile_id: SchemaId::new(read_utf8(&segment, 2)?)?,
    };
    if value.schema_version != 1 {
        return Err(PhysicsContractError::UnsupportedVersion(u32::from(
            value.schema_version,
        )));
    }
    require_round_trip(bytes, value.canonical_bytes()?)?;
    Ok(value)
}

fn decode_limits_profile(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<PhysicsLimitsProfileV1, PhysicsContractError> {
    let segment = decode_contract(
        bytes,
        limits,
        PHYSICS_OWNER_ID,
        "nextengine.physics-limits-profile",
        SEGMENT_V1,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_UTF8_NFC),
            (3, CANONICAL_TYPE_U32),
            (4, CANONICAL_TYPE_U32),
            (5, CANONICAL_TYPE_U32),
        ],
    )?;
    let value = PhysicsLimitsProfileV1 {
        schema_version: read_u16(&segment, 1)?,
        profile_id: SchemaId::new(read_utf8(&segment, 2)?)?,
        maximum_bodies: u32::from_le_bytes(exact(field(&segment, 3)?)?),
        maximum_shapes: u32::from_le_bytes(exact(field(&segment, 4)?)?),
        maximum_contacts_per_substep: u32::from_le_bytes(exact(field(&segment, 5)?)?),
    };
    value.validate()?;
    require_round_trip(bytes, value.canonical_bytes()?)?;
    Ok(value)
}

fn decode_solver_profile(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<PhysicsSolverSemanticsProfileV1, PhysicsContractError> {
    let segment = decode_contract(
        bytes,
        limits,
        PHYSICS_OWNER_ID,
        "nextengine.physics-solver-semantics-profile",
        SEGMENT_V1,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_UTF8_NFC),
            (3, CANONICAL_TYPE_BYTES),
        ],
    )?;
    let value = PhysicsSolverSemanticsProfileV1 {
        schema_version: read_u16(&segment, 1)?,
        profile_id: SchemaId::new(read_utf8(&segment, 2)?)?,
        gravity_micrometres_per_second_squared: decode_i64_vec3(field(&segment, 3)?)?,
    };
    value.validate()?;
    require_round_trip(bytes, value.canonical_bytes()?)?;
    Ok(value)
}

fn profile_hash(bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.profile.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
    fields: &[(u32, u8)],
) -> Result<DecodedCanonicalSegment, PhysicsContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != segment_id
    {
        return Err(PhysicsContractError::WrongEnvelope);
    }
    require_fields(&segment.fields, fields)?;
    Ok(segment)
}

fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), PhysicsContractError> {
    for field in actual {
        let Some((_, expected_tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(PhysicsContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *expected_tag {
            return Err(PhysicsContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(PhysicsContractError::MissingField(*id));
        }
    }
    Ok(())
}

fn field(segment: &DecodedCanonicalSegment, id: u32) -> Result<&[u8], PhysicsContractError> {
    Ok(&segment
        .field(id)
        .ok_or(PhysicsContractError::MissingField(id))?
        .payload)
}

fn field_from(fields: &[CanonicalField], id: u32) -> Result<&CanonicalField, PhysicsContractError> {
    fields
        .binary_search_by_key(&id, |field| field.field_id)
        .ok()
        .map(|index| &fields[index])
        .ok_or(PhysicsContractError::MissingField(id))
}

fn read_u8(segment: &DecodedCanonicalSegment, id: u32) -> Result<u8, PhysicsContractError> {
    Ok(exact::<1>(field(segment, id)?)?[0])
}

fn read_u16(segment: &DecodedCanonicalSegment, id: u32) -> Result<u16, PhysicsContractError> {
    Ok(u16::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_u64(segment: &DecodedCanonicalSegment, id: u32) -> Result<u64, PhysicsContractError> {
    Ok(u64::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_bool(segment: &DecodedCanonicalSegment, id: u32) -> Result<bool, PhysicsContractError> {
    match read_u8(segment, id)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(PhysicsContractError::FieldType),
    }
}

fn read_hash(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<ContentHash, PhysicsContractError> {
    Ok(content_hash_from_bytes(exact(field(segment, id)?)?))
}

fn read_utf8(segment: &DecodedCanonicalSegment, id: u32) -> Result<&str, PhysicsContractError> {
    std::str::from_utf8(field(segment, id)?)
        .map_err(|_| PhysicsContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

fn read_u8_fields(fields: &[CanonicalField], id: u32) -> Result<u8, PhysicsContractError> {
    Ok(exact::<1>(&field_from(fields, id)?.payload)?[0])
}

fn read_u16_fields(fields: &[CanonicalField], id: u32) -> Result<u16, PhysicsContractError> {
    Ok(u16::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

fn read_u32_fields(fields: &[CanonicalField], id: u32) -> Result<u32, PhysicsContractError> {
    Ok(u32::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

fn read_u64_fields(fields: &[CanonicalField], id: u32) -> Result<u64, PhysicsContractError> {
    Ok(u64::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

fn read_i64_fields(fields: &[CanonicalField], id: u32) -> Result<i64, PhysicsContractError> {
    Ok(i64::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

fn read_bool_fields(fields: &[CanonicalField], id: u32) -> Result<bool, PhysicsContractError> {
    match read_u8_fields(fields, id)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(PhysicsContractError::FieldType),
    }
}

fn read_utf8_fields(fields: &[CanonicalField], id: u32) -> Result<&str, PhysicsContractError> {
    std::str::from_utf8(&field_from(fields, id)?.payload)
        .map_err(|_| PhysicsContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

fn read_hash_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<ContentHash, PhysicsContractError> {
    Ok(content_hash_from_bytes(exact(
        &field_from(fields, id)?.payload,
    )?))
}

fn exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], PhysicsContractError> {
    bytes
        .try_into()
        .map_err(|_| PhysicsContractError::FieldLength)
}

fn field_u8(id: u32, value: u8) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U8, vec![value])
}

fn field_u16(id: u32, value: u16) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U16, value.to_le_bytes().to_vec())
}

fn field_u32(id: u32, value: u32) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U32, value.to_le_bytes().to_vec())
}

fn field_u64(id: u32, value: u64) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U64, value.to_le_bytes().to_vec())
}

fn field_i64(id: u32, value: i64) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_I64, value.to_le_bytes().to_vec())
}

fn field_bool(id: u32, value: bool) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_BOOL, vec![u8::from(value)])
}

fn field_id<const N: usize>(id: u32, value: &[u8; N]) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.to_vec())
}

fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

fn nested(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = vec![type_tag];
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

fn read_nested<'a>(
    cursor: &mut CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
) -> Result<(u8, &'a [u8]), PhysicsContractError> {
    let tag = cursor.read_u8()?;
    let length =
        usize::try_from(cursor.read_u64()?).map_err(|_| PhysicsContractError::FieldLength)?;
    if length > limits.max_field_payload_bytes {
        return Err(PhysicsContractError::FieldLength);
    }
    Ok((tag, cursor.read_exact(length)?))
}

fn encode_struct(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    if fields
        .windows(2)
        .any(|pair| pair[0].field_id == pair[1].field_id)
    {
        return Err(CanonicalError::DuplicateField(0));
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        bytes.extend_from_slice(&field.field_id.to_le_bytes());
        bytes.push(field.type_tag);
        bytes.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&field.payload);
    }
    Ok(bytes)
}

fn decode_struct(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CanonicalField>, PhysicsContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let id = cursor.read_u32()?;
        if previous.is_some_and(|previous| id <= previous) {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        previous = Some(id);
        let tag = cursor.read_u8()?;
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| PhysicsContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(PhysicsContractError::FieldLength);
        }
        fields.push(CanonicalField::new(
            id,
            tag,
            cursor.read_exact(length)?.to_vec(),
        ));
    }
    cursor.finish()?;
    Ok(fields)
}

fn encode_sequence(items: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(items.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for item in items {
        bytes.extend_from_slice(
            &u64::try_from(item.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&item);
    }
    Ok(bytes)
}

fn decode_sequence(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<Vec<u8>>, PhysicsContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| PhysicsContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(PhysicsContractError::FieldLength);
        }
        items.push(cursor.read_exact(length)?.to_vec());
    }
    cursor.finish()?;
    Ok(items)
}

fn require_round_trip(original: &[u8], encoded: Vec<u8>) -> Result<(), PhysicsContractError> {
    if original != encoded {
        return Err(PhysicsContractError::NonCanonicalEncoding);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(byte: u8) -> ContentHash {
        content_hash_from_bytes([byte; 32])
    }

    #[test]
    fn physical_command_accepts_only_axial_q15() {
        PhysicalCommandV1::SetCapsuleLocomotionIntent {
            direction_q15: [0, 32_767],
        }
        .validate()
        .expect("forward is valid");
        assert_eq!(
            PhysicalCommandV1::SetCapsuleLocomotionIntent {
                direction_q15: [32_767, 32_767],
            }
            .validate(),
            Err(PhysicsContractError::DirectionOutOfProfile)
        );
    }

    #[test]
    fn profiles_and_snapshot_round_trip() {
        let tick = TickRateProfileV1::at_30_hz();
        let quantization =
            PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
        numeric.validate().expect("numeric valid");
        let catalog = PhysicsWorldCatalogV1::new(
            PhysicsWorldId::from_bytes([1; 16]),
            PhysicsWorldCatalogProfilesV1 {
                coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinate"),
                limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
                solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
                tick_rate_hash: tick.profile_hash().expect("tick hash"),
                authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
                quantization_hash: quantization.profile_hash().expect("quantization hash"),
            },
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .expect("catalog");
        let snapshot =
            PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick, &numeric, &quantization)
                .expect("snapshot");
        snapshot
            .validate_profile_closure(&catalog, &tick, &numeric, &quantization)
            .expect("profile closure");
        let bytes = snapshot.canonical_bytes().expect("snapshot encode");
        assert_eq!(
            PhysicsCanonicalSnapshotV2::from_canonical_bytes(
                &bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("snapshot decode"),
            snapshot
        );
        let mut corrupt_profile = snapshot;
        corrupt_profile.tick_rate_profile_hash = hash(10);
        assert_eq!(
            corrupt_profile.validate_profile_closure(&catalog, &tick, &numeric, &quantization),
            Err(PhysicsContractError::ProfileMismatch)
        );
    }

    #[test]
    fn grounded_capsule_v2_quantization_is_structured_and_fail_closed() {
        let profile = PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("v2 profile");
        profile.validate().expect("v2 profile valid");
        let bytes = profile.canonical_bytes().expect("encode v2 profile");
        assert_eq!(
            PhysicsQuantizationProfileV1::from_canonical_bytes(
                &bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("decode v2 profile"),
            profile
        );
        assert_ne!(
            profile.profile_hash().expect("v2 hash"),
            PhysicsQuantizationProfileV1::capsule_reference_v1()
                .expect("v1 profile")
                .profile_hash()
                .expect("v1 hash")
        );

        let mut invalid_rule = profile.clone();
        invalid_rule
            .rules
            .get_mut(&SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID).expect("distance field ID"))
            .expect("distance rule")
            .scale_denominator = 0;
        assert_eq!(
            invalid_rule.validate(),
            Err(PhysicsContractError::InvalidProfile)
        );

        let mut mismatched_key = profile;
        let distance_id =
            SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID).expect("distance field ID");
        let distance = mismatched_key
            .rules
            .remove(&distance_id)
            .expect("distance rule");
        mismatched_key.rules.insert(
            SchemaId::new("nextengine.physics.raw.wrong-key").expect("wrong key"),
            distance,
        );
        assert_eq!(
            mismatched_key.validate(),
            Err(PhysicsContractError::DuplicateKey)
        );
    }

    #[test]
    fn legacy_physics_snapshot_v1_is_rejected_from_header_only() {
        let bytes = encode_canonical_segment(
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_SNAPSHOT_SCHEMA_ID,
            LEGACY_PHYSICS_SNAPSHOT_SEGMENT_ID,
            [field_u16(1, LEGACY_PHYSICS_SNAPSHOT_SCHEMA_VERSION)],
        )
        .expect("legacy header");
        assert_eq!(
            PhysicsCanonicalSnapshotV2::from_canonical_bytes(
                &bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(PhysicsContractError::UnsupportedVersion(1))
        );
    }
}
