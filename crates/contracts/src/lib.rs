#![forbid(unsafe_code)]

mod canonical;
mod command;
mod ids;
mod manifest_jcs;
mod persistence;
mod rpg;
mod snapshot;

pub use canonical::{
    CANONICAL_BINARY_V1_MAGIC, CANONICAL_BINARY_V1_VERSION, CANONICAL_TYPE_BYTES,
    CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U8, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError,
    CanonicalField, DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
    sha256,
};
pub use command::{
    COMMAND_SCHEMA_VERSION, CommandDecodeError, CommandPayload, CommandPhase, DomainEvent,
    EventPayload, IssuerPrincipal, NOOP_COMMAND_CAPABILITY_ID, NOOP_COMMAND_SCHEMA_ID,
    PrincipalDecodeError, WorldCommand, compute_command_id_from_canonical,
};
pub use ids::{
    AssetId, CapabilityId, CommandId, CommandLedgerHash, CommandStreamId, ContentHash, EventId,
    IdentifierError, MechanicPackageId, PersistentId, PlayerPrincipalId, PluginId, SchemaId,
    StateRoot, SystemId, WorldNamespaceId, command_ledger_hash_from_bytes, content_hash_from_bytes,
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
