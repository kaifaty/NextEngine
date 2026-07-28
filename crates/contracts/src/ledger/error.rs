use super::*;

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
