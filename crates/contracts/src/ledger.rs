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

mod aggregate;
mod archive;
mod causal;
mod codec;
mod collision;
mod error;
mod hashes;
mod identity_index;
mod receipt;
mod stream;
mod wire;

pub use aggregate::CommandLedgerV2;
pub use archive::{ArchiveInsertResult, CommandBodyArchiveManifestV1, CommandBodyArchiveV1};
pub use causal::{CausalIdentityKey, CausalIdentityKind, CausalIdentityRegistryV1};
pub use collision::{CommandCollisionCandidateV1, CommandCollisionIncidentV1};
pub use error::CommandLedgerError;
pub use hashes::{
    causal_provenance_hash, command_body_archive_root, command_collision_candidates_root,
    command_collision_incident_digest, command_identity_index_root, command_receipt_chain_genesis,
    command_receipt_chain_next, command_receipt_digest,
};
pub use identity_index::{
    CommandIdentityBindingState, CommandIdentityBindingV1, CommandIdentityIndexBodyV1,
    CommandIdentityIndexV1, CommandIdentityOccurrenceV1, IdentityInsertResult,
};
pub use receipt::{CommandFinalResultV1, CommandReceiptSubjectV1, CommandReceiptV1};
pub use stream::{CommandReservationV1, CommandStreamLedgerV2, CommandStreamStateV1};

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

#[cfg(test)]
mod tests;
