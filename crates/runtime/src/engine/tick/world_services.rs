use next_contracts::ids::{InputSourceId, SchemaId, StateRoot};
use next_contracts::input::{ActionMapManifestV1, InputContextStackV1};
use next_contracts::persistence::SaveSegmentDescriptor;
use next_contracts::physics::{
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
};
use next_contracts::rpg::{
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION, RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
};
use next_contracts::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, WorldCheckpointCanonicalComponentsV1,
    world_checkpoint_with_streaming_and_routine_v1_state_root_from_canonical_components,
    world_checkpoint_with_streaming_v1_state_root_from_canonical_components,
};
use next_contracts::world::{
    WORLD_STREAMING_SNAPSHOT_OWNER_ID, WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
    WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION, WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
    WorldStreamingSnapshotV1,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_SCHEMA_VERSION, WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
    WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID, WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID, WorldRoutineSnapshotV1,
};

use super::*;

/// Opaque workspace-only candidate for the specialized Runtime + World
/// Services transaction. It is not a durable contracts schema.
pub struct PreparedRuntimeWorldServicesTickV1 {
    runtime: PreparedRuntimeTick,
    routine: next_world::PreparedWorldRoutinePublicationV1,
    base_world_state_hash: next_contracts::ids::ContentHash,
    staged_world_snapshot: WorldStreamingSnapshotV1,
    streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
}

/// Opaque jointly validated generation retaining every base for final
/// preflight. Application evidence is an optional composition-root concern,
/// not part of the atomic publication precondition.
struct ValidatedWorldServicesGenerationV1 {
    runtime: ValidatedRuntimeTick,
    routine: next_world::ValidatedWorldRoutinePublicationV1,
    base_world_state_hash: next_contracts::ids::ContentHash,
    staged_world_snapshot: WorldStreamingSnapshotV1,
    streaming: Option<next_world::ValidatedWorldStreamingPublicationV1>,
}

/// Opaque jointly validated candidate carrying the full application closure.
pub struct ValidatedRuntimeWorldServicesTickV1 {
    generation: ValidatedWorldServicesGenerationV1,
    application_owner_segments: Vec<SaveSegmentDescriptor>,
    application_state_root: StateRoot,
}

/// Opaque jointly validated candidate for a live advance that does not need
/// to materialize the full application closure on every tick.
pub struct ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
    generation: ValidatedWorldServicesGenerationV1,
}

struct CommittedWorldServicesGenerationV1 {
    runtime_report: TickReport,
    world_streaming_snapshot: WorldStreamingSnapshotV1,
    routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    streaming_transition_or_none: Option<next_world::WorldTransitionCommitV1>,
}

/// Immutable result of one committed Runtime + World Services generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldServicesTickCommitV1 {
    pub runtime_report: TickReport,
    pub world_streaming_snapshot: WorldStreamingSnapshotV1,
    pub routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    pub streaming_transition_or_none: Option<next_world::WorldTransitionCommitV1>,
    pub application_owner_segments: Vec<SaveSegmentDescriptor>,
    pub application_state_root: StateRoot,
}

impl RuntimeTickPreparation<'_> {
    pub fn prepare_with_world_services(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        world: &next_world::WorldStreamerV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(commands, routine, world, None, None)
    }

    pub fn prepare_with_world_services_and_streaming(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        world: &next_world::WorldStreamerV1,
        publication: next_world::PreparedWorldStreamingPublicationV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(commands, routine, world, Some(publication), None)
    }

    fn prepare_world_services_internal(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        world: &next_world::WorldStreamerV1,
        streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
        replay_ingress: Option<next_contracts::input::ClosedIngressBatchV1>,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        let base_world_state_hash = world.snapshot().state_hash()?;
        let staged_world_snapshot = streaming.as_ref().map_or_else(
            || world.snapshot().clone(),
            |value| value.next_snapshot().clone(),
        );
        let mut routine_stage = WorldRoutineStageContextV1::capture(self.runtime, routine)?;
        let runtime = self.runtime.prepare_tick_internal(
            self.base_generation,
            self.ingress_checkpoint,
            commands,
            &mut NoOutcomes,
            replay_ingress,
            streaming
                .as_ref()
                .map(|publication| WorldStreamingStageContext { world, publication }),
            Some(&mut routine_stage),
        )?;
        let routine_snapshot_or_none = routine_stage.finish(runtime.next_tick())?;
        let routine = routine
            .prepare_publication(routine_snapshot_or_none, runtime.next_tick())
            .map_err(map_routine_owner_error)?;
        Ok(PreparedRuntimeWorldServicesTickV1 {
            runtime,
            routine,
            base_world_state_hash,
            staged_world_snapshot,
            streaming,
        })
    }
}

impl PreparedRuntimeWorldServicesTickV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.runtime.next_tick()
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        self.runtime.events()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.runtime.physics_snapshot()
    }

    #[must_use]
    pub fn rpg_snapshot(&self) -> next_contracts::rpg::RpgSnapshotV2 {
        self.runtime.rpg_snapshot()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.routine.snapshot_or_none()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.runtime.world_checkpoint_with_canonical_components()
    }
}

impl ValidatedRuntimeWorldServicesTickV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.generation.runtime.next_tick()
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        self.generation.runtime.events()
    }

    #[must_use]
    pub fn report(&self) -> &TickReport {
        self.generation.runtime.report()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.generation.runtime.physics_snapshot()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.generation.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.generation.routine.snapshot_or_none()
    }

    #[must_use]
    pub const fn application_state_root(&self) -> StateRoot {
        self.application_state_root
    }

    #[must_use]
    pub fn application_owner_segments(&self) -> &[SaveSegmentDescriptor] {
        &self.application_owner_segments
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.generation
            .runtime
            .world_checkpoint_with_canonical_components()
    }
}

impl ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.generation.runtime.next_tick()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.generation.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.generation.routine.snapshot_or_none()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.generation
            .runtime
            .world_checkpoint_with_canonical_components()
    }
}

impl RuntimeState {
    pub(in crate::engine) fn prepare_replay_world_services_tick(
        &self,
        closed_ingress_batch: next_contracts::input::ClosedIngressBatchV1,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        world: &next_world::WorldStreamerV1,
        streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.tick_preparation().prepare_world_services_internal(
            commands,
            routine,
            world,
            streaming,
            Some(closed_ingress_batch),
        )
    }

    /// Stages a controller configuration revision inside an already prepared
    /// Runtime + World Services generation without exposing its Runtime-only
    /// candidate to the composition root.
    pub fn stage_player_input_configuration_activation_world_services(
        &self,
        mut prepared: PreparedRuntimeWorldServicesTickV1,
        source_id: InputSourceId,
        action_map: ActionMapManifestV1,
        context_stack: InputContextStackV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        prepared.runtime = self.stage_player_input_configuration_activation(
            prepared.runtime,
            source_id,
            action_map,
            context_stack,
        )?;
        Ok(prepared)
    }

    pub fn validate_prepared_world_services_tick(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        let generation =
            self.validate_prepared_world_services_generation(routine, world, prepared)?;
        let (_, components) = generation
            .runtime
            .world_checkpoint_with_canonical_components()?;
        let routine_snapshot_or_none = generation.routine.snapshot_or_none().copied();
        let (application_owner_segments, application_state_root) = application_closure(
            &components,
            &generation.staged_world_snapshot,
            routine_snapshot_or_none.as_ref(),
        )?;
        Ok(ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        })
    }

    /// Validates the same joint Runtime + World Services generation used by
    /// the evidence-bearing path, but defers full application canonicalization
    /// until a checkpoint, replay proof, or save boundary actually requests it.
    pub fn validate_prepared_world_services_tick_without_application_evidence(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1, RuntimeFatalError>
    {
        Ok(
            ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
                generation: self
                    .validate_prepared_world_services_generation(routine, world, prepared)?,
            },
        )
    }

    fn validate_prepared_world_services_generation(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedWorldServicesGenerationV1, RuntimeFatalError> {
        if world.snapshot().state_hash()? != prepared.base_world_state_hash {
            return Err(RuntimeFatalError::PreparedWorldServicesGenerationStale);
        }
        let runtime = self
            .validate_prepared_tick(prepared.runtime)
            .map_err(|error| {
                if error == RuntimeFatalError::PreparedGenerationStale {
                    RuntimeFatalError::PreparedWorldServicesGenerationStale
                } else {
                    error
                }
            })?;
        let routine = routine
            .validate_prepared_publication(prepared.routine)
            .map_err(map_routine_owner_error)?;
        let streaming = prepared
            .streaming
            .map(|publication| {
                world.validate_prepared_publication(publication, runtime.report().tick)
            })
            .transpose()?;
        Ok(ValidatedWorldServicesGenerationV1 {
            runtime,
            routine,
            base_world_state_hash: prepared.base_world_state_hash,
            staged_world_snapshot: prepared.staged_world_snapshot,
            streaming,
        })
    }

    /// Performs one final read-only preflight, then publishes every live owner
    /// in fixed order with no fallible operation after the first write.
    pub fn commit_validated_world_services_tick(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickV1,
    ) -> Result<WorldServicesTickCommitV1, RuntimeFatalError> {
        let ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        } = validated;
        let committed =
            self.commit_validated_world_services_generation(routine, world, generation)?;
        Ok(WorldServicesTickCommitV1 {
            runtime_report: committed.runtime_report,
            world_streaming_snapshot: committed.world_streaming_snapshot,
            routine_snapshot_or_none: committed.routine_snapshot_or_none,
            streaming_transition_or_none: committed.streaming_transition_or_none,
            application_owner_segments,
            application_state_root,
        })
    }

    /// Commits a jointly validated live generation without constructing a
    /// full application root that the caller will discard.
    pub fn commit_validated_world_services_tick_without_application_evidence(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1,
    ) -> Result<TickReport, RuntimeFatalError> {
        Ok(self
            .commit_validated_world_services_generation(routine, world, validated.generation)?
            .runtime_report)
    }

    fn commit_validated_world_services_generation(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedWorldServicesGenerationV1,
    ) -> Result<CommittedWorldServicesGenerationV1, RuntimeFatalError> {
        if !validated.runtime.0.base_generation.matches(self)
            || world.snapshot().state_hash()? != validated.base_world_state_hash
            || routine
                .preflight_validated_publication(&validated.routine)
                .is_err()
            || validated.streaming.as_ref().is_some_and(|streaming| {
                world
                    .preflight_validated_publication(streaming, validated.runtime.report().tick)
                    .is_err()
            })
        {
            return Err(RuntimeFatalError::PreparedWorldServicesGenerationStale);
        }

        let ValidatedWorldServicesGenerationV1 {
            runtime: validated_runtime,
            routine: validated_routine,
            staged_world_snapshot,
            streaming,
            ..
        } = validated;
        let runtime_report = self.commit_validated_tick(validated_runtime);
        routine.commit_validated_publication(validated_routine);
        let streaming_transition_or_none =
            streaming.and_then(|streaming| world.commit_validated_publication(streaming));
        Ok(CommittedWorldServicesGenerationV1 {
            runtime_report,
            world_streaming_snapshot: staged_world_snapshot,
            routine_snapshot_or_none: routine.snapshot_or_none().copied(),
            streaming_transition_or_none,
        })
    }
}

fn application_closure(
    components: &WorldCheckpointCanonicalComponentsV1,
    streaming: &WorldStreamingSnapshotV1,
    routine_or_none: Option<&WorldRoutineSnapshotV1>,
) -> Result<(Vec<SaveSegmentDescriptor>, StateRoot), RuntimeFatalError> {
    let streaming_bytes = streaming.canonical_bytes()?;
    let mut descriptors = vec![
        segment_descriptor(
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            RUNTIME_SNAPSHOT_SCHEMA_VERSION,
            components.runtime_snapshot_bytes(),
        )?,
        segment_descriptor(
            RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
            components.rpg_snapshot_bytes(),
        )?,
        segment_descriptor(
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION),
            components.physics_checkpoint_bytes(),
        )?,
        segment_descriptor(
            WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION,
            &streaming_bytes,
        )?,
    ];
    let application_state_root = match routine_or_none {
        Some(routine) => {
            let routine_bytes = routine.canonical_bytes()?;
            descriptors.push(segment_descriptor(
                WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
                WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
                WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
                u32::from(WORLD_ROUTINE_SCHEMA_VERSION),
                &routine_bytes,
            )?);
            world_checkpoint_with_streaming_and_routine_v1_state_root_from_canonical_components(
                components, streaming, routine,
            )?
        }
        None => world_checkpoint_with_streaming_v1_state_root_from_canonical_components(
            components, streaming,
        )?,
    };
    descriptors.sort();
    if descriptors.windows(2).any(|pair| {
        (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
            == (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
    }) {
        return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
    }
    Ok((descriptors, application_state_root))
}

fn segment_descriptor(
    owner_id: &str,
    schema_id: &str,
    segment_id: &str,
    schema_version: u32,
    bytes: &[u8],
) -> Result<SaveSegmentDescriptor, RuntimeFatalError> {
    Ok(SaveSegmentDescriptor::for_bytes(
        SchemaId::new(owner_id).map_err(next_contracts::canonical::CanonicalError::from)?,
        SchemaId::new(schema_id).map_err(next_contracts::canonical::CanonicalError::from)?,
        SchemaId::new(segment_id).map_err(next_contracts::canonical::CanonicalError::from)?,
        schema_version,
        bytes,
    )?)
}

fn map_routine_owner_error(error: next_world::WorldRoutineOwnerError) -> RuntimeFatalError {
    if matches!(error, next_world::WorldRoutineOwnerError::PublicationStale) {
        RuntimeFatalError::PreparedWorldServicesGenerationStale
    } else {
        RuntimeFatalError::WorldRoutineInternalInvariant
    }
}
