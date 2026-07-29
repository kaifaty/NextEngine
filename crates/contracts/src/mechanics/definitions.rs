use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateTransitionV1 {
    pub transition_id: SchemaId,
    pub source_state_id: SchemaId,
    pub target_state_id: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogueDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub entry_node_id: SchemaId,
    pub transitions: Vec<StateTransitionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub entry_state_id: SchemaId,
    pub transitions: Vec<StateTransitionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub dimension_id: SchemaId,
    pub minimum_value: i32,
    pub maximum_value: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub interaction_id: SchemaId,
    pub dialogue_definition: AssetRevisionRefV1,
    pub dialogue_transition_id: SchemaId,
    pub quest_definition: AssetRevisionRefV1,
    pub quest_transition_id: SchemaId,
    pub relationship_definition: AssetRevisionRefV1,
    pub relationship_source_value: i32,
    pub relationship_delta: i32,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AbilityTargetKindV1 {
    ContactCharacter,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CooldownSpecV1 {
    pub duration_ticks: u32,
    pub group_id: SchemaId,
}

impl CooldownSpecV1 {
    pub fn validate_boundary(
        &self,
        prior_commit_tick: Option<u64>,
        current_tick: u64,
    ) -> Result<(), MechanicsContractError> {
        if self.duration_ticks == 0 {
            return Err(MechanicsContractError::InvalidBounds);
        }
        if prior_commit_tick.is_some_and(|prior| {
            current_tick < prior.saturating_add(u64::from(self.duration_ticks))
        }) {
            return Err(MechanicsContractError::CooldownActive);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MechanicAffordanceV1 {
    pub semantic_action_id: SchemaId,
    pub planner_visible: bool,
    pub requires_equipped_item: bool,
    pub requires_contact: bool,
    pub expected_resource_delta_minimum: i32,
    pub expected_resource_delta_maximum: i32,
    pub failure_modes: Vec<SchemaId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub package_id: MechanicPackageId,
    pub ability_id: SchemaId,
    pub required_item_definition: AssetRevisionRefV1,
    pub required_equipment_slot_id: SchemaId,
    pub target_kind: AbilityTargetKindV1,
    pub resource_id: SchemaId,
    pub resource_delta: i32,
    pub cooldown: CooldownSpecV1,
    pub required_capabilities: Vec<CapabilityId>,
    pub affordance: MechanicAffordanceV1,
}

impl AbilityDefinitionV1 {
    pub fn validate(&self) -> Result<(), MechanicsContractError> {
        if self.resource_delta == 0
            || self.cooldown.duration_ticks == 0
            || self.required_capabilities.is_empty()
            || !strictly_sorted(&self.required_capabilities)
            || !strictly_sorted(&self.affordance.failure_modes)
            || (self.affordance.planner_visible && self.affordance.failure_modes.is_empty())
            || !self.affordance.requires_equipped_item
            || !self.affordance.requires_contact
            || self.affordance.expected_resource_delta_minimum
                > self.affordance.expected_resource_delta_maximum
            || self.resource_delta < self.affordance.expected_resource_delta_minimum
            || self.resource_delta > self.affordance.expected_resource_delta_maximum
        {
            return Err(MechanicsContractError::InvalidAbility);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EffectRequestV1 {
    pub request_id: ContentHash,
    pub package_id: MechanicPackageId,
    pub ability_id: SchemaId,
    pub ability_definition_hash: ContentHash,
    pub source_character_id: crate::ids::PersistentId,
    pub source_character_revision: u64,
    pub target_character_id: crate::ids::PersistentId,
    pub target_character_revision: u64,
    pub resource_id: SchemaId,
    pub expected_resource_value: i32,
    pub delta: i32,
    pub physical_fact_hash: ContentHash,
    pub gameplay_tick: u64,
}

impl EffectRequestV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the effect proposal identity binds every immutable source, target and physical fact"
    )]
    pub fn new(
        package_id: MechanicPackageId,
        ability_id: SchemaId,
        ability_definition_hash: ContentHash,
        source_character_id: crate::ids::PersistentId,
        source_character_revision: u64,
        target_character_id: crate::ids::PersistentId,
        target_character_revision: u64,
        resource_id: SchemaId,
        expected_resource_value: i32,
        delta: i32,
        physical_fact_hash: ContentHash,
        gameplay_tick: u64,
    ) -> Result<Self, MechanicsContractError> {
        if source_character_id == target_character_id || delta == 0 {
            return Err(MechanicsContractError::InvalidEffect);
        }
        let mut value = Self {
            request_id: ContentHash::default(),
            package_id,
            ability_id,
            ability_definition_hash,
            source_character_id,
            source_character_revision,
            target_character_id,
            target_character_revision,
            resource_id,
            expected_resource_value,
            delta,
            physical_fact_hash,
            gameplay_tick,
        };
        value.request_id = value.computed_id();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), MechanicsContractError> {
        if self.source_character_id == self.target_character_id
            || self.delta == 0
            || self.computed_id() != self.request_id
        {
            return Err(MechanicsContractError::InvalidEffect);
        }
        Ok(())
    }

    fn computed_id(&self) -> ContentHash {
        let mut bytes = Vec::new();
        extend_text(&mut bytes, self.package_id.as_str());
        extend_text(&mut bytes, self.ability_id.as_str());
        bytes.extend_from_slice(self.ability_definition_hash.as_bytes());
        bytes.extend_from_slice(self.source_character_id.as_bytes());
        bytes.extend_from_slice(&self.source_character_revision.to_le_bytes());
        bytes.extend_from_slice(self.target_character_id.as_bytes());
        bytes.extend_from_slice(&self.target_character_revision.to_le_bytes());
        extend_text(&mut bytes, self.resource_id.as_str());
        bytes.extend_from_slice(&self.expected_resource_value.to_le_bytes());
        bytes.extend_from_slice(&self.delta.to_le_bytes());
        bytes.extend_from_slice(self.physical_fact_hash.as_bytes());
        bytes.extend_from_slice(&self.gameplay_tick.to_le_bytes());
        domain_hash("nextengine.effect-request.v1", &bytes)
    }
}
