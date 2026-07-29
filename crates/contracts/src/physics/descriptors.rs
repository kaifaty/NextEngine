use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_MAP, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeError, CanonicalDecodeLimits,
    CanonicalError, CanonicalField,
};
use crate::ids::SchemaId;

use super::codec::*;
use super::error::PhysicsContractError;
use super::primitives::{
    PhysicsBodyIdV1, PhysicsContactReportingV1, PhysicsGeometryV1, PhysicsMotionKindV1,
    PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeIdV1,
};

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

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
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

    pub(super) fn from_record(
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

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
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

    pub(super) fn from_record(
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
