//! ADR-105: the exact-level buoyancy reaction batch, the first one-pass
//! coupling consumer of the water table.
//!
//! The physics owner computes one `WaterBuoyancyBatchV1` per gameplay tick
//! from the committed water levels and the committed canonical body poses
//! of the previous tick; its records ride the canonical step input as
//! `ExternalImpulseV1` and are applied exactly once at the first substep.
//! Everything here is integer arithmetic over the ADR-081 exchange tuple;
//! nothing reads presentation state.

use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I64, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64,
    CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalError, CanonicalField,
};
use crate::ids::{ContentHash, PersistentId, PhysicsWorldId, SchemaId};

use super::catalog::PhysicsWorldCatalogV1;
use super::codec::*;
use super::error::PhysicsContractError;
use super::primitives::PhysicsBodyIdV1;
use super::primitives::{PhysicsGeometryV1, PhysicsMotionKindV1};
use super::snapshot::PhysicsCanonicalSnapshotV2;
use super::water::WaterVolumeSetV1;

/// ADR-105 bound: one record per dynamic body, at most this many per tick
/// (the water-volume bound).
pub const WATER_BUOYANCY_MAX_RECORDS: usize = 64;
/// Every impulse component must lie in `-MAX..=MAX` micronewton-seconds
/// (`10^9 N s`); a larger value is an overflow fault.
pub const MAX_EXTERNAL_IMPULSE_MICRONEWTON_SECONDS: i64 = 1_000_000_000_000_000;
/// Exchange tuple constants of the water-buoyancy edge (ADR-081 tuple).
pub const WATER_BUOYANCY_EXCHANGE_NAMESPACE: &str = "nextengine.exchange.water-buoyancy";
pub const WATER_EXCHANGE_SOURCE_OWNER: &str = "nextengine.owner.water";
pub const WATER_EXCHANGE_DESTINATION_OWNER: &str = "nextengine.owner.physics";
pub const WATER_BUOYANCY_EDGE_PROFILE_ID: &str = "nextengine.water-buoyancy.v1";
pub const WATER_BUOYANCY_PROFILE_ID: &str = "nextengine.physics.water-buoyancy.reference-v1";

const MICROMETRES_CUBED_PER_CUBIC_MILLIMETRE: i128 = 1_000_000_000;
const MICRO_PER_UNIT: i128 = 1_000_000;

/// How a body's submerged volume is derived.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WaterBuoyancyBoundsRuleV1 {
    /// The body's canonical axis-aligned bounds (the union of its box
    /// shapes at the committed pose) clipped by the cell and the level
    /// plane.
    CanonicalAabb = 1,
}

impl WaterBuoyancyBoundsRuleV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::CanonicalAabb),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

/// The batch profile: permille/integer constants that are data, not code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterBuoyancyProfileV1 {
    pub profile_id: SchemaId,
    pub profile_revision: u32,
    /// `rho_water` in kilograms per cubic metre.
    pub water_density_kilograms_per_cubic_metre: u32,
    /// Gravity magnitude used for the buoyant force, micrometres per
    /// second squared.
    pub gravity_micrometres_per_second_squared: i64,
    /// `k_damp`: the drag impulse is `-k * rho * V * v * dt` with `k` in
    /// permille per second.
    pub damping_permille_per_second: u32,
    pub bounds_rule: WaterBuoyancyBoundsRuleV1,
}

impl WaterBuoyancyProfileV1 {
    /// Plan `continuum-water/08`: `rho = 1000 kg/m^3`, `g = 9.81 m/s^2`,
    /// `k_damp = 2000` permille per second, canonical bounds.
    pub fn reference_v1() -> Result<Self, PhysicsContractError> {
        Ok(Self {
            profile_id: SchemaId::new(WATER_BUOYANCY_PROFILE_ID)?,
            profile_revision: 1,
            water_density_kilograms_per_cubic_metre: 1_000,
            gravity_micrometres_per_second_squared: 9_810_000,
            damping_permille_per_second: 2_000,
            bounds_rule: WaterBuoyancyBoundsRuleV1::CanonicalAabb,
        })
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.profile_revision == 0
            || self.water_density_kilograms_per_cubic_metre == 0
            || self.water_density_kilograms_per_cubic_metre > 100_000
            || self.gravity_micrometres_per_second_squared <= 0
            || self.gravity_micrometres_per_second_squared > 1_000_000_000_000
            || self.damping_permille_per_second > 1_000_000
        {
            return Err(PhysicsContractError::WaterBuoyancyInvalid);
        }
        Ok(())
    }

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.profile_id.as_str().as_bytes().to_vec(),
            ),
            field_u32(2, self.profile_revision),
            field_u32(3, self.water_density_kilograms_per_cubic_metre),
            field_i64(4, self.gravity_micrometres_per_second_squared),
            field_u32(5, self.damping_permille_per_second),
            field_u8(6, self.bounds_rule as u8),
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
                (4, CANONICAL_TYPE_I64),
                (5, CANONICAL_TYPE_U32),
                (6, CANONICAL_TYPE_U8),
            ],
        )?;
        let value = Self {
            profile_id: SchemaId::new(read_utf8_fields(&fields, 1)?)?,
            profile_revision: read_u32_fields(&fields, 2)?,
            water_density_kilograms_per_cubic_metre: read_u32_fields(&fields, 3)?,
            gravity_micrometres_per_second_squared: read_i64_fields(&fields, 4)?,
            damping_permille_per_second: read_u32_fields(&fields, 5)?,
            bounds_rule: WaterBuoyancyBoundsRuleV1::from_tag(read_u8_fields(&fields, 6)?)?,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.water-buoyancy-profile.v1\0",
            &self.canonical_record()?,
        )
    }
}

/// The ADR-081 exchange tuple bound by every cross-owner record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterExchangeTupleV1 {
    pub namespace: SchemaId,
    pub source_owner: SchemaId,
    pub destination_owner: SchemaId,
    pub world_id: PhysicsWorldId,
    /// The highest committed water record revision of the source table.
    pub source_revision: u64,
    /// The committed water table hash the record was computed from.
    pub source_root: ContentHash,
    /// The committed physics world revision the record targets.
    pub destination_revision: u64,
    /// The committed physics snapshot hash the record targets.
    pub destination_root: ContentHash,
    pub gameplay_tick: u64,
    pub substep: u32,
    pub edge_profile: SchemaId,
    pub body_id: PhysicsBodyIdV1,
    pub operation_slot: u32,
}

/// The committed roots an exchange tuple binds; supplied by the physics
/// owner when it computes the batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterExchangeContextV1 {
    pub world_id: PhysicsWorldId,
    pub source_revision: u64,
    pub source_root: ContentHash,
    pub destination_revision: u64,
    pub destination_root: ContentHash,
}

impl WaterExchangeTupleV1 {
    pub fn water_buoyancy(
        context: &WaterExchangeContextV1,
        gameplay_tick: u64,
        body_id: PhysicsBodyIdV1,
    ) -> Result<Self, PhysicsContractError> {
        Ok(Self {
            namespace: SchemaId::new(WATER_BUOYANCY_EXCHANGE_NAMESPACE)?,
            source_owner: SchemaId::new(WATER_EXCHANGE_SOURCE_OWNER)?,
            destination_owner: SchemaId::new(WATER_EXCHANGE_DESTINATION_OWNER)?,
            world_id: context.world_id,
            source_revision: context.source_revision,
            source_root: context.source_root,
            destination_revision: context.destination_revision,
            destination_root: context.destination_root,
            gameplay_tick,
            substep: 0,
            edge_profile: SchemaId::new(WATER_BUOYANCY_EDGE_PROFILE_ID)?,
            body_id,
            operation_slot: 0,
        })
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.namespace.as_str() != WATER_BUOYANCY_EXCHANGE_NAMESPACE
            || self.source_owner.as_str() != WATER_EXCHANGE_SOURCE_OWNER
            || self.destination_owner.as_str() != WATER_EXCHANGE_DESTINATION_OWNER
            || self.edge_profile.as_str() != WATER_BUOYANCY_EDGE_PROFILE_ID
            || self.substep != 0
            || self.operation_slot != 0
        {
            return Err(PhysicsContractError::WaterBuoyancyInvalid);
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            utf8_field(1, &self.namespace),
            utf8_field(2, &self.source_owner),
            utf8_field(3, &self.destination_owner),
            field_id(4, self.world_id.as_bytes()),
            field_u64(5, self.source_revision),
            field_hash(6, self.source_root),
            field_u64(7, self.destination_revision),
            field_hash(8, self.destination_root),
            field_u64(9, self.gameplay_tick),
            field_u32(10, self.substep),
            utf8_field(11, &self.edge_profile),
            CanonicalField::new(12, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            field_u32(13, self.operation_slot),
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
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_UTF8_NFC),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_U64),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_HASH256),
                (9, CANONICAL_TYPE_U64),
                (10, CANONICAL_TYPE_U32),
                (11, CANONICAL_TYPE_UTF8_NFC),
                (12, CANONICAL_TYPE_STRUCT),
                (13, CANONICAL_TYPE_U32),
            ],
        )?;
        let value = Self {
            namespace: SchemaId::new(read_utf8_fields(&fields, 1)?)?,
            source_owner: SchemaId::new(read_utf8_fields(&fields, 2)?)?,
            destination_owner: SchemaId::new(read_utf8_fields(&fields, 3)?)?,
            world_id: PhysicsWorldId::from_bytes(exact(&field_from(&fields, 4)?.payload)?),
            source_revision: read_u64_fields(&fields, 5)?,
            source_root: read_hash_fields(&fields, 6)?,
            destination_revision: read_u64_fields(&fields, 7)?,
            destination_root: read_hash_fields(&fields, 8)?,
            gameplay_tick: read_u64_fields(&fields, 9)?,
            substep: read_u32_fields(&fields, 10)?,
            edge_profile: SchemaId::new(read_utf8_fields(&fields, 11)?)?,
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 12)?.payload, limits)?,
            operation_slot: read_u32_fields(&fields, 13)?,
        };
        value.validate()?;
        Ok(value)
    }
}

fn utf8_field(id: u32, value: &SchemaId) -> CanonicalField {
    CanonicalField::new(
        id,
        CANONICAL_TYPE_UTF8_NFC,
        value.as_str().as_bytes().to_vec(),
    )
}

/// One external linear impulse delivered through the canonical step input
/// and applied once at the first substep of the tick.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalImpulseV1 {
    pub body_id: PhysicsBodyIdV1,
    pub impulse_micronewton_seconds: [i64; 3],
    pub application_point_micrometres: [i64; 3],
    pub exchange: WaterExchangeTupleV1,
}

impl ExternalImpulseV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self
            .impulse_micronewton_seconds
            .iter()
            .any(|component| component.abs() > MAX_EXTERNAL_IMPULSE_MICRONEWTON_SECONDS)
        {
            return Err(PhysicsContractError::WaterBuoyancyInvalid);
        }
        self.exchange.validate()?;
        if self.exchange.body_id != self.body_id {
            return Err(PhysicsContractError::ReferenceInvalid);
        }
        Ok(())
    }

    /// Checks the tuple against the step input the impulse rides.
    pub fn validate_against_step(
        &self,
        world_id: PhysicsWorldId,
        gameplay_tick: u64,
        expected_world_revision: u64,
        expected_snapshot_hash: ContentHash,
    ) -> Result<(), PhysicsContractError> {
        self.validate()?;
        if self.exchange.world_id != world_id
            || self.exchange.gameplay_tick != gameplay_tick
            || self.exchange.destination_revision != expected_world_revision
            || self.exchange.destination_root != expected_snapshot_hash
        {
            return Err(PhysicsContractError::ProfileMismatch);
        }
        Ok(())
    }

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.impulse_micronewton_seconds),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.application_point_micrometres),
            ),
            CanonicalField::new(4, CANONICAL_TYPE_STRUCT, self.exchange.canonical_record()?),
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
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_STRUCT),
            ],
        )?;
        let value = Self {
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?,
            impulse_micronewton_seconds: decode_i64_vec3(&field_from(&fields, 2)?.payload)?,
            application_point_micrometres: decode_i64_vec3(&field_from(&fields, 3)?.payload)?,
            exchange: WaterExchangeTupleV1::from_record(&field_from(&fields, 4)?.payload, limits)?,
        };
        value.validate()?;
        Ok(value)
    }
}

/// Validates a sorted external-impulse sequence of one step input.
pub fn validate_external_impulses(
    impulses: &[ExternalImpulseV1],
    world_id: PhysicsWorldId,
    gameplay_tick: u64,
    expected_world_revision: u64,
    expected_snapshot_hash: ContentHash,
) -> Result<(), PhysicsContractError> {
    if impulses.len() > WATER_BUOYANCY_MAX_RECORDS {
        return Err(PhysicsContractError::LimitExceeded);
    }
    if impulses
        .windows(2)
        .any(|pair| pair[0].body_id >= pair[1].body_id)
    {
        return Err(PhysicsContractError::NonCanonicalOrder);
    }
    for impulse in impulses {
        impulse.validate_against_step(
            world_id,
            gameplay_tick,
            expected_world_revision,
            expected_snapshot_hash,
        )?;
    }
    Ok(())
}

/// One buoyancy record: the displaced volume and the impulse it produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterBuoyancyRecordV1 {
    pub volume_id: PersistentId,
    pub displaced_volume_cubic_millimetres: i64,
    pub impulse: ExternalImpulseV1,
}

/// The batch of one gameplay tick, sorted by body id.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterBuoyancyBatchV1 {
    pub gameplay_tick: u64,
    pub records: Vec<WaterBuoyancyRecordV1>,
}

impl WaterBuoyancyBatchV1 {
    /// Computes the batch from committed state: for every active dynamic
    /// body whose canonical bounds intersect a water volume horizontally
    /// and lie below its effective level at `gameplay_tick`, the exact
    /// clipped volume, the buoyancy impulse `rho g V dt` upward at the
    /// clipped centroid and the drag impulse `-k rho V v dt`. `dt` is one
    /// gameplay tick. A body inside several volumes binds the one with
    /// the largest clipped volume (lowest id on ties). Bodies outside every
    /// volume receive no record.
    pub fn compute(
        profile: &WaterBuoyancyProfileV1,
        volumes: &WaterVolumeSetV1,
        catalog: &PhysicsWorldCatalogV1,
        snapshot: &PhysicsCanonicalSnapshotV2,
        gameplay_tick: u64,
        gameplay_hz: u32,
        context: &WaterExchangeContextV1,
    ) -> Result<Self, PhysicsContractError> {
        profile.validate()?;
        if gameplay_hz == 0 {
            return Err(PhysicsContractError::InvalidProfile);
        }
        let mut records = Vec::new();
        for (body_id, body) in &catalog.bodies {
            if body.motion_kind != PhysicsMotionKindV1::Dynamic {
                continue;
            }
            let Some(state) = snapshot.sorted_body_states.get(body_id) else {
                continue;
            };
            if !state.active {
                continue;
            }
            let Some(bounds) = body_bounds(body, state.pose.translation_micrometres)? else {
                continue;
            };
            let mut best: Option<(PersistentId, ClippedBoundsV1)> = None;
            for (volume_id, definition) in &volumes.definitions {
                let Some(level) = volumes.effective_level(*volume_id, gameplay_tick) else {
                    continue;
                };
                let Some(clipped) = clip_bounds(
                    &bounds,
                    definition.minimum_micrometres,
                    definition.maximum_micrometres,
                    level,
                )?
                else {
                    continue;
                };
                let better = match &best {
                    None => true,
                    Some((_, current)) => {
                        clipped.volume_cubic_millimetres > current.volume_cubic_millimetres
                    }
                };
                if better {
                    best = Some((*volume_id, clipped));
                }
            }
            let Some((volume_id, clipped)) = best else {
                continue;
            };
            if records.len() >= WATER_BUOYANCY_MAX_RECORDS {
                return Err(PhysicsContractError::LimitExceeded);
            }
            let impulse = impulse_for(
                profile,
                &clipped,
                state.linear_velocity_micrometres_per_second,
                gameplay_hz,
            )?;
            records.push(WaterBuoyancyRecordV1 {
                volume_id,
                displaced_volume_cubic_millimetres: clipped.volume_cubic_millimetres,
                impulse: ExternalImpulseV1 {
                    body_id: *body_id,
                    impulse_micronewton_seconds: impulse,
                    application_point_micrometres: clipped.centroid_micrometres,
                    exchange: WaterExchangeTupleV1::water_buoyancy(
                        context,
                        gameplay_tick,
                        *body_id,
                    )?,
                },
            });
        }
        let value = Self {
            gameplay_tick,
            records,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.records.len() > WATER_BUOYANCY_MAX_RECORDS {
            return Err(PhysicsContractError::LimitExceeded);
        }
        if self
            .records
            .windows(2)
            .any(|pair| pair[0].impulse.body_id >= pair[1].impulse.body_id)
        {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        for record in &self.records {
            if record.displaced_volume_cubic_millimetres <= 0
                || record.impulse.exchange.gameplay_tick != self.gameplay_tick
            {
                return Err(PhysicsContractError::WaterBuoyancyInvalid);
            }
            record.impulse.validate()?;
        }
        Ok(())
    }

    /// The impulses in step-input order.
    #[must_use]
    pub fn external_impulses(&self) -> Vec<ExternalImpulseV1> {
        self.records
            .iter()
            .map(|record| record.impulse.clone())
            .collect()
    }

    /// The record of one body, if any.
    #[must_use]
    pub fn record(&self, body_id: PhysicsBodyIdV1) -> Option<&WaterBuoyancyRecordV1> {
        self.records
            .iter()
            .find(|record| record.impulse.body_id == body_id)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Records keyed by body id.
    #[must_use]
    pub fn by_body(&self) -> BTreeMap<PhysicsBodyIdV1, &WaterBuoyancyRecordV1> {
        self.records
            .iter()
            .map(|record| (record.impulse.body_id, record))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BoundsV1 {
    minimum: [i64; 3],
    maximum: [i64; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClippedBoundsV1 {
    volume_cubic_millimetres: i64,
    centroid_micrometres: [i64; 3],
}

/// The union of the body's box shapes at its committed translation
/// (identity rotation in the bounded profile); `None` for a body without
/// box shapes.
fn body_bounds(
    body: &super::descriptors::PhysicsBodyDescriptorV1,
    translation_micrometres: [i64; 3],
) -> Result<Option<BoundsV1>, PhysicsContractError> {
    let mut bounds: Option<BoundsV1> = None;
    for shape in body.shapes.values() {
        let PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } = shape.geometry
        else {
            continue;
        };
        let mut minimum = [0_i64; 3];
        let mut maximum = [0_i64; 3];
        for axis in 0..3 {
            let centre = translation_micrometres[axis]
                .checked_add(shape.local_pose.translation_micrometres[axis])
                .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?;
            minimum[axis] = centre
                .checked_sub(half_extents_micrometres[axis])
                .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?;
            maximum[axis] = centre
                .checked_add(half_extents_micrometres[axis])
                .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?;
        }
        bounds = Some(match bounds {
            None => BoundsV1 { minimum, maximum },
            Some(current) => BoundsV1 {
                minimum: [
                    current.minimum[0].min(minimum[0]),
                    current.minimum[1].min(minimum[1]),
                    current.minimum[2].min(minimum[2]),
                ],
                maximum: [
                    current.maximum[0].max(maximum[0]),
                    current.maximum[1].max(maximum[1]),
                    current.maximum[2].max(maximum[2]),
                ],
            },
        });
    }
    Ok(bounds)
}

/// Clips the bounds to the cell and to the level plane; `None` when no
/// volume is displaced.
fn clip_bounds(
    bounds: &BoundsV1,
    cell_minimum: [i64; 3],
    cell_maximum: [i64; 3],
    level_micrometres: i64,
) -> Result<Option<ClippedBoundsV1>, PhysicsContractError> {
    let mut low = [0_i64; 3];
    let mut high = [0_i64; 3];
    for axis in 0..3 {
        low[axis] = bounds.minimum[axis].max(cell_minimum[axis]);
        high[axis] = bounds.maximum[axis].min(cell_maximum[axis]);
    }
    high[1] = high[1].min(level_micrometres);
    if (0..3).any(|axis| high[axis] <= low[axis]) {
        return Ok(None);
    }
    let mut volume: i128 = 1;
    let mut centroid = [0_i64; 3];
    for axis in 0..3 {
        let extent = i128::from(high[axis]) - i128::from(low[axis]);
        volume = volume
            .checked_mul(extent)
            .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?;
        centroid[axis] = low[axis]
            .checked_add(high[axis])
            .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?
            / 2;
    }
    let volume_cubic_millimetres = i64::try_from(volume / MICROMETRES_CUBED_PER_CUBIC_MILLIMETRE)
        .map_err(|_| PhysicsContractError::WaterBuoyancyInvalid)?;
    if volume_cubic_millimetres <= 0 {
        return Ok(None);
    }
    Ok(Some(ClippedBoundsV1 {
        volume_cubic_millimetres,
        centroid_micrometres: centroid,
    }))
}

/// `J_y = rho g V / hz` upward and `J = -k rho V v / (1000 hz)` per axis,
/// in micronewton-seconds (`rho` kg/m^3, `g` um/s^2, `V` mm^3, `v` um/s).
fn impulse_for(
    profile: &WaterBuoyancyProfileV1,
    clipped: &ClippedBoundsV1,
    velocity_micrometres_per_second: [i64; 3],
    gameplay_hz: u32,
) -> Result<[i64; 3], PhysicsContractError> {
    let rho = i128::from(profile.water_density_kilograms_per_cubic_metre);
    let volume = i128::from(clipped.volume_cubic_millimetres);
    let hz = i128::from(gameplay_hz);
    // rho [kg/m^3] * g [um/s^2] * V [mm^3] = 1e-15 N; * 1e6 / hz -> uN s.
    let buoyancy = rho
        .checked_mul(i128::from(profile.gravity_micrometres_per_second_squared))
        .and_then(|value| value.checked_mul(volume))
        .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?
        / (MICROMETRES_CUBED_PER_CUBIC_MILLIMETRE * hz);
    let mut impulse = [0_i128; 3];
    for axis in 0..3 {
        // k [permille/s] * rho * V * v [um/s]: 1e-3 * kg * 1e-9 * 1e-6 m/s
        // = 1e-18 N s per second; / hz then * 1e6 -> uN s.
        let drag = i128::from(profile.damping_permille_per_second)
            .checked_mul(rho)
            .and_then(|value| value.checked_mul(volume))
            .and_then(|value| value.checked_mul(i128::from(velocity_micrometres_per_second[axis])))
            .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?
            / (MICROMETRES_CUBED_PER_CUBIC_MILLIMETRE * 1_000 * hz);
        impulse[axis] = -drag;
    }
    impulse[1] = impulse[1]
        .checked_add(buoyancy)
        .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?;
    let mut result = [0_i64; 3];
    for axis in 0..3 {
        result[axis] =
            i64::try_from(impulse[axis]).map_err(|_| PhysicsContractError::WaterBuoyancyInvalid)?;
        if result[axis].abs() > MAX_EXTERNAL_IMPULSE_MICRONEWTON_SECONDS {
            return Err(PhysicsContractError::WaterBuoyancyInvalid);
        }
    }
    Ok(result)
}

/// Velocity change of a body of `mass_microkilograms` under `impulse`
/// (micronewton-seconds), in micrometres per second: `J / m` metres per
/// second scaled by `10^6`, truncated toward zero.
pub fn velocity_delta_micrometres_per_second(
    impulse_micronewton_seconds: i64,
    mass_microkilograms: u64,
) -> Result<i64, PhysicsContractError> {
    if mass_microkilograms == 0 {
        return Err(PhysicsContractError::InvalidDescriptor);
    }
    let delta = i128::from(impulse_micronewton_seconds)
        .checked_mul(MICRO_PER_UNIT)
        .ok_or(PhysicsContractError::WaterBuoyancyInvalid)?
        / i128::from(mass_microkilograms);
    i64::try_from(delta).map_err(|_| PhysicsContractError::WaterBuoyancyInvalid)
}
