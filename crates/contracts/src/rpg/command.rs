use super::codec::*;
use super::*;

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
    AdjustCharacterResource {
        source_character_id: PersistentId,
        character_id: PersistentId,
        resource_id: SchemaId,
        expected_value: i32,
        delta: i32,
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
            Self::AdjustCharacterResource {
                source_character_id,
                character_id,
                ..
            } => vec![
                (RpgAggregateKindV1::Character, *source_character_id),
                (RpgAggregateKindV1::Character, *character_id),
            ],
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
            Self::AdjustCharacterResource {
                source_character_id,
                character_id,
                resource_id,
                expected_value,
                delta,
            } => {
                bytes.push(8);
                bytes.extend_from_slice(source_character_id.as_bytes());
                bytes.extend_from_slice(character_id.as_bytes());
                extend_schema_id(&mut bytes, resource_id)?;
                bytes.extend_from_slice(&expected_value.to_le_bytes());
                bytes.extend_from_slice(&delta.to_le_bytes());
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
            8 => Self::AdjustCharacterResource {
                source_character_id: read_id(&mut cursor)?,
                character_id: read_id(&mut cursor)?,
                resource_id: read_schema_id(&mut cursor, limits)?,
                expected_value: read_i32(&mut cursor)?,
                delta: read_i32(&mut cursor)?,
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
            } if *expected_value > crate::rpg::SKILL_PROFICIENCY_MAX
                || *new_value > crate::rpg::SKILL_PROFICIENCY_MAX =>
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_SKILL_PROFICIENCY_OUT_OF_RANGE",
                ));
            }
            RpgOperationPayloadV1::AdjustCharacterResource {
                source_character_id,
                character_id,
                delta,
                ..
            } if *delta == 0 || source_character_id == character_id => {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_CHARACTER_RESOURCE_INVALID",
                ));
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
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

    pub(super) fn from_canonical_record(
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
