use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
    extend_u32_length_prefixed, sha256,
};
use crate::{
    CommandDecodeError, CommandId, CommandLedgerHash, CommandStreamId, IssuerPrincipal,
    PrincipalDecodeError, WorldCommand, command_ledger_hash_from_bytes,
    compute_command_id_from_body_bytes,
};

pub const RUNTIME_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const RUNTIME_SNAPSHOT_OWNER_ID: &str = "runtime";
pub const RUNTIME_SNAPSHOT_SCHEMA_ID: &str = "nextengine.runtime.snapshot";
pub const RUNTIME_SNAPSHOT_SEGMENT_ID: &str = "command-ledger";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandLedgerSnapshot {
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub last_sequence: u64,
    pub command_id: CommandId,
    pub canonical_command_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSnapshot {
    pub next_tick: u64,
    pub committed_event_count: u64,
    pub authoritative_revision: u64,
    pub command_ledgers: Vec<CommandLedgerSnapshot>,
}

impl RuntimeSnapshot {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut ledgers = self.command_ledgers.clone();
        ledgers.sort_by(|left, right| {
            (&left.stream_id, &left.issuer).cmp(&(&right.stream_id, &right.issuer))
        });
        if ledgers
            .windows(2)
            .any(|pair| pair[0].stream_id == pair[1].stream_id && pair[0].issuer == pair[1].issuer)
        {
            return Err(CanonicalError::DuplicateSequenceValue);
        }

        let mut ledger_bytes = Vec::new();
        ledger_bytes.extend_from_slice(
            &u32::try_from(ledgers.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        for ledger in ledgers {
            ledger_bytes.extend_from_slice(ledger.stream_id.as_bytes());
            let principal = ledger.issuer.canonical_bytes()?;
            extend_u32_length_prefixed(&mut ledger_bytes, &principal)?;
            ledger_bytes.extend_from_slice(&ledger.last_sequence.to_le_bytes());
            ledger_bytes.extend_from_slice(ledger.command_id.as_bytes());
            extend_u32_length_prefixed(&mut ledger_bytes, &ledger.canonical_command_bytes)?;
        }

        encode_canonical_segment(
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    RUNTIME_SNAPSHOT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_U64, self.next_tick.to_le_bytes().to_vec()),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_U64,
                    self.committed_event_count.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_U64,
                    self.authoritative_revision.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(5, CANONICAL_TYPE_SEQUENCE, ledger_bytes),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SnapshotDecodeError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_snapshot_envelope(&segment)?;
        validate_snapshot_fields(&segment)?;
        let schema_version = decode_u32(&segment, 1)?;
        if schema_version != RUNTIME_SNAPSHOT_SCHEMA_VERSION {
            return Err(SnapshotDecodeError::UnsupportedSchemaVersion(
                schema_version,
            ));
        }
        let snapshot = Self {
            next_tick: decode_u64(&segment, 2)?,
            committed_event_count: decode_u64(&segment, 3)?,
            authoritative_revision: decode_u64(&segment, 4)?,
            command_ledgers: decode_ledgers(&segment, limits)?,
        };
        if snapshot.canonical_bytes()? != bytes {
            return Err(SnapshotDecodeError::NonCanonicalEncoding);
        }
        Ok(snapshot)
    }

    pub fn command_ledger_hash(&self) -> Result<CommandLedgerHash, CanonicalError> {
        let bytes = self.canonical_bytes()?;
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.command-ledger.v1\0");
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&bytes);
        Ok(command_ledger_hash_from_bytes(sha256(&preimage)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SnapshotDecodeError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Principal(PrincipalDecodeError),
    Command(CommandDecodeError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    FieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedSchemaVersion(u32),
    TooManyLedgers {
        actual: usize,
        limit: usize,
    },
    LedgersNotStrictlySorted,
    CommandIdMismatch,
    LedgerCommandMismatch,
    NonCanonicalEncoding,
}

impl Display for SnapshotDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "snapshot encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "snapshot canonicalization failed: {error}")
            }
            Self::Principal(error) => write!(formatter, "snapshot principal is invalid: {error}"),
            Self::Command(error) => write!(formatter, "snapshot command is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("snapshot envelope does not match"),
            Self::UnknownField(field_id) => write!(formatter, "unknown snapshot field {field_id}"),
            Self::MissingField(field_id) => write!(formatter, "missing snapshot field {field_id}"),
            Self::FieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "snapshot field {field_id} has type {actual}; expected {expected}"
            ),
            Self::FieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "snapshot field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported snapshot schema version {version}")
            }
            Self::TooManyLedgers { actual, limit } => {
                write!(formatter, "snapshot has {actual} ledgers; limit is {limit}")
            }
            Self::LedgersNotStrictlySorted => {
                formatter.write_str("snapshot ledgers are not strictly sorted")
            }
            Self::CommandIdMismatch => {
                formatter.write_str("snapshot ledger command id does not match canonical bytes")
            }
            Self::LedgerCommandMismatch => {
                formatter.write_str("snapshot ledger key does not match its canonical command")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("decoded snapshot does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for SnapshotDecodeError {}

impl From<CanonicalDecodeError> for SnapshotDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for SnapshotDecodeError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<PrincipalDecodeError> for SnapshotDecodeError {
    fn from(error: PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<CommandDecodeError> for SnapshotDecodeError {
    fn from(error: CommandDecodeError) -> Self {
        Self::Command(error)
    }
}

fn validate_snapshot_envelope(
    segment: &DecodedCanonicalSegment,
) -> Result<(), SnapshotDecodeError> {
    if segment.owner_id != RUNTIME_SNAPSHOT_OWNER_ID
        || segment.schema_id != RUNTIME_SNAPSHOT_SCHEMA_ID
        || segment.segment_id != RUNTIME_SNAPSHOT_SEGMENT_ID
    {
        return Err(SnapshotDecodeError::WrongEnvelope);
    }
    Ok(())
}

fn validate_snapshot_fields(segment: &DecodedCanonicalSegment) -> Result<(), SnapshotDecodeError> {
    const EXPECTED: [(u32, u8); 5] = [
        (1, CANONICAL_TYPE_U32),
        (2, CANONICAL_TYPE_U64),
        (3, CANONICAL_TYPE_U64),
        (4, CANONICAL_TYPE_U64),
        (5, CANONICAL_TYPE_SEQUENCE),
    ];
    for field in &segment.fields {
        if !EXPECTED
            .iter()
            .any(|(field_id, _)| *field_id == field.field_id)
        {
            return Err(SnapshotDecodeError::UnknownField(field.field_id));
        }
    }
    for (field_id, type_tag) in EXPECTED {
        let _ = field(segment, field_id, type_tag)?;
    }
    Ok(())
}

fn field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<&CanonicalField, SnapshotDecodeError> {
    let field = segment
        .field(field_id)
        .ok_or(SnapshotDecodeError::MissingField(field_id))?;
    if field.type_tag != expected_type {
        return Err(SnapshotDecodeError::FieldType {
            field_id,
            expected: expected_type,
            actual: field.type_tag,
        });
    }
    Ok(field)
}

fn decode_fixed<const LENGTH: usize>(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<[u8; LENGTH], SnapshotDecodeError> {
    let payload = &field(segment, field_id, expected_type)?.payload;
    payload
        .as_slice()
        .try_into()
        .map_err(|_| SnapshotDecodeError::FieldLength {
            field_id,
            expected: LENGTH,
            actual: payload.len(),
        })
}

fn decode_u32(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u32, SnapshotDecodeError> {
    Ok(u32::from_le_bytes(decode_fixed::<4>(
        segment,
        field_id,
        CANONICAL_TYPE_U32,
    )?))
}

fn decode_u64(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u64, SnapshotDecodeError> {
    Ok(u64::from_le_bytes(decode_fixed::<8>(
        segment,
        field_id,
        CANONICAL_TYPE_U64,
    )?))
}

fn decode_ledgers(
    segment: &DecodedCanonicalSegment,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CommandLedgerSnapshot>, SnapshotDecodeError> {
    let payload = &field(segment, 5, CANONICAL_TYPE_SEQUENCE)?.payload;
    let mut cursor = CanonicalCursor::new(payload);
    let count = usize::try_from(cursor.read_u32()?)
        .map_err(|_| SnapshotDecodeError::Canonical(CanonicalDecodeError::LengthOverflow))?;
    if count > limits.max_sequence_items {
        return Err(SnapshotDecodeError::TooManyLedgers {
            actual: count,
            limit: limits.max_sequence_items,
        });
    }
    let mut ledgers = Vec::with_capacity(count);
    for _ in 0..count {
        let stream_id = CommandStreamId::from_bytes(
            cursor
                .read_exact(16)?
                .try_into()
                .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?,
        );
        let principal_limit = limits.max_identifier_bytes.saturating_add(5).max(17);
        let principal_bytes = cursor.read_u32_length_prefixed(principal_limit)?;
        let issuer = IssuerPrincipal::from_canonical_bytes(principal_bytes, limits)?;
        let last_sequence = cursor.read_u64()?;
        let command_id = CommandId::from_bytes(
            cursor
                .read_exact(16)?
                .try_into()
                .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?,
        );
        let canonical_command_bytes = cursor
            .read_u32_length_prefixed(limits.max_total_bytes)?
            .to_vec();
        let expected = compute_command_id_from_body_bytes(&canonical_command_bytes)?;
        if expected != command_id {
            return Err(SnapshotDecodeError::CommandIdMismatch);
        }
        let command = WorldCommand::from_canonical_bytes(&canonical_command_bytes, limits)?;
        if command.stream_id != stream_id
            || command.issuer != issuer
            || command.sequence != last_sequence
            || command.claimed_command_id != Some(command_id)
        {
            return Err(SnapshotDecodeError::LedgerCommandMismatch);
        }
        ledgers.push(CommandLedgerSnapshot {
            stream_id,
            issuer,
            last_sequence,
            command_id,
            canonical_command_bytes,
        });
    }
    cursor.finish()?;
    if ledgers
        .windows(2)
        .any(|pair| (&pair[0].stream_id, &pair[0].issuer) >= (&pair[1].stream_id, &pair[1].issuer))
    {
        return Err(SnapshotDecodeError::LedgersNotStrictlySorted);
    }
    Ok(ledgers)
}

#[cfg(test)]
mod tests {
    use crate::{
        CanonicalDecodeLimits, CommandId, CommandLedgerSnapshot, CommandStreamId, IssuerPrincipal,
        PlayerPrincipalId, RuntimeSnapshot, WorldCommand,
    };

    fn ledger(stream: u8, issuer: u8, sequence: u64) -> CommandLedgerSnapshot {
        let command = WorldCommand::noop(
            CommandStreamId::from_bytes([stream; 16]),
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16])),
            sequence,
            0,
        )
        .expect("test command is canonical");
        CommandLedgerSnapshot {
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            last_sequence: sequence,
            command_id: command
                .claimed_command_id
                .expect("constructor computes command ID claim"),
            canonical_command_bytes: command.canonical_bytes().expect("canonical command"),
        }
    }

    #[test]
    fn ledger_order_does_not_change_snapshot_bytes() {
        let first = ledger(1, 1, 4);
        let second = ledger(2, 2, 5);
        let ordered = RuntimeSnapshot {
            next_tick: 6,
            committed_event_count: 2,
            authoritative_revision: 2,
            command_ledgers: vec![first.clone(), second.clone()],
        };
        let reversed = RuntimeSnapshot {
            command_ledgers: vec![second, first],
            ..ordered.clone()
        };
        assert_eq!(
            ordered.canonical_bytes().expect("canonical snapshot"),
            reversed.canonical_bytes().expect("canonical snapshot")
        );
    }

    #[test]
    fn snapshot_round_trip_is_byte_exact() {
        let snapshot = RuntimeSnapshot {
            next_tick: 7,
            committed_event_count: 2,
            authoritative_revision: 2,
            command_ledgers: vec![ledger(1, 2, 4), ledger(2, 1, 5)],
        };
        let bytes = snapshot.canonical_bytes().expect("canonical snapshot");
        let restored =
            RuntimeSnapshot::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("snapshot decodes");

        assert_eq!(restored, snapshot);
        assert_eq!(
            restored
                .canonical_bytes()
                .expect("restored canonical bytes"),
            bytes
        );
    }

    #[test]
    fn tampered_ledger_command_is_rejected_without_mutating_input() {
        let snapshot = RuntimeSnapshot {
            next_tick: 1,
            committed_event_count: 1,
            authoritative_revision: 1,
            command_ledgers: vec![ledger(1, 2, 0)],
        };
        let mut bytes = snapshot.canonical_bytes().expect("canonical snapshot");
        let original = bytes.clone();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        let tampered = bytes.clone();

        assert!(
            RuntimeSnapshot::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .is_err()
        );
        assert_eq!(bytes, tampered);
        assert_ne!(bytes, original);
    }

    #[test]
    fn mismatched_declared_command_id_is_rejected() {
        let mut ledger = ledger(1, 2, 0);
        ledger.command_id = CommandId::from_bytes([9; 16]);
        let snapshot = RuntimeSnapshot {
            next_tick: 1,
            committed_event_count: 1,
            authoritative_revision: 1,
            command_ledgers: vec![ledger],
        };
        let bytes = snapshot
            .canonical_bytes()
            .expect("snapshot can encode bad input");
        assert!(matches!(
            RuntimeSnapshot::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default()),
            Err(super::SnapshotDecodeError::CommandIdMismatch)
        ));
    }
}
