use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_OPTIONAL,
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UNIT,
    CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    encode_canonical_segment, sha256,
};
use crate::command::{
    CommandDecodeError, CommandPhase, IssuerPrincipal, WorldCommand,
    compute_command_id_from_body_bytes,
};
use crate::ids::{
    CommandBodyHash, CommandId, CommandStreamId, ContentHash, EventId, SchemaId, WorldNamespaceId,
    command_body_hash_from_bytes, content_hash_from_bytes,
};

pub const COMMAND_RESERVATION_SCHEMA_VERSION: u16 = 1;
pub const COMMAND_RECEIPT_SCHEMA_VERSION: u16 = 1;
pub const COMMAND_STREAM_LEDGER_SCHEMA_VERSION: u16 = 2;
pub const COMMAND_LEDGER_SCHEMA_VERSION: u16 = 2;
pub const COMMAND_BODY_ARCHIVE_SCHEMA_VERSION: u16 = 1;
pub const COMMAND_IDENTITY_INDEX_SCHEMA_VERSION: u16 = 1;
pub const CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const COMMAND_RECEIPT_WINDOW_CAPACITY: usize = 4096;
pub const COMMAND_PENDING_CAPACITY: usize = 256;
pub const COMMAND_RECEIPT_OWNER_ID: &str = "nextengine.runtime";
pub const COMMAND_RECEIPT_SCHEMA_ID: &str = "nextengine.command-receipt";
pub const COMMAND_RECEIPT_SEGMENT_ID: &str = "v1";
pub const COMMAND_IDENTITY_INDEX_BODY_OWNER_ID: &str = "nextengine.runtime";
pub const COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID: &str = "nextengine.command-identity-index-body";
pub const COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID: &str = "v1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CommandStreamStateV1 {
    Open = 0,
    CollisionLocked = 1,
    Exhausted = 2,
    Closed = 3,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandReservationV1 {
    pub schema_version: u16,
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub sequence: u64,
    pub command_id: CommandId,
    pub body_hash: CommandBodyHash,
    pub canonical_body_ref: CommandBodyHash,
    pub reserved_at_tick: u64,
    pub target_tick: u64,
    pub phase: CommandPhase,
    pub priority_class: u16,
    pub command_kind_registry_hash: ContentHash,
}

impl CommandReservationV1 {
    pub fn from_command(
        command: &WorldCommand,
        reserved_at_tick: u64,
        priority_class: u16,
        command_kind_registry_hash: ContentHash,
    ) -> Result<Self, CanonicalError> {
        let body_hash = command.body_hash()?;
        Ok(Self {
            schema_version: COMMAND_RESERVATION_SCHEMA_VERSION,
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            sequence: command.sequence,
            command_id: command.compute_command_id()?,
            body_hash,
            canonical_body_ref: body_hash,
            reserved_at_tick,
            target_tick: command.target_tick,
            phase: command.phase,
            priority_class,
            command_kind_registry_hash,
        })
    }
}

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
    pub bindings: BTreeMap<CommandId, CommandIdentityBindingV1>,
    pub command_id_count: u64,
    pub occurrence_count: u64,
}

impl CommandIdentityIndexBodyV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
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
        for (command_id, binding) in &self.bindings {
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
            bindings,
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
        let mut next = self.clone();
        let result = match next.body.bindings.get_mut(&command_id) {
            None => {
                next.body.bindings.insert(
                    command_id,
                    CommandIdentityBindingV1 {
                        command_id,
                        occurrences: vec![occurrence],
                        state: CommandIdentityBindingState::Unique,
                    },
                );
                IdentityInsertResult::Inserted
            }
            Some(binding)
                if binding
                    .occurrences
                    .iter()
                    .any(|existing| existing.body_hash == occurrence.body_hash) =>
            {
                IdentityInsertResult::Existing
            }
            Some(binding) => {
                binding.occurrences.push(occurrence);
                binding
                    .occurrences
                    .sort_by_key(|occurrence| occurrence.body_hash);
                binding.state = CommandIdentityBindingState::Collision;
                IdentityInsertResult::Collision
            }
        };
        next.recompute_metadata()?;
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

    fn recompute_metadata(&mut self) -> Result<(), CommandLedgerError> {
        self.body.command_id_count = u64::try_from(self.body.bindings.len())
            .map_err(|_| CommandLedgerError::CountOverflow)?;
        self.body.occurrence_count =
            self.body
                .bindings
                .values()
                .try_fold(0_u64, |count, binding| {
                    count
                        .checked_add(
                            u64::try_from(binding.occurrences.len())
                                .map_err(|_| CommandLedgerError::CountOverflow)?,
                        )
                        .ok_or(CommandLedgerError::CountOverflow)
                })?;
        self.index_root = command_identity_index_root(&self.body)?;
        self.validate()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityInsertResult {
    Inserted,
    Existing,
    Collision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandBodyArchiveManifestV1 {
    pub schema_version: u16,
    pub entry_count: u64,
    pub archive_root: ContentHash,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandBodyArchiveV1 {
    entries: BTreeMap<CommandBodyHash, Vec<u8>>,
}

impl CommandBodyArchiveV1 {
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<CommandBodyHash, Vec<u8>> {
        &self.entries
    }

    pub fn insert_body_bytes(
        &mut self,
        body_bytes: Vec<u8>,
    ) -> Result<ArchiveInsertResult, CommandLedgerError> {
        let command =
            WorldCommand::from_canonical_bytes(&body_bytes, CanonicalDecodeLimits::default())?;
        if command.canonical_bytes()? != body_bytes {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        let body_hash = command_body_hash_from_bytes(sha256(&body_bytes));
        match self.entries.get(&body_hash) {
            Some(existing) if existing == &body_bytes => {
                Ok(ArchiveInsertResult::Existing(body_hash))
            }
            Some(_) => Err(CommandLedgerError::CommandBodyHashCollision),
            None => {
                self.entries.insert(body_hash, body_bytes);
                Ok(ArchiveInsertResult::Inserted(body_hash))
            }
        }
    }

    pub fn insert_command(
        &mut self,
        command: &WorldCommand,
    ) -> Result<ArchiveInsertResult, CommandLedgerError> {
        self.insert_body_bytes(command.canonical_bytes()?)
    }

    pub fn manifest(&self) -> Result<CommandBodyArchiveManifestV1, CommandLedgerError> {
        let entry_count =
            u64::try_from(self.entries.len()).map_err(|_| CommandLedgerError::CountOverflow)?;
        Ok(CommandBodyArchiveManifestV1 {
            schema_version: COMMAND_BODY_ARCHIVE_SCHEMA_VERSION,
            entry_count,
            archive_root: command_body_archive_root(&self.entries)?,
        })
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        for (declared_hash, body_bytes) in &self.entries {
            let command =
                WorldCommand::from_canonical_bytes(body_bytes, CanonicalDecodeLimits::default())?;
            if command.canonical_bytes()? != *body_bytes {
                return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
            }
            let computed_hash = command_body_hash_from_bytes(sha256(body_bytes));
            if declared_hash != &computed_hash {
                return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
            }
        }
        let _ = self.manifest()?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInsertResult {
    Inserted(CommandBodyHash),
    Existing(CommandBodyHash),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandCollisionCandidateV1 {
    pub command_id: CommandId,
    pub body_hash: CommandBodyHash,
    pub canonical_body_ref: CommandBodyHash,
}

impl CommandCollisionCandidateV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        struct_record([
            CanonicalField::new(1, CANONICAL_TYPE_ID128, self.command_id.as_bytes().to_vec()),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_HASH256,
                self.body_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_HASH256,
                self.canonical_body_ref.as_bytes().to_vec(),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandCollisionIncidentV1 {
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub sequence: u64,
    pub candidates: Vec<CommandCollisionCandidateV1>,
    pub candidates_root: ContentHash,
    pub incident_digest: ContentHash,
}

impl CommandCollisionIncidentV1 {
    pub fn new(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        mut candidates: Vec<CommandCollisionCandidateV1>,
    ) -> Result<Self, CommandLedgerError> {
        candidates.sort_by(|left, right| {
            (&left.body_hash, &left.command_id).cmp(&(&right.body_hash, &right.command_id))
        });
        if !collision_candidates_are_canonical(&candidates) {
            return Err(CommandLedgerError::CollisionCandidatesInvalid);
        }
        let candidates_root = command_collision_candidates_root(&candidates)?;
        let incident_digest =
            command_collision_incident_digest(stream_id, sequence, candidates_root);
        Ok(Self {
            stream_id,
            issuer,
            sequence,
            candidates,
            candidates_root,
            incident_digest,
        })
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        if !collision_candidates_are_canonical(&self.candidates) {
            return Err(CommandLedgerError::CollisionCandidatesInvalid);
        }
        if command_collision_candidates_root(&self.candidates)? != self.candidates_root {
            return Err(CommandLedgerError::CollisionCandidatesRootMismatch);
        }
        if command_collision_incident_digest(self.stream_id, self.sequence, self.candidates_root)
            != self.incident_digest
        {
            return Err(CommandLedgerError::CollisionIncidentDigestMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandReceiptSubjectV1 {
    Command {
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        command_id: CommandId,
        body_hash: CommandBodyHash,
        canonical_body_ref: CommandBodyHash,
    },
    CollisionSet {
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        candidates_root: ContentHash,
        candidate_count: u32,
        candidates: Vec<CommandCollisionCandidateV1>,
    },
}

impl CommandReceiptSubjectV1 {
    #[must_use]
    pub const fn stream_id(&self) -> CommandStreamId {
        match self {
            Self::Command { stream_id, .. } | Self::CollisionSet { stream_id, .. } => *stream_id,
        }
    }

    #[must_use]
    pub fn issuer(&self) -> &IssuerPrincipal {
        match self {
            Self::Command { issuer, .. } | Self::CollisionSet { issuer, .. } => issuer,
        }
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        match self {
            Self::Command { sequence, .. } | Self::CollisionSet { sequence, .. } => *sequence,
        }
    }

    fn canonical_payload(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, record) = match self {
            Self::Command {
                stream_id,
                issuer,
                sequence,
                command_id,
                body_hash,
                canonical_body_ref,
            } => (
                0x01,
                struct_record([
                    CanonicalField::new(1, CANONICAL_TYPE_ID128, stream_id.as_bytes().to_vec()),
                    CanonicalField::new(2, CANONICAL_TYPE_TAGGED_UNION, principal_payload(issuer)?),
                    CanonicalField::new(3, CANONICAL_TYPE_U64, sequence.to_le_bytes().to_vec()),
                    CanonicalField::new(4, CANONICAL_TYPE_ID128, command_id.as_bytes().to_vec()),
                    CanonicalField::new(5, CANONICAL_TYPE_HASH256, body_hash.as_bytes().to_vec()),
                    CanonicalField::new(
                        6,
                        CANONICAL_TYPE_HASH256,
                        canonical_body_ref.as_bytes().to_vec(),
                    ),
                ])?,
            ),
            Self::CollisionSet {
                stream_id,
                issuer,
                sequence,
                candidates_root,
                candidate_count,
                candidates,
            } => {
                let candidates = encode_sequence(
                    candidates
                        .iter()
                        .map(CommandCollisionCandidateV1::canonical_record)
                        .collect::<Result<Vec<_>, _>>()?,
                )?;
                (
                    0x02,
                    struct_record([
                        CanonicalField::new(1, CANONICAL_TYPE_ID128, stream_id.as_bytes().to_vec()),
                        CanonicalField::new(
                            2,
                            CANONICAL_TYPE_TAGGED_UNION,
                            principal_payload(issuer)?,
                        ),
                        CanonicalField::new(3, CANONICAL_TYPE_U64, sequence.to_le_bytes().to_vec()),
                        CanonicalField::new(
                            4,
                            CANONICAL_TYPE_HASH256,
                            candidates_root.as_bytes().to_vec(),
                        ),
                        CanonicalField::new(
                            5,
                            CANONICAL_TYPE_U32,
                            candidate_count.to_le_bytes().to_vec(),
                        ),
                        CanonicalField::new(6, CANONICAL_TYPE_SEQUENCE, candidates),
                    ])?,
                )
            }
        };
        let mut payload = vec![tag];
        payload.extend_from_slice(&record);
        Ok(payload)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandFinalResultV1 {
    Committed,
    Rejected { code: SchemaId },
    Collision { code: SchemaId },
}

impl CommandFinalResultV1 {
    fn canonical_payload(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, record) = match self {
            Self::Committed => (0x01, nested_value(CANONICAL_TYPE_UNIT, &[])?),
            Self::Rejected { code } => (
                0x02,
                struct_record([CanonicalField::new(
                    1,
                    CANONICAL_TYPE_UTF8_NFC,
                    code.as_str().as_bytes().to_vec(),
                )])?,
            ),
            Self::Collision { code } => (
                0x03,
                struct_record([CanonicalField::new(
                    1,
                    CANONICAL_TYPE_UTF8_NFC,
                    code.as_str().as_bytes().to_vec(),
                )])?,
            ),
        };
        let mut payload = vec![tag];
        payload.extend_from_slice(&record);
        Ok(payload)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandReceiptV1 {
    pub schema_version: u16,
    pub finalization_ordinal: u64,
    pub subject: CommandReceiptSubjectV1,
    pub phase: CommandPhase,
    pub target_tick: u64,
    pub finalized_at_tick: u64,
    pub priority_class: u16,
    pub command_kind_registry_hash: ContentHash,
    pub result: CommandFinalResultV1,
    pub diagnostic_digest: Option<ContentHash>,
    pub event_ids: Vec<EventId>,
    pub transaction_result_root: ContentHash,
}

impl CommandReceiptV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let diagnostic = option_hash(self.diagnostic_digest.as_ref())?;
        let events = encode_sequence(
            self.event_ids
                .iter()
                .map(|event_id| nested_value(CANONICAL_TYPE_ID128, event_id.as_bytes()))
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        encode_canonical_segment(
            COMMAND_RECEIPT_OWNER_ID,
            COMMAND_RECEIPT_SCHEMA_ID,
            COMMAND_RECEIPT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_U64,
                    self.finalization_ordinal.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_TAGGED_UNION,
                    self.subject.canonical_payload()?,
                ),
                CanonicalField::new(4, CANONICAL_TYPE_U8, vec![self.phase as u8]),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_U64,
                    self.target_tick.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_U64,
                    self.finalized_at_tick.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_U16,
                    self.priority_class.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_HASH256,
                    self.command_kind_registry_hash.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_TAGGED_UNION,
                    self.result.canonical_payload()?,
                ),
                CanonicalField::new(10, CANONICAL_TYPE_OPTIONAL, diagnostic),
                CanonicalField::new(11, CANONICAL_TYPE_SEQUENCE, events),
                CanonicalField::new(
                    12,
                    CANONICAL_TYPE_HASH256,
                    self.transaction_result_root.as_bytes().to_vec(),
                ),
            ],
        )
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_RECEIPT_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedReceiptVersion(
                self.schema_version,
            ));
        }
        match (&self.subject, &self.result) {
            (
                CommandReceiptSubjectV1::Command {
                    body_hash,
                    canonical_body_ref,
                    ..
                },
                CommandFinalResultV1::Committed | CommandFinalResultV1::Rejected { .. },
            ) if body_hash == canonical_body_ref => {}
            (
                CommandReceiptSubjectV1::CollisionSet {
                    candidates_root,
                    candidate_count,
                    candidates,
                    ..
                },
                CommandFinalResultV1::Collision { .. },
            ) => {
                if usize::try_from(*candidate_count).ok() != Some(candidates.len())
                    || !collision_candidates_are_canonical(candidates)
                    || command_collision_candidates_root(candidates)? != *candidates_root
                {
                    return Err(CommandLedgerError::CollisionCandidatesInvalid);
                }
            }
            _ => return Err(CommandLedgerError::ReceiptResultMismatch),
        }
        let _ = self.canonical_bytes()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandStreamLedgerV2 {
    pub schema_version: u16,
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub stream_slot: u32,
    pub stream_epoch: u32,
    pub state: CommandStreamStateV1,
    pub admission_high_watermark: Option<u64>,
    pub greatest_reserved_target_tick: Option<u64>,
    pub pending: BTreeMap<u64, CommandReservationV1>,
    pub receipt_window: Vec<CommandReceiptV1>,
    pub finalized_receipt_count: u64,
    pub receipt_chain_root: ContentHash,
    pub collision_incident: Option<CommandCollisionIncidentV1>,
}

impl CommandStreamLedgerV2 {
    #[must_use]
    pub fn genesis(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        stream_slot: u32,
        stream_epoch: u32,
    ) -> Self {
        Self {
            schema_version: COMMAND_STREAM_LEDGER_SCHEMA_VERSION,
            stream_id,
            issuer,
            stream_slot,
            stream_epoch,
            state: CommandStreamStateV1::Open,
            admission_high_watermark: None,
            greatest_reserved_target_tick: None,
            pending: BTreeMap::new(),
            receipt_window: Vec::new(),
            finalized_receipt_count: 0,
            receipt_chain_root: command_receipt_chain_genesis(),
            collision_incident: None,
        }
    }

    pub fn reserve(&mut self, reservation: CommandReservationV1) -> Result<(), CommandLedgerError> {
        self.validate()?;
        if reservation.schema_version != COMMAND_RESERVATION_SCHEMA_VERSION
            || reservation.stream_id != self.stream_id
            || reservation.issuer != self.issuer
            || reservation.body_hash != reservation.canonical_body_ref
        {
            return Err(CommandLedgerError::ReservationMismatch);
        }
        if let Some(existing) = self.pending.get(&reservation.sequence) {
            return if existing == &reservation {
                Ok(())
            } else {
                Err(CommandLedgerError::SequenceNotNew)
            };
        }
        if self.finalized_receipt_count == u64::MAX {
            return Err(CommandLedgerError::FinalizationOrdinalExhausted);
        }
        if self.state != CommandStreamStateV1::Open {
            return Err(CommandLedgerError::StreamNotOpen);
        }
        if self
            .admission_high_watermark
            .is_some_and(|high_watermark| reservation.sequence <= high_watermark)
        {
            return Err(CommandLedgerError::SequenceNotNew);
        }
        if self.pending.len() >= COMMAND_PENDING_CAPACITY {
            return Err(CommandLedgerError::PendingLimit);
        }
        if self
            .greatest_reserved_target_tick
            .is_some_and(|greatest| reservation.target_tick < greatest)
        {
            return Err(CommandLedgerError::TargetTickRegression);
        }
        self.admission_high_watermark = Some(reservation.sequence);
        self.greatest_reserved_target_tick = Some(reservation.target_tick);
        if reservation.sequence == u64::MAX {
            self.state = CommandStreamStateV1::Exhausted;
        }
        self.pending.insert(reservation.sequence, reservation);
        self.validate()
    }

    pub fn lock_for_collision(
        &mut self,
        incident: CommandCollisionIncidentV1,
    ) -> Result<(), CommandLedgerError> {
        self.record_collision(incident, None)
    }

    pub fn append_collision_receipt(
        &mut self,
        incident: CommandCollisionIncidentV1,
        receipt: CommandReceiptV1,
    ) -> Result<(), CommandLedgerError> {
        self.record_collision(incident, Some(receipt))
    }

    fn record_collision(
        &mut self,
        incident: CommandCollisionIncidentV1,
        receipt: Option<CommandReceiptV1>,
    ) -> Result<(), CommandLedgerError> {
        self.validate()?;
        incident.validate()?;
        if incident.stream_id != self.stream_id || incident.issuer != self.issuer {
            return Err(CommandLedgerError::CollisionIncidentMismatch);
        }
        if self.state == CommandStreamStateV1::CollisionLocked {
            return if self.collision_incident.as_ref() == Some(&incident) && receipt.is_none() {
                Ok(())
            } else {
                Err(CommandLedgerError::UnexpectedCollisionReceipt)
            };
        }
        if self.finalized_receipt_count == u64::MAX {
            return Err(CommandLedgerError::FinalizationOrdinalExhausted);
        }
        if self.state != CommandStreamStateV1::Open {
            return Err(CommandLedgerError::StreamNotOpen);
        }
        let sequence_is_retained = self.pending.contains_key(&incident.sequence)
            || self
                .receipt_window
                .iter()
                .any(|receipt| receipt.subject.sequence() == incident.sequence);
        match (sequence_is_retained, receipt.as_ref()) {
            (true, Some(_)) => return Err(CommandLedgerError::UnexpectedCollisionReceipt),
            (false, None) => return Err(CommandLedgerError::CollisionReceiptRequired),
            (false, Some(_))
                if self
                    .admission_high_watermark
                    .is_some_and(|high_watermark| incident.sequence <= high_watermark) =>
            {
                return Err(CommandLedgerError::SequenceNotNew);
            }
            _ => {}
        }

        let mut next = self.clone();
        next.admission_high_watermark = Some(
            next.admission_high_watermark
                .map_or(incident.sequence, |high_watermark| {
                    high_watermark.max(incident.sequence)
                }),
        );
        next.state = CommandStreamStateV1::CollisionLocked;
        next.collision_incident = Some(incident);
        if let Some(receipt) = receipt {
            next.append_receipt_inner(receipt, true)?;
        }
        next.validate()?;
        *self = next;
        Ok(())
    }

    pub fn append_receipt(&mut self, receipt: CommandReceiptV1) -> Result<(), CommandLedgerError> {
        self.append_receipt_inner(receipt, false)
    }

    fn append_receipt_inner(
        &mut self,
        receipt: CommandReceiptV1,
        allow_collision: bool,
    ) -> Result<(), CommandLedgerError> {
        self.validate_append_boundary()?;
        receipt.validate()?;
        if self.finalized_receipt_count == u64::MAX {
            return Err(CommandLedgerError::FinalizationOrdinalExhausted);
        }
        if matches!(
            receipt.subject,
            CommandReceiptSubjectV1::CollisionSet { .. }
        ) && !allow_collision
        {
            return Err(CommandLedgerError::UnexpectedCollisionReceipt);
        }
        if receipt.subject.stream_id() != self.stream_id || receipt.subject.issuer() != &self.issuer
        {
            return Err(CommandLedgerError::ReceiptSubjectMismatch);
        }
        if receipt.finalization_ordinal != self.finalized_receipt_count {
            return Err(CommandLedgerError::FinalizationOrdinalMismatch);
        }
        let sequence = receipt.subject.sequence();
        let pending = self.pending.get(&sequence);
        if let Some(reservation) = pending {
            validate_receipt_against_reservation(&receipt, reservation)?;
        }
        let collision_is_recorded =
            receipt_matches_collision_incident(&receipt, self.collision_incident.as_ref());
        if self
            .admission_high_watermark
            .is_some_and(|high_watermark| sequence <= high_watermark)
            && pending.is_none()
            && !collision_is_recorded
        {
            return Err(CommandLedgerError::SequenceNotNew);
        }
        if self.state != CommandStreamStateV1::Open && pending.is_none() && !collision_is_recorded {
            return Err(CommandLedgerError::StreamNotOpen);
        }
        if allow_collision && !collision_is_recorded {
            return Err(CommandLedgerError::CollisionIncidentMismatch);
        }

        let next_high_watermark = Some(
            self.admission_high_watermark
                .map_or(sequence, |high_watermark| high_watermark.max(sequence)),
        );
        let next_chain_root = command_receipt_chain_next(self.receipt_chain_root, &receipt)?;
        let next_count = self
            .finalized_receipt_count
            .checked_add(1)
            .ok_or(CommandLedgerError::FinalizationOrdinalExhausted)?;

        self.admission_high_watermark = next_high_watermark;
        self.pending.remove(&sequence);
        self.receipt_chain_root = next_chain_root;
        self.receipt_window.push(receipt);
        if self.receipt_window.len() > COMMAND_RECEIPT_WINDOW_CAPACITY {
            self.receipt_window.remove(0);
        }
        self.finalized_receipt_count = next_count;
        if self.state == CommandStreamStateV1::Open
            && (sequence == u64::MAX || self.finalized_receipt_count == u64::MAX)
        {
            self.state = CommandStreamStateV1::Exhausted;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_STREAM_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedStreamLedgerVersion(
                self.schema_version,
            ));
        }
        if self.pending.len() > COMMAND_PENDING_CAPACITY {
            return Err(CommandLedgerError::PendingLimit);
        }
        for (sequence, reservation) in &self.pending {
            if sequence != &reservation.sequence
                || reservation.stream_id != self.stream_id
                || reservation.issuer != self.issuer
                || reservation.schema_version != COMMAND_RESERVATION_SCHEMA_VERSION
                || reservation.body_hash != reservation.canonical_body_ref
                || self
                    .admission_high_watermark
                    .is_none_or(|high_watermark| *sequence > high_watermark)
            {
                return Err(CommandLedgerError::ReservationMismatch);
            }
            if self
                .greatest_reserved_target_tick
                .is_none_or(|greatest| reservation.target_tick > greatest)
            {
                return Err(CommandLedgerError::TargetTickRegression);
            }
        }
        let expected_window_len = usize::try_from(
            self.finalized_receipt_count
                .min(COMMAND_RECEIPT_WINDOW_CAPACITY as u64),
        )
        .map_err(|_| CommandLedgerError::CountOverflow)?;
        if self.receipt_window.len() != expected_window_len {
            return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
        }
        let first_ordinal = self
            .finalized_receipt_count
            .checked_sub(
                u64::try_from(self.receipt_window.len())
                    .map_err(|_| CommandLedgerError::CountOverflow)?,
            )
            .ok_or(CommandLedgerError::CountOverflow)?;
        let mut retained_sequences = BTreeSet::new();
        for (index, receipt) in self.receipt_window.iter().enumerate() {
            receipt.validate()?;
            let expected_ordinal = first_ordinal
                .checked_add(u64::try_from(index).map_err(|_| CommandLedgerError::CountOverflow)?)
                .ok_or(CommandLedgerError::CountOverflow)?;
            if receipt.finalization_ordinal != expected_ordinal
                || receipt.subject.stream_id() != self.stream_id
                || receipt.subject.issuer() != &self.issuer
                || self
                    .admission_high_watermark
                    .is_none_or(|high_watermark| receipt.subject.sequence() > high_watermark)
                || !retained_sequences.insert(receipt.subject.sequence())
                || matches!(
                    receipt.subject,
                    CommandReceiptSubjectV1::CollisionSet { .. }
                ) && !receipt_matches_collision_incident(
                    receipt,
                    self.collision_incident.as_ref(),
                )
            {
                return Err(CommandLedgerError::ReceiptWindowInvalid);
            }
        }
        if self.finalized_receipt_count == 0 {
            if self.receipt_chain_root != command_receipt_chain_genesis() {
                return Err(CommandLedgerError::ReceiptChainRootMismatch);
            }
        } else if self.finalized_receipt_count <= COMMAND_RECEIPT_WINDOW_CAPACITY as u64 {
            let mut root = command_receipt_chain_genesis();
            for receipt in &self.receipt_window {
                root = command_receipt_chain_next(root, receipt)?;
            }
            if root != self.receipt_chain_root {
                return Err(CommandLedgerError::ReceiptChainRootMismatch);
            }
        }
        match (&self.state, &self.collision_incident) {
            (CommandStreamStateV1::CollisionLocked, Some(incident)) => {
                incident.validate()?;
                if incident.stream_id != self.stream_id || incident.issuer != self.issuer {
                    return Err(CommandLedgerError::CollisionIncidentMismatch);
                }
            }
            (CommandStreamStateV1::CollisionLocked, None) => {
                return Err(CommandLedgerError::CollisionIncidentMismatch);
            }
            (_, Some(_)) => return Err(CommandLedgerError::CollisionIncidentMismatch),
            _ => {}
        }
        if self.state == CommandStreamStateV1::Open
            && (self.admission_high_watermark == Some(u64::MAX)
                || self.finalized_receipt_count == u64::MAX)
        {
            return Err(CommandLedgerError::OpenStreamExhausted);
        }
        if self.admission_high_watermark.is_none()
            && (!self.pending.is_empty() || !self.receipt_window.is_empty())
        {
            return Err(CommandLedgerError::HighWatermarkMissing);
        }
        Ok(())
    }

    fn validate_append_boundary(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_STREAM_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedStreamLedgerVersion(
                self.schema_version,
            ));
        }
        let expected_window_len = usize::try_from(
            self.finalized_receipt_count
                .min(COMMAND_RECEIPT_WINDOW_CAPACITY as u64),
        )
        .map_err(|_| CommandLedgerError::CountOverflow)?;
        if self.receipt_window.len() != expected_window_len {
            return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
        }
        if self.finalized_receipt_count == 0 {
            if self.receipt_chain_root != command_receipt_chain_genesis() {
                return Err(CommandLedgerError::ReceiptChainRootMismatch);
            }
            return Ok(());
        }
        let first_ordinal = self
            .finalized_receipt_count
            .checked_sub(
                u64::try_from(self.receipt_window.len())
                    .map_err(|_| CommandLedgerError::CountOverflow)?,
            )
            .ok_or(CommandLedgerError::CountOverflow)?;
        let last_ordinal = self
            .finalized_receipt_count
            .checked_sub(1)
            .ok_or(CommandLedgerError::CountOverflow)?;
        if self
            .receipt_window
            .first()
            .is_none_or(|receipt| receipt.finalization_ordinal != first_ordinal)
            || self
                .receipt_window
                .last()
                .is_none_or(|receipt| receipt.finalization_ordinal != last_ordinal)
        {
            return Err(CommandLedgerError::ReceiptWindowInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CausalIdentityKind {
    PlayerPrincipal = 1,
    CommandStream = 2,
    DomainEvent = 3,
    PersistentRecord = 4,
    RngStream = 5,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CausalIdentityKey {
    pub identity_kind: CausalIdentityKind,
    pub identity_bytes: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalIdentityRegistryV1 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub bindings: BTreeMap<CausalIdentityKey, ContentHash>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandLedgerV2 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub streams: BTreeMap<CommandStreamId, CommandStreamLedgerV2>,
    pub command_kind_registry_hash: ContentHash,
    pub runtime_determinism_profile_hash: ContentHash,
    pub body_archive: CommandBodyArchiveManifestV1,
    pub identity_index: CommandIdentityIndexV1,
    pub causal_identity_registry: CausalIdentityRegistryV1,
}

impl CommandLedgerV2 {
    pub fn empty(
        world_namespace: WorldNamespaceId,
        command_kind_registry_hash: ContentHash,
        runtime_determinism_profile_hash: ContentHash,
    ) -> Result<(Self, CommandBodyArchiveV1), CommandLedgerError> {
        let archive = CommandBodyArchiveV1::default();
        let ledger = Self {
            schema_version: COMMAND_LEDGER_SCHEMA_VERSION,
            world_namespace,
            streams: BTreeMap::new(),
            command_kind_registry_hash,
            runtime_determinism_profile_hash,
            body_archive: archive.manifest()?,
            identity_index: CommandIdentityIndexV1::empty()?,
            causal_identity_registry: CausalIdentityRegistryV1 {
                schema_version: CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION,
                world_namespace,
                bindings: BTreeMap::new(),
            },
        };
        ledger.validate(&archive)?;
        Ok((ledger, archive))
    }

    pub fn synchronize_archive(
        &mut self,
        archive: &CommandBodyArchiveV1,
    ) -> Result<(), CommandLedgerError> {
        archive.validate()?;
        let mut next = self.clone();
        next.body_archive = archive.manifest()?;
        next.validate(archive)?;
        *self = next;
        Ok(())
    }

    pub fn validate(&self, archive: &CommandBodyArchiveV1) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedLedgerVersion(
                self.schema_version,
            ));
        }
        if self.causal_identity_registry.schema_version != CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION
            || self.causal_identity_registry.world_namespace != self.world_namespace
        {
            return Err(CommandLedgerError::CausalIdentityRegistryMismatch);
        }
        archive.validate()?;
        if archive.manifest()? != self.body_archive {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        self.identity_index.validate()?;
        let archive_hashes: BTreeSet<_> = archive.entries().keys().copied().collect();
        let occurrence_hashes: BTreeSet<_> = self
            .identity_index
            .body
            .bindings
            .values()
            .flat_map(|binding| {
                binding
                    .occurrences
                    .iter()
                    .map(|occurrence| occurrence.body_hash)
            })
            .collect();
        if archive_hashes != occurrence_hashes {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        for (command_id, binding) in &self.identity_index.body.bindings {
            for occurrence in &binding.occurrences {
                let body_bytes = archive
                    .entries()
                    .get(&occurrence.body_hash)
                    .ok_or(CommandLedgerError::BodyReferenceMissing)?;
                if compute_command_id_from_body_bytes(body_bytes)? != *command_id {
                    return Err(CommandLedgerError::IdentityCommandIdMismatch);
                }
            }
        }
        for (stream_id, stream) in &self.streams {
            if stream_id != &stream.stream_id {
                return Err(CommandLedgerError::StreamKeyMismatch);
            }
            stream.validate()?;
            for reservation in stream.pending.values() {
                validate_body_reference(
                    &archive_hashes,
                    reservation.command_id,
                    reservation.body_hash,
                    &self.identity_index,
                )?;
            }
            for receipt in &stream.receipt_window {
                validate_receipt_references(receipt, &archive_hashes, &self.identity_index)?;
            }
            if let Some(incident) = &stream.collision_incident {
                for candidate in &incident.candidates {
                    validate_body_reference(
                        &archive_hashes,
                        candidate.command_id,
                        candidate.body_hash,
                        &self.identity_index,
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommandLedgerError {
    Canonical(CanonicalError),
    Command(CommandDecodeError),
    CountOverflow,
    UnsupportedLedgerVersion(u16),
    UnsupportedStreamLedgerVersion(u16),
    UnsupportedReceiptVersion(u16),
    UnsupportedIdentityIndexVersion(u16),
    CommandBodyHashCollision,
    CommandBodyArchiveCorrupt,
    IdentityIndexCountMismatch,
    IdentityIndexKeyMismatch,
    IdentityOccurrencesInvalid,
    IdentityBindingStateMismatch,
    IdentityIndexRootMismatch,
    StreamKeyMismatch,
    StreamNotOpen,
    SequenceNotNew,
    TargetTickRegression,
    PendingLimit,
    ReservationMismatch,
    ReceiptSubjectMismatch,
    ReceiptResultMismatch,
    FinalizationOrdinalMismatch,
    FinalizationOrdinalExhausted,
    ReceiptWindowLengthMismatch,
    ReceiptWindowInvalid,
    ReceiptChainRootMismatch,
    CollisionCandidatesInvalid,
    CollisionCandidatesRootMismatch,
    CollisionIncidentDigestMismatch,
    CollisionIncidentMismatch,
    CollisionReceiptRequired,
    UnexpectedCollisionReceipt,
    OpenStreamExhausted,
    HighWatermarkMissing,
    CausalIdentityRegistryMismatch,
    BodyReferenceMissing,
    IdentityReferenceMissing,
    IdentityCommandIdMismatch,
}

impl Display for CommandLedgerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "ledger canonicalization failed: {error}"),
            Self::Command(error) => write!(formatter, "ledger command body is invalid: {error}"),
            Self::CountOverflow => formatter.write_str("ledger count overflow"),
            Self::UnsupportedLedgerVersion(version) => {
                write!(formatter, "unsupported command ledger version {version}")
            }
            Self::UnsupportedStreamLedgerVersion(version) => {
                write!(
                    formatter,
                    "unsupported command stream ledger version {version}"
                )
            }
            Self::UnsupportedReceiptVersion(version) => {
                write!(formatter, "unsupported command receipt version {version}")
            }
            Self::UnsupportedIdentityIndexVersion(version) => {
                write!(
                    formatter,
                    "unsupported command identity index version {version}"
                )
            }
            Self::CommandBodyHashCollision => {
                formatter.write_str("command body hash maps to different canonical bytes")
            }
            Self::CommandBodyArchiveCorrupt => {
                formatter.write_str("command body archive closure is corrupt")
            }
            Self::IdentityIndexCountMismatch => {
                formatter.write_str("command identity index counts do not match its bindings")
            }
            Self::IdentityIndexKeyMismatch => {
                formatter.write_str("command identity index map key does not match its binding")
            }
            Self::IdentityOccurrencesInvalid => {
                formatter.write_str("command identity occurrences are empty, duplicate or unsorted")
            }
            Self::IdentityBindingStateMismatch => {
                formatter.write_str("command identity binding state does not match occurrences")
            }
            Self::IdentityIndexRootMismatch => {
                formatter.write_str("command identity index root does not match canonical body")
            }
            Self::StreamKeyMismatch => {
                formatter.write_str("command ledger stream map key does not match stream ledger")
            }
            Self::StreamNotOpen => formatter.write_str("command stream is not open"),
            Self::SequenceNotNew => {
                formatter.write_str("command sequence does not exceed admission high-watermark")
            }
            Self::TargetTickRegression => {
                formatter.write_str("command target tick regresses within its stream")
            }
            Self::PendingLimit => formatter.write_str("command stream pending limit exceeded"),
            Self::ReservationMismatch => {
                formatter.write_str("command reservation does not match its stream or map key")
            }
            Self::ReceiptSubjectMismatch => {
                formatter.write_str("command receipt subject does not match its stream")
            }
            Self::ReceiptResultMismatch => {
                formatter.write_str("command receipt subject and final result do not match")
            }
            Self::FinalizationOrdinalMismatch => {
                formatter.write_str("command receipt finalization ordinal is not next")
            }
            Self::FinalizationOrdinalExhausted => {
                formatter.write_str("command receipt finalization ordinal exhausted")
            }
            Self::ReceiptWindowLengthMismatch => {
                formatter.write_str("command receipt window length violates fixed capacity")
            }
            Self::ReceiptWindowInvalid => {
                formatter.write_str("command receipt window is not the exact ordinal suffix")
            }
            Self::ReceiptChainRootMismatch => {
                formatter.write_str("command receipt chain root does not match retained history")
            }
            Self::CollisionCandidatesInvalid => {
                formatter.write_str("command collision candidates are invalid")
            }
            Self::CollisionCandidatesRootMismatch => {
                formatter.write_str("command collision candidates root does not match candidates")
            }
            Self::CollisionIncidentDigestMismatch => {
                formatter.write_str("command collision incident digest does not match incident")
            }
            Self::CollisionIncidentMismatch => {
                formatter.write_str("command collision incident does not match stream state")
            }
            Self::CollisionReceiptRequired => {
                formatter.write_str("new command collision requires an atomic terminal receipt")
            }
            Self::UnexpectedCollisionReceipt => {
                formatter.write_str("collision receipt is not valid for retained stream state")
            }
            Self::OpenStreamExhausted => {
                formatter.write_str("open command stream has exhausted sequence or receipt count")
            }
            Self::HighWatermarkMissing => {
                formatter.write_str("command stream history exists without a high-watermark")
            }
            Self::CausalIdentityRegistryMismatch => {
                formatter.write_str("causal identity registry does not match ledger world")
            }
            Self::BodyReferenceMissing => {
                formatter.write_str("ledger body reference is missing from body archive")
            }
            Self::IdentityReferenceMissing => {
                formatter.write_str("ledger body reference is missing from identity index")
            }
            Self::IdentityCommandIdMismatch => {
                formatter.write_str("identity index command ID does not match archived body bytes")
            }
        }
    }
}

impl Error for CommandLedgerError {}

impl From<CanonicalError> for CommandLedgerError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CommandDecodeError> for CommandLedgerError {
    fn from(error: CommandDecodeError) -> Self {
        Self::Command(error)
    }
}

fn collision_candidates_are_canonical(candidates: &[CommandCollisionCandidateV1]) -> bool {
    !candidates.is_empty()
        && candidates
            .iter()
            .all(|candidate| candidate.body_hash == candidate.canonical_body_ref)
        && candidates.windows(2).all(|pair| {
            pair[0].body_hash != pair[1].body_hash
                && (&pair[0].body_hash, &pair[0].command_id)
                    < (&pair[1].body_hash, &pair[1].command_id)
        })
}

pub fn command_body_archive_root(
    entries: &BTreeMap<CommandBodyHash, Vec<u8>>,
) -> Result<ContentHash, CanonicalError> {
    let mut nodes = Vec::with_capacity(entries.len());
    for (body_hash, body_bytes) in entries {
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.command-body-archive-leaf.v1\0");
        preimage.extend_from_slice(body_hash.as_bytes());
        preimage.extend_from_slice(
            &u64::try_from(body_bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(body_bytes);
        nodes.push(sha256(&preimage));
    }
    let merkle_root = if nodes.is_empty() {
        sha256(b"nextengine.command-body-archive-empty.v1\0")
    } else {
        while nodes.len() > 1 {
            let mut parents = Vec::with_capacity(nodes.len().div_ceil(2));
            for pair in nodes.chunks(2) {
                let mut preimage = Vec::new();
                if let [left, right] = pair {
                    preimage.extend_from_slice(b"nextengine.command-body-archive-node.v1\0");
                    preimage.extend_from_slice(left);
                    preimage.extend_from_slice(right);
                } else {
                    preimage.extend_from_slice(b"nextengine.command-body-archive-carry.v1\0");
                    preimage.extend_from_slice(&pair[0]);
                }
                parents.push(sha256(&preimage));
            }
            nodes = parents;
        }
        nodes[0]
    };
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-body-archive-root.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(entries.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(&merkle_root);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub fn command_identity_index_root(
    body: &CommandIdentityIndexBodyV1,
) -> Result<ContentHash, CanonicalError> {
    let bytes = body.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-identity-index.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(&bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

#[must_use]
pub fn command_receipt_chain_genesis() -> ContentHash {
    content_hash_from_bytes(sha256(b"nextengine.command-ledger-chain.genesis.v1\0"))
}

pub fn command_receipt_digest(receipt: &CommandReceiptV1) -> Result<ContentHash, CanonicalError> {
    let bytes = receipt.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-receipt.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(&bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub fn command_receipt_chain_next(
    previous_root: ContentHash,
    receipt: &CommandReceiptV1,
) -> Result<ContentHash, CanonicalError> {
    let receipt_digest = command_receipt_digest(receipt)?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-ledger-chain.v1\0");
    preimage.extend_from_slice(previous_root.as_bytes());
    preimage.extend_from_slice(receipt_digest.as_bytes());
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub fn command_collision_candidates_root(
    candidates: &[CommandCollisionCandidateV1],
) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-collision-set.v1\0");
    preimage.extend_from_slice(
        &u32::try_from(candidates.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for candidate in candidates {
        preimage.extend_from_slice(candidate.body_hash.as_bytes());
        preimage.extend_from_slice(candidate.command_id.as_bytes());
    }
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

#[must_use]
pub fn command_collision_incident_digest(
    stream_id: CommandStreamId,
    sequence: u64,
    candidates_root: ContentHash,
) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-collision-incident.v1\0");
    preimage.extend_from_slice(stream_id.as_bytes());
    preimage.extend_from_slice(&sequence.to_le_bytes());
    preimage.extend_from_slice(candidates_root.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

fn validate_body_reference(
    archive_hashes: &BTreeSet<CommandBodyHash>,
    command_id: CommandId,
    body_hash: CommandBodyHash,
    index: &CommandIdentityIndexV1,
) -> Result<(), CommandLedgerError> {
    if !archive_hashes.contains(&body_hash) {
        return Err(CommandLedgerError::BodyReferenceMissing);
    }
    let Some(binding) = index.body.bindings.get(&command_id) else {
        return Err(CommandLedgerError::IdentityReferenceMissing);
    };
    if !binding
        .occurrences
        .iter()
        .any(|occurrence| occurrence.body_hash == body_hash)
    {
        return Err(CommandLedgerError::IdentityReferenceMissing);
    }
    Ok(())
}

fn validate_receipt_references(
    receipt: &CommandReceiptV1,
    archive_hashes: &BTreeSet<CommandBodyHash>,
    index: &CommandIdentityIndexV1,
) -> Result<(), CommandLedgerError> {
    match &receipt.subject {
        CommandReceiptSubjectV1::Command {
            command_id,
            body_hash,
            canonical_body_ref,
            ..
        } => {
            if body_hash != canonical_body_ref {
                return Err(CommandLedgerError::BodyReferenceMissing);
            }
            validate_body_reference(archive_hashes, *command_id, *body_hash, index)
        }
        CommandReceiptSubjectV1::CollisionSet { candidates, .. } => {
            for candidate in candidates {
                if candidate.body_hash != candidate.canonical_body_ref {
                    return Err(CommandLedgerError::BodyReferenceMissing);
                }
                validate_body_reference(
                    archive_hashes,
                    candidate.command_id,
                    candidate.body_hash,
                    index,
                )?;
            }
            Ok(())
        }
    }
}

fn validate_receipt_against_reservation(
    receipt: &CommandReceiptV1,
    reservation: &CommandReservationV1,
) -> Result<(), CommandLedgerError> {
    let CommandReceiptSubjectV1::Command {
        stream_id,
        issuer,
        sequence,
        command_id,
        body_hash,
        canonical_body_ref,
    } = &receipt.subject
    else {
        return Err(CommandLedgerError::ReservationMismatch);
    };
    if *stream_id != reservation.stream_id
        || issuer != &reservation.issuer
        || *sequence != reservation.sequence
        || *command_id != reservation.command_id
        || *body_hash != reservation.body_hash
        || *canonical_body_ref != reservation.canonical_body_ref
        || receipt.phase != reservation.phase
        || receipt.target_tick != reservation.target_tick
        || receipt.priority_class != reservation.priority_class
        || receipt.command_kind_registry_hash != reservation.command_kind_registry_hash
    {
        return Err(CommandLedgerError::ReservationMismatch);
    }
    Ok(())
}

fn receipt_matches_collision_incident(
    receipt: &CommandReceiptV1,
    incident: Option<&CommandCollisionIncidentV1>,
) -> bool {
    let (
        CommandReceiptSubjectV1::CollisionSet {
            stream_id,
            issuer,
            sequence,
            candidates_root,
            candidates,
            ..
        },
        Some(incident),
    ) = (&receipt.subject, incident)
    else {
        return false;
    };
    *stream_id == incident.stream_id
        && issuer == &incident.issuer
        && *sequence == incident.sequence
        && *candidates_root == incident.candidates_root
        && candidates == &incident.candidates
}

fn nested_value(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.push(type_tag);
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

fn struct_record(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    if let Some(pair) = fields
        .windows(2)
        .find(|pair| pair[0].field_id == pair[1].field_id)
    {
        return Err(CanonicalError::DuplicateField(pair[0].field_id));
    }
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        payload.extend_from_slice(&field.field_id.to_le_bytes());
        payload.push(field.type_tag);
        payload.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        payload.extend_from_slice(&field.payload);
    }
    nested_value(CANONICAL_TYPE_STRUCT, &payload)
}

fn encode_sequence(records: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(records.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for record in records {
        payload.extend_from_slice(&record);
    }
    Ok(payload)
}

fn encode_map(mut entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<Vec<u8>, CanonicalError> {
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(entries.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for (key, value) in entries {
        payload.extend_from_slice(&key);
        payload.extend_from_slice(&value);
    }
    Ok(payload)
}

fn option_hash(value: Option<&ContentHash>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut payload = vec![1];
    payload.extend_from_slice(&nested_value(CANONICAL_TYPE_HASH256, value.as_bytes())?);
    Ok(payload)
}

fn principal_payload(principal: &IssuerPrincipal) -> Result<Vec<u8>, CanonicalError> {
    let encoded = principal.canonical_bytes()?;
    let header_length = 1 + std::mem::size_of::<u64>();
    encoded
        .get(header_length..)
        .map(ToOwned::to_owned)
        .ok_or(CanonicalError::LengthOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CommandStreamId, IssuerPrincipal, PlayerPrincipalId, SchemaId, WorldCommand,
        content_hash_from_bytes,
    };

    fn command(sequence: u64, target_tick: u64) -> WorldCommand {
        WorldCommand::noop(
            CommandStreamId::from_bytes([1; 16]),
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16])),
            sequence,
            target_tick,
        )
        .expect("test command is canonical")
    }

    fn receipt(ordinal: u64, sequence: u64, result: CommandFinalResultV1) -> CommandReceiptV1 {
        let command = command(sequence, sequence);
        let body_hash = command.body_hash().expect("body hash");
        CommandReceiptV1 {
            schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
            finalization_ordinal: ordinal,
            subject: CommandReceiptSubjectV1::Command {
                stream_id: command.stream_id,
                issuer: command.issuer.clone(),
                sequence,
                command_id: command.compute_command_id().expect("command ID"),
                body_hash,
                canonical_body_ref: body_hash,
            },
            phase: CommandPhase::Ingress,
            target_tick: sequence,
            finalized_at_tick: sequence,
            priority_class: 10,
            command_kind_registry_hash: content_hash_from_bytes([3; 32]),
            result,
            diagnostic_digest: None,
            event_ids: Vec::new(),
            transaction_result_root: content_hash_from_bytes([4; 32]),
        }
    }

    fn collision_candidate(command: &WorldCommand) -> CommandCollisionCandidateV1 {
        let body_hash = command.body_hash().expect("body hash");
        CommandCollisionCandidateV1 {
            command_id: command.compute_command_id().expect("command ID"),
            body_hash,
            canonical_body_ref: body_hash,
        }
    }

    fn collision_receipt(ordinal: u64, incident: &CommandCollisionIncidentV1) -> CommandReceiptV1 {
        CommandReceiptV1 {
            schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
            finalization_ordinal: ordinal,
            subject: CommandReceiptSubjectV1::CollisionSet {
                stream_id: incident.stream_id,
                issuer: incident.issuer.clone(),
                sequence: incident.sequence,
                candidates_root: incident.candidates_root,
                candidate_count: u32::try_from(incident.candidates.len())
                    .expect("bounded candidate count"),
                candidates: incident.candidates.clone(),
            },
            phase: CommandPhase::Ingress,
            target_tick: incident.sequence,
            finalized_at_tick: incident.sequence,
            priority_class: 10,
            command_kind_registry_hash: content_hash_from_bytes([3; 32]),
            result: CommandFinalResultV1::Collision {
                code: SchemaId::new("COMMAND_ID_COLLISION").expect("stable code"),
            },
            diagnostic_digest: Some(incident.incident_digest),
            event_ids: Vec::new(),
            transaction_result_root: content_hash_from_bytes([4; 32]),
        }
    }

    #[test]
    fn body_archive_is_order_independent_and_rejects_corrupt_keys() {
        let mut forward = CommandBodyArchiveV1::default();
        let mut reverse = CommandBodyArchiveV1::default();
        let first = command(1, 1);
        let second = command(2, 2);
        forward.insert_command(&first).expect("first body inserts");
        forward
            .insert_command(&second)
            .expect("second body inserts");
        reverse
            .insert_command(&second)
            .expect("second body inserts");
        reverse.insert_command(&first).expect("first body inserts");

        assert_eq!(
            forward.manifest().expect("manifest"),
            reverse.manifest().expect("manifest")
        );
        assert_eq!(
            forward
                .insert_command(&first)
                .expect("exact retry is idempotent"),
            ArchiveInsertResult::Existing(first.body_hash().expect("body hash"))
        );
        forward.validate().expect("archive closure validates");

        let body_bytes = first.canonical_bytes().expect("canonical body");
        forward
            .entries
            .insert(command_body_hash_from_bytes([9; 32]), body_bytes);
        assert_eq!(
            forward.validate(),
            Err(CommandLedgerError::CommandBodyArchiveCorrupt)
        );
        assert!(matches!(
            CommandBodyArchiveV1::default().insert_body_bytes(vec![1, 2, 3]),
            Err(CommandLedgerError::Command(_))
        ));
    }

    #[test]
    fn identity_index_records_full_hash_collisions_without_replacing_first_occurrence() {
        let mut index = CommandIdentityIndexV1::empty().expect("empty index");
        let command_id = CommandId::from_bytes([7; 16]);
        let first = CommandIdentityOccurrenceV1 {
            body_hash: command_body_hash_from_bytes([1; 32]),
            first_stream_id: CommandStreamId::from_bytes([2; 16]),
            first_sequence: 3,
        };
        let second = CommandIdentityOccurrenceV1 {
            body_hash: command_body_hash_from_bytes([4; 32]),
            first_stream_id: CommandStreamId::from_bytes([5; 16]),
            first_sequence: 6,
        };

        assert_eq!(
            index
                .insert_occurrence(command_id, first.clone())
                .expect("first occurrence"),
            IdentityInsertResult::Inserted
        );
        assert_eq!(
            index
                .insert_occurrence(command_id, first)
                .expect("exact occurrence"),
            IdentityInsertResult::Existing
        );
        assert_eq!(
            index
                .insert_occurrence(command_id, second)
                .expect("collision occurrence"),
            IdentityInsertResult::Collision
        );
        index.validate().expect("index remains valid");
        assert_eq!(
            index.body.bindings[&command_id].state,
            CommandIdentityBindingState::Collision
        );
        assert_eq!(index.body.occurrence_count, 2);
    }

    #[test]
    fn pending_limit_is_exact_at_256() {
        let issuer = command(0, 0).issuer.clone();
        let mut stream =
            CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
        for sequence in 0..COMMAND_PENDING_CAPACITY as u64 {
            let command = command(sequence, sequence);
            stream
                .reserve(
                    CommandReservationV1::from_command(
                        &command,
                        0,
                        10,
                        content_hash_from_bytes([3; 32]),
                    )
                    .expect("reservation"),
                )
                .expect("within pending limit");
            if sequence == 254 {
                assert_eq!(stream.pending.len(), 255);
            }
        }
        assert_eq!(stream.pending.len(), COMMAND_PENDING_CAPACITY);
        stream
            .reserve(
                CommandReservationV1::from_command(
                    &command(0, 0),
                    0,
                    10,
                    content_hash_from_bytes([3; 32]),
                )
                .expect("retry reservation"),
            )
            .expect("exact retry is idempotent at capacity");
        assert_eq!(stream.pending.len(), COMMAND_PENDING_CAPACITY);
        let overflow = command(
            COMMAND_PENDING_CAPACITY as u64,
            COMMAND_PENDING_CAPACITY as u64,
        );
        assert_eq!(
            stream.reserve(
                CommandReservationV1::from_command(
                    &overflow,
                    0,
                    10,
                    content_hash_from_bytes([3; 32]),
                )
                .expect("reservation"),
            ),
            Err(CommandLedgerError::PendingLimit)
        );
    }

    #[test]
    fn receipt_window_is_the_exact_4096_suffix() {
        let issuer = command(0, 0).issuer.clone();
        let mut stream =
            CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
        assert!(stream.receipt_window.is_empty());
        assert_eq!(stream.finalized_receipt_count, 0);
        for ordinal in 0..=COMMAND_RECEIPT_WINDOW_CAPACITY as u64 {
            stream
                .append_receipt(receipt(ordinal, ordinal, CommandFinalResultV1::Committed))
                .expect("receipt appends");
            match ordinal {
                0 => {
                    assert_eq!(stream.receipt_window.len(), 1);
                    assert_eq!(stream.finalized_receipt_count, 1);
                }
                4094 => assert_eq!(stream.receipt_window.len(), 4095),
                4095 => assert_eq!(stream.receipt_window.len(), 4096),
                _ => {}
            }
        }
        assert_eq!(
            stream.finalized_receipt_count,
            COMMAND_RECEIPT_WINDOW_CAPACITY as u64 + 1
        );
        assert_eq!(stream.receipt_window.len(), COMMAND_RECEIPT_WINDOW_CAPACITY);
        assert_eq!(stream.receipt_window[0].finalization_ordinal, 1);
        assert_eq!(
            stream
                .receipt_window
                .last()
                .expect("last receipt")
                .finalization_ordinal,
            COMMAND_RECEIPT_WINDOW_CAPACITY as u64
        );
        stream.validate().expect("window suffix validates");
    }

    #[test]
    fn finalized_sequence_cannot_receive_a_second_receipt() {
        let issuer = command(0, 0).issuer.clone();
        let mut stream =
            CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
        stream
            .append_receipt(receipt(0, 7, CommandFinalResultV1::Committed))
            .expect("first receipt appends");
        let before = stream.clone();
        assert_eq!(
            stream.append_receipt(receipt(1, 7, CommandFinalResultV1::Committed)),
            Err(CommandLedgerError::SequenceNotNew)
        );
        assert_eq!(stream, before);
    }

    #[test]
    fn pending_receipt_must_match_its_reservation_without_partial_mutation() {
        let issuer = command(0, 0).issuer.clone();
        let mut stream =
            CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
        let reserved = command(3, 9);
        stream
            .reserve(
                CommandReservationV1::from_command(
                    &reserved,
                    1,
                    10,
                    content_hash_from_bytes([3; 32]),
                )
                .expect("reservation"),
            )
            .expect("reservation appends");
        let before = stream.clone();
        let mismatched = receipt(0, 3, CommandFinalResultV1::Committed);
        assert_eq!(
            stream.append_receipt(mismatched),
            Err(CommandLedgerError::ReservationMismatch)
        );
        assert_eq!(stream, before);
    }

    #[test]
    fn maximum_sequence_exhausts_stream_atomically() {
        let issuer = command(0, 0).issuer.clone();
        let mut stream =
            CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
        stream
            .append_receipt(receipt(0, u64::MAX, CommandFinalResultV1::Committed))
            .expect("maximum sequence finalizes");
        assert_eq!(stream.state, CommandStreamStateV1::Exhausted);
        assert_eq!(stream.admission_high_watermark, Some(u64::MAX));
        stream.validate().expect("exhausted stream validates");
    }

    #[test]
    fn maximum_sequence_pending_retry_remains_idempotent_after_exhaustion() {
        let maximum = command(u64::MAX, u64::MAX);
        let reservation =
            CommandReservationV1::from_command(&maximum, 0, 10, content_hash_from_bytes([3; 32]))
                .expect("maximum reservation");
        let mut stream =
            CommandStreamLedgerV2::genesis(maximum.stream_id, maximum.issuer.clone(), 0, 0);
        stream
            .reserve(reservation.clone())
            .expect("maximum sequence reserves");
        assert_eq!(stream.state, CommandStreamStateV1::Exhausted);
        stream
            .reserve(reservation)
            .expect("pending exact retry remains idempotent");
        assert_eq!(stream.pending.len(), 1);
    }

    #[test]
    fn new_collision_locks_and_finalizes_in_one_atomic_operation() {
        let first = command(7, 7);
        let second = command(7, 8);
        let incident = CommandCollisionIncidentV1::new(
            first.stream_id,
            first.issuer.clone(),
            first.sequence,
            vec![collision_candidate(&second), collision_candidate(&first)],
        )
        .expect("collision incident");
        let mut stream =
            CommandStreamLedgerV2::genesis(first.stream_id, first.issuer.clone(), 0, 0);
        let receipt = collision_receipt(0, &incident);
        stream
            .append_collision_receipt(incident.clone(), receipt)
            .expect("collision locks and finalizes");

        assert_eq!(stream.state, CommandStreamStateV1::CollisionLocked);
        assert_eq!(stream.collision_incident, Some(incident.clone()));
        assert_eq!(stream.admission_high_watermark, Some(7));
        assert_eq!(stream.finalized_receipt_count, 1);
        assert_eq!(stream.receipt_window.len(), 1);
        stream
            .validate()
            .expect("collision-locked stream validates");

        let before = stream.clone();
        assert_eq!(
            stream.append_collision_receipt(incident.clone(), collision_receipt(1, &incident),),
            Err(CommandLedgerError::UnexpectedCollisionReceipt)
        );
        assert_eq!(stream, before);
    }

    #[test]
    fn ledger_recomputes_command_id_from_archived_body() {
        let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
        let (mut ledger, mut archive) = CommandLedgerV2::empty(
            world_namespace,
            content_hash_from_bytes([3; 32]),
            content_hash_from_bytes([4; 32]),
        )
        .expect("empty ledger");
        let archived = command(1, 1);
        let body_hash = archived.body_hash().expect("body hash");
        archive.insert_command(&archived).expect("archive insert");
        ledger
            .identity_index
            .insert_occurrence(
                CommandId::from_bytes([9; 16]),
                CommandIdentityOccurrenceV1 {
                    body_hash,
                    first_stream_id: archived.stream_id,
                    first_sequence: archived.sequence,
                },
            )
            .expect("index insert");
        ledger.body_archive = archive.manifest().expect("archive manifest");

        assert_eq!(
            ledger.validate(&archive),
            Err(CommandLedgerError::IdentityCommandIdMismatch)
        );
    }

    #[test]
    fn archive_root_is_stable_across_deterministic_insert_permutations() {
        let commands: Vec<_> = (0..16)
            .map(|sequence| command(sequence, sequence + 1))
            .collect();
        let mut baseline = CommandBodyArchiveV1::default();
        for command in &commands {
            baseline.insert_command(command).expect("baseline insert");
        }
        let expected = baseline.manifest().expect("baseline manifest");

        for rotation in 0..commands.len() {
            let mut order: Vec<_> = (0..commands.len()).collect();
            order.rotate_left(rotation);
            if rotation % 2 == 1 {
                order.reverse();
            }
            let mut archive = CommandBodyArchiveV1::default();
            for index in order {
                archive
                    .insert_command(&commands[index])
                    .expect("permuted insert");
            }
            assert_eq!(archive.manifest().expect("permuted manifest"), expected);
        }
    }

    #[test]
    fn archive_synchronization_is_atomic_on_corrupt_input() {
        let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
        let (mut ledger, _) = CommandLedgerV2::empty(
            world_namespace,
            content_hash_from_bytes([3; 32]),
            content_hash_from_bytes([4; 32]),
        )
        .expect("empty ledger");
        let before = ledger.clone();
        let mut corrupt = CommandBodyArchiveV1::default();
        corrupt
            .entries
            .insert(command_body_hash_from_bytes([9; 32]), vec![1, 2, 3]);

        assert!(ledger.synchronize_archive(&corrupt).is_err());
        assert_eq!(ledger, before);
    }

    #[test]
    fn receipt_canonical_bytes_and_chain_are_result_sensitive() {
        let committed = receipt(0, 0, CommandFinalResultV1::Committed);
        let rejected = receipt(
            0,
            0,
            CommandFinalResultV1::Rejected {
                code: SchemaId::new("COMMAND_PRECONDITION_FAILED").expect("stable code"),
            },
        );
        assert_ne!(
            committed.canonical_bytes().expect("receipt bytes"),
            rejected.canonical_bytes().expect("receipt bytes")
        );
        assert_ne!(
            command_receipt_chain_next(command_receipt_chain_genesis(), &committed)
                .expect("chain root"),
            command_receipt_chain_next(command_receipt_chain_genesis(), &rejected)
                .expect("chain root")
        );
    }
}
