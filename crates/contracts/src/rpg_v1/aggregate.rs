use super::codec::*;
use super::*;

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
    pub(super) fn from_tag(tag: u8) -> Result<Self, RpgContractErrorV1> {
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
    pub resources: Vec<CharacterResourceEntryV1>,
    pub skills: Vec<SkillProficiencyEntryV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CharacterResourceEntryV1 {
    pub resource_id: SchemaId,
    pub current_value: i32,
    pub minimum_value: i32,
    pub maximum_value: i32,
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
                extend_count(&mut bytes, payload.resources.len())?;
                for resource in &payload.resources {
                    extend_schema_id(&mut bytes, &resource.resource_id)?;
                    bytes.extend_from_slice(&resource.current_value.to_le_bytes());
                    bytes.extend_from_slice(&resource.minimum_value.to_le_bytes());
                    bytes.extend_from_slice(&resource.maximum_value.to_le_bytes());
                }
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
                resources: read_resources(&mut cursor, limits)?,
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

    pub(super) fn from_canonical_record(
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
