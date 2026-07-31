use super::hashes::*;
use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CommandIdentityBindingState {
    Unique = 0,
    Collision = 1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandIdentityOccurrenceV1 {
    pub body_hash: CommandBodyHash,
    pub first_stream_id: CommandStreamId,
    pub first_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandIdentityBindingV1 {
    pub command_id: CommandId,
    pub occurrences: Vec<CommandIdentityOccurrenceV1>,
    pub state: CommandIdentityBindingState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandIdentityIndexBodyV1 {
    pub schema_version: u16,
    pub bindings: Arc<BTreeMap<CommandId, CommandIdentityBindingV1>>,
    pub command_id_count: u64,
    pub occurrence_count: u64,
}

impl CommandIdentityIndexBodyV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let bindings = encode_identity_bindings(&self.bindings)?;
        encode_canonical_segment(
            COMMAND_IDENTITY_INDEX_BODY_OWNER_ID,
            COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID,
            COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_MAP, bindings),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_U64,
                    self.command_id_count.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_U64,
                    self.occurrence_count.to_le_bytes().to_vec(),
                ),
            ],
        )
    }

    #[cfg(test)]
    fn canonical_bytes_reference(&self) -> Result<Vec<u8>, CanonicalError> {
        let bindings = encode_map(
            self.bindings
                .iter()
                .map(|(command_id, binding)| {
                    Ok((
                        nested_value(CANONICAL_TYPE_ID128, command_id.as_bytes())?,
                        binding.canonical_record()?,
                    ))
                })
                .collect::<Result<Vec<_>, CanonicalError>>()?,
        )?;
        encode_canonical_segment(
            COMMAND_IDENTITY_INDEX_BODY_OWNER_ID,
            COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID,
            COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_MAP, bindings),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_U64,
                    self.command_id_count.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_U64,
                    self.occurrence_count.to_le_bytes().to_vec(),
                ),
            ],
        )
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedIdentityIndexVersion(
                self.schema_version,
            ));
        }
        let command_id_count =
            u64::try_from(self.bindings.len()).map_err(|_| CommandLedgerError::CountOverflow)?;
        if command_id_count != self.command_id_count {
            return Err(CommandLedgerError::IdentityIndexCountMismatch);
        }
        let mut occurrence_count = 0_u64;
        for (command_id, binding) in self.bindings.iter() {
            if command_id != &binding.command_id {
                return Err(CommandLedgerError::IdentityIndexKeyMismatch);
            }
            if binding.occurrences.is_empty()
                || binding
                    .occurrences
                    .windows(2)
                    .any(|pair| pair[0].body_hash >= pair[1].body_hash)
            {
                return Err(CommandLedgerError::IdentityOccurrencesInvalid);
            }
            let expected_state = if binding.occurrences.len() == 1 {
                CommandIdentityBindingState::Unique
            } else {
                CommandIdentityBindingState::Collision
            };
            if binding.state != expected_state {
                return Err(CommandLedgerError::IdentityBindingStateMismatch);
            }
            occurrence_count = occurrence_count
                .checked_add(
                    u64::try_from(binding.occurrences.len())
                        .map_err(|_| CommandLedgerError::CountOverflow)?,
                )
                .ok_or(CommandLedgerError::CountOverflow)?;
        }
        if occurrence_count != self.occurrence_count {
            return Err(CommandLedgerError::IdentityIndexCountMismatch);
        }
        Ok(())
    }
}

fn encode_identity_bindings(
    bindings: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(bindings.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for (command_id, binding) in bindings {
        append_nested_value(&mut bytes, CANONICAL_TYPE_ID128, command_id.as_bytes())?;
        append_identity_binding(&mut bytes, binding)?;
    }
    Ok(bytes)
}

fn append_identity_binding(
    bytes: &mut Vec<u8>,
    binding: &CommandIdentityBindingV1,
) -> Result<(), CanonicalError> {
    let occurrences_bytes = std::mem::size_of::<u32>()
        .checked_add(
            binding
                .occurrences
                .len()
                .checked_mul(IDENTITY_OCCURRENCE_RECORD_BYTES)
                .ok_or(CanonicalError::LengthOverflow)?,
        )
        .ok_or(CanonicalError::LengthOverflow)?;
    let payload_bytes = std::mem::size_of::<u32>()
        .checked_add(canonical_field_bytes(binding.command_id.as_bytes().len())?)
        .and_then(|length| length.checked_add(canonical_field_bytes(occurrences_bytes).ok()?))
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u8>()).ok()?)
        })
        .ok_or(CanonicalError::LengthOverflow)?;
    append_nested_header(bytes, CANONICAL_TYPE_STRUCT, payload_bytes)?;
    bytes.extend_from_slice(&3_u32.to_le_bytes());
    append_field(
        bytes,
        1,
        CANONICAL_TYPE_ID128,
        binding.command_id.as_bytes(),
    )?;
    append_field_header(bytes, 2, CANONICAL_TYPE_SEQUENCE, occurrences_bytes)?;
    bytes.extend_from_slice(
        &u32::try_from(binding.occurrences.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for occurrence in &binding.occurrences {
        append_identity_occurrence(bytes, occurrence)?;
    }
    append_field(bytes, 3, CANONICAL_TYPE_U8, &[binding.state as u8])
}

fn append_identity_occurrence(
    bytes: &mut Vec<u8>,
    occurrence: &CommandIdentityOccurrenceV1,
) -> Result<(), CanonicalError> {
    let payload_bytes = std::mem::size_of::<u32>()
        .checked_add(canonical_field_bytes(
            occurrence.body_hash.as_bytes().len(),
        )?)
        .and_then(|length| {
            length.checked_add(
                canonical_field_bytes(occurrence.first_stream_id.as_bytes().len()).ok()?,
            )
        })
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
        })
        .ok_or(CanonicalError::LengthOverflow)?;
    append_nested_header(bytes, CANONICAL_TYPE_STRUCT, payload_bytes)?;
    bytes.extend_from_slice(&3_u32.to_le_bytes());
    append_field(
        bytes,
        1,
        CANONICAL_TYPE_HASH256,
        occurrence.body_hash.as_bytes(),
    )?;
    append_field(
        bytes,
        2,
        CANONICAL_TYPE_ID128,
        occurrence.first_stream_id.as_bytes(),
    )?;
    append_field(
        bytes,
        3,
        CANONICAL_TYPE_U64,
        &occurrence.first_sequence.to_le_bytes(),
    )
}

const CANONICAL_NESTED_HEADER_BYTES: usize = std::mem::size_of::<u8>() + std::mem::size_of::<u64>();
const CANONICAL_FIELD_HEADER_BYTES: usize =
    std::mem::size_of::<u32>() + std::mem::size_of::<u8>() + std::mem::size_of::<u64>();
const IDENTITY_OCCURRENCE_RECORD_BYTES: usize = CANONICAL_NESTED_HEADER_BYTES
    + std::mem::size_of::<u32>()
    + (3 * CANONICAL_FIELD_HEADER_BYTES)
    + 32
    + 16
    + 8;

fn canonical_field_bytes(payload_bytes: usize) -> Result<usize, CanonicalError> {
    std::mem::size_of::<u32>()
        .checked_add(std::mem::size_of::<u8>())
        .and_then(|length| length.checked_add(std::mem::size_of::<u64>()))
        .and_then(|length| length.checked_add(payload_bytes))
        .ok_or(CanonicalError::LengthOverflow)
}

fn append_nested_value(
    bytes: &mut Vec<u8>,
    type_tag: u8,
    payload: &[u8],
) -> Result<(), CanonicalError> {
    append_nested_header(bytes, type_tag, payload.len())?;
    bytes.extend_from_slice(payload);
    Ok(())
}

fn append_nested_header(
    bytes: &mut Vec<u8>,
    type_tag: u8,
    payload_bytes: usize,
) -> Result<(), CanonicalError> {
    bytes.push(type_tag);
    bytes.extend_from_slice(
        &u64::try_from(payload_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

fn append_field(
    bytes: &mut Vec<u8>,
    field_id: u32,
    type_tag: u8,
    payload: &[u8],
) -> Result<(), CanonicalError> {
    append_field_header(bytes, field_id, type_tag, payload.len())?;
    bytes.extend_from_slice(payload);
    Ok(())
}

fn append_field_header(
    bytes: &mut Vec<u8>,
    field_id: u32,
    type_tag: u8,
    payload_bytes: usize,
) -> Result<(), CanonicalError> {
    bytes.extend_from_slice(&field_id.to_le_bytes());
    bytes.push(type_tag);
    bytes.extend_from_slice(
        &u64::try_from(payload_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

#[cfg(test)]
impl CommandIdentityBindingV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let occurrences = encode_sequence(
            self.occurrences
                .iter()
                .map(CommandIdentityOccurrenceV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        struct_record([
            CanonicalField::new(1, CANONICAL_TYPE_ID128, self.command_id.as_bytes().to_vec()),
            CanonicalField::new(2, CANONICAL_TYPE_SEQUENCE, occurrences),
            CanonicalField::new(3, CANONICAL_TYPE_U8, vec![self.state as u8]),
        ])
    }
}

#[cfg(test)]
impl CommandIdentityOccurrenceV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        struct_record([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_HASH256,
                self.body_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_ID128,
                self.first_stream_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_U64,
                self.first_sequence.to_le_bytes().to_vec(),
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimized_identity_encoding_is_byte_exact_for_unique_and_collision_bindings() {
        let mut index = CommandIdentityIndexV1::empty().expect("empty index");
        let collision_id = CommandId::from_bytes([7; 16]);
        for (body, stream, sequence) in [([1; 32], [2; 16], 3), ([4; 32], [5; 16], 6)] {
            index
                .insert_occurrence(
                    collision_id,
                    CommandIdentityOccurrenceV1 {
                        body_hash: command_body_hash_from_bytes(body),
                        first_stream_id: CommandStreamId::from_bytes(stream),
                        first_sequence: sequence,
                    },
                )
                .expect("collision occurrence");
        }
        index
            .insert_occurrence(
                CommandId::from_bytes([8; 16]),
                CommandIdentityOccurrenceV1 {
                    body_hash: command_body_hash_from_bytes([9; 32]),
                    first_stream_id: CommandStreamId::from_bytes([10; 16]),
                    first_sequence: 11,
                },
            )
            .expect("unique occurrence");

        assert_eq!(
            index.body.canonical_bytes().expect("optimized encoding"),
            index
                .body
                .canonical_bytes_reference()
                .expect("reference encoding")
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandIdentityIndexV1 {
    pub schema_version: u16,
    pub body: CommandIdentityIndexBodyV1,
    pub index_root: ContentHash,
}

impl CommandIdentityIndexV1 {
    pub fn empty() -> Result<Self, CanonicalError> {
        Self::from_bindings(BTreeMap::new())
    }

    pub fn from_bindings(
        bindings: BTreeMap<CommandId, CommandIdentityBindingV1>,
    ) -> Result<Self, CanonicalError> {
        let command_id_count =
            u64::try_from(bindings.len()).map_err(|_| CanonicalError::LengthOverflow)?;
        let occurrence_count = bindings.values().try_fold(0_u64, |count, binding| {
            count
                .checked_add(
                    u64::try_from(binding.occurrences.len())
                        .map_err(|_| CanonicalError::LengthOverflow)?,
                )
                .ok_or(CanonicalError::LengthOverflow)
        })?;
        let body = CommandIdentityIndexBodyV1 {
            schema_version: COMMAND_IDENTITY_INDEX_SCHEMA_VERSION,
            bindings: Arc::new(bindings),
            command_id_count,
            occurrence_count,
        };
        let index_root = command_identity_index_root(&body)?;
        Ok(Self {
            schema_version: COMMAND_IDENTITY_INDEX_SCHEMA_VERSION,
            body,
            index_root,
        })
    }

    pub fn insert_occurrence(
        &mut self,
        command_id: CommandId,
        occurrence: CommandIdentityOccurrenceV1,
    ) -> Result<IdentityInsertResult, CommandLedgerError> {
        self.validate()?;
        self.insert_occurrence_incremental(command_id, occurrence)
    }

    /// Stages one occurrence against an index that was validated at the
    /// transaction boundary. The affected binding and exact public root are
    /// rebuilt, while historical bindings are not decoded or revalidated.
    pub fn insert_occurrence_incremental(
        &mut self,
        command_id: CommandId,
        occurrence: CommandIdentityOccurrenceV1,
    ) -> Result<IdentityInsertResult, CommandLedgerError> {
        if self.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION
            || self.body.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION
            || self.body.command_id_count
                != u64::try_from(self.body.bindings.len())
                    .map_err(|_| CommandLedgerError::CountOverflow)?
        {
            return Err(CommandLedgerError::IdentityIndexCountMismatch);
        }
        let mut next = self.clone();
        let bindings = Arc::make_mut(&mut next.body.bindings);
        let result = match bindings.get_mut(&command_id) {
            None => {
                bindings.insert(
                    command_id,
                    CommandIdentityBindingV1 {
                        command_id,
                        occurrences: vec![occurrence],
                        state: CommandIdentityBindingState::Unique,
                    },
                );
                next.body.command_id_count = next
                    .body
                    .command_id_count
                    .checked_add(1)
                    .ok_or(CommandLedgerError::CountOverflow)?;
                next.body.occurrence_count = next
                    .body
                    .occurrence_count
                    .checked_add(1)
                    .ok_or(CommandLedgerError::CountOverflow)?;
                IdentityInsertResult::Inserted
            }
            Some(binding)
                if binding
                    .occurrences
                    .iter()
                    .any(|existing| existing.body_hash == occurrence.body_hash) =>
            {
                return Ok(IdentityInsertResult::Existing);
            }
            Some(binding) => {
                binding.occurrences.push(occurrence);
                binding
                    .occurrences
                    .sort_by_key(|occurrence| occurrence.body_hash);
                binding.state = CommandIdentityBindingState::Collision;
                next.body.occurrence_count = next
                    .body
                    .occurrence_count
                    .checked_add(1)
                    .ok_or(CommandLedgerError::CountOverflow)?;
                IdentityInsertResult::Collision
            }
        };
        next.index_root = command_identity_index_root(&next.body)?;
        *self = next;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedIdentityIndexVersion(
                self.schema_version,
            ));
        }
        self.body.validate()?;
        if command_identity_index_root(&self.body)? != self.index_root {
            return Err(CommandLedgerError::IdentityIndexRootMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityInsertResult {
    Inserted,
    Existing,
    Collision,
}
