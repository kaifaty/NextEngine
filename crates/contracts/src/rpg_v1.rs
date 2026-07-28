use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U32, CanonicalCursor, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, decode_canonical_segment,
    encode_canonical_segment, extend_u32_length_prefixed,
};
use crate::{
    AssetId, CommandBodyHash, CommandId, ContentHash, IdentifierError, PersistentId,
    PhysicsContactId, SchemaId, SkillProficiency, content_hash_from_bytes, sha256,
};

pub const RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION: u32 = 2;
pub const RPG_AGGREGATE_SNAPSHOT_OWNER_ID: &str = "rpg";
pub const RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID: &str = "nextengine.rpg.snapshot";
pub const RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID: &str = "domain-state";
pub const RPG_TRANSACTION_PLAN_SCHEMA_ID: &str = "nextengine.rpg-transaction-plan.v1";
pub const RPG_EVENT_DIALOGUE_ADVANCED_SCHEMA_ID: &str = "nextengine.event.rpg.dialogue-advanced.v1";
pub const RPG_EVENT_QUEST_TRANSITIONED_SCHEMA_ID: &str =
    "nextengine.event.rpg.quest-transitioned.v1";
pub const RPG_EVENT_RELATIONSHIP_ADJUSTED_SCHEMA_ID: &str =
    "nextengine.event.rpg.relationship-adjusted.v1";
pub const RPG_EVENT_SKILL_PROFICIENCY_SET_SCHEMA_ID: &str =
    "nextengine.event.rpg.skill-proficiency-set.v1";
pub const RPG_EVENT_ITEM_TRANSFERRED_V1_SCHEMA_ID: &str =
    "nextengine.event.rpg.item-transferred.v1";
pub const RPG_EVENT_EQUIPMENT_ASSIGNED_SCHEMA_ID: &str =
    "nextengine.event.rpg.equipment-assigned.v1";
pub const RPG_EVENT_INTERACTIVE_OBJECT_TRANSITIONED_SCHEMA_ID: &str =
    "nextengine.event.rpg.interactive-object-transitioned.v1";
pub const RPG_EVENT_CHARACTER_RESOURCE_ADJUSTED_SCHEMA_ID: &str =
    "nextengine.event.rpg.character-resource-adjusted.v1";
pub const RPG_TRANSACTION_COMMAND_SCHEMA_VERSION: u32 = 2;
pub const RPG_MAX_OPERATIONS_PER_COMMAND: usize = 64;
pub const RPG_MAX_AGGREGATES_PER_SNAPSHOT: usize = 16_384;
pub const RPG_MAX_COLLECTION_ENTRIES: usize = 4_096;

mod aggregate;
mod codec;
mod command;
mod transaction;

pub use aggregate::{
    CharacterPayloadV1, CharacterResourceEntryV1, DefinitionRefV1, DialoguePayloadV1,
    DivineStandingPayloadV1, EquipmentPayloadV1, EquipmentSlotAssignmentV1,
    FactionMembershipPayloadV1, FactionPayloadV1, InteractiveObjectPayloadV1, InventoryPayloadV1,
    InventoryReservationV1, ItemPayloadV1, ProvenanceBindingV1, QuestPayloadV1,
    RelationshipDimensionV1, RelationshipPayloadV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1,
    RpgAggregatePayloadV1, RpgPhysicalContactFactV1, RpgRuntimeBindingsV1, RpgSnapshotV2,
    SkillProficiencyEntryV1,
};
pub use codec::RpgContractErrorV1;
pub use command::{RpgAggregateRefV1, RpgCommandV1, RpgOperationPayloadV1, RpgOperationV1};
pub use transaction::{
    RpgEventDraftV1, RpgEventV1, RpgReadSetEntryV1, RpgTransactionPlanV1, RpgWriteSetEntryV1,
};

#[cfg(test)]
mod tests;
