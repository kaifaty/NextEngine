use std::collections::BTreeMap;

use next_contracts::content::{NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::ids::{AssetId, CapabilityId, ContentHash, MechanicPackageId, SchemaId};
use next_contracts::mechanics::{
    AbilityDefinitionV1, AbilityTargetKindV1, CooldownSpecV1, DialogueDefinitionV1,
    InteractionDefinitionV2, LockedMechanicPackageV1, MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID,
    MechanicAffordanceV1, MechanicPackageManifestV1, MechanicsLockV1,
    PHYSICS_QUERY_CONTACT_CAPABILITY_ID, QuestDefinitionV1, RelationshipDefinitionV1,
    RpgDefinitionRegistryV2, StateTransitionV1, ability_definition_hash,
    interaction_definition_hash_v2,
};
use next_contracts::rpg::RPG_COMMAND_CAPABILITY_ID;
use next_contracts::world_routine::{
    WorldRoutineActivityConditionV1, WorldRoutineActivityV1, WorldRoutineCatalogV1,
    WorldRoutineInteractionBindingV1,
};

use crate::cook::{
    CORE_COMBAT_PACKAGE_ID, CORE_INTERACTION_PACKAGE_ID, ProjectCookError, asset_revision,
};

pub(crate) fn compile_rpg_definitions_v2(
    records: &[NeutralRecordV1],
    routine_binding_or_none: Option<&WorldRoutineInteractionBindingV1>,
) -> Result<RpgDefinitionRegistryV2, ProjectCookError> {
    let mut by_kind = BTreeMap::new();
    let mut interaction_records = Vec::new();
    for record in records {
        if !matches!(
            record.kind,
            NeutralRecordKindV1::DialogueDefinition
                | NeutralRecordKindV1::QuestDefinition
                | NeutralRecordKindV1::RelationshipDefinition
                | NeutralRecordKindV1::InteractionDefinition
                | NeutralRecordKindV1::AbilityDefinition
                | NeutralRecordKindV1::ItemDefinition
        ) {
            continue;
        }
        if record.kind == NeutralRecordKindV1::InteractionDefinition {
            interaction_records.push(record);
            continue;
        }
        if by_kind.insert(record.kind, record).is_some() {
            return Err(ProjectCookError::DuplicateIdentity);
        }
    }
    let dialogue_record = by_kind
        .get(&NeutralRecordKindV1::DialogueDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let quest_record = by_kind
        .get(&NeutralRecordKindV1::QuestDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let relationship_record = by_kind
        .get(&NeutralRecordKindV1::RelationshipDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    if interaction_records.is_empty() {
        return Err(ProjectCookError::MissingReference);
    }
    interaction_records.sort_by_key(|record| record.asset_id);
    let ability_record = by_kind
        .get(&NeutralRecordKindV1::AbilityDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let item_record = by_kind
        .get(&NeutralRecordKindV1::ItemDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let dialogue_revision = asset_revision(dialogue_record)?;
    let quest_revision = asset_revision(quest_record)?;
    let relationship_revision = asset_revision(relationship_record)?;
    let ability_revision = asset_revision(ability_record)?;
    let dialogue_entry = property_id(dialogue_record, "nextengine.dialogue.entry-node")?;
    let quest_entry = property_id(quest_record, "nextengine.quest.entry-state")?;
    let dialogue_transitions = interaction_records
        .iter()
        .map(|record| {
            Ok(StateTransitionV1 {
                transition_id: property_id(record, "nextengine.interaction.dialogue-transition")?,
                source_state_id: property_id(record, "nextengine.interaction.dialogue-source")?,
                target_state_id: property_id(record, "nextengine.interaction.dialogue-target")?,
            })
        })
        .collect::<Result<Vec<_>, ProjectCookError>>()?;
    let quest_transitions = interaction_records
        .iter()
        .map(|record| {
            Ok(StateTransitionV1 {
                transition_id: property_id(record, "nextengine.interaction.quest-transition")?,
                source_state_id: property_id(record, "nextengine.interaction.quest-source")?,
                target_state_id: property_id(record, "nextengine.interaction.quest-target")?,
            })
        })
        .collect::<Result<Vec<_>, ProjectCookError>>()?;
    let dialogue = DialogueDefinitionV1 {
        asset_revision: dialogue_revision,
        entry_node_id: dialogue_entry,
        transitions: dialogue_transitions,
    };
    let quest = QuestDefinitionV1 {
        asset_revision: quest_revision,
        entry_state_id: quest_entry,
        transitions: quest_transitions,
    };
    let relationship = RelationshipDefinitionV1 {
        asset_revision: relationship_revision,
        dimension_id: property_id(relationship_record, "nextengine.relationship.dimension")?,
        minimum_value: -100,
        maximum_value: 100,
    };
    let interactions = interaction_records
        .iter()
        .map(|interaction_record| {
            let dependency_revisions: BTreeMap<_, _> = interaction_record
                .asset_dependencies
                .iter()
                .map(|asset_id| {
                    let record = records
                        .iter()
                        .find(|record| record.asset_id == *asset_id)
                        .ok_or(ProjectCookError::MissingReference)?;
                    Ok((record.kind, asset_revision(record)?))
                })
                .collect::<Result<_, ProjectCookError>>()?;
            let interaction_id =
                property_id(interaction_record, "nextengine.interaction.definition-id")?;
            let availability_condition_or_none = routine_binding_or_none
                .filter(|binding| binding.interaction_id == interaction_id)
                .map(|binding| WorldRoutineActivityConditionV1 {
                    subject_id: binding.subject_id,
                    required_activity: binding.required_activity,
                });
            Ok(InteractionDefinitionV2 {
                asset_revision: asset_revision(interaction_record)?,
                interaction_id,
                dialogue_definition: *dependency_revisions
                    .get(&NeutralRecordKindV1::DialogueDefinition)
                    .ok_or(ProjectCookError::MissingReference)?,
                dialogue_transition_id: property_id(
                    interaction_record,
                    "nextengine.interaction.dialogue-transition",
                )?,
                quest_definition: *dependency_revisions
                    .get(&NeutralRecordKindV1::QuestDefinition)
                    .ok_or(ProjectCookError::MissingReference)?,
                quest_transition_id: property_id(
                    interaction_record,
                    "nextengine.interaction.quest-transition",
                )?,
                relationship_definition: *dependency_revisions
                    .get(&NeutralRecordKindV1::RelationshipDefinition)
                    .ok_or(ProjectCookError::MissingReference)?,
                relationship_source_value: property_i32(
                    interaction_record,
                    "nextengine.interaction.relationship-source",
                )?,
                relationship_delta: property_i32(
                    interaction_record,
                    "nextengine.interaction.relationship-delta",
                )?,
                availability_condition_or_none,
            })
        })
        .collect::<Result<Vec<_>, ProjectCookError>>()?;
    if routine_binding_or_none.is_some_and(|binding| {
        interactions
            .iter()
            .filter(|interaction| interaction.interaction_id == binding.interaction_id)
            .count()
            != 1
    }) {
        return Err(ProjectCookError::MissingReference);
    }
    let interaction_hashes = interactions
        .iter()
        .map(interaction_definition_hash_v2)
        .collect();
    let rpg_capability =
        CapabilityId::new(RPG_COMMAND_CAPABILITY_ID).expect("engine-owned RPG capability is valid");
    let interaction_package = MechanicPackageManifestV1::new(
        MechanicPackageId::new(CORE_INTERACTION_PACKAGE_ID)?,
        1,
        vec![rpg_capability.clone()],
        interaction_hashes,
        Vec::new(),
    )?;
    let effect_capability = CapabilityId::new(MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID)
        .expect("engine-owned effect capability is valid");
    let contact_capability = CapabilityId::new(PHYSICS_QUERY_CONTACT_CAPABILITY_ID)
        .expect("engine-owned contact capability is valid");
    let resource_delta = property_i32(ability_record, "nextengine.ability.resource-delta")?;
    let ability = AbilityDefinitionV1 {
        asset_revision: ability_revision,
        package_id: MechanicPackageId::new(CORE_COMBAT_PACKAGE_ID)?,
        ability_id: property_id(ability_record, "nextengine.ability.definition-id")?,
        required_item_definition: asset_revision(item_record)?,
        required_equipment_slot_id: property_id(
            ability_record,
            "nextengine.ability.equipment-slot",
        )?,
        target_kind: AbilityTargetKindV1::ContactCharacter,
        resource_id: property_id(ability_record, "nextengine.ability.resource")?,
        resource_delta,
        cooldown: CooldownSpecV1 {
            duration_ticks: property_u32(ability_record, "nextengine.ability.cooldown-ticks")?,
            group_id: property_id(ability_record, "nextengine.ability.cooldown-group")?,
        },
        required_capabilities: vec![effect_capability.clone(), contact_capability.clone()],
        affordance: MechanicAffordanceV1 {
            semantic_action_id: property_id(ability_record, "nextengine.ability.semantic-action")?,
            planner_visible: true,
            requires_equipped_item: true,
            requires_contact: true,
            expected_resource_delta_minimum: resource_delta,
            expected_resource_delta_maximum: resource_delta,
            failure_modes: vec![
                SchemaId::new("nextengine.mechanics.failure.contact-required")?,
                SchemaId::new("nextengine.mechanics.failure.cooldown-active")?,
                SchemaId::new("nextengine.mechanics.failure.equipment-required")?,
            ],
        },
    };
    let ability_hash = ability_definition_hash(&ability);
    let combat_package = MechanicPackageManifestV1::new(
        MechanicPackageId::new(CORE_COMBAT_PACKAGE_ID)?,
        1,
        vec![effect_capability.clone(), contact_capability.clone()],
        Vec::new(),
        vec![ability_hash],
    )?;
    let mechanics_lock = MechanicsLockV1::new(vec![
        LockedMechanicPackageV1 {
            package_id: interaction_package.package_id.clone(),
            package_manifest_sha256: interaction_package.package_manifest_sha256,
            granted_capabilities: vec![rpg_capability],
        },
        LockedMechanicPackageV1 {
            package_id: combat_package.package_id.clone(),
            package_manifest_sha256: combat_package.package_manifest_sha256,
            granted_capabilities: vec![effect_capability, contact_capability],
        },
    ])?;
    Ok(RpgDefinitionRegistryV2::new(
        vec![dialogue],
        vec![quest],
        vec![relationship],
        interactions,
        vec![ability],
        vec![interaction_package, combat_package],
        mechanics_lock,
    )?)
}

pub(crate) fn rpg_definitions_v2_manifest_bytes(
    registry: &RpgDefinitionRegistryV2,
) -> Result<Vec<u8>, ProjectCookError> {
    registry.validate()?;
    let mut bytes = b"nextengine.rpg-definitions-v2.v1\0".to_vec();
    bytes.extend_from_slice(registry.registry_sha256.as_bytes());
    bytes.extend_from_slice(
        &u32::try_from(registry.interactions.len())
            .map_err(|_| ProjectCookError::InvalidValue)?
            .to_le_bytes(),
    );
    for interaction in &registry.interactions {
        bytes.extend_from_slice(interaction.asset_revision.asset_id.as_bytes());
        bytes.extend_from_slice(interaction.asset_revision.record_sha256.as_bytes());
        extend_text(&mut bytes, interaction.interaction_id.as_str())?;
        bytes.extend_from_slice(interaction_definition_hash_v2(interaction).as_bytes());
        match interaction.availability_condition_or_none {
            None => bytes.push(0),
            Some(condition) => {
                bytes.push(1);
                bytes.extend_from_slice(condition.subject_id.as_bytes());
                bytes.push(condition.required_activity as u8);
            }
        }
    }
    Ok(bytes)
}

pub(crate) fn activate_rpg_definitions_v2(
    records: &[NeutralRecordV1],
    catalog_or_none: Option<&WorldRoutineCatalogV1>,
    bytes: &[u8],
) -> Result<RpgDefinitionRegistryV2, ProjectCookError> {
    let mut reader = ManifestReader::new(bytes);
    reader.expect(b"nextengine.rpg-definitions-v2.v1\0")?;
    let expected_registry_hash = ContentHash::from_bytes(reader.array()?);
    let count = usize::try_from(reader.u32()?).map_err(|_| ProjectCookError::InvalidValue)?;
    let mut expected = Vec::with_capacity(count);
    let mut routine_binding_or_none = None;
    for _ in 0..count {
        let asset_id = AssetId::from_bytes(reader.array()?);
        let record_sha256 = ContentHash::from_bytes(reader.array()?);
        let interaction_id = SchemaId::new(reader.text()?)?;
        let interaction_hash = ContentHash::from_bytes(reader.array()?);
        let condition = match reader.u8()? {
            0 => None,
            1 => {
                let condition = WorldRoutineActivityConditionV1 {
                    subject_id: next_contracts::ids::PersistentId::from_bytes(reader.array()?),
                    required_activity: match reader.u8()? {
                        1 => WorldRoutineActivityV1::Duty,
                        2 => WorldRoutineActivityV1::Rest,
                        _ => return Err(ProjectCookError::InvalidValue),
                    },
                };
                if routine_binding_or_none
                    .replace(WorldRoutineInteractionBindingV1 {
                        interaction_id: interaction_id.clone(),
                        subject_id: condition.subject_id,
                        required_activity: condition.required_activity,
                    })
                    .is_some()
                {
                    return Err(ProjectCookError::InvalidValue);
                }
                Some(condition)
            }
            _ => return Err(ProjectCookError::InvalidValue),
        };
        expected.push((
            asset_id,
            record_sha256,
            interaction_id,
            interaction_hash,
            condition,
        ));
    }
    reader.finish()?;
    match (catalog_or_none, routine_binding_or_none.as_ref()) {
        (Some(catalog), Some(binding)) => binding
            .validate_against(catalog)
            .map_err(|_| ProjectCookError::InvalidValue)?,
        (None, None) => {}
        _ => return Err(ProjectCookError::InvalidValue),
    }
    let registry = compile_rpg_definitions_v2(records, routine_binding_or_none.as_ref())?;
    if registry.registry_sha256 != expected_registry_hash
        || registry.interactions.len() != expected.len()
        || registry.interactions.iter().zip(expected).any(
            |(
                interaction,
                (asset_id, record_sha256, interaction_id, interaction_hash, condition),
            )| {
                interaction.asset_revision.asset_id != asset_id
                    || interaction.asset_revision.record_sha256 != record_sha256
                    || interaction.interaction_id != interaction_id
                    || interaction_definition_hash_v2(interaction) != interaction_hash
                    || interaction.availability_condition_or_none != condition
            },
        )
    {
        return Err(ProjectCookError::InvalidValue);
    }
    Ok(registry)
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProjectCookError> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| ProjectCookError::InvalidValue)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

struct ManifestReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ManifestReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], ProjectCookError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(ProjectCookError::InvalidValue)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(ProjectCookError::InvalidValue)?;
        self.offset = end;
        Ok(value)
    }

    fn expect(&mut self, expected: &[u8]) -> Result<(), ProjectCookError> {
        if self.take(expected.len())? != expected {
            return Err(ProjectCookError::InvalidValue);
        }
        Ok(())
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], ProjectCookError> {
        self.take(N)?
            .try_into()
            .map_err(|_| ProjectCookError::InvalidValue)
    }

    fn u8(&mut self) -> Result<u8, ProjectCookError> {
        Ok(self.array::<1>()?[0])
    }

    fn u32(&mut self) -> Result<u32, ProjectCookError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn text(&mut self) -> Result<&'a str, ProjectCookError> {
        let length = usize::try_from(self.u32()?).map_err(|_| ProjectCookError::InvalidValue)?;
        std::str::from_utf8(self.take(length)?).map_err(|_| ProjectCookError::InvalidValue)
    }

    fn finish(self) -> Result<(), ProjectCookError> {
        if self.offset != self.bytes.len() {
            return Err(ProjectCookError::InvalidValue);
        }
        Ok(())
    }
}

fn property_id(record: &NeutralRecordV1, property_id: &str) -> Result<SchemaId, ProjectCookError> {
    record
        .properties
        .iter()
        .find(|property| property.property_id.as_str() == property_id)
        .map(|property| property.value_id.clone())
        .ok_or(ProjectCookError::MissingReference)
}

fn property_i32(record: &NeutralRecordV1, property_key: &str) -> Result<i32, ProjectCookError> {
    let value = property_id(record, property_key)?;
    value
        .as_str()
        .strip_prefix("nextengine.value.i32.")
        .ok_or(ProjectCookError::InvalidValue)?
        .parse()
        .map_err(|_| ProjectCookError::InvalidValue)
}

fn property_u32(record: &NeutralRecordV1, property_key: &str) -> Result<u32, ProjectCookError> {
    let value = property_id(record, property_key)?;
    value
        .as_str()
        .strip_prefix("nextengine.value.u32.")
        .ok_or(ProjectCookError::InvalidValue)?
        .parse()
        .map_err(|_| ProjectCookError::InvalidValue)
}
