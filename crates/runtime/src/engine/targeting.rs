use next_contracts::ids::{CommandStreamId, SchemaId};
use next_contracts::input::{CORE_INTERACT_ACTION_ID, CORE_PICKUP_ACTION_ID};
use next_contracts::physics::{
    PHYSICS_QUERY_SCHEMA_VERSION, PhysicsQueryCardinalityV1, PhysicsQueryFilterV1,
    PhysicsQueryGeometryV1, PhysicsQueryIdV1, PhysicsQueryKindV1, PhysicsQueryRequestV1,
    PhysicsQueryResultPayloadV1, PhysicsQueryResultV1, PhysicsSnapshotSelectorV1,
};
use next_contracts::targeting::{
    AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION, AuthoritativeTargetingQueryV1,
    TARGETING_INTENT_SCHEMA_VERSION, TARGETING_QUERY_PROFILE_SCHEMA_VERSION,
    TargetVisibilityPolicyV1, TargetingIntentV1, TargetingQueryProfileV1,
};
use next_physics_api::{PhysicsSceneQueryError, PhysicsWorldHost, execute_scene_query};

use super::RuntimeFatalError;
use super::ingress::{InteractionIntentKind, PendingInteractionIntent};

const INTERACTION_TARGETING_PROFILE_ID: &str = "nextengine.targeting.interaction-nearby.v1";
const INTERACT_ABILITY_ID: &str = "nextengine.ability.interact";
const PICKUP_ABILITY_ID: &str = "nextengine.ability.pickup";
const INTERACTION_DISTANCE_MICROMETRES: i64 = 1_000_000;
const INTERACTION_PUBLISHED_HIT_LIMIT: u32 = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ResolvedInteractionTargetingV1 {
    pub(super) intent: TargetingIntentV1,
    pub(super) query: AuthoritativeTargetingQueryV1,
    pub(super) result: PhysicsQueryResultV1,
}

pub(super) fn resolve_interaction_targeting(
    pending: &PendingInteractionIntent,
    assigned_gameplay_tick: u64,
    query_slot: u32,
    issuer_stream_id: CommandStreamId,
    physics: &PhysicsWorldHost,
) -> Result<ResolvedInteractionTargetingV1, RuntimeFatalError> {
    if pending.assignment.assigned_tick != assigned_gameplay_tick {
        return Err(RuntimeFatalError::IngressCheckpointCorrupt);
    }
    let action_id = match pending.kind {
        InteractionIntentKind::General => CORE_INTERACT_ACTION_ID,
        InteractionIntentKind::Pickup => CORE_PICKUP_ACTION_ID,
        InteractionIntentKind::EquipUse | InteractionIntentKind::Melee => {
            return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
        }
    };
    let ability_id = match pending.kind {
        InteractionIntentKind::General => INTERACT_ABILITY_ID,
        InteractionIntentKind::Pickup => PICKUP_ABILITY_ID,
        InteractionIntentKind::EquipUse | InteractionIntentKind::Melee => unreachable!(),
    };
    let checkpoint = physics.checkpoint();
    let snapshot = &checkpoint.snapshot;
    let actor_body_id = *checkpoint
        .catalog
        .avatar_bindings
        .get(&pending.controlled_body_id)
        .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
    let actor_state = snapshot
        .sorted_body_states
        .get(&actor_body_id)
        .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;

    let profile = interaction_targeting_profile()?;
    let profile_hash = profile.profile_hash()?;
    let intent = TargetingIntentV1 {
        schema_version: TARGETING_INTENT_SCHEMA_VERSION,
        assignment: pending.assignment.clone(),
        player_id: pending.controlled_body_id,
        ability_id: SchemaId::new(ability_id)
            .expect("engine-owned targeting ability identifier is valid"),
        action_id: SchemaId::new(action_id)
            .expect("engine-owned semantic action identifier is valid"),
        aim_q15: [0, 0],
        query_kind: PhysicsQueryKindV1::ClosestPoint,
        targeting_profile_id: profile.profile_id.clone(),
        targeting_profile_hash: profile_hash,
        proposed_target_id: None,
    };
    intent.validate()?;

    let mut filter = profile.filter.clone();
    let exclusion_index = filter
        .excluded_bodies
        .binary_search(&actor_body_id)
        .unwrap_or_else(|index| index);
    if filter
        .excluded_bodies
        .get(exclusion_index)
        .is_none_or(|excluded| *excluded != actor_body_id)
    {
        filter
            .excluded_bodies
            .insert(exclusion_index, actor_body_id);
    }
    let physics_snapshot_hash = snapshot.snapshot_hash()?;
    let snapshot_selector = PhysicsSnapshotSelectorV1 {
        physics_tick: snapshot.physics_tick,
        completed_substep: 0,
        physics_snapshot_hash,
    };
    let physics_query = PhysicsQueryRequestV1 {
        schema_version: PHYSICS_QUERY_SCHEMA_VERSION,
        query_id: PhysicsQueryIdV1 {
            physics_tick: snapshot.physics_tick,
            query_slot,
            issuer_stream_id,
        },
        world_id: snapshot.world_id,
        snapshot_selector,
        geometry: PhysicsQueryGeometryV1::ClosestPoint {
            point_micrometres: actor_state.pose.translation_micrometres,
            maximum_distance_micrometres: profile.maximum_distance_micrometres,
        },
        filter,
        cardinality: PhysicsQueryCardinalityV1::All,
        maximum_published_hits: profile.maximum_candidate_hits,
    };
    let query = AuthoritativeTargetingQueryV1 {
        schema_version: AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION,
        intent_hash: intent.intent_hash()?,
        assignment: pending.assignment.clone(),
        actor_id: pending.controlled_body_id,
        actor_body_id,
        actor_body_revision: actor_state.body_revision,
        targeting_profile_hash: profile_hash,
        physics_query,
    };
    query.validate_against(&intent, &profile)?;
    let result = execute_scene_query(checkpoint, &query.physics_query)?;
    if !matches!(
        &result.payload,
        PhysicsQueryResultPayloadV1::All {
            truncated: false,
            ..
        }
    ) {
        return Err(RuntimeFatalError::PhysicsQuery(
            PhysicsSceneQueryError::CapacityExceeded,
        ));
    }
    Ok(ResolvedInteractionTargetingV1 {
        intent,
        query,
        result,
    })
}

fn interaction_targeting_profile() -> Result<TargetingQueryProfileV1, RuntimeFatalError> {
    let profile = TargetingQueryProfileV1 {
        schema_version: TARGETING_QUERY_PROFILE_SCHEMA_VERSION,
        profile_id: SchemaId::new(INTERACTION_TARGETING_PROFILE_ID)
            .expect("engine-owned targeting profile identifier is valid"),
        profile_revision: 1,
        query_kind: PhysicsQueryKindV1::ClosestPoint,
        maximum_distance_micrometres: INTERACTION_DISTANCE_MICROMETRES,
        maximum_candidate_hits: INTERACTION_PUBLISHED_HIT_LIMIT,
        filter: PhysicsQueryFilterV1 {
            query_collision_layer: 0,
            query_collision_mask: 1,
            include_solid: true,
            include_sensor: true,
            include_query_only: true,
            excluded_bodies: Vec::new(),
            excluded_shapes: Vec::new(),
        },
        visibility_policy: TargetVisibilityPolicyV1::NotRequiredB0,
    };
    profile.validate()?;
    Ok(profile)
}
