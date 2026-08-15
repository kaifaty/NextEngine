#![deny(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::ids::{PersistentId, SchemaId};
use next_contracts::mechanics::{
    AbilityDefinitionV1, AbilityTargetKindV1, EffectRequestV1, MechanicsContractError,
    RpgDefinitionRegistryV2, ability_definition_hash,
};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::rpg::{
    DefinitionRefV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1,
    RpgAggregateRefV1, RpgCommandV1, RpgContractErrorV1, RpgOperationPayloadV1, RpgOperationV1,
    RpgPhysicalContactFactV1, RpgSnapshotV2,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityInvocationV1 {
    pub gameplay_tick: u64,
    pub source_character_id: PersistentId,
    pub semantic_action_id: SchemaId,
    pub prior_cooldown_commit_tick: Option<u64>,
    pub physical_contact_facts: Vec<RpgPhysicalContactFactV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledAbilityEffectV1 {
    pub ability_definition_hash: next_contracts::ids::ContentHash,
    pub effect_request: EffectRequestV1,
    pub rpg_command: RpgCommandV1,
}

pub fn compile_contact_ability_v1(
    registry: &RpgDefinitionRegistryV2,
    snapshot: &RpgSnapshotV2,
    mut invocation: AbilityInvocationV1,
) -> Result<CompiledAbilityEffectV1, MechanicsHostError> {
    registry.validate()?;
    snapshot.validate()?;
    invocation.physical_contact_facts.sort_unstable();
    invocation.physical_contact_facts.dedup();

    let source = aggregate(
        snapshot,
        RpgAggregateKindV1::Character,
        invocation.source_character_id,
    )
    .ok_or(MechanicsHostError::SourceCharacterMissing)?;
    let source_character = character(source)?;
    let inventory_id = source_character
        .inventory_id
        .ok_or(MechanicsHostError::EquipmentRequired)?;
    let equipment_id = source_character
        .equipment_id
        .ok_or(MechanicsHostError::EquipmentRequired)?;
    let inventory = aggregate(snapshot, RpgAggregateKindV1::Inventory, inventory_id)
        .ok_or(MechanicsHostError::EquipmentRequired)?;
    let equipment = aggregate(snapshot, RpgAggregateKindV1::Equipment, equipment_id)
        .ok_or(MechanicsHostError::EquipmentRequired)?;
    let inventory_payload = match &inventory.payload {
        RpgAggregatePayloadV1::Inventory(payload) => payload,
        _ => return Err(MechanicsHostError::AggregateKindMismatch),
    };
    let equipment_payload = match &equipment.payload {
        RpgAggregatePayloadV1::Equipment(payload) => payload,
        _ => return Err(MechanicsHostError::AggregateKindMismatch),
    };

    let (ability, item) = registry
        .abilities
        .iter()
        .filter(|ability| ability.affordance.semantic_action_id == invocation.semantic_action_id)
        .filter_map(|ability| {
            let assignment = equipment_payload
                .assignments
                .iter()
                .find(|assignment| assignment.slot_id == ability.required_equipment_slot_id)?;
            if inventory_payload
                .item_ids
                .binary_search(&assignment.item_id)
                .is_err()
            {
                return None;
            }
            let item = aggregate(snapshot, RpgAggregateKindV1::Item, assignment.item_id)?;
            if exact_definition_revision(item) != Some(ability.required_item_definition) {
                return None;
            }
            Some((ability, item))
        })
        .next()
        .ok_or(MechanicsHostError::EquipmentRequired)?;
    validate_package_grants(registry, ability)?;
    ability
        .cooldown
        .validate_boundary(
            invocation.prior_cooldown_commit_tick,
            invocation.gameplay_tick,
        )
        .map_err(|error| match error {
            MechanicsContractError::CooldownActive => MechanicsHostError::CooldownActive,
            other => MechanicsHostError::Contract(other),
        })?;

    let (target, contact_fact) = invocation
        .physical_contact_facts
        .iter()
        .filter(|fact| fact.gameplay_tick == invocation.gameplay_tick)
        .filter(|fact| {
            fact.connects(
                invocation.source_character_id,
                if fact.subject_low == invocation.source_character_id {
                    fact.subject_high
                } else {
                    fact.subject_low
                },
            )
        })
        .filter_map(|fact| {
            let other = if fact.subject_low == invocation.source_character_id {
                fact.subject_high
            } else if fact.subject_high == invocation.source_character_id {
                fact.subject_low
            } else {
                return None;
            };
            let target = aggregate(snapshot, RpgAggregateKindV1::Character, other)?;
            Some((target, fact))
        })
        .next()
        .ok_or(MechanicsHostError::ContactRequired)?;
    if ability.target_kind != AbilityTargetKindV1::ContactCharacter {
        return Err(MechanicsHostError::TargetInvalid);
    }
    contact_fact.validate()?;
    let target_character = character(target)?;
    let resource = target_character
        .resources
        .binary_search_by(|entry| entry.resource_id.cmp(&ability.resource_id))
        .ok()
        .map(|index| &target_character.resources[index])
        .ok_or(MechanicsHostError::TargetResourceMissing)?;
    let next_value = resource
        .current_value
        .checked_add(ability.resource_delta)
        .ok_or(MechanicsHostError::ResourceBounds)?;
    if next_value < resource.minimum_value || next_value > resource.maximum_value {
        return Err(MechanicsHostError::ResourceBounds);
    }

    let definition_hash = ability_definition_hash(ability);
    let effect_request = EffectRequestV1::new(
        ability.package_id.clone(),
        ability.ability_id.clone(),
        definition_hash,
        source.persistent_id,
        source.revision,
        target.persistent_id,
        target.revision,
        ability.resource_id.clone(),
        resource.current_value,
        ability.resource_delta,
        contact_fact.fact_hash()?,
        invocation.gameplay_tick,
    )?;
    effect_request.validate()?;
    let mut targets = vec![
        RpgAggregateRefV1 {
            aggregate_kind: RpgAggregateKindV1::Character,
            persistent_id: source.persistent_id,
            expected_revision: source.revision,
        },
        RpgAggregateRefV1 {
            aggregate_kind: RpgAggregateKindV1::Character,
            persistent_id: target.persistent_id,
            expected_revision: target.revision,
        },
    ];
    targets.sort_unstable();
    let rpg_command = RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets,
            definition_policy_hashes: vec![definition_hash],
            payload: RpgOperationPayloadV1::AdjustCharacterResource {
                source_character_id: source.persistent_id,
                character_id: target.persistent_id,
                resource_id: ability.resource_id.clone(),
                expected_value: resource.current_value,
                delta: ability.resource_delta,
            },
        }],
    };
    rpg_command.validate()?;
    let _ = item;
    Ok(CompiledAbilityEffectV1 {
        ability_definition_hash: definition_hash,
        effect_request,
        rpg_command,
    })
}

fn aggregate(
    snapshot: &RpgSnapshotV2,
    kind: RpgAggregateKindV1,
    id: PersistentId,
) -> Option<&RpgAggregateEnvelopeV1> {
    snapshot
        .aggregates
        .binary_search_by_key(&(kind, id), |aggregate| {
            (aggregate.aggregate_kind, aggregate.persistent_id)
        })
        .ok()
        .map(|index| &snapshot.aggregates[index])
}

fn character(
    aggregate: &RpgAggregateEnvelopeV1,
) -> Result<&next_contracts::rpg::CharacterPayloadV1, MechanicsHostError> {
    match &aggregate.payload {
        RpgAggregatePayloadV1::Character(payload) => Ok(payload),
        _ => Err(MechanicsHostError::AggregateKindMismatch),
    }
}

fn exact_definition_revision(aggregate: &RpgAggregateEnvelopeV1) -> Option<AssetRevisionRefV1> {
    match aggregate.definition_ref {
        DefinitionRefV1::Exact {
            asset_id,
            content_hash,
        } => Some(AssetRevisionRefV1 {
            asset_id,
            record_sha256: content_hash,
        }),
        DefinitionRefV1::None => None,
    }
}

fn validate_package_grants(
    registry: &RpgDefinitionRegistryV2,
    ability: &AbilityDefinitionV1,
) -> Result<(), MechanicsHostError> {
    let locked = registry
        .mechanics_lock
        .packages
        .iter()
        .find(|package| package.package_id == ability.package_id)
        .ok_or(MechanicsHostError::CapabilityDenied)?;
    if ability.required_capabilities.iter().any(|capability| {
        locked
            .granted_capabilities
            .binary_search(capability)
            .is_err()
    }) {
        return Err(MechanicsHostError::CapabilityDenied);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MechanicsHostError {
    Contract(MechanicsContractError),
    RpgContract(RpgContractErrorV1),
    Canonical(next_contracts::canonical::CanonicalError),
    SourceCharacterMissing,
    AggregateKindMismatch,
    EquipmentRequired,
    ContactRequired,
    TargetInvalid,
    TargetResourceMissing,
    ResourceBounds,
    CapabilityDenied,
    CooldownActive,
}

impl Display for MechanicsHostError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "mechanics contract: {error}"),
            Self::RpgContract(error) => write!(formatter, "RPG contract: {error}"),
            Self::Canonical(error) => write!(formatter, "canonical encoding: {error}"),
            Self::SourceCharacterMissing => formatter.write_str("source character is missing"),
            Self::AggregateKindMismatch => formatter.write_str("aggregate kind mismatch"),
            Self::EquipmentRequired => formatter.write_str("required equipment is unavailable"),
            Self::ContactRequired => {
                formatter.write_str("required physical contact is unavailable")
            }
            Self::TargetInvalid => formatter.write_str("ability target is invalid"),
            Self::TargetResourceMissing => formatter.write_str("target resource is missing"),
            Self::ResourceBounds => formatter.write_str("effect exceeds resource bounds"),
            Self::CapabilityDenied => formatter.write_str("package capability denied"),
            Self::CooldownActive => formatter.write_str("ability cooldown is active"),
        }
    }
}

impl Error for MechanicsHostError {}

impl From<MechanicsContractError> for MechanicsHostError {
    fn from(value: MechanicsContractError) -> Self {
        Self::Contract(value)
    }
}

impl From<RpgContractErrorV1> for MechanicsHostError {
    fn from(value: RpgContractErrorV1) -> Self {
        Self::RpgContract(value)
    }
}

impl From<next_contracts::canonical::CanonicalError> for MechanicsHostError {
    fn from(value: next_contracts::canonical::CanonicalError) -> Self {
        Self::Canonical(value)
    }
}

#[cfg(test)]
mod tests {
    use next_contracts::ids::{AssetId, ContentHash, PersistentId, PhysicsContactId, SchemaId};
    use next_contracts::mechanics::{
        CooldownSpecV1, MechanicsContractError, MechanicsLockV1, RpgDefinitionRegistryV2,
    };
    use next_contracts::rpg::{
        CharacterPayloadV1, CharacterResourceEntryV1, DefinitionRefV1, EquipmentPayloadV1,
        EquipmentSlotAssignmentV1, InventoryPayloadV1, ItemPayloadV1, ProvenanceBindingV1,
        RpgAggregateEnvelopeV1, RpgAggregatePayloadV1, RpgPhysicalContactFactV1, RpgSnapshotV2,
    };

    #[test]
    fn cooldown_boundary_is_exact() {
        let cooldown = CooldownSpecV1 {
            duration_ticks: 2,
            group_id: SchemaId::new("nextengine.test.cooldown").expect("ID"),
        };
        assert_eq!(
            cooldown.validate_boundary(Some(10), 11),
            Err(next_contracts::mechanics::MechanicsContractError::CooldownActive)
        );
        assert_eq!(cooldown.validate_boundary(Some(10), 12), Ok(()));
    }

    #[test]
    fn contact_ability_requires_equipment_character_contact_and_package_grants() {
        let registry = cooked_registry();
        let (snapshot, source_id, target_id) = combat_snapshot(&registry);
        let invocation = |facts, prior_cooldown_commit_tick| super::AbilityInvocationV1 {
            gameplay_tick: 7,
            source_character_id: source_id,
            semantic_action_id: SchemaId::new("nextengine.action.melee").expect("action"),
            prior_cooldown_commit_tick,
            physical_contact_facts: facts,
        };

        assert_eq!(
            super::compile_contact_ability_v1(&registry, &snapshot, invocation(vec![], None)),
            Err(super::MechanicsHostError::ContactRequired)
        );
        let invalid_target_fact = contact_fact(7, source_id, PersistentId::from_bytes([3; 16]));
        assert_eq!(
            super::compile_contact_ability_v1(
                &registry,
                &snapshot,
                invocation(vec![invalid_target_fact], None),
            ),
            Err(super::MechanicsHostError::ContactRequired)
        );
        let valid_fact = contact_fact(7, source_id, target_id);
        let compiled = super::compile_contact_ability_v1(
            &registry,
            &snapshot,
            invocation(vec![valid_fact], None),
        )
        .expect("public package request compiles");
        assert_eq!(compiled.effect_request.expected_resource_value, 100);
        assert_eq!(compiled.effect_request.delta, -50);
        assert_eq!(compiled.rpg_command.operations.len(), 1);
        assert_eq!(
            super::compile_contact_ability_v1(
                &registry,
                &snapshot,
                invocation(vec![valid_fact], Some(6)),
            ),
            Err(super::MechanicsHostError::CooldownActive)
        );

        let mut locked = registry.mechanics_lock.packages.clone();
        let combat = locked
            .iter_mut()
            .find(|package| package.package_id.as_str() == "org.nextengine.core.combat")
            .expect("combat lock");
        combat.granted_capabilities.pop();
        let denied_lock = MechanicsLockV1::new(locked).expect("canonical denied lock");
        assert_eq!(
            RpgDefinitionRegistryV2::new(
                registry.dialogues.clone(),
                registry.quests.clone(),
                registry.relationships.clone(),
                registry.interactions.clone(),
                registry.abilities.clone(),
                registry.packages.clone(),
                denied_lock,
            ),
            Err(MechanicsContractError::CapabilityDenied)
        );
    }

    fn cooked_registry() -> RpgDefinitionRegistryV2 {
        next_project::cook_project_v4(next_reference_game::project_source_v4().expect("source"))
            .expect("cook")
            .rpg_definitions
    }

    fn combat_snapshot(
        registry: &RpgDefinitionRegistryV2,
    ) -> (RpgSnapshotV2, PersistentId, PersistentId) {
        let ability = registry.abilities.first().expect("ability");
        let source_id = PersistentId::from_bytes([1; 16]);
        let target_id = PersistentId::from_bytes([2; 16]);
        let item_id = PersistentId::from_bytes([3; 16]);
        let inventory_id = PersistentId::from_bytes([4; 16]);
        let equipment_id = PersistentId::from_bytes([5; 16]);
        let health = || CharacterResourceEntryV1 {
            resource_id: ability.resource_id.clone(),
            current_value: 100,
            minimum_value: 0,
            maximum_value: 100,
        };
        let exact = |asset_id, content_hash| DefinitionRefV1::Exact {
            asset_id,
            content_hash,
        };
        let mut aggregates = vec![
            aggregate(
                source_id,
                DefinitionRefV1::None,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: Some(inventory_id),
                    equipment_id: Some(equipment_id),
                    resources: vec![health()],
                    skills: vec![],
                }),
            ),
            aggregate(
                target_id,
                DefinitionRefV1::None,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: None,
                    equipment_id: None,
                    resources: vec![health()],
                    skills: vec![],
                }),
            ),
            aggregate(
                item_id,
                exact(
                    ability.required_item_definition.asset_id,
                    ability.required_item_definition.record_sha256,
                ),
                RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                    quantity: 1,
                    durability: 100,
                    custom_state: vec![],
                }),
            ),
            aggregate(
                inventory_id,
                DefinitionRefV1::None,
                RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                    owner_id: source_id,
                    capacity: 1,
                    item_ids: vec![item_id],
                    reservations: vec![],
                }),
            ),
            aggregate(
                equipment_id,
                exact(
                    AssetId::from_bytes([9; 16]),
                    ContentHash::from_bytes([9; 32]),
                ),
                RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                    character_id: source_id,
                    slot_policy: exact(
                        AssetId::from_bytes([8; 16]),
                        ContentHash::from_bytes([8; 32]),
                    ),
                    assignments: vec![EquipmentSlotAssignmentV1 {
                        slot_id: ability.required_equipment_slot_id.clone(),
                        item_id,
                    }],
                }),
            ),
        ];
        aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
        (RpgSnapshotV2 { aggregates }, source_id, target_id)
    }

    fn aggregate(
        persistent_id: PersistentId,
        definition_ref: DefinitionRefV1,
        payload: RpgAggregatePayloadV1,
    ) -> RpgAggregateEnvelopeV1 {
        RpgAggregateEnvelopeV1::new(
            persistent_id,
            1,
            0,
            definition_ref,
            ProvenanceBindingV1::None,
            payload,
        )
        .expect("aggregate")
    }

    fn contact_fact(
        gameplay_tick: u64,
        first: PersistentId,
        second: PersistentId,
    ) -> RpgPhysicalContactFactV1 {
        let (subject_low, subject_high) = if first < second {
            (first, second)
        } else {
            (second, first)
        };
        RpgPhysicalContactFactV1 {
            gameplay_tick,
            contact_id: PhysicsContactId::from_bytes([7; 16]),
            subject_low,
            subject_high,
            physics_checkpoint_revision: 7,
            source_snapshot_hash: ContentHash::from_bytes([8; 32]),
            contact_batch_hash: ContentHash::from_bytes([9; 32]),
        }
    }
}
