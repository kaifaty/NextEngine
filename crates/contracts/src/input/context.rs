use crate::canonical::*;
use crate::ids::*;

use super::action_map::{
    ActionMapManifestV1, decode_id_sequence, encode_id_sequence, validate_id, validate_strict_ids,
};
use super::codec::*;
use super::constants::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum InputContextCapturePolicyV1 {
    CaptureAll = 1,
    CaptureMatched = 2,
    Passthrough = 3,
}

impl InputContextCapturePolicyV1 {
    fn from_tag(tag: u8) -> Result<Self, InputContractError> {
        match tag {
            1 => Ok(Self::CaptureAll),
            2 => Ok(Self::CaptureMatched),
            3 => Ok(Self::Passthrough),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputContextV1 {
    pub schema_version: u16,
    pub context_id: SchemaId,
    pub revision: u64,
    pub priority: u32,
    pub capture_policy: InputContextCapturePolicyV1,
    pub allowed_action_ids: Vec<SchemaId>,
}

impl InputContextV1 {
    pub fn new(
        context_id: SchemaId,
        revision: u64,
        priority: u32,
        capture_policy: InputContextCapturePolicyV1,
        mut allowed_action_ids: Vec<SchemaId>,
    ) -> Result<Self, InputContractError> {
        allowed_action_ids.sort();
        let value = Self {
            schema_version: INPUT_CONTEXT_SCHEMA_VERSION,
            context_id,
            revision,
            priority,
            capture_policy,
            allowed_action_ids,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != INPUT_CONTEXT_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "input context",
                version: u32::from(self.schema_version),
            });
        }
        validate_id(&self.context_id)?;
        if self.allowed_action_ids.len() > MAX_INPUT_CONTEXT_ACTIONS {
            return Err(InputContractError::ResourceLimit);
        }
        if self.revision == 0 {
            return Err(InputContractError::InvalidProfile);
        }
        validate_strict_ids(&self.allowed_action_ids)
    }

    #[must_use]
    pub fn allows_action(&self, action_id: &SchemaId) -> bool {
        self.allowed_action_ids.binary_search(action_id).is_ok()
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u16(1, self.schema_version),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_UTF8_NFC,
                self.context_id.as_str().as_bytes().to_vec(),
            ),
            field_u64(3, self.revision),
            field_u32(4, self.priority),
            field_u8(5, self.capture_policy as u8),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_SEQUENCE,
                encode_id_sequence(&self.allowed_action_ids)?,
            ),
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
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_U8),
                (6, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: u16::from_le_bytes(exact(&field_from(&fields, 1)?.payload)?),
            context_id: SchemaId::new(read_utf8_field(&fields, 2)?)?,
            revision: read_u64_fields(&fields, 3)?,
            priority: read_u32_fields(&fields, 4)?,
            capture_policy: InputContextCapturePolicyV1::from_tag(read_u8_fields(&fields, 5)?)?,
            allowed_action_ids: decode_id_sequence(&field_from(&fields, 6)?.payload, limits)?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputContextStackV1 {
    pub schema_version: u16,
    pub stack_id: SchemaId,
    pub revision: u64,
    pub entries: Vec<InputContextV1>,
    pub content_hash: ContentHash,
}

impl InputContextStackV1 {
    pub fn new(
        stack_id: SchemaId,
        revision: u64,
        mut entries: Vec<InputContextV1>,
    ) -> Result<Self, InputContractError> {
        entries.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.context_id.cmp(&right.context_id))
        });
        let mut value = Self {
            schema_version: INPUT_CONTEXT_STACK_SCHEMA_VERSION,
            stack_id,
            revision,
            entries,
            content_hash: ContentHash::default(),
        };
        value.validate_body()?;
        value.content_hash = value.computed_hash()?;
        Ok(value)
    }

    pub fn gameplay_v1() -> Result<Self, InputContractError> {
        let allowed_action_ids = [
            CORE_CAMERA_ORBIT_ACTION_ID,
            CORE_EQUIP_USE_ACTION_ID,
            CORE_INTERACT_ACTION_ID,
            CORE_MELEE_ACTION_ID,
            CORE_MOVE_ACTION_ID,
            CORE_PICKUP_ACTION_ID,
        ]
        .into_iter()
        .map(SchemaId::new)
        .collect::<Result<Vec<_>, _>>()?;
        Self::new(
            SchemaId::new(CORE_GAMEPLAY_CONTEXT_STACK_ID)?,
            1,
            vec![InputContextV1::new(
                SchemaId::new(CORE_GAMEPLAY_CONTEXT_ID)?,
                1,
                100,
                InputContextCapturePolicyV1::CaptureAll,
                allowed_action_ids,
            )?],
        )
    }

    pub fn validate(&self) -> Result<(), InputContractError> {
        self.validate_body()?;
        if self.computed_hash()? != self.content_hash {
            return Err(InputContractError::HashMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), InputContractError> {
        if self.schema_version != INPUT_CONTEXT_STACK_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "input context stack",
                version: u32::from(self.schema_version),
            });
        }
        validate_id(&self.stack_id)?;
        if self.entries.len() > MAX_INPUT_CONTEXTS {
            return Err(InputContractError::ResourceLimit);
        }
        if self.revision == 0 || self.entries.is_empty() {
            return Err(InputContractError::InvalidProfile);
        }
        for entry in &self.entries {
            entry.validate()?;
        }
        if self.entries.windows(2).any(|pair| {
            pair[0].priority < pair[1].priority
                || (pair[0].priority == pair[1].priority
                    && pair[0].context_id >= pair[1].context_id)
        }) {
            return Err(InputContractError::NonCanonicalOrder);
        }
        if self.entries.iter().enumerate().any(|(index, entry)| {
            self.entries[..index]
                .iter()
                .any(|previous| previous.context_id == entry.context_id)
        }) {
            return Err(InputContractError::DuplicateKey);
        }
        Ok(())
    }

    #[must_use]
    pub fn allows_action(&self, action_id: &SchemaId) -> bool {
        self.action_priority(action_id).is_some()
    }

    #[must_use]
    pub fn action_priority(&self, action_id: &SchemaId) -> Option<u32> {
        for entry in &self.entries {
            if entry.allows_action(action_id) {
                return Some(entry.priority);
            }
            if entry.capture_policy == InputContextCapturePolicyV1::CaptureAll {
                return None;
            }
        }
        None
    }

    pub fn validate_against_action_map(
        &self,
        action_map: &ActionMapManifestV1,
    ) -> Result<(), InputContractError> {
        self.validate()?;
        action_map.validate()?;
        for context in &self.entries {
            for action_id in &context.allowed_action_ids {
                let action = action_map
                    .action(action_id)
                    .ok_or(InputContractError::InvalidProfile)?;
                if action
                    .allowed_context_ids
                    .binary_search(&context.context_id)
                    .is_err()
                {
                    return Err(InputContractError::InvalidProfile);
                }
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            INPUT_OWNER_ID,
            INPUT_CONTEXT_STACK_SCHEMA_ID,
            SEGMENT_V1,
            self.canonical_fields(true)?,
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
            INPUT_CONTEXT_STACK_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let entries = decode_sequence(field(&segment, 4)?, limits)?
            .into_iter()
            .map(|record| InputContextV1::from_record(&record, limits))
            .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            stack_id: SchemaId::new(read_utf8(&segment, 2)?)?,
            revision: read_u64(&segment, 3)?,
            entries,
            content_hash: read_hash(&segment, 5)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    fn computed_hash(&self) -> Result<ContentHash, CanonicalError> {
        let body = encode_canonical_segment(
            INPUT_OWNER_ID,
            INPUT_CONTEXT_STACK_SCHEMA_ID,
            "body-v1",
            self.canonical_fields(false)?,
        )?;
        hash_canonical_profile(&body)
    }

    fn canonical_fields(&self, include_hash: bool) -> Result<Vec<CanonicalField>, CanonicalError> {
        let mut fields = vec![
            field_u16(1, self.schema_version),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_UTF8_NFC,
                self.stack_id.as_str().as_bytes().to_vec(),
            ),
            field_u64(3, self.revision),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.entries
                        .iter()
                        .map(InputContextV1::canonical_record)
                        .collect::<Result<Vec<_>, _>>()?,
                )?,
            ),
        ];
        if include_hash {
            fields.push(field_hash(5, self.content_hash));
        }
        Ok(fields)
    }
}
