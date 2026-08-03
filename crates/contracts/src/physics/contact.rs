use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    encode_canonical_segment, sha256,
};
use crate::ids::{ContentHash, PhysicsContactId};

use super::catalog::{PhysicsWorldCatalogV1, catalog_shape};
use super::codec::*;
use super::error::PhysicsContractError;
use super::primitives::{PhysicsShapeIdV1, primitive_feature_is_valid};
use super::{CLOSED_PHYSICS_CONTACT_BATCH_SCHEMA_VERSION, PHYSICS_OWNER_ID, SEGMENT_V1};

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
    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
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

    pub(super) fn from_record(
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
        // A fresh constructor result always satisfies the hash self-check
        // inside `validate()`; the structural checks still run here.
        value.validate_structure()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        self.validate_structure()?;
        if self.compute_batch_hash()? != self.batch_hash {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    fn validate_structure(&self) -> Result<(), PhysicsContractError> {
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
        self.validate_events_against_catalog(catalog)
    }

    /// Validates only the catalog-relative invariants (event count limit and
    /// per-event catalog membership). The caller must guarantee the batch
    /// already satisfies [`Self::validate`], for example because it is a
    /// fresh constructor result; decoded or otherwise untrusted batches must
    /// use [`Self::validate_against_catalog`].
    pub fn validate_events_against_catalog(
        &self,
        catalog: &PhysicsWorldCatalogV1,
    ) -> Result<(), PhysicsContractError> {
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
