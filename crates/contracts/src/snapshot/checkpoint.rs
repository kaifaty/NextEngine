use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::canonical::CanonicalError;
use crate::command::IssuerPrincipal;
use crate::identity::{CommandStreamRegistryV1, PrincipalRegistryV1};
use crate::ids::{CommandLedgerHash, StateRoot, SystemId, command_ledger_hash_from_bytes};
use crate::input::{PLAYER_INTERACTION_SYSTEM_ID, PlayerControllerRegistryV1};
use crate::physics::{
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID, PhysicsContractError,
    PhysicsMotionKindV1, PhysicsWorldCheckpointV1,
};
use crate::rpg::{
    CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_DIALOGUE_OFFER_NODE_ID, CORE_DIALOGUE_QUEST_TRUST_DELTA,
    CORE_QUEST_ACTIVE_STATE_ID, CORE_QUEST_AVAILABLE_STATE_ID,
    CORE_RELATIONSHIP_TRUST_DIMENSION_ID, CoreDialogueQuestClosureError,
};
use crate::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgContractErrorV1, RpgSnapshotV2};

use super::runtime::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID,
    RuntimeSnapshotV3, SnapshotDecodeError, snapshot_validation_error,
};

mod error;
mod physical_animation_root;
mod state_root;

pub use error::WorldCheckpointError;
pub use physical_animation_root::world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root;
pub use state_root::state_root_from_save_segment_descriptors;

use state_root::{state_root_from_segment_slices, state_root_from_segments};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCheckpointV4 {
    pub runtime_snapshot: RuntimeSnapshotV3,
    pub rpg_snapshot: RpgSnapshotV2,
    pub physics_checkpoint: PhysicsWorldCheckpointV1,
    pub state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCheckpointCanonicalComponentsV1 {
    runtime_snapshot: Arc<[u8]>,
    command_ledger: Arc<[u8]>,
    command_ledger_hash: CommandLedgerHash,
    rpg_snapshot: Arc<[u8]>,
    physics_checkpoint: Arc<[u8]>,
}

impl WorldCheckpointCanonicalComponentsV1 {
    #[must_use]
    pub fn runtime_snapshot_bytes(&self) -> &[u8] {
        &self.runtime_snapshot
    }

    #[must_use]
    pub fn runtime_snapshot_shared_bytes(&self) -> Arc<[u8]> {
        self.runtime_snapshot.clone()
    }

    #[must_use]
    pub fn rpg_snapshot_bytes(&self) -> &[u8] {
        &self.rpg_snapshot
    }

    #[must_use]
    pub fn rpg_snapshot_shared_bytes(&self) -> Arc<[u8]> {
        self.rpg_snapshot.clone()
    }

    pub const fn command_ledger_hash(&self) -> Result<CommandLedgerHash, CanonicalError> {
        Ok(self.command_ledger_hash)
    }

    fn compute_command_ledger_hash(
        command_ledger: &[u8],
    ) -> Result<CommandLedgerHash, CanonicalError> {
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest as _;
        hasher.update(b"nextengine.command-ledger.v2\0");
        hasher.update(
            u64::try_from(command_ledger.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        hasher.update(command_ledger);
        Ok(command_ledger_hash_from_bytes(hasher.finalize().into()))
    }

    #[must_use]
    pub fn physics_checkpoint_bytes(&self) -> &[u8] {
        &self.physics_checkpoint
    }

    #[must_use]
    pub fn physics_checkpoint_shared_bytes(&self) -> Arc<[u8]> {
        self.physics_checkpoint.clone()
    }
}

impl WorldCheckpointV4 {
    pub fn new(
        runtime_snapshot: RuntimeSnapshotV3,
        rpg_snapshot: RpgSnapshotV2,
        physics_checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<Self, WorldCheckpointError> {
        Self::new_with_canonical_components(runtime_snapshot, rpg_snapshot, physics_checkpoint)
            .map(|(checkpoint, _)| checkpoint)
    }

    pub fn new_with_canonical_components(
        runtime_snapshot: RuntimeSnapshotV3,
        rpg_snapshot: RpgSnapshotV2,
        physics_checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<(Self, WorldCheckpointCanonicalComponentsV1), WorldCheckpointError> {
        Self::new_with_canonical_components_internal(
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            RuntimeCheckpointValidation::Complete,
        )
    }

    /// Materializes a checkpoint from a live runtime generation whose command
    /// history was produced by prepared incremental ledger/archive commits.
    ///
    /// This remains a safe validation path rather than a trust escape hatch:
    /// the ledger/archive caches and mutation APIs that prove each incremental
    /// addition are private to this crate and cannot be forged by a caller.
    /// Durable decode, restore, recovery and migration must still use
    /// [`Self::new_with_canonical_components`] (or the canonical decoders),
    /// which retain complete historical revalidation of untrusted bytes.
    #[doc(hidden)]
    pub fn new_with_incrementally_validated_canonical_components(
        runtime_snapshot: RuntimeSnapshotV3,
        rpg_snapshot: RpgSnapshotV2,
        physics_checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<(Self, WorldCheckpointCanonicalComponentsV1), WorldCheckpointError> {
        Self::new_with_canonical_components_internal(
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            RuntimeCheckpointValidation::IncrementalLive,
        )
    }

    fn new_with_canonical_components_internal(
        runtime_snapshot: RuntimeSnapshotV3,
        rpg_snapshot: RpgSnapshotV2,
        physics_checkpoint: PhysicsWorldCheckpointV1,
        validation: RuntimeCheckpointValidation,
    ) -> Result<(Self, WorldCheckpointCanonicalComponentsV1), WorldCheckpointError> {
        let mut checkpoint = Self {
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            state_root: StateRoot::default(),
        };
        let rpg_snapshot = checkpoint.validate_components_with_rpg_bytes(validation)?;
        let (runtime_snapshot, command_ledger) = checkpoint
            .runtime_snapshot
            .canonical_bytes_and_ledger_bytes_validated()?;
        let command_ledger_hash =
            WorldCheckpointCanonicalComponentsV1::compute_command_ledger_hash(&command_ledger)?;
        let components = WorldCheckpointCanonicalComponentsV1 {
            runtime_snapshot: Arc::from(runtime_snapshot),
            command_ledger: Arc::from(command_ledger),
            command_ledger_hash,
            rpg_snapshot: Arc::from(rpg_snapshot),
            physics_checkpoint: Arc::from(checkpoint.physics_checkpoint.canonical_bytes()?),
        };
        checkpoint.state_root =
            world_checkpoint_v4_state_root_from_canonical_components(&components)?;
        Ok((checkpoint, components))
    }

    pub fn validate(&self) -> Result<(), WorldCheckpointError> {
        self.validate_components()?;
        if self.state_root
            != world_checkpoint_v4_state_root_validated(
                &self.runtime_snapshot,
                &self.rpg_snapshot,
                &self.physics_checkpoint,
            )?
        {
            return Err(WorldCheckpointError::ClosureMismatch);
        }
        Ok(())
    }

    fn validate_components(&self) -> Result<(), WorldCheckpointError> {
        self.validate_components_with_rpg_bytes(RuntimeCheckpointValidation::Complete)
            .map(drop)
    }

    fn validate_components_with_rpg_bytes(
        &self,
        validation: RuntimeCheckpointValidation,
    ) -> Result<Vec<u8>, WorldCheckpointError> {
        match validation {
            RuntimeCheckpointValidation::Complete => self.runtime_snapshot.validate()?,
            RuntimeCheckpointValidation::IncrementalLive => {
                self.runtime_snapshot.validate_incremental_checkpoint()?
            }
        }
        let rpg_bytes = self.rpg_snapshot.canonical_bytes()?;
        self.physics_checkpoint.validate()?;
        validate_world_checkpoint_component_closures(
            &self.runtime_snapshot,
            &self.rpg_snapshot,
            &self.physics_checkpoint,
        )?;
        Ok(rpg_bytes)
    }
}

/// Validates the cross-component closures of a world checkpoint whose
/// individual component values have already been validated.
///
/// This performs the same closure checks as checkpoint construction but
/// without re-validating the components, re-encoding them or recomputing
/// the state root. It exists for callers holding fully decoded (and
/// therefore fully validated) components — such as save generation probing —
/// where only the cross-component closure still determines validity and the
/// state root is not stored anywhere it could be compared against.
pub fn validate_world_checkpoint_component_closures(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<(), WorldCheckpointError> {
    physics_checkpoint.snapshot.validate_profile_closure(
        &physics_checkpoint.catalog,
        &runtime_snapshot.tick_rate_profile,
        &runtime_snapshot.authoritative_numeric_profile,
        &runtime_snapshot.physics_quantization_profile,
    )?;
    validate_core_dialogue_quest_world_closure_v2(
        rpg_snapshot,
        &runtime_snapshot.player_controller_registry,
        &runtime_snapshot.principal_registry,
        &runtime_snapshot.stream_registry,
        physics_checkpoint,
    )?;
    let expected_physics_tick = runtime_snapshot
        .next_tick
        .checked_mul(u64::from(
            runtime_snapshot
                .tick_rate_profile
                .physics_substeps_per_gameplay_tick,
        ))
        .ok_or(WorldCheckpointError::ClosureMismatch)?;
    let bindings_close = runtime_snapshot
        .player_controller_registry
        .bindings
        .values()
        .all(|binding| {
            physics_checkpoint
                .catalog
                .avatar_bindings
                .get(&binding.controlled_body_id)
                .is_some_and(|body_id| {
                    physics_checkpoint
                        .snapshot
                        .sorted_body_states
                        .contains_key(body_id)
                })
        });
    if physics_checkpoint.snapshot.checkpoint_revision != runtime_snapshot.authoritative_revision
        || physics_checkpoint.snapshot.physics_tick != expected_physics_tick
        || !bindings_close
    {
        return Err(WorldCheckpointError::ClosureMismatch);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum RuntimeCheckpointValidation {
    Complete,
    IncrementalLive,
}

pub fn validate_core_dialogue_quest_world_closure_v2(
    rpg_snapshot: &RpgSnapshotV2,
    player_controller_registry: &PlayerControllerRegistryV1,
    principal_registry: &PrincipalRegistryV1,
    stream_registry: &CommandStreamRegistryV1,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<(), CoreDialogueQuestClosureError> {
    let has_core_records = rpg_snapshot.aggregates.iter().any(|aggregate| {
        matches!(
            &aggregate.payload,
            RpgAggregatePayloadV1::Dialogue(payload)
                if matches!(
                    payload.node_id.as_str(),
                    CORE_DIALOGUE_OFFER_NODE_ID | CORE_DIALOGUE_ACCEPTED_NODE_ID
                )
        ) || matches!(
            &aggregate.payload,
            RpgAggregatePayloadV1::Quest(payload)
                if matches!(
                    payload.state_id.as_str(),
                    CORE_QUEST_AVAILABLE_STATE_ID | CORE_QUEST_ACTIVE_STATE_ID
                )
        )
    });
    if !has_core_records {
        return Ok(());
    }
    if player_controller_registry.bindings.len() != 1 {
        return Err(CoreDialogueQuestClosureError);
    }
    let controller = player_controller_registry
        .bindings
        .values()
        .next()
        .ok_or(CoreDialogueQuestClosureError)?;
    let player_id = controller.controlled_body_id;

    let mut dialogues = rpg_snapshot.aggregates.iter().filter_map(|aggregate| {
        let RpgAggregatePayloadV1::Dialogue(payload) = &aggregate.payload else {
            return None;
        };
        (payload.listener_id == player_id
            && matches!(
                payload.node_id.as_str(),
                CORE_DIALOGUE_OFFER_NODE_ID | CORE_DIALOGUE_ACCEPTED_NODE_ID
            ))
        .then_some(payload)
    });
    let dialogue = dialogues.next().ok_or(CoreDialogueQuestClosureError)?;
    if dialogues.next().is_some()
        || !contains_aggregate(rpg_snapshot, RpgAggregateKindV1::Character, player_id)
        || !contains_aggregate(
            rpg_snapshot,
            RpgAggregateKindV1::Character,
            dialogue.speaker_id,
        )
    {
        return Err(CoreDialogueQuestClosureError);
    }

    let mut quests = rpg_snapshot.aggregates.iter().filter_map(|aggregate| {
        let RpgAggregatePayloadV1::Quest(payload) = &aggregate.payload else {
            return None;
        };
        matches!(
            payload.state_id.as_str(),
            CORE_QUEST_AVAILABLE_STATE_ID | CORE_QUEST_ACTIVE_STATE_ID
        )
        .then_some(payload)
    });
    let quest = quests.next().ok_or(CoreDialogueQuestClosureError)?;
    if quests.next().is_some() {
        return Err(CoreDialogueQuestClosureError);
    }

    let mut relationships = rpg_snapshot.aggregates.iter().filter_map(|aggregate| {
        let RpgAggregatePayloadV1::Relationship(payload) = &aggregate.payload else {
            return None;
        };
        (payload.source_id == dialogue.speaker_id && payload.target_id == player_id)
            .then_some(payload)
    });
    let relationship = relationships.next().ok_or(CoreDialogueQuestClosureError)?;
    if relationships.next().is_some() {
        return Err(CoreDialogueQuestClosureError);
    }
    let trust = relationship
        .dimensions
        .iter()
        .find(|dimension| dimension.dimension_id.as_str() == CORE_RELATIONSHIP_TRUST_DIMENSION_ID)
        .ok_or(CoreDialogueQuestClosureError)?
        .value;
    if !matches!(
        (dialogue.node_id.as_str(), quest.state_id.as_str(), trust,),
        (
            CORE_DIALOGUE_OFFER_NODE_ID,
            CORE_QUEST_AVAILABLE_STATE_ID,
            0
        ) | (
            CORE_DIALOGUE_ACCEPTED_NODE_ID,
            CORE_QUEST_ACTIVE_STATE_ID,
            CORE_DIALOGUE_QUEST_TRUST_DELTA
        )
    ) {
        return Err(CoreDialogueQuestClosureError);
    }
    if !physics_checkpoint
        .catalog
        .avatar_bindings
        .contains_key(&player_id)
    {
        return Err(CoreDialogueQuestClosureError);
    }
    let mut npc_bodies = physics_checkpoint
        .catalog
        .bodies
        .values()
        .filter(|body| body.body_id.subject_id == dialogue.speaker_id);
    if npc_bodies
        .next()
        .is_none_or(|body| body.motion_kind != PhysicsMotionKindV1::Static)
        || npc_bodies.next().is_some()
    {
        return Err(CoreDialogueQuestClosureError);
    }
    let interaction_principal = IssuerPrincipal::InternalSystem(
        SystemId::new(PLAYER_INTERACTION_SYSTEM_ID)
            .expect("built-in player interaction system identifier is valid"),
    );
    if !principal_registry.is_active(&interaction_principal)
        || !stream_registry.entries.keys().any(|key| {
            key.principal == interaction_principal && key.stream_slot == 0 && key.stream_epoch == 0
        })
    {
        return Err(CoreDialogueQuestClosureError);
    }
    Ok(())
}

fn contains_aggregate(
    snapshot: &RpgSnapshotV2,
    kind: RpgAggregateKindV1,
    persistent_id: crate::ids::PersistentId,
) -> bool {
    snapshot.aggregates.iter().any(|aggregate| {
        aggregate.aggregate_kind == kind && aggregate.persistent_id == persistent_id
    })
}

pub fn world_checkpoint_v4_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<StateRoot, CanonicalError> {
    runtime_snapshot
        .validate()
        .map_err(snapshot_validation_error)?;
    world_checkpoint_v4_state_root_validated(runtime_snapshot, rpg_snapshot, physics_checkpoint)
}

fn world_checkpoint_v4_state_root_validated(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<StateRoot, CanonicalError> {
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            runtime_snapshot.canonical_bytes_validated()?,
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            rpg_snapshot.canonical_bytes()?,
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            physics_checkpoint.canonical_bytes()?,
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    state_root_from_segments(segments)
}

fn world_checkpoint_v4_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
) -> Result<StateRoot, CanonicalError> {
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            components.runtime_snapshot_bytes(),
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            components.rpg_snapshot_bytes(),
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            components.physics_checkpoint_bytes(),
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    state_root_from_segments(segments)
}

pub fn world_checkpoint_with_streaming_v1_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    world_streaming_snapshot.validate()?;
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            runtime_snapshot.canonical_bytes()?,
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            rpg_snapshot.canonical_bytes()?,
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            physics_checkpoint.canonical_bytes()?,
        ),
        (
            crate::world::WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            world_streaming_snapshot.canonical_bytes()?,
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    Ok(state_root_from_segments(segments)?)
}

pub fn world_checkpoint_with_streaming_v1_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    world_streaming_snapshot.validate()?;
    let streaming_bytes = world_streaming_snapshot.canonical_bytes()?;
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            components.runtime_snapshot_bytes(),
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            components.rpg_snapshot_bytes(),
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            components.physics_checkpoint_bytes(),
        ),
        (
            crate::world::WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            streaming_bytes.as_slice(),
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    Ok(state_root_from_segments(segments)?)
}

pub fn world_checkpoint_with_streaming_and_routine_v1_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot: &crate::world_routine::WorldRoutineSnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    world_streaming_snapshot.validate()?;
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            runtime_snapshot.canonical_bytes()?,
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            rpg_snapshot.canonical_bytes()?,
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            physics_checkpoint.canonical_bytes()?,
        ),
        (
            crate::world::WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            world_streaming_snapshot.canonical_bytes()?,
        ),
        (
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            world_routine_snapshot.canonical_bytes()?,
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    Ok(state_root_from_segments(segments)?)
}

pub fn world_checkpoint_with_streaming_and_routine_v1_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot: &crate::world_routine::WorldRoutineSnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    world_streaming_snapshot.validate()?;
    let streaming_bytes = world_streaming_snapshot.canonical_bytes()?;
    let routine_bytes = world_routine_snapshot.canonical_bytes()?;
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            components.runtime_snapshot_bytes(),
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            components.rpg_snapshot_bytes(),
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            components.physics_checkpoint_bytes(),
        ),
        (
            crate::world::WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            streaming_bytes.as_slice(),
        ),
        (
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            routine_bytes.as_slice(),
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    Ok(state_root_from_segments(segments)?)
}

pub fn world_checkpoint_with_world_services_v1_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot_or_none: Option<&crate::world_population::WorldPopulationSnapshotV1>,
) -> Result<StateRoot, WorldCheckpointError> {
    world_streaming_snapshot.validate()?;
    let mut segments = vec![
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            components.runtime_snapshot_bytes().to_vec(),
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            components.rpg_snapshot_bytes().to_vec(),
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            components.physics_checkpoint_bytes().to_vec(),
        ),
        (
            crate::world::WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            world_streaming_snapshot.canonical_bytes()?,
        ),
    ];
    if let Some(routine) = world_routine_snapshot_or_none {
        segments.push((
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            routine.canonical_bytes()?,
        ));
    }
    if let Some(population) = world_population_snapshot_or_none {
        segments.push((
            crate::world_population::WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            crate::world_population::WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
            crate::world_population::WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
            population.canonical_bytes()?,
        ));
    }
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    let segment_refs = segments
        .iter()
        .map(|(owner, schema, segment, bytes)| (*owner, *schema, *segment, bytes.as_slice()))
        .collect::<Vec<_>>();
    Ok(state_root_from_segment_slices(&segment_refs)?)
}

pub fn world_checkpoint_with_cognition_v1_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot_or_none: Option<&crate::world_population::WorldPopulationSnapshotV1>,
    agent_snapshot: &crate::cognition::AgentCognitionSnapshotV1,
    memory_snapshot: &crate::cognition::AgentMemorySnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    world_checkpoint_with_cognition_and_activity_v1_state_root_from_canonical_components(
        components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        world_population_snapshot_or_none,
        None,
        agent_snapshot,
        memory_snapshot,
        None,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the R4d application root keeps every authoritative owner projection explicit"
)]
pub fn world_checkpoint_with_systemic_cognition_v1_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot: &crate::world_population::WorldPopulationSnapshotV1,
    world_activity_snapshot: &crate::world_activity::WorldActivitySnapshotV1,
    agent_snapshot: &crate::cognition::AgentCognitionSnapshotV1,
    memory_snapshot: &crate::cognition::AgentMemorySnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    world_checkpoint_with_cognition_and_activity_v1_state_root_from_canonical_components(
        components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        Some(world_population_snapshot),
        Some(world_activity_snapshot),
        agent_snapshot,
        memory_snapshot,
        None,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the R5a application root keeps every authoritative owner projection explicit"
)]
pub fn world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot: &crate::world_population::WorldPopulationSnapshotV1,
    world_activity_snapshot: &crate::world_activity::WorldActivitySnapshotV1,
    agent_snapshot: &crate::cognition::AgentCognitionSnapshotV1,
    memory_snapshot: &crate::cognition::AgentMemorySnapshotV1,
    physical_animation_snapshot: &crate::physical_animation::PhysicalAnimationSnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    world_checkpoint_with_cognition_and_activity_v1_state_root_from_canonical_components(
        components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        Some(world_population_snapshot),
        Some(world_activity_snapshot),
        agent_snapshot,
        memory_snapshot,
        Some(physical_animation_snapshot),
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the owner-complete state-root helper names every independently hashed segment"
)]
fn world_checkpoint_with_cognition_and_activity_v1_state_root_from_canonical_components(
    components: &WorldCheckpointCanonicalComponentsV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot_or_none: Option<&crate::world_population::WorldPopulationSnapshotV1>,
    world_activity_snapshot_or_none: Option<&crate::world_activity::WorldActivitySnapshotV1>,
    agent_snapshot: &crate::cognition::AgentCognitionSnapshotV1,
    memory_snapshot: &crate::cognition::AgentMemorySnapshotV1,
    physical_animation_snapshot_or_none: Option<
        &crate::physical_animation::PhysicalAnimationSnapshotV1,
    >,
) -> Result<StateRoot, WorldCheckpointError> {
    world_streaming_snapshot.validate()?;
    agent_snapshot
        .validate()
        .map_err(|_| WorldCheckpointError::AgentCognition)?;
    memory_snapshot
        .validate()
        .map_err(|_| WorldCheckpointError::AgentCognition)?;
    if agent_snapshot.subject_id != memory_snapshot.subject_id
        || agent_snapshot.revision != memory_snapshot.revision
    {
        return Err(WorldCheckpointError::AgentCognition);
    }
    let mut segments = vec![
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            components.runtime_snapshot_bytes().to_vec(),
        ),
        (
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::rpg::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            components.rpg_snapshot_bytes().to_vec(),
        ),
        (
            crate::physics::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            components.physics_checkpoint_bytes().to_vec(),
        ),
        (
            crate::world::WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            crate::world::WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            world_streaming_snapshot.canonical_bytes()?,
        ),
        (
            crate::cognition::AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
            crate::cognition::AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
            crate::cognition::AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID,
            agent_snapshot
                .canonical_bytes()
                .map_err(|_| WorldCheckpointError::AgentCognition)?,
        ),
        (
            crate::cognition::AGENT_MEMORY_SNAPSHOT_OWNER_ID,
            crate::cognition::AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
            crate::cognition::AGENT_MEMORY_SNAPSHOT_SEGMENT_ID,
            memory_snapshot
                .canonical_bytes()
                .map_err(|_| WorldCheckpointError::AgentCognition)?,
        ),
    ];
    if let Some(routine) = world_routine_snapshot_or_none {
        segments.push((
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            crate::world_routine::WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            routine.canonical_bytes()?,
        ));
    }
    if let Some(population) = world_population_snapshot_or_none {
        segments.push((
            crate::world_population::WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            crate::world_population::WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
            crate::world_population::WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
            population.canonical_bytes()?,
        ));
    }
    if let Some(activity) = world_activity_snapshot_or_none {
        segments.push((
            crate::world_activity::WORLD_ACTIVITY_SNAPSHOT_OWNER_ID,
            crate::world_activity::WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID,
            crate::world_activity::WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID,
            activity.canonical_bytes()?,
        ));
    }
    if let Some(physical_animation) = physical_animation_snapshot_or_none {
        segments.push((
            crate::physical_animation::PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID,
            crate::physical_animation::PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID,
            crate::physical_animation::PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID,
            physical_animation.canonical_bytes()?,
        ));
    }
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    let segment_refs = segments
        .iter()
        .map(|(owner, schema, segment, bytes)| (*owner, *schema, *segment, bytes.as_slice()))
        .collect::<Vec<_>>();
    Ok(state_root_from_segment_slices(&segment_refs)?)
}

pub fn world_checkpoint_with_world_services_v1_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot_or_none: Option<&crate::world_population::WorldPopulationSnapshotV1>,
) -> Result<StateRoot, WorldCheckpointError> {
    let (_, components) = WorldCheckpointV4::new_with_canonical_components(
        runtime_snapshot.clone(),
        rpg_snapshot.clone(),
        physics_checkpoint.clone(),
    )?;
    world_checkpoint_with_world_services_v1_state_root_from_canonical_components(
        &components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        world_population_snapshot_or_none,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the full cognition application root keeps every authoritative owner projection explicit"
)]
pub fn world_checkpoint_with_cognition_v1_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot_or_none: Option<&crate::world_population::WorldPopulationSnapshotV1>,
    agent_snapshot: &crate::cognition::AgentCognitionSnapshotV1,
    memory_snapshot: &crate::cognition::AgentMemorySnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    let (_, components) = WorldCheckpointV4::new_with_canonical_components(
        runtime_snapshot.clone(),
        rpg_snapshot.clone(),
        physics_checkpoint.clone(),
    )?;
    world_checkpoint_with_cognition_v1_state_root_from_canonical_components(
        &components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        world_population_snapshot_or_none,
        agent_snapshot,
        memory_snapshot,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the R4d application root keeps every authoritative owner projection explicit"
)]
pub fn world_checkpoint_with_systemic_cognition_v1_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
    world_streaming_snapshot: &crate::world::WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<&crate::world_routine::WorldRoutineSnapshotV1>,
    world_population_snapshot: &crate::world_population::WorldPopulationSnapshotV1,
    world_activity_snapshot: &crate::world_activity::WorldActivitySnapshotV1,
    agent_snapshot: &crate::cognition::AgentCognitionSnapshotV1,
    memory_snapshot: &crate::cognition::AgentMemorySnapshotV1,
) -> Result<StateRoot, WorldCheckpointError> {
    let (_, components) = WorldCheckpointV4::new_with_canonical_components(
        runtime_snapshot.clone(),
        rpg_snapshot.clone(),
        physics_checkpoint.clone(),
    )?;
    world_checkpoint_with_systemic_cognition_v1_state_root_from_canonical_components(
        &components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        world_population_snapshot,
        world_activity_snapshot,
        agent_snapshot,
        memory_snapshot,
    )
}
