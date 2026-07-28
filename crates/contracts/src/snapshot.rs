use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    AuthoritativeNumericProfileV1, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_DIALOGUE_OFFER_NODE_ID,
    CORE_DIALOGUE_QUEST_TRUST_DELTA, CORE_QUEST_ACTIVE_STATE_ID, CORE_QUEST_AVAILABLE_STATE_ID,
    CORE_RELATIONSHIP_TRUST_DIMENSION_ID, CanonicalDecodeError, CanonicalDecodeLimits,
    CanonicalError, CanonicalField, CausalIdentityKey, CausalIdentityKind, CommandBodyArchiveV1,
    CommandLedgerError, CommandLedgerHash, CommandLedgerV2, CommandStreamRegistryV1,
    CoreDialogueQuestClosureError, IdentityContractError, IngressAssignmentProfileV1,
    IngressCheckpointV1, InputContractError, IssuerPrincipal, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID, PLAYER_INTERACTION_SYSTEM_ID, PhysicsContractError,
    PhysicsMotionKindV1, PhysicsQuantizationProfileV1, PhysicsWorldCheckpointV1,
    PlayerControllerRegistryV1, PrincipalRegistryV1, ResolvedCoreDialogueQuestBinding,
    RpgAggregateKindV1, RpgAggregatePayloadV1, RpgContractErrorV1, RpgDecodeError,
    RpgRuntimeBindingsV1, RpgSnapshot, RpgSnapshotV2, RuntimeAdmissionLimitsV1,
    RuntimeDeterminismProfileV1, StateRoot, SystemId, TickRateProfileV1, WorldIdentityManifestV1,
    causal_provenance_hash, decode_canonical_segment, encode_canonical_segment, sha256,
};

pub const RUNTIME_SNAPSHOT_SCHEMA_VERSION: u32 = 3;
pub const RUNTIME_SNAPSHOT_OWNER_ID: &str = "nextengine.runtime";
pub const RUNTIME_SNAPSHOT_SCHEMA_ID: &str = "nextengine.runtime-snapshot";
pub const RUNTIME_SNAPSHOT_SEGMENT_ID: &str = "v3";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSnapshotV3 {
    pub next_tick: u64,
    pub committed_event_count: u64,
    pub authoritative_revision: u64,
    pub world_identity: WorldIdentityManifestV1,
    pub principal_registry: PrincipalRegistryV1,
    pub stream_registry: CommandStreamRegistryV1,
    pub runtime_profile: RuntimeDeterminismProfileV1,
    pub admission_limits: RuntimeAdmissionLimitsV1,
    pub tick_rate_profile: TickRateProfileV1,
    pub ingress_assignment_profile: IngressAssignmentProfileV1,
    pub authoritative_numeric_profile: AuthoritativeNumericProfileV1,
    pub physics_quantization_profile: PhysicsQuantizationProfileV1,
    pub player_controller_registry: PlayerControllerRegistryV1,
    pub ingress_checkpoint: IngressCheckpointV1,
    pub rpg_runtime_bindings: RpgRuntimeBindingsV1,
    pub command_ledger: CommandLedgerV2,
    pub body_archive: CommandBodyArchiveV1,
}

pub type RuntimeSnapshot = RuntimeSnapshotV3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCheckpointV3 {
    pub runtime_snapshot: RuntimeSnapshotV3,
    pub rpg_snapshot: RpgSnapshot,
    pub physics_checkpoint: PhysicsWorldCheckpointV1,
    pub state_root: StateRoot,
}

impl WorldCheckpointV3 {
    pub fn new(
        runtime_snapshot: RuntimeSnapshotV3,
        rpg_snapshot: RpgSnapshot,
        physics_checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<Self, WorldCheckpointError> {
        let state_root =
            world_checkpoint_v3_state_root(&runtime_snapshot, &rpg_snapshot, &physics_checkpoint)?;
        let checkpoint = Self {
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            state_root,
        };
        checkpoint.validate()?;
        Ok(checkpoint)
    }

    pub fn validate(&self) -> Result<(), WorldCheckpointError> {
        self.runtime_snapshot.validate()?;
        let rpg_bytes = self.rpg_snapshot.canonical_bytes()?;
        if RpgSnapshot::from_canonical_bytes(&rpg_bytes, CanonicalDecodeLimits::default())?
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
        validate_core_dialogue_quest_world_closure(
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
            || self.state_root
                != world_checkpoint_v3_state_root(
                    &self.runtime_snapshot,
                    &self.rpg_snapshot,
                    &self.physics_checkpoint,
                )?
        {
            return Err(WorldCheckpointError::ClosureMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCheckpointV4 {
    pub runtime_snapshot: RuntimeSnapshotV3,
    pub rpg_snapshot: RpgSnapshotV2,
    pub physics_checkpoint: PhysicsWorldCheckpointV1,
    pub state_root: StateRoot,
}

impl WorldCheckpointV4 {
    pub fn new(
        runtime_snapshot: RuntimeSnapshotV3,
        rpg_snapshot: RpgSnapshotV2,
        physics_checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<Self, WorldCheckpointError> {
        let state_root =
            world_checkpoint_v4_state_root(&runtime_snapshot, &rpg_snapshot, &physics_checkpoint)?;
        let checkpoint = Self {
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            state_root,
        };
        checkpoint.validate()?;
        Ok(checkpoint)
    }

    pub fn validate(&self) -> Result<(), WorldCheckpointError> {
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
        validate_core_dialogue_quest_world_closure_v2(
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
            || self.state_root
                != world_checkpoint_v4_state_root(
                    &self.runtime_snapshot,
                    &self.rpg_snapshot,
                    &self.physics_checkpoint,
                )?
        {
            return Err(WorldCheckpointError::ClosureMismatch);
        }
        Ok(())
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
    persistent_id: crate::PersistentId,
) -> bool {
    snapshot.aggregates.iter().any(|aggregate| {
        aggregate.aggregate_kind == kind && aggregate.persistent_id == persistent_id
    })
}

pub fn validate_core_dialogue_quest_world_closure(
    rpg_snapshot: &RpgSnapshot,
    player_controller_registry: &PlayerControllerRegistryV1,
    principal_registry: &PrincipalRegistryV1,
    stream_registry: &CommandStreamRegistryV1,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<Option<ResolvedCoreDialogueQuestBinding>, CoreDialogueQuestClosureError> {
    if !rpg_snapshot.has_core_dialogue_quest_records() {
        return Ok(None);
    }
    if player_controller_registry.bindings.len() != 1 {
        return Err(CoreDialogueQuestClosureError);
    }
    let controller = player_controller_registry
        .bindings
        .values()
        .next()
        .ok_or(CoreDialogueQuestClosureError)?;
    let binding = rpg_snapshot
        .resolve_core_dialogue_quest_binding(controller.controlled_body_id)?
        .ok_or(CoreDialogueQuestClosureError)?;
    if !physics_checkpoint
        .catalog
        .avatar_bindings
        .contains_key(&binding.player_id)
    {
        return Err(CoreDialogueQuestClosureError);
    }
    let mut npc_bodies = physics_checkpoint
        .catalog
        .bodies
        .values()
        .filter(|body| body.body_id.subject_id == binding.npc_id);
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
    Ok(Some(binding))
}

pub fn world_checkpoint_v3_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshot,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<StateRoot, CanonicalError> {
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            runtime_snapshot.canonical_bytes()?,
        ),
        (
            crate::RPG_SNAPSHOT_OWNER_ID,
            crate::RPG_SNAPSHOT_SCHEMA_ID,
            crate::RPG_SNAPSHOT_SEGMENT_ID,
            rpg_snapshot.canonical_bytes()?,
        ),
        (
            crate::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            physics_checkpoint.canonical_bytes()?,
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    state_root_from_segments(segments)
}

pub fn world_checkpoint_v4_state_root(
    runtime_snapshot: &RuntimeSnapshotV3,
    rpg_snapshot: &RpgSnapshotV2,
    physics_checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<StateRoot, CanonicalError> {
    let mut segments = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            runtime_snapshot.canonical_bytes()?,
        ),
        (
            crate::RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            crate::RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            crate::RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            rpg_snapshot.canonical_bytes()?,
        ),
        (
            crate::PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            physics_checkpoint.canonical_bytes()?,
        ),
    ];
    segments.sort_by_key(|(owner, schema, segment, _)| (*owner, *schema, *segment));
    state_root_from_segments(segments)
}

fn state_root_from_segments<const N: usize>(
    segments: [(&str, &str, &str, Vec<u8>); N],
) -> Result<StateRoot, CanonicalError> {
    let leaf_count = u64::try_from(segments.len()).map_err(|_| CanonicalError::LengthOverflow)?;
    let mut nodes = Vec::with_capacity(segments.len());
    for (owner, schema, segment, bytes) in segments {
        let mut segment_preimage = Vec::new();
        segment_preimage.extend_from_slice(b"nextengine.state-segment.v1\0");
        segment_preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        segment_preimage.extend_from_slice(&bytes);

        let mut leaf_preimage = Vec::new();
        leaf_preimage.extend_from_slice(b"nextengine.state-leaf.v1\0");
        extend_state_root_identifier(&mut leaf_preimage, owner)?;
        extend_state_root_identifier(&mut leaf_preimage, schema)?;
        extend_state_root_identifier(&mut leaf_preimage, segment)?;
        leaf_preimage.extend_from_slice(&sha256(&segment_preimage));
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
    Rpg(RpgDecodeError),
    RpgV2(RpgContractErrorV1),
    Physics(PhysicsContractError),
    CoreInteractionClosure(CoreDialogueQuestClosureError),
    ClosureMismatch,
}

impl WorldCheckpointError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::ClosureMismatch => "WORLD_CHECKPOINT_CLOSURE_CORRUPT",
            Self::Runtime(error) => error.stable_code(),
            Self::Rpg(_) => "WORLD_CHECKPOINT_RPG_CORRUPT",
            Self::RpgV2(error) => error.stable_code(),
            Self::Physics(_) => "WORLD_CHECKPOINT_PHYSICS_CORRUPT",
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

impl From<RpgDecodeError> for WorldCheckpointError {
    fn from(error: RpgDecodeError) -> Self {
        Self::Rpg(error)
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

impl From<CoreDialogueQuestClosureError> for WorldCheckpointError {
    fn from(error: CoreDialogueQuestClosureError) -> Self {
        Self::CoreInteractionClosure(error)
    }
}

impl RuntimeSnapshotV3 {
    pub fn validate(&self) -> Result<(), SnapshotDecodeError> {
        self.world_identity.validate()?;
        self.principal_registry.validate()?;
        self.stream_registry.validate()?;
        self.runtime_profile.validate()?;
        self.admission_limits.validate()?;
        self.tick_rate_profile.validate()?;
        self.ingress_assignment_profile.validate()?;
        self.authoritative_numeric_profile.validate()?;
        self.physics_quantization_profile.validate()?;
        self.player_controller_registry.validate()?;
        self.ingress_checkpoint.validate(&self.admission_limits)?;
        self.rpg_runtime_bindings.validate()?;
        let profile_hash = self.runtime_profile.profile_hash()?;
        let world = self.world_identity.world_namespace;
        if self.world_identity.runtime_determinism_profile_hash != profile_hash
            || self.principal_registry.world_namespace != world
            || self.stream_registry.world_namespace != world
            || self.player_controller_registry.world_namespace != world
            || self.command_ledger.world_namespace != world
            || self.command_ledger.runtime_determinism_profile_hash != profile_hash
            || self.command_ledger.command_kind_registry_hash
                != self.runtime_profile.command_kind_registry_hash
        {
            return Err(SnapshotDecodeError::ClosureMismatch);
        }
        if self.runtime_profile.admission_limits_profile_hash
            != self.admission_limits.profile_hash()?
            || self.runtime_profile.tick_rate_profile_hash
                != self.tick_rate_profile.profile_hash()?
            || self.runtime_profile.ingress_assignment_profile_hash
                != self.ingress_assignment_profile.profile_hash()?
            || self.runtime_profile.numeric_profile_hash
                != self.authoritative_numeric_profile.profile_hash()?
            || self.runtime_profile.physics_quantization_profile_hash
                != self.physics_quantization_profile.profile_hash()?
            || self.ingress_assignment_profile.admission_limits_hash
                != self.admission_limits.profile_hash()?
            || self
                .authoritative_numeric_profile
                .physics_quantization_profile_hash
                != self.physics_quantization_profile.profile_hash()?
            || self.ingress_checkpoint.current_tick != self.next_tick
        {
            return Err(SnapshotDecodeError::ProfileClosureMismatch);
        }
        for binding in self.player_controller_registry.bindings.values() {
            if !self.principal_registry.is_active(&binding.principal)
                || self.stream_registry.entries.iter().all(|(key, stream)| {
                    key.principal != binding.principal || stream != &binding.command_stream_id
                })
            {
                return Err(SnapshotDecodeError::ControllerRegistryMismatch);
            }
        }
        for (stream_key, stream_id) in &self.stream_registry.entries {
            if !self.principal_registry.is_active(&stream_key.principal) {
                return Err(SnapshotDecodeError::InactivePrincipal);
            }
            let stream = self
                .command_ledger
                .streams
                .get(stream_id)
                .ok_or(SnapshotDecodeError::StreamRegistryMismatch)?;
            if stream.issuer != stream_key.principal
                || stream.stream_slot != stream_key.stream_slot
                || stream.stream_epoch != stream_key.stream_epoch
            {
                return Err(SnapshotDecodeError::StreamRegistryMismatch);
            }
            let principal_bytes = stream_key.principal.canonical_bytes()?;
            let mut provenance = Vec::new();
            provenance.extend_from_slice(world.as_bytes());
            provenance.extend_from_slice(
                &u64::try_from(principal_bytes.len())
                    .map_err(|_| CanonicalError::LengthOverflow)?
                    .to_le_bytes(),
            );
            provenance.extend_from_slice(&principal_bytes);
            provenance.extend_from_slice(&stream_key.stream_slot.to_le_bytes());
            provenance.extend_from_slice(&stream_key.stream_epoch.to_le_bytes());
            let expected = causal_provenance_hash(CausalIdentityKind::CommandStream, &provenance)?;
            if self
                .command_ledger
                .causal_identity_registry
                .bindings
                .get(&CausalIdentityKey {
                    identity_kind: CausalIdentityKind::CommandStream,
                    identity_bytes: *stream_id.as_bytes(),
                })
                != Some(&expected)
            {
                return Err(SnapshotDecodeError::CausalRegistryMismatch);
            }
        }
        if self.command_ledger.streams.len() != self.stream_registry.entries.len() {
            return Err(SnapshotDecodeError::StreamRegistryMismatch);
        }
        for (principal, record) in &self.principal_registry.principals {
            if let IssuerPrincipal::Player(id) = principal
                && self
                    .command_ledger
                    .causal_identity_registry
                    .bindings
                    .get(&CausalIdentityKey {
                        identity_kind: CausalIdentityKind::PlayerPrincipal,
                        identity_bytes: *id.as_bytes(),
                    })
                    != Some(&record.provenance_hash)
            {
                return Err(SnapshotDecodeError::CausalRegistryMismatch);
            }
        }
        let domain_event_count = self
            .command_ledger
            .causal_identity_registry
            .bindings
            .keys()
            .filter(|key| key.identity_kind == CausalIdentityKind::DomainEvent)
            .count();
        if u64::try_from(domain_event_count).map_err(|_| CanonicalError::LengthOverflow)?
            != self.committed_event_count
        {
            return Err(SnapshotDecodeError::CausalRegistryMismatch);
        }
        self.command_ledger.validate(&self.body_archive)?;
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate().map_err(|error| match error {
            SnapshotDecodeError::Canonicalization(error) => error,
            SnapshotDecodeError::Identity(IdentityContractError::Canonical(error)) => error,
            SnapshotDecodeError::Ledger(CommandLedgerError::Canonical(error)) => error,
            _ => CanonicalError::DuplicateSequenceValue,
        })?;
        let ledger_bytes = self
            .command_ledger
            .canonical_bytes(&self.body_archive)
            .map_err(|error| match error {
                CommandLedgerError::Canonical(error) => error,
                _ => CanonicalError::DuplicateSequenceValue,
            })?;
        encode_canonical_segment(
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    u16::try_from(RUNTIME_SNAPSHOT_SCHEMA_VERSION)
                        .map_err(|_| CanonicalError::LengthOverflow)?
                        .to_le_bytes()
                        .to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_U64, self.next_tick.to_le_bytes().to_vec()),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_U64,
                    self.committed_event_count.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_U64,
                    self.authoritative_revision.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_BYTES,
                    self.world_identity.canonical_bytes()?,
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_BYTES,
                    self.principal_registry.canonical_bytes()?,
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_BYTES,
                    self.stream_registry.canonical_bytes()?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_BYTES,
                    self.runtime_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_BYTES,
                    self.admission_limits.canonical_bytes()?,
                ),
                CanonicalField::new(
                    10,
                    CANONICAL_TYPE_BYTES,
                    self.tick_rate_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    11,
                    CANONICAL_TYPE_BYTES,
                    self.ingress_assignment_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    12,
                    CANONICAL_TYPE_BYTES,
                    self.authoritative_numeric_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    13,
                    CANONICAL_TYPE_BYTES,
                    self.physics_quantization_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    14,
                    CANONICAL_TYPE_BYTES,
                    self.player_controller_registry.canonical_bytes()?,
                ),
                CanonicalField::new(
                    15,
                    CANONICAL_TYPE_BYTES,
                    self.ingress_checkpoint.canonical_bytes()?,
                ),
                CanonicalField::new(16, CANONICAL_TYPE_BYTES, ledger_bytes),
                CanonicalField::new(
                    17,
                    CANONICAL_TYPE_BYTES,
                    self.body_archive.canonical_bytes()?,
                ),
                CanonicalField::new(
                    18,
                    CANONICAL_TYPE_BYTES,
                    self.rpg_runtime_bindings.canonical_bytes()?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SnapshotDecodeError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != RUNTIME_SNAPSHOT_OWNER_ID
            || segment.schema_id != RUNTIME_SNAPSHOT_SCHEMA_ID
            || segment.segment_id != RUNTIME_SNAPSHOT_SEGMENT_ID
        {
            return Err(SnapshotDecodeError::WrongEnvelope);
        }
        let version_field = segment
            .field(1)
            .ok_or(SnapshotDecodeError::MissingField(1))?;
        let version = match version_field.type_tag {
            CANONICAL_TYPE_U16 if version_field.payload.len() == 2 => u32::from(
                u16::from_le_bytes(version_field.payload.as_slice().try_into().map_err(|_| {
                    SnapshotDecodeError::FieldLength {
                        field_id: 1,
                        expected: 2,
                        actual: version_field.payload.len(),
                    }
                })?),
            ),
            CANONICAL_TYPE_U32 if version_field.payload.len() == 4 => {
                u32::from_le_bytes(version_field.payload.as_slice().try_into().map_err(|_| {
                    SnapshotDecodeError::FieldLength {
                        field_id: 1,
                        expected: 4,
                        actual: version_field.payload.len(),
                    }
                })?)
            }
            _ => {
                return Err(SnapshotDecodeError::FieldType {
                    field_id: 1,
                    expected: CANONICAL_TYPE_U16,
                    actual: version_field.type_tag,
                });
            }
        };
        if version != RUNTIME_SNAPSHOT_SCHEMA_VERSION {
            return Err(SnapshotDecodeError::UnsupportedSchemaVersion(version));
        }
        require_fields(&segment)?;
        let admission_limits =
            RuntimeAdmissionLimitsV1::from_canonical_bytes(field(&segment, 9)?, limits)?;
        let archive = CommandBodyArchiveV1::from_canonical_bytes(field(&segment, 17)?, limits)?;
        let snapshot = Self {
            next_tick: read_u64(&segment, 2)?,
            committed_event_count: read_u64(&segment, 3)?,
            authoritative_revision: read_u64(&segment, 4)?,
            world_identity: WorldIdentityManifestV1::from_canonical_bytes(
                field(&segment, 5)?,
                limits,
            )?,
            principal_registry: PrincipalRegistryV1::from_canonical_bytes(
                field(&segment, 6)?,
                limits,
            )?,
            stream_registry: CommandStreamRegistryV1::from_canonical_bytes(
                field(&segment, 7)?,
                limits,
            )?,
            runtime_profile: RuntimeDeterminismProfileV1::from_canonical_bytes(
                field(&segment, 8)?,
                limits,
            )?,
            admission_limits,
            tick_rate_profile: TickRateProfileV1::from_canonical_bytes(
                field(&segment, 10)?,
                limits,
            )?,
            ingress_assignment_profile: IngressAssignmentProfileV1::from_canonical_bytes(
                field(&segment, 11)?,
                limits,
            )?,
            authoritative_numeric_profile: AuthoritativeNumericProfileV1::from_canonical_bytes(
                field(&segment, 12)?,
                limits,
            )?,
            physics_quantization_profile: PhysicsQuantizationProfileV1::from_canonical_bytes(
                field(&segment, 13)?,
                limits,
            )?,
            player_controller_registry: PlayerControllerRegistryV1::from_canonical_bytes(
                field(&segment, 14)?,
                limits,
            )?,
            ingress_checkpoint: IngressCheckpointV1::from_canonical_bytes(
                field(&segment, 15)?,
                limits,
                &admission_limits,
            )?,
            rpg_runtime_bindings: RpgRuntimeBindingsV1::from_canonical_bytes(field(&segment, 18)?)?,
            command_ledger: CommandLedgerV2::from_canonical_bytes(
                field(&segment, 16)?,
                &archive,
                limits,
            )?,
            body_archive: archive,
        };
        snapshot.validate()?;
        if snapshot.canonical_bytes()? != bytes {
            return Err(SnapshotDecodeError::NonCanonicalEncoding);
        }
        Ok(snapshot)
    }

    pub fn command_ledger_hash(&self) -> Result<CommandLedgerHash, CanonicalError> {
        self.command_ledger
            .command_ledger_hash(&self.body_archive)
            .map_err(|error| match error {
                CommandLedgerError::Canonical(error) => error,
                _ => CanonicalError::DuplicateSequenceValue,
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SnapshotDecodeError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identity(IdentityContractError),
    Ledger(CommandLedgerError),
    Input(InputContractError),
    Physics(PhysicsContractError),
    RpgBindings(RpgContractErrorV1),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    FieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedSchemaVersion(u32),
    ClosureMismatch,
    ProfileClosureMismatch,
    ControllerRegistryMismatch,
    InactivePrincipal,
    StreamRegistryMismatch,
    CausalRegistryMismatch,
    NonCanonicalEncoding,
}

impl SnapshotDecodeError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion(_) => "UNSUPPORTED_RUNTIME_SNAPSHOT_VERSION",
            Self::ClosureMismatch
            | Self::ProfileClosureMismatch
            | Self::ControllerRegistryMismatch
            | Self::InactivePrincipal
            | Self::StreamRegistryMismatch
            | Self::CausalRegistryMismatch
            | Self::Ledger(_)
            | Self::RpgBindings(_) => "RUNTIME_SNAPSHOT_CLOSURE_CORRUPT",
            _ => "RUNTIME_SNAPSHOT_INVALID",
        }
    }
}

impl Display for SnapshotDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "snapshot encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "snapshot canonicalization failed: {error}")
            }
            Self::Identity(error) => write!(formatter, "snapshot identity is invalid: {error}"),
            Self::Ledger(error) => write!(formatter, "snapshot ledger is invalid: {error}"),
            Self::Input(error) => write!(formatter, "snapshot ingress is invalid: {error}"),
            Self::Physics(error) => {
                write!(formatter, "snapshot physics profile is invalid: {error}")
            }
            Self::RpgBindings(error) => {
                write!(
                    formatter,
                    "snapshot RPG runtime bindings are invalid: {error}"
                )
            }
            Self::WrongEnvelope => formatter.write_str("snapshot envelope does not match V3"),
            Self::UnknownField(id) => write!(formatter, "unknown snapshot field {id}"),
            Self::MissingField(id) => write!(formatter, "missing snapshot field {id}"),
            Self::FieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "snapshot field {field_id} has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::FieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "snapshot field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported snapshot schema version {version}")
            }
            Self::ClosureMismatch => {
                formatter.write_str("snapshot world/profile/registry closure does not match")
            }
            Self::ProfileClosureMismatch => {
                formatter.write_str("snapshot component profile closure does not match")
            }
            Self::ControllerRegistryMismatch => {
                formatter.write_str("snapshot controller registry closure does not match")
            }
            Self::InactivePrincipal => {
                formatter.write_str("snapshot stream references an inactive principal")
            }
            Self::StreamRegistryMismatch => {
                formatter.write_str("snapshot stream registry does not match ledger streams")
            }
            Self::CausalRegistryMismatch => {
                formatter.write_str("snapshot causal registry does not close identity provenance")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("snapshot does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for SnapshotDecodeError {}

impl From<CanonicalDecodeError> for SnapshotDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for SnapshotDecodeError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<IdentityContractError> for SnapshotDecodeError {
    fn from(error: IdentityContractError) -> Self {
        Self::Identity(error)
    }
}

impl From<CommandLedgerError> for SnapshotDecodeError {
    fn from(error: CommandLedgerError) -> Self {
        Self::Ledger(error)
    }
}

impl From<InputContractError> for SnapshotDecodeError {
    fn from(error: InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<PhysicsContractError> for SnapshotDecodeError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<RpgContractErrorV1> for SnapshotDecodeError {
    fn from(error: RpgContractErrorV1) -> Self {
        Self::RpgBindings(error)
    }
}

fn require_fields(segment: &crate::DecodedCanonicalSegment) -> Result<(), SnapshotDecodeError> {
    const EXPECTED: [(u32, u8); 18] = [
        (1, CANONICAL_TYPE_U16),
        (2, CANONICAL_TYPE_U64),
        (3, CANONICAL_TYPE_U64),
        (4, CANONICAL_TYPE_U64),
        (5, CANONICAL_TYPE_BYTES),
        (6, CANONICAL_TYPE_BYTES),
        (7, CANONICAL_TYPE_BYTES),
        (8, CANONICAL_TYPE_BYTES),
        (9, CANONICAL_TYPE_BYTES),
        (10, CANONICAL_TYPE_BYTES),
        (11, CANONICAL_TYPE_BYTES),
        (12, CANONICAL_TYPE_BYTES),
        (13, CANONICAL_TYPE_BYTES),
        (14, CANONICAL_TYPE_BYTES),
        (15, CANONICAL_TYPE_BYTES),
        (16, CANONICAL_TYPE_BYTES),
        (17, CANONICAL_TYPE_BYTES),
        (18, CANONICAL_TYPE_BYTES),
    ];
    for actual in &segment.fields {
        if !EXPECTED.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(SnapshotDecodeError::UnknownField(actual.field_id));
        }
    }
    for (id, expected) in EXPECTED {
        let actual = segment
            .field(id)
            .ok_or(SnapshotDecodeError::MissingField(id))?;
        if actual.type_tag != expected {
            return Err(SnapshotDecodeError::FieldType {
                field_id: id,
                expected,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

fn field(segment: &crate::DecodedCanonicalSegment, id: u32) -> Result<&[u8], SnapshotDecodeError> {
    Ok(&segment
        .field(id)
        .ok_or(SnapshotDecodeError::MissingField(id))?
        .payload)
}

fn read_u64(segment: &crate::DecodedCanonicalSegment, id: u32) -> Result<u64, SnapshotDecodeError> {
    let payload = field(segment, id)?;
    Ok(u64::from_le_bytes(payload.try_into().map_err(|_| {
        SnapshotDecodeError::FieldLength {
            field_id: id,
            expected: 8,
            actual: payload.len(),
        }
    })?))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        AuthoritativeNumericProfileV1, CanonicalDecodeLimits, CommandLedgerV2,
        CommandStreamLedgerV2, ContentHash, IngressAssignmentProfileV1, IngressCheckpointV1,
        IssuerPrincipal, PhysicsQuantizationProfileV1, PlayerControllerRegistryV1,
        PlayerPrincipalId, PrincipalRecordV1, PrincipalStatus, ProjectId, RuntimeAdmissionLimitsV1,
        RuntimeDeterminismProfileV1, SchemaId, TickRateProfileV1, WorldIdentityManifestV1,
    };

    use super::*;

    fn fixture() -> RuntimeSnapshotV3 {
        let command_hash = ContentHash::from_bytes([7; 32]);
        let profile = RuntimeDeterminismProfileV1::bootstrap_default(command_hash);
        let world = WorldIdentityManifestV1::new(
            ProjectId::new("nextengine.snapshot-fixture").expect("project id"),
            [1; 32],
            [2; 32],
            profile.profile_hash().expect("profile hash"),
        )
        .expect("world identity");
        let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([3; 16]));
        let mut principals = PrincipalRegistryV1::empty(world.world_namespace);
        principals
            .register(
                principal.clone(),
                PrincipalRecordV1 {
                    provenance_hash: ContentHash::from_bytes([4; 32]),
                    capability_subject_id: SchemaId::new("fixture.player").expect("subject"),
                    status: PrincipalStatus::Active,
                },
            )
            .expect("principal");
        let mut streams = CommandStreamRegistryV1::empty(world.world_namespace);
        let stream_id = streams
            .allocate_stream(principal.clone())
            .expect("stream allocation");
        let (mut ledger, archive) = CommandLedgerV2::empty(
            world.world_namespace,
            command_hash,
            profile.profile_hash().expect("profile hash"),
        )
        .expect("empty ledger");
        ledger.streams.insert(
            stream_id,
            CommandStreamLedgerV2::genesis(stream_id, principal.clone(), 0, 0),
        );
        ledger
            .causal_identity_registry
            .compare_or_insert(
                CausalIdentityKey {
                    identity_kind: CausalIdentityKind::PlayerPrincipal,
                    identity_bytes: match &principal {
                        IssuerPrincipal::Player(id) => *id.as_bytes(),
                        _ => unreachable!("fixture principal is a player"),
                    },
                },
                ContentHash::from_bytes([4; 32]),
            )
            .expect("player provenance");
        let principal_bytes = principal.canonical_bytes().expect("principal bytes");
        let mut provenance = Vec::new();
        provenance.extend_from_slice(world.world_namespace.as_bytes());
        provenance.extend_from_slice(&(principal_bytes.len() as u64).to_le_bytes());
        provenance.extend_from_slice(&principal_bytes);
        provenance.extend_from_slice(&0_u32.to_le_bytes());
        provenance.extend_from_slice(&0_u32.to_le_bytes());
        ledger
            .causal_identity_registry
            .compare_or_insert_provenance(
                CausalIdentityKind::CommandStream,
                *stream_id.as_bytes(),
                &provenance,
            )
            .expect("stream provenance");
        let admission_limits = RuntimeAdmissionLimitsV1::default();
        let tick_rate_profile = TickRateProfileV1::at_30_hz();
        let ingress_assignment_profile =
            IngressAssignmentProfileV1::core_v1(&admission_limits).expect("ingress profile");
        let physics_quantization_profile =
            PhysicsQuantizationProfileV1::capsule_reference_v1().expect("physics profile");
        let authoritative_numeric_profile =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&physics_quantization_profile)
                .expect("numeric profile");
        let world_namespace = world.world_namespace;
        RuntimeSnapshotV3 {
            next_tick: 5,
            committed_event_count: 0,
            authoritative_revision: 3,
            world_identity: world,
            principal_registry: principals,
            stream_registry: streams,
            runtime_profile: profile,
            admission_limits,
            tick_rate_profile,
            ingress_assignment_profile,
            authoritative_numeric_profile,
            physics_quantization_profile,
            player_controller_registry: PlayerControllerRegistryV1 {
                schema_version: 1,
                world_namespace,
                bindings: BTreeMap::new(),
            },
            ingress_checkpoint: IngressCheckpointV1 {
                schema_version: 1,
                current_tick: 5,
                current_generation: 5,
                current_samples: Vec::new(),
                next_samples: Vec::new(),
                last_closed_batch_hash: None,
            },
            rpg_runtime_bindings: RpgRuntimeBindingsV1 {
                project_composition_lock_hash: ContentHash::from_bytes([21; 32]),
                schema_registry_hash: ContentHash::from_bytes([22; 32]),
                budget_policy_hash: ContentHash::from_bytes([23; 32]),
                active_definition_policy_hashes: vec![ContentHash::from_bytes([24; 32])],
            },
            command_ledger: ledger,
            body_archive: archive,
        }
    }

    #[test]
    fn v3_snapshot_round_trip_is_byte_exact() {
        let snapshot = fixture();
        let bytes = snapshot.canonical_bytes().expect("snapshot bytes");
        let decoded =
            RuntimeSnapshotV3::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("snapshot decodes");
        assert_eq!(decoded, snapshot);
        assert_eq!(decoded.canonical_bytes().expect("decoded bytes"), bytes);
    }

    #[test]
    fn v3_snapshot_rejects_ingress_controller_and_profile_closure_corruption() {
        let mut corrupt_ingress = fixture();
        corrupt_ingress.ingress_checkpoint.current_tick -= 1;
        assert_eq!(
            corrupt_ingress.validate(),
            Err(SnapshotDecodeError::ProfileClosureMismatch)
        );

        let mut corrupt_controller = fixture();
        corrupt_controller
            .player_controller_registry
            .world_namespace = crate::WorldNamespaceId::from_bytes([0xff; 16]);
        assert_eq!(
            corrupt_controller.validate(),
            Err(SnapshotDecodeError::ClosureMismatch)
        );

        let mut corrupt_profile = fixture();
        corrupt_profile
            .tick_rate_profile
            .physics_substeps_per_gameplay_tick = 4;
        assert_eq!(
            corrupt_profile.validate(),
            Err(SnapshotDecodeError::ProfileClosureMismatch)
        );
    }

    #[test]
    fn v1_is_rejected_before_nested_state_decoding() {
        let bytes = encode_canonical_segment(
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            [CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                1_u32.to_le_bytes().to_vec(),
            )],
        )
        .expect("legacy-shaped bytes");
        let error =
            RuntimeSnapshotV3::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect_err("V1 must be rejected");
        assert_eq!(error.stable_code(), "UNSUPPORTED_RUNTIME_SNAPSHOT_VERSION");
    }
}
