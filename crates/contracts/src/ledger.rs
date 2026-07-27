use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP,
    CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_STRUCT,
    CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CANONICAL_TYPE_UNIT, CANONICAL_TYPE_UTF8_NFC, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::command::{
    CommandDecodeError, CommandPhase, IssuerPrincipal, PrincipalDecodeError, WorldCommand,
    compute_command_id_from_body_bytes,
};
use crate::ids::{
    CommandBodyHash, CommandId, CommandLedgerHash, CommandStreamId, ContentHash, EventId, SchemaId,
    WorldNamespaceId, command_body_hash_from_bytes, command_ledger_hash_from_bytes,
    content_hash_from_bytes,
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
pub const COMMAND_LEDGER_OWNER_ID: &str = "nextengine.runtime";
pub const COMMAND_LEDGER_SCHEMA_ID: &str = "nextengine.command-ledger";
pub const COMMAND_LEDGER_SEGMENT_ID: &str = "v2";
pub const COMMAND_BODY_ARCHIVE_OWNER_ID: &str = "nextengine.runtime";
pub const COMMAND_BODY_ARCHIVE_SCHEMA_ID: &str = "nextengine.command-body-archive";
pub const COMMAND_BODY_ARCHIVE_SEGMENT_ID: &str = "v1";

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

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut entries = Vec::new();
        entries.extend_from_slice(
            &u32::try_from(self.entries.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        for (body_hash, body_bytes) in &self.entries {
            entries.extend_from_slice(body_hash.as_bytes());
            extend_u32_bytes(&mut entries, body_bytes)?;
        }
        encode_canonical_segment(
            COMMAND_BODY_ARCHIVE_OWNER_ID,
            COMMAND_BODY_ARCHIVE_SCHEMA_ID,
            COMMAND_BODY_ARCHIVE_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    COMMAND_BODY_ARCHIVE_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_MAP, entries),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandLedgerError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != COMMAND_BODY_ARCHIVE_OWNER_ID
            || segment.schema_id != COMMAND_BODY_ARCHIVE_SCHEMA_ID
            || segment.segment_id != COMMAND_BODY_ARCHIVE_SEGMENT_ID
        {
            return Err(CommandLedgerError::WrongEnvelope);
        }
        require_ledger_fields(
            &segment,
            &[(1, CANONICAL_TYPE_U16), (2, CANONICAL_TYPE_MAP)],
        )?;
        let version = u16::from_le_bytes(fixed_field(&segment, 1)?);
        if version != COMMAND_BODY_ARCHIVE_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedArchiveVersion(version));
        }
        let mut cursor = CanonicalCursor::new(&ledger_field(&segment, 2)?.payload);
        let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
            CanonicalDecodeError::TooManyFields { actual, limit }
        })?;
        let mut entries = BTreeMap::new();
        let mut previous = None;
        for _ in 0..count {
            let body_hash = CommandBodyHash::from_bytes(read_array(&mut cursor)?);
            if previous.is_some_and(|prior| prior >= body_hash) {
                return Err(CommandLedgerError::MapNotStrictlySorted);
            }
            let body_bytes = cursor
                .read_u32_length_prefixed(limits.max_total_bytes)?
                .to_vec();
            entries.insert(body_hash, body_bytes);
            previous = Some(body_hash);
        }
        cursor.finish()?;
        let archive = Self { entries };
        archive.validate()?;
        if archive.canonical_bytes()? != bytes {
            return Err(CommandLedgerError::NonCanonicalEncoding);
        }
        Ok(archive)
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

impl CausalIdentityRegistryV1 {
    pub fn compare_or_insert(
        &mut self,
        key: CausalIdentityKey,
        provenance_hash: ContentHash,
    ) -> Result<IdentityInsertResult, CommandLedgerError> {
        if self.schema_version != CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION {
            return Err(CommandLedgerError::CausalIdentityRegistryMismatch);
        }
        match self.bindings.get(&key) {
            Some(existing) if *existing == provenance_hash => Ok(IdentityInsertResult::Existing),
            Some(_) => Ok(IdentityInsertResult::Collision),
            None => {
                let mut next = self.clone();
                next.bindings.insert(key, provenance_hash);
                *self = next;
                Ok(IdentityInsertResult::Inserted)
            }
        }
    }

    pub fn compare_or_insert_provenance(
        &mut self,
        identity_kind: CausalIdentityKind,
        identity_bytes: [u8; 16],
        canonical_provenance_bytes: &[u8],
    ) -> Result<IdentityInsertResult, CommandLedgerError> {
        let provenance_hash = causal_provenance_hash(identity_kind, canonical_provenance_bytes)?;
        self.compare_or_insert(
            CausalIdentityKey {
                identity_kind,
                identity_bytes,
            },
            provenance_hash,
        )
    }
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

    pub fn canonical_bytes(
        &self,
        archive: &CommandBodyArchiveV1,
    ) -> Result<Vec<u8>, CommandLedgerError> {
        self.validate(archive)?;
        encode_canonical_segment(
            COMMAND_LEDGER_OWNER_ID,
            COMMAND_LEDGER_SCHEMA_ID,
            COMMAND_LEDGER_SEGMENT_ID,
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
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_streams(&self.streams)?),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_HASH256,
                    self.command_kind_registry_hash.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_HASH256,
                    self.runtime_determinism_profile_hash.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_STRUCT,
                    encode_archive_manifest(self.body_archive),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_BYTES,
                    encode_identity_index(&self.identity_index)?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_MAP,
                    encode_causal_registry(&self.causal_identity_registry)?,
                ),
            ],
        )
        .map_err(Into::into)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        archive: &CommandBodyArchiveV1,
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandLedgerError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != COMMAND_LEDGER_OWNER_ID
            || segment.schema_id != COMMAND_LEDGER_SCHEMA_ID
            || segment.segment_id != COMMAND_LEDGER_SEGMENT_ID
        {
            return Err(CommandLedgerError::WrongEnvelope);
        }
        require_ledger_fields(
            &segment,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_MAP),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_HASH256),
                (6, CANONICAL_TYPE_STRUCT),
                (7, CANONICAL_TYPE_BYTES),
                (8, CANONICAL_TYPE_MAP),
            ],
        )?;
        let schema_version = u16::from_le_bytes(fixed_field(&segment, 1)?);
        if schema_version != COMMAND_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedLedgerVersion(schema_version));
        }
        let world_namespace = WorldNamespaceId::from_bytes(fixed_field(&segment, 2)?);
        let ledger = Self {
            schema_version,
            world_namespace,
            streams: decode_streams(&ledger_field(&segment, 3)?.payload, limits)?,
            command_kind_registry_hash: ContentHash::from_bytes(fixed_field(&segment, 4)?),
            runtime_determinism_profile_hash: ContentHash::from_bytes(fixed_field(&segment, 5)?),
            body_archive: decode_archive_manifest(&ledger_field(&segment, 6)?.payload)?,
            identity_index: decode_identity_index(&ledger_field(&segment, 7)?.payload, limits)?,
            causal_identity_registry: decode_causal_registry(
                &ledger_field(&segment, 8)?.payload,
                world_namespace,
                limits,
            )?,
        };
        ledger.validate(archive)?;
        if ledger.canonical_bytes(archive)? != bytes {
            return Err(CommandLedgerError::NonCanonicalEncoding);
        }
        Ok(ledger)
    }

    pub fn command_ledger_hash(
        &self,
        archive: &CommandBodyArchiveV1,
    ) -> Result<CommandLedgerHash, CommandLedgerError> {
        let bytes = self.canonical_bytes(archive)?;
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.command-ledger.v2\0");
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CommandLedgerError::CountOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&bytes);
        Ok(command_ledger_hash_from_bytes(sha256(&preimage)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommandLedgerError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Command(CommandDecodeError),
    Principal(PrincipalDecodeError),
    Identifier(crate::IdentifierError),
    CountOverflow,
    UnsupportedLedgerVersion(u16),
    UnsupportedStreamLedgerVersion(u16),
    UnsupportedReceiptVersion(u16),
    UnsupportedIdentityIndexVersion(u16),
    UnsupportedArchiveVersion(u16),
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
    WrongEnvelope,
    MissingField(u32),
    UnknownField(u32),
    WrongFieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    InvalidFieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnknownTag(u8),
    MapNotStrictlySorted,
    NonCanonicalEncoding,
}

impl Display for CommandLedgerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "ledger canonicalization failed: {error}"),
            Self::Decode(error) => write!(formatter, "ledger encoding is invalid: {error}"),
            Self::Command(error) => write!(formatter, "ledger command body is invalid: {error}"),
            Self::Principal(error) => write!(formatter, "ledger principal is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "ledger identifier is invalid: {error}"),
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
            Self::UnsupportedArchiveVersion(version) => {
                write!(
                    formatter,
                    "unsupported command body archive version {version}"
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
            Self::WrongEnvelope => formatter.write_str("ledger envelope does not match"),
            Self::MissingField(field_id) => write!(formatter, "ledger field {field_id} is missing"),
            Self::UnknownField(field_id) => write!(formatter, "ledger field {field_id} is unknown"),
            Self::WrongFieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "ledger field {field_id} has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::InvalidFieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "ledger field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnknownTag(tag) => write!(formatter, "ledger tag {tag} is unknown"),
            Self::MapNotStrictlySorted => formatter.write_str("ledger map is not strictly sorted"),
            Self::NonCanonicalEncoding => {
                formatter.write_str("ledger does not re-encode byte-exactly")
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

impl From<CanonicalDecodeError> for CommandLedgerError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<CommandDecodeError> for CommandLedgerError {
    fn from(error: CommandDecodeError) -> Self {
        Self::Command(error)
    }
}

impl From<PrincipalDecodeError> for CommandLedgerError {
    fn from(error: PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<crate::IdentifierError> for CommandLedgerError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

pub fn causal_provenance_hash(
    identity_kind: CausalIdentityKind,
    canonical_provenance_bytes: &[u8],
) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.causal-provenance.v1\0");
    preimage.push(identity_kind as u8);
    preimage.extend_from_slice(
        &u64::try_from(canonical_provenance_bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(canonical_provenance_bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn encode_streams(
    streams: &BTreeMap<CommandStreamId, CommandStreamLedgerV2>,
) -> Result<Vec<u8>, CommandLedgerError> {
    let mut writer = LedgerWriter::default();
    writer.count(streams.len())?;
    for (stream_id, stream) in streams {
        writer.bytes(stream_id.as_bytes());
        encode_stream(&mut writer, stream)?;
    }
    Ok(writer.finish())
}

fn decode_streams(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<BTreeMap<CommandStreamId, CommandStreamLedgerV2>, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, limits);
    let count = reader.count()?;
    let mut streams = BTreeMap::new();
    let mut previous = None;
    for _ in 0..count {
        let stream_id = CommandStreamId::from_bytes(reader.array()?);
        if previous.is_some_and(|prior| prior >= stream_id) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        let stream = decode_stream(&mut reader)?;
        if stream.stream_id != stream_id || streams.insert(stream_id, stream).is_some() {
            return Err(CommandLedgerError::StreamKeyMismatch);
        }
        previous = Some(stream_id);
    }
    reader.finish()?;
    Ok(streams)
}

fn encode_stream(
    writer: &mut LedgerWriter,
    stream: &CommandStreamLedgerV2,
) -> Result<(), CommandLedgerError> {
    writer.u16(stream.schema_version);
    writer.bytes(stream.stream_id.as_bytes());
    writer.principal(&stream.issuer)?;
    writer.u32(stream.stream_slot);
    writer.u32(stream.stream_epoch);
    writer.u8(stream.state as u8);
    writer.option_u64(stream.admission_high_watermark);
    writer.option_u64(stream.greatest_reserved_target_tick);
    writer.count(stream.pending.len())?;
    for (sequence, reservation) in &stream.pending {
        writer.u64(*sequence);
        encode_reservation(writer, reservation)?;
    }
    writer.count(stream.receipt_window.len())?;
    for receipt in &stream.receipt_window {
        encode_receipt(writer, receipt)?;
    }
    writer.u64(stream.finalized_receipt_count);
    writer.bytes(stream.receipt_chain_root.as_bytes());
    match &stream.collision_incident {
        None => writer.u8(0),
        Some(incident) => {
            writer.u8(1);
            encode_incident(writer, incident)?;
        }
    }
    Ok(())
}

fn decode_stream(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandStreamLedgerV2, CommandLedgerError> {
    let schema_version = reader.u16()?;
    if schema_version != COMMAND_STREAM_LEDGER_SCHEMA_VERSION {
        return Err(CommandLedgerError::UnsupportedStreamLedgerVersion(
            schema_version,
        ));
    }
    let stream_id = CommandStreamId::from_bytes(reader.array()?);
    let issuer = reader.principal()?;
    let stream_slot = reader.u32()?;
    let stream_epoch = reader.u32()?;
    let state = match reader.u8()? {
        0 => CommandStreamStateV1::Open,
        1 => CommandStreamStateV1::CollisionLocked,
        2 => CommandStreamStateV1::Exhausted,
        3 => CommandStreamStateV1::Closed,
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    let admission_high_watermark = reader.option_u64()?;
    let greatest_reserved_target_tick = reader.option_u64()?;
    let pending_count = reader.count()?;
    if pending_count > COMMAND_PENDING_CAPACITY {
        return Err(CommandLedgerError::PendingLimit);
    }
    let mut pending = BTreeMap::new();
    let mut previous = None;
    for _ in 0..pending_count {
        let sequence = reader.u64()?;
        if previous.is_some_and(|prior| prior >= sequence) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        let reservation = decode_reservation(reader)?;
        if reservation.sequence != sequence || pending.insert(sequence, reservation).is_some() {
            return Err(CommandLedgerError::ReservationMismatch);
        }
        previous = Some(sequence);
    }
    let receipt_count = reader.count()?;
    if receipt_count > COMMAND_RECEIPT_WINDOW_CAPACITY {
        return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
    }
    let mut receipt_window = Vec::with_capacity(receipt_count);
    for _ in 0..receipt_count {
        receipt_window.push(decode_receipt(reader)?);
    }
    let finalized_receipt_count = reader.u64()?;
    let receipt_chain_root = ContentHash::from_bytes(reader.array()?);
    let collision_incident = match reader.u8()? {
        0 => None,
        1 => Some(decode_incident(reader)?),
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    Ok(CommandStreamLedgerV2 {
        schema_version,
        stream_id,
        issuer,
        stream_slot,
        stream_epoch,
        state,
        admission_high_watermark,
        greatest_reserved_target_tick,
        pending,
        receipt_window,
        finalized_receipt_count,
        receipt_chain_root,
        collision_incident,
    })
}

fn encode_reservation(
    writer: &mut LedgerWriter,
    reservation: &CommandReservationV1,
) -> Result<(), CommandLedgerError> {
    writer.u16(reservation.schema_version);
    writer.bytes(reservation.stream_id.as_bytes());
    writer.principal(&reservation.issuer)?;
    writer.u64(reservation.sequence);
    writer.bytes(reservation.command_id.as_bytes());
    writer.bytes(reservation.body_hash.as_bytes());
    writer.bytes(reservation.canonical_body_ref.as_bytes());
    writer.u64(reservation.reserved_at_tick);
    writer.u64(reservation.target_tick);
    writer.u8(reservation.phase as u8);
    writer.u16(reservation.priority_class);
    writer.bytes(reservation.command_kind_registry_hash.as_bytes());
    Ok(())
}

fn decode_reservation(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandReservationV1, CommandLedgerError> {
    let schema_version = reader.u16()?;
    if schema_version != COMMAND_RESERVATION_SCHEMA_VERSION {
        return Err(CommandLedgerError::ReservationMismatch);
    }
    Ok(CommandReservationV1 {
        schema_version,
        stream_id: CommandStreamId::from_bytes(reader.array()?),
        issuer: reader.principal()?,
        sequence: reader.u64()?,
        command_id: CommandId::from_bytes(reader.array()?),
        body_hash: CommandBodyHash::from_bytes(reader.array()?),
        canonical_body_ref: CommandBodyHash::from_bytes(reader.array()?),
        reserved_at_tick: reader.u64()?,
        target_tick: reader.u64()?,
        phase: decode_phase(reader.u8()?)?,
        priority_class: reader.u16()?,
        command_kind_registry_hash: ContentHash::from_bytes(reader.array()?),
    })
}

fn encode_candidate(writer: &mut LedgerWriter, candidate: &CommandCollisionCandidateV1) {
    writer.bytes(candidate.command_id.as_bytes());
    writer.bytes(candidate.body_hash.as_bytes());
    writer.bytes(candidate.canonical_body_ref.as_bytes());
}

fn decode_candidate(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandCollisionCandidateV1, CommandLedgerError> {
    Ok(CommandCollisionCandidateV1 {
        command_id: CommandId::from_bytes(reader.array()?),
        body_hash: CommandBodyHash::from_bytes(reader.array()?),
        canonical_body_ref: CommandBodyHash::from_bytes(reader.array()?),
    })
}

fn encode_incident(
    writer: &mut LedgerWriter,
    incident: &CommandCollisionIncidentV1,
) -> Result<(), CommandLedgerError> {
    writer.bytes(incident.stream_id.as_bytes());
    writer.principal(&incident.issuer)?;
    writer.u64(incident.sequence);
    writer.count(incident.candidates.len())?;
    for candidate in &incident.candidates {
        encode_candidate(writer, candidate);
    }
    writer.bytes(incident.candidates_root.as_bytes());
    writer.bytes(incident.incident_digest.as_bytes());
    Ok(())
}

fn decode_incident(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandCollisionIncidentV1, CommandLedgerError> {
    let stream_id = CommandStreamId::from_bytes(reader.array()?);
    let issuer = reader.principal()?;
    let sequence = reader.u64()?;
    let count = reader.count()?;
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        candidates.push(decode_candidate(reader)?);
    }
    let incident = CommandCollisionIncidentV1 {
        stream_id,
        issuer,
        sequence,
        candidates,
        candidates_root: ContentHash::from_bytes(reader.array()?),
        incident_digest: ContentHash::from_bytes(reader.array()?),
    };
    incident.validate()?;
    Ok(incident)
}

fn encode_receipt(
    writer: &mut LedgerWriter,
    receipt: &CommandReceiptV1,
) -> Result<(), CommandLedgerError> {
    writer.u16(receipt.schema_version);
    writer.u64(receipt.finalization_ordinal);
    match &receipt.subject {
        CommandReceiptSubjectV1::Command {
            stream_id,
            issuer,
            sequence,
            command_id,
            body_hash,
            canonical_body_ref,
        } => {
            writer.u8(1);
            writer.bytes(stream_id.as_bytes());
            writer.principal(issuer)?;
            writer.u64(*sequence);
            writer.bytes(command_id.as_bytes());
            writer.bytes(body_hash.as_bytes());
            writer.bytes(canonical_body_ref.as_bytes());
        }
        CommandReceiptSubjectV1::CollisionSet {
            stream_id,
            issuer,
            sequence,
            candidates_root,
            candidate_count,
            candidates,
        } => {
            writer.u8(2);
            writer.bytes(stream_id.as_bytes());
            writer.principal(issuer)?;
            writer.u64(*sequence);
            writer.bytes(candidates_root.as_bytes());
            writer.u32(*candidate_count);
            writer.count(candidates.len())?;
            for candidate in candidates {
                encode_candidate(writer, candidate);
            }
        }
    }
    writer.u8(receipt.phase as u8);
    writer.u64(receipt.target_tick);
    writer.u64(receipt.finalized_at_tick);
    writer.u16(receipt.priority_class);
    writer.bytes(receipt.command_kind_registry_hash.as_bytes());
    match &receipt.result {
        CommandFinalResultV1::Committed => writer.u8(1),
        CommandFinalResultV1::Rejected { code } => {
            writer.u8(2);
            writer.text(code.as_str())?;
        }
        CommandFinalResultV1::Collision { code } => {
            writer.u8(3);
            writer.text(code.as_str())?;
        }
    }
    writer.option_hash(receipt.diagnostic_digest);
    writer.count(receipt.event_ids.len())?;
    for event_id in &receipt.event_ids {
        writer.bytes(event_id.as_bytes());
    }
    writer.bytes(receipt.transaction_result_root.as_bytes());
    Ok(())
}

fn decode_receipt(reader: &mut LedgerReader<'_>) -> Result<CommandReceiptV1, CommandLedgerError> {
    let schema_version = reader.u16()?;
    if schema_version != COMMAND_RECEIPT_SCHEMA_VERSION {
        return Err(CommandLedgerError::UnsupportedReceiptVersion(
            schema_version,
        ));
    }
    let finalization_ordinal = reader.u64()?;
    let subject = match reader.u8()? {
        1 => CommandReceiptSubjectV1::Command {
            stream_id: CommandStreamId::from_bytes(reader.array()?),
            issuer: reader.principal()?,
            sequence: reader.u64()?,
            command_id: CommandId::from_bytes(reader.array()?),
            body_hash: CommandBodyHash::from_bytes(reader.array()?),
            canonical_body_ref: CommandBodyHash::from_bytes(reader.array()?),
        },
        2 => {
            let stream_id = CommandStreamId::from_bytes(reader.array()?);
            let issuer = reader.principal()?;
            let sequence = reader.u64()?;
            let candidates_root = ContentHash::from_bytes(reader.array()?);
            let candidate_count = reader.u32()?;
            let count = reader.count()?;
            let mut candidates = Vec::with_capacity(count);
            for _ in 0..count {
                candidates.push(decode_candidate(reader)?);
            }
            CommandReceiptSubjectV1::CollisionSet {
                stream_id,
                issuer,
                sequence,
                candidates_root,
                candidate_count,
                candidates,
            }
        }
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    let phase = decode_phase(reader.u8()?)?;
    let target_tick = reader.u64()?;
    let finalized_at_tick = reader.u64()?;
    let priority_class = reader.u16()?;
    let command_kind_registry_hash = ContentHash::from_bytes(reader.array()?);
    let result = match reader.u8()? {
        1 => CommandFinalResultV1::Committed,
        2 => CommandFinalResultV1::Rejected {
            code: SchemaId::new(reader.text()?)?,
        },
        3 => CommandFinalResultV1::Collision {
            code: SchemaId::new(reader.text()?)?,
        },
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    let diagnostic_digest = reader.option_hash()?;
    let event_count = reader.count()?;
    let mut event_ids = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        event_ids.push(EventId::from_bytes(reader.array()?));
    }
    let receipt = CommandReceiptV1 {
        schema_version,
        finalization_ordinal,
        subject,
        phase,
        target_tick,
        finalized_at_tick,
        priority_class,
        command_kind_registry_hash,
        result,
        diagnostic_digest,
        event_ids,
        transaction_result_root: ContentHash::from_bytes(reader.array()?),
    };
    receipt.validate()?;
    Ok(receipt)
}

fn encode_archive_manifest(manifest: CommandBodyArchiveManifestV1) -> Vec<u8> {
    let mut writer = LedgerWriter::default();
    writer.u16(manifest.schema_version);
    writer.u64(manifest.entry_count);
    writer.bytes(manifest.archive_root.as_bytes());
    writer.finish()
}

fn decode_archive_manifest(
    bytes: &[u8],
) -> Result<CommandBodyArchiveManifestV1, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, CanonicalDecodeLimits::default());
    let manifest = CommandBodyArchiveManifestV1 {
        schema_version: reader.u16()?,
        entry_count: reader.u64()?,
        archive_root: ContentHash::from_bytes(reader.array()?),
    };
    reader.finish()?;
    if manifest.schema_version != COMMAND_BODY_ARCHIVE_SCHEMA_VERSION {
        return Err(CommandLedgerError::UnsupportedArchiveVersion(
            manifest.schema_version,
        ));
    }
    Ok(manifest)
}

fn encode_identity_index(index: &CommandIdentityIndexV1) -> Result<Vec<u8>, CommandLedgerError> {
    let mut writer = LedgerWriter::default();
    writer.u16(index.schema_version);
    writer.u16(index.body.schema_version);
    writer.u64(index.body.command_id_count);
    writer.u64(index.body.occurrence_count);
    writer.count(index.body.bindings.len())?;
    for (command_id, binding) in &index.body.bindings {
        writer.bytes(command_id.as_bytes());
        writer.bytes(binding.command_id.as_bytes());
        writer.u8(binding.state as u8);
        writer.count(binding.occurrences.len())?;
        for occurrence in &binding.occurrences {
            writer.bytes(occurrence.body_hash.as_bytes());
            writer.bytes(occurrence.first_stream_id.as_bytes());
            writer.u64(occurrence.first_sequence);
        }
    }
    writer.bytes(index.index_root.as_bytes());
    Ok(writer.finish())
}

fn decode_identity_index(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<CommandIdentityIndexV1, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, limits);
    let schema_version = reader.u16()?;
    let body_schema_version = reader.u16()?;
    let command_id_count = reader.u64()?;
    let occurrence_count = reader.u64()?;
    let count = reader.count()?;
    let mut bindings = BTreeMap::new();
    let mut previous = None;
    for _ in 0..count {
        let key = CommandId::from_bytes(reader.array()?);
        if previous.is_some_and(|prior| prior >= key) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        let command_id = CommandId::from_bytes(reader.array()?);
        let state = match reader.u8()? {
            0 => CommandIdentityBindingState::Unique,
            1 => CommandIdentityBindingState::Collision,
            tag => return Err(CommandLedgerError::UnknownTag(tag)),
        };
        let occurrence_len = reader.count()?;
        let mut occurrences = Vec::with_capacity(occurrence_len);
        for _ in 0..occurrence_len {
            occurrences.push(CommandIdentityOccurrenceV1 {
                body_hash: CommandBodyHash::from_bytes(reader.array()?),
                first_stream_id: CommandStreamId::from_bytes(reader.array()?),
                first_sequence: reader.u64()?,
            });
        }
        bindings.insert(
            key,
            CommandIdentityBindingV1 {
                command_id,
                occurrences,
                state,
            },
        );
        previous = Some(key);
    }
    let index = CommandIdentityIndexV1 {
        schema_version,
        body: CommandIdentityIndexBodyV1 {
            schema_version: body_schema_version,
            bindings,
            command_id_count,
            occurrence_count,
        },
        index_root: ContentHash::from_bytes(reader.array()?),
    };
    reader.finish()?;
    index.validate()?;
    Ok(index)
}

fn encode_causal_registry(
    registry: &CausalIdentityRegistryV1,
) -> Result<Vec<u8>, CommandLedgerError> {
    let mut writer = LedgerWriter::default();
    writer.u16(registry.schema_version);
    writer.bytes(registry.world_namespace.as_bytes());
    writer.count(registry.bindings.len())?;
    for (key, hash) in &registry.bindings {
        writer.u8(key.identity_kind as u8);
        writer.bytes(&key.identity_bytes);
        writer.bytes(hash.as_bytes());
    }
    Ok(writer.finish())
}

fn decode_causal_registry(
    bytes: &[u8],
    expected_world: WorldNamespaceId,
    limits: CanonicalDecodeLimits,
) -> Result<CausalIdentityRegistryV1, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, limits);
    let schema_version = reader.u16()?;
    let world_namespace = WorldNamespaceId::from_bytes(reader.array()?);
    let count = reader.count()?;
    let mut bindings = BTreeMap::new();
    let mut previous = None;
    for _ in 0..count {
        let identity_kind = match reader.u8()? {
            1 => CausalIdentityKind::PlayerPrincipal,
            2 => CausalIdentityKind::CommandStream,
            3 => CausalIdentityKind::DomainEvent,
            4 => CausalIdentityKind::PersistentRecord,
            5 => CausalIdentityKind::RngStream,
            tag => return Err(CommandLedgerError::UnknownTag(tag)),
        };
        let key = CausalIdentityKey {
            identity_kind,
            identity_bytes: reader.array()?,
        };
        if previous.is_some_and(|prior| prior >= key) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        bindings.insert(key, ContentHash::from_bytes(reader.array()?));
        previous = Some(key);
    }
    reader.finish()?;
    if schema_version != CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION
        || world_namespace != expected_world
    {
        return Err(CommandLedgerError::CausalIdentityRegistryMismatch);
    }
    Ok(CausalIdentityRegistryV1 {
        schema_version,
        world_namespace,
        bindings,
    })
}

fn decode_phase(tag: u8) -> Result<CommandPhase, CommandLedgerError> {
    match tag {
        0 => Ok(CommandPhase::Ingress),
        1 => Ok(CommandPhase::Outcome),
        _ => Err(CommandLedgerError::UnknownTag(tag)),
    }
}

fn extend_u32_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CanonicalError> {
    output.extend_from_slice(
        &u32::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    output.extend_from_slice(bytes);
    Ok(())
}

fn require_ledger_fields(
    segment: &crate::DecodedCanonicalSegment,
    expected: &[(u32, u8)],
) -> Result<(), CommandLedgerError> {
    for actual in &segment.fields {
        if !expected.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(CommandLedgerError::UnknownField(actual.field_id));
        }
    }
    for (id, tag) in expected {
        let actual = ledger_field(segment, *id)?;
        if actual.type_tag != *tag {
            return Err(CommandLedgerError::WrongFieldType {
                field_id: *id,
                expected: *tag,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

fn ledger_field(
    segment: &crate::DecodedCanonicalSegment,
    id: u32,
) -> Result<&CanonicalField, CommandLedgerError> {
    segment
        .field(id)
        .ok_or(CommandLedgerError::MissingField(id))
}

fn fixed_field<const N: usize>(
    segment: &crate::DecodedCanonicalSegment,
    id: u32,
) -> Result<[u8; N], CommandLedgerError> {
    ledger_field(segment, id)?
        .payload
        .as_slice()
        .try_into()
        .map_err(|_| CommandLedgerError::InvalidFieldLength {
            field_id: id,
            expected: N,
            actual: ledger_field(segment, id).map_or(0, |field| field.payload.len()),
        })
}

#[derive(Default)]
struct LedgerWriter {
    bytes: Vec<u8>,
}

impl LedgerWriter {
    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn count(&mut self, value: usize) -> Result<(), CommandLedgerError> {
        self.u32(u32::try_from(value).map_err(|_| CommandLedgerError::CountOverflow)?);
        Ok(())
    }

    fn sized_bytes(&mut self, value: &[u8]) -> Result<(), CommandLedgerError> {
        self.count(value.len())?;
        self.bytes(value);
        Ok(())
    }

    fn text(&mut self, value: &str) -> Result<(), CommandLedgerError> {
        self.sized_bytes(value.as_bytes())
    }

    fn principal(&mut self, principal: &IssuerPrincipal) -> Result<(), CommandLedgerError> {
        self.sized_bytes(&principal.canonical_bytes()?)
    }

    fn option_u64(&mut self, value: Option<u64>) {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.u64(value);
            }
        }
    }

    fn option_hash(&mut self, value: Option<ContentHash>) {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.bytes(value.as_bytes());
            }
        }
    }
}

struct LedgerReader<'a> {
    cursor: CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
}

impl<'a> LedgerReader<'a> {
    fn new(bytes: &'a [u8], limits: CanonicalDecodeLimits) -> Self {
        Self {
            cursor: CanonicalCursor::new(bytes),
            limits,
        }
    }

    fn finish(self) -> Result<(), CommandLedgerError> {
        self.cursor.finish().map_err(Into::into)
    }

    fn u8(&mut self) -> Result<u8, CommandLedgerError> {
        self.cursor.read_u8().map_err(Into::into)
    }

    fn u16(&mut self) -> Result<u16, CommandLedgerError> {
        self.cursor.read_u16().map_err(Into::into)
    }

    fn u32(&mut self) -> Result<u32, CommandLedgerError> {
        self.cursor.read_u32().map_err(Into::into)
    }

    fn u64(&mut self) -> Result<u64, CommandLedgerError> {
        self.cursor.read_u64().map_err(Into::into)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], CommandLedgerError> {
        read_array(&mut self.cursor)
    }

    fn count(&mut self) -> Result<usize, CommandLedgerError> {
        self.cursor
            .read_count(self.limits.max_sequence_items, |actual, limit| {
                CanonicalDecodeError::TooManyFields { actual, limit }
            })
            .map_err(Into::into)
    }

    fn sized_bytes(&mut self) -> Result<&'a [u8], CommandLedgerError> {
        self.cursor
            .read_u32_length_prefixed(self.limits.max_field_payload_bytes)
            .map_err(Into::into)
    }

    fn text(&mut self) -> Result<&'a str, CommandLedgerError> {
        std::str::from_utf8(self.sized_bytes()?)
            .map_err(|_| CommandLedgerError::Decode(CanonicalDecodeError::InvalidUtf8))
    }

    fn principal(&mut self) -> Result<IssuerPrincipal, CommandLedgerError> {
        let limits = self.limits;
        let bytes = self.sized_bytes()?;
        IssuerPrincipal::from_canonical_bytes(bytes, limits).map_err(Into::into)
    }

    fn option_u64(&mut self) -> Result<Option<u64>, CommandLedgerError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u64()?)),
            tag => Err(CommandLedgerError::UnknownTag(tag)),
        }
    }

    fn option_hash(&mut self) -> Result<Option<ContentHash>, CommandLedgerError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(ContentHash::from_bytes(self.array()?))),
            tag => Err(CommandLedgerError::UnknownTag(tag)),
        }
    }
}

fn read_array<const N: usize>(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<[u8; N], CommandLedgerError> {
    cursor
        .read_exact(N)?
        .try_into()
        .map_err(|_| CommandLedgerError::Decode(CanonicalDecodeError::UnexpectedEnd))
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
    fn complete_ledger_round_trip_rejects_corrupt_archive_index_and_chain_roots() {
        let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
        let (mut ledger, mut archive) = CommandLedgerV2::empty(
            world_namespace,
            content_hash_from_bytes([3; 32]),
            content_hash_from_bytes([5; 32]),
        )
        .expect("empty ledger");
        let command = command(0, 0);
        let command_id = command.compute_command_id().expect("command ID");
        let body_hash = command.body_hash().expect("body hash");
        archive.insert_command(&command).expect("archive command");
        ledger
            .identity_index
            .insert_occurrence(
                command_id,
                CommandIdentityOccurrenceV1 {
                    body_hash,
                    first_stream_id: command.stream_id,
                    first_sequence: command.sequence,
                },
            )
            .expect("identity occurrence");
        let mut stream =
            CommandStreamLedgerV2::genesis(command.stream_id, command.issuer.clone(), 0, 0);
        stream
            .append_receipt(receipt(0, 0, CommandFinalResultV1::Committed))
            .expect("receipt");
        ledger.streams.insert(command.stream_id, stream);
        ledger
            .synchronize_archive(&archive)
            .expect("archive closure");

        let archive_bytes = archive.canonical_bytes().expect("archive bytes");
        let decoded_archive = CommandBodyArchiveV1::from_canonical_bytes(
            &archive_bytes,
            CanonicalDecodeLimits::default(),
        )
        .expect("archive decodes");
        assert_eq!(decoded_archive, archive);
        let bytes = ledger.canonical_bytes(&archive).expect("ledger bytes");
        let decoded = CommandLedgerV2::from_canonical_bytes(
            &bytes,
            &archive,
            CanonicalDecodeLimits::default(),
        )
        .expect("ledger decodes");
        assert_eq!(decoded, ledger);
        assert_eq!(
            decoded
                .canonical_bytes(&archive)
                .expect("ledger re-encodes"),
            bytes
        );

        let mut corrupt_chain = ledger.clone();
        corrupt_chain
            .streams
            .get_mut(&command.stream_id)
            .expect("stream")
            .receipt_chain_root = ContentHash::from_bytes([9; 32]);
        assert_eq!(
            corrupt_chain.validate(&archive),
            Err(CommandLedgerError::ReceiptChainRootMismatch)
        );

        let mut corrupt_archive_root = ledger.clone();
        corrupt_archive_root.body_archive.archive_root = ContentHash::from_bytes([9; 32]);
        assert_eq!(
            corrupt_archive_root.validate(&archive),
            Err(CommandLedgerError::CommandBodyArchiveCorrupt)
        );

        let mut corrupt_index_root = ledger.clone();
        corrupt_index_root.identity_index.index_root = ContentHash::from_bytes([9; 32]);
        assert_eq!(
            corrupt_index_root.validate(&archive),
            Err(CommandLedgerError::IdentityIndexRootMismatch)
        );

        assert_eq!(
            ledger.validate(&CommandBodyArchiveV1::default()),
            Err(CommandLedgerError::CommandBodyArchiveCorrupt)
        );
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
