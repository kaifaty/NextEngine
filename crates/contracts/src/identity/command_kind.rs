use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_MAP, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SET,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_UTF8_NFC,
    CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::command::{
    COMMAND_SCHEMA_VERSION, CapabilityRefV1, CommandPayload, CommandPhase,
    NOOP_COMMAND_CAPABILITY_ID, NOOP_COMMAND_SCHEMA_ID,
};
use crate::ids::{CapabilityId, ContentHash, SchemaId, content_hash_from_bytes};
use crate::physics::{
    PHYSICAL_COMMAND_CAPABILITY_ID, PHYSICAL_COMMAND_SCHEMA_ID, PHYSICAL_COMMAND_SCHEMA_VERSION,
};
use crate::rpg::{
    RPG_COMMAND_CAPABILITY_ID, RPG_COMMAND_SCHEMA_ID, RPG_TRANSACTION_COMMAND_SCHEMA_VERSION,
};
use crate::world_population::{
    WORLD_POPULATION_CAPABILITY_ID, WORLD_POPULATION_COMMAND_KIND_ID,
    WORLD_POPULATION_COMMAND_SCHEMA_ID, WORLD_POPULATION_COMMAND_SCHEMA_VERSION,
    WORLD_POPULATION_PRIORITY_CLASS,
};
use crate::world_routine::{
    WORLD_ROUTINE_CAPABILITY_ID, WORLD_ROUTINE_COMMAND_KIND_ID, WORLD_ROUTINE_COMMAND_SCHEMA_ID,
    WORLD_ROUTINE_COMMAND_SCHEMA_VERSION,
};

use super::codec::{
    decode_map, decode_nested_struct, encode_map, field, nested_array, nested_payload,
    nested_struct, nested_text, nested_u32, nested_value, read_u16, require_envelope,
    require_fields, require_nested_fields,
};
use super::error::IdentityContractError;

pub const COMMAND_KIND_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const COMMAND_KIND_REGISTRY_OWNER_ID: &str = "nextengine.runtime";
pub const COMMAND_KIND_REGISTRY_SCHEMA_ID: &str = "nextengine.command-kind-registry";
pub const COMMAND_KIND_REGISTRY_SEGMENT_ID: &str = "v1";
pub const NOOP_COMMAND_KIND_ID: &str = "nextengine.command-kind.noop";
pub const RPG_COMMAND_KIND_ID: &str = "nextengine.command-kind.rpg";
pub const PHYSICAL_COMMAND_KIND_ID: &str = "nextengine.command-kind.physical";
pub const NOOP_PRIORITY_CLASS: u16 = 100;
pub const RPG_PRIORITY_CLASS: u16 = 200;
pub const PHYSICAL_PRIORITY_CLASS: u16 = 300;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandKindRegistryKeyV1 {
    pub payload_schema_id: SchemaId,
    pub payload_schema_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandKindRegistryEntryV1 {
    pub payload_schema_id: SchemaId,
    pub payload_schema_version: u32,
    pub command_kind_id: SchemaId,
    pub priority_class: u16,
    pub validator_profile_hash: ContentHash,
    pub required_capabilities: Vec<CapabilityRefV1>,
}

impl CommandKindRegistryEntryV1 {
    #[must_use]
    pub fn schema_id(&self) -> &SchemaId {
        &self.payload_schema_id
    }

    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.payload_schema_version
    }

    #[must_use]
    pub const fn priority_class(&self) -> u16 {
        self.priority_class
    }

    #[must_use]
    pub fn required_capabilities(&self) -> &[CapabilityRefV1] {
        &self.required_capabilities
    }

    #[must_use]
    pub fn accepts_payload(&self, payload: &CommandPayload) -> bool {
        CommandKindRegistryV1::accepts_payload(self, payload)
    }

    #[must_use]
    pub fn allows_phase(&self, phase: CommandPhase) -> bool {
        CommandKindRegistryV1::allows_phase(self, phase)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandKindRegistryV1 {
    pub schema_version: u16,
    pub entries: BTreeMap<CommandKindRegistryKeyV1, CommandKindRegistryEntryV1>,
}

impl CommandKindRegistryV1 {
    pub fn core_r4b() -> Result<Self, IdentityContractError> {
        let entry = |payload_schema_id: &str,
                     payload_schema_version: u32,
                     command_kind_id: &str,
                     priority_class: u16,
                     validator_domain: &[u8],
                     capability_id: &str|
         -> Result<_, IdentityContractError> {
            let payload_schema_id = SchemaId::new(payload_schema_id)?;
            let value = CommandKindRegistryEntryV1 {
                payload_schema_id: payload_schema_id.clone(),
                payload_schema_version,
                command_kind_id: SchemaId::new(command_kind_id)?,
                priority_class,
                validator_profile_hash: content_hash_from_bytes(sha256(validator_domain)),
                required_capabilities: vec![CapabilityRefV1::unscoped(capability_id)?],
            };
            Ok((
                CommandKindRegistryKeyV1 {
                    payload_schema_id,
                    payload_schema_version,
                },
                value,
            ))
        };
        let value = Self {
            schema_version: COMMAND_KIND_REGISTRY_SCHEMA_VERSION,
            entries: BTreeMap::from([
                entry(
                    NOOP_COMMAND_SCHEMA_ID,
                    COMMAND_SCHEMA_VERSION,
                    NOOP_COMMAND_KIND_ID,
                    NOOP_PRIORITY_CLASS,
                    b"nextengine.command-validator.noop.v1\0",
                    NOOP_COMMAND_CAPABILITY_ID,
                )?,
                entry(
                    RPG_COMMAND_SCHEMA_ID,
                    RPG_TRANSACTION_COMMAND_SCHEMA_VERSION,
                    RPG_COMMAND_KIND_ID,
                    RPG_PRIORITY_CLASS,
                    b"nextengine.command-validator.rpg.v2\0",
                    RPG_COMMAND_CAPABILITY_ID,
                )?,
                entry(
                    PHYSICAL_COMMAND_SCHEMA_ID,
                    PHYSICAL_COMMAND_SCHEMA_VERSION,
                    PHYSICAL_COMMAND_KIND_ID,
                    PHYSICAL_PRIORITY_CLASS,
                    b"nextengine.command-validator.physical.v1\0",
                    PHYSICAL_COMMAND_CAPABILITY_ID,
                )?,
                entry(
                    WORLD_ROUTINE_COMMAND_SCHEMA_ID,
                    WORLD_ROUTINE_COMMAND_SCHEMA_VERSION,
                    WORLD_ROUTINE_COMMAND_KIND_ID,
                    crate::world_routine::WORLD_ROUTINE_PRIORITY_CLASS,
                    b"nextengine.command-validator.world-routine.v1\0",
                    WORLD_ROUTINE_CAPABILITY_ID,
                )?,
                entry(
                    WORLD_POPULATION_COMMAND_SCHEMA_ID,
                    WORLD_POPULATION_COMMAND_SCHEMA_VERSION,
                    WORLD_POPULATION_COMMAND_KIND_ID,
                    WORLD_POPULATION_PRIORITY_CLASS,
                    b"nextengine.command-validator.world-population.v1\0",
                    WORLD_POPULATION_CAPABILITY_ID,
                )?,
            ]),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != COMMAND_KIND_REGISTRY_SCHEMA_VERSION {
            return Err(IdentityContractError::UnsupportedVersion {
                contract: "command kind registry",
                version: self.schema_version,
            });
        }
        if self.entries.is_empty() {
            return Err(IdentityContractError::DuplicateKey);
        }
        for (key, entry) in &self.entries {
            if key.payload_schema_id != entry.payload_schema_id
                || key.payload_schema_version != entry.payload_schema_version
                || entry.priority_class == 0
                || entry.validator_profile_hash == ContentHash::default()
                || entry.required_capabilities.is_empty()
                || entry
                    .required_capabilities
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
            {
                return Err(IdentityContractError::RegistryClosureInvalid);
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn descriptor(
        &self,
        schema_id: &SchemaId,
        schema_version: u32,
    ) -> Option<&CommandKindRegistryEntryV1> {
        self.entries.get(&CommandKindRegistryKeyV1 {
            payload_schema_id: schema_id.clone(),
            payload_schema_version: schema_version,
        })
    }

    #[must_use]
    pub fn accepts_payload(entry: &CommandKindRegistryEntryV1, payload: &CommandPayload) -> bool {
        matches!(
            (entry.payload_schema_id.as_str(), payload),
            (NOOP_COMMAND_SCHEMA_ID, CommandPayload::Noop)
                | (RPG_COMMAND_SCHEMA_ID, CommandPayload::Rpg(_))
                | (PHYSICAL_COMMAND_SCHEMA_ID, CommandPayload::Physical(_))
                | (
                    WORLD_ROUTINE_COMMAND_SCHEMA_ID,
                    CommandPayload::WorldRoutine(_)
                )
                | (
                    WORLD_POPULATION_COMMAND_SCHEMA_ID,
                    CommandPayload::WorldPopulation(_)
                )
        )
    }

    #[must_use]
    pub fn allows_phase(entry: &CommandKindRegistryEntryV1, phase: CommandPhase) -> bool {
        match entry.payload_schema_id.as_str() {
            NOOP_COMMAND_SCHEMA_ID | RPG_COMMAND_SCHEMA_ID => true,
            PHYSICAL_COMMAND_SCHEMA_ID => phase == CommandPhase::Ingress,
            WORLD_ROUTINE_COMMAND_SCHEMA_ID => phase == CommandPhase::Outcome,
            WORLD_POPULATION_COMMAND_SCHEMA_ID => phase == CommandPhase::Outcome,
            _ => false,
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let entries = self
            .entries
            .iter()
            .map(|(key, entry)| {
                let key = nested_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_UTF8_NFC,
                        key.payload_schema_id.as_str().as_bytes().to_vec(),
                    ),
                    CanonicalField::new(
                        2,
                        CANONICAL_TYPE_U32,
                        key.payload_schema_version.to_le_bytes().to_vec(),
                    ),
                ])?;
                let capabilities = encode_set(
                    entry
                        .required_capabilities
                        .iter()
                        .map(|capability| {
                            let scope = match capability.scope_hash {
                                None => vec![0],
                                Some(hash) => {
                                    let mut bytes = vec![1];
                                    bytes.extend_from_slice(&nested_value(
                                        CANONICAL_TYPE_HASH256,
                                        hash.as_bytes(),
                                    )?);
                                    bytes
                                }
                            };
                            nested_struct([
                                CanonicalField::new(
                                    1,
                                    CANONICAL_TYPE_UTF8_NFC,
                                    capability.capability_id.as_str().as_bytes().to_vec(),
                                ),
                                CanonicalField::new(2, CANONICAL_TYPE_OPTIONAL, scope),
                            ])
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )?;
                let value = nested_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_UTF8_NFC,
                        entry.payload_schema_id.as_str().as_bytes().to_vec(),
                    ),
                    CanonicalField::new(
                        2,
                        CANONICAL_TYPE_U32,
                        entry.payload_schema_version.to_le_bytes().to_vec(),
                    ),
                    CanonicalField::new(
                        3,
                        CANONICAL_TYPE_UTF8_NFC,
                        entry.command_kind_id.as_str().as_bytes().to_vec(),
                    ),
                    CanonicalField::new(
                        4,
                        CANONICAL_TYPE_U16,
                        entry.priority_class.to_le_bytes().to_vec(),
                    ),
                    CanonicalField::new(
                        5,
                        CANONICAL_TYPE_HASH256,
                        entry.validator_profile_hash.as_bytes().to_vec(),
                    ),
                    CanonicalField::new(6, CANONICAL_TYPE_SET, capabilities),
                ])?;
                Ok((key, value))
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        encode_canonical_segment(
            COMMAND_KIND_REGISTRY_OWNER_ID,
            COMMAND_KIND_REGISTRY_SCHEMA_ID,
            COMMAND_KIND_REGISTRY_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_MAP, encode_map(entries)?),
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
            COMMAND_KIND_REGISTRY_OWNER_ID,
            COMMAND_KIND_REGISTRY_SCHEMA_ID,
            COMMAND_KIND_REGISTRY_SEGMENT_ID,
        )?;
        require_fields(
            &segment,
            &[(1, CANONICAL_TYPE_U16), (2, CANONICAL_TYPE_MAP)],
        )?;
        let mut entries = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 2)?.payload.as_slice(), limits)? {
            let key_fields = decode_nested_struct(&key, limits)?;
            require_nested_fields(
                &key_fields,
                &[(1, CANONICAL_TYPE_UTF8_NFC), (2, CANONICAL_TYPE_U32)],
            )?;
            let key = CommandKindRegistryKeyV1 {
                payload_schema_id: SchemaId::new(nested_text(&key_fields, 1)?)?,
                payload_schema_version: nested_u32(&key_fields, 2)?,
            };
            let fields = decode_nested_struct(&value, limits)?;
            require_nested_fields(
                &fields,
                &[
                    (1, CANONICAL_TYPE_UTF8_NFC),
                    (2, CANONICAL_TYPE_U32),
                    (3, CANONICAL_TYPE_UTF8_NFC),
                    (4, CANONICAL_TYPE_U16),
                    (5, CANONICAL_TYPE_HASH256),
                    (6, CANONICAL_TYPE_SET),
                ],
            )?;
            let entry = CommandKindRegistryEntryV1 {
                payload_schema_id: SchemaId::new(nested_text(&fields, 1)?)?,
                payload_schema_version: nested_u32(&fields, 2)?,
                command_kind_id: SchemaId::new(nested_text(&fields, 3)?)?,
                priority_class: u16::from_le_bytes(nested_array(&fields, 4)?),
                validator_profile_hash: ContentHash::from_bytes(nested_array(&fields, 5)?),
                required_capabilities: decode_capability_set(nested_payload(&fields, 6)?, limits)?,
            };
            if entries.insert(key, entry).is_some() {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            entries,
        };
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn canonical_hash(&self) -> Result<ContentHash, IdentityContractError> {
        self.validate()?;
        let bytes = self.canonical_bytes()?;
        let mut preimage = b"nextengine.command-kind-registry.v1\0".to_vec();
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&bytes);
        Ok(content_hash_from_bytes(sha256(&preimage)))
    }
}

fn encode_set(mut records: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    records.sort();
    if records.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(records.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for record in records {
        bytes.extend_from_slice(&record);
    }
    Ok(bytes)
}

fn decode_capability_set(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CapabilityRefV1>, IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut result = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let tag = cursor.read_u8()?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
        if tag != CANONICAL_TYPE_STRUCT || length > limits.max_field_payload_bytes {
            return Err(IdentityContractError::RegistryClosureInvalid);
        }
        let encoded = nested_value(tag, cursor.read_exact(length)?)?;
        if previous.as_ref().is_some_and(|prior| prior >= &encoded) {
            return Err(IdentityContractError::DuplicateKey);
        }
        let fields = decode_nested_struct(&encoded, limits)?;
        require_nested_fields(
            &fields,
            &[(1, CANONICAL_TYPE_UTF8_NFC), (2, CANONICAL_TYPE_OPTIONAL)],
        )?;
        let scope = decode_optional_hash(nested_payload(&fields, 2)?)?;
        result.push(CapabilityRefV1 {
            capability_id: CapabilityId::new(nested_text(&fields, 1)?)?,
            scope_hash: scope,
        });
        previous = Some(encoded);
    }
    cursor.finish()?;
    Ok(result)
}

fn decode_optional_hash(bytes: &[u8]) -> Result<Option<ContentHash>, IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let value = match cursor.read_u8()? {
        0 => None,
        1 => {
            let tag = cursor.read_u8()?;
            let length = cursor.read_u64()?;
            if tag != CANONICAL_TYPE_HASH256 || length != 32 {
                return Err(IdentityContractError::RegistryClosureInvalid);
            }
            Some(ContentHash::from_bytes(
                cursor
                    .read_exact(32)?
                    .try_into()
                    .map_err(|_| IdentityContractError::RegistryClosureInvalid)?,
            ))
        }
        _ => return Err(IdentityContractError::RegistryClosureInvalid),
    };
    cursor.finish()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r4b_registry_is_five_entry_canonical_round_trip() {
        let registry = CommandKindRegistryV1::core_r4b().expect("core registry builds");
        assert_eq!(registry.entries.len(), 5);
        let bytes = registry.canonical_bytes().expect("registry encodes");
        assert_eq!(
            CommandKindRegistryV1::from_canonical_bytes(&bytes, Default::default()),
            Ok(registry.clone())
        );
        assert_ne!(
            registry.canonical_hash().expect("registry hashes"),
            ContentHash::default()
        );
    }

    #[test]
    fn routine_entry_is_outcome_only_at_priority_250() {
        let registry = CommandKindRegistryV1::core_r4b().expect("core registry builds");
        let schema = SchemaId::new(WORLD_ROUTINE_COMMAND_SCHEMA_ID).expect("schema is valid");
        let entry = registry
            .descriptor(&schema, WORLD_ROUTINE_COMMAND_SCHEMA_VERSION)
            .expect("routine command is registered");
        assert_eq!(entry.priority_class, 250);
        assert!(!CommandKindRegistryV1::allows_phase(
            entry,
            CommandPhase::Ingress
        ));
        assert!(CommandKindRegistryV1::allows_phase(
            entry,
            CommandPhase::Outcome
        ));
    }

    #[test]
    fn population_entry_is_outcome_only_at_priority_260() {
        let registry = CommandKindRegistryV1::core_r4b().expect("core registry builds");
        let schema = SchemaId::new(WORLD_POPULATION_COMMAND_SCHEMA_ID).expect("schema is valid");
        let entry = registry
            .descriptor(&schema, WORLD_POPULATION_COMMAND_SCHEMA_VERSION)
            .expect("population command is registered");
        assert_eq!(entry.priority_class, WORLD_POPULATION_PRIORITY_CLASS);
        assert!(!CommandKindRegistryV1::allows_phase(
            entry,
            CommandPhase::Ingress
        ));
        assert!(CommandKindRegistryV1::allows_phase(
            entry,
            CommandPhase::Outcome
        ));
    }
}
