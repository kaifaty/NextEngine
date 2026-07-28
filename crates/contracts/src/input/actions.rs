use crate::canonical::*;
use crate::ids::*;

use super::codec::*;
use super::constants::*;
use super::profiles::RuntimeAdmissionLimitsV1;

#[must_use]
pub fn core_player_action_map_v1_hash() -> ContentHash {
    let mut preimage = b"nextengine.core-player-action-map.v1\0".to_vec();
    for action_id in [CORE_INTERACT_ACTION_ID, CORE_MOVE_ACTION_ID] {
        preimage.extend_from_slice(
            &u32::try_from(action_id.len())
                .expect("built-in action identifier length fits u32")
                .to_le_bytes(),
        );
        preimage.extend_from_slice(action_id.as_bytes());
    }
    ContentHash::from_bytes(sha256(&preimage))
}

#[must_use]
pub fn core_player_action_map_v2_hash() -> ContentHash {
    let mut preimage = b"nextengine.core-player-action-map.v2\0".to_vec();
    for action_id in [
        CORE_EQUIP_USE_ACTION_ID,
        CORE_INTERACT_ACTION_ID,
        CORE_MELEE_ACTION_ID,
        CORE_MOVE_ACTION_ID,
        CORE_PICKUP_ACTION_ID,
    ] {
        preimage.extend_from_slice(
            &u32::try_from(action_id.len())
                .expect("built-in action identifier length fits u32")
                .to_le_bytes(),
        );
        preimage.extend_from_slice(action_id.as_bytes());
    }
    ContentHash::from_bytes(sha256(&preimage))
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PlayerActionPhaseV1 {
    Started = 1,
    Performed = 2,
    Completed = 3,
    Cancelled = 4,
}

impl PlayerActionPhaseV1 {
    fn from_tag(tag: u8) -> Result<Self, InputContractError> {
        match tag {
            1 => Ok(Self::Started),
            2 => Ok(Self::Performed),
            3 => Ok(Self::Completed),
            4 => Ok(Self::Cancelled),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlayerActionValueV1 {
    Digital(bool),
    ScalarQ15(i16),
    Vector2Q15([i16; 2]),
}

impl PlayerActionValueV1 {
    fn canonical_tagged_payload(self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, nested_type, payload) = match self {
            Self::Digital(value) => (1, CANONICAL_TYPE_BOOL, vec![u8::from(value)]),
            Self::ScalarQ15(value) => (2, CANONICAL_TYPE_I16, value.to_le_bytes().to_vec()),
            Self::Vector2Q15(value) => {
                let mut payload = Vec::with_capacity(4);
                payload.extend_from_slice(&value[0].to_le_bytes());
                payload.extend_from_slice(&value[1].to_le_bytes());
                (3, CANONICAL_TYPE_BYTES, payload)
            }
        };
        let mut bytes = vec![tag];
        bytes.extend_from_slice(&encode_nested(nested_type, &payload)?);
        Ok(bytes)
    }

    fn from_tagged_payload(
        payload: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let mut cursor = CanonicalCursor::new(payload);
        let tag = cursor.read_u8()?;
        let (nested_type, nested) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        match tag {
            1 if nested_type == CANONICAL_TYPE_BOOL && nested.len() == 1 => match nested[0] {
                0 => Ok(Self::Digital(false)),
                1 => Ok(Self::Digital(true)),
                _ => Err(InputContractError::InvalidValue),
            },
            2 if nested_type == CANONICAL_TYPE_I16 => {
                Ok(Self::ScalarQ15(i16::from_le_bytes(exact(nested)?)))
            }
            3 if nested_type == CANONICAL_TYPE_BYTES && nested.len() == 4 => {
                Ok(Self::Vector2Q15([
                    i16::from_le_bytes(exact(&nested[..2])?),
                    i16::from_le_bytes(exact(&nested[2..])?),
                ]))
            }
            1..=3 => Err(InputContractError::FieldType),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlayerActionV1 {
    pub action_id: SchemaId,
    pub phase: PlayerActionPhaseV1,
    pub value: PlayerActionValueV1,
    pub semantic_occurrence_ordinal: u32,
}

impl PlayerActionV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.action_id.as_str().as_bytes().to_vec(),
            ),
            field_u8(2, self.phase as u8),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_TAGGED_UNION,
                self.value.canonical_tagged_payload()?,
            ),
            field_u32(4, self.semantic_occurrence_ordinal),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_TAGGED_UNION),
                (4, CANONICAL_TYPE_U32),
            ],
        )?;
        Ok(Self {
            action_id: SchemaId::new(read_utf8_field(&fields, 1)?)?,
            phase: PlayerActionPhaseV1::from_tag(read_u8_fields(&fields, 2)?)?,
            value: PlayerActionValueV1::from_tagged_payload(
                field_from(&fields, 3)?.payload.as_slice(),
                limits,
            )?,
            semantic_occurrence_ordinal: read_u32_fields(&fields, 4)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerActionFrameV1 {
    pub schema_version: u16,
    pub controller_id: PersistentId,
    pub logical_frame_sequence: u64,
    pub action_map_hash: ContentHash,
    pub action_map_revision: u64,
    pub context_stack_hash: ContentHash,
    pub context_stack_revision: u64,
    pub actions: Vec<PlayerActionV1>,
}

impl PlayerActionFrameV1 {
    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != PLAYER_ACTION_FRAME_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "player action frame",
                version: u32::from(self.schema_version),
            });
        }
        if self.actions.len() > MAX_PLAYER_ACTIONS_PER_FRAME {
            return Err(InputContractError::ResourceLimit);
        }
        let order_is_invalid = self.actions.windows(2).any(|pair| {
            (&pair[0].action_id, pair[0].semantic_occurrence_ordinal)
                >= (&pair[1].action_id, pair[1].semantic_occurrence_ordinal)
        });
        let mut ordinals = self
            .actions
            .iter()
            .map(|action| action.semantic_occurrence_ordinal)
            .collect::<Vec<_>>();
        ordinals.sort_unstable();
        let ordinals_are_invalid = ordinals
            .iter()
            .enumerate()
            .any(|(index, ordinal)| usize::try_from(*ordinal).ok() != Some(index));
        if order_is_invalid || ordinals_are_invalid {
            return Err(InputContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let actions = encode_sequence(
            self.actions
                .iter()
                .map(PlayerActionV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        encode_canonical_segment(
            INPUT_OWNER_ID,
            PLAYER_ACTION_FRAME_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.controller_id.as_bytes()),
                field_u64(3, self.logical_frame_sequence),
                field_hash(4, self.action_map_hash),
                field_u64(5, self.action_map_revision),
                field_hash(6, self.context_stack_hash),
                field_u64(7, self.context_stack_revision),
                CanonicalField::new(8, CANONICAL_TYPE_SEQUENCE, actions),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            INPUT_OWNER_ID,
            PLAYER_ACTION_FRAME_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_U64),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let actions = decode_sequence(field(&segment, 8)?, limits)?
            .into_iter()
            .map(|bytes| PlayerActionV1::from_record(&bytes, limits))
            .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            controller_id: PersistentId::from_bytes(exact(field(&segment, 2)?)?),
            logical_frame_sequence: read_u64(&segment, 3)?,
            action_map_hash: read_hash(&segment, 4)?,
            action_map_revision: read_u64(&segment, 5)?,
            context_stack_hash: read_hash(&segment, 6)?,
            context_stack_revision: read_u64(&segment, 7)?,
            actions,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputSampleV1 {
    pub schema_version: u16,
    pub source_class: SchemaId,
    pub source_id: InputSourceId,
    pub source_sequence: u64,
    pub payload_schema_id: SchemaId,
    pub payload_schema_version: u32,
    pub payload: Vec<u8>,
    pub sampled_wall_time: Option<i64>,
}

impl InputSampleV1 {
    pub fn validate(&self, limits: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != INPUT_SAMPLE_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "input sample",
                version: u32::from(self.schema_version),
            });
        }
        if self.source_class.as_str().len()
            > usize::try_from(limits.max_namespaced_id_bytes)
                .map_err(|_| InputContractError::ResourceLimit)?
            || self.payload_schema_id.as_str().len()
                > usize::try_from(limits.max_namespaced_id_bytes)
                    .map_err(|_| InputContractError::ResourceLimit)?
            || self.payload.len()
                > usize::try_from(limits.max_input_payload_bytes)
                    .map_err(|_| InputContractError::ResourceLimit)?
        {
            return Err(InputContractError::ResourceLimit);
        }
        Ok(())
    }

    pub fn payload_hash(&self) -> Result<ContentHash, CanonicalError> {
        let schema = self.payload_schema_id.as_str().as_bytes();
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.input-payload.v1\0");
        preimage.extend_from_slice(
            &u32::try_from(schema.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(schema);
        preimage.extend_from_slice(&self.payload_schema_version.to_le_bytes());
        preimage.extend_from_slice(
            &u64::try_from(self.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&self.payload);
        Ok(content_hash_from_bytes(sha256(&preimage)))
    }

    pub fn sort_key(&self) -> Result<(String, InputSourceId, u64, ContentHash), CanonicalError> {
        Ok((
            self.source_class.as_str().to_owned(),
            self.source_id,
            self.source_sequence,
            self.payload_hash()?,
        ))
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INPUT_SAMPLE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.source_class.as_str().as_bytes().to_vec(),
                ),
                field_id(3, self.source_id.as_bytes()),
                field_u64(4, self.source_sequence),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.payload_schema_id.as_str().as_bytes().to_vec(),
                ),
                field_u32(6, self.payload_schema_version),
                CanonicalField::new(7, CANONICAL_TYPE_BYTES, self.payload.clone()),
                CanonicalField::new(8, CANONICAL_TYPE_OPTIONAL, vec![0]),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INPUT_SAMPLE_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_UTF8_NFC),
                (6, CANONICAL_TYPE_U32),
                (7, CANONICAL_TYPE_BYTES),
                (8, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        if field(&segment, 8)? != [0] {
            return Err(InputContractError::InvalidOptional);
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            source_class: SchemaId::new(read_utf8(&segment, 2)?)?,
            source_id: InputSourceId::from_bytes(exact(field(&segment, 3)?)?),
            source_sequence: read_u64(&segment, 4)?,
            payload_schema_id: SchemaId::new(read_utf8(&segment, 5)?)?,
            payload_schema_version: read_u32(&segment, 6)?,
            payload: field(&segment, 7)?.to_vec(),
            sampled_wall_time: None,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}
