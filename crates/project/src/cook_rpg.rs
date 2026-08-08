use std::collections::BTreeMap;

use next_contracts::content::{NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::ids::{CapabilityId, MechanicPackageId, SchemaId};
use next_contracts::mechanics::{
    AbilityDefinitionV1, AbilityTargetKindV1, CooldownSpecV1, DialogueDefinitionV1,
    InteractionDefinitionV1, LockedMechanicPackageV1, MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID,
    MechanicAffordanceV1, MechanicPackageManifestV1, MechanicsLockV1,
    PHYSICS_QUERY_CONTACT_CAPABILITY_ID, QuestDefinitionV1, RelationshipDefinitionV1,
    RpgDefinitionRegistryV1, StateTransitionV1, ability_definition_hash,
    interaction_definition_hash,
};
use next_contracts::rpg::RPG_COMMAND_CAPABILITY_ID;

use crate::cook::{
    CORE_COMBAT_PACKAGE_ID, CORE_INTERACTION_PACKAGE_ID, ProjectCookError, asset_revision,
};

pub(crate) fn compile_rpg_definitions_v1(
    records: &[NeutralRecordV1],
) -> Result<RpgDefinitionRegistryV1, ProjectCookError> {
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
            Ok(InteractionDefinitionV1 {
                asset_revision: asset_revision(interaction_record)?,
                interaction_id: property_id(
                    interaction_record,
                    "nextengine.interaction.definition-id",
                )?,
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
            })
        })
        .collect::<Result<Vec<_>, ProjectCookError>>()?;
    let interaction_hashes = interactions
        .iter()
        .map(interaction_definition_hash)
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
    Ok(RpgDefinitionRegistryV1::new(
        vec![dialogue],
        vec![quest],
        vec![relationship],
        interactions,
        vec![ability],
        vec![interaction_package, combat_package],
        mechanics_lock,
    )?)
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
