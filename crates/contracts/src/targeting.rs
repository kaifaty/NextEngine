use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I64, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, DecodedCanonicalSegment,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::ids::{ContentHash, IdentifierError, PersistentId, SchemaId, content_hash_from_bytes};
use crate::input::{INGRESS_ASSIGNMENT_SCHEMA_VERSION, IngressAssignmentV1, InputContractError};
use crate::physics::{
    MAX_PHYSICS_QUERY_DISTANCE_MICROMETRES, MAX_PHYSICS_QUERY_PUBLISHED_HITS, PhysicsBodyIdV1,
    PhysicsContractError, PhysicsQueryCardinalityV1, PhysicsQueryFilterV1, PhysicsQueryGeometryV1,
    PhysicsQueryKindV1, PhysicsQueryRequestV1,
};

pub const TARGETING_QUERY_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const TARGETING_INTENT_SCHEMA_VERSION: u16 = 1;
pub const AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION: u16 = 1;

const TARGETING_OWNER_ID: &str = "nextengine.player-experience";
const TARGETING_PROFILE_SCHEMA_ID: &str = "nextengine.targeting-query-profile";
const TARGETING_INTENT_SCHEMA_ID: &str = "nextengine.targeting-intent";
const AUTHORITATIVE_TARGETING_QUERY_SCHEMA_ID: &str = "nextengine.authoritative-targeting-query";
const SEGMENT_V1: &str = "v1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TargetVisibilityPolicyV1 {
    NotRequiredB0 = 1,
}

impl TargetVisibilityPolicyV1 {
    fn from_tag(tag: u8) -> Result<Self, TargetingContractError> {
        match tag {
            1 => Ok(Self::NotRequiredB0),
            other => Err(TargetingContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetingQueryProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub profile_revision: u32,
    pub query_kind: PhysicsQueryKindV1,
    pub maximum_distance_micrometres: i64,
    pub maximum_candidate_hits: u32,
    pub filter: PhysicsQueryFilterV1,
    pub visibility_policy: TargetVisibilityPolicyV1,
}

impl TargetingQueryProfileV1 {
    pub fn validate(&self) -> Result<(), TargetingContractError> {
        if self.schema_version != TARGETING_QUERY_PROFILE_SCHEMA_VERSION
            || self.profile_revision == 0
            || !(0..=MAX_PHYSICS_QUERY_DISTANCE_MICROMETRES)
                .contains(&self.maximum_distance_micrometres)
            || self.maximum_candidate_hits == 0
            || self.maximum_candidate_hits > MAX_PHYSICS_QUERY_PUBLISHED_HITS
        {
            return Err(TargetingContractError::InvalidProfile);
        }
        self.filter.validate()?;
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            TARGETING_OWNER_ID,
            TARGETING_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.profile_id.as_str().as_bytes().to_vec(),
                ),
                field_u32(3, self.profile_revision),
                field_u8(4, self.query_kind as u8),
                field_i64(5, self.maximum_distance_micrometres),
                field_u32(6, self.maximum_candidate_hits),
                CanonicalField::new(7, CANONICAL_TYPE_BYTES, self.filter.canonical_bytes()?),
                field_u8(8, self.visibility_policy as u8),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, TargetingContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            TARGETING_OWNER_ID,
            TARGETING_PROFILE_SCHEMA_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_U8),
                (5, CANONICAL_TYPE_I64),
                (6, CANONICAL_TYPE_U32),
                (7, CANONICAL_TYPE_BYTES),
                (8, CANONICAL_TYPE_U8),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            profile_id: SchemaId::new(read_utf8(&segment, 2)?)?,
            profile_revision: read_u32(&segment, 3)?,
            query_kind: PhysicsQueryKindV1::from_tag(read_u8(&segment, 4)?)?,
            maximum_distance_micrometres: read_i64(&segment, 5)?,
            maximum_candidate_hits: read_u32(&segment, 6)?,
            filter: PhysicsQueryFilterV1::from_canonical_bytes(field(&segment, 7)?, limits)?,
            visibility_policy: TargetVisibilityPolicyV1::from_tag(read_u8(&segment, 8)?)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        contract_hash(
            b"nextengine.targeting-query-profile.v1\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetingIntentV1 {
    pub schema_version: u16,
    pub assignment: IngressAssignmentV1,
    pub player_id: PersistentId,
    pub ability_id: SchemaId,
    pub action_id: SchemaId,
    pub aim_q15: [i16; 2],
    pub query_kind: PhysicsQueryKindV1,
    pub targeting_profile_id: SchemaId,
    pub targeting_profile_hash: ContentHash,
    pub proposed_target_id: Option<PersistentId>,
}

impl TargetingIntentV1 {
    pub fn validate(&self) -> Result<(), TargetingContractError> {
        if self.schema_version != TARGETING_INTENT_SCHEMA_VERSION
            || self.assignment.schema_version != INGRESS_ASSIGNMENT_SCHEMA_VERSION
        {
            return Err(TargetingContractError::InvalidIntent);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut aim = Vec::with_capacity(4);
        aim.extend_from_slice(&self.aim_q15[0].to_le_bytes());
        aim.extend_from_slice(&self.aim_q15[1].to_le_bytes());
        encode_canonical_segment(
            TARGETING_OWNER_ID,
            TARGETING_INTENT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_BYTES, self.assignment.canonical_bytes()?),
                field_id(3, self.player_id.as_bytes()),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.ability_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.action_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(6, CANONICAL_TYPE_BYTES, aim),
                field_u8(7, self.query_kind as u8),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.targeting_profile_id.as_str().as_bytes().to_vec(),
                ),
                field_hash(9, self.targeting_profile_hash),
                CanonicalField::new(
                    10,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_optional_id(self.proposed_target_id),
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, TargetingContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            TARGETING_OWNER_ID,
            TARGETING_INTENT_SCHEMA_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_UTF8_NFC),
                (5, CANONICAL_TYPE_UTF8_NFC),
                (6, CANONICAL_TYPE_BYTES),
                (7, CANONICAL_TYPE_U8),
                (8, CANONICAL_TYPE_UTF8_NFC),
                (9, CANONICAL_TYPE_HASH256),
                (10, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        let aim = field(&segment, 6)?;
        if aim.len() != 4 {
            return Err(TargetingContractError::FieldLength);
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            assignment: IngressAssignmentV1::from_canonical_bytes(field(&segment, 2)?, limits)?,
            player_id: PersistentId::from_bytes(exact(field(&segment, 3)?)?),
            ability_id: SchemaId::new(read_utf8(&segment, 4)?)?,
            action_id: SchemaId::new(read_utf8(&segment, 5)?)?,
            aim_q15: [
                i16::from_le_bytes(exact(&aim[..2])?),
                i16::from_le_bytes(exact(&aim[2..])?),
            ],
            query_kind: PhysicsQueryKindV1::from_tag(read_u8(&segment, 7)?)?,
            targeting_profile_id: SchemaId::new(read_utf8(&segment, 8)?)?,
            targeting_profile_hash: read_hash(&segment, 9)?,
            proposed_target_id: decode_optional_id(field(&segment, 10)?)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn intent_hash(&self) -> Result<ContentHash, CanonicalError> {
        contract_hash(
            b"nextengine.targeting-intent.v1\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeTargetingQueryV1 {
    pub schema_version: u16,
    pub intent_hash: ContentHash,
    pub assignment: IngressAssignmentV1,
    pub actor_id: PersistentId,
    pub actor_body_id: PhysicsBodyIdV1,
    pub actor_body_revision: u64,
    pub targeting_profile_hash: ContentHash,
    pub physics_query: PhysicsQueryRequestV1,
}

impl AuthoritativeTargetingQueryV1 {
    pub fn validate_against(
        &self,
        intent: &TargetingIntentV1,
        profile: &TargetingQueryProfileV1,
    ) -> Result<(), TargetingContractError> {
        intent.validate()?;
        profile.validate()?;
        self.physics_query.validate()?;
        let mut expected_filter = profile.filter.clone();
        if let Err(index) = expected_filter
            .excluded_bodies
            .binary_search(&self.actor_body_id)
        {
            expected_filter
                .excluded_bodies
                .insert(index, self.actor_body_id);
        }
        expected_filter.validate()?;
        let cardinality_matches_profile = match self.physics_query.cardinality {
            PhysicsQueryCardinalityV1::Closest => {
                profile.maximum_candidate_hits == 1
                    && self.physics_query.maximum_published_hits == 1
            }
            PhysicsQueryCardinalityV1::All => {
                self.physics_query.maximum_published_hits == profile.maximum_candidate_hits
            }
            PhysicsQueryCardinalityV1::Any => false,
        };
        if self.schema_version != AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION
            || self.intent_hash != intent.intent_hash()?
            || self.assignment != intent.assignment
            || self.actor_id != intent.player_id
            || self.actor_id != self.actor_body_id.subject_id
            || self.targeting_profile_hash != intent.targeting_profile_hash
            || self.targeting_profile_hash != profile.profile_hash()?
            || intent.targeting_profile_id != profile.profile_id
            || intent.query_kind != profile.query_kind
            || self.physics_query.geometry.kind() != profile.query_kind
            || self.physics_query.filter != expected_filter
            || !cardinality_matches_profile
        {
            return Err(TargetingContractError::ProfileMismatch);
        }
        match self.physics_query.geometry {
            PhysicsQueryGeometryV1::RayCast {
                maximum_distance_micrometres,
                ..
            }
            | PhysicsQueryGeometryV1::ShapeCast {
                maximum_distance_micrometres,
                ..
            }
            | PhysicsQueryGeometryV1::ClosestPoint {
                maximum_distance_micrometres,
                ..
            } if maximum_distance_micrometres == profile.maximum_distance_micrometres => {}
            PhysicsQueryGeometryV1::Overlap { .. } if profile.maximum_distance_micrometres == 0 => {
            }
            _ => return Err(TargetingContractError::ProfileMismatch),
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            TARGETING_OWNER_ID,
            AUTHORITATIVE_TARGETING_QUERY_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_hash(2, self.intent_hash),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, self.assignment.canonical_bytes()?),
                field_id(4, self.actor_id.as_bytes()),
                CanonicalField::new(5, CANONICAL_TYPE_STRUCT, encode_body_id(self.actor_body_id)),
                field_u64(6, self.actor_body_revision),
                field_hash(7, self.targeting_profile_hash),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_BYTES,
                    self.physics_query.canonical_bytes()?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, TargetingContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            TARGETING_OWNER_ID,
            AUTHORITATIVE_TARGETING_QUERY_SCHEMA_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_HASH256),
                (3, CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_STRUCT),
                (6, CANONICAL_TYPE_U64),
                (7, CANONICAL_TYPE_HASH256),
                (8, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            intent_hash: read_hash(&segment, 2)?,
            assignment: IngressAssignmentV1::from_canonical_bytes(field(&segment, 3)?, limits)?,
            actor_id: PersistentId::from_bytes(exact(field(&segment, 4)?)?),
            actor_body_id: decode_body_id(field(&segment, 5)?, limits)?,
            actor_body_revision: read_u64(&segment, 6)?,
            targeting_profile_hash: read_hash(&segment, 7)?,
            physics_query: PhysicsQueryRequestV1::from_canonical_bytes(
                field(&segment, 8)?,
                limits,
            )?,
        };
        if value.schema_version != AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION {
            return Err(TargetingContractError::UnsupportedVersion(u32::from(
                value.schema_version,
            )));
        }
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn query_hash(&self) -> Result<ContentHash, CanonicalError> {
        contract_hash(
            b"nextengine.authoritative-targeting-query.v1\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetingContractError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identifier(IdentifierError),
    Input(InputContractError),
    Physics(PhysicsContractError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType,
    FieldLength,
    UnknownTag(u8),
    UnsupportedVersion(u32),
    InvalidProfile,
    InvalidIntent,
    ProfileMismatch,
    LimitExceeded,
    NonCanonicalEncoding,
}

impl TargetingContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::InvalidProfile | Self::ProfileMismatch => "TARGETING_PROFILE_MISMATCH",
            Self::InvalidIntent => "TARGETING_INTENT_INVALID",
            Self::LimitExceeded => "TARGETING_LIMIT_EXCEEDED",
            Self::UnsupportedVersion(_) => "TARGETING_SCHEMA_UNSUPPORTED",
            _ => "TARGETING_CONTRACT_INVALID",
        }
    }
}

impl Display for TargetingContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for TargetingContractError {}

impl From<CanonicalDecodeError> for TargetingContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for TargetingContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<IdentifierError> for TargetingContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<InputContractError> for TargetingContractError {
    fn from(error: InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<PhysicsContractError> for TargetingContractError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    expected: &[(u32, u8)],
) -> Result<DecodedCanonicalSegment, TargetingContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != SEGMENT_V1
    {
        return Err(TargetingContractError::WrongEnvelope);
    }
    for field in &segment.fields {
        let Some((_, expected_tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(TargetingContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *expected_tag {
            return Err(TargetingContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if segment.field(*id).is_none() {
            return Err(TargetingContractError::MissingField(*id));
        }
    }
    Ok(segment)
}

fn field(segment: &DecodedCanonicalSegment, id: u32) -> Result<&[u8], TargetingContractError> {
    Ok(&segment
        .field(id)
        .ok_or(TargetingContractError::MissingField(id))?
        .payload)
}

fn exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], TargetingContractError> {
    bytes
        .try_into()
        .map_err(|_| TargetingContractError::FieldLength)
}

fn read_u8(segment: &DecodedCanonicalSegment, id: u32) -> Result<u8, TargetingContractError> {
    Ok(exact::<1>(field(segment, id)?)?[0])
}

fn read_u16(segment: &DecodedCanonicalSegment, id: u32) -> Result<u16, TargetingContractError> {
    Ok(u16::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_u32(segment: &DecodedCanonicalSegment, id: u32) -> Result<u32, TargetingContractError> {
    Ok(u32::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_u64(segment: &DecodedCanonicalSegment, id: u32) -> Result<u64, TargetingContractError> {
    Ok(u64::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_i64(segment: &DecodedCanonicalSegment, id: u32) -> Result<i64, TargetingContractError> {
    Ok(i64::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_hash(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<ContentHash, TargetingContractError> {
    Ok(content_hash_from_bytes(exact(field(segment, id)?)?))
}

fn read_utf8(segment: &DecodedCanonicalSegment, id: u32) -> Result<&str, TargetingContractError> {
    std::str::from_utf8(field(segment, id)?)
        .map_err(|_| TargetingContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
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

fn field_id<const N: usize>(id: u32, value: &[u8; N]) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.to_vec())
}

fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

fn encode_optional_id(value: Option<PersistentId>) -> Vec<u8> {
    match value {
        None => vec![0],
        Some(value) => {
            let mut bytes = Vec::with_capacity(17);
            bytes.push(1);
            bytes.extend_from_slice(value.as_bytes());
            bytes
        }
    }
}

fn decode_optional_id(bytes: &[u8]) -> Result<Option<PersistentId>, TargetingContractError> {
    match bytes {
        [0] => Ok(None),
        [1, rest @ ..] if rest.len() == 16 => Ok(Some(PersistentId::from_bytes(exact(rest)?))),
        _ => Err(TargetingContractError::FieldLength),
    }
}

fn encode_body_id(body_id: PhysicsBodyIdV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(20);
    bytes.extend_from_slice(body_id.subject_id.as_bytes());
    bytes.extend_from_slice(&body_id.body_slot.to_le_bytes());
    bytes
}

fn decode_body_id(
    bytes: &[u8],
    _limits: CanonicalDecodeLimits,
) -> Result<PhysicsBodyIdV1, TargetingContractError> {
    if bytes.len() != 20 {
        return Err(TargetingContractError::FieldLength);
    }
    Ok(PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes(exact(&bytes[..16])?),
        body_slot: u32::from_le_bytes(exact(&bytes[16..])?),
    })
}

fn require_round_trip(original: &[u8], encoded: Vec<u8>) -> Result<(), TargetingContractError> {
    if original != encoded {
        return Err(TargetingContractError::NonCanonicalEncoding);
    }
    Ok(())
}

fn contract_hash(domain: &[u8], bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{CommandStreamId, InputSourceId, PhysicsWorldId, content_hash_from_bytes};
    use crate::input::IngressAssignmentV1;
    use crate::physics::{
        PHYSICS_QUERY_SCHEMA_VERSION, PhysicsQueryIdV1, PhysicsSnapshotSelectorV1,
    };

    fn assignment() -> IngressAssignmentV1 {
        IngressAssignmentV1 {
            schema_version: INGRESS_ASSIGNMENT_SCHEMA_VERSION,
            queue_generation: 2,
            assigned_tick: 9,
            source_class: SchemaId::new("nextengine.input.player-action").expect("source class"),
            source_id: InputSourceId::from_bytes([1; 16]),
            source_sequence: 3,
            payload_hash: content_hash_from_bytes([2; 32]),
        }
    }

    fn filter() -> PhysicsQueryFilterV1 {
        PhysicsQueryFilterV1 {
            query_collision_layer: 0,
            query_collision_mask: 1,
            include_solid: true,
            include_sensor: false,
            include_query_only: false,
            excluded_bodies: Vec::new(),
            excluded_shapes: Vec::new(),
        }
    }

    fn profile() -> TargetingQueryProfileV1 {
        TargetingQueryProfileV1 {
            schema_version: TARGETING_QUERY_PROFILE_SCHEMA_VERSION,
            profile_id: SchemaId::new("nextengine.targeting.interaction-nearby.v1")
                .expect("profile id"),
            profile_revision: 1,
            query_kind: PhysicsQueryKindV1::ClosestPoint,
            maximum_distance_micrometres: 2_000_000,
            maximum_candidate_hits: 256,
            filter: filter(),
            visibility_policy: TargetVisibilityPolicyV1::NotRequiredB0,
        }
    }

    fn intent(profile: &TargetingQueryProfileV1) -> TargetingIntentV1 {
        TargetingIntentV1 {
            schema_version: TARGETING_INTENT_SCHEMA_VERSION,
            assignment: assignment(),
            player_id: PersistentId::from_bytes([3; 16]),
            ability_id: SchemaId::new("nextengine.ability.interact").expect("ability id"),
            action_id: SchemaId::new("nextengine.action.interact").expect("action id"),
            aim_q15: [0, 0],
            query_kind: PhysicsQueryKindV1::ClosestPoint,
            targeting_profile_id: profile.profile_id.clone(),
            targeting_profile_hash: profile.profile_hash().expect("profile hash"),
            proposed_target_id: Some(PersistentId::from_bytes([4; 16])),
        }
    }

    #[test]
    fn profile_intent_and_authoritative_query_round_trip_exactly() {
        let profile = profile();
        let intent = intent(&profile);
        let actor_id = intent.player_id;
        let body_id = PhysicsBodyIdV1 {
            subject_id: actor_id,
            body_slot: 0,
        };
        let physics_query = PhysicsQueryRequestV1 {
            schema_version: PHYSICS_QUERY_SCHEMA_VERSION,
            query_id: PhysicsQueryIdV1 {
                physics_tick: 13,
                query_slot: 0,
                issuer_stream_id: CommandStreamId::from_bytes([5; 16]),
            },
            world_id: PhysicsWorldId::from_bytes([6; 16]),
            snapshot_selector: PhysicsSnapshotSelectorV1 {
                physics_tick: 13,
                completed_substep: 0,
                physics_snapshot_hash: content_hash_from_bytes([7; 32]),
            },
            geometry: PhysicsQueryGeometryV1::ClosestPoint {
                point_micrometres: [0, 900_000, 0],
                maximum_distance_micrometres: profile.maximum_distance_micrometres,
            },
            filter: PhysicsQueryFilterV1 {
                excluded_bodies: vec![body_id],
                ..profile.filter.clone()
            },
            cardinality: PhysicsQueryCardinalityV1::All,
            maximum_published_hits: profile.maximum_candidate_hits,
        };
        let query = AuthoritativeTargetingQueryV1 {
            schema_version: AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION,
            intent_hash: intent.intent_hash().expect("intent hash"),
            assignment: intent.assignment.clone(),
            actor_id,
            actor_body_id: body_id,
            actor_body_revision: 4,
            targeting_profile_hash: profile.profile_hash().expect("profile hash"),
            physics_query,
        };
        query
            .validate_against(&intent, &profile)
            .expect("query closure");
        assert_ne!(
            query.assignment.assigned_tick, query.physics_query.snapshot_selector.physics_tick,
            "gameplay ingress and closed physics clocks are distinct selectors",
        );
        let mut actor_not_excluded = query.clone();
        actor_not_excluded
            .physics_query
            .filter
            .excluded_bodies
            .clear();
        assert_eq!(
            actor_not_excluded.validate_against(&intent, &profile),
            Err(TargetingContractError::ProfileMismatch),
        );
        let mut substituted_actor = query.clone();
        let other_actor = PersistentId::from_bytes([99; 16]);
        substituted_actor.actor_id = other_actor;
        substituted_actor.actor_body_id.subject_id = other_actor;
        substituted_actor.physics_query.filter.excluded_bodies =
            vec![substituted_actor.actor_body_id];
        assert_eq!(
            substituted_actor.validate_against(&intent, &profile),
            Err(TargetingContractError::ProfileMismatch),
        );

        let profile_bytes = profile.canonical_bytes().expect("profile bytes");
        assert_eq!(
            TargetingQueryProfileV1::from_canonical_bytes(
                &profile_bytes,
                CanonicalDecodeLimits::default(),
            )
            .expect("profile round trip"),
            profile,
        );
        let intent_bytes = intent.canonical_bytes().expect("intent bytes");
        assert_eq!(
            TargetingIntentV1::from_canonical_bytes(
                &intent_bytes,
                CanonicalDecodeLimits::default(),
            )
            .expect("intent round trip"),
            intent,
        );
        let query_bytes = query.canonical_bytes().expect("query bytes");
        assert_eq!(
            AuthoritativeTargetingQueryV1::from_canonical_bytes(
                &query_bytes,
                CanonicalDecodeLimits::default(),
            )
            .expect("query round trip"),
            query,
        );
    }

    #[test]
    fn authoritative_query_rejects_camera_independent_profile_drift() {
        let profile = profile();
        let intent = intent(&profile);
        let body_id = PhysicsBodyIdV1 {
            subject_id: intent.player_id,
            body_slot: 0,
        };
        let mut query = AuthoritativeTargetingQueryV1 {
            schema_version: AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION,
            intent_hash: intent.intent_hash().expect("intent hash"),
            assignment: intent.assignment.clone(),
            actor_id: intent.player_id,
            actor_body_id: body_id,
            actor_body_revision: 0,
            targeting_profile_hash: profile.profile_hash().expect("profile hash"),
            physics_query: PhysicsQueryRequestV1 {
                schema_version: PHYSICS_QUERY_SCHEMA_VERSION,
                query_id: PhysicsQueryIdV1 {
                    physics_tick: intent.assignment.assigned_tick,
                    query_slot: 0,
                    issuer_stream_id: CommandStreamId::from_bytes([8; 16]),
                },
                world_id: PhysicsWorldId::from_bytes([9; 16]),
                snapshot_selector: PhysicsSnapshotSelectorV1 {
                    physics_tick: intent.assignment.assigned_tick,
                    completed_substep: 0,
                    physics_snapshot_hash: content_hash_from_bytes([10; 32]),
                },
                geometry: PhysicsQueryGeometryV1::ClosestPoint {
                    point_micrometres: [0; 3],
                    maximum_distance_micrometres: profile.maximum_distance_micrometres,
                },
                filter: PhysicsQueryFilterV1 {
                    excluded_bodies: vec![body_id],
                    ..profile.filter.clone()
                },
                cardinality: PhysicsQueryCardinalityV1::All,
                maximum_published_hits: profile.maximum_candidate_hits,
            },
        };
        query.physics_query.snapshot_selector.physics_tick += 1;
        assert_eq!(
            query.validate_against(&intent, &profile),
            Err(TargetingContractError::Physics(
                PhysicsContractError::InvalidQuery
            ))
        );
    }
}
