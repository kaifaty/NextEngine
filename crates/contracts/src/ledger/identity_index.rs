use super::hashes::*;
use super::*;
use sha2::{Digest, Sha256};

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

#[derive(Clone)]
pub struct CommandIdentityIndexBodyV1 {
    pub schema_version: u16,
    pub bindings: Arc<BTreeMap<CommandId, CommandIdentityBindingV1>>,
    pub command_id_count: u64,
    pub occurrence_count: u64,
    /// Derived count-prefixed canonical encoding of the bindings map,
    /// rebuilt lazily after every mutation. Identity-index roots and ledger
    /// encodes stream from this buffer instead of re-walking every binding;
    /// it never changes the logical value, so it is excluded from
    /// `PartialEq`/`Debug`.
    bindings_concat: std::sync::OnceLock<Arc<[u8]>>,
}

impl PartialEq for CommandIdentityIndexBodyV1 {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.bindings == other.bindings
            && self.command_id_count == other.command_id_count
            && self.occurrence_count == other.occurrence_count
    }
}

impl Eq for CommandIdentityIndexBodyV1 {}

impl std::fmt::Debug for CommandIdentityIndexBodyV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CommandIdentityIndexBodyV1")
            .field("schema_version", &self.schema_version)
            .field("bindings", &self.bindings)
            .field("command_id_count", &self.command_id_count)
            .field("occurrence_count", &self.occurrence_count)
            .finish_non_exhaustive()
    }
}

impl CommandIdentityIndexBodyV1 {
    /// Canonical count-prefixed bindings encoding shared by the index root,
    /// the body segment and the ledger encode. The buffer is built once per
    /// generation and reused; concurrent builders produce identical bytes.
    pub(crate) fn bindings_concat(&self) -> Result<&[u8], CanonicalError> {
        if let Some(bytes) = self.bindings_concat.get() {
            return Ok(bytes);
        }
        let bytes: Arc<[u8]> = encode_identity_bindings(&self.bindings)?.into();
        let _ = self.bindings_concat.set(bytes);
        Ok(self
            .bindings_concat
            .get()
            .expect("bindings concat initialized"))
    }

    /// Resets the derived encoding after a bindings mutation. Callers must
    /// invoke this on the mutated value, never on the pre-mutation clone.
    fn invalidate_bindings_concat(&mut self) {
        self.bindings_concat = std::sync::OnceLock::new();
    }

    pub(crate) fn from_parts(
        schema_version: u16,
        bindings: BTreeMap<CommandId, CommandIdentityBindingV1>,
        command_id_count: u64,
        occurrence_count: u64,
    ) -> Self {
        Self {
            schema_version,
            bindings: Arc::new(bindings),
            command_id_count,
            occurrence_count,
            bindings_concat: std::sync::OnceLock::new(),
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let bindings = self.bindings_concat()?.to_vec();
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

    pub(super) fn canonical_layout(
        &self,
    ) -> Result<CommandIdentityIndexCanonicalLayout, CanonicalError> {
        let bindings_bytes = self.bindings_concat()?.len();
        let total_bytes = crate::canonical::CANONICAL_BINARY_V1_MAGIC
            .len()
            .checked_add(std::mem::size_of::<u16>())
            .and_then(|length| {
                length.checked_add(
                    std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_OWNER_ID.len(),
                )
            })
            .and_then(|length| {
                length.checked_add(
                    std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID.len(),
                )
            })
            .and_then(|length| {
                length.checked_add(
                    std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID.len(),
                )
            })
            .and_then(|length| length.checked_add(std::mem::size_of::<u32>()))
            .and_then(|length| {
                length.checked_add(canonical_field_bytes(std::mem::size_of::<u16>()).ok()?)
            })
            .and_then(|length| length.checked_add(canonical_field_bytes(bindings_bytes).ok()?))
            .and_then(|length| {
                length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
            })
            .and_then(|length| {
                length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
            })
            .ok_or(CanonicalError::LengthOverflow)?;
        Ok(CommandIdentityIndexCanonicalLayout {
            bindings_bytes,
            total_bytes,
        })
    }

    pub(super) fn visit_canonical_bytes(
        &self,
        layout: CommandIdentityIndexCanonicalLayout,
        write: &mut impl FnMut(&[u8]),
    ) -> Result<(), CanonicalError> {
        write(&crate::canonical::CANONICAL_BINARY_V1_MAGIC);
        write(&crate::canonical::CANONICAL_BINARY_V1_VERSION.to_le_bytes());
        visit_u32_length_prefixed(write, COMMAND_IDENTITY_INDEX_BODY_OWNER_ID.as_bytes())?;
        visit_u32_length_prefixed(write, COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID.as_bytes())?;
        visit_u32_length_prefixed(write, COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID.as_bytes())?;
        write(&4_u32.to_le_bytes());
        visit_field(
            write,
            1,
            CANONICAL_TYPE_U16,
            &self.schema_version.to_le_bytes(),
        )?;
        visit_field_header(write, 2, CANONICAL_TYPE_MAP, layout.bindings_bytes)?;
        write(self.bindings_concat()?);
        visit_field(
            write,
            3,
            CANONICAL_TYPE_U64,
            &self.command_id_count.to_le_bytes(),
        )?;
        visit_field(
            write,
            4,
            CANONICAL_TYPE_U64,
            &self.occurrence_count.to_le_bytes(),
        )
    }

    #[cfg(test)]
    pub(crate) fn canonical_bytes_reference(&self) -> Result<Vec<u8>, CanonicalError> {
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

#[derive(Clone, Copy)]
pub(super) struct CommandIdentityIndexCanonicalLayout {
    bindings_bytes: usize,
    pub(super) total_bytes: usize,
}

fn encode_identity_bindings(
    bindings: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<Vec<u8>, CanonicalError> {
    let expected_bytes = identity_bindings_byte_len(bindings)?;
    let mut bytes = Vec::with_capacity(expected_bytes);
    visit_identity_bindings(&mut |chunk| bytes.extend_from_slice(chunk), bindings)?;
    debug_assert_eq!(bytes.len(), expected_bytes);
    Ok(bytes)
}

fn identity_bindings_byte_len(
    bindings: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<usize, CanonicalError> {
    u32::try_from(bindings.len()).map_err(|_| CanonicalError::LengthOverflow)?;
    bindings
        .values()
        .try_fold(std::mem::size_of::<u32>(), |length, binding| {
            let binding_bytes = identity_binding_byte_len(binding)?;
            length
                .checked_add(CANONICAL_NESTED_HEADER_BYTES + std::mem::size_of::<u128>())
                .and_then(|length| length.checked_add(binding_bytes))
                .ok_or(CanonicalError::LengthOverflow)
        })
}

fn visit_identity_bindings(
    write: &mut impl FnMut(&[u8]),
    bindings: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<(), CanonicalError> {
    write(
        &u32::try_from(bindings.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for (command_id, binding) in bindings {
        visit_nested_value(write, CANONICAL_TYPE_ID128, command_id.as_bytes())?;
        visit_identity_binding(write, binding)?;
    }
    Ok(())
}

fn identity_binding_byte_len(binding: &CommandIdentityBindingV1) -> Result<usize, CanonicalError> {
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
    CANONICAL_NESTED_HEADER_BYTES
        .checked_add(payload_bytes)
        .ok_or(CanonicalError::LengthOverflow)
}

fn visit_identity_binding(
    write: &mut impl FnMut(&[u8]),
    binding: &CommandIdentityBindingV1,
) -> Result<(), CanonicalError> {
    let binding_bytes = identity_binding_byte_len(binding)?;
    let payload_bytes = binding_bytes
        .checked_sub(CANONICAL_NESTED_HEADER_BYTES)
        .ok_or(CanonicalError::LengthOverflow)?;
    let occurrences_bytes = std::mem::size_of::<u32>()
        .checked_add(
            binding
                .occurrences
                .len()
                .checked_mul(IDENTITY_OCCURRENCE_RECORD_BYTES)
                .ok_or(CanonicalError::LengthOverflow)?,
        )
        .ok_or(CanonicalError::LengthOverflow)?;
    visit_nested_header(write, CANONICAL_TYPE_STRUCT, payload_bytes)?;
    write(&3_u32.to_le_bytes());
    visit_field(
        write,
        1,
        CANONICAL_TYPE_ID128,
        binding.command_id.as_bytes(),
    )?;
    visit_field_header(write, 2, CANONICAL_TYPE_SEQUENCE, occurrences_bytes)?;
    write(
        &u32::try_from(binding.occurrences.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for occurrence in &binding.occurrences {
        visit_identity_occurrence(write, occurrence)?;
    }
    visit_field(write, 3, CANONICAL_TYPE_U8, &[binding.state as u8])
}

fn visit_identity_occurrence(
    write: &mut impl FnMut(&[u8]),
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
    visit_nested_header(write, CANONICAL_TYPE_STRUCT, payload_bytes)?;
    write(&3_u32.to_le_bytes());
    visit_field(
        write,
        1,
        CANONICAL_TYPE_HASH256,
        occurrence.body_hash.as_bytes(),
    )?;
    visit_field(
        write,
        2,
        CANONICAL_TYPE_ID128,
        occurrence.first_stream_id.as_bytes(),
    )?;
    visit_field(
        write,
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

fn visit_u32_length_prefixed(
    write: &mut impl FnMut(&[u8]),
    payload: &[u8],
) -> Result<(), CanonicalError> {
    write(
        &u32::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    write(payload);
    Ok(())
}

fn visit_nested_value(
    write: &mut impl FnMut(&[u8]),
    type_tag: u8,
    payload: &[u8],
) -> Result<(), CanonicalError> {
    visit_nested_header(write, type_tag, payload.len())?;
    write(payload);
    Ok(())
}

fn visit_nested_header(
    write: &mut impl FnMut(&[u8]),
    type_tag: u8,
    payload_bytes: usize,
) -> Result<(), CanonicalError> {
    write(&[type_tag]);
    write(
        &u64::try_from(payload_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

fn visit_field(
    write: &mut impl FnMut(&[u8]),
    field_id: u32,
    type_tag: u8,
    payload: &[u8],
) -> Result<(), CanonicalError> {
    visit_field_header(write, field_id, type_tag, payload.len())?;
    write(payload);
    Ok(())
}

fn visit_field_header(
    write: &mut impl FnMut(&[u8]),
    field_id: u32,
    type_tag: u8,
    payload_bytes: usize,
) -> Result<(), CanonicalError> {
    write(&field_id.to_le_bytes());
    write(&[type_tag]);
    write(
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandIdentityIndexV1 {
    pub schema_version: u16,
    pub body: CommandIdentityIndexBodyV1,
    pub index_root: ContentHash,
}

/// Opaque, fully checked replacement set for the mutable identity index.
/// Only bindings touched by the prepared transaction are retained here.
#[derive(Clone, Debug)]
pub struct PreparedCommandIdentityIndexUpdate {
    schema_version: u16,
    base_bindings: Arc<BTreeMap<CommandId, CommandIdentityBindingV1>>,
    replacements: BTreeMap<CommandId, CommandIdentityBindingV1>,
    command_id_count: u64,
    occurrence_count: u64,
    index_root: std::sync::OnceLock<ContentHash>,
}

impl PreparedCommandIdentityIndexUpdate {
    #[must_use]
    pub fn index_root(&self) -> ContentHash {
        *self.index_root.get_or_init(|| {
            command_identity_index_root_with_replacements(
                self.schema_version,
                &self.base_bindings,
                &self.replacements,
                self.command_id_count,
                self.occurrence_count,
            )
            .expect("validated identity-index update has a representable root")
        })
    }

    #[must_use]
    pub const fn command_id_count(&self) -> u64 {
        self.command_id_count
    }

    #[must_use]
    pub const fn occurrence_count(&self) -> u64 {
        self.occurrence_count
    }
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
        let body = CommandIdentityIndexBodyV1::from_parts(
            COMMAND_IDENTITY_INDEX_SCHEMA_VERSION,
            bindings,
            command_id_count,
            occurrence_count,
        );
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
        // The clone may carry the pre-mutation derived encoding; only the
        // mutated value may rebuild it.
        next.body.invalidate_bindings_concat();
        next.index_root = command_identity_index_root(&next.body)?;
        *self = next;
        Ok(result)
    }

    /// Validates the complete touched-binding set without cloning the
    /// retained historical map. The exact public root remains lazy until a
    /// snapshot or durable checkpoint needs it.
    pub fn prepare_replacements(
        &self,
        replacements: BTreeMap<CommandId, CommandIdentityBindingV1>,
    ) -> Result<PreparedCommandIdentityIndexUpdate, CommandLedgerError> {
        if self.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION
            || self.body.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION
        {
            return Err(CommandLedgerError::UnsupportedIdentityIndexVersion(
                self.schema_version,
            ));
        }
        let mut command_id_count = self.body.command_id_count;
        let mut occurrence_count = self.body.occurrence_count;
        for (command_id, replacement) in &replacements {
            validate_identity_binding(command_id, replacement)?;
            match self.body.bindings.get(command_id) {
                Some(previous) => {
                    occurrence_count = occurrence_count
                        .checked_sub(
                            u64::try_from(previous.occurrences.len())
                                .map_err(|_| CommandLedgerError::CountOverflow)?,
                        )
                        .and_then(|count| {
                            count.checked_add(u64::try_from(replacement.occurrences.len()).ok()?)
                        })
                        .ok_or(CommandLedgerError::CountOverflow)?;
                }
                None => {
                    command_id_count = command_id_count
                        .checked_add(1)
                        .ok_or(CommandLedgerError::CountOverflow)?;
                    occurrence_count = occurrence_count
                        .checked_add(
                            u64::try_from(replacement.occurrences.len())
                                .map_err(|_| CommandLedgerError::CountOverflow)?,
                        )
                        .ok_or(CommandLedgerError::CountOverflow)?;
                }
            }
        }
        Ok(PreparedCommandIdentityIndexUpdate {
            schema_version: self.body.schema_version,
            base_bindings: self.body.bindings.clone(),
            replacements,
            command_id_count,
            occurrence_count,
            index_root: std::sync::OnceLock::new(),
        })
    }

    /// Applies an update after its enclosing runtime generation was validated.
    pub fn commit_prepared_replacements(&mut self, update: PreparedCommandIdentityIndexUpdate) {
        let index_root = update.index_root();
        self.commit_prepared_replacements_deferred(update);
        self.index_root = index_root;
    }

    /// Applies only the authoritative body of a validated update. Callers
    /// must keep the derived public root private until they materialize it.
    pub fn commit_prepared_replacements_deferred(
        &mut self,
        update: PreparedCommandIdentityIndexUpdate,
    ) {
        let PreparedCommandIdentityIndexUpdate {
            schema_version: _,
            base_bindings,
            replacements,
            command_id_count,
            occurrence_count,
            index_root: _,
        } = update;
        assert!(
            Arc::ptr_eq(&self.body.bindings, &base_bindings),
            "prepared identity-index update belongs to a different base generation"
        );
        drop(base_bindings);
        let bindings = Arc::make_mut(&mut self.body.bindings);
        for (command_id, binding) in replacements {
            bindings.insert(command_id, binding);
        }
        self.body.command_id_count = command_id_count;
        self.body.occurrence_count = occurrence_count;
        self.body.invalidate_bindings_concat();
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

fn validate_identity_binding(
    command_id: &CommandId,
    binding: &CommandIdentityBindingV1,
) -> Result<(), CommandLedgerError> {
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
    Ok(())
}

fn command_identity_index_root_with_replacements(
    schema_version: u16,
    base: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    replacements: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    command_id_count: u64,
    occurrence_count: u64,
) -> Result<ContentHash, CanonicalError> {
    let bindings_bytes = merged_identity_bindings_byte_len(base, replacements)?;
    let total_bytes = crate::canonical::CANONICAL_BINARY_V1_MAGIC
        .len()
        .checked_add(std::mem::size_of::<u16>())
        .and_then(|length| {
            length.checked_add(
                std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_OWNER_ID.len(),
            )
        })
        .and_then(|length| {
            length.checked_add(
                std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID.len(),
            )
        })
        .and_then(|length| {
            length.checked_add(
                std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID.len(),
            )
        })
        .and_then(|length| length.checked_add(std::mem::size_of::<u32>()))
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u16>()).ok()?)
        })
        .and_then(|length| length.checked_add(canonical_field_bytes(bindings_bytes).ok()?))
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
        })
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
        })
        .ok_or(CanonicalError::LengthOverflow)?;
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.command-identity-index.v1\0");
    hasher.update(
        u64::try_from(total_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    let mut write = |chunk: &[u8]| hasher.update(chunk);
    write(&crate::canonical::CANONICAL_BINARY_V1_MAGIC);
    write(&crate::canonical::CANONICAL_BINARY_V1_VERSION.to_le_bytes());
    visit_u32_length_prefixed(&mut write, COMMAND_IDENTITY_INDEX_BODY_OWNER_ID.as_bytes())?;
    visit_u32_length_prefixed(&mut write, COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID.as_bytes())?;
    visit_u32_length_prefixed(
        &mut write,
        COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID.as_bytes(),
    )?;
    write(&4_u32.to_le_bytes());
    visit_field(
        &mut write,
        1,
        CANONICAL_TYPE_U16,
        &schema_version.to_le_bytes(),
    )?;
    visit_field_header(&mut write, 2, CANONICAL_TYPE_MAP, bindings_bytes)?;
    write(
        &u32::try_from(command_id_count)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for_each_merged_identity_binding(base, replacements, |command_id, binding| {
        visit_nested_value(&mut write, CANONICAL_TYPE_ID128, command_id.as_bytes())?;
        visit_identity_binding(&mut write, binding)
    })?;
    visit_field(
        &mut write,
        3,
        CANONICAL_TYPE_U64,
        &command_id_count.to_le_bytes(),
    )?;
    visit_field(
        &mut write,
        4,
        CANONICAL_TYPE_U64,
        &occurrence_count.to_le_bytes(),
    )?;
    Ok(content_hash_from_bytes(hasher.finalize().into()))
}

fn merged_identity_bindings_byte_len(
    base: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    replacements: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<usize, CanonicalError> {
    let count = base
        .len()
        .checked_add(
            replacements
                .keys()
                .filter(|command_id| !base.contains_key(command_id))
                .count(),
        )
        .ok_or(CanonicalError::LengthOverflow)?;
    u32::try_from(count).map_err(|_| CanonicalError::LengthOverflow)?;
    let mut length = std::mem::size_of::<u32>();
    for_each_merged_identity_binding(base, replacements, |_, binding| {
        length = length
            .checked_add(CANONICAL_NESTED_HEADER_BYTES + std::mem::size_of::<u128>())
            .and_then(|length| length.checked_add(identity_binding_byte_len(binding).ok()?))
            .ok_or(CanonicalError::LengthOverflow)?;
        Ok(())
    })?;
    Ok(length)
}

fn for_each_merged_identity_binding(
    base: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    replacements: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    mut visit: impl FnMut(&CommandId, &CommandIdentityBindingV1) -> Result<(), CanonicalError>,
) -> Result<(), CanonicalError> {
    let mut base = base.iter().peekable();
    let mut replacements = replacements.iter().peekable();
    loop {
        match (base.peek(), replacements.peek()) {
            (Some((base_id, base_binding)), Some((replacement_id, replacement_binding))) => {
                match base_id.cmp(replacement_id) {
                    std::cmp::Ordering::Less => {
                        visit(base_id, base_binding)?;
                        base.next();
                    }
                    std::cmp::Ordering::Equal => {
                        visit(replacement_id, replacement_binding)?;
                        base.next();
                        replacements.next();
                    }
                    std::cmp::Ordering::Greater => {
                        visit(replacement_id, replacement_binding)?;
                        replacements.next();
                    }
                }
            }
            (Some((command_id, binding)), None) => {
                visit(command_id, binding)?;
                base.next();
            }
            (None, Some((command_id, binding))) => {
                visit(command_id, binding)?;
                replacements.next();
            }
            (None, None) => return Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityInsertResult {
    Inserted,
    Existing,
    Collision,
}
