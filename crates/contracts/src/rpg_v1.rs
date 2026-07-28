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
pub const RPG_TRANSACTION_COMMAND_SCHEMA_VERSION: u32 = 2;
pub const RPG_MAX_OPERATIONS_PER_COMMAND: usize = 64;
pub const RPG_MAX_AGGREGATES_PER_SNAPSHOT: usize = 16_384;
pub const RPG_MAX_COLLECTION_ENTRIES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgRuntimeBindingsV1 {
    pub project_composition_lock_hash: ContentHash,
    pub schema_registry_hash: ContentHash,
    pub budget_policy_hash: ContentHash,
    pub active_definition_policy_hashes: Vec<ContentHash>,
}

impl RpgRuntimeBindingsV1 {
    pub fn validate(&self) -> Result<(), RpgContractErrorV1> {
        if self.active_definition_policy_hashes.len() > RPG_MAX_COLLECTION_ENTRIES
            || !strictly_ordered_unique(&self.active_definition_policy_hashes)
        {
            return Err(RpgContractErrorV1::DefinitionPolicySetInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate().map_err(contract_as_canonical)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"nextengine.rpg-runtime-bindings.v1\0");
        bytes.extend_from_slice(self.project_composition_lock_hash.as_bytes());
        bytes.extend_from_slice(self.schema_registry_hash.as_bytes());
        bytes.extend_from_slice(self.budget_policy_hash.as_bytes());
        extend_hashes(&mut bytes, &self.active_definition_policy_hashes)?;
        Ok(bytes)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, RpgContractErrorV1> {
        const PREFIX: &[u8] = b"nextengine.rpg-runtime-bindings.v1\0";
        let mut cursor = CanonicalCursor::new(bytes);
        if cursor.read_exact(PREFIX.len())? != PREFIX {
            return Err(RpgContractErrorV1::EnvelopeMismatch);
        }
        let bindings = Self {
            project_composition_lock_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
            schema_registry_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
            budget_policy_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
            active_definition_policy_hashes: read_hashes(&mut cursor)?,
        };
        cursor.finish()?;
        bindings.validate()?;
        if bindings.canonical_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(bindings)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgPhysicalContactFactV1 {
    pub gameplay_tick: u64,
    pub contact_id: PhysicsContactId,
    pub subject_low: PersistentId,
    pub subject_high: PersistentId,
    pub physics_checkpoint_revision: u64,
    pub source_snapshot_hash: ContentHash,
    pub contact_batch_hash: ContentHash,
}

impl RpgPhysicalContactFactV1 {
    pub fn validate(self) -> Result<(), RpgContractErrorV1> {
        if self.subject_low >= self.subject_high {
            return Err(RpgContractErrorV1::PhysicalFactInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(self) -> Result<Vec<u8>, CanonicalError> {
        self.validate().map_err(contract_as_canonical)?;
        let mut bytes = b"nextengine.rpg-physical-contact-fact.v1\0".to_vec();
        bytes.extend_from_slice(&self.gameplay_tick.to_le_bytes());
        bytes.extend_from_slice(self.contact_id.as_bytes());
        bytes.extend_from_slice(self.subject_low.as_bytes());
        bytes.extend_from_slice(self.subject_high.as_bytes());
        bytes.extend_from_slice(&self.physics_checkpoint_revision.to_le_bytes());
        bytes.extend_from_slice(self.source_snapshot_hash.as_bytes());
        bytes.extend_from_slice(self.contact_batch_hash.as_bytes());
        Ok(bytes)
    }

    pub fn fact_hash(self) -> Result<ContentHash, CanonicalError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }

    #[must_use]
    pub fn connects(self, first: PersistentId, second: PersistentId) -> bool {
        let (low, high) = if first < second {
            (first, second)
        } else {
            (second, first)
        };
        (self.subject_low, self.subject_high) == (low, high)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum RpgAggregateKindV1 {
    Character = 1,
    Item = 2,
    Inventory = 3,
    Equipment = 4,
    Quest = 5,
    Dialogue = 6,
    Faction = 7,
    FactionMembership = 8,
    Relationship = 9,
    DivineStanding = 10,
    InteractiveObject = 11,
}

impl RpgAggregateKindV1 {
    fn from_tag(tag: u8) -> Result<Self, RpgContractErrorV1> {
        match tag {
            1 => Ok(Self::Character),
            2 => Ok(Self::Item),
            3 => Ok(Self::Inventory),
            4 => Ok(Self::Equipment),
            5 => Ok(Self::Quest),
            6 => Ok(Self::Dialogue),
            7 => Ok(Self::Faction),
            8 => Ok(Self::FactionMembership),
            9 => Ok(Self::Relationship),
            10 => Ok(Self::DivineStanding),
            11 => Ok(Self::InteractiveObject),
            _ => Err(RpgContractErrorV1::UnknownAggregateKind(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DefinitionRefV1 {
    None,
    Exact {
        asset_id: AssetId,
        content_hash: ContentHash,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProvenanceBindingV1 {
    None,
    Exact(ContentHash),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CharacterPayloadV1 {
    pub inventory_id: Option<PersistentId>,
    pub equipment_id: Option<PersistentId>,
    pub skills: Vec<SkillProficiencyEntryV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SkillProficiencyEntryV1 {
    pub skill_id: SchemaId,
    pub proficiency: SkillProficiency,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ItemPayloadV1 {
    pub quantity: u32,
    pub durability: u32,
    pub custom_state: Vec<u8>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InventoryReservationV1 {
    pub reservation_id: PersistentId,
    pub item_id: PersistentId,
    pub quantity: u32,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InventoryPayloadV1 {
    pub owner_id: PersistentId,
    pub capacity: u32,
    pub item_ids: Vec<PersistentId>,
    pub reservations: Vec<InventoryReservationV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EquipmentSlotAssignmentV1 {
    pub slot_id: SchemaId,
    pub item_id: PersistentId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EquipmentPayloadV1 {
    pub character_id: PersistentId,
    pub slot_policy: DefinitionRefV1,
    pub assignments: Vec<EquipmentSlotAssignmentV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QuestPayloadV1 {
    pub state_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DialoguePayloadV1 {
    pub speaker_id: PersistentId,
    pub listener_id: PersistentId,
    pub node_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactionPayloadV1 {
    pub state_id: SchemaId,
    pub directed_policy_refs: Vec<DefinitionRefV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactionMembershipPayloadV1 {
    pub character_id: PersistentId,
    pub faction_id: PersistentId,
    pub state_id: SchemaId,
    pub rank_id: SchemaId,
    pub policy_ref: DefinitionRefV1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationshipDimensionV1 {
    pub dimension_id: SchemaId,
    pub value: i32,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationshipPayloadV1 {
    pub source_id: PersistentId,
    pub target_id: PersistentId,
    pub dimensions: Vec<RelationshipDimensionV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DivineStandingPayloadV1 {
    pub subject_character_id: PersistentId,
    pub favor: i32,
    pub attention: u32,
    pub state_id: SchemaId,
    pub offer_ids: Vec<PersistentId>,
    pub warning_ids: Vec<PersistentId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InteractiveObjectPayloadV1 {
    pub state_id: SchemaId,
    pub linked_item_id: Option<PersistentId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RpgAggregatePayloadV1 {
    Character(CharacterPayloadV1),
    Item(ItemPayloadV1),
    Inventory(InventoryPayloadV1),
    Equipment(EquipmentPayloadV1),
    Quest(QuestPayloadV1),
    Dialogue(DialoguePayloadV1),
    Faction(FactionPayloadV1),
    FactionMembership(FactionMembershipPayloadV1),
    Relationship(RelationshipPayloadV1),
    DivineStanding(DivineStandingPayloadV1),
    InteractiveObject(InteractiveObjectPayloadV1),
}

impl RpgAggregatePayloadV1 {
    #[must_use]
    pub const fn aggregate_kind(&self) -> RpgAggregateKindV1 {
        match self {
            Self::Character(_) => RpgAggregateKindV1::Character,
            Self::Item(_) => RpgAggregateKindV1::Item,
            Self::Inventory(_) => RpgAggregateKindV1::Inventory,
            Self::Equipment(_) => RpgAggregateKindV1::Equipment,
            Self::Quest(_) => RpgAggregateKindV1::Quest,
            Self::Dialogue(_) => RpgAggregateKindV1::Dialogue,
            Self::Faction(_) => RpgAggregateKindV1::Faction,
            Self::FactionMembership(_) => RpgAggregateKindV1::FactionMembership,
            Self::Relationship(_) => RpgAggregateKindV1::Relationship,
            Self::DivineStanding(_) => RpgAggregateKindV1::DivineStanding,
            Self::InteractiveObject(_) => RpgAggregateKindV1::InteractiveObject,
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        bytes.push(self.aggregate_kind() as u8);
        match self {
            Self::Character(payload) => {
                extend_optional_id(&mut bytes, payload.inventory_id);
                extend_optional_id(&mut bytes, payload.equipment_id);
                extend_count(&mut bytes, payload.skills.len())?;
                for skill in &payload.skills {
                    extend_schema_id(&mut bytes, &skill.skill_id)?;
                    bytes.extend_from_slice(&skill.proficiency.get().to_le_bytes());
                }
            }
            Self::Item(payload) => {
                bytes.extend_from_slice(&payload.quantity.to_le_bytes());
                bytes.extend_from_slice(&payload.durability.to_le_bytes());
                extend_u32_length_prefixed(&mut bytes, &payload.custom_state)?;
            }
            Self::Inventory(payload) => {
                bytes.extend_from_slice(payload.owner_id.as_bytes());
                bytes.extend_from_slice(&payload.capacity.to_le_bytes());
                extend_ids(&mut bytes, &payload.item_ids)?;
                extend_count(&mut bytes, payload.reservations.len())?;
                for reservation in &payload.reservations {
                    bytes.extend_from_slice(reservation.reservation_id.as_bytes());
                    bytes.extend_from_slice(reservation.item_id.as_bytes());
                    bytes.extend_from_slice(&reservation.quantity.to_le_bytes());
                }
            }
            Self::Equipment(payload) => {
                bytes.extend_from_slice(payload.character_id.as_bytes());
                extend_definition_ref(&mut bytes, &payload.slot_policy);
                extend_count(&mut bytes, payload.assignments.len())?;
                for assignment in &payload.assignments {
                    extend_schema_id(&mut bytes, &assignment.slot_id)?;
                    bytes.extend_from_slice(assignment.item_id.as_bytes());
                }
            }
            Self::Quest(payload) => extend_schema_id(&mut bytes, &payload.state_id)?,
            Self::Dialogue(payload) => {
                bytes.extend_from_slice(payload.speaker_id.as_bytes());
                bytes.extend_from_slice(payload.listener_id.as_bytes());
                extend_schema_id(&mut bytes, &payload.node_id)?;
            }
            Self::Faction(payload) => {
                extend_schema_id(&mut bytes, &payload.state_id)?;
                extend_count(&mut bytes, payload.directed_policy_refs.len())?;
                for policy_ref in &payload.directed_policy_refs {
                    extend_definition_ref(&mut bytes, policy_ref);
                }
            }
            Self::FactionMembership(payload) => {
                bytes.extend_from_slice(payload.character_id.as_bytes());
                bytes.extend_from_slice(payload.faction_id.as_bytes());
                extend_schema_id(&mut bytes, &payload.state_id)?;
                extend_schema_id(&mut bytes, &payload.rank_id)?;
                extend_definition_ref(&mut bytes, &payload.policy_ref);
            }
            Self::Relationship(payload) => {
                bytes.extend_from_slice(payload.source_id.as_bytes());
                bytes.extend_from_slice(payload.target_id.as_bytes());
                extend_count(&mut bytes, payload.dimensions.len())?;
                for dimension in &payload.dimensions {
                    extend_schema_id(&mut bytes, &dimension.dimension_id)?;
                    bytes.extend_from_slice(&dimension.value.to_le_bytes());
                }
            }
            Self::DivineStanding(payload) => {
                bytes.extend_from_slice(payload.subject_character_id.as_bytes());
                bytes.extend_from_slice(&payload.favor.to_le_bytes());
                bytes.extend_from_slice(&payload.attention.to_le_bytes());
                extend_schema_id(&mut bytes, &payload.state_id)?;
                extend_ids(&mut bytes, &payload.offer_ids)?;
                extend_ids(&mut bytes, &payload.warning_ids)?;
            }
            Self::InteractiveObject(payload) => {
                extend_schema_id(&mut bytes, &payload.state_id)?;
                extend_optional_id(&mut bytes, payload.linked_item_id);
            }
        }
        Ok(bytes)
    }

    fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        let mut cursor = CanonicalCursor::new(bytes);
        let kind = RpgAggregateKindV1::from_tag(cursor.read_u8()?)?;
        let payload = match kind {
            RpgAggregateKindV1::Character => Self::Character(CharacterPayloadV1 {
                inventory_id: read_optional_id(&mut cursor)?,
                equipment_id: read_optional_id(&mut cursor)?,
                skills: read_skills(&mut cursor, limits)?,
            }),
            RpgAggregateKindV1::Item => Self::Item(ItemPayloadV1 {
                quantity: cursor.read_u32()?,
                durability: cursor.read_u32()?,
                custom_state: cursor
                    .read_u32_length_prefixed(limits.max_field_payload_bytes)?
                    .to_vec(),
            }),
            RpgAggregateKindV1::Inventory => Self::Inventory(InventoryPayloadV1 {
                owner_id: read_id(&mut cursor)?,
                capacity: cursor.read_u32()?,
                item_ids: read_ids(&mut cursor, limits)?,
                reservations: read_reservations(&mut cursor, limits)?,
            }),
            RpgAggregateKindV1::Equipment => Self::Equipment(EquipmentPayloadV1 {
                character_id: read_id(&mut cursor)?,
                slot_policy: read_definition_ref(&mut cursor)?,
                assignments: read_assignments(&mut cursor, limits)?,
            }),
            RpgAggregateKindV1::Quest => Self::Quest(QuestPayloadV1 {
                state_id: read_schema_id(&mut cursor, limits)?,
            }),
            RpgAggregateKindV1::Dialogue => Self::Dialogue(DialoguePayloadV1 {
                speaker_id: read_id(&mut cursor)?,
                listener_id: read_id(&mut cursor)?,
                node_id: read_schema_id(&mut cursor, limits)?,
            }),
            RpgAggregateKindV1::Faction => Self::Faction(FactionPayloadV1 {
                state_id: read_schema_id(&mut cursor, limits)?,
                directed_policy_refs: read_definition_refs(&mut cursor)?,
            }),
            RpgAggregateKindV1::FactionMembership => {
                Self::FactionMembership(FactionMembershipPayloadV1 {
                    character_id: read_id(&mut cursor)?,
                    faction_id: read_id(&mut cursor)?,
                    state_id: read_schema_id(&mut cursor, limits)?,
                    rank_id: read_schema_id(&mut cursor, limits)?,
                    policy_ref: read_definition_ref(&mut cursor)?,
                })
            }
            RpgAggregateKindV1::Relationship => Self::Relationship(RelationshipPayloadV1 {
                source_id: read_id(&mut cursor)?,
                target_id: read_id(&mut cursor)?,
                dimensions: read_dimensions(&mut cursor, limits)?,
            }),
            RpgAggregateKindV1::DivineStanding => Self::DivineStanding(DivineStandingPayloadV1 {
                subject_character_id: read_id(&mut cursor)?,
                favor: read_i32(&mut cursor)?,
                attention: cursor.read_u32()?,
                state_id: read_schema_id(&mut cursor, limits)?,
                offer_ids: read_ids(&mut cursor, limits)?,
                warning_ids: read_ids(&mut cursor, limits)?,
            }),
            RpgAggregateKindV1::InteractiveObject => {
                Self::InteractiveObject(InteractiveObjectPayloadV1 {
                    state_id: read_schema_id(&mut cursor, limits)?,
                    linked_item_id: read_optional_id(&mut cursor)?,
                })
            }
        };
        cursor.finish()?;
        if payload.canonical_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(payload)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgAggregateEnvelopeV1 {
    pub aggregate_kind: RpgAggregateKindV1,
    pub persistent_id: PersistentId,
    pub schema_version: u32,
    pub revision: u64,
    pub definition_ref: DefinitionRefV1,
    pub provenance: ProvenanceBindingV1,
    pub payload: RpgAggregatePayloadV1,
    pub payload_hash: ContentHash,
}

impl RpgAggregateEnvelopeV1 {
    pub fn new(
        persistent_id: PersistentId,
        schema_version: u32,
        revision: u64,
        definition_ref: DefinitionRefV1,
        provenance: ProvenanceBindingV1,
        payload: RpgAggregatePayloadV1,
    ) -> Result<Self, RpgContractErrorV1> {
        let payload_hash = content_hash_from_bytes(sha256(&payload.canonical_bytes()?));
        let aggregate = Self {
            aggregate_kind: payload.aggregate_kind(),
            persistent_id,
            schema_version,
            revision,
            definition_ref,
            provenance,
            payload,
            payload_hash,
        };
        aggregate.validate()?;
        Ok(aggregate)
    }

    pub fn validate(&self) -> Result<(), RpgContractErrorV1> {
        if self.schema_version == 0 {
            return Err(RpgContractErrorV1::ZeroSchemaVersion);
        }
        if self.aggregate_kind != self.payload.aggregate_kind() {
            return Err(RpgContractErrorV1::PayloadKindMismatch);
        }
        validate_payload(&self.payload)?;
        let expected_hash = content_hash_from_bytes(sha256(&self.payload.canonical_bytes()?));
        if self.payload_hash != expected_hash {
            return Err(RpgContractErrorV1::PayloadHashMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let payload = self.payload.canonical_bytes()?;
        let mut bytes = Vec::new();
        bytes.push(self.aggregate_kind as u8);
        bytes.extend_from_slice(self.persistent_id.as_bytes());
        bytes.extend_from_slice(&self.schema_version.to_le_bytes());
        bytes.extend_from_slice(&self.revision.to_le_bytes());
        extend_definition_ref(&mut bytes, &self.definition_ref);
        extend_provenance(&mut bytes, &self.provenance);
        extend_u32_length_prefixed(&mut bytes, &payload)?;
        bytes.extend_from_slice(self.payload_hash.as_bytes());
        Ok(bytes)
    }

    pub fn state_hash(&self) -> Result<ContentHash, CanonicalError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }

    fn from_canonical_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        let mut cursor = CanonicalCursor::new(bytes);
        let aggregate_kind = RpgAggregateKindV1::from_tag(cursor.read_u8()?)?;
        let persistent_id = read_id(&mut cursor)?;
        let schema_version = cursor.read_u32()?;
        let revision = cursor.read_u64()?;
        let definition_ref = read_definition_ref(&mut cursor)?;
        let provenance = read_provenance(&mut cursor)?;
        let payload_bytes = cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?;
        let payload_hash = ContentHash::from_bytes(read_array(&mut cursor)?);
        cursor.finish()?;
        let aggregate = Self {
            aggregate_kind,
            persistent_id,
            schema_version,
            revision,
            definition_ref,
            provenance,
            payload: RpgAggregatePayloadV1::from_canonical_bytes(payload_bytes, limits)?,
            payload_hash,
        };
        aggregate.validate()?;
        if aggregate.canonical_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(aggregate)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RpgSnapshotV2 {
    pub aggregates: Vec<RpgAggregateEnvelopeV1>,
}

impl RpgSnapshotV2 {
    pub fn validate(&self) -> Result<(), RpgContractErrorV1> {
        if self.aggregates.len() > RPG_MAX_AGGREGATES_PER_SNAPSHOT {
            return Err(RpgContractErrorV1::CollectionLimitExceeded);
        }
        let mut previous = None;
        for aggregate in &self.aggregates {
            aggregate.validate()?;
            let key = (aggregate.aggregate_kind, aggregate.persistent_id);
            if previous.is_some_and(|value| value >= key) {
                return Err(RpgContractErrorV1::AggregateOrderInvalid);
            }
            previous = Some(key);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate().map_err(contract_as_canonical)?;
        let mut aggregates = Vec::new();
        extend_count(&mut aggregates, self.aggregates.len())?;
        for aggregate in &self.aggregates {
            extend_u32_length_prefixed(&mut aggregates, &aggregate.canonical_bytes()?)?;
        }
        encode_canonical_segment(
            RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_SEQUENCE, aggregates),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != RPG_AGGREGATE_SNAPSHOT_OWNER_ID
            || segment.schema_id != RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID
            || segment.segment_id != RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID
        {
            return Err(RpgContractErrorV1::EnvelopeMismatch);
        }
        if segment.fields.len() != 2
            || segment.fields[0].field_id != 1
            || segment.fields[0].type_tag != CANONICAL_TYPE_U32
            || segment.fields[1].field_id != 2
            || segment.fields[1].type_tag != CANONICAL_TYPE_SEQUENCE
        {
            return Err(RpgContractErrorV1::FieldSetMismatch);
        }
        let version_bytes: [u8; 4] = segment.fields[0]
            .payload
            .as_slice()
            .try_into()
            .map_err(|_| RpgContractErrorV1::FieldSetMismatch)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION {
            return Err(RpgContractErrorV1::UnsupportedSchemaVersion(version));
        }
        let mut cursor = CanonicalCursor::new(&segment.fields[1].payload);
        let count = read_bounded_count(&mut cursor, RPG_MAX_AGGREGATES_PER_SNAPSHOT)?;
        let mut aggregates = Vec::with_capacity(count);
        for _ in 0..count {
            let record = cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?;
            aggregates.push(RpgAggregateEnvelopeV1::from_canonical_record(
                record, limits,
            )?);
        }
        cursor.finish()?;
        let snapshot = Self { aggregates };
        snapshot.validate()?;
        if snapshot.canonical_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(snapshot)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgAggregateRefV1 {
    pub aggregate_kind: RpgAggregateKindV1,
    pub persistent_id: PersistentId,
    pub expected_revision: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RpgOperationPayloadV1 {
    AdvanceDialogue {
        dialogue_id: PersistentId,
        expected_node_id: SchemaId,
        next_node_id: SchemaId,
    },
    TransitionQuest {
        quest_id: PersistentId,
        expected_state_id: SchemaId,
        next_state_id: SchemaId,
    },
    AdjustRelationship {
        relationship_id: PersistentId,
        dimension_id: SchemaId,
        delta: i32,
    },
    SetSkillProficiency {
        character_id: PersistentId,
        skill_id: SchemaId,
        expected_value: u16,
        new_value: u16,
    },
    TransferItem {
        item_id: PersistentId,
        source_inventory_id: Option<PersistentId>,
        destination_inventory_id: Option<PersistentId>,
        quantity: u32,
    },
    AssignEquipment {
        equipment_id: PersistentId,
        inventory_id: PersistentId,
        item_id: PersistentId,
        slot_id: SchemaId,
    },
    TransitionInteractiveObject {
        object_id: PersistentId,
        expected_state_id: SchemaId,
        next_state_id: SchemaId,
    },
}

impl RpgOperationPayloadV1 {
    fn expected_targets(&self) -> Vec<(RpgAggregateKindV1, PersistentId)> {
        let mut targets = match self {
            Self::AdvanceDialogue { dialogue_id, .. } => {
                vec![(RpgAggregateKindV1::Dialogue, *dialogue_id)]
            }
            Self::TransitionQuest { quest_id, .. } => {
                vec![(RpgAggregateKindV1::Quest, *quest_id)]
            }
            Self::AdjustRelationship {
                relationship_id, ..
            } => vec![(RpgAggregateKindV1::Relationship, *relationship_id)],
            Self::SetSkillProficiency { character_id, .. } => {
                vec![(RpgAggregateKindV1::Character, *character_id)]
            }
            Self::TransferItem {
                item_id,
                source_inventory_id,
                destination_inventory_id,
                ..
            } => {
                let mut values = vec![(RpgAggregateKindV1::Item, *item_id)];
                values.extend(
                    [source_inventory_id, destination_inventory_id]
                        .into_iter()
                        .flatten()
                        .map(|id| (RpgAggregateKindV1::Inventory, *id)),
                );
                values
            }
            Self::AssignEquipment {
                equipment_id,
                inventory_id,
                item_id,
                ..
            } => vec![
                (RpgAggregateKindV1::Item, *item_id),
                (RpgAggregateKindV1::Inventory, *inventory_id),
                (RpgAggregateKindV1::Equipment, *equipment_id),
            ],
            Self::TransitionInteractiveObject { object_id, .. } => {
                vec![(RpgAggregateKindV1::InteractiveObject, *object_id)]
            }
        };
        targets.sort_unstable();
        targets.dedup();
        targets
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        match self {
            Self::AdvanceDialogue {
                dialogue_id,
                expected_node_id,
                next_node_id,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(dialogue_id.as_bytes());
                extend_schema_id(&mut bytes, expected_node_id)?;
                extend_schema_id(&mut bytes, next_node_id)?;
            }
            Self::TransitionQuest {
                quest_id,
                expected_state_id,
                next_state_id,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(quest_id.as_bytes());
                extend_schema_id(&mut bytes, expected_state_id)?;
                extend_schema_id(&mut bytes, next_state_id)?;
            }
            Self::AdjustRelationship {
                relationship_id,
                dimension_id,
                delta,
            } => {
                bytes.push(3);
                bytes.extend_from_slice(relationship_id.as_bytes());
                extend_schema_id(&mut bytes, dimension_id)?;
                bytes.extend_from_slice(&delta.to_le_bytes());
            }
            Self::SetSkillProficiency {
                character_id,
                skill_id,
                expected_value,
                new_value,
            } => {
                bytes.push(4);
                bytes.extend_from_slice(character_id.as_bytes());
                extend_schema_id(&mut bytes, skill_id)?;
                bytes.extend_from_slice(&expected_value.to_le_bytes());
                bytes.extend_from_slice(&new_value.to_le_bytes());
            }
            Self::TransferItem {
                item_id,
                source_inventory_id,
                destination_inventory_id,
                quantity,
            } => {
                bytes.push(5);
                bytes.extend_from_slice(item_id.as_bytes());
                extend_optional_id(&mut bytes, *source_inventory_id);
                extend_optional_id(&mut bytes, *destination_inventory_id);
                bytes.extend_from_slice(&quantity.to_le_bytes());
            }
            Self::AssignEquipment {
                equipment_id,
                inventory_id,
                item_id,
                slot_id,
            } => {
                bytes.push(6);
                bytes.extend_from_slice(equipment_id.as_bytes());
                bytes.extend_from_slice(inventory_id.as_bytes());
                bytes.extend_from_slice(item_id.as_bytes());
                extend_schema_id(&mut bytes, slot_id)?;
            }
            Self::TransitionInteractiveObject {
                object_id,
                expected_state_id,
                next_state_id,
            } => {
                bytes.push(7);
                bytes.extend_from_slice(object_id.as_bytes());
                extend_schema_id(&mut bytes, expected_state_id)?;
                extend_schema_id(&mut bytes, next_state_id)?;
            }
        }
        Ok(bytes)
    }

    fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        let mut cursor = CanonicalCursor::new(bytes);
        let payload = match cursor.read_u8()? {
            1 => Self::AdvanceDialogue {
                dialogue_id: read_id(&mut cursor)?,
                expected_node_id: read_schema_id(&mut cursor, limits)?,
                next_node_id: read_schema_id(&mut cursor, limits)?,
            },
            2 => Self::TransitionQuest {
                quest_id: read_id(&mut cursor)?,
                expected_state_id: read_schema_id(&mut cursor, limits)?,
                next_state_id: read_schema_id(&mut cursor, limits)?,
            },
            3 => Self::AdjustRelationship {
                relationship_id: read_id(&mut cursor)?,
                dimension_id: read_schema_id(&mut cursor, limits)?,
                delta: read_i32(&mut cursor)?,
            },
            4 => Self::SetSkillProficiency {
                character_id: read_id(&mut cursor)?,
                skill_id: read_schema_id(&mut cursor, limits)?,
                expected_value: cursor.read_u16()?,
                new_value: cursor.read_u16()?,
            },
            5 => Self::TransferItem {
                item_id: read_id(&mut cursor)?,
                source_inventory_id: read_optional_id(&mut cursor)?,
                destination_inventory_id: read_optional_id(&mut cursor)?,
                quantity: cursor.read_u32()?,
            },
            6 => Self::AssignEquipment {
                equipment_id: read_id(&mut cursor)?,
                inventory_id: read_id(&mut cursor)?,
                item_id: read_id(&mut cursor)?,
                slot_id: read_schema_id(&mut cursor, limits)?,
            },
            7 => Self::TransitionInteractiveObject {
                object_id: read_id(&mut cursor)?,
                expected_state_id: read_schema_id(&mut cursor, limits)?,
                next_state_id: read_schema_id(&mut cursor, limits)?,
            },
            tag => return Err(RpgContractErrorV1::UnknownOperationTag(tag)),
        };
        cursor.finish()?;
        if payload.canonical_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(payload)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgOperationV1 {
    pub operation_slot: u32,
    pub targets: Vec<RpgAggregateRefV1>,
    pub definition_policy_hashes: Vec<ContentHash>,
    pub payload: RpgOperationPayloadV1,
}

impl RpgOperationV1 {
    fn validate(&self) -> Result<(), RpgContractErrorV1> {
        if self.targets.is_empty() || self.targets.len() > RPG_MAX_COLLECTION_ENTRIES {
            return Err(RpgContractErrorV1::TargetSetInvalid);
        }
        if self.targets.windows(2).any(|pair| {
            (pair[0].aggregate_kind, pair[0].persistent_id)
                >= (pair[1].aggregate_kind, pair[1].persistent_id)
        }) {
            return Err(RpgContractErrorV1::TargetSetInvalid);
        }
        if self.definition_policy_hashes.len() > RPG_MAX_COLLECTION_ENTRIES
            || !strictly_ordered_unique(&self.definition_policy_hashes)
        {
            return Err(RpgContractErrorV1::DefinitionPolicySetInvalid);
        }
        if self.payload.expected_targets()
            != self
                .targets
                .iter()
                .map(|target| (target.aggregate_kind, target.persistent_id))
                .collect::<Vec<_>>()
        {
            return Err(RpgContractErrorV1::TargetSetInvalid);
        }
        match &self.payload {
            RpgOperationPayloadV1::TransferItem {
                source_inventory_id,
                destination_inventory_id,
                quantity,
                ..
            } if *quantity == 0
                || (source_inventory_id.is_none() && destination_inventory_id.is_none())
                || source_inventory_id == destination_inventory_id =>
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_OWNERSHIP_CONFLICT",
                ));
            }
            RpgOperationPayloadV1::SetSkillProficiency {
                expected_value,
                new_value,
                ..
            } if *expected_value > crate::SKILL_PROFICIENCY_MAX
                || *new_value > crate::SKILL_PROFICIENCY_MAX =>
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_SKILL_PROFICIENCY_OUT_OF_RANGE",
                ));
            }
            _ => {}
        }
        Ok(())
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.operation_slot.to_le_bytes());
        extend_count(&mut bytes, self.targets.len())?;
        for target in &self.targets {
            bytes.push(target.aggregate_kind as u8);
            bytes.extend_from_slice(target.persistent_id.as_bytes());
            bytes.extend_from_slice(&target.expected_revision.to_le_bytes());
        }
        extend_hashes(&mut bytes, &self.definition_policy_hashes)?;
        extend_u32_length_prefixed(&mut bytes, &self.payload.canonical_bytes()?)?;
        Ok(bytes)
    }

    fn from_canonical_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        let mut cursor = CanonicalCursor::new(bytes);
        let operation_slot = cursor.read_u32()?;
        let count = read_bounded_count(&mut cursor, RPG_MAX_COLLECTION_ENTRIES)?;
        let mut targets = Vec::with_capacity(count);
        for _ in 0..count {
            targets.push(RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::from_tag(cursor.read_u8()?)?,
                persistent_id: read_id(&mut cursor)?,
                expected_revision: cursor.read_u64()?,
            });
        }
        let definition_policy_hashes = read_hashes(&mut cursor)?;
        let payload_bytes = cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?;
        let operation = Self {
            operation_slot,
            targets,
            definition_policy_hashes,
            payload: RpgOperationPayloadV1::from_canonical_bytes(payload_bytes, limits)?,
        };
        cursor.finish()?;
        operation.validate()?;
        if operation.canonical_record()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(operation)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgCommandV1 {
    pub operations: Vec<RpgOperationV1>,
}

impl RpgCommandV1 {
    pub fn validate(&self) -> Result<(), RpgContractErrorV1> {
        if self.operations.is_empty() || self.operations.len() > RPG_MAX_OPERATIONS_PER_COMMAND {
            return Err(RpgContractErrorV1::OperationCountInvalid);
        }
        for (slot, operation) in self.operations.iter().enumerate() {
            if operation.operation_slot != u32::try_from(slot).unwrap_or(u32::MAX) {
                return Err(RpgContractErrorV1::OperationOrderInvalid);
            }
            operation.validate()?;
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate().map_err(contract_as_canonical)?;
        let mut bytes = Vec::new();
        extend_count(&mut bytes, self.operations.len())?;
        for operation in &self.operations {
            extend_u32_length_prefixed(&mut bytes, &operation.canonical_record()?)?;
        }
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        if bytes.len() > limits.max_field_payload_bytes {
            return Err(RpgContractErrorV1::InputTooLarge);
        }
        let mut cursor = CanonicalCursor::new(bytes);
        let count = read_bounded_count(&mut cursor, RPG_MAX_OPERATIONS_PER_COMMAND)?;
        let mut operations = Vec::with_capacity(count);
        for _ in 0..count {
            operations.push(RpgOperationV1::from_canonical_record(
                cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?,
                limits,
            )?);
        }
        cursor.finish()?;
        let command = Self { operations };
        command.validate()?;
        if command.canonical_payload_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(command)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RpgEventV1 {
    DialogueAdvanced {
        dialogue_id: PersistentId,
        node_id: SchemaId,
    },
    QuestTransitioned {
        quest_id: PersistentId,
        state_id: SchemaId,
    },
    RelationshipAdjusted {
        relationship_id: PersistentId,
        dimension_id: SchemaId,
        value: i32,
    },
    SkillProficiencySet {
        character_id: PersistentId,
        skill_id: SchemaId,
        value: SkillProficiency,
    },
    ItemTransferred {
        item_id: PersistentId,
        source_inventory_id: Option<PersistentId>,
        destination_inventory_id: Option<PersistentId>,
        quantity: u32,
    },
    EquipmentAssigned {
        equipment_id: PersistentId,
        item_id: PersistentId,
        slot_id: SchemaId,
    },
    InteractiveObjectTransitioned {
        object_id: PersistentId,
        state_id: SchemaId,
    },
}

impl RpgEventV1 {
    #[must_use]
    pub const fn schema_id(&self) -> &'static str {
        match self {
            Self::DialogueAdvanced { .. } => RPG_EVENT_DIALOGUE_ADVANCED_SCHEMA_ID,
            Self::QuestTransitioned { .. } => RPG_EVENT_QUEST_TRANSITIONED_SCHEMA_ID,
            Self::RelationshipAdjusted { .. } => RPG_EVENT_RELATIONSHIP_ADJUSTED_SCHEMA_ID,
            Self::SkillProficiencySet { .. } => RPG_EVENT_SKILL_PROFICIENCY_SET_SCHEMA_ID,
            Self::ItemTransferred { .. } => RPG_EVENT_ITEM_TRANSFERRED_V1_SCHEMA_ID,
            Self::EquipmentAssigned { .. } => RPG_EVENT_EQUIPMENT_ASSIGNED_SCHEMA_ID,
            Self::InteractiveObjectTransitioned { .. } => {
                RPG_EVENT_INTERACTIVE_OBJECT_TRANSITIONED_SCHEMA_ID
            }
        }
    }

    #[must_use]
    pub const fn primary_aggregate(&self) -> (RpgAggregateKindV1, PersistentId) {
        match self {
            Self::DialogueAdvanced { dialogue_id, .. } => {
                (RpgAggregateKindV1::Dialogue, *dialogue_id)
            }
            Self::QuestTransitioned { quest_id, .. } => (RpgAggregateKindV1::Quest, *quest_id),
            Self::RelationshipAdjusted {
                relationship_id, ..
            } => (RpgAggregateKindV1::Relationship, *relationship_id),
            Self::SkillProficiencySet { character_id, .. } => {
                (RpgAggregateKindV1::Character, *character_id)
            }
            Self::ItemTransferred { item_id, .. } => (RpgAggregateKindV1::Item, *item_id),
            Self::EquipmentAssigned { equipment_id, .. } => {
                (RpgAggregateKindV1::Equipment, *equipment_id)
            }
            Self::InteractiveObjectTransitioned { object_id, .. } => {
                (RpgAggregateKindV1::InteractiveObject, *object_id)
            }
        }
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        match self {
            Self::DialogueAdvanced {
                dialogue_id,
                node_id,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(dialogue_id.as_bytes());
                extend_schema_id(&mut bytes, node_id)?;
            }
            Self::QuestTransitioned { quest_id, state_id } => {
                bytes.push(2);
                bytes.extend_from_slice(quest_id.as_bytes());
                extend_schema_id(&mut bytes, state_id)?;
            }
            Self::RelationshipAdjusted {
                relationship_id,
                dimension_id,
                value,
            } => {
                bytes.push(3);
                bytes.extend_from_slice(relationship_id.as_bytes());
                extend_schema_id(&mut bytes, dimension_id)?;
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            Self::SkillProficiencySet {
                character_id,
                skill_id,
                value,
            } => {
                bytes.push(4);
                bytes.extend_from_slice(character_id.as_bytes());
                extend_schema_id(&mut bytes, skill_id)?;
                bytes.extend_from_slice(&value.get().to_le_bytes());
            }
            Self::ItemTransferred {
                item_id,
                source_inventory_id,
                destination_inventory_id,
                quantity,
            } => {
                bytes.push(5);
                bytes.extend_from_slice(item_id.as_bytes());
                extend_optional_id(&mut bytes, *source_inventory_id);
                extend_optional_id(&mut bytes, *destination_inventory_id);
                bytes.extend_from_slice(&quantity.to_le_bytes());
            }
            Self::EquipmentAssigned {
                equipment_id,
                item_id,
                slot_id,
            } => {
                bytes.push(6);
                bytes.extend_from_slice(equipment_id.as_bytes());
                bytes.extend_from_slice(item_id.as_bytes());
                extend_schema_id(&mut bytes, slot_id)?;
            }
            Self::InteractiveObjectTransitioned {
                object_id,
                state_id,
            } => {
                bytes.push(7);
                bytes.extend_from_slice(object_id.as_bytes());
                extend_schema_id(&mut bytes, state_id)?;
            }
        }
        Ok(bytes)
    }

    fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        let mut cursor = CanonicalCursor::new(bytes);
        let event = match cursor.read_u8()? {
            1 => Self::DialogueAdvanced {
                dialogue_id: read_id(&mut cursor)?,
                node_id: read_schema_id(&mut cursor, limits)?,
            },
            2 => Self::QuestTransitioned {
                quest_id: read_id(&mut cursor)?,
                state_id: read_schema_id(&mut cursor, limits)?,
            },
            3 => Self::RelationshipAdjusted {
                relationship_id: read_id(&mut cursor)?,
                dimension_id: read_schema_id(&mut cursor, limits)?,
                value: read_i32(&mut cursor)?,
            },
            4 => Self::SkillProficiencySet {
                character_id: read_id(&mut cursor)?,
                skill_id: read_schema_id(&mut cursor, limits)?,
                value: SkillProficiency::new(cursor.read_u16()?).map_err(|_| {
                    RpgContractErrorV1::PayloadInvariant("RPG_SKILL_PROFICIENCY_OUT_OF_RANGE")
                })?,
            },
            5 => Self::ItemTransferred {
                item_id: read_id(&mut cursor)?,
                source_inventory_id: read_optional_id(&mut cursor)?,
                destination_inventory_id: read_optional_id(&mut cursor)?,
                quantity: cursor.read_u32()?,
            },
            6 => Self::EquipmentAssigned {
                equipment_id: read_id(&mut cursor)?,
                item_id: read_id(&mut cursor)?,
                slot_id: read_schema_id(&mut cursor, limits)?,
            },
            7 => Self::InteractiveObjectTransitioned {
                object_id: read_id(&mut cursor)?,
                state_id: read_schema_id(&mut cursor, limits)?,
            },
            tag => return Err(RpgContractErrorV1::UnknownEventTag(tag)),
        };
        cursor.finish()?;
        if event.canonical_payload_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(event)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgReadSetEntryV1 {
    pub aggregate_ref: RpgAggregateRefV1,
    pub state_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgWriteSetEntryV1 {
    pub aggregate_kind: RpgAggregateKindV1,
    pub persistent_id: PersistentId,
    pub before_revision: u64,
    pub after_revision: u64,
    pub before_state_hash: ContentHash,
    pub after_state_hash: ContentHash,
    pub after: RpgAggregateEnvelopeV1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgEventDraftV1 {
    pub operation_slot: u32,
    pub event_local_slot: u16,
    pub event_schema_id: SchemaId,
    pub primary_aggregate_kind: RpgAggregateKindV1,
    pub primary_persistent_id: PersistentId,
    pub event: RpgEventV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgTransactionPlanV1 {
    pub causal_command_id: CommandId,
    pub canonical_command_body_hash: CommandBodyHash,
    pub project_composition_lock_hash: ContentHash,
    pub schema_registry_hash: ContentHash,
    pub definition_policy_hashes: Vec<ContentHash>,
    pub validated_fact_hashes: Vec<ContentHash>,
    pub ordered_operations: Vec<RpgOperationV1>,
    pub ordered_read_set: Vec<RpgReadSetEntryV1>,
    pub ordered_write_set: Vec<RpgWriteSetEntryV1>,
    pub ordered_event_drafts: Vec<RpgEventDraftV1>,
    pub budget_policy_hash: ContentHash,
    pub plan_hash: ContentHash,
}

impl RpgTransactionPlanV1 {
    pub fn canonical_body_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(RPG_TRANSACTION_PLAN_SCHEMA_ID.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.causal_command_id.as_bytes());
        bytes.extend_from_slice(self.canonical_command_body_hash.as_bytes());
        bytes.extend_from_slice(self.project_composition_lock_hash.as_bytes());
        bytes.extend_from_slice(self.schema_registry_hash.as_bytes());
        extend_hashes(&mut bytes, &self.definition_policy_hashes)?;
        extend_hashes(&mut bytes, &self.validated_fact_hashes)?;
        extend_count(&mut bytes, self.ordered_operations.len())?;
        for operation in &self.ordered_operations {
            extend_u32_length_prefixed(&mut bytes, &operation.canonical_record()?)?;
        }
        extend_count(&mut bytes, self.ordered_read_set.len())?;
        for entry in &self.ordered_read_set {
            bytes.push(entry.aggregate_ref.aggregate_kind as u8);
            bytes.extend_from_slice(entry.aggregate_ref.persistent_id.as_bytes());
            bytes.extend_from_slice(&entry.aggregate_ref.expected_revision.to_le_bytes());
            bytes.extend_from_slice(entry.state_hash.as_bytes());
        }
        extend_count(&mut bytes, self.ordered_write_set.len())?;
        for entry in &self.ordered_write_set {
            bytes.push(entry.aggregate_kind as u8);
            bytes.extend_from_slice(entry.persistent_id.as_bytes());
            bytes.extend_from_slice(&entry.before_revision.to_le_bytes());
            bytes.extend_from_slice(&entry.after_revision.to_le_bytes());
            bytes.extend_from_slice(entry.before_state_hash.as_bytes());
            bytes.extend_from_slice(entry.after_state_hash.as_bytes());
            extend_u32_length_prefixed(&mut bytes, &entry.after.canonical_bytes()?)?;
        }
        extend_count(&mut bytes, self.ordered_event_drafts.len())?;
        for draft in &self.ordered_event_drafts {
            bytes.extend_from_slice(&draft.operation_slot.to_le_bytes());
            bytes.extend_from_slice(&draft.event_local_slot.to_le_bytes());
            extend_schema_id(&mut bytes, &draft.event_schema_id)?;
            bytes.push(draft.primary_aggregate_kind as u8);
            bytes.extend_from_slice(draft.primary_persistent_id.as_bytes());
            extend_u32_length_prefixed(&mut bytes, &draft.event.canonical_payload_bytes()?)?;
        }
        bytes.extend_from_slice(self.budget_policy_hash.as_bytes());
        Ok(bytes)
    }

    pub fn validate(&self) -> Result<(), RpgContractErrorV1> {
        if self.ordered_operations.is_empty()
            || self.ordered_operations.len() > RPG_MAX_OPERATIONS_PER_COMMAND
        {
            return Err(RpgContractErrorV1::OperationCountInvalid);
        }
        RpgCommandV1 {
            operations: self.ordered_operations.clone(),
        }
        .validate()?;
        if self.validated_fact_hashes.len() > RPG_MAX_COLLECTION_ENTRIES
            || !strictly_ordered_unique(&self.definition_policy_hashes)
            || (!self.validated_fact_hashes.is_empty()
                && !strictly_ordered_unique(&self.validated_fact_hashes))
            || !strictly_ordered_by(&self.ordered_read_set, |entry| {
                (
                    entry.aggregate_ref.aggregate_kind,
                    entry.aggregate_ref.persistent_id,
                )
            })
            || !strictly_ordered_by(&self.ordered_write_set, |entry| {
                (entry.aggregate_kind, entry.persistent_id)
            })
            || !strictly_ordered_by(&self.ordered_event_drafts, |entry| {
                (
                    entry.operation_slot,
                    entry.event_local_slot,
                    entry.event_schema_id.clone(),
                    entry.primary_aggregate_kind,
                    entry.primary_persistent_id,
                )
            })
        {
            return Err(RpgContractErrorV1::PlanOrderInvalid);
        }
        for write in &self.ordered_write_set {
            write.after.validate()?;
            let read = self
                .ordered_read_set
                .iter()
                .find(|read| {
                    read.aggregate_ref.aggregate_kind == write.aggregate_kind
                        && read.aggregate_ref.persistent_id == write.persistent_id
                })
                .ok_or(RpgContractErrorV1::PlanWriteInvalid)?;
            let computed_after_hash = write.after.state_hash()?;
            if write.aggregate_kind != write.after.aggregate_kind
                || write.persistent_id != write.after.persistent_id
                || write.after_revision
                    != write
                        .before_revision
                        .checked_add(1)
                        .ok_or(RpgContractErrorV1::RevisionExhausted)?
                || write.after.revision != write.after_revision
                || read.aggregate_ref.expected_revision != write.before_revision
                || read.state_hash != write.before_state_hash
                || computed_after_hash != write.after_state_hash
            {
                return Err(RpgContractErrorV1::PlanWriteInvalid);
            }
        }
        let mut expected_hashes = self
            .ordered_operations
            .iter()
            .flat_map(|operation| operation.definition_policy_hashes.iter().copied())
            .collect::<Vec<_>>();
        expected_hashes.sort_unstable();
        expected_hashes.dedup();
        if expected_hashes != self.definition_policy_hashes {
            return Err(RpgContractErrorV1::DefinitionPolicySetInvalid);
        }
        validate_event_drafts(&self.ordered_operations, &self.ordered_event_drafts)?;
        let expected_hash = content_hash_from_bytes(sha256(&self.canonical_body_bytes()?));
        if self.plan_hash != expected_hash {
            return Err(RpgContractErrorV1::PlanHashMismatch);
        }
        Ok(())
    }

    pub fn recompute_plan_hash(&self) -> Result<ContentHash, CanonicalError> {
        Ok(content_hash_from_bytes(sha256(
            &self.canonical_body_bytes()?,
        )))
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate().map_err(contract_as_canonical)?;
        let body = self.canonical_body_bytes()?;
        let mut bytes = Vec::new();
        extend_u32_length_prefixed(&mut bytes, &body)?;
        bytes.extend_from_slice(self.plan_hash.as_bytes());
        Ok(bytes)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgContractErrorV1> {
        if bytes.len() > limits.max_total_bytes {
            return Err(RpgContractErrorV1::InputTooLarge);
        }
        let mut outer = CanonicalCursor::new(bytes);
        let body = outer.read_u32_length_prefixed(limits.max_field_payload_bytes)?;
        let plan_hash = ContentHash::from_bytes(read_array(&mut outer)?);
        outer.finish()?;

        let mut cursor = CanonicalCursor::new(body);
        let domain = cursor.read_exact(RPG_TRANSACTION_PLAN_SCHEMA_ID.len() + 1)?;
        if domain[..RPG_TRANSACTION_PLAN_SCHEMA_ID.len()]
            != *RPG_TRANSACTION_PLAN_SCHEMA_ID.as_bytes()
            || domain[RPG_TRANSACTION_PLAN_SCHEMA_ID.len()] != 0
        {
            return Err(RpgContractErrorV1::EnvelopeMismatch);
        }
        let causal_command_id = CommandId::from_bytes(read_array(&mut cursor)?);
        let canonical_command_body_hash = CommandBodyHash::from_bytes(read_array(&mut cursor)?);
        let project_composition_lock_hash = ContentHash::from_bytes(read_array(&mut cursor)?);
        let schema_registry_hash = ContentHash::from_bytes(read_array(&mut cursor)?);
        let definition_policy_hashes = read_hashes(&mut cursor)?;
        let validated_fact_hashes = read_hashes(&mut cursor)?;

        let operation_count = read_bounded_count(&mut cursor, RPG_MAX_OPERATIONS_PER_COMMAND)?;
        let mut ordered_operations = Vec::with_capacity(operation_count);
        for _ in 0..operation_count {
            ordered_operations.push(RpgOperationV1::from_canonical_record(
                cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?,
                limits,
            )?);
        }

        let read_count = read_bounded_count(&mut cursor, RPG_MAX_AGGREGATES_PER_SNAPSHOT)?;
        let mut ordered_read_set = Vec::with_capacity(read_count);
        for _ in 0..read_count {
            ordered_read_set.push(RpgReadSetEntryV1 {
                aggregate_ref: RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::from_tag(cursor.read_u8()?)?,
                    persistent_id: read_id(&mut cursor)?,
                    expected_revision: cursor.read_u64()?,
                },
                state_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
            });
        }

        let write_count = read_bounded_count(&mut cursor, RPG_MAX_AGGREGATES_PER_SNAPSHOT)?;
        let mut ordered_write_set = Vec::with_capacity(write_count);
        for _ in 0..write_count {
            ordered_write_set.push(RpgWriteSetEntryV1 {
                aggregate_kind: RpgAggregateKindV1::from_tag(cursor.read_u8()?)?,
                persistent_id: read_id(&mut cursor)?,
                before_revision: cursor.read_u64()?,
                after_revision: cursor.read_u64()?,
                before_state_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
                after_state_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
                after: RpgAggregateEnvelopeV1::from_canonical_record(
                    cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?,
                    limits,
                )?,
            });
        }

        let event_count = read_bounded_count(
            &mut cursor,
            RPG_MAX_OPERATIONS_PER_COMMAND * RPG_MAX_COLLECTION_ENTRIES,
        )?;
        let mut ordered_event_drafts = Vec::with_capacity(event_count);
        for _ in 0..event_count {
            ordered_event_drafts.push(RpgEventDraftV1 {
                operation_slot: cursor.read_u32()?,
                event_local_slot: cursor.read_u16()?,
                event_schema_id: read_schema_id(&mut cursor, limits)?,
                primary_aggregate_kind: RpgAggregateKindV1::from_tag(cursor.read_u8()?)?,
                primary_persistent_id: read_id(&mut cursor)?,
                event: RpgEventV1::from_canonical_payload_bytes(
                    cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?,
                    limits,
                )?,
            });
        }
        let budget_policy_hash = ContentHash::from_bytes(read_array(&mut cursor)?);
        cursor.finish()?;

        let plan = Self {
            causal_command_id,
            canonical_command_body_hash,
            project_composition_lock_hash,
            schema_registry_hash,
            definition_policy_hashes,
            validated_fact_hashes,
            ordered_operations,
            ordered_read_set,
            ordered_write_set,
            ordered_event_drafts,
            budget_policy_hash,
            plan_hash,
        };
        plan.validate()?;
        if plan.canonical_bytes()? != bytes {
            return Err(RpgContractErrorV1::NonCanonicalEncoding);
        }
        Ok(plan)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgContractErrorV1 {
    Canonical(CanonicalDecodeError),
    Canonicalize(CanonicalError),
    Identifier(IdentifierError),
    UnsupportedSchemaVersion(u32),
    UnknownAggregateKind(u8),
    UnknownOperationTag(u8),
    UnknownEventTag(u8),
    InputTooLarge,
    EnvelopeMismatch,
    FieldSetMismatch,
    NonCanonicalEncoding,
    ZeroSchemaVersion,
    PayloadKindMismatch,
    PayloadHashMismatch,
    CollectionLimitExceeded,
    AggregateOrderInvalid,
    OperationCountInvalid,
    OperationOrderInvalid,
    TargetSetInvalid,
    DefinitionPolicySetInvalid,
    PhysicalFactInvalid,
    InvalidTag(u8),
    PayloadInvariant(&'static str),
    PlanOrderInvalid,
    PlanWriteInvalid,
    PlanHashMismatch,
    EventOrderInvalid,
    RevisionExhausted,
}

impl RpgContractErrorV1 {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion(_) => "RPG_SCHEMA_UNSUPPORTED",
            Self::UnknownAggregateKind(_) | Self::PayloadKindMismatch => {
                "RPG_AGGREGATE_KIND_INVALID"
            }
            Self::UnknownOperationTag(_) | Self::OperationCountInvalid => "RPG_OPERATION_INVALID",
            Self::UnknownEventTag(_) => "RPG_EVENT_ORDER_INVALID",
            Self::InputTooLarge | Self::CollectionLimitExceeded => "RPG_INPUT_LIMIT_EXCEEDED",
            Self::EnvelopeMismatch | Self::FieldSetMismatch => "RPG_SCHEMA_INVALID",
            Self::NonCanonicalEncoding => "RPG_NON_CANONICAL",
            Self::ZeroSchemaVersion => "RPG_SCHEMA_INVALID",
            Self::PayloadHashMismatch => "RPG_PAYLOAD_HASH_MISMATCH",
            Self::AggregateOrderInvalid | Self::PlanOrderInvalid => "RPG_ORDER_INVALID",
            Self::OperationOrderInvalid => "RPG_OPERATION_ORDER_INVALID",
            Self::TargetSetInvalid => "RPG_TARGET_SET_INVALID",
            Self::DefinitionPolicySetInvalid => "RPG_DEFINITION_MISMATCH",
            Self::PhysicalFactInvalid => "RPG_PHYSICAL_PRECONDITION_MISSING",
            Self::InvalidTag(_) => "RPG_SCHEMA_INVALID",
            Self::PayloadInvariant(code) => code,
            Self::PlanWriteInvalid => "RPG_PLAN_WRITE_INVALID",
            Self::PlanHashMismatch => "RPG_PLAN_HASH_MISMATCH",
            Self::EventOrderInvalid => "RPG_EVENT_ORDER_INVALID",
            Self::RevisionExhausted => "RPG_REVISION_EXHAUSTED",
            Self::Canonical(_) | Self::Canonicalize(_) | Self::Identifier(_) => {
                "RPG_CANONICALIZATION_FAILED"
            }
        }
    }
}

impl Display for RpgContractErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "{}: {version}", self.stable_code())
            }
            Self::UnknownAggregateKind(tag)
            | Self::UnknownOperationTag(tag)
            | Self::UnknownEventTag(tag) => {
                write!(formatter, "{}: {tag}", self.stable_code())
            }
            Self::InvalidTag(tag) => write!(formatter, "{}: {tag}", self.stable_code()),
            Self::Canonical(error) => write!(formatter, "{}: {error}", self.stable_code()),
            Self::Canonicalize(error) => write!(formatter, "{}: {error}", self.stable_code()),
            Self::Identifier(error) => write!(formatter, "{}: {error}", self.stable_code()),
            _ => formatter.write_str(self.stable_code()),
        }
    }
}

impl Error for RpgContractErrorV1 {}

impl From<CanonicalDecodeError> for RpgContractErrorV1 {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for RpgContractErrorV1 {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalize(error)
    }
}

impl From<IdentifierError> for RpgContractErrorV1 {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

fn validate_payload(payload: &RpgAggregatePayloadV1) -> Result<(), RpgContractErrorV1> {
    match payload {
        RpgAggregatePayloadV1::Character(payload) => {
            if !strictly_ordered_by(&payload.skills, |entry| entry.skill_id.clone()) {
                return Err(RpgContractErrorV1::PayloadInvariant("RPG_DUPLICATE_SKILL"));
            }
        }
        RpgAggregatePayloadV1::Item(payload) => {
            if payload.quantity == 0 {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_ITEM_QUANTITY_ZERO",
                ));
            }
        }
        RpgAggregatePayloadV1::Inventory(payload) => {
            if payload.item_ids.len() > RPG_MAX_COLLECTION_ENTRIES
                || payload.reservations.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_unique(&payload.item_ids)
                || !strictly_ordered_by(&payload.reservations, |entry| entry.reservation_id)
                || payload.reservations.iter().any(|entry| entry.quantity == 0)
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_INVENTORY_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Equipment(payload) => {
            if payload.assignments.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_by(&payload.assignments, |entry| entry.slot_id.clone())
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_EQUIPMENT_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Faction(payload) => {
            if payload.directed_policy_refs.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_unique(&payload.directed_policy_refs)
            {
                return Err(RpgContractErrorV1::PayloadInvariant("RPG_FACTION_INVALID"));
            }
        }
        RpgAggregatePayloadV1::FactionMembership(payload) => {
            if payload.character_id == payload.faction_id {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_FACTION_MEMBERSHIP_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Relationship(payload) => {
            if payload.source_id == payload.target_id
                || !strictly_ordered_by(&payload.dimensions, |entry| entry.dimension_id.clone())
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_RELATIONSHIP_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::DivineStanding(payload) => {
            if payload.offer_ids.len() > RPG_MAX_COLLECTION_ENTRIES
                || payload.warning_ids.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_unique(&payload.offer_ids)
                || !strictly_ordered_unique(&payload.warning_ids)
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_DIVINE_STANDING_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Quest(_)
        | RpgAggregatePayloadV1::Dialogue(_)
        | RpgAggregatePayloadV1::InteractiveObject(_) => {}
    }
    Ok(())
}

fn validate_event_drafts(
    operations: &[RpgOperationV1],
    drafts: &[RpgEventDraftV1],
) -> Result<(), RpgContractErrorV1> {
    for operation in operations {
        let operation_drafts = drafts
            .iter()
            .filter(|draft| draft.operation_slot == operation.operation_slot)
            .collect::<Vec<_>>();
        if operation_drafts.is_empty() {
            return Err(RpgContractErrorV1::EventOrderInvalid);
        }
        for (expected_slot, draft) in operation_drafts.into_iter().enumerate() {
            let expected_slot =
                u16::try_from(expected_slot).map_err(|_| RpgContractErrorV1::EventOrderInvalid)?;
            let primary = draft.event.primary_aggregate();
            if draft.event_local_slot != expected_slot
                || draft.event_schema_id.as_str() != draft.event.schema_id()
                || (draft.primary_aggregate_kind, draft.primary_persistent_id) != primary
                || !operation
                    .targets
                    .iter()
                    .any(|target| (target.aggregate_kind, target.persistent_id) == primary)
            {
                return Err(RpgContractErrorV1::EventOrderInvalid);
            }
        }
    }
    if drafts.iter().any(|draft| {
        usize::try_from(draft.operation_slot).map_or(true, |slot| slot >= operations.len())
    }) {
        return Err(RpgContractErrorV1::EventOrderInvalid);
    }
    Ok(())
}

fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), CanonicalError> {
    let count = u32::try_from(count).map_err(|_| CanonicalError::LengthOverflow)?;
    bytes.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn extend_schema_id(bytes: &mut Vec<u8>, value: &SchemaId) -> Result<(), CanonicalError> {
    extend_u32_length_prefixed(bytes, value.as_str().as_bytes())
}

fn extend_ids(bytes: &mut Vec<u8>, values: &[PersistentId]) -> Result<(), CanonicalError> {
    extend_count(bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(value.as_bytes());
    }
    Ok(())
}

fn extend_hashes(bytes: &mut Vec<u8>, values: &[ContentHash]) -> Result<(), CanonicalError> {
    extend_count(bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(value.as_bytes());
    }
    Ok(())
}

fn extend_definition_ref(bytes: &mut Vec<u8>, value: &DefinitionRefV1) {
    match value {
        DefinitionRefV1::None => bytes.push(0),
        DefinitionRefV1::Exact {
            asset_id,
            content_hash,
        } => {
            bytes.push(1);
            bytes.extend_from_slice(asset_id.as_bytes());
            bytes.extend_from_slice(content_hash.as_bytes());
        }
    }
}

fn extend_provenance(bytes: &mut Vec<u8>, value: &ProvenanceBindingV1) {
    match value {
        ProvenanceBindingV1::None => bytes.push(0),
        ProvenanceBindingV1::Exact(value) => {
            bytes.push(1);
            bytes.extend_from_slice(value.as_bytes());
        }
    }
}

fn extend_optional_id(bytes: &mut Vec<u8>, value: Option<PersistentId>) {
    match value {
        None => bytes.push(0),
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(value.as_bytes());
        }
    }
}

fn read_id(cursor: &mut CanonicalCursor<'_>) -> Result<PersistentId, CanonicalDecodeError> {
    Ok(PersistentId::from_bytes(read_array(cursor)?))
}

fn read_array<const N: usize>(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<[u8; N], CanonicalDecodeError> {
    cursor
        .read_exact(N)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)
}

fn read_schema_id(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, RpgContractErrorV1> {
    let bytes = cursor.read_u32_length_prefixed(limits.max_identifier_bytes)?;
    let text = std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?;
    Ok(SchemaId::new(text)?)
}

fn read_optional_id(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Option<PersistentId>, RpgContractErrorV1> {
    match cursor.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(read_id(cursor)?)),
        tag => Err(RpgContractErrorV1::InvalidTag(tag)),
    }
}

fn read_provenance(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<ProvenanceBindingV1, RpgContractErrorV1> {
    match cursor.read_u8()? {
        0 => Ok(ProvenanceBindingV1::None),
        1 => Ok(ProvenanceBindingV1::Exact(ContentHash::from_bytes(
            read_array(cursor)?,
        ))),
        tag => Err(RpgContractErrorV1::InvalidTag(tag)),
    }
}

fn read_definition_ref(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<DefinitionRefV1, RpgContractErrorV1> {
    match cursor.read_u8()? {
        0 => Ok(DefinitionRefV1::None),
        1 => Ok(DefinitionRefV1::Exact {
            asset_id: AssetId::from_bytes(read_array(cursor)?),
            content_hash: ContentHash::from_bytes(read_array(cursor)?),
        }),
        tag => Err(RpgContractErrorV1::InvalidTag(tag)),
    }
}

fn read_definition_refs(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Vec<DefinitionRefV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count).map(|_| read_definition_ref(cursor)).collect()
}

fn read_hashes(cursor: &mut CanonicalCursor<'_>) -> Result<Vec<ContentHash>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| Ok(ContentHash::from_bytes(read_array(cursor)?)))
        .collect()
}

fn read_ids(
    cursor: &mut CanonicalCursor<'_>,
    _limits: CanonicalDecodeLimits,
) -> Result<Vec<PersistentId>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count).map(|_| Ok(read_id(cursor)?)).collect()
}

fn read_skills(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<SkillProficiencyEntryV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(SkillProficiencyEntryV1 {
                skill_id: read_schema_id(cursor, limits)?,
                proficiency: SkillProficiency::new(cursor.read_u16()?).map_err(|_| {
                    RpgContractErrorV1::PayloadInvariant("RPG_SKILL_PROFICIENCY_OUT_OF_RANGE")
                })?,
            })
        })
        .collect()
}

fn read_reservations(
    cursor: &mut CanonicalCursor<'_>,
    _limits: CanonicalDecodeLimits,
) -> Result<Vec<InventoryReservationV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(InventoryReservationV1 {
                reservation_id: read_id(cursor)?,
                item_id: read_id(cursor)?,
                quantity: cursor.read_u32()?,
            })
        })
        .collect()
}

fn read_assignments(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<EquipmentSlotAssignmentV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(EquipmentSlotAssignmentV1 {
                slot_id: read_schema_id(cursor, limits)?,
                item_id: read_id(cursor)?,
            })
        })
        .collect()
}

fn read_dimensions(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<RelationshipDimensionV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(RelationshipDimensionV1 {
                dimension_id: read_schema_id(cursor, limits)?,
                value: read_i32(cursor)?,
            })
        })
        .collect()
}

fn read_i32(cursor: &mut CanonicalCursor<'_>) -> Result<i32, CanonicalDecodeError> {
    Ok(i32::from_le_bytes(read_array(cursor)?))
}

fn read_bounded_count(
    cursor: &mut CanonicalCursor<'_>,
    limit: usize,
) -> Result<usize, CanonicalDecodeError> {
    cursor.read_count(limit, |actual, limit| CanonicalDecodeError::InputTooLarge {
        actual,
        limit,
    })
}

fn strictly_ordered_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn strictly_ordered_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn contract_as_canonical(error: RpgContractErrorV1) -> CanonicalError {
    match error {
        RpgContractErrorV1::Canonicalize(error) => error,
        _ => CanonicalError::LengthOverflow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(byte: u8) -> PersistentId {
        PersistentId::from_bytes([byte; 16])
    }

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test schema ID is valid")
    }

    fn definition(byte: u8) -> DefinitionRefV1 {
        DefinitionRefV1::Exact {
            asset_id: AssetId::from_bytes([byte; 16]),
            content_hash: ContentHash::from_bytes([byte; 32]),
        }
    }

    fn aggregate(id_byte: u8, payload: RpgAggregatePayloadV1) -> RpgAggregateEnvelopeV1 {
        RpgAggregateEnvelopeV1::new(
            id(id_byte),
            1,
            0,
            definition(id_byte),
            ProvenanceBindingV1::Exact(ContentHash::from_bytes([id_byte; 32])),
            payload,
        )
        .expect("test aggregate is valid")
    }

    #[test]
    fn aggregate_snapshot_round_trip_is_byte_exact() {
        let snapshot = RpgSnapshotV2 {
            aggregates: vec![
                aggregate(
                    1,
                    RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                        inventory_id: Some(id(3)),
                        equipment_id: Some(id(4)),
                        skills: vec![],
                    }),
                ),
                aggregate(
                    2,
                    RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                        quantity: 1,
                        durability: 100,
                        custom_state: Vec::new(),
                    }),
                ),
                aggregate(
                    3,
                    RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                        owner_id: id(1),
                        capacity: 8,
                        item_ids: vec![id(2)],
                        reservations: vec![],
                    }),
                ),
                aggregate(
                    4,
                    RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                        character_id: id(1),
                        slot_policy: definition(9),
                        assignments: vec![],
                    }),
                ),
                aggregate(
                    5,
                    RpgAggregatePayloadV1::Relationship(RelationshipPayloadV1 {
                        source_id: id(1),
                        target_id: id(6),
                        dimensions: vec![RelationshipDimensionV1 {
                            dimension_id: schema("nextengine.relationship.trust"),
                            value: 7,
                        }],
                    }),
                ),
            ],
        };

        let bytes = snapshot.canonical_bytes().expect("snapshot encodes");
        let decoded = RpgSnapshotV2::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("snapshot decodes");
        assert_eq!(decoded, snapshot);
        assert_eq!(decoded.canonical_bytes().expect("re-encodes"), bytes);
    }

    #[test]
    fn snapshot_requires_canonical_aggregate_order() {
        let mut snapshot = RpgSnapshotV2 {
            aggregates: vec![
                aggregate(
                    2,
                    RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                        quantity: 1,
                        durability: 1,
                        custom_state: Vec::new(),
                    }),
                ),
                aggregate(
                    1,
                    RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                        inventory_id: None,
                        equipment_id: None,
                        skills: vec![],
                    }),
                ),
            ],
        };
        assert_eq!(
            snapshot.validate(),
            Err(RpgContractErrorV1::AggregateOrderInvalid)
        );
        snapshot.aggregates.sort();
        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn command_rejects_operation_gap_and_unsorted_targets() {
        let command = RpgCommandV1 {
            operations: vec![RpgOperationV1 {
                operation_slot: 1,
                targets: vec![RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::Dialogue,
                    persistent_id: id(1),
                    expected_revision: 0,
                }],
                definition_policy_hashes: vec![],
                payload: RpgOperationPayloadV1::AdvanceDialogue {
                    dialogue_id: id(1),
                    expected_node_id: schema("nextengine.dialogue.offer"),
                    next_node_id: schema("nextengine.dialogue.accepted"),
                },
            }],
        };
        assert_eq!(
            command.validate(),
            Err(RpgContractErrorV1::OperationOrderInvalid)
        );

        let mut unsorted = command;
        unsorted.operations[0].operation_slot = 0;
        unsorted.operations[0].targets = vec![
            RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Dialogue,
                persistent_id: id(1),
                expected_revision: 0,
            },
            RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Quest,
                persistent_id: id(2),
                expected_revision: 0,
            },
        ];
        assert_eq!(
            unsorted.validate(),
            Err(RpgContractErrorV1::TargetSetInvalid)
        );
    }

    #[test]
    fn command_round_trip_is_byte_exact() {
        let command = RpgCommandV1 {
            operations: vec![
                RpgOperationV1 {
                    operation_slot: 0,
                    targets: vec![RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Dialogue,
                        persistent_id: id(1),
                        expected_revision: 0,
                    }],
                    definition_policy_hashes: vec![],
                    payload: RpgOperationPayloadV1::AdvanceDialogue {
                        dialogue_id: id(1),
                        expected_node_id: schema("nextengine.dialogue.offer"),
                        next_node_id: schema("nextengine.dialogue.accepted"),
                    },
                },
                RpgOperationV1 {
                    operation_slot: 1,
                    targets: vec![RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Quest,
                        persistent_id: id(2),
                        expected_revision: 0,
                    }],
                    definition_policy_hashes: vec![],
                    payload: RpgOperationPayloadV1::TransitionQuest {
                        quest_id: id(2),
                        expected_state_id: schema("nextengine.quest.available"),
                        next_state_id: schema("nextengine.quest.active"),
                    },
                },
            ],
        };

        let bytes = command.canonical_payload_bytes().expect("command encodes");
        let decoded =
            RpgCommandV1::from_canonical_payload_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("command decodes");
        assert_eq!(decoded, command);
        assert_eq!(
            decoded.canonical_payload_bytes().expect("re-encodes"),
            bytes
        );
    }

    #[test]
    fn payload_hash_mismatch_is_fail_closed() {
        let mut aggregate = aggregate(
            2,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 1,
                custom_state: Vec::new(),
            }),
        );
        aggregate.payload_hash = ContentHash::from_bytes([0xff; 32]);
        assert_eq!(
            aggregate.validate(),
            Err(RpgContractErrorV1::PayloadHashMismatch)
        );
    }

    fn dialogue_plan() -> RpgTransactionPlanV1 {
        let before = aggregate(
            7,
            RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                speaker_id: id(1),
                listener_id: id(2),
                node_id: schema("nextengine.dialogue.offer"),
            }),
        );
        let after = RpgAggregateEnvelopeV1::new(
            id(7),
            1,
            1,
            definition(7),
            ProvenanceBindingV1::Exact(ContentHash::from_bytes([7; 32])),
            RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                speaker_id: id(1),
                listener_id: id(2),
                node_id: schema("nextengine.dialogue.accepted"),
            }),
        )
        .expect("after aggregate is valid");
        let policy_hash = ContentHash::from_bytes([8; 32]);
        let operation = RpgOperationV1 {
            operation_slot: 0,
            targets: vec![RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Dialogue,
                persistent_id: id(7),
                expected_revision: 0,
            }],
            definition_policy_hashes: vec![policy_hash],
            payload: RpgOperationPayloadV1::AdvanceDialogue {
                dialogue_id: id(7),
                expected_node_id: schema("nextengine.dialogue.offer"),
                next_node_id: schema("nextengine.dialogue.accepted"),
            },
        };
        let event = RpgEventV1::DialogueAdvanced {
            dialogue_id: id(7),
            node_id: schema("nextengine.dialogue.accepted"),
        };
        let before_state_hash = before.state_hash().expect("before hashes");
        let after_state_hash = after.state_hash().expect("after hashes");
        let mut plan = RpgTransactionPlanV1 {
            causal_command_id: CommandId::from_bytes([1; 16]),
            canonical_command_body_hash: CommandBodyHash::from_bytes([2; 32]),
            project_composition_lock_hash: ContentHash::from_bytes([3; 32]),
            schema_registry_hash: ContentHash::from_bytes([4; 32]),
            definition_policy_hashes: vec![policy_hash],
            validated_fact_hashes: vec![],
            ordered_operations: vec![operation],
            ordered_read_set: vec![RpgReadSetEntryV1 {
                aggregate_ref: RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::Dialogue,
                    persistent_id: id(7),
                    expected_revision: 0,
                },
                state_hash: before_state_hash,
            }],
            ordered_write_set: vec![RpgWriteSetEntryV1 {
                aggregate_kind: RpgAggregateKindV1::Dialogue,
                persistent_id: id(7),
                before_revision: 0,
                after_revision: 1,
                before_state_hash,
                after_state_hash,
                after,
            }],
            ordered_event_drafts: vec![RpgEventDraftV1 {
                operation_slot: 0,
                event_local_slot: 0,
                event_schema_id: schema(event.schema_id()),
                primary_aggregate_kind: RpgAggregateKindV1::Dialogue,
                primary_persistent_id: id(7),
                event,
            }],
            budget_policy_hash: ContentHash::from_bytes([5; 32]),
            plan_hash: ContentHash::from_bytes([0; 32]),
        };
        plan.plan_hash = plan.recompute_plan_hash().expect("plan hashes");
        plan
    }

    #[test]
    fn transaction_plan_round_trip_is_byte_exact() {
        let plan = dialogue_plan();
        let bytes = plan.canonical_bytes().expect("plan encodes");
        let decoded =
            RpgTransactionPlanV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("plan decodes");
        assert_eq!(decoded, plan);
        assert_eq!(decoded.canonical_bytes().expect("plan re-encodes"), bytes);
    }

    #[test]
    fn transaction_plan_rejects_tampered_hash_and_write_binding() {
        let mut plan = dialogue_plan();
        plan.plan_hash = ContentHash::from_bytes([9; 32]);
        assert_eq!(plan.validate(), Err(RpgContractErrorV1::PlanHashMismatch));

        let mut plan = dialogue_plan();
        plan.ordered_write_set[0].after_revision = 2;
        plan.plan_hash = plan.recompute_plan_hash().expect("tampered plan hashes");
        assert_eq!(plan.validate(), Err(RpgContractErrorV1::PlanWriteInvalid));
    }

    #[test]
    fn transaction_plan_rejects_event_schema_or_slot_mismatch() {
        let mut plan = dialogue_plan();
        plan.ordered_event_drafts[0].event_local_slot = 1;
        plan.plan_hash = plan.recompute_plan_hash().expect("tampered plan hashes");
        assert_eq!(plan.validate(), Err(RpgContractErrorV1::EventOrderInvalid));

        let mut plan = dialogue_plan();
        plan.ordered_event_drafts[0].event_schema_id = schema("nextengine.event.rpg.wrong.v1");
        plan.plan_hash = plan.recompute_plan_hash().expect("tampered plan hashes");
        assert_eq!(plan.validate(), Err(RpgContractErrorV1::EventOrderInvalid));
    }

    #[test]
    fn physical_contact_fact_is_canonical_and_subject_ordered() {
        let fact = RpgPhysicalContactFactV1 {
            gameplay_tick: 7,
            contact_id: PhysicsContactId::from_bytes([1; 16]),
            subject_low: id(1),
            subject_high: id(2),
            physics_checkpoint_revision: 9,
            source_snapshot_hash: ContentHash::from_bytes([3; 32]),
            contact_batch_hash: ContentHash::from_bytes([4; 32]),
        };
        assert_eq!(fact.validate(), Ok(()));
        assert_ne!(
            fact.fact_hash().expect("fact hashes"),
            ContentHash::default()
        );
        assert!(fact.connects(id(2), id(1)));

        let invalid = RpgPhysicalContactFactV1 {
            subject_low: id(2),
            subject_high: id(1),
            ..fact
        };
        assert_eq!(
            invalid.validate(),
            Err(RpgContractErrorV1::PhysicalFactInvalid)
        );
    }
}
