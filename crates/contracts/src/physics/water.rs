//! Authoritative water volumes (ADR-100).
//!
//! A `WaterVolumeSetV1` is the bounded set of sealed water regions owned by
//! Physical Embodiment inside `PhysicsWorldCheckpointV1`. Every field is
//! exact: integer micrometre extents, an integer still-water level, an
//! optional authored linear level ramp evaluated from the gameplay tick, and
//! a per-volume record revision that changes only through a validated
//! `WaterVolumeCommandV1`. Presentation water never writes here.

use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_I64, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP,
    CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalError, CanonicalField,
};
use crate::ids::{ContentHash, PersistentId};

use super::codec::*;
use super::error::PhysicsContractError;

pub const WATER_VOLUME_SCHEMA_VERSION: u16 = 1;
pub const WATER_VOLUME_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const WATER_VOLUME_EVENT_SCHEMA_VERSION: u32 = 1;
pub const WATER_VOLUME_COMMAND_SCHEMA_ID: &str = "nextengine.command.water-volume";
pub const WATER_VOLUME_COMMAND_KIND_ID: &str = "nextengine.command-kind.water-volume";
pub const WATER_VOLUME_EVENT_SCHEMA_ID: &str = "nextengine.event.water-volume-changed";
pub const WATER_VOLUME_CAPABILITY_ID: &str = "nextengine.capability.water-volume-level";
pub const WATER_VOLUME_PRIORITY_CLASS: u16 = 290;
/// Hard bound on sealed volumes in one physics world.
pub const MAX_WATER_VOLUMES: usize = 64;
/// SPEC-26 canonical position range, inclusive.
pub const WATER_POSITION_LIMIT_MICROMETRES: i64 = 8_388_608_000_000;

const WATER_VOLUME_COMMAND_SET_LEVEL_TAG: u8 = 1;
const WATER_VOLUME_COMMAND_SET_LEVEL_LENGTH: usize = 1 + 16 + 8 + 8;
const WATER_VOLUME_EVENT_LENGTH: usize = 16 + 8 + 8 + 8 + 8;

/// Authored linear level ramp. Before `start_tick` the level is
/// `start_level_micrometres`, after `end_tick` it is `end_level_micrometres`,
/// in between it is the exact integer interpolation truncated toward zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterLevelRampV1 {
    pub start_tick: u64,
    pub end_tick: u64,
    pub start_level_micrometres: i64,
    pub end_level_micrometres: i64,
}

impl WaterLevelRampV1 {
    pub fn validate(&self, minimum_y: i64, maximum_y: i64) -> Result<(), PhysicsContractError> {
        if self.end_tick <= self.start_tick
            || self.start_level_micrometres < minimum_y
            || self.start_level_micrometres > maximum_y
            || self.end_level_micrometres < minimum_y
            || self.end_level_micrometres > maximum_y
        {
            return Err(PhysicsContractError::WaterVolumeInvalid);
        }
        Ok(())
    }

    #[must_use]
    pub fn level_at(&self, tick: u64) -> i64 {
        if tick <= self.start_tick {
            return self.start_level_micrometres;
        }
        if tick >= self.end_tick {
            return self.end_level_micrometres;
        }
        let elapsed = i128::from(tick - self.start_tick);
        let duration = i128::from(self.end_tick - self.start_tick);
        let delta =
            i128::from(self.end_level_micrometres) - i128::from(self.start_level_micrometres);
        let interpolated = i128::from(self.start_level_micrometres) + delta * elapsed / duration;
        i64::try_from(interpolated).expect("interpolation stays between two i64 levels")
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u64(1, self.start_tick),
            field_u64(2, self.end_tick),
            field_i64(3, self.start_level_micrometres),
            field_i64(4, self.end_level_micrometres),
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
                (1, CANONICAL_TYPE_U64),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_I64),
                (4, CANONICAL_TYPE_I64),
            ],
        )?;
        Ok(Self {
            start_tick: read_u64_fields(&fields, 1)?,
            end_tick: read_u64_fields(&fields, 2)?,
            start_level_micrometres: read_i64_fields(&fields, 3)?,
            end_level_micrometres: read_i64_fields(&fields, 4)?,
        })
    }
}

/// Immutable authored definition of one sealed water region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterVolumeDefinitionV1 {
    pub volume_id: PersistentId,
    pub minimum_micrometres: [i64; 3],
    pub maximum_micrometres: [i64; 3],
    pub initial_level_micrometres: i64,
    /// Depth at or above which a submerged point classifies as swimming.
    pub swimming_depth_micrometres: i64,
    pub level_ramp: Option<WaterLevelRampV1>,
    pub profile_revision: u32,
}

impl WaterVolumeDefinitionV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        for axis in 0..3 {
            let minimum = self.minimum_micrometres[axis];
            let maximum = self.maximum_micrometres[axis];
            if minimum >= maximum
                || minimum < -WATER_POSITION_LIMIT_MICROMETRES
                || maximum > WATER_POSITION_LIMIT_MICROMETRES
            {
                return Err(PhysicsContractError::WaterVolumeInvalid);
            }
        }
        if !self.level_in_extent(self.initial_level_micrometres)
            || self.swimming_depth_micrometres <= 0
            || self.profile_revision == 0
        {
            return Err(PhysicsContractError::WaterVolumeInvalid);
        }
        if let Some(ramp) = &self.level_ramp {
            ramp.validate(self.minimum_micrometres[1], self.maximum_micrometres[1])?;
        }
        Ok(())
    }

    #[must_use]
    pub fn level_in_extent(&self, level_micrometres: i64) -> bool {
        level_micrometres >= self.minimum_micrometres[1]
            && level_micrometres <= self.maximum_micrometres[1]
    }

    #[must_use]
    pub fn contains_horizontally(&self, point_micrometres: [i64; 3]) -> bool {
        (0..3).filter(|axis| *axis != 1).all(|axis| {
            point_micrometres[axis] >= self.minimum_micrometres[axis]
                && point_micrometres[axis] <= self.maximum_micrometres[axis]
        })
    }

    /// Positive-measure intersection on every axis (SPEC-38 2.3, plan
    /// `continuum-water/19`): volumes that share a face, an edge or a
    /// corner are disjoint, so lattice cells may touch.
    fn overlaps(&self, other: &Self) -> bool {
        (0..3).all(|axis| {
            self.minimum_micrometres[axis] < other.maximum_micrometres[axis]
                && other.minimum_micrometres[axis] < self.maximum_micrometres[axis]
        })
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let ramp = match &self.level_ramp {
            Some(ramp) => nested(CANONICAL_TYPE_STRUCT, &ramp.canonical_record()?)?,
            None => Vec::new(),
        };
        encode_struct([
            field_id(1, self.volume_id.as_bytes()),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.minimum_micrometres),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.maximum_micrometres),
            ),
            field_i64(4, self.initial_level_micrometres),
            field_i64(5, self.swimming_depth_micrometres),
            CanonicalField::new(6, CANONICAL_TYPE_OPTIONAL, ramp),
            field_u32(7, self.profile_revision),
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
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_I64),
                (5, CANONICAL_TYPE_I64),
                (6, CANONICAL_TYPE_OPTIONAL),
                (7, CANONICAL_TYPE_U32),
            ],
        )?;
        let ramp_bytes = &field_from(&fields, 6)?.payload;
        let level_ramp = if ramp_bytes.is_empty() {
            None
        } else {
            let mut cursor = crate::canonical::CanonicalCursor::new(ramp_bytes);
            let (tag, payload) = read_nested(&mut cursor, limits)?;
            if tag != CANONICAL_TYPE_STRUCT {
                return Err(PhysicsContractError::FieldType);
            }
            cursor.finish()?;
            Some(WaterLevelRampV1::from_record(payload, limits)?)
        };
        let value = Self {
            volume_id: PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            minimum_micrometres: decode_i64_vec3(&field_from(&fields, 2)?.payload)?,
            maximum_micrometres: decode_i64_vec3(&field_from(&fields, 3)?.payload)?,
            initial_level_micrometres: read_i64_fields(&fields, 4)?,
            swimming_depth_micrometres: read_i64_fields(&fields, 5)?,
            level_ramp,
            profile_revision: read_u32_fields(&fields, 7)?,
        };
        value.validate()?;
        Ok(value)
    }
}

/// Mutable authoritative state of one water volume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterVolumeStateV1 {
    pub volume_id: PersistentId,
    /// Starts at `0` and increments once per committed level command.
    pub record_revision: u64,
    pub level_micrometres: i64,
    /// A committed level command suspends the authored ramp for good.
    pub ramp_suspended: bool,
}

impl WaterVolumeStateV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_id(1, self.volume_id.as_bytes()),
            field_u64(2, self.record_revision),
            field_i64(3, self.level_micrometres),
            field_bool(4, self.ramp_suspended),
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
                (3, CANONICAL_TYPE_I64),
                (4, crate::canonical::CANONICAL_TYPE_BOOL),
            ],
        )?;
        Ok(Self {
            volume_id: PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            record_revision: read_u64_fields(&fields, 2)?,
            level_micrometres: read_i64_fields(&fields, 3)?,
            ramp_suspended: read_bool_fields(&fields, 4)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WaterSubmersionClassV1 {
    #[default]
    Dry = 1,
    Wading = 2,
    Swimming = 3,
}

/// Exact answer of one submersion query against a committed water set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterSubmersionV1 {
    pub volume_id: Option<PersistentId>,
    pub level_micrometres: Option<i64>,
    /// `0` when dry; otherwise the exact `level - point.y`.
    pub depth_micrometres: i64,
    pub class: WaterSubmersionClassV1,
}

impl WaterSubmersionV1 {
    #[must_use]
    pub const fn dry() -> Self {
        Self {
            volume_id: None,
            level_micrometres: None,
            depth_micrometres: 0,
            class: WaterSubmersionClassV1::Dry,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WaterVolumeCommandV1 {
    SetLevel {
        volume_id: PersistentId,
        expected_record_revision: u64,
        level_micrometres: i64,
    },
}

impl WaterVolumeCommandV1 {
    #[must_use]
    pub const fn volume_id(&self) -> PersistentId {
        match self {
            Self::SetLevel { volume_id, .. } => *volume_id,
        }
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let Self::SetLevel {
            volume_id,
            expected_record_revision,
            level_micrometres,
        } = self;
        let mut bytes = Vec::with_capacity(WATER_VOLUME_COMMAND_SET_LEVEL_LENGTH);
        bytes.push(WATER_VOLUME_COMMAND_SET_LEVEL_TAG);
        bytes.extend_from_slice(volume_id.as_bytes());
        bytes.extend_from_slice(&expected_record_revision.to_le_bytes());
        bytes.extend_from_slice(&level_micrometres.to_le_bytes());
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        _limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        if bytes.len() != WATER_VOLUME_COMMAND_SET_LEVEL_LENGTH {
            return Err(PhysicsContractError::FieldLength);
        }
        if bytes[0] != WATER_VOLUME_COMMAND_SET_LEVEL_TAG {
            return Err(PhysicsContractError::UnknownTag(bytes[0]));
        }
        Ok(Self::SetLevel {
            volume_id: PersistentId::from_bytes(exact(&bytes[1..17])?),
            expected_record_revision: u64::from_le_bytes(exact(&bytes[17..25])?),
            level_micrometres: i64::from_le_bytes(exact(&bytes[25..33])?),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterVolumeChangedV1 {
    pub volume_id: PersistentId,
    pub record_revision: u64,
    pub previous_level_micrometres: i64,
    pub current_level_micrometres: i64,
    pub boundary_tick: u64,
}

impl WaterVolumeChangedV1 {
    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::with_capacity(WATER_VOLUME_EVENT_LENGTH);
        bytes.extend_from_slice(self.volume_id.as_bytes());
        bytes.extend_from_slice(&self.record_revision.to_le_bytes());
        bytes.extend_from_slice(&self.previous_level_micrometres.to_le_bytes());
        bytes.extend_from_slice(&self.current_level_micrometres.to_le_bytes());
        bytes.extend_from_slice(&self.boundary_tick.to_le_bytes());
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(bytes: &[u8]) -> Result<Self, PhysicsContractError> {
        if bytes.len() != WATER_VOLUME_EVENT_LENGTH {
            return Err(PhysicsContractError::FieldLength);
        }
        Ok(Self {
            volume_id: PersistentId::from_bytes(exact(&bytes[0..16])?),
            record_revision: u64::from_le_bytes(exact(&bytes[16..24])?),
            previous_level_micrometres: i64::from_le_bytes(exact(&bytes[24..32])?),
            current_level_micrometres: i64::from_le_bytes(exact(&bytes[32..40])?),
            boundary_tick: u64::from_le_bytes(exact(&bytes[40..48])?),
        })
    }
}

/// Deterministic rejection of one water command; never a fatal error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaterVolumeRejectionV1 {
    UnknownVolume,
    RevisionStale,
    LevelOutOfExtent,
    RevisionExhausted,
}

impl WaterVolumeRejectionV1 {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::UnknownVolume => "WATER_VOLUME_UNKNOWN",
            Self::RevisionStale => "WATER_VOLUME_REVISION_STALE",
            Self::LevelOutOfExtent => "WATER_VOLUME_LEVEL_OUT_OF_EXTENT",
            Self::RevisionExhausted => "WATER_VOLUME_REVISION_EXHAUSTED",
        }
    }
}

/// The bounded authoritative water table of one physics world.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterVolumeSetV1 {
    pub schema_version: u16,
    pub definitions: BTreeMap<PersistentId, WaterVolumeDefinitionV1>,
    pub states: BTreeMap<PersistentId, WaterVolumeStateV1>,
}

impl Default for WaterVolumeSetV1 {
    fn default() -> Self {
        Self::empty()
    }
}

impl WaterVolumeSetV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schema_version: WATER_VOLUME_SCHEMA_VERSION,
            definitions: BTreeMap::new(),
            states: BTreeMap::new(),
        }
    }

    /// Genesis state: every volume rests at its authored initial level with
    /// record revision `0` and its ramp active.
    pub fn from_definitions(
        definitions: impl IntoIterator<Item = WaterVolumeDefinitionV1>,
    ) -> Result<Self, PhysicsContractError> {
        let mut map = BTreeMap::new();
        let mut states = BTreeMap::new();
        for definition in definitions {
            definition.validate()?;
            let state = WaterVolumeStateV1 {
                volume_id: definition.volume_id,
                record_revision: 0,
                level_micrometres: definition.initial_level_micrometres,
                ramp_suspended: false,
            };
            if map.insert(definition.volume_id, definition).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
            states.insert(state.volume_id, state);
        }
        let value = Self {
            schema_version: WATER_VOLUME_SCHEMA_VERSION,
            definitions: map,
            states,
        };
        value.validate()?;
        Ok(value)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != WATER_VOLUME_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                self.schema_version,
            )));
        }
        if self.definitions.len() > MAX_WATER_VOLUMES {
            return Err(PhysicsContractError::LimitExceeded);
        }
        if !self.definitions.keys().eq(self.states.keys()) {
            return Err(PhysicsContractError::WaterVolumeInvalid);
        }
        let definitions: Vec<_> = self.definitions.values().collect();
        for (index, definition) in definitions.iter().enumerate() {
            definition.validate()?;
            if definition.volume_id != *self.definitions.keys().nth(index).expect("same length") {
                return Err(PhysicsContractError::WaterVolumeInvalid);
            }
            if definitions[..index]
                .iter()
                .any(|other| other.overlaps(definition))
            {
                return Err(PhysicsContractError::WaterVolumeInvalid);
            }
            let state = &self.states[&definition.volume_id];
            if state.volume_id != definition.volume_id
                || !definition.level_in_extent(state.level_micrometres)
            {
                return Err(PhysicsContractError::WaterVolumeInvalid);
            }
        }
        Ok(())
    }

    /// Exact level of one volume at a gameplay tick: the authored ramp while
    /// it is active, otherwise the committed record level.
    #[must_use]
    pub fn effective_level(&self, volume_id: PersistentId, tick: u64) -> Option<i64> {
        let definition = self.definitions.get(&volume_id)?;
        let state = self.states.get(&volume_id)?;
        match (&definition.level_ramp, state.ramp_suspended) {
            (Some(ramp), false) => Some(ramp.level_at(tick)),
            _ => Some(state.level_micrometres),
        }
    }

    /// Submersion of one point. Volumes are disjoint, so at most one can
    /// contain the point horizontally; a point above the level or outside
    /// every volume is dry.
    #[must_use]
    pub fn submersion_at(&self, point_micrometres: [i64; 3], tick: u64) -> WaterSubmersionV1 {
        for definition in self.definitions.values() {
            if !definition.contains_horizontally(point_micrometres)
                || point_micrometres[1] < definition.minimum_micrometres[1]
            {
                continue;
            }
            let Some(level) = self.effective_level(definition.volume_id, tick) else {
                continue;
            };
            let depth = level.saturating_sub(point_micrometres[1]);
            if depth <= 0 {
                return WaterSubmersionV1 {
                    volume_id: Some(definition.volume_id),
                    level_micrometres: Some(level),
                    depth_micrometres: 0,
                    class: WaterSubmersionClassV1::Dry,
                };
            }
            let class = if depth >= definition.swimming_depth_micrometres {
                WaterSubmersionClassV1::Swimming
            } else {
                WaterSubmersionClassV1::Wading
            };
            return WaterSubmersionV1 {
                volume_id: Some(definition.volume_id),
                level_micrometres: Some(level),
                depth_micrometres: depth,
                class,
            };
        }
        WaterSubmersionV1::dry()
    }

    /// Applies one validated command, returning the successor set and the
    /// committed event, or the deterministic rejection.
    pub fn apply_command(
        &self,
        command: &WaterVolumeCommandV1,
        boundary_tick: u64,
    ) -> Result<(Self, WaterVolumeChangedV1), WaterVolumeRejectionV1> {
        let WaterVolumeCommandV1::SetLevel {
            volume_id,
            expected_record_revision,
            level_micrometres,
        } = command;
        let definition = self
            .definitions
            .get(volume_id)
            .ok_or(WaterVolumeRejectionV1::UnknownVolume)?;
        let state = self
            .states
            .get(volume_id)
            .ok_or(WaterVolumeRejectionV1::UnknownVolume)?;
        if state.record_revision != *expected_record_revision {
            return Err(WaterVolumeRejectionV1::RevisionStale);
        }
        if !definition.level_in_extent(*level_micrometres) {
            return Err(WaterVolumeRejectionV1::LevelOutOfExtent);
        }
        let record_revision = state
            .record_revision
            .checked_add(1)
            .ok_or(WaterVolumeRejectionV1::RevisionExhausted)?;
        let previous_level_micrometres = self
            .effective_level(*volume_id, boundary_tick)
            .ok_or(WaterVolumeRejectionV1::UnknownVolume)?;
        let mut next = self.clone();
        next.states.insert(
            *volume_id,
            WaterVolumeStateV1 {
                volume_id: *volume_id,
                record_revision,
                level_micrometres: *level_micrometres,
                ramp_suspended: true,
            },
        );
        Ok((
            next,
            WaterVolumeChangedV1 {
                volume_id: *volume_id,
                record_revision,
                previous_level_micrometres,
                current_level_micrometres: *level_micrometres,
                boundary_tick,
            },
        ))
    }

    pub fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let definitions = self
            .definitions
            .values()
            .map(WaterVolumeDefinitionV1::canonical_record)
            .collect::<Result<Vec<_>, _>>()?;
        let states = self
            .states
            .values()
            .map(WaterVolumeStateV1::canonical_record)
            .collect::<Result<Vec<_>, _>>()?;
        encode_struct([
            field_u16(1, self.schema_version),
            CanonicalField::new(2, CANONICAL_TYPE_MAP, encode_sequence(definitions)?),
            CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_sequence(states)?),
        ])
    }

    pub fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_MAP),
                (3, CANONICAL_TYPE_MAP),
            ],
        )?;
        let schema_version = read_u16_fields(&fields, 1)?;
        if schema_version != WATER_VOLUME_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                schema_version,
            )));
        }
        let mut definitions = BTreeMap::new();
        let mut previous: Option<PersistentId> = None;
        for entry in decode_sequence(&field_from(&fields, 2)?.payload, limits)? {
            let definition = WaterVolumeDefinitionV1::from_record(&entry, limits)?;
            if previous.is_some_and(|previous| definition.volume_id <= previous) {
                return Err(PhysicsContractError::NonCanonicalOrder);
            }
            previous = Some(definition.volume_id);
            definitions.insert(definition.volume_id, definition);
        }
        let mut states = BTreeMap::new();
        let mut previous: Option<PersistentId> = None;
        for entry in decode_sequence(&field_from(&fields, 3)?.payload, limits)? {
            let state = WaterVolumeStateV1::from_record(&entry, limits)?;
            if previous.is_some_and(|previous| state.volume_id <= previous) {
                return Err(PhysicsContractError::NonCanonicalOrder);
            }
            previous = Some(state.volume_id);
            states.insert(state.volume_id, state);
        }
        let value = Self {
            schema_version,
            definitions,
            states,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_record()?)?;
        Ok(value)
    }

    pub fn set_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-water-volume-set.v1\0",
            &self.canonical_record()?,
        )
    }
}
