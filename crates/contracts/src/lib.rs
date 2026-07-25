#![forbid(unsafe_code)]

mod canonical;
mod command;
mod ids;
mod ledger;
mod manifest_jcs;
mod persistence;
mod rpg;
mod snapshot;

pub use canonical::{
    CANONICAL_BINARY_V1_MAGIC, CANONICAL_BINARY_V1_VERSION, CANONICAL_TYPE_BOOL,
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_F32_BITS, CANONICAL_TYPE_F64_BITS, CANONICAL_TYPE_HASH256,
    CANONICAL_TYPE_I8, CANONICAL_TYPE_I16, CANONICAL_TYPE_I32, CANONICAL_TYPE_I64,
    CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_SET, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UNIT,
    CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError,
    CanonicalField, DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
    sha256,
};
pub use command::{
    COMMAND_BODY_OWNER_ID, COMMAND_BODY_SCHEMA_ID, COMMAND_BODY_SCHEMA_VERSION,
    COMMAND_BODY_SEGMENT_ID, COMMAND_ENVELOPE_SCHEMA_VERSION, COMMAND_SCHEMA_VERSION,
    CanonicalCommandBodyV2, CapabilityRefV1, CommandDecodeError, CommandPayload, CommandPhase,
    CommandPreconditionV1, DomainEvent, EventPayload, IssuerPrincipal, IssuerPrincipalV2,
    NOOP_COMMAND_CAPABILITY_ID, NOOP_COMMAND_SCHEMA_ID, PrincipalDecodeError, WorldCommand,
    WorldCommandEnvelopeV2, compute_command_id_from_body_bytes,
};
pub use ids::{
    AssetId, CapabilityId, CommandBodyHash, CommandId, CommandLedgerHash, CommandStreamId,
    ContentHash, EventId, IdentifierError, MechanicPackageId, PersistentId, PlayerPrincipalId,
    PluginId, SchemaId, ScriptPrincipalId, StateRoot, SystemId, ToolPrincipalId, WorldNamespaceId,
    command_body_hash_from_bytes, command_ledger_hash_from_bytes, content_hash_from_bytes,
};
pub use ledger::{
    ArchiveInsertResult, CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION,
    COMMAND_BODY_ARCHIVE_SCHEMA_VERSION, COMMAND_IDENTITY_INDEX_BODY_OWNER_ID,
    COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID, COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID,
    COMMAND_IDENTITY_INDEX_SCHEMA_VERSION, COMMAND_LEDGER_SCHEMA_VERSION, COMMAND_PENDING_CAPACITY,
    COMMAND_RECEIPT_OWNER_ID, COMMAND_RECEIPT_SCHEMA_ID, COMMAND_RECEIPT_SCHEMA_VERSION,
    COMMAND_RECEIPT_SEGMENT_ID, COMMAND_RECEIPT_WINDOW_CAPACITY,
    COMMAND_RESERVATION_SCHEMA_VERSION, COMMAND_STREAM_LEDGER_SCHEMA_VERSION, CausalIdentityKey,
    CausalIdentityKind, CausalIdentityRegistryV1, CommandBodyArchiveManifestV1,
    CommandBodyArchiveV1, CommandCollisionCandidateV1, CommandCollisionIncidentV1,
    CommandFinalResultV1, CommandIdentityBindingState, CommandIdentityBindingV1,
    CommandIdentityIndexBodyV1, CommandIdentityIndexV1, CommandIdentityOccurrenceV1,
    CommandLedgerError, CommandLedgerV2, CommandReceiptSubjectV1, CommandReceiptV1,
    CommandReservationV1, CommandStreamLedgerV2, CommandStreamStateV1, IdentityInsertResult,
    command_body_archive_root, command_collision_candidates_root,
    command_collision_incident_digest, command_identity_index_root, command_receipt_chain_genesis,
    command_receipt_chain_next, command_receipt_digest,
};
pub use persistence::{
    AuthorityGrant, CommandLedgerDescriptor, HashBinding, ManifestCodecError,
    ManifestValidationError, REPLAY_MANIFEST_SCHEMA_VERSION, ReplayCommandRecord,
    ReplayComparePoint, ReplayManifestV1, ReplayTickManifest, SAVE_MANIFEST_SCHEMA_VERSION,
    SaveCompatibility, SaveManifestV1, SaveSegmentDescriptor, SchemaBinding, TickSettings,
};
pub use rpg::{
    CharacterSnapshot, DialogueSnapshot, FactionSnapshot, InteractiveObjectSnapshot, ItemSnapshot,
    QuestSnapshot, RPG_COMMAND_CAPABILITY_ID, RPG_COMMAND_SCHEMA_ID,
    RPG_EVENT_DIALOGUE_QUEST_ADVANCED_SCHEMA_ID,
    RPG_EVENT_INTERACTIVE_OBJECT_STATE_CHANGED_SCHEMA_ID, RPG_EVENT_ITEM_TRANSFERRED_SCHEMA_ID,
    RPG_EVENT_SKILL_LEARNED_SCHEMA_ID, RPG_SNAPSHOT_OWNER_ID, RPG_SNAPSHOT_SCHEMA_ID,
    RPG_SNAPSHOT_SCHEMA_VERSION, RPG_SNAPSHOT_SEGMENT_ID, RelationshipEntry, RpgCommand,
    RpgDecodeError, RpgEvent, RpgSnapshot, SKILL_PROFICIENCY_MAX, SkillProficiency,
    SkillProficiencyEntry, SkillProficiencyError, WorldChunkRecordSnapshot,
};
pub use snapshot::{
    CommandLedgerSnapshot, RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID,
    RUNTIME_SNAPSHOT_SCHEMA_VERSION, RUNTIME_SNAPSHOT_SEGMENT_ID, RuntimeSnapshot,
    SnapshotDecodeError,
};
