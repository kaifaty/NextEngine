use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_UTF8_NFC,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, encode_canonical_segment, sha256,
};
use crate::{ContentHash, PersistentId, PhysicsWorldId, SchemaId, content_hash_from_bytes};

use super::codec::*;
use super::descriptors::{
    PhysicsBodyDescriptorV1, PhysicsMaterialDescriptorV1, PhysicsShapeDescriptorV1,
};
use super::error::PhysicsContractError;
use super::primitives::{PhysicsBodyIdV1, PhysicsShapeIdV1};
use super::{PHYSICS_OWNER_ID, REFERENCE_GRAVITY_MICROMETRES_PER_SECOND_SQUARED, SEGMENT_V1};

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
    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
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

pub(super) fn catalog_shape(
    catalog: &PhysicsWorldCatalogV1,
    shape_id: PhysicsShapeIdV1,
) -> Option<&PhysicsShapeDescriptorV1> {
    catalog.bodies.get(&shape_id.body_id)?.shapes.get(&shape_id)
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
