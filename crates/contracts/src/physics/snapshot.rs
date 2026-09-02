use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_MAP, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{ContentHash, PhysicsContactId, PhysicsWorldId};
use crate::input::TickRateProfileV1;

use super::catalog::{PhysicsWorldCatalogV1, catalog_shape};
use super::codec::*;
use super::contact::{PhysicsContactContinuityStateV1, derive_physics_contact_id};
use super::error::PhysicsContractError;
use super::primitives::{
    PhysicsBodyIdV1, PhysicsGeometryV1, PhysicsMotionKindV1, PhysicsPoseV1,
    primitive_feature_is_valid,
};
use super::profiles::{AuthoritativeNumericProfileV1, PhysicsQuantizationProfileV1};
use super::water::WaterVolumeSetV1;
use super::water_flow::WaterFlowNetworkV1;
use super::{
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_SNAPSHOT_SCHEMA_ID, PHYSICS_SNAPSHOT_SCHEMA_VERSION,
    PHYSICS_SNAPSHOT_SEGMENT_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
};

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
    /// ADR-100 authoritative water table; empty for worlds without water.
    pub water_volumes: WaterVolumeSetV1,
    /// ADR-103 authoritative flow network over the water table; empty for
    /// worlds whose water does not move.
    pub water_flow: WaterFlowNetworkV1,
}

impl PhysicsWorldCheckpointV1 {
    pub fn new(
        catalog: PhysicsWorldCatalogV1,
        snapshot: PhysicsCanonicalSnapshotV2,
    ) -> Result<Self, PhysicsContractError> {
        Self::with_water_volumes(catalog, snapshot, WaterVolumeSetV1::empty())
    }

    pub fn with_water_volumes(
        catalog: PhysicsWorldCatalogV1,
        snapshot: PhysicsCanonicalSnapshotV2,
        water_volumes: WaterVolumeSetV1,
    ) -> Result<Self, PhysicsContractError> {
        Self::with_water(
            catalog,
            snapshot,
            water_volumes,
            WaterFlowNetworkV1::empty(),
        )
    }

    pub fn with_water(
        catalog: PhysicsWorldCatalogV1,
        snapshot: PhysicsCanonicalSnapshotV2,
        water_volumes: WaterVolumeSetV1,
        water_flow: WaterFlowNetworkV1,
    ) -> Result<Self, PhysicsContractError> {
        let value = Self {
            schema_version: PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION,
            catalog,
            snapshot,
            water_volumes,
            water_flow,
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
        self.water_volumes.validate()?;
        self.water_flow.validate()?;
        self.water_flow.validate_against(&self.water_volumes)?;
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
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_STRUCT,
                    self.water_volumes.canonical_record()?,
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_STRUCT,
                    self.water_flow.canonical_record()?,
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
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_STRUCT),
                (5, CANONICAL_TYPE_STRUCT),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            catalog: PhysicsWorldCatalogV1::from_canonical_bytes(field(&segment, 2)?, limits)?,
            snapshot: PhysicsCanonicalSnapshotV2::from_canonical_bytes(
                field(&segment, 3)?,
                limits,
            )?,
            water_volumes: WaterVolumeSetV1::from_record(field(&segment, 4)?, limits)?,
            water_flow: WaterFlowNetworkV1::from_record(field(&segment, 5)?, limits)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn checkpoint_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-world-checkpoint.v3\0",
            &self.canonical_bytes()?,
        )
    }
}
