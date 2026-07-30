use std::collections::{BTreeMap, BTreeSet};

use crate::canonical::*;
use crate::command::IssuerPrincipal;
use crate::ids::*;

use super::action_map::ActionMapManifestV1;
use super::codec::*;
use super::constants::*;
use super::context::InputContextStackV1;

#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub action_map: ActionMapManifestV1,
    pub context_stack: InputContextStackV1,
}

impl PlayerControllerBindingV1 {
    pub fn validate(&self) -> Result<(), InputContractError> {
        self.action_map.validate()?;
        self.context_stack
            .validate_against_action_map(&self.action_map)?;
        if self.action_map_hash != self.action_map.content_hash
            || self.action_map_revision != self.action_map.revision
            || self.context_stack_hash != self.context_stack.content_hash
            || self.context_stack_revision != self.context_stack.revision
        {
            return Err(InputContractError::InvalidProfile);
        }
        Ok(())
    }

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
            CanonicalField::new(10, CANONICAL_TYPE_BYTES, self.action_map.canonical_bytes()?),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_BYTES,
                self.context_stack.canonical_bytes()?,
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
                (1, CANONICAL_TYPE_BYTES),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_ID128),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_HASH256),
                (9, CANONICAL_TYPE_U64),
                (10, CANONICAL_TYPE_BYTES),
                (11, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let value = Self {
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
            action_map: ActionMapManifestV1::from_canonical_bytes(
                &field_from(&fields, 10)?.payload,
                limits,
            )?,
            context_stack: InputContextStackV1::from_canonical_bytes(
                &field_from(&fields, 11)?.payload,
                limits,
            )?,
        };
        value.validate()?;
        Ok(value)
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
            binding.validate()?;
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

    pub fn activate_input_configuration(
        &mut self,
        source_id: InputSourceId,
        action_map: ActionMapManifestV1,
        context_stack: InputContextStackV1,
    ) -> Result<(), InputContractError> {
        action_map.validate()?;
        context_stack.validate_against_action_map(&action_map)?;
        let binding = self
            .bindings
            .get_mut(&source_id)
            .ok_or(InputContractError::RegistryCollision)?;
        if action_map.action_map_id != binding.action_map.action_map_id
            || context_stack.stack_id != binding.context_stack.stack_id
            || !action_shapes_match(&action_map, &binding.action_map)
        {
            return Err(InputContractError::RegistryCollision);
        }
        validate_revision_registration(
            binding.action_map_revision,
            binding.action_map_hash,
            action_map.revision,
            action_map.content_hash,
        )?;
        validate_revision_registration(
            binding.context_stack_revision,
            binding.context_stack_hash,
            context_stack.revision,
            context_stack.content_hash,
        )?;

        binding.action_map_hash = action_map.content_hash;
        binding.action_map_revision = action_map.revision;
        binding.context_stack_hash = context_stack.content_hash;
        binding.context_stack_revision = context_stack.revision;
        binding.action_map = action_map;
        binding.context_stack = context_stack;
        binding.validate()
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

fn validate_revision_registration(
    current_revision: u64,
    current_hash: ContentHash,
    candidate_revision: u64,
    candidate_hash: ContentHash,
) -> Result<(), InputContractError> {
    if candidate_revision < current_revision {
        return Err(InputContractError::InvalidProfile);
    }
    if candidate_revision == current_revision && candidate_hash != current_hash {
        return Err(InputContractError::RegistryCollision);
    }
    Ok(())
}

fn action_shapes_match(left: &ActionMapManifestV1, right: &ActionMapManifestV1) -> bool {
    left.actions.len() == right.actions.len()
        && left
            .actions
            .iter()
            .zip(&right.actions)
            .all(|(left, right)| {
                left.action_id == right.action_id && left.value_kind == right.value_kind
            })
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputActionMappingResultV2 {
    pub frame_action_ordinal: u32,
    pub semantic_occurrence_ordinal: u32,
    pub action_id: SchemaId,
    pub mapping_code: InputMappingCodeV1,
    pub first_command_ordinal: u32,
    pub command_count: u32,
}

impl InputActionMappingResultV2 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u32(1, self.frame_action_ordinal),
            field_u32(2, self.semantic_occurrence_ordinal),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_UTF8_NFC,
                self.action_id.as_str().as_bytes().to_vec(),
            ),
            field_u8(4, self.mapping_code as u8),
            field_u32(5, self.first_command_ordinal),
            field_u32(6, self.command_count),
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
                (1, CANONICAL_TYPE_U32),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_UTF8_NFC),
                (4, CANONICAL_TYPE_U8),
                (5, CANONICAL_TYPE_U32),
                (6, CANONICAL_TYPE_U32),
            ],
        )?;
        Ok(Self {
            frame_action_ordinal: read_u32_fields(&fields, 1)?,
            semantic_occurrence_ordinal: read_u32_fields(&fields, 2)?,
            action_id: SchemaId::new(read_utf8_field(&fields, 3)?)?,
            mapping_code: InputMappingCodeV1::from_tag(read_u8_fields(&fields, 4)?)?,
            first_command_ordinal: read_u32_fields(&fields, 5)?,
            command_count: read_u32_fields(&fields, 6)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputDerivedCommandRefV2 {
    pub command_ordinal: u32,
    pub source_action_ordinal: u32,
    pub mapper_command_slot: u32,
    pub command_id: CommandId,
}

impl InputDerivedCommandRefV2 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u32(1, self.command_ordinal),
            field_u32(2, self.source_action_ordinal),
            field_u32(3, self.mapper_command_slot),
            field_id(4, self.command_id.as_bytes()),
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
                (1, CANONICAL_TYPE_U32),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_ID128),
            ],
        )?;
        Ok(Self {
            command_ordinal: read_u32_fields(&fields, 1)?,
            source_action_ordinal: read_u32_fields(&fields, 2)?,
            mapper_command_slot: read_u32_fields(&fields, 3)?,
            command_id: CommandId::from_bytes(exact(&field_from(&fields, 4)?.payload)?),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputMappingReceiptV2 {
    pub schema_version: u16,
    pub assigned_tick: u64,
    pub source_id: InputSourceId,
    pub source_sequence: u64,
    pub payload_hash: ContentHash,
    pub frame_code: InputMappingCodeV1,
    pub action_results: Vec<InputActionMappingResultV2>,
    pub derived_commands: Vec<InputDerivedCommandRefV2>,
}

impl InputMappingReceiptV2 {
    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != INPUT_MAPPING_RECEIPT_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "input mapping receipt",
                version: u32::from(self.schema_version),
            });
        }
        if self.action_results.len() > MAX_PLAYER_ACTIONS_PER_FRAME
            || self.derived_commands.len() > MAX_DERIVED_COMMANDS_PER_INPUT_FRAME
        {
            return Err(InputContractError::ResourceLimit);
        }
        if self.frame_code != InputMappingCodeV1::Accepted && !self.derived_commands.is_empty() {
            return Err(InputContractError::InvalidValue);
        }

        let action_count = u32::try_from(self.action_results.len())
            .map_err(|_| InputContractError::ResourceLimit)?;
        let command_count = u32::try_from(self.derived_commands.len())
            .map_err(|_| InputContractError::ResourceLimit)?;
        let mut semantic_ordinals = BTreeSet::new();
        let mut expected_first_command = 0_u32;
        for (index, result) in self.action_results.iter().enumerate() {
            if usize::try_from(result.frame_action_ordinal).ok() != Some(index)
                || !semantic_ordinals.insert(result.semantic_occurrence_ordinal)
            {
                return Err(InputContractError::NonCanonicalOrder);
            }
            if result.mapping_code != InputMappingCodeV1::Accepted && result.command_count != 0 {
                return Err(InputContractError::InvalidValue);
            }
            if result.first_command_ordinal != expected_first_command {
                return Err(InputContractError::NonCanonicalOrder);
            }
            expected_first_command = expected_first_command
                .checked_add(result.command_count)
                .ok_or(InputContractError::ResourceLimit)?;
            if expected_first_command > command_count {
                return Err(InputContractError::InvalidValue);
            }
        }
        if semantic_ordinals
            .iter()
            .enumerate()
            .any(|(index, ordinal)| usize::try_from(*ordinal).ok() != Some(index))
            || expected_first_command != command_count
        {
            return Err(InputContractError::NonCanonicalOrder);
        }

        let mut previous_key = None;
        let mut command_ids = BTreeSet::new();
        for (index, command) in self.derived_commands.iter().enumerate() {
            if usize::try_from(command.command_ordinal).ok() != Some(index) {
                return Err(InputContractError::NonCanonicalOrder);
            }
            if command.source_action_ordinal >= action_count
                || !command_ids.insert(command.command_id)
            {
                return Err(InputContractError::InvalidValue);
            }
            let key = (
                command.source_action_ordinal,
                command.mapper_command_slot,
                *command.command_id.as_bytes(),
            );
            if previous_key
                .as_ref()
                .is_some_and(|previous| previous >= &key)
            {
                return Err(InputContractError::NonCanonicalOrder);
            }
            previous_key = Some(key);
        }

        for result in &self.action_results {
            let start = usize::try_from(result.first_command_ordinal)
                .map_err(|_| InputContractError::ResourceLimit)?;
            let end = usize::try_from(
                result
                    .first_command_ordinal
                    .checked_add(result.command_count)
                    .ok_or(InputContractError::ResourceLimit)?,
            )
            .map_err(|_| InputContractError::ResourceLimit)?;
            for (slot, command) in self.derived_commands[start..end].iter().enumerate() {
                if command.source_action_ordinal != result.frame_action_ordinal
                    || usize::try_from(command.mapper_command_slot).ok() != Some(slot)
                {
                    return Err(InputContractError::NonCanonicalOrder);
                }
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let action_results = self
            .action_results
            .iter()
            .map(InputActionMappingResultV2::canonical_record)
            .collect::<Result<Vec<_>, _>>()?;
        let derived_commands = self
            .derived_commands
            .iter()
            .map(InputDerivedCommandRefV2::canonical_record)
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INPUT_MAPPING_RECEIPT_SCHEMA_ID,
            SEGMENT_V2,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.assigned_tick),
                field_id(3, self.source_id.as_bytes()),
                field_u64(4, self.source_sequence),
                field_hash(5, self.payload_hash),
                field_u8(6, self.frame_code as u8),
                CanonicalField::new(7, CANONICAL_TYPE_SEQUENCE, encode_sequence(action_results)?),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(derived_commands)?,
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
            SEGMENT_V2,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_HASH256),
                (6, CANONICAL_TYPE_U8),
                (7, CANONICAL_TYPE_SEQUENCE),
                (8, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            assigned_tick: read_u64(&segment, 2)?,
            source_id: InputSourceId::from_bytes(exact(field(&segment, 3)?)?),
            source_sequence: read_u64(&segment, 4)?,
            payload_hash: read_hash(&segment, 5)?,
            frame_code: InputMappingCodeV1::from_tag(read_u8(&segment, 6)?)?,
            action_results: decode_sequence(field(&segment, 7)?, limits)?
                .into_iter()
                .map(|bytes| InputActionMappingResultV2::from_record(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
            derived_commands: decode_sequence(field(&segment, 8)?, limits)?
                .into_iter()
                .map(|bytes| InputDerivedCommandRefV2::from_record(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}
