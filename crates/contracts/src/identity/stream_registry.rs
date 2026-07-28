use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment,
};
use crate::{CommandStreamId, IssuerPrincipal, WorldNamespaceId};

use super::codec::{
    decode_map, decode_nested_fixed, decode_nested_id, decode_nested_struct, encode_map, field,
    nested_payload, nested_struct, nested_u32, nested_value, principal_from_union_payload,
    principal_union_payload, read_array, read_u16, require_envelope, require_fields,
    require_nested_fields,
};
use super::derivation::derive_command_stream_id;
use super::error::IdentityContractError;

pub const COMMAND_STREAM_REGISTRY_SCHEMA_VERSION: u16 = 1;

const STREAM_REGISTRY_OWNER_ID: &str = "nextengine.runtime";
const STREAM_REGISTRY_SCHEMA_ID: &str = "nextengine.command-stream-registry";
const STREAM_REGISTRY_SEGMENT_ID: &str = "v1";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandStreamKeyV1 {
    pub principal: IssuerPrincipal,
    pub stream_slot: u32,
    pub stream_epoch: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandStreamRegistryV1 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub entries: BTreeMap<CommandStreamKeyV1, CommandStreamId>,
    pub next_stream_slot: BTreeMap<IssuerPrincipal, u32>,
}

impl CommandStreamRegistryV1 {
    #[must_use]
    pub const fn empty(world_namespace: WorldNamespaceId) -> Self {
        Self {
            schema_version: COMMAND_STREAM_REGISTRY_SCHEMA_VERSION,
            world_namespace,
            entries: BTreeMap::new(),
            next_stream_slot: BTreeMap::new(),
        }
    }

    pub fn allocate_stream(
        &mut self,
        principal: IssuerPrincipal,
    ) -> Result<CommandStreamId, IdentityContractError> {
        let slot = self.next_stream_slot.get(&principal).copied().unwrap_or(0);
        let next = slot
            .checked_add(1)
            .ok_or(IdentityContractError::StreamSlotExhausted)?;
        let key = CommandStreamKeyV1 {
            principal: principal.clone(),
            stream_slot: slot,
            stream_epoch: 0,
        };
        let stream_id = derive_command_stream_id(
            self.world_namespace,
            &principal,
            key.stream_slot,
            key.stream_epoch,
        )?;
        if self.entries.insert(key, stream_id).is_some() {
            return Err(IdentityContractError::DuplicateKey);
        }
        self.next_stream_slot.insert(principal, next);
        Ok(stream_id)
    }

    pub fn binding(
        &self,
        stream_id: CommandStreamId,
    ) -> Option<(&CommandStreamKeyV1, CommandStreamId)> {
        self.entries
            .iter()
            .find(|(_, candidate)| **candidate == stream_id)
            .map(|(key, id)| (key, *id))
    }

    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != COMMAND_STREAM_REGISTRY_SCHEMA_VERSION {
            return Err(IdentityContractError::UnsupportedVersion {
                contract: "command stream registry",
                version: self.schema_version,
            });
        }
        let mut ids = BTreeMap::new();
        for (key, stream_id) in &self.entries {
            if derive_command_stream_id(
                self.world_namespace,
                &key.principal,
                key.stream_slot,
                key.stream_epoch,
            )? != *stream_id
            {
                return Err(IdentityContractError::CommandStreamMismatch);
            }
            if ids.insert(*stream_id, key).is_some() {
                return Err(IdentityContractError::CommandStreamCollision);
            }
            let next = self
                .next_stream_slot
                .get(&key.principal)
                .ok_or(IdentityContractError::NextStreamSlotMismatch)?;
            if *next <= key.stream_slot {
                return Err(IdentityContractError::NextStreamSlotMismatch);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let entries = self
            .entries
            .iter()
            .map(|(key, stream_id)| {
                Ok((
                    nested_struct([
                        CanonicalField::new(
                            1,
                            CANONICAL_TYPE_TAGGED_UNION,
                            principal_union_payload(&key.principal)?,
                        ),
                        CanonicalField::new(
                            2,
                            CANONICAL_TYPE_U32,
                            key.stream_slot.to_le_bytes().to_vec(),
                        ),
                        CanonicalField::new(
                            3,
                            CANONICAL_TYPE_U32,
                            key.stream_epoch.to_le_bytes().to_vec(),
                        ),
                    ])?,
                    nested_value(CANONICAL_TYPE_ID128, stream_id.as_bytes())?,
                ))
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        let next_slots = self
            .next_stream_slot
            .iter()
            .map(|(principal, next)| {
                Ok((
                    principal.canonical_bytes()?,
                    nested_value(CANONICAL_TYPE_U32, &next.to_le_bytes())?,
                ))
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        encode_canonical_segment(
            STREAM_REGISTRY_OWNER_ID,
            STREAM_REGISTRY_SCHEMA_ID,
            STREAM_REGISTRY_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_ID128,
                    self.world_namespace.as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_map(entries)?),
                CanonicalField::new(4, CANONICAL_TYPE_MAP, encode_map(next_slots)?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, IdentityContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment,
            STREAM_REGISTRY_OWNER_ID,
            STREAM_REGISTRY_SCHEMA_ID,
            STREAM_REGISTRY_SEGMENT_ID,
        )?;
        require_fields(
            &segment,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_MAP),
                (4, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut entries = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 3)?.payload.as_slice(), limits)? {
            let key_fields = decode_nested_struct(&key, limits)?;
            require_nested_fields(
                &key_fields,
                &[
                    (1, CANONICAL_TYPE_TAGGED_UNION),
                    (2, CANONICAL_TYPE_U32),
                    (3, CANONICAL_TYPE_U32),
                ],
            )?;
            let stream_key = CommandStreamKeyV1 {
                principal: principal_from_union_payload(nested_payload(&key_fields, 1)?, limits)?,
                stream_slot: nested_u32(&key_fields, 2)?,
                stream_epoch: nested_u32(&key_fields, 3)?,
            };
            let stream_id =
                CommandStreamId::from_bytes(decode_nested_id(&value, CANONICAL_TYPE_ID128)?);
            if entries.insert(stream_key, stream_id).is_some() {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let mut next_stream_slot = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 4)?.payload.as_slice(), limits)? {
            let principal = IssuerPrincipal::from_canonical_bytes(&key, limits)?;
            let next = u32::from_le_bytes(decode_nested_fixed(&value, CANONICAL_TYPE_U32)?);
            if next_stream_slot.insert(principal, next).is_some() {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let registry = Self {
            schema_version: read_u16(&segment, 1)?,
            world_namespace: WorldNamespaceId::from_bytes(read_array(&segment, 2)?),
            entries,
            next_stream_slot,
        };
        registry.validate()?;
        if registry.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(registry)
    }
}
