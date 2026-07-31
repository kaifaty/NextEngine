use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::canonical::{CanonicalDecodeLimits, CanonicalError, sha256};
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
    rpg_snapshot: Arc<[u8]>,
    physics_checkpoint: Arc<[u8]>,
}

impl WorldCheckpointCanonicalComponentsV1 {
    #[must_use]
    pub fn runtime_snapshot_bytes(&self) -> &[u8] {
        &self.runtime_snapshot
    }

    #[must_use]
    pub fn rpg_snapshot_bytes(&self) -> &[u8] {
        &self.rpg_snapshot
    }

    pub fn command_ledger_hash(&self) -> Result<CommandLedgerHash, CanonicalError> {
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest as _;
        hasher.update(b"nextengine.command-ledger.v2\0");
        hasher.update(
            u64::try_from(self.command_ledger.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        hasher.update(&self.command_ledger);
        Ok(command_ledger_hash_from_bytes(hasher.finalize().into()))
    }

    #[must_use]
    pub fn physics_checkpoint_bytes(&self) -> &[u8] {
        &self.physics_checkpoint
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
        let mut checkpoint = Self {
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            state_root: StateRoot::default(),
        };
        let rpg_snapshot = checkpoint.validate_components_with_rpg_bytes()?;
        let (runtime_snapshot, command_ledger) = checkpoint
            .runtime_snapshot
            .canonical_bytes_and_ledger_bytes_validated()?;
        let components = WorldCheckpointCanonicalComponentsV1 {
            runtime_snapshot: Arc::from(runtime_snapshot),
            command_ledger: Arc::from(command_ledger),
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
        self.validate_components_with_rpg_bytes().map(drop)
    }

    fn validate_components_with_rpg_bytes(&self) -> Result<Vec<u8>, WorldCheckpointError> {
        self.runtime_snapshot.validate()?;
        let rpg_bytes = self.rpg_snapshot.canonical_bytes()?;
        if RpgSnapshotV2::from_canonical_bytes(&rpg_bytes, CanonicalDecodeLimits::default())?
            != self.rpg_snapshot
        {
            return Err(WorldCheckpointError::ClosureMismatch);
        }
        self.physics_checkpoint.validate()?;
        self.physics_checkpoint.snapshot.validate_profile_closure(
            &self.physics_checkpoint.catalog,
            &self.runtime_snapshot.tick_rate_profile,
            &self.runtime_snapshot.authoritative_numeric_profile,
            &self.runtime_snapshot.physics_quantization_profile,
        )?;
        super::validate_core_dialogue_quest_world_closure_v2(
            &self.rpg_snapshot,
            &self.runtime_snapshot.player_controller_registry,
            &self.runtime_snapshot.principal_registry,
            &self.runtime_snapshot.stream_registry,
            &self.physics_checkpoint,
        )?;
        let expected_physics_tick = self
            .runtime_snapshot
            .next_tick
            .checked_mul(u64::from(
                self.runtime_snapshot
                    .tick_rate_profile
                    .physics_substeps_per_gameplay_tick,
            ))
            .ok_or(WorldCheckpointError::ClosureMismatch)?;
        let bindings_close = self
            .runtime_snapshot
            .player_controller_registry
            .bindings
            .values()
            .all(|binding| {
                self.physics_checkpoint
                    .catalog
                    .avatar_bindings
                    .get(&binding.controlled_body_id)
                    .is_some_and(|body_id| {
                        self.physics_checkpoint
                            .snapshot
                            .sorted_body_states
                            .contains_key(body_id)
                    })
            });
        if self.physics_checkpoint.snapshot.checkpoint_revision
            != self.runtime_snapshot.authoritative_revision
            || self.physics_checkpoint.snapshot.physics_tick != expected_physics_tick
            || !bindings_close
        {
            return Err(WorldCheckpointError::ClosureMismatch);
        }
        Ok(rpg_bytes)
    }
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

fn state_root_from_segments<const N: usize, B: AsRef<[u8]>>(
    segments: [(&str, &str, &str, B); N],
) -> Result<StateRoot, CanonicalError> {
    let leaf_count = u64::try_from(segments.len()).map_err(|_| CanonicalError::LengthOverflow)?;
    let mut nodes = Vec::with_capacity(segments.len());
    for (owner, schema, segment, bytes) in segments {
        let bytes = bytes.as_ref();
        let mut segment_hasher = sha2::Sha256::new();
        use sha2::Digest as _;
        segment_hasher.update(b"nextengine.state-segment.v1\0");
        segment_hasher.update(
            u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        segment_hasher.update(bytes);

        let mut leaf_preimage = Vec::new();
        leaf_preimage.extend_from_slice(b"nextengine.state-leaf.v1\0");
        extend_state_root_identifier(&mut leaf_preimage, owner)?;
        extend_state_root_identifier(&mut leaf_preimage, schema)?;
        extend_state_root_identifier(&mut leaf_preimage, segment)?;
        leaf_preimage.extend_from_slice(&segment_hasher.finalize());
        nodes.push(sha256(&leaf_preimage));
    }
    while nodes.len() > 1 {
        let mut parents = Vec::with_capacity(nodes.len().div_ceil(2));
        for pair in nodes.chunks(2) {
            let mut preimage = Vec::new();
            if let [left, right] = pair {
                preimage.extend_from_slice(b"nextengine.state-node.v1\0");
                preimage.extend_from_slice(left);
                preimage.extend_from_slice(right);
            } else {
                preimage.extend_from_slice(b"nextengine.state-carry.v1\0");
                preimage.extend_from_slice(&pair[0]);
            }
            parents.push(sha256(&preimage));
        }
        nodes = parents;
    }
    let mut root_preimage = Vec::new();
    root_preimage.extend_from_slice(b"nextengine.state-root.v1\0");
    root_preimage.extend_from_slice(&leaf_count.to_le_bytes());
    root_preimage.extend_from_slice(
        nodes
            .first()
            .expect("a world checkpoint always contains owner segments"),
    );
    Ok(StateRoot::from_bytes(sha256(&root_preimage)))
}

fn extend_state_root_identifier(
    target: &mut Vec<u8>,
    identifier: &str,
) -> Result<(), CanonicalError> {
    target.extend_from_slice(
        &u32::try_from(identifier.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    target.extend_from_slice(identifier.as_bytes());
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldCheckpointError {
    Canonicalization(CanonicalError),
    Runtime(SnapshotDecodeError),
    RpgV2(RpgContractErrorV1),
    Physics(PhysicsContractError),
    WorldStreaming(crate::world::WorldStreamingContractError),
    CoreInteractionClosure(CoreDialogueQuestClosureError),
    ClosureMismatch,
}

impl WorldCheckpointError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::ClosureMismatch => "WORLD_CHECKPOINT_CLOSURE_CORRUPT",
            Self::Runtime(error) => error.stable_code(),
            Self::RpgV2(error) => error.stable_code(),
            Self::Physics(_) => "WORLD_CHECKPOINT_PHYSICS_CORRUPT",
            Self::WorldStreaming(error) => match error {
                crate::world::WorldStreamingContractError::UnsupportedVersion(_) => {
                    "WORLD_STREAM_SCHEMA_UNSUPPORTED"
                }
                _ => "WORLD_CHECKPOINT_STREAMING_CORRUPT",
            },
            Self::CoreInteractionClosure(error) => error.stable_code(),
            Self::Canonicalization(_) => "WORLD_CHECKPOINT_CANONICALIZATION_FAILED",
        }
    }
}

impl Display for WorldCheckpointError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for WorldCheckpointError {}

impl From<CanonicalError> for WorldCheckpointError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<SnapshotDecodeError> for WorldCheckpointError {
    fn from(error: SnapshotDecodeError) -> Self {
        Self::Runtime(error)
    }
}

impl From<RpgContractErrorV1> for WorldCheckpointError {
    fn from(error: RpgContractErrorV1) -> Self {
        Self::RpgV2(error)
    }
}

impl From<PhysicsContractError> for WorldCheckpointError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<crate::world::WorldStreamingContractError> for WorldCheckpointError {
    fn from(error: crate::world::WorldStreamingContractError) -> Self {
        Self::WorldStreaming(error)
    }
}

impl From<CoreDialogueQuestClosureError> for WorldCheckpointError {
    fn from(error: CoreDialogueQuestClosureError) -> Self {
        Self::CoreInteractionClosure(error)
    }
}
