#![forbid(unsafe_code)]

pub mod cognition;
mod tier_cadence;

pub use tier_cadence::{
    TIER_COGNITION_MAX_WORK_ITEMS_V1, TierCognitionServiceReportV1, TierCognitionWorkItemV1,
    TierCognitionWorkKindV1, dispatch_tier_cognition_v1,
};

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::agent::{
    AgentAffordanceCandidateV1, AgentContractError, AgentIntentV1, AgentPlannerSnapshotV1,
    MotorCapabilityStateV1, ProceduralAvatarProjectionV1, ProceduralAvatarRouteV1,
    ProceduralFallbackReasonV1,
};
use next_contracts::canonical::sha256;
use next_contracts::command::{IssuerPrincipal, WorldCommand};
use next_contracts::ids::{CommandStreamId, PersistentId, SchemaId, content_hash_from_bytes};
use next_contracts::mechanics::{RpgDefinitionRegistryV2, ability_definition_hash};
use next_contracts::rpg::{RpgAggregateKindV1, RpgPhysicalContactFactV1, RpgSnapshotV2};
use next_mechanics::{
    AbilityInvocationV1, CompiledAbilityEffectV1, MechanicsHostError, compile_contact_ability_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPlanningRequestV1<'a> {
    pub gameplay_tick: u64,
    pub world_generation: u64,
    pub decision_seed: u64,
    pub source_character_id: PersistentId,
    pub target_character_id: PersistentId,
    pub allowed_semantic_actions: Vec<SchemaId>,
    pub motor_state: MotorCapabilityStateV1,
    pub ai_host_available: bool,
    pub model_available: bool,
    pub rpg_snapshot: &'a RpgSnapshotV2,
    pub definitions: &'a RpgDefinitionRegistryV2,
    pub physical_contact_facts: &'a [RpgPhysicalContactFactV1],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommandRouteV1 {
    pub issuer: IssuerPrincipal,
    pub stream_id: CommandStreamId,
    pub sequence: u64,
    pub target_tick: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedAgentCommandV1 {
    pub planner_snapshot: AgentPlannerSnapshotV1,
    pub intent: AgentIntentV1,
    pub procedural_projection: ProceduralAvatarProjectionV1,
    pub compiled_effect: CompiledAbilityEffectV1,
    pub world_command: WorldCommand,
}

pub fn build_planner_snapshot_v1(
    request: &AgentPlanningRequestV1<'_>,
) -> Result<AgentPlannerSnapshotV1, AgentPlannerError> {
    request.rpg_snapshot.validate()?;
    request
        .definitions
        .validate()
        .map_err(MechanicsHostError::Contract)?;
    let rpg_state_hash = content_hash_from_bytes(sha256(&request.rpg_snapshot.canonical_bytes()?));
    if aggregate_revision(request.rpg_snapshot, request.source_character_id).is_none()
        || aggregate_revision(request.rpg_snapshot, request.target_character_id).is_none()
    {
        return Err(AgentPlannerError::CharacterMissing);
    }
    let contact_is_current = request.physical_contact_facts.iter().any(|fact| {
        fact.gameplay_tick == request.gameplay_tick
            && fact.connects(request.source_character_id, request.target_character_id)
            && fact.validate().is_ok()
    });
    let mut candidates = request
        .definitions
        .abilities
        .iter()
        .filter(|ability| ability.affordance.planner_visible)
        .filter(|ability| {
            request
                .allowed_semantic_actions
                .contains(&ability.affordance.semantic_action_id)
        })
        .filter(|ability| !ability.affordance.requires_contact || contact_is_current)
        .map(|ability| {
            let utility_q16 = ability
                .affordance
                .expected_resource_delta_minimum
                .saturating_neg()
                .saturating_mul(1 << 16);
            AgentAffordanceCandidateV1 {
                semantic_action_id: ability.affordance.semantic_action_id.clone(),
                ability_definition_hash: ability_definition_hash(ability),
                target_character_id: request.target_character_id,
                utility_q16,
                required_route: ProceduralAvatarRouteV1::Melee,
            }
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    AgentPlannerSnapshotV1::new(
        request.gameplay_tick,
        request.world_generation,
        rpg_state_hash,
        request.definitions.mechanics_lock.mechanics_lock_sha256,
        request.decision_seed,
        request.source_character_id,
        request.motor_state,
        request.allowed_semantic_actions.clone(),
        candidates,
    )
    .map_err(AgentPlannerError::Contract)
}

pub fn propose_world_command_v1(
    request: &AgentPlanningRequestV1<'_>,
    route: AgentCommandRouteV1,
) -> Result<PlannedAgentCommandV1, AgentPlannerError> {
    let planner_snapshot = build_planner_snapshot_v1(request)?;
    propose_world_command_from_snapshot_v1(request, planner_snapshot, route)
}

pub fn propose_world_command_from_snapshot_v1(
    request: &AgentPlanningRequestV1<'_>,
    planner_snapshot: AgentPlannerSnapshotV1,
    route: AgentCommandRouteV1,
) -> Result<PlannedAgentCommandV1, AgentPlannerError> {
    if route.target_tick
        != request
            .gameplay_tick
            .checked_add(1)
            .ok_or(AgentPlannerError::TickExhausted)?
    {
        return Err(AgentPlannerError::InvalidCommandRoute);
    }
    planner_snapshot.validate()?;
    if planner_snapshot.motor_state == MotorCapabilityStateV1::Unavailable {
        return Err(AgentPlannerError::MotorUnavailable);
    }
    let current_rpg_hash =
        content_hash_from_bytes(sha256(&request.rpg_snapshot.canonical_bytes()?));
    if current_rpg_hash != planner_snapshot.rpg_state_hash
        || request.world_generation != planner_snapshot.world_generation
        || request.definitions.mechanics_lock.mechanics_lock_sha256
            != planner_snapshot.mechanics_lock_hash
    {
        return Err(AgentPlannerError::StaleSnapshot);
    }
    let candidate = planner_snapshot
        .affordances
        .first()
        .ok_or(AgentPlannerError::AffordanceUnavailable)?;
    let intent = AgentIntentV1::new(&planner_snapshot, candidate)?;
    let compiled_effect = compile_contact_ability_v1(
        request.definitions,
        request.rpg_snapshot,
        AbilityInvocationV1 {
            gameplay_tick: request.gameplay_tick,
            source_character_id: request.source_character_id,
            semantic_action_id: candidate.semantic_action_id.clone(),
            prior_cooldown_commit_tick: None,
            physical_contact_facts: request.physical_contact_facts.to_vec(),
        },
    )?;
    if compiled_effect.ability_definition_hash != candidate.ability_definition_hash
        || compiled_effect.effect_request.target_character_id != request.target_character_id
    {
        return Err(AgentPlannerError::SelectedAffordanceMismatch);
    }
    let fallback_reason = if !request.ai_host_available {
        ProceduralFallbackReasonV1::AiHostUnavailable
    } else if !request.model_available {
        ProceduralFallbackReasonV1::ModelUnavailable
    } else {
        ProceduralFallbackReasonV1::LearnedPolicyNotActivated
    };
    let procedural_projection = ProceduralAvatarProjectionV1::from_intent(
        &intent,
        candidate.required_route,
        fallback_reason,
    )?;
    let world_command = WorldCommand::rpg(
        route.stream_id,
        route.issuer,
        route.sequence,
        route.target_tick,
        compiled_effect.rpg_command.clone(),
    )?;
    Ok(PlannedAgentCommandV1 {
        planner_snapshot,
        intent,
        procedural_projection,
        compiled_effect,
        world_command,
    })
}

fn aggregate_revision(snapshot: &RpgSnapshotV2, id: PersistentId) -> Option<u64> {
    snapshot
        .aggregates
        .iter()
        .find(|aggregate| {
            aggregate.aggregate_kind == RpgAggregateKindV1::Character
                && aggregate.persistent_id == id
        })
        .map(|aggregate| aggregate.revision)
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AgentPlannerError {
    Contract(AgentContractError),
    RpgContract(next_contracts::rpg::RpgContractErrorV1),
    Mechanics(MechanicsHostError),
    Canonical(next_contracts::canonical::CanonicalError),
    CharacterMissing,
    AffordanceUnavailable,
    MotorUnavailable,
    StaleSnapshot,
    SelectedAffordanceMismatch,
    InvalidCommandRoute,
    TickExhausted,
}

impl Display for AgentPlannerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "agent contract: {error}"),
            Self::RpgContract(error) => write!(formatter, "RPG contract: {error}"),
            Self::Mechanics(error) => write!(formatter, "mechanics host: {error}"),
            Self::Canonical(error) => write!(formatter, "canonical encoding: {error}"),
            Self::CharacterMissing => formatter.write_str("agent source or target is missing"),
            Self::AffordanceUnavailable => {
                formatter.write_str("planner-visible affordance is unavailable")
            }
            Self::MotorUnavailable => formatter.write_str("motor capability is unavailable"),
            Self::StaleSnapshot => formatter.write_str("agent planner snapshot is stale"),
            Self::SelectedAffordanceMismatch => {
                formatter.write_str("compiled mechanics effect differs from selected affordance")
            }
            Self::InvalidCommandRoute => formatter.write_str("agent command route is invalid"),
            Self::TickExhausted => formatter.write_str("agent gameplay tick is exhausted"),
        }
    }
}

impl Error for AgentPlannerError {}

impl From<AgentContractError> for AgentPlannerError {
    fn from(value: AgentContractError) -> Self {
        Self::Contract(value)
    }
}

impl From<next_contracts::rpg::RpgContractErrorV1> for AgentPlannerError {
    fn from(value: next_contracts::rpg::RpgContractErrorV1) -> Self {
        Self::RpgContract(value)
    }
}

impl From<MechanicsHostError> for AgentPlannerError {
    fn from(value: MechanicsHostError) -> Self {
        Self::Mechanics(value)
    }
}

impl From<next_contracts::canonical::CanonicalError> for AgentPlannerError {
    fn from(value: next_contracts::canonical::CanonicalError) -> Self {
        Self::Canonical(value)
    }
}

#[cfg(test)]
mod tests {
    use next_contracts::agent::MotorCapabilityStateV1;
    use next_contracts::ids::{AssetId, ContentHash, PersistentId, PhysicsContactId, SchemaId};
    use next_contracts::physics::ContactPhaseV1;
    use next_contracts::rpg::{
        CharacterPayloadV1, CharacterResourceEntryV1, DefinitionRefV1, EquipmentPayloadV1,
        EquipmentSlotAssignmentV1, InventoryPayloadV1, ItemPayloadV1, ProvenanceBindingV1,
        RpgAggregateEnvelopeV1, RpgAggregatePayloadV1, RpgPhysicalContactFactV1, RpgSnapshotV2,
    };

    use super::{AgentPlannerError, AgentPlanningRequestV1, build_planner_snapshot_v1};

    #[test]
    fn input_order_and_equal_seed_produce_identical_plan() {
        let registry = registry();
        let (snapshot, source, target, facts) = state(&registry);
        let allowed = vec![
            SchemaId::new("nextengine.action.unavailable").expect("action"),
            SchemaId::new(next_contracts::input::CORE_MELEE_ACTION_ID).expect("melee"),
        ];
        let first = request(
            &snapshot,
            &registry,
            &facts,
            allowed.clone(),
            source,
            target,
        );
        let mut reversed_facts = facts.clone();
        reversed_facts.reverse();
        let mut reversed_allowed = allowed;
        reversed_allowed.reverse();
        let second = request(
            &snapshot,
            &registry,
            &reversed_facts,
            reversed_allowed,
            source,
            target,
        );
        assert_eq!(
            build_planner_snapshot_v1(&first).expect("first plan"),
            build_planner_snapshot_v1(&second).expect("permuted plan")
        );
    }

    #[test]
    fn unavailable_affordance_and_motor_fail_closed() {
        let registry = registry();
        let (snapshot, source, target, facts) = state(&registry);
        let no_contact = request(&snapshot, &registry, &[], vec![melee()], source, target);
        assert!(
            build_planner_snapshot_v1(&no_contact)
                .expect("valid empty snapshot")
                .affordances
                .is_empty()
        );
        let mut unavailable = request(&snapshot, &registry, &facts, vec![melee()], source, target);
        unavailable.motor_state = MotorCapabilityStateV1::Unavailable;
        let route = super::AgentCommandRouteV1 {
            issuer: next_contracts::command::IssuerPrincipal::InternalSystem(
                next_contracts::ids::SystemId::new("nextengine.test.agent").expect("system"),
            ),
            stream_id: next_contracts::ids::CommandStreamId::from_bytes([7; 16]),
            sequence: 0,
            target_tick: 8,
        };
        assert!(matches!(
            super::propose_world_command_v1(&unavailable, route),
            Err(AgentPlannerError::MotorUnavailable)
        ));
    }

    #[test]
    fn stale_snapshot_and_route_tick_are_rejected() {
        let registry = registry();
        let (snapshot, source, target, facts) = state(&registry);
        let initial_request = request(&snapshot, &registry, &facts, vec![melee()], source, target);
        let planner_snapshot =
            build_planner_snapshot_v1(&initial_request).expect("planner snapshot");
        let route = super::AgentCommandRouteV1 {
            issuer: next_contracts::command::IssuerPrincipal::InternalSystem(
                next_contracts::ids::SystemId::new("nextengine.test.agent").expect("system"),
            ),
            stream_id: next_contracts::ids::CommandStreamId::from_bytes([7; 16]),
            sequence: 0,
            target_tick: 9,
        };
        assert!(matches!(
            super::propose_world_command_v1(&initial_request, route),
            Err(AgentPlannerError::InvalidCommandRoute)
        ));

        let mut changed = snapshot.clone();
        let target_aggregate = changed
            .aggregates
            .iter_mut()
            .find(|aggregate| aggregate.persistent_id == target)
            .expect("target");
        *target_aggregate = RpgAggregateEnvelopeV1::new(
            target_aggregate.persistent_id,
            target_aggregate.schema_version,
            target_aggregate.revision + 1,
            target_aggregate.definition_ref.clone(),
            target_aggregate.provenance.clone(),
            target_aggregate.payload.clone(),
        )
        .expect("revision change remains canonical");
        let changed_request = request(&changed, &registry, &facts, vec![melee()], source, target);
        let valid_route = super::AgentCommandRouteV1 {
            issuer: next_contracts::command::IssuerPrincipal::InternalSystem(
                next_contracts::ids::SystemId::new("nextengine.test.agent").expect("system"),
            ),
            stream_id: next_contracts::ids::CommandStreamId::from_bytes([7; 16]),
            sequence: 0,
            target_tick: 8,
        };
        assert!(matches!(
            super::propose_world_command_from_snapshot_v1(
                &changed_request,
                planner_snapshot,
                valid_route
            ),
            Err(AgentPlannerError::StaleSnapshot)
        ));
    }

    fn request<'a>(
        snapshot: &'a RpgSnapshotV2,
        definitions: &'a next_contracts::mechanics::RpgDefinitionRegistryV2,
        facts: &'a [RpgPhysicalContactFactV1],
        allowed_semantic_actions: Vec<SchemaId>,
        source_character_id: PersistentId,
        target_character_id: PersistentId,
    ) -> AgentPlanningRequestV1<'a> {
        AgentPlanningRequestV1 {
            gameplay_tick: 7,
            world_generation: 2,
            decision_seed: 42,
            source_character_id,
            target_character_id,
            allowed_semantic_actions,
            motor_state: MotorCapabilityStateV1::ProceduralFallback,
            ai_host_available: false,
            model_available: false,
            rpg_snapshot: snapshot,
            definitions,
            physical_contact_facts: facts,
        }
    }

    fn melee() -> SchemaId {
        SchemaId::new(next_contracts::input::CORE_MELEE_ACTION_ID).expect("melee")
    }

    fn registry() -> next_contracts::mechanics::RpgDefinitionRegistryV2 {
        next_project::cook_project_v6(next_reference_game::project_source_v6().expect("source"))
            .expect("cook")
            .rpg_definitions
    }

    fn state(
        registry: &next_contracts::mechanics::RpgDefinitionRegistryV2,
    ) -> (
        RpgSnapshotV2,
        PersistentId,
        PersistentId,
        Vec<RpgPhysicalContactFactV1>,
    ) {
        let ability = registry.abilities.first().expect("ability");
        let source = PersistentId::from_bytes([1; 16]);
        let target = PersistentId::from_bytes([2; 16]);
        let item = PersistentId::from_bytes([3; 16]);
        let inventory = PersistentId::from_bytes([4; 16]);
        let equipment = PersistentId::from_bytes([5; 16]);
        let health = || CharacterResourceEntryV1 {
            resource_id: ability.resource_id.clone(),
            current_value: 100,
            minimum_value: 0,
            maximum_value: 100,
        };
        let mut aggregates = vec![
            aggregate(
                source,
                DefinitionRefV1::None,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: Some(inventory),
                    equipment_id: Some(equipment),
                    resources: vec![health()],
                    skills: vec![],
                }),
            ),
            aggregate(
                target,
                DefinitionRefV1::None,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: None,
                    equipment_id: None,
                    resources: vec![health()],
                    skills: vec![],
                }),
            ),
            aggregate(
                item,
                DefinitionRefV1::Exact {
                    asset_id: ability.required_item_definition.asset_id,
                    content_hash: ability.required_item_definition.record_sha256,
                },
                RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                    quantity: 1,
                    durability: 100,
                    custom_state: vec![],
                }),
            ),
            aggregate(
                inventory,
                DefinitionRefV1::None,
                RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                    owner_id: source,
                    capacity: 1,
                    item_ids: vec![item],
                    reservations: vec![],
                }),
            ),
            aggregate(
                equipment,
                DefinitionRefV1::Exact {
                    asset_id: AssetId::from_bytes([9; 16]),
                    content_hash: ContentHash::from_bytes([9; 32]),
                },
                RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                    character_id: source,
                    slot_policy: DefinitionRefV1::Exact {
                        asset_id: AssetId::from_bytes([8; 16]),
                        content_hash: ContentHash::from_bytes([8; 32]),
                    },
                    assignments: vec![EquipmentSlotAssignmentV1 {
                        slot_id: ability.required_equipment_slot_id.clone(),
                        item_id: item,
                    }],
                }),
            ),
        ];
        aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
        let fact = RpgPhysicalContactFactV1 {
            gameplay_tick: 7,
            contact_id: PhysicsContactId::from_bytes([6; 16]),
            subject_low: source,
            subject_high: target,
            physics_checkpoint_revision: 3,
            source_snapshot_hash: ContentHash::from_bytes([7; 32]),
            contact_batch_hash: ContentHash::from_bytes([8; 32]),
        };
        let _ = ContactPhaseV1::Persist;
        (RpgSnapshotV2 { aggregates }, source, target, vec![fact])
    }

    fn aggregate(
        id: PersistentId,
        definition_ref: DefinitionRefV1,
        payload: RpgAggregatePayloadV1,
    ) -> RpgAggregateEnvelopeV1 {
        RpgAggregateEnvelopeV1::new(id, 1, 0, definition_ref, ProvenanceBindingV1::None, payload)
            .expect("aggregate")
    }
}
