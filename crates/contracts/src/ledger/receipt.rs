use super::hashes::*;
use super::*;

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
