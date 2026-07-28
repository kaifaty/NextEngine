use std::collections::{BTreeMap, BTreeSet};

use crate::canonical::*;
use crate::command::IssuerPrincipal;
use crate::ids::*;

use super::codec::*;
use super::constants::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlayerControllerBindingV1 {
    pub principal: IssuerPrincipal,
    pub source_id: InputSourceId,
    pub controller_id: PersistentId,
    pub controlled_body_id: PersistentId,
    pub command_stream_id: CommandStreamId,
    pub action_map_hash: ContentHash,
    pub action_map_revision: u64,
    pub context_stack_hash: ContentHash,
    pub context_stack_revision: u64,
}

impl PlayerControllerBindingV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_BYTES, self.principal.canonical_bytes()?),
            field_id(2, self.source_id.as_bytes()),
            field_id(3, self.controller_id.as_bytes()),
            field_id(4, self.controlled_body_id.as_bytes()),
            field_id(5, self.command_stream_id.as_bytes()),
            field_hash(6, self.action_map_hash),
            field_u64(7, self.action_map_revision),
            field_hash(8, self.context_stack_hash),
            field_u64(9, self.context_stack_revision),
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
                (1, CANONICAL_TYPE_BYTES),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_ID128),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_HASH256),
                (9, CANONICAL_TYPE_U64),
            ],
        )?;
        Ok(Self {
            principal: IssuerPrincipal::from_canonical_bytes(
                &field_from(&fields, 1)?.payload,
                limits,
            )?,
            source_id: InputSourceId::from_bytes(exact(&field_from(&fields, 2)?.payload)?),
            controller_id: PersistentId::from_bytes(exact(&field_from(&fields, 3)?.payload)?),
            controlled_body_id: PersistentId::from_bytes(exact(&field_from(&fields, 4)?.payload)?),
            command_stream_id: CommandStreamId::from_bytes(exact(
                &field_from(&fields, 5)?.payload,
            )?),
            action_map_hash: content_hash_from_bytes(exact(&field_from(&fields, 6)?.payload)?),
            action_map_revision: read_u64_fields(&fields, 7)?,
            context_stack_hash: content_hash_from_bytes(exact(&field_from(&fields, 8)?.payload)?),
            context_stack_revision: read_u64_fields(&fields, 9)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerControllerRegistryV1 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub bindings: BTreeMap<InputSourceId, PlayerControllerBindingV1>,
}

impl PlayerControllerRegistryV1 {
    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "player controller registry",
                version: u32::from(self.schema_version),
            });
        }
        let mut controllers = BTreeSet::new();
        let mut bodies = BTreeSet::new();
        let mut streams = BTreeSet::new();
        for (source, binding) in &self.bindings {
            if source != &binding.source_id
                || !controllers.insert(binding.controller_id)
                || !bodies.insert(binding.controlled_body_id)
                || !streams.insert(binding.command_stream_id)
                || !matches!(binding.principal, IssuerPrincipal::Player(_))
            {
                return Err(InputContractError::RegistryCollision);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let entries = self
            .bindings
            .iter()
            .map(|(source, binding)| {
                encode_struct([
                    field_id(1, source.as_bytes()),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, binding.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            PLAYER_CONTROLLER_REGISTRY_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.world_namespace.as_bytes()),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_sequence(entries)?),
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
            RUNTIME_OWNER_ID,
            PLAYER_CONTROLLER_REGISTRY_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut bindings = BTreeMap::new();
        for bytes in decode_sequence(field(&segment, 3)?, limits)? {
            let fields = decode_struct(&bytes, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let source = InputSourceId::from_bytes(exact(&field_from(&fields, 1)?.payload)?);
            let binding =
                PlayerControllerBindingV1::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if bindings.insert(source, binding).is_some() {
                return Err(InputContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            world_namespace: WorldNamespaceId::from_bytes(exact(field(&segment, 2)?)?),
            bindings,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum InputMappingCodeV1 {
    Accepted = 0,
    PrincipalUnbound = 1,
    PayloadSchemaUnsupported = 2,
    ActionMapStale = 3,
    ContextStackStale = 4,
    ActionUnmapped = 5,
    ValueOutOfProfile = 6,
    FrameInvalid = 7,
}

impl InputMappingCodeV1 {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::Accepted => "INPUT_ACCEPTED",
            Self::PrincipalUnbound => "INPUT_PRINCIPAL_UNBOUND",
            Self::PayloadSchemaUnsupported => "INPUT_SCHEMA_UNSUPPORTED",
            Self::ActionMapStale => "INPUT_ACTION_MAP_STALE",
            Self::ContextStackStale => "INPUT_CONTEXT_STALE",
            Self::ActionUnmapped => "INPUT_ACTION_UNMAPPED",
            Self::ValueOutOfProfile => "INPUT_VALUE_OUT_OF_PROFILE",
            Self::FrameInvalid => "INPUT_FRAME_INVALID",
        }
    }

    fn from_tag(tag: u8) -> Result<Self, InputContractError> {
        match tag {
            0 => Ok(Self::Accepted),
            1 => Ok(Self::PrincipalUnbound),
            2 => Ok(Self::PayloadSchemaUnsupported),
            3 => Ok(Self::ActionMapStale),
            4 => Ok(Self::ContextStackStale),
            5 => Ok(Self::ActionUnmapped),
            6 => Ok(Self::ValueOutOfProfile),
            7 => Ok(Self::FrameInvalid),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputMappingReceiptV1 {
    pub assigned_tick: u64,
    pub source_id: InputSourceId,
    pub source_sequence: u64,
    pub payload_hash: ContentHash,
    pub code: InputMappingCodeV1,
    pub derived_command_id: Option<CommandId>,
}

impl InputMappingReceiptV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INPUT_MAPPING_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u64(1, self.assigned_tick),
                field_id(2, self.source_id.as_bytes()),
                field_u64(3, self.source_sequence),
                field_hash(4, self.payload_hash),
                field_u8(5, self.code as u8),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_optional_id(self.derived_command_id.as_ref())?,
                ),
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
            RUNTIME_OWNER_ID,
            INPUT_MAPPING_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U64),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_U8),
                (6, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        let value = Self {
            assigned_tick: read_u64(&segment, 1)?,
            source_id: InputSourceId::from_bytes(exact(field(&segment, 2)?)?),
            source_sequence: read_u64(&segment, 3)?,
            payload_hash: read_hash(&segment, 4)?,
            code: InputMappingCodeV1::from_tag(read_u8(&segment, 5)?)?,
            derived_command_id: decode_optional_id(field(&segment, 6)?, limits)?,
        };
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}
