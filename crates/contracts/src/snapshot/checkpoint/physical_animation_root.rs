use crate::cognition::{AgentCognitionSnapshotV1, AgentMemorySnapshotV1};
use crate::ids::StateRoot;
use crate::physical_animation::PhysicalAnimationSnapshotV1;
use crate::physics::PhysicsWorldCheckpointV1;
use crate::rpg::RpgSnapshotV2;
use crate::snapshot::RuntimeSnapshotV3;
use crate::world::WorldStreamingSnapshotV1;
use crate::world_activity::WorldActivitySnapshotV1;
use crate::world_population::WorldPopulationSnapshotV1;
use crate::world_routine::WorldRoutineSnapshotV1;

use super::{
    WorldCheckpointError, WorldCheckpointV4,
    world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root_from_canonical_components,
};

#[allow(
    clippy::too_many_arguments,
    reason = "the R5a application root keeps every authoritative owner projection explicit"
)]
pub fn world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
    world_streaming_snapshot: &WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&WorldRoutineSnapshotV1>,
    world_population_snapshot: &WorldPopulationSnapshotV1,
    world_activity_snapshot: &WorldActivitySnapshotV1,
    agent_snapshot: &AgentCognitionSnapshotV1,
    memory_snapshot: &AgentMemorySnapshotV1,
    physical_animation_snapshot: &PhysicalAnimationSnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    let (_, components) = WorldCheckpointV4::new_with_canonical_components(
        runtime_snapshot.clone(),
        rpg_snapshot.clone(),
        physics_checkpoint.clone(),
    )?;
    world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root_from_canonical_components(
        &components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        world_population_snapshot,
        world_activity_snapshot,
        agent_snapshot,
        memory_snapshot,
        physical_animation_snapshot,
    )
}
