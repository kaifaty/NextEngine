use super::codec::*;
use super::*;

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
    CharacterResourceAdjusted {
        character_id: PersistentId,
        resource_id: SchemaId,
        value: i32,
    },
    CommitmentTransitioned {
        commitment_id: PersistentId,
        state: CommitmentStateV1,
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
            Self::CharacterResourceAdjusted { .. } => {
                RPG_EVENT_CHARACTER_RESOURCE_ADJUSTED_SCHEMA_ID
            }
            Self::CommitmentTransitioned { .. } => RPG_EVENT_COMMITMENT_TRANSITIONED_SCHEMA_ID,
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
            Self::CharacterResourceAdjusted { character_id, .. } => {
                (RpgAggregateKindV1::Character, *character_id)
            }
            Self::CommitmentTransitioned { commitment_id, .. } => {
                (RpgAggregateKindV1::Commitment, *commitment_id)
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
            Self::CharacterResourceAdjusted {
                character_id,
                resource_id,
                value,
            } => {
                bytes.push(8);
                bytes.extend_from_slice(character_id.as_bytes());
                extend_schema_id(&mut bytes, resource_id)?;
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            Self::CommitmentTransitioned {
                commitment_id,
                state,
            } => {
                bytes.push(9);
                bytes.extend_from_slice(commitment_id.as_bytes());
                bytes.push(*state as u8);
            }
        }
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
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
            8 => Self::CharacterResourceAdjusted {
                character_id: read_id(&mut cursor)?,
                resource_id: read_schema_id(&mut cursor, limits)?,
                value: read_i32(&mut cursor)?,
            },
            9 => Self::CommitmentTransitioned {
                commitment_id: read_id(&mut cursor)?,
                state: CommitmentStateV1::from_tag(cursor.read_u8()?)?,
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
