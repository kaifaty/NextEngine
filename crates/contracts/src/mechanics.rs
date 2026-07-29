use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::sha256;
use crate::ids::{CapabilityId, ContentHash, MechanicPackageId, SchemaId, content_hash_from_bytes};
use crate::project::AssetRevisionRefV1;

pub const DATA_ONLY_PACKAGE_KIND_V1: &str = "nextengine.mechanic-package.data-only.v1";
pub const MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID: &str = "mechanics.effect.propose";
pub const PHYSICS_QUERY_CONTACT_CAPABILITY_ID: &str = "physics.query.contact";
pub const CORE_CHARACTER_HEALTH_RESOURCE_ID: &str = "nextengine.rpg.resource.health";

mod definitions;
pub use definitions::*;
mod registry;
pub use registry::*;

#[must_use]
pub fn dialogue_definition_hash(definition: &DialogueDefinitionV1) -> ContentHash {
    definition_with_transitions_hash(
        "nextengine.dialogue-definition.v1",
        definition.asset_revision,
        &definition.entry_node_id,
        &definition.transitions,
    )
}

#[must_use]
pub fn quest_definition_hash(definition: &QuestDefinitionV1) -> ContentHash {
    definition_with_transitions_hash(
        "nextengine.quest-definition.v1",
        definition.asset_revision,
        &definition.entry_state_id,
        &definition.transitions,
    )
}

#[must_use]
pub fn relationship_definition_hash(definition: &RelationshipDefinitionV1) -> ContentHash {
    let mut bytes = asset_revision_bytes(definition.asset_revision);
    extend_text(&mut bytes, definition.dimension_id.as_str());
    bytes.extend_from_slice(&definition.minimum_value.to_le_bytes());
    bytes.extend_from_slice(&definition.maximum_value.to_le_bytes());
    domain_hash("nextengine.relationship-definition.v1", &bytes)
}

#[must_use]
pub fn interaction_definition_hash(definition: &InteractionDefinitionV1) -> ContentHash {
    let mut bytes = asset_revision_bytes(definition.asset_revision);
    extend_text(&mut bytes, definition.interaction_id.as_str());
    bytes.extend_from_slice(&asset_revision_bytes(definition.dialogue_definition));
    extend_text(&mut bytes, definition.dialogue_transition_id.as_str());
    bytes.extend_from_slice(&asset_revision_bytes(definition.quest_definition));
    extend_text(&mut bytes, definition.quest_transition_id.as_str());
    bytes.extend_from_slice(&asset_revision_bytes(definition.relationship_definition));
    bytes.extend_from_slice(&definition.relationship_source_value.to_le_bytes());
    bytes.extend_from_slice(&definition.relationship_delta.to_le_bytes());
    domain_hash("nextengine.interaction-definition.v1", &bytes)
}

#[must_use]
pub fn ability_definition_hash(definition: &AbilityDefinitionV1) -> ContentHash {
    let mut bytes = asset_revision_bytes(definition.asset_revision);
    extend_text(&mut bytes, definition.package_id.as_str());
    extend_text(&mut bytes, definition.ability_id.as_str());
    bytes.extend_from_slice(&asset_revision_bytes(definition.required_item_definition));
    extend_text(&mut bytes, definition.required_equipment_slot_id.as_str());
    bytes.push(match definition.target_kind {
        AbilityTargetKindV1::ContactCharacter => 1,
    });
    extend_text(&mut bytes, definition.resource_id.as_str());
    bytes.extend_from_slice(&definition.resource_delta.to_le_bytes());
    bytes.extend_from_slice(&definition.cooldown.duration_ticks.to_le_bytes());
    extend_text(&mut bytes, definition.cooldown.group_id.as_str());
    bytes.extend_from_slice(&(definition.required_capabilities.len() as u32).to_le_bytes());
    for capability in &definition.required_capabilities {
        extend_text(&mut bytes, capability.as_str());
    }
    extend_text(
        &mut bytes,
        definition.affordance.semantic_action_id.as_str(),
    );
    bytes.push(u8::from(definition.affordance.planner_visible));
    bytes.push(u8::from(definition.affordance.requires_equipped_item));
    bytes.push(u8::from(definition.affordance.requires_contact));
    bytes.extend_from_slice(
        &definition
            .affordance
            .expected_resource_delta_minimum
            .to_le_bytes(),
    );
    bytes.extend_from_slice(
        &definition
            .affordance
            .expected_resource_delta_maximum
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&(definition.affordance.failure_modes.len() as u32).to_le_bytes());
    for failure_mode in &definition.affordance.failure_modes {
        extend_text(&mut bytes, failure_mode.as_str());
    }
    domain_hash("nextengine.ability-definition.v1", &bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MechanicsContractError {
    ZeroVersion,
    DuplicateIdentity,
    HashMismatch,
    PackageLockMismatch,
    MissingDefinition,
    MissingTransition,
    InvalidTransitionGraph,
    InvalidBounds,
    CapabilityDenied,
    InvalidAbility,
    InvalidEffect,
    CooldownActive,
}

impl Display for MechanicsContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ZeroVersion => "mechanic package version must be positive",
            Self::DuplicateIdentity => "mechanic identity is duplicated or unordered",
            Self::HashMismatch => "mechanic hash mismatch",
            Self::PackageLockMismatch => "mechanic package/lock mismatch",
            Self::MissingDefinition => "mechanic definition is missing",
            Self::MissingTransition => "mechanic transition is missing",
            Self::InvalidTransitionGraph => "mechanic transition graph is invalid",
            Self::InvalidBounds => "mechanic bounds are invalid",
            Self::CapabilityDenied => "mechanic capability denied",
            Self::InvalidAbility => "mechanic ability is invalid",
            Self::InvalidEffect => "mechanic effect request is invalid",
            Self::CooldownActive => "mechanic cooldown is active",
        })
    }
}

impl Error for MechanicsContractError {}

fn validate_transitions(
    transitions: &[StateTransitionV1],
    entry: &SchemaId,
) -> Result<(), MechanicsContractError> {
    if transitions.is_empty()
        || !strictly_sorted(transitions)
        || !transitions
            .iter()
            .any(|transition| &transition.source_state_id == entry)
    {
        return Err(MechanicsContractError::InvalidTransitionGraph);
    }
    let mut sources = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for transition in transitions {
        if transition.source_state_id == transition.target_state_id
            || !sources.insert(transition.source_state_id.clone())
            || !ids.insert(transition.transition_id.clone())
        {
            return Err(MechanicsContractError::InvalidTransitionGraph);
        }
    }
    Ok(())
}

fn definition_with_transitions_hash(
    domain: &str,
    asset: AssetRevisionRefV1,
    entry: &SchemaId,
    transitions: &[StateTransitionV1],
) -> ContentHash {
    let mut bytes = asset_revision_bytes(asset);
    extend_text(&mut bytes, entry.as_str());
    for transition in transitions {
        extend_text(&mut bytes, transition.transition_id.as_str());
        extend_text(&mut bytes, transition.source_state_id.as_str());
        extend_text(&mut bytes, transition.target_state_id.as_str());
    }
    domain_hash(domain, &bytes)
}

fn asset_revision_bytes(asset: AssetRevisionRefV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(48);
    bytes.extend_from_slice(asset.asset_id.as_bytes());
    bytes.extend_from_slice(asset.record_sha256.as_bytes());
    bytes
}

fn domain_hash(domain: &str, body: &[u8]) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    bytes.extend_from_slice(body);
    content_hash_from_bytes(sha256(&bytes))
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn ensure_unique<T: Ord>(values: &[T]) -> Result<(), MechanicsContractError> {
    if strictly_sorted(values) || values.len() < 2 {
        Ok(())
    } else {
        Err(MechanicsContractError::DuplicateIdentity)
    }
}

fn ensure_unique_by<T>(
    values: &[T],
    key: impl Fn(&T) -> &str,
) -> Result<(), MechanicsContractError> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        Err(MechanicsContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn strictly_sorted_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

#[cfg(test)]
mod tests;
