//! Authoritative water flow network (ADR-103).
//!
//! A `WaterFlowNetworkV1` joins the sealed `WaterVolume` cells of one
//! physics world with edges that move water by head: open sills, pipes,
//! gates, pumps, sources and sinks. Every quantity is an integer (levels in
//! micrometres, plan areas in square millimetres, volumes in cubic
//! millimetres, rates in cubic millimetres per second, coefficients in
//! permille) and the per-tick step is an exact Jacobi update with integer
//! square roots, so the network needs no floating-point execution profile
//! and conserves volume to the cubic millimetre. Presentation never writes
//! here.

use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_I64, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalDecodeLimits,
    CanonicalError, CanonicalField,
};
use crate::ids::{ContentHash, PersistentId};

use super::codec::*;
use super::error::PhysicsContractError;
use super::water::{WaterVolumeSetV1, WaterVolumeStateV1};

pub const WATER_FLOW_SCHEMA_VERSION: u16 = 1;
pub const WATER_FLOW_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const WATER_FLOW_EVENT_SCHEMA_VERSION: u32 = 1;
pub const WATER_FLOW_COMMAND_SCHEMA_ID: &str = "nextengine.command.water-flow";
pub const WATER_FLOW_COMMAND_KIND_ID: &str = "nextengine.command-kind.water-flow";
pub const WATER_FLOW_EVENT_SCHEMA_ID: &str = "nextengine.event.water-flow-changed";
pub const WATER_FLOW_CAPABILITY_ID: &str = "nextengine.capability.water-flow-control";
pub const WATER_FLOW_PRIORITY_CLASS: u16 = 291;
/// Hard bound on edges in one network.
pub const MAX_WATER_FLOW_EDGES: usize = 256;
/// Standard gravity in micrometres per second squared.
pub const WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED: i64 = 9_810_000;
/// Bound on any authored or commanded rate: one cubic metre per second.
pub const MAX_WATER_FLOW_RATE_CUBIC_MILLIMETRES_PER_SECOND: i64 = 1_000_000_000;
/// Bound on authored areas and widths (`1000 m^2`, `1000 m`).
const MAX_AREA_SQUARE_MILLIMETRES: i64 = 1_000_000_000;
const MAX_WIDTH_MILLIMETRES: i64 = 1_000_000;
const MAX_PERMILLE: u32 = 1000;
const MAX_TICKS_PER_SECOND: u32 = 1000;

const KIND_OPEN: u32 = 1;
const KIND_PIPE: u32 = 2;
const KIND_GATE: u32 = 3;
const KIND_PUMP: u32 = 4;
const KIND_SOURCE: u32 = 5;
const KIND_SINK: u32 = 6;

const COMMAND_SET_GATE_TAG: u8 = 1;
const COMMAND_SET_PUMP_TAG: u8 = 2;
const COMMAND_SET_SOURCE_TAG: u8 = 3;
const COMMAND_LENGTH: usize = 1 + 16 + 8 + 8;
const EVENT_LENGTH: usize = 16 + 8 + 4 + 1 + 8 + 8;

/// Exact integer square root (floor).
#[must_use]
pub fn isqrt_i128(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let mut low: i128 = 0;
    let mut high: i128 = 1;
    while high.saturating_mul(high) <= value {
        high = high.saturating_mul(2);
        if high > 1 << 64 {
            break;
        }
    }
    while low < high {
        let middle = low + (high - low) / 2 + 1;
        if middle
            .checked_mul(middle)
            .is_some_and(|square| square <= value)
        {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}

/// Head-driven edge kinds. Every coefficient is a permille profile value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaterFlowEdgeKindV1 {
    /// Free-surface exchange over a sill (weir law).
    Open {
        sill_micrometres: i64,
        width_millimetres: i64,
        coefficient_permille: u32,
    },
    /// Orifice of a fixed area with an invert height (Torricelli law), in
    /// either direction.
    Pipe {
        invert_micrometres: i64,
        area_square_millimetres: i64,
        coefficient_permille: u32,
    },
    /// A pipe scaled by a commanded opening.
    Gate {
        invert_micrometres: i64,
        area_square_millimetres: i64,
        coefficient_permille: u32,
        initial_opening_permille: u32,
    },
    /// Signed constant rate (`a -> b` when positive) up to a maximum head.
    Pump {
        rate_cubic_millimetres_per_second: i64,
        maximum_head_micrometres: i64,
        initially_enabled: bool,
    },
    /// Constant inflow into `cell_a` from outside.
    Source {
        rate_cubic_millimetres_per_second: i64,
    },
    /// Constant outflow from `cell_a` to outside, limited by its water.
    Sink {
        rate_cubic_millimetres_per_second: i64,
    },
}

impl WaterFlowEdgeKindV1 {
    #[must_use]
    pub const fn tag(&self) -> u32 {
        match self {
            Self::Open { .. } => KIND_OPEN,
            Self::Pipe { .. } => KIND_PIPE,
            Self::Gate { .. } => KIND_GATE,
            Self::Pump { .. } => KIND_PUMP,
            Self::Source { .. } => KIND_SOURCE,
            Self::Sink { .. } => KIND_SINK,
        }
    }

    #[must_use]
    pub const fn joins_two_cells(&self) -> bool {
        matches!(
            self,
            Self::Open { .. } | Self::Pipe { .. } | Self::Gate { .. } | Self::Pump { .. }
        )
    }

    fn validate(&self) -> Result<(), PhysicsContractError> {
        let valid = match *self {
            Self::Open {
                sill_micrometres,
                width_millimetres,
                coefficient_permille,
            } => {
                sill_micrometres.abs() <= super::water::WATER_POSITION_LIMIT_MICROMETRES
                    && width_millimetres > 0
                    && width_millimetres <= MAX_WIDTH_MILLIMETRES
                    && coefficient_permille <= MAX_PERMILLE
            }
            Self::Pipe {
                invert_micrometres,
                area_square_millimetres,
                coefficient_permille,
            } => {
                invert_micrometres.abs() <= super::water::WATER_POSITION_LIMIT_MICROMETRES
                    && area_square_millimetres > 0
                    && area_square_millimetres <= MAX_AREA_SQUARE_MILLIMETRES
                    && coefficient_permille <= MAX_PERMILLE
            }
            Self::Gate {
                invert_micrometres,
                area_square_millimetres,
                coefficient_permille,
                initial_opening_permille,
            } => {
                invert_micrometres.abs() <= super::water::WATER_POSITION_LIMIT_MICROMETRES
                    && area_square_millimetres > 0
                    && area_square_millimetres <= MAX_AREA_SQUARE_MILLIMETRES
                    && coefficient_permille <= MAX_PERMILLE
                    && initial_opening_permille <= MAX_PERMILLE
            }
            Self::Pump {
                rate_cubic_millimetres_per_second,
                maximum_head_micrometres,
                ..
            } => {
                rate_cubic_millimetres_per_second.abs()
                    <= MAX_WATER_FLOW_RATE_CUBIC_MILLIMETRES_PER_SECOND
                    && (0..=super::water::WATER_POSITION_LIMIT_MICROMETRES)
                        .contains(&maximum_head_micrometres)
            }
            Self::Source {
                rate_cubic_millimetres_per_second,
            }
            | Self::Sink {
                rate_cubic_millimetres_per_second,
            } => (0..=MAX_WATER_FLOW_RATE_CUBIC_MILLIMETRES_PER_SECOND)
                .contains(&rate_cubic_millimetres_per_second),
        };
        if valid {
            Ok(())
        } else {
            Err(PhysicsContractError::WaterFlowInvalid)
        }
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let (a, b, c, d, e) = match *self {
            Self::Open {
                sill_micrometres,
                width_millimetres,
                coefficient_permille,
            } => (
                sill_micrometres,
                width_millimetres,
                coefficient_permille,
                0,
                false,
            ),
            Self::Pipe {
                invert_micrometres,
                area_square_millimetres,
                coefficient_permille,
            } => (
                invert_micrometres,
                area_square_millimetres,
                coefficient_permille,
                0,
                false,
            ),
            Self::Gate {
                invert_micrometres,
                area_square_millimetres,
                coefficient_permille,
                initial_opening_permille,
            } => (
                invert_micrometres,
                area_square_millimetres,
                coefficient_permille,
                initial_opening_permille,
                false,
            ),
            Self::Pump {
                rate_cubic_millimetres_per_second,
                maximum_head_micrometres,
                initially_enabled,
            } => (
                rate_cubic_millimetres_per_second,
                maximum_head_micrometres,
                0,
                0,
                initially_enabled,
            ),
            Self::Source {
                rate_cubic_millimetres_per_second,
            }
            | Self::Sink {
                rate_cubic_millimetres_per_second,
            } => (rate_cubic_millimetres_per_second, 0, 0, 0, false),
        };
        encode_struct([
            field_u32(1, self.tag()),
            field_i64(2, a),
            field_i64(3, b),
            field_u32(4, c),
            field_u32(5, d),
            field_bool(6, e),
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
                (1, CANONICAL_TYPE_U32),
                (2, CANONICAL_TYPE_I64),
                (3, CANONICAL_TYPE_I64),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_U32),
                (6, CANONICAL_TYPE_BOOL),
            ],
        )?;
        let tag = read_u32_fields(&fields, 1)?;
        let a = read_i64_fields(&fields, 2)?;
        let b = read_i64_fields(&fields, 3)?;
        let c = read_u32_fields(&fields, 4)?;
        let d = read_u32_fields(&fields, 5)?;
        let e = read_bool_fields(&fields, 6)?;
        let kind = match tag {
            KIND_OPEN => Self::Open {
                sill_micrometres: a,
                width_millimetres: b,
                coefficient_permille: c,
            },
            KIND_PIPE => Self::Pipe {
                invert_micrometres: a,
                area_square_millimetres: b,
                coefficient_permille: c,
            },
            KIND_GATE => Self::Gate {
                invert_micrometres: a,
                area_square_millimetres: b,
                coefficient_permille: c,
                initial_opening_permille: d,
            },
            KIND_PUMP => Self::Pump {
                rate_cubic_millimetres_per_second: a,
                maximum_head_micrometres: b,
                initially_enabled: e,
            },
            KIND_SOURCE => Self::Source {
                rate_cubic_millimetres_per_second: a,
            },
            KIND_SINK => Self::Sink {
                rate_cubic_millimetres_per_second: a,
            },
            other => {
                return Err(PhysicsContractError::UnknownTag(
                    u8::try_from(other).unwrap_or(u8::MAX),
                ));
            }
        };
        // Unused slots must be canonical zeros.
        let canonical = kind.canonical_record()?;
        if canonical != bytes {
            return Err(PhysicsContractError::WaterFlowInvalid);
        }
        Ok(kind)
    }
}

/// Immutable authored edge. `cell_b` is `None` for sources and sinks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterFlowEdgeV1 {
    pub edge_id: PersistentId,
    pub cell_a: PersistentId,
    pub cell_b: Option<PersistentId>,
    pub kind: WaterFlowEdgeKindV1,
}

impl WaterFlowEdgeV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        self.kind.validate()?;
        if self.kind.joins_two_cells() != self.cell_b.is_some() || self.cell_b == Some(self.cell_a)
        {
            return Err(PhysicsContractError::WaterFlowInvalid);
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let cell_b = self.cell_b.unwrap_or(PersistentId::from_bytes([0; 16]));
        encode_struct([
            field_id(1, self.edge_id.as_bytes()),
            field_id(2, self.cell_a.as_bytes()),
            field_bool(3, self.cell_b.is_some()),
            field_id(4, cell_b.as_bytes()),
            CanonicalField::new(5, CANONICAL_TYPE_MAP, self.kind.canonical_record()?),
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
                (3, CANONICAL_TYPE_BOOL),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_MAP),
            ],
        )?;
        let has_b = read_bool_fields(&fields, 3)?;
        let cell_b = PersistentId::from_bytes(exact(&field_from(&fields, 4)?.payload)?);
        if !has_b && cell_b != PersistentId::from_bytes([0; 16]) {
            return Err(PhysicsContractError::WaterFlowInvalid);
        }
        let value = Self {
            edge_id: PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            cell_a: PersistentId::from_bytes(exact(&field_from(&fields, 2)?.payload)?),
            cell_b: has_b.then_some(cell_b),
            kind: WaterFlowEdgeKindV1::from_record(&field_from(&fields, 5)?.payload, limits)?,
        };
        value.validate()?;
        Ok(value)
    }
}

/// Mutable authoritative state of one edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterFlowEdgeStateV1 {
    pub edge_id: PersistentId,
    /// Starts at `0` and increments once per committed command.
    pub record_revision: u64,
    /// Gate opening; `1000` for every other kind.
    pub opening_permille: u32,
    /// Pump switch; `true` for every other kind.
    pub enabled: bool,
    /// Current source/sink/pump rate.
    pub rate_cubic_millimetres_per_second: i64,
    /// Signed volume moved `a -> b` (or in/out for one-cell edges) in the
    /// last step.
    pub last_flux_cubic_millimetres: i64,
}

impl WaterFlowEdgeStateV1 {
    fn genesis(edge: &WaterFlowEdgeV1) -> Self {
        let (opening, enabled, rate) = match edge.kind {
            WaterFlowEdgeKindV1::Gate {
                initial_opening_permille,
                ..
            } => (initial_opening_permille, true, 0),
            WaterFlowEdgeKindV1::Pump {
                rate_cubic_millimetres_per_second,
                initially_enabled,
                ..
            } => (
                MAX_PERMILLE,
                initially_enabled,
                rate_cubic_millimetres_per_second,
            ),
            WaterFlowEdgeKindV1::Source {
                rate_cubic_millimetres_per_second,
            }
            | WaterFlowEdgeKindV1::Sink {
                rate_cubic_millimetres_per_second,
            } => (MAX_PERMILLE, true, rate_cubic_millimetres_per_second),
            _ => (MAX_PERMILLE, true, 0),
        };
        Self {
            edge_id: edge.edge_id,
            record_revision: 0,
            opening_permille: opening,
            enabled,
            rate_cubic_millimetres_per_second: rate,
            last_flux_cubic_millimetres: 0,
        }
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_id(1, self.edge_id.as_bytes()),
            field_u64(2, self.record_revision),
            field_u32(3, self.opening_permille),
            field_bool(4, self.enabled),
            field_i64(5, self.rate_cubic_millimetres_per_second),
            field_i64(6, self.last_flux_cubic_millimetres),
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
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_BOOL),
                (5, CANONICAL_TYPE_I64),
                (6, CANONICAL_TYPE_I64),
            ],
        )?;
        Ok(Self {
            edge_id: PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            record_revision: read_u64_fields(&fields, 2)?,
            opening_permille: read_u32_fields(&fields, 3)?,
            enabled: read_bool_fields(&fields, 4)?,
            rate_cubic_millimetres_per_second: read_i64_fields(&fields, 5)?,
            last_flux_cubic_millimetres: read_i64_fields(&fields, 6)?,
        })
    }
}

/// Exact stored water of one cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterFlowCellStateV1 {
    pub cell_id: PersistentId,
    pub volume_cubic_millimetres: i64,
    /// Record revision of the `WaterVolume` state this volume was last
    /// synchronised with; a newer revision (an authored `SetLevel`) rewrites
    /// the volume from the level at the next step.
    pub synced_record_revision: u64,
}

impl WaterFlowCellStateV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_id(1, self.cell_id.as_bytes()),
            field_i64(2, self.volume_cubic_millimetres),
            field_u64(3, self.synced_record_revision),
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
                (2, CANONICAL_TYPE_I64),
                (3, CANONICAL_TYPE_U64),
            ],
        )?;
        Ok(Self {
            cell_id: PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            volume_cubic_millimetres: read_i64_fields(&fields, 2)?,
            synced_record_revision: read_u64_fields(&fields, 3)?,
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WaterFlowCommandV1 {
    SetGate {
        edge_id: PersistentId,
        expected_record_revision: u64,
        opening_permille: u32,
    },
    SetPump {
        edge_id: PersistentId,
        expected_record_revision: u64,
        enabled: bool,
    },
    SetSource {
        edge_id: PersistentId,
        expected_record_revision: u64,
        rate_cubic_millimetres_per_second: i64,
    },
}

impl WaterFlowCommandV1 {
    #[must_use]
    pub const fn edge_id(&self) -> PersistentId {
        match self {
            Self::SetGate { edge_id, .. }
            | Self::SetPump { edge_id, .. }
            | Self::SetSource { edge_id, .. } => *edge_id,
        }
    }

    #[must_use]
    pub const fn expected_record_revision(&self) -> u64 {
        match self {
            Self::SetGate {
                expected_record_revision,
                ..
            }
            | Self::SetPump {
                expected_record_revision,
                ..
            }
            | Self::SetSource {
                expected_record_revision,
                ..
            } => *expected_record_revision,
        }
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, value) = match self {
            Self::SetGate {
                opening_permille, ..
            } => (COMMAND_SET_GATE_TAG, i64::from(*opening_permille)),
            Self::SetPump { enabled, .. } => (COMMAND_SET_PUMP_TAG, i64::from(*enabled)),
            Self::SetSource {
                rate_cubic_millimetres_per_second,
                ..
            } => (COMMAND_SET_SOURCE_TAG, *rate_cubic_millimetres_per_second),
        };
        let mut bytes = Vec::with_capacity(COMMAND_LENGTH);
        bytes.push(tag);
        bytes.extend_from_slice(self.edge_id().as_bytes());
        bytes.extend_from_slice(&self.expected_record_revision().to_le_bytes());
        bytes.extend_from_slice(&value.to_le_bytes());
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        _limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        if bytes.len() != COMMAND_LENGTH {
            return Err(PhysicsContractError::FieldLength);
        }
        let edge_id = PersistentId::from_bytes(exact(&bytes[1..17])?);
        let expected_record_revision = u64::from_le_bytes(exact(&bytes[17..25])?);
        let value = i64::from_le_bytes(exact(&bytes[25..33])?);
        match bytes[0] {
            COMMAND_SET_GATE_TAG => Ok(Self::SetGate {
                edge_id,
                expected_record_revision,
                opening_permille: u32::try_from(value)
                    .map_err(|_| PhysicsContractError::WaterFlowInvalid)?,
            }),
            COMMAND_SET_PUMP_TAG => match value {
                0 | 1 => Ok(Self::SetPump {
                    edge_id,
                    expected_record_revision,
                    enabled: value == 1,
                }),
                _ => Err(PhysicsContractError::WaterFlowInvalid),
            },
            COMMAND_SET_SOURCE_TAG => Ok(Self::SetSource {
                edge_id,
                expected_record_revision,
                rate_cubic_millimetres_per_second: value,
            }),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterFlowChangedV1 {
    pub edge_id: PersistentId,
    pub record_revision: u64,
    pub opening_permille: u32,
    pub enabled: bool,
    pub rate_cubic_millimetres_per_second: i64,
    pub boundary_tick: u64,
}

impl WaterFlowChangedV1 {
    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::with_capacity(EVENT_LENGTH);
        bytes.extend_from_slice(self.edge_id.as_bytes());
        bytes.extend_from_slice(&self.record_revision.to_le_bytes());
        bytes.extend_from_slice(&self.opening_permille.to_le_bytes());
        bytes.push(u8::from(self.enabled));
        bytes.extend_from_slice(&self.rate_cubic_millimetres_per_second.to_le_bytes());
        bytes.extend_from_slice(&self.boundary_tick.to_le_bytes());
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(bytes: &[u8]) -> Result<Self, PhysicsContractError> {
        if bytes.len() != EVENT_LENGTH {
            return Err(PhysicsContractError::FieldLength);
        }
        let enabled = match bytes[28] {
            0 => false,
            1 => true,
            _ => return Err(PhysicsContractError::WaterFlowInvalid),
        };
        Ok(Self {
            edge_id: PersistentId::from_bytes(exact(&bytes[0..16])?),
            record_revision: u64::from_le_bytes(exact(&bytes[16..24])?),
            opening_permille: u32::from_le_bytes(exact(&bytes[24..28])?),
            enabled,
            rate_cubic_millimetres_per_second: i64::from_le_bytes(exact(&bytes[29..37])?),
            boundary_tick: u64::from_le_bytes(exact(&bytes[37..45])?),
        })
    }
}

/// Deterministic rejection of one flow command; never a fatal error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaterFlowRejectionV1 {
    UnknownEdge,
    WrongKind,
    RevisionStale,
    OpeningOutOfRange,
    RateOutOfRange,
    RevisionExhausted,
}

impl WaterFlowRejectionV1 {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::UnknownEdge => "WATER_FLOW_EDGE_UNKNOWN",
            Self::WrongKind => "WATER_FLOW_EDGE_KIND_MISMATCH",
            Self::RevisionStale => "WATER_FLOW_REVISION_STALE",
            Self::OpeningOutOfRange => "WATER_FLOW_OPENING_OUT_OF_RANGE",
            Self::RateOutOfRange => "WATER_FLOW_RATE_OUT_OF_RANGE",
            Self::RevisionExhausted => "WATER_FLOW_REVISION_EXHAUSTED",
        }
    }
}

/// Plan area of a cell in square millimetres from its micrometre extent.
#[must_use]
pub fn cell_area_square_millimetres(minimum: [i64; 3], maximum: [i64; 3]) -> i64 {
    let dx = i128::from(maximum[0] - minimum[0]) / 1000;
    let dz = i128::from(maximum[2] - minimum[2]) / 1000;
    i64::try_from((dx * dz).max(1)).unwrap_or(i64::MAX)
}

/// Exact level of a cell from its volume, truncated toward the floor and
/// saturated at the ceiling (an overfull cell reads as full).
#[must_use]
pub fn level_from_volume(
    floor_micrometres: i64,
    ceiling_micrometres: i64,
    area_square_millimetres: i64,
    volume_cubic_millimetres: i64,
) -> i64 {
    if volume_cubic_millimetres <= 0 || area_square_millimetres <= 0 {
        return floor_micrometres;
    }
    let rise = i128::from(volume_cubic_millimetres) * 1000 / i128::from(area_square_millimetres);
    let level = i128::from(floor_micrometres) + rise;
    i64::try_from(level.min(i128::from(ceiling_micrometres))).unwrap_or(ceiling_micrometres)
}

/// Exact volume of a cell from a level (the authored override direction).
#[must_use]
pub fn volume_from_level(
    floor_micrometres: i64,
    area_square_millimetres: i64,
    level_micrometres: i64,
) -> i64 {
    let rise = i128::from(level_micrometres.saturating_sub(floor_micrometres)).max(0);
    i64::try_from(rise * i128::from(area_square_millimetres) / 1000).unwrap_or(i64::MAX)
}

/// Scales the entries of `parts` so that they sum to at most `budget`
/// (largest remainder; exact integer sum when scaling applies).
fn scale_to_budget(parts: &mut [i64], budget: i64) {
    let total: i128 = parts.iter().map(|part| i128::from(*part)).sum();
    if total <= i128::from(budget) || total <= 0 {
        return;
    }
    let budget = i128::from(budget.max(0));
    let mut remainders: Vec<(i128, usize)> = Vec::with_capacity(parts.len());
    let mut assigned: i128 = 0;
    for (index, part) in parts.iter_mut().enumerate() {
        let product = i128::from(*part) * budget;
        let scaled = product / total;
        remainders.push((product - scaled * total, index));
        *part = i64::try_from(scaled).unwrap_or(i64::MAX);
        assigned += scaled;
    }
    remainders.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut leftover = budget - assigned;
    for (_, index) in remainders {
        if leftover <= 0 {
            break;
        }
        parts[index] += 1;
        leftover -= 1;
    }
}

/// The bounded authoritative flow network of one physics world.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterFlowNetworkV1 {
    pub schema_version: u16,
    pub ticks_per_second: u32,
    pub edges: BTreeMap<PersistentId, WaterFlowEdgeV1>,
    pub edge_states: BTreeMap<PersistentId, WaterFlowEdgeStateV1>,
    pub cells: BTreeMap<PersistentId, WaterFlowCellStateV1>,
}

impl Default for WaterFlowNetworkV1 {
    fn default() -> Self {
        Self::empty()
    }
}

/// Result of one exact step: the successor network and the volume set with
/// the projected cell levels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterFlowStepV1 {
    pub network: WaterFlowNetworkV1,
    pub volumes: WaterVolumeSetV1,
}

impl WaterFlowNetworkV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schema_version: WATER_FLOW_SCHEMA_VERSION,
            ticks_per_second: 0,
            edges: BTreeMap::new(),
            edge_states: BTreeMap::new(),
            cells: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty() && self.cells.is_empty()
    }

    /// Genesis network: every referenced volume becomes a cell whose volume
    /// is derived from its authored initial level; every edge starts at its
    /// authored state.
    pub fn from_edges(
        ticks_per_second: u32,
        edges: impl IntoIterator<Item = WaterFlowEdgeV1>,
        volumes: &WaterVolumeSetV1,
    ) -> Result<Self, PhysicsContractError> {
        let mut map = BTreeMap::new();
        let mut states = BTreeMap::new();
        let mut cells = BTreeMap::new();
        for edge in edges {
            edge.validate()?;
            for cell in [Some(edge.cell_a), edge.cell_b].into_iter().flatten() {
                let definition = volumes
                    .definitions
                    .get(&cell)
                    .ok_or(PhysicsContractError::WaterFlowInvalid)?;
                let state = volumes
                    .states
                    .get(&cell)
                    .ok_or(PhysicsContractError::WaterFlowInvalid)?;
                cells.entry(cell).or_insert(WaterFlowCellStateV1 {
                    cell_id: cell,
                    volume_cubic_millimetres: volume_from_level(
                        definition.minimum_micrometres[1],
                        cell_area_square_millimetres(
                            definition.minimum_micrometres,
                            definition.maximum_micrometres,
                        ),
                        state.level_micrometres,
                    ),
                    synced_record_revision: state.record_revision,
                });
            }
            states.insert(edge.edge_id, WaterFlowEdgeStateV1::genesis(&edge));
            if map.insert(edge.edge_id, edge).is_some() {
                return Err(PhysicsContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: WATER_FLOW_SCHEMA_VERSION,
            ticks_per_second,
            edges: map,
            edge_states: states,
            cells,
        };
        value.validate()?;
        value.validate_against(volumes)?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != WATER_FLOW_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                self.schema_version,
            )));
        }
        if self.is_empty() {
            if self.ticks_per_second != 0 || !self.edge_states.is_empty() {
                return Err(PhysicsContractError::WaterFlowInvalid);
            }
            return Ok(());
        }
        if self.ticks_per_second == 0
            || self.ticks_per_second > MAX_TICKS_PER_SECOND
            || self.edges.len() > MAX_WATER_FLOW_EDGES
            || self.cells.len() > super::water::MAX_WATER_VOLUMES
            || !self.edges.keys().eq(self.edge_states.keys())
        {
            return Err(PhysicsContractError::WaterFlowInvalid);
        }
        for (edge_id, edge) in &self.edges {
            if edge.edge_id != *edge_id {
                return Err(PhysicsContractError::WaterFlowInvalid);
            }
            edge.validate()?;
            for cell in [Some(edge.cell_a), edge.cell_b].into_iter().flatten() {
                if !self.cells.contains_key(&cell) {
                    return Err(PhysicsContractError::WaterFlowInvalid);
                }
            }
            let state = &self.edge_states[edge_id];
            if state.edge_id != *edge_id
                || state.opening_permille > MAX_PERMILLE
                || state.rate_cubic_millimetres_per_second.abs()
                    > MAX_WATER_FLOW_RATE_CUBIC_MILLIMETRES_PER_SECOND
            {
                return Err(PhysicsContractError::WaterFlowInvalid);
            }
        }
        for (cell_id, cell) in &self.cells {
            if cell.cell_id != *cell_id || cell.volume_cubic_millimetres < 0 {
                return Err(PhysicsContractError::WaterFlowInvalid);
            }
            if !self
                .edges
                .values()
                .any(|edge| edge.cell_a == *cell_id || edge.cell_b == Some(*cell_id))
            {
                return Err(PhysicsContractError::WaterFlowInvalid);
            }
        }
        Ok(())
    }

    /// Every cell must be a declared volume without an authored ramp.
    pub fn validate_against(&self, volumes: &WaterVolumeSetV1) -> Result<(), PhysicsContractError> {
        for cell_id in self.cells.keys() {
            let definition = volumes
                .definitions
                .get(cell_id)
                .ok_or(PhysicsContractError::WaterFlowInvalid)?;
            if definition.level_ramp.is_some() {
                return Err(PhysicsContractError::WaterFlowInvalid);
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn cell_volume(&self, cell_id: PersistentId) -> Option<i64> {
        self.cells
            .get(&cell_id)
            .map(|cell| cell.volume_cubic_millimetres)
    }

    #[must_use]
    pub fn edge_flux(&self, edge_id: PersistentId) -> Option<i64> {
        self.edge_states
            .get(&edge_id)
            .map(|state| state.last_flux_cubic_millimetres)
    }

    /// Total stored water of the network in cubic millimetres.
    #[must_use]
    pub fn total_volume(&self) -> i128 {
        self.cells
            .values()
            .map(|cell| i128::from(cell.volume_cubic_millimetres))
            .sum()
    }

    /// Applies one validated command, returning the successor network and
    /// the committed event, or the deterministic rejection.
    pub fn apply_command(
        &self,
        command: &WaterFlowCommandV1,
        boundary_tick: u64,
    ) -> Result<(Self, WaterFlowChangedV1), WaterFlowRejectionV1> {
        let edge_id = command.edge_id();
        let edge = self
            .edges
            .get(&edge_id)
            .ok_or(WaterFlowRejectionV1::UnknownEdge)?;
        let state = self
            .edge_states
            .get(&edge_id)
            .ok_or(WaterFlowRejectionV1::UnknownEdge)?;
        if state.record_revision != command.expected_record_revision() {
            return Err(WaterFlowRejectionV1::RevisionStale);
        }
        let mut next_state = *state;
        match (command, &edge.kind) {
            (
                WaterFlowCommandV1::SetGate {
                    opening_permille, ..
                },
                WaterFlowEdgeKindV1::Gate { .. },
            ) => {
                if *opening_permille > MAX_PERMILLE {
                    return Err(WaterFlowRejectionV1::OpeningOutOfRange);
                }
                next_state.opening_permille = *opening_permille;
            }
            (WaterFlowCommandV1::SetPump { enabled, .. }, WaterFlowEdgeKindV1::Pump { .. }) => {
                next_state.enabled = *enabled;
            }
            (
                WaterFlowCommandV1::SetSource {
                    rate_cubic_millimetres_per_second,
                    ..
                },
                WaterFlowEdgeKindV1::Source { .. } | WaterFlowEdgeKindV1::Sink { .. },
            ) => {
                if !(0..=MAX_WATER_FLOW_RATE_CUBIC_MILLIMETRES_PER_SECOND)
                    .contains(rate_cubic_millimetres_per_second)
                {
                    return Err(WaterFlowRejectionV1::RateOutOfRange);
                }
                next_state.rate_cubic_millimetres_per_second = *rate_cubic_millimetres_per_second;
            }
            _ => return Err(WaterFlowRejectionV1::WrongKind),
        }
        next_state.record_revision = state
            .record_revision
            .checked_add(1)
            .ok_or(WaterFlowRejectionV1::RevisionExhausted)?;
        let mut next = self.clone();
        next.edge_states.insert(edge_id, next_state);
        Ok((
            next,
            WaterFlowChangedV1 {
                edge_id,
                record_revision: next_state.record_revision,
                opening_permille: next_state.opening_permille,
                enabled: next_state.enabled,
                rate_cubic_millimetres_per_second: next_state.rate_cubic_millimetres_per_second,
                boundary_tick,
            },
        ))
    }

    /// One exact Jacobi step (ADR-103): every edge flux from the levels at
    /// the start of the tick, limited by the water above the sill on its
    /// source side and by half the equalising volume; per-cell outflows are
    /// scaled so no cell goes negative; then all fluxes apply and the cell
    /// levels are projected into the volume set.
    pub fn step(
        &self,
        volumes: &WaterVolumeSetV1,
    ) -> Result<WaterFlowStepV1, PhysicsContractError> {
        if self.is_empty() {
            return Ok(WaterFlowStepV1 {
                network: self.clone(),
                volumes: volumes.clone(),
            });
        }
        self.validate_against(volumes)?;
        // Cell geometry and the volume synchronised with authored overrides.
        struct Cell {
            floor: i64,
            ceiling: i64,
            area: i64,
            volume: i64,
            level: i64,
            outflow: i64,
            inflow: i64,
        }
        let mut cells: BTreeMap<PersistentId, Cell> = BTreeMap::new();
        let mut next = self.clone();
        for (cell_id, cell_state) in &mut next.cells {
            let definition = &volumes.definitions[cell_id];
            let state = &volumes.states[cell_id];
            let area = cell_area_square_millimetres(
                definition.minimum_micrometres,
                definition.maximum_micrometres,
            );
            if state.record_revision != cell_state.synced_record_revision {
                cell_state.volume_cubic_millimetres = volume_from_level(
                    definition.minimum_micrometres[1],
                    area,
                    state.level_micrometres,
                );
                cell_state.synced_record_revision = state.record_revision;
            }
            let level = level_from_volume(
                definition.minimum_micrometres[1],
                definition.maximum_micrometres[1],
                area,
                cell_state.volume_cubic_millimetres,
            );
            cells.insert(
                *cell_id,
                Cell {
                    floor: definition.minimum_micrometres[1],
                    ceiling: definition.maximum_micrometres[1],
                    area,
                    volume: cell_state.volume_cubic_millimetres,
                    level,
                    outflow: 0,
                    inflow: 0,
                },
            );
        }
        let hz = i128::from(self.ticks_per_second);
        let gravity = i128::from(WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
        // Signed flux per edge, positive from `cell_a` to `cell_b` (or into
        // the cell for sources, out of it for sinks as a negative value).
        let mut fluxes: Vec<(PersistentId, i64)> = Vec::with_capacity(self.edges.len());
        for (edge_id, edge) in &self.edges {
            let state = &self.edge_states[edge_id];
            let a = &cells[&edge.cell_a];
            let b = edge.cell_b.map(|id| &cells[&id]);
            let mut flux: i128 = match edge.kind {
                WaterFlowEdgeKindV1::Open {
                    sill_micrometres,
                    width_millimetres,
                    coefficient_permille,
                } => {
                    let b = b.expect("validated two-cell edge");
                    let head_a = i128::from(a.level.saturating_sub(sill_micrometres)).max(0);
                    let head_b = i128::from(b.level.saturating_sub(sill_micrometres)).max(0);
                    let (head, sign) = if head_a >= head_b {
                        (head_a - head_b, 1)
                    } else {
                        (head_b - head_a, -1)
                    };
                    // q = c_w * w * sqrt(g h^3): sqrt in um^2/s, width in mm.
                    let root = isqrt_i128(gravity * head * head * head);
                    let per_second =
                        i128::from(coefficient_permille) * i128::from(width_millimetres) * root
                            / 1_000_000
                            / 1000;
                    sign * per_second / hz
                }
                WaterFlowEdgeKindV1::Pipe {
                    invert_micrometres,
                    area_square_millimetres,
                    coefficient_permille,
                }
                | WaterFlowEdgeKindV1::Gate {
                    invert_micrometres,
                    area_square_millimetres,
                    coefficient_permille,
                    ..
                } => {
                    let b = b.expect("validated two-cell edge");
                    let level_a = i128::from(a.level.max(invert_micrometres));
                    let level_b = i128::from(b.level.max(invert_micrometres));
                    let (head, sign) = if level_a >= level_b {
                        (level_a - level_b, 1)
                    } else {
                        (level_b - level_a, -1)
                    };
                    let velocity = isqrt_i128(2 * gravity * head);
                    let opening = if matches!(edge.kind, WaterFlowEdgeKindV1::Gate { .. }) {
                        i128::from(state.opening_permille)
                    } else {
                        i128::from(MAX_PERMILLE)
                    };
                    // q = c_d * A * v: area in mm^2, velocity in um/s.
                    let per_second = i128::from(coefficient_permille)
                        * opening
                        * i128::from(area_square_millimetres)
                        * velocity
                        / 1000
                        / 1000
                        / 1000;
                    sign * per_second / hz
                }
                WaterFlowEdgeKindV1::Pump {
                    maximum_head_micrometres,
                    ..
                } => {
                    let b = b.expect("validated two-cell edge");
                    let rate = i128::from(state.rate_cubic_millimetres_per_second);
                    let against = if rate >= 0 {
                        i128::from(b.level) - i128::from(a.level)
                    } else {
                        i128::from(a.level) - i128::from(b.level)
                    };
                    if !state.enabled || against > i128::from(maximum_head_micrometres) {
                        0
                    } else {
                        rate / hz
                    }
                }
                WaterFlowEdgeKindV1::Source { .. } => {
                    i128::from(state.rate_cubic_millimetres_per_second) / hz
                }
                WaterFlowEdgeKindV1::Sink { .. } => {
                    -(i128::from(state.rate_cubic_millimetres_per_second) / hz)
                }
            };
            // Limits: water above the sill on the source side and half the
            // equalising volume for two-cell edges; the whole cell for sinks.
            if let Some(b) = b {
                let (source, sink) = if flux >= 0 { (a, b) } else { (b, a) };
                let sill = match edge.kind {
                    WaterFlowEdgeKindV1::Open {
                        sill_micrometres, ..
                    } => sill_micrometres,
                    WaterFlowEdgeKindV1::Pipe {
                        invert_micrometres, ..
                    }
                    | WaterFlowEdgeKindV1::Gate {
                        invert_micrometres, ..
                    } => invert_micrometres,
                    _ => i64::MIN,
                };
                let above_sill = if sill == i64::MIN {
                    i128::from(source.volume)
                } else {
                    volume_above(source.floor, source.area, source.volume, sill)
                };
                let equalising = if source.level > sink.level {
                    i128::from(source.level - sink.level)
                        * i128::from(source.area)
                        * i128::from(sink.area)
                        / (i128::from(source.area) + i128::from(sink.area))
                        / 1000
                } else {
                    0
                };
                let cap = if matches!(edge.kind, WaterFlowEdgeKindV1::Pump { .. }) {
                    above_sill
                } else {
                    above_sill.min(equalising / 2)
                };
                flux = flux.signum() * flux.abs().min(cap.max(0));
            } else if flux < 0 {
                flux = -((-flux).min(i128::from(a.volume)));
            }
            let flux = i64::try_from(flux).map_err(|_| PhysicsContractError::WaterFlowInvalid)?;
            fluxes.push((*edge_id, flux));
        }
        // Per-cell outflow scaling so no cell goes negative (largest
        // remainder keeps the scaled sum exact).
        let mut outflows: BTreeMap<PersistentId, Vec<usize>> = BTreeMap::new();
        for (index, (edge_id, flux)) in fluxes.iter().enumerate() {
            let edge = &self.edges[edge_id];
            let (source, _) = flow_endpoints(edge, *flux);
            if let Some(source) = source {
                outflows.entry(source).or_default().push(index);
            }
        }
        for (cell_id, indices) in outflows {
            let budget = cells[&cell_id].volume;
            let mut parts: Vec<i64> = indices.iter().map(|index| fluxes[*index].1.abs()).collect();
            scale_to_budget(&mut parts, budget);
            for (slot, index) in indices.iter().enumerate() {
                let sign = fluxes[*index].1.signum();
                fluxes[*index].1 = sign * parts[slot];
            }
        }
        for (edge_id, flux) in &fluxes {
            let edge = &self.edges[edge_id];
            let (source, sink) = flow_endpoints(edge, *flux);
            if let Some(source) = source {
                let cell = cells.get_mut(&source).expect("validated cell");
                cell.outflow = cell
                    .outflow
                    .checked_add(flux.abs())
                    .ok_or(PhysicsContractError::WaterFlowInvalid)?;
            }
            if let Some(sink) = sink {
                let cell = cells.get_mut(&sink).expect("validated cell");
                cell.inflow = cell
                    .inflow
                    .checked_add(flux.abs())
                    .ok_or(PhysicsContractError::WaterFlowInvalid)?;
            }
            next.edge_states
                .get_mut(edge_id)
                .expect("validated edge")
                .last_flux_cubic_millimetres = *flux;
        }
        let mut next_volumes = volumes.clone();
        for (cell_id, cell) in &cells {
            let volume = cell
                .volume
                .checked_add(cell.inflow)
                .and_then(|value| value.checked_sub(cell.outflow))
                .ok_or(PhysicsContractError::WaterFlowInvalid)?;
            if volume < 0 {
                return Err(PhysicsContractError::WaterFlowInvalid);
            }
            let level = level_from_volume(cell.floor, cell.ceiling, cell.area, volume);
            let cell_state = next.cells.get_mut(cell_id).expect("validated cell");
            cell_state.volume_cubic_millimetres = volume;
            let state = next_volumes
                .states
                .get_mut(cell_id)
                .expect("validated cell");
            *state = WaterVolumeStateV1 {
                volume_id: *cell_id,
                record_revision: state.record_revision,
                level_micrometres: level,
                ramp_suspended: state.ramp_suspended,
            };
        }
        next_volumes.validate()?;
        Ok(WaterFlowStepV1 {
            network: next,
            volumes: next_volumes,
        })
    }

    pub fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let edges = self
            .edges
            .values()
            .map(WaterFlowEdgeV1::canonical_record)
            .collect::<Result<Vec<_>, _>>()?;
        let states = self
            .edge_states
            .values()
            .map(WaterFlowEdgeStateV1::canonical_record)
            .collect::<Result<Vec<_>, _>>()?;
        let cells = self
            .cells
            .values()
            .map(WaterFlowCellStateV1::canonical_record)
            .collect::<Result<Vec<_>, _>>()?;
        encode_struct([
            field_u16(1, self.schema_version),
            field_u32(2, self.ticks_per_second),
            CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_sequence(edges)?),
            CanonicalField::new(4, CANONICAL_TYPE_MAP, encode_sequence(states)?),
            CanonicalField::new(5, CANONICAL_TYPE_MAP, encode_sequence(cells)?),
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
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_MAP),
                (4, CANONICAL_TYPE_MAP),
                (5, CANONICAL_TYPE_MAP),
            ],
        )?;
        let schema_version = read_u16_fields(&fields, 1)?;
        if schema_version != WATER_FLOW_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                schema_version,
            )));
        }
        let mut edges = BTreeMap::new();
        let mut previous: Option<PersistentId> = None;
        for entry in decode_sequence(&field_from(&fields, 3)?.payload, limits)? {
            let edge = WaterFlowEdgeV1::from_record(&entry, limits)?;
            if previous.is_some_and(|previous| edge.edge_id <= previous) {
                return Err(PhysicsContractError::NonCanonicalOrder);
            }
            previous = Some(edge.edge_id);
            edges.insert(edge.edge_id, edge);
        }
        let mut edge_states = BTreeMap::new();
        let mut previous: Option<PersistentId> = None;
        for entry in decode_sequence(&field_from(&fields, 4)?.payload, limits)? {
            let state = WaterFlowEdgeStateV1::from_record(&entry, limits)?;
            if previous.is_some_and(|previous| state.edge_id <= previous) {
                return Err(PhysicsContractError::NonCanonicalOrder);
            }
            previous = Some(state.edge_id);
            edge_states.insert(state.edge_id, state);
        }
        let mut cells = BTreeMap::new();
        let mut previous: Option<PersistentId> = None;
        for entry in decode_sequence(&field_from(&fields, 5)?.payload, limits)? {
            let cell = WaterFlowCellStateV1::from_record(&entry, limits)?;
            if previous.is_some_and(|previous| cell.cell_id <= previous) {
                return Err(PhysicsContractError::NonCanonicalOrder);
            }
            previous = Some(cell.cell_id);
            cells.insert(cell.cell_id, cell);
        }
        let value = Self {
            schema_version,
            ticks_per_second: read_u32_fields(&fields, 2)?,
            edges,
            edge_states,
            cells,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_record()?)?;
        Ok(value)
    }

    pub fn network_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-water-flow-network.v1\0",
            &self.canonical_record()?,
        )
    }
}

/// Volume of a cell above a sill height, zero when the level is below it.
fn volume_above(floor: i64, area: i64, volume: i64, sill: i64) -> i128 {
    let below_sill = i128::from(sill.saturating_sub(floor)).max(0) * i128::from(area) / 1000;
    (i128::from(volume) - below_sill).max(0)
}

/// `(source, sink)` cells of a signed flux along an edge.
fn flow_endpoints(
    edge: &WaterFlowEdgeV1,
    flux: i64,
) -> (Option<PersistentId>, Option<PersistentId>) {
    match (edge.cell_b, flux >= 0) {
        (Some(b), true) => (Some(edge.cell_a), Some(b)),
        (Some(b), false) => (Some(b), Some(edge.cell_a)),
        // Source (positive) fills the cell; sink (negative) drains it.
        (None, true) => (None, Some(edge.cell_a)),
        (None, false) => (Some(edge.cell_a), None),
    }
}

#[cfg(test)]
mod tests {
    use super::super::water::WaterVolumeDefinitionV1;
    use super::*;

    fn id(byte: u8) -> PersistentId {
        PersistentId::from_bytes([byte; 16])
    }

    fn vessel(
        byte: u8,
        floor: i64,
        plan_x: i64,
        plan_z: i64,
        level: i64,
    ) -> WaterVolumeDefinitionV1 {
        let x0 = i64::from(byte) * 10_000_000;
        WaterVolumeDefinitionV1 {
            volume_id: id(byte),
            minimum_micrometres: [x0, floor, 0],
            maximum_micrometres: [x0 + plan_x, floor + 2_000_000, plan_z],
            initial_level_micrometres: level,
            swimming_depth_micrometres: 1_200_000,
            level_ramp: None,
            profile_revision: 1,
        }
    }

    fn two_vessels() -> (WaterVolumeSetV1, WaterFlowNetworkV1) {
        let volumes = WaterVolumeSetV1::from_definitions([
            vessel(1, 1_000_000, 2_000_000, 1_500_000, 1_500_000),
            vessel(2, 0, 2_800_000, 1_500_000, 0),
        ])
        .expect("volumes");
        let network = WaterFlowNetworkV1::from_edges(
            30,
            [
                WaterFlowEdgeV1 {
                    edge_id: id(0x21),
                    cell_a: id(1),
                    cell_b: Some(id(2)),
                    kind: WaterFlowEdgeKindV1::Gate {
                        invert_micrometres: 600_000,
                        area_square_millimetres: 40_000,
                        coefficient_permille: 400,
                        initial_opening_permille: 1000,
                    },
                },
                WaterFlowEdgeV1 {
                    edge_id: id(0x22),
                    cell_a: id(1),
                    cell_b: None,
                    kind: WaterFlowEdgeKindV1::Source {
                        rate_cubic_millimetres_per_second: 500_000,
                    },
                },
                WaterFlowEdgeV1 {
                    edge_id: id(0x23),
                    cell_a: id(2),
                    cell_b: None,
                    kind: WaterFlowEdgeKindV1::Sink {
                        rate_cubic_millimetres_per_second: 500_000,
                    },
                },
            ],
            &volumes,
        )
        .expect("network");
        (volumes, network)
    }

    #[test]
    fn integer_square_root_is_exact() {
        for value in [0_i128, 1, 2, 3, 4, 15, 16, 17, 1_000_000, (1 << 62) + 12345] {
            let root = isqrt_i128(value);
            assert!(root * root <= value);
            assert!((root + 1) * (root + 1) > value);
        }
        assert_eq!(isqrt_i128(-5), 0);
    }

    #[test]
    fn level_and_volume_round_trip() {
        let area = cell_area_square_millimetres([0, 0, 0], [2_000_000, 2_000_000, 1_500_000]);
        assert_eq!(area, 3_000_000);
        let volume = volume_from_level(1_000_000, area, 1_500_000);
        assert_eq!(volume, 1_500_000_000);
        assert_eq!(
            level_from_volume(1_000_000, 3_000_000, area, volume),
            1_500_000
        );
        assert_eq!(level_from_volume(1_000_000, 3_000_000, area, 0), 1_000_000);
        assert_eq!(
            level_from_volume(1_000_000, 1_200_000, area, volume),
            1_200_000
        );
    }

    #[test]
    fn largest_remainder_scaling_is_exact() {
        let mut parts = vec![7, 7, 7];
        scale_to_budget(&mut parts, 10);
        assert_eq!(parts.iter().sum::<i64>(), 10);
        assert_eq!(parts, vec![4, 3, 3]);
        let mut parts = vec![1, 2];
        scale_to_budget(&mut parts, 10);
        assert_eq!(parts, vec![1, 2]);
    }

    #[test]
    fn two_vessels_conserve_volume_and_equalise() {
        let (mut volumes, mut network) = two_vessels();
        let initial = network.total_volume();
        let mut sources: i128 = 0;
        let mut sinks: i128 = 0;
        for _ in 0..1_200 {
            let step = network.step(&volumes).expect("step");
            for state in step.network.edge_states.values() {
                match network.edges[&state.edge_id].kind {
                    WaterFlowEdgeKindV1::Source { .. } => {
                        sources += i128::from(state.last_flux_cubic_millimetres);
                    }
                    WaterFlowEdgeKindV1::Sink { .. } => {
                        sinks += i128::from(-state.last_flux_cubic_millimetres);
                    }
                    _ => {}
                }
            }
            network = step.network;
            volumes = step.volumes;
            assert_eq!(network.total_volume(), initial + sources - sinks);
        }
        // Vessel A's floor (1.0 m) is above the level B can reach with all
        // the water (1.5 m^3 over 4.2 m^2 = 0.357 m), so the equilibrium is A
        // drained to its floor (plus the head that passes the source rate)
        // and B holding the rest.
        let level_a = volumes.states[&id(1)].level_micrometres;
        let level_b = volumes.states[&id(2)].level_micrometres;
        assert!(level_a - 1_000_000 <= 1_000, "{level_a}");
        let expected_b = level_from_volume(
            0,
            2_000_000,
            cell_area_square_millimetres([0, 0, 0], [2_800_000, 0, 1_500_000]),
            network.cell_volume(id(2)).unwrap(),
        );
        assert_eq!(level_b, expected_b);
        assert!(network.edge_flux(id(0x21)).is_some());
    }

    #[test]
    fn communicating_vessels_equalise_when_floors_allow() {
        let volumes = WaterVolumeSetV1::from_definitions([
            vessel(1, 0, 2_000_000, 1_500_000, 1_500_000),
            vessel(2, 0, 2_800_000, 1_500_000, 0),
        ])
        .expect("volumes");
        let mut network = WaterFlowNetworkV1::from_edges(
            30,
            [WaterFlowEdgeV1 {
                edge_id: id(0x21),
                cell_a: id(1),
                cell_b: Some(id(2)),
                kind: WaterFlowEdgeKindV1::Pipe {
                    invert_micrometres: 0,
                    area_square_millimetres: 40_000,
                    coefficient_permille: 400,
                },
            }],
            &volumes,
        )
        .expect("network");
        let mut volumes = volumes;
        let initial = network.total_volume();
        // Analytic equalisation time of the Torricelli law:
        // tau = 2 sqrt(dh0) * (Aa Ab / (Aa + Ab)) / (cd A sqrt(2 g))
        //     = 2 * 1.2247 * 1.75 / (0.016 * 4.429) = 60.5 s; run 2 tau.
        for _ in 0..(2 * 61 * 30) {
            let step = network.step(&volumes).expect("step");
            network = step.network;
            volumes = step.volumes;
        }
        assert_eq!(network.total_volume(), initial);
        let level_a = volumes.states[&id(1)].level_micrometres;
        let level_b = volumes.states[&id(2)].level_micrometres;
        // Common level: 1.5 * 3 / (3 + 4.2) m = 0.625 m.
        assert!((level_a - level_b).abs() <= 1_000, "{level_a} vs {level_b}");
        assert!((level_a - 625_000).abs() <= 1_000, "{level_a}");
    }

    #[test]
    fn closed_gate_stops_the_exchange() {
        let (volumes, network) = two_vessels();
        let (closed, event) = network
            .apply_command(
                &WaterFlowCommandV1::SetGate {
                    edge_id: id(0x21),
                    expected_record_revision: 0,
                    opening_permille: 0,
                },
                7,
            )
            .expect("gate command");
        assert_eq!(event.record_revision, 1);
        assert_eq!(event.opening_permille, 0);
        let step = closed.step(&volumes).expect("step");
        assert_eq!(step.network.edge_flux(id(0x21)), Some(0));
        assert!(
            network
                .step(&volumes)
                .expect("open step")
                .network
                .edge_flux(id(0x21))
                .unwrap()
                > 0
        );
    }

    #[test]
    fn commands_are_rejected_deterministically() {
        let (_, network) = two_vessels();
        let unknown = WaterFlowCommandV1::SetGate {
            edge_id: id(0x99),
            expected_record_revision: 0,
            opening_permille: 500,
        };
        assert_eq!(
            network.apply_command(&unknown, 1).unwrap_err(),
            WaterFlowRejectionV1::UnknownEdge
        );
        let wrong_kind = WaterFlowCommandV1::SetPump {
            edge_id: id(0x21),
            expected_record_revision: 0,
            enabled: false,
        };
        assert_eq!(
            network.apply_command(&wrong_kind, 1).unwrap_err(),
            WaterFlowRejectionV1::WrongKind
        );
        let stale = WaterFlowCommandV1::SetGate {
            edge_id: id(0x21),
            expected_record_revision: 3,
            opening_permille: 500,
        };
        assert_eq!(
            network.apply_command(&stale, 1).unwrap_err(),
            WaterFlowRejectionV1::RevisionStale
        );
        let out_of_range = WaterFlowCommandV1::SetGate {
            edge_id: id(0x21),
            expected_record_revision: 0,
            opening_permille: 1001,
        };
        assert_eq!(
            network.apply_command(&out_of_range, 1).unwrap_err(),
            WaterFlowRejectionV1::OpeningOutOfRange
        );
    }

    #[test]
    fn records_and_payloads_round_trip() {
        let (volumes, network) = two_vessels();
        let stepped = network.step(&volumes).expect("step").network;
        let bytes = stepped.canonical_record().expect("record");
        let decoded = WaterFlowNetworkV1::from_record(&bytes, CanonicalDecodeLimits::default())
            .expect("decode");
        assert_eq!(decoded, stepped);
        let command = WaterFlowCommandV1::SetSource {
            edge_id: id(0x22),
            expected_record_revision: 0,
            rate_cubic_millimetres_per_second: 250_000,
        };
        let payload = command.canonical_payload_bytes().expect("payload");
        assert_eq!(
            WaterFlowCommandV1::from_canonical_payload_bytes(
                &payload,
                CanonicalDecodeLimits::default()
            )
            .expect("command"),
            command
        );
        let event = WaterFlowChangedV1 {
            edge_id: id(0x22),
            record_revision: 1,
            opening_permille: 1000,
            enabled: true,
            rate_cubic_millimetres_per_second: 250_000,
            boundary_tick: 9,
        };
        let payload = event.canonical_payload_bytes().expect("payload");
        assert_eq!(
            WaterFlowChangedV1::from_canonical_payload_bytes(&payload).expect("event"),
            event
        );
        assert!(WaterFlowNetworkV1::empty().validate().is_ok());
    }

    #[test]
    fn authored_level_override_resynchronises_the_volume() {
        let (volumes, network) = two_vessels();
        let (volumes, _) = volumes
            .apply_command(
                &super::super::water::WaterVolumeCommandV1::SetLevel {
                    volume_id: id(1),
                    expected_record_revision: 0,
                    level_micrometres: 1_200_000,
                },
                1,
            )
            .expect("level");
        let step = network.step(&volumes).expect("step");
        let cell = step.network.cells[&id(1)];
        assert_eq!(cell.synced_record_revision, 1);
        assert!(cell.volume_cubic_millimetres < 1_500_000_000);
    }
}
