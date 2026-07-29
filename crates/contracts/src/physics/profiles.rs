use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I64, CANONICAL_TYPE_MAP,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U64,
    CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    encode_canonical_segment,
};
use crate::{ContentHash, SchemaId};

use super::codec::*;
use super::error::PhysicsContractError;
use super::{
    AUTHORITATIVE_NUMERIC_PROFILE_SCHEMA_VERSION, NUMERIC_OWNER_ID, NUMERIC_PROFILE_SCHEMA_ID,
    PHYSICS_CONTACT_NORMAL_X_FIELD_ID, PHYSICS_CONTACT_NORMAL_Y_FIELD_ID,
    PHYSICS_CONTACT_NORMAL_Z_FIELD_ID, PHYSICS_METRES_UNIT_ID, PHYSICS_MICROMETRES_FIXED_POINT_ID,
    PHYSICS_MICROMETRES_UNIT_ID, PHYSICS_OWNER_ID, PHYSICS_Q1_30_FIXED_POINT_ID,
    PHYSICS_QUANTIZATION_PROFILE_SCHEMA_VERSION, PHYSICS_SCALAR_UNIT_ID,
    PHYSICS_SWEEP_DISTANCE_FIELD_ID, QUANTIZATION_PROFILE_SCHEMA_ID, SEGMENT_V1,
};

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
