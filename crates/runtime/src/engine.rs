use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    AcceptedLocomotionIntentV2, AuthoritativeNumericProfileV1,
    CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION, CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
    COMMAND_ENVELOPE_SCHEMA_VERSION, COMMAND_RECEIPT_SCHEMA_VERSION, CORE_INTERACT_ACTION_ID,
    CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID, CORE_INTERACTIVE_OBJECT_ARCHETYPE_ID,
    CORE_INTERACTIVE_OBJECT_READY_STATE_ID, CORE_MOVE_ACTION_ID, CanonicalDecodeLimits,
    CanonicalError, CapabilityId, CausalIdentityKey, CausalIdentityKind,
    ClosedCommandAdmissionBatchBodyV2, ClosedCommandAdmissionBatchV2, ClosedIngressBatchBodyV1,
    ClosedIngressBatchV1, ClosedPhysicsContactBatchV1, CommandBodyArchiveV1,
    CommandCollisionCandidateV1, CommandCollisionIncidentV1, CommandFinalResultV1, CommandId,
    CommandIdentityOccurrenceV1, CommandLedgerError, CommandLedgerV2, CommandPayload, CommandPhase,
    CommandReceiptSubjectV1, CommandReceiptV1, CommandReservationV1, CommandStreamId,
    CommandStreamLedgerV2, CommandStreamRegistryV1, CommandStreamStateV1, ContentHash, DomainEvent,
    IdentityContractError, IdentityInsertResult, IngressAssignmentProfileV1, IngressAssignmentV1,
    IngressCheckpointV1, IngressEquivalenceReceiptV1, InputContractError, InputMappingCodeV1,
    InputMappingReceiptV1, InputSampleV1, IssuerPrincipal, PHYSICS_STEP_INPUT_SCHEMA_VERSION,
    PLAYER_ACTION_FRAME_SCHEMA_ID, PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS,
    PLAYER_INTERACTION_SYSTEM_ID, PersistentId, PhysicalCommandV1, PhysicalEventV1,
    PhysicsCanonicalSnapshotV2, PhysicsContractError, PhysicsCoordinateProfileV1,
    PhysicsLimitsProfileV1, PhysicsQuantizationProfileV1, PhysicsSolverSemanticsProfileV1,
    PhysicsStepInputV2, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1, PhysicsWorldId, PlayerActionFrameV1, PlayerActionPhaseV1,
    PlayerActionValueV1, PlayerControllerBindingV1, PlayerControllerRegistryV1,
    PrincipalRegistryV1, ProjectId, RPG_COMMAND_CAPABILITY_ID, RpgCommand, RpgDecodeError,
    RpgSnapshot, RuntimeAdmissionLimitsV1, RuntimeDeterminismProfileV1, RuntimeSnapshot, SchemaId,
    SnapshotDecodeError, SystemId, TickRateProfileV1, WorldCheckpointError, WorldCheckpointV3,
    WorldCommand, WorldIdentityManifestV1, content_hash_from_bytes, sha256,
};
use next_physics_api::{ReferencePhysicsError, ReferencePhysicsWorld};
use next_rpg::{RpgApplyError, RpgState, RpgStateError};

use crate::authority::AuthorityRegistry;
use crate::outcome::{
    NoOutcomes, OutcomeCollectionError, OutcomeContext, OutcomeProvider, OutcomeSink,
};
use crate::registry::CommandKindRegistry;

type CollisionKey = (CommandStreamId, IssuerPrincipal, u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeBootstrapV3 {
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
    pub physics_checkpoint: PhysicsWorldCheckpointV1,
}

impl RuntimeBootstrapV3 {
    pub fn new(
        world_identity: WorldIdentityManifestV1,
        principal_registry: PrincipalRegistryV1,
        stream_registry: CommandStreamRegistryV1,
        runtime_profile: RuntimeDeterminismProfileV1,
    ) -> Self {
        let admission_limits = RuntimeAdmissionLimitsV1::default();
        let tick_rate_profile = TickRateProfileV1::at_30_hz();
        let ingress_assignment_profile = IngressAssignmentProfileV1::core_v1(&admission_limits)
            .expect("built-in ingress profile is canonical");
        let physics_quantization_profile = PhysicsQuantizationProfileV1::capsule_reference_v1()
            .expect("built-in quantization profile identifiers are valid");
        let authoritative_numeric_profile =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&physics_quantization_profile)
                .expect("built-in numeric profile is canonical");
        let player_controller_registry = PlayerControllerRegistryV1 {
            schema_version: 1,
            world_namespace: world_identity.world_namespace,
            bindings: BTreeMap::new(),
        };
        let physics_checkpoint = empty_physics_checkpoint(
            PhysicsWorldId::from_bytes(*world_identity.world_namespace.as_bytes()),
            &tick_rate_profile,
            &authoritative_numeric_profile,
            &physics_quantization_profile,
        )
        .expect("built-in empty physical snapshot is canonical");
        Self {
            world_identity,
            principal_registry,
            stream_registry,
            runtime_profile,
            admission_limits,
            tick_rate_profile,
            ingress_assignment_profile,
            authoritative_numeric_profile,
            physics_quantization_profile,
            player_controller_registry,
            physics_checkpoint,
        }
    }

    #[must_use]
    pub fn with_player_input_and_physics(
        mut self,
        player_controller_registry: PlayerControllerRegistryV1,
        physics_checkpoint: PhysicsWorldCheckpointV1,
    ) -> Self {
        self.player_controller_registry = player_controller_registry;
        self.physics_checkpoint = physics_checkpoint;
        self
    }

    pub fn neutral_empty() -> Result<Self, SnapshotRestoreError> {
        let registry_hash = CommandKindRegistry::core_v1().canonical_hash();
        let profile = RuntimeDeterminismProfileV1::bootstrap_default(registry_hash);
        let world_identity = WorldIdentityManifestV1::new(
            ProjectId::new("nextengine.runtime-empty")
                .expect("built-in neutral project identifier is valid"),
            [0; 32],
            [0; 32],
            profile.profile_hash()?,
        )?;
        Ok(Self::new(
            world_identity.clone(),
            PrincipalRegistryV1::empty(world_identity.world_namespace),
            CommandStreamRegistryV1::empty(world_identity.world_namespace),
            profile,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeState {
    registry: CommandKindRegistry,
    authority: AuthorityRegistry,
    next_tick: u64,
    committed_event_count: u64,
    authoritative_revision: u64,
    world_identity: WorldIdentityManifestV1,
    principal_registry: PrincipalRegistryV1,
    stream_registry: CommandStreamRegistryV1,
    runtime_profile: RuntimeDeterminismProfileV1,
    admission_limits: RuntimeAdmissionLimitsV1,
    tick_rate_profile: TickRateProfileV1,
    ingress_assignment_profile: IngressAssignmentProfileV1,
    authoritative_numeric_profile: AuthoritativeNumericProfileV1,
    physics_quantization_profile: PhysicsQuantizationProfileV1,
    player_controller_registry: PlayerControllerRegistryV1,
    ingress_checkpoint: IngressCheckpointV1,
    physics: ReferencePhysicsWorld,
    last_closed_ingress_batch: Option<ClosedIngressBatchV1>,
    last_mapping_receipts: Vec<InputMappingReceiptV1>,
    last_command_batches: Vec<ClosedCommandAdmissionBatchV2>,
    command_ledger: CommandLedgerV2,
    body_archive: CommandBodyArchiveV1,
    rpg: RpgState,
}

impl RuntimeState {
    pub fn new(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::from_bootstrap(bootstrap, authority, RpgState::default())
    }

    pub fn with_rpg_snapshot(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
        snapshot: RpgSnapshot,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::from_bootstrap(bootstrap, authority, validate_rpg_snapshot(snapshot)?)
    }

    fn from_bootstrap(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
        rpg: RpgState,
    ) -> Result<Self, SnapshotRestoreError> {
        let registry = CommandKindRegistry::core_v1();
        validate_bootstrap(&bootstrap, &authority, &registry)?;
        let physics = ReferencePhysicsWorld::new(
            bootstrap.physics_checkpoint,
            bootstrap.tick_rate_profile,
            bootstrap.authoritative_numeric_profile.clone(),
            bootstrap.physics_quantization_profile.clone(),
        )?;
        let profile_hash = bootstrap.runtime_profile.profile_hash()?;
        let (mut command_ledger, body_archive) = CommandLedgerV2::empty(
            bootstrap.world_identity.world_namespace,
            registry.canonical_hash(),
            profile_hash,
        )?;
        register_bootstrap_identities(
            &mut command_ledger,
            &bootstrap.principal_registry,
            &bootstrap.stream_registry,
        )?;
        for (key, stream_id) in &bootstrap.stream_registry.entries {
            command_ledger.streams.insert(
                *stream_id,
                CommandStreamLedgerV2::genesis(
                    *stream_id,
                    key.principal.clone(),
                    key.stream_slot,
                    key.stream_epoch,
                ),
            );
        }
        command_ledger.validate(&body_archive)?;
        Ok(Self {
            registry,
            authority,
            next_tick: 0,
            committed_event_count: 0,
            authoritative_revision: 0,
            world_identity: bootstrap.world_identity,
            principal_registry: bootstrap.principal_registry,
            stream_registry: bootstrap.stream_registry,
            runtime_profile: bootstrap.runtime_profile,
            admission_limits: bootstrap.admission_limits,
            tick_rate_profile: bootstrap.tick_rate_profile,
            ingress_assignment_profile: bootstrap.ingress_assignment_profile,
            authoritative_numeric_profile: bootstrap.authoritative_numeric_profile,
            physics_quantization_profile: bootstrap.physics_quantization_profile,
            player_controller_registry: bootstrap.player_controller_registry,
            ingress_checkpoint: IngressCheckpointV1 {
                schema_version: 1,
                current_tick: 0,
                current_generation: 0,
                current_samples: Vec::new(),
                next_samples: Vec::new(),
                last_closed_batch_hash: None,
            },
            physics,
            last_closed_ingress_batch: None,
            last_mapping_receipts: Vec::new(),
            last_command_batches: Vec::new(),
            command_ledger,
            body_archive,
            rpg,
        })
    }

    pub fn restore_world_checkpoint(
        checkpoint: WorldCheckpointV3,
        authority: AuthorityRegistry,
    ) -> Result<Self, SnapshotRestoreError> {
        checkpoint.validate()?;
        let runtime_bytes = checkpoint.runtime_snapshot.canonical_bytes()?;
        let snapshot = RuntimeSnapshot::from_canonical_bytes(
            &runtime_bytes,
            CanonicalDecodeLimits::default(),
        )?;
        let rpg = validate_rpg_snapshot(checkpoint.rpg_snapshot)?;
        let physics_bytes = checkpoint.physics_checkpoint.canonical_bytes()?;
        let physics_checkpoint = PhysicsWorldCheckpointV1::from_canonical_bytes(
            &physics_bytes,
            CanonicalDecodeLimits::default(),
        )?;
        Self::restore_from_parts(snapshot, rpg, physics_checkpoint, authority)
    }

    fn restore_from_parts(
        snapshot: RuntimeSnapshot,
        rpg: RpgState,
        physical: PhysicsWorldCheckpointV1,
        authority: AuthorityRegistry,
    ) -> Result<Self, SnapshotRestoreError> {
        let registry = CommandKindRegistry::core_v1();
        validate_bootstrap(
            &RuntimeBootstrapV3 {
                world_identity: snapshot.world_identity.clone(),
                principal_registry: snapshot.principal_registry.clone(),
                stream_registry: snapshot.stream_registry.clone(),
                runtime_profile: snapshot.runtime_profile,
                admission_limits: snapshot.admission_limits,
                tick_rate_profile: snapshot.tick_rate_profile,
                ingress_assignment_profile: snapshot.ingress_assignment_profile,
                authoritative_numeric_profile: snapshot.authoritative_numeric_profile.clone(),
                physics_quantization_profile: snapshot.physics_quantization_profile.clone(),
                player_controller_registry: snapshot.player_controller_registry.clone(),
                physics_checkpoint: physical.clone(),
            },
            &authority,
            &registry,
        )?;
        if snapshot.command_ledger.command_kind_registry_hash != registry.canonical_hash() {
            return Err(SnapshotRestoreError::CommandRegistryMismatch);
        }
        let physics = ReferencePhysicsWorld::new(
            physical,
            snapshot.tick_rate_profile,
            snapshot.authoritative_numeric_profile.clone(),
            snapshot.physics_quantization_profile.clone(),
        )?;
        Ok(Self {
            registry,
            authority,
            next_tick: snapshot.next_tick,
            committed_event_count: snapshot.committed_event_count,
            authoritative_revision: snapshot.authoritative_revision,
            world_identity: snapshot.world_identity,
            principal_registry: snapshot.principal_registry,
            stream_registry: snapshot.stream_registry,
            runtime_profile: snapshot.runtime_profile,
            admission_limits: snapshot.admission_limits,
            tick_rate_profile: snapshot.tick_rate_profile,
            ingress_assignment_profile: snapshot.ingress_assignment_profile,
            authoritative_numeric_profile: snapshot.authoritative_numeric_profile,
            physics_quantization_profile: snapshot.physics_quantization_profile,
            player_controller_registry: snapshot.player_controller_registry,
            ingress_checkpoint: snapshot.ingress_checkpoint,
            physics,
            last_closed_ingress_batch: None,
            last_mapping_receipts: Vec::new(),
            last_command_batches: Vec::new(),
            command_ledger: snapshot.command_ledger,
            body_archive: snapshot.body_archive,
            rpg,
        })
    }

    pub fn world_checkpoint(&self) -> Result<WorldCheckpointV3, WorldCheckpointError> {
        WorldCheckpointV3::new(
            self.snapshot(),
            self.rpg_snapshot(),
            self.physics.checkpoint().clone(),
        )
    }

    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.next_tick
    }

    #[must_use]
    pub const fn authoritative_revision(&self) -> u64 {
        self.authoritative_revision
    }

    #[must_use]
    pub fn command_kind_registry(&self) -> &CommandKindRegistry {
        &self.registry
    }

    #[must_use]
    pub fn command_ledger(&self) -> &CommandLedgerV2 {
        &self.command_ledger
    }

    #[must_use]
    pub fn body_archive(&self) -> &CommandBodyArchiveV1 {
        &self.body_archive
    }

    #[must_use]
    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            next_tick: self.next_tick,
            committed_event_count: self.committed_event_count,
            authoritative_revision: self.authoritative_revision,
            world_identity: self.world_identity.clone(),
            principal_registry: self.principal_registry.clone(),
            stream_registry: self.stream_registry.clone(),
            runtime_profile: self.runtime_profile,
            admission_limits: self.admission_limits,
            tick_rate_profile: self.tick_rate_profile,
            ingress_assignment_profile: self.ingress_assignment_profile,
            authoritative_numeric_profile: self.authoritative_numeric_profile.clone(),
            physics_quantization_profile: self.physics_quantization_profile.clone(),
            player_controller_registry: self.player_controller_registry.clone(),
            ingress_checkpoint: self.ingress_checkpoint.clone(),
            command_ledger: self.command_ledger.clone(),
            body_archive: self.body_archive.clone(),
        }
    }

    #[must_use]
    pub fn rpg_snapshot(&self) -> RpgSnapshot {
        self.rpg.snapshot()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &PhysicsCanonicalSnapshotV2 {
        self.physics.snapshot()
    }

    #[must_use]
    pub fn physics_checkpoint(&self) -> &PhysicsWorldCheckpointV1 {
        self.physics.checkpoint()
    }

    #[must_use]
    pub const fn last_closed_ingress_batch(&self) -> Option<&ClosedIngressBatchV1> {
        self.last_closed_ingress_batch.as_ref()
    }

    #[must_use]
    pub fn last_mapping_receipts(&self) -> &[InputMappingReceiptV1] {
        &self.last_mapping_receipts
    }

    #[must_use]
    pub fn last_command_batches(&self) -> &[ClosedCommandAdmissionBatchV2] {
        &self.last_command_batches
    }

    pub fn enqueue_input_sample(
        &mut self,
        principal: &IssuerPrincipal,
        sample: InputSampleV1,
    ) -> Result<(), InputAdmissionError> {
        sample
            .validate(&self.admission_limits)
            .map_err(InputAdmissionError::Contract)?;
        if !self.principal_registry.is_active(principal)
            || !self.authority.is_authenticated(principal)
        {
            return Err(InputAdmissionError::PrincipalUnauthenticated);
        }
        let Some(binding) = self
            .player_controller_registry
            .bindings
            .get(&sample.source_id)
        else {
            return Err(InputAdmissionError::SourceUnbound);
        };
        if &binding.principal != principal {
            return Err(InputAdmissionError::SourceUnbound);
        }
        let maximum = usize::try_from(self.admission_limits.max_commands_per_closed_batch)
            .map_err(|_| InputAdmissionError::ResourceLimit)?;
        if self.ingress_checkpoint.current_samples.len() >= maximum {
            return Err(InputAdmissionError::ResourceLimit);
        }
        let canonical = sample
            .canonical_bytes()
            .map_err(InputAdmissionError::Canonical)?;
        let sample = InputSampleV1::from_canonical_bytes(
            &canonical,
            CanonicalDecodeLimits::default(),
            &self.admission_limits,
        )
        .map_err(InputAdmissionError::Contract)?;
        self.ingress_checkpoint.current_samples.push(sample);
        self.ingress_checkpoint
            .current_samples
            .sort_by(|left, right| {
                left.sort_key()
                    .expect("validated input sample has a canonical sort key")
                    .cmp(
                        &right
                            .sort_key()
                            .expect("validated input sample has a canonical sort key"),
                    )
            });
        Ok(())
    }

    pub fn run_tick(
        &mut self,
        commands: impl IntoIterator<Item = WorldCommand>,
    ) -> Result<TickReport, RuntimeFatalError> {
        self.run_tick_with_outcomes(commands, &mut NoOutcomes)
    }

    pub fn run_tick_with_outcomes(
        &mut self,
        commands: impl IntoIterator<Item = WorldCommand>,
        outcome_provider: &mut impl OutcomeProvider,
    ) -> Result<TickReport, RuntimeFatalError> {
        self.run_tick_internal(commands, outcome_provider, None)
    }

    fn replay_closed_ingress_tick(
        &mut self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: impl IntoIterator<Item = WorldCommand>,
    ) -> Result<TickReport, RuntimeFatalError> {
        self.run_tick_internal(direct_commands, &mut NoOutcomes, Some(closed_ingress_batch))
    }

    fn preview_replay_ingress_batch(
        &self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: &[WorldCommand],
    ) -> Result<ClosedCommandAdmissionBatchV2, RuntimeFatalError> {
        let following_tick = self
            .next_tick
            .checked_add(1)
            .ok_or(RuntimeFatalError::TickExhausted)?;
        let mut staged_ingress = self.ingress_checkpoint.clone();
        let interaction_enabled = resolve_interaction_outcome_route(
            &self.principal_registry,
            &self.stream_registry,
            &self.authority,
        )
        .is_some();
        let closed_ingress = accept_closed_ingress(
            self.next_tick,
            following_tick,
            &self.admission_limits,
            &self.player_controller_registry,
            interaction_enabled,
            &mut staged_ingress,
            closed_ingress_batch,
        )?;
        let mut ingress_commands = closed_ingress.derived_commands;
        ingress_commands.extend_from_slice(direct_commands);
        sort_command_batch(&mut ingress_commands)?;
        let ingress_batch =
            ClosedCommandAdmissionBatchV2::from_body(ClosedCommandAdmissionBatchBodyV2 {
                schema_version: CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION,
                simulation_tick: self.next_tick,
                phase: CommandPhase::Ingress,
                batch_ordinal: 0,
                envelopes: ingress_commands,
            })?;
        ingress_batch.validate(&self.admission_limits)?;
        Ok(ingress_batch)
    }

    fn run_tick_internal(
        &mut self,
        commands: impl IntoIterator<Item = WorldCommand>,
        outcome_provider: &mut impl OutcomeProvider,
        replay_ingress: Option<ClosedIngressBatchV1>,
    ) -> Result<TickReport, RuntimeFatalError> {
        let following_tick = self
            .next_tick
            .checked_add(1)
            .ok_or(RuntimeFatalError::TickExhausted)?;
        let tick = self.next_tick;
        let interaction_route = resolve_interaction_outcome_route(
            &self.principal_registry,
            &self.stream_registry,
            &self.authority,
        );
        let mut staged = StagedAuthoritativeState {
            ledger: self.command_ledger.clone(),
            archive: self.body_archive.clone(),
            event_count: self.committed_event_count,
            revision: self.authoritative_revision,
            rpg: self.rpg.clone(),
            physics: self.physics.clone(),
            ingress: self.ingress_checkpoint.clone(),
        };

        let mut closed_ingress = match replay_ingress {
            Some(batch) => accept_closed_ingress(
                tick,
                following_tick,
                &self.admission_limits,
                &self.player_controller_registry,
                interaction_route.is_some(),
                &mut staged.ingress,
                batch,
            )?,
            None => close_ingress(
                tick,
                following_tick,
                &self.admission_limits,
                &self.player_controller_registry,
                interaction_route.is_some(),
                &mut staged.ingress,
            )?,
        };
        let mut ingress_commands = closed_ingress.derived_commands.clone();
        ingress_commands.extend(commands);
        sort_command_batch(&mut ingress_commands)?;
        let ingress_batch =
            ClosedCommandAdmissionBatchV2::from_body(ClosedCommandAdmissionBatchBodyV2 {
                schema_version: CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION,
                simulation_tick: tick,
                phase: CommandPhase::Ingress,
                batch_ordinal: 0,
                envelopes: ingress_commands.clone(),
            })?;
        ingress_batch.validate(&self.admission_limits)?;

        let ingress_revision = staged.revision;
        let ingress = process_phase(
            PhaseContext {
                registry: &self.registry,
                authority: &self.authority,
                principals: &self.principal_registry,
                streams: &self.stream_registry,
                profile: &self.runtime_profile,
                controllers: &self.player_controller_registry,
                source: ValidationSource::ExternalIngress,
                tick,
                phase: CommandPhase::Ingress,
                phase_revision: ingress_revision,
            },
            ingress_commands,
            &mut staged,
        )?;
        let physics_step_input = ingress
            .physics_step_input
            .clone()
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
        let contact_batch = ingress
            .contact_batch
            .clone()
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;

        let built_in_outcomes = build_interaction_outcomes(
            &closed_ingress.pending_interactions,
            interaction_route.as_ref(),
            &staged.physics,
            &staged.rpg,
            staged.revision,
        )?;
        let mut outcome_sink = OutcomeSink::new();
        outcome_provider
            .collect(
                OutcomeContext {
                    tick,
                    authoritative_revision: staged.revision,
                    ingress_events: &ingress.events,
                },
                &mut outcome_sink,
            )
            .map_err(RuntimeFatalError::OutcomeCollection)?;
        let external_proposals = outcome_sink.into_proposals();
        let proposal_count = count(
            built_in_outcomes
                .len()
                .checked_add(external_proposals.len())
                .ok_or(RuntimeFatalError::TraceCountExhausted)?,
        )?;
        let mut outcome_commands = Vec::with_capacity(
            built_in_outcomes
                .len()
                .checked_add(external_proposals.len())
                .ok_or(RuntimeFatalError::TraceCountExhausted)?,
        );
        for built_in in built_in_outcomes {
            let command = built_in
                .proposal
                .into_command(tick)
                .map_err(RuntimeFatalError::InternalCanonicalization)?;
            let command_id = command.compute_command_id()?;
            let receipt = closed_ingress
                .mapping_receipts
                .iter_mut()
                .find(|receipt| {
                    receipt.source_id == built_in.source_id
                        && receipt.source_sequence == built_in.source_sequence
                        && receipt.payload_hash == built_in.payload_hash
                })
                .ok_or(RuntimeFatalError::IngressCheckpointCorrupt)?;
            receipt.derived_command_id = Some(command_id);
            outcome_commands.push(command);
        }
        outcome_commands.extend(
            external_proposals
                .into_iter()
                .map(|proposal| {
                    proposal
                        .into_command(tick)
                        .map_err(RuntimeFatalError::InternalCanonicalization)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        sort_command_batch(&mut outcome_commands)?;
        let outcome_batch =
            ClosedCommandAdmissionBatchV2::from_body(ClosedCommandAdmissionBatchBodyV2 {
                schema_version: CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION,
                simulation_tick: tick,
                phase: CommandPhase::Outcome,
                batch_ordinal: 1,
                envelopes: outcome_commands.clone(),
            })?;
        outcome_batch.validate(&self.admission_limits)?;

        let outcome_revision = staged.revision;
        let outcome = process_phase(
            PhaseContext {
                registry: &self.registry,
                authority: &self.authority,
                principals: &self.principal_registry,
                streams: &self.stream_registry,
                profile: &self.runtime_profile,
                controllers: &self.player_controller_registry,
                source: ValidationSource::InternalOutcome,
                tick,
                phase: CommandPhase::Outcome,
                phase_revision: outcome_revision,
            },
            outcome_commands,
            &mut staged,
        )?;

        staged.ledger.synchronize_archive(&staged.archive)?;
        staged.ledger.validate(&staged.archive)?;
        staged.physics.set_checkpoint_revision(staged.revision);
        let mut ordered_results = ingress.results;
        ordered_results.extend(outcome.results);
        ordered_results.sort();
        let results = ordered_results
            .into_iter()
            .map(|result| result.result)
            .collect();
        let mut events = ingress.events;
        events.extend(outcome.events);

        let snapshot = RuntimeSnapshot {
            next_tick: following_tick,
            committed_event_count: staged.event_count,
            authoritative_revision: staged.revision,
            world_identity: self.world_identity.clone(),
            principal_registry: self.principal_registry.clone(),
            stream_registry: self.stream_registry.clone(),
            runtime_profile: self.runtime_profile,
            admission_limits: self.admission_limits,
            tick_rate_profile: self.tick_rate_profile,
            ingress_assignment_profile: self.ingress_assignment_profile,
            authoritative_numeric_profile: self.authoritative_numeric_profile.clone(),
            physics_quantization_profile: self.physics_quantization_profile.clone(),
            player_controller_registry: self.player_controller_registry.clone(),
            ingress_checkpoint: staged.ingress.clone(),
            command_ledger: staged.ledger.clone(),
            body_archive: staged.archive.clone(),
        };
        snapshot.validate().map_err(RuntimeFatalError::Snapshot)?;

        let mut stage_trace = vec![StageTraceEntry {
            stage: TransactionStage::IngressClose,
            received: count(closed_ingress.batch.body.input_samples.len())?,
            accepted: count(closed_ingress.batch.body.input_assignments.len())?,
            rejected: count(closed_ingress.batch.body.equivalence_receipts.len())?,
            committed: 1,
            deduplicated: closed_ingress.deduplicated,
        }];
        stage_trace.extend(ingress.stage_trace);
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::PhysicalStep,
            received: count(
                ingress_batch
                    .body
                    .envelopes
                    .iter()
                    .filter(|command| matches!(&command.payload, CommandPayload::Physical(_)))
                    .count(),
            )?,
            accepted: count(
                ingress_batch
                    .body
                    .envelopes
                    .iter()
                    .filter(|command| matches!(&command.payload, CommandPayload::Physical(_)))
                    .count(),
            )?,
            rejected: 0,
            committed: 1,
            deduplicated: 0,
        });
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::PhysicsContactPublication,
            received: count(contact_batch.events.len())?,
            accepted: count(contact_batch.events.len())?,
            rejected: 0,
            committed: 1,
            deduplicated: 0,
        });
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::OutcomeCollection,
            received: proposal_count,
            accepted: proposal_count,
            rejected: 0,
            committed: 0,
            deduplicated: 0,
        });
        stage_trace.extend(outcome.stage_trace);
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::SnapshotPublication,
            received: 1,
            accepted: 1,
            rejected: 0,
            committed: 1,
            deduplicated: 0,
        });

        self.next_tick = following_tick;
        self.committed_event_count = staged.event_count;
        self.authoritative_revision = staged.revision;
        self.command_ledger = staged.ledger;
        self.body_archive = staged.archive;
        self.rpg = staged.rpg;
        self.physics = staged.physics;
        self.ingress_checkpoint = staged.ingress;
        self.last_closed_ingress_batch = Some(closed_ingress.batch);
        self.last_mapping_receipts = closed_ingress.mapping_receipts;
        self.last_command_batches = vec![ingress_batch, outcome_batch];

        Ok(TickReport {
            tick,
            results,
            events,
            stage_trace,
            snapshot,
            rpg_snapshot: self.rpg.snapshot(),
            physics_snapshot: self.physics.snapshot().clone(),
            physics_step_input,
            contact_batch,
            physics_checkpoint_hash: self.physics.checkpoint_hash()?,
            closed_ingress_batch: self
                .last_closed_ingress_batch
                .clone()
                .expect("successful tick publishes its closed ingress batch"),
            mapping_receipts: self.last_mapping_receipts.clone(),
            command_batches: self.last_command_batches.clone(),
        })
    }
}

#[derive(Debug)]
pub struct RuntimeReplayDriver {
    runtime: RuntimeState,
}

impl RuntimeReplayDriver {
    pub fn new(
        checkpoint: WorldCheckpointV3,
        authority: AuthorityRegistry,
    ) -> Result<Self, SnapshotRestoreError> {
        Ok(Self {
            runtime: RuntimeState::restore_world_checkpoint(checkpoint, authority)?,
        })
    }

    pub fn replay_tick(
        &mut self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: Vec<WorldCommand>,
        expected_ingress_batch: &ClosedCommandAdmissionBatchV2,
        expected_physics_step_input: &PhysicsStepInputV2,
        expected_contact_batch: &ClosedPhysicsContactBatchV1,
        expected_outcome_batch: &ClosedCommandAdmissionBatchV2,
    ) -> Result<TickReport, RuntimeReplayError> {
        let ingress_batch = self
            .runtime
            .preview_replay_ingress_batch(closed_ingress_batch.clone(), &direct_commands)?;
        if &ingress_batch != expected_ingress_batch {
            return Err(RuntimeReplayError::CommandBatchMismatch {
                tick: self.runtime.next_tick,
                phase: CommandPhase::Ingress,
            });
        }
        expected_contact_batch
            .validate_against_catalog(&self.runtime.physics.checkpoint().catalog)
            .map_err(ReferencePhysicsError::from)
            .map_err(RuntimeFatalError::from)?;
        let mut staged_runtime = self.runtime.clone();
        let report =
            staged_runtime.replay_closed_ingress_tick(closed_ingress_batch, direct_commands)?;
        if &report.physics_step_input != expected_physics_step_input {
            return Err(RuntimeReplayError::PhysicsStepInputMismatch { tick: report.tick });
        }
        if &report.contact_batch != expected_contact_batch {
            return Err(RuntimeReplayError::ContactBatchMismatch { tick: report.tick });
        }
        if report.command_batches.get(1) != Some(expected_outcome_batch) {
            return Err(RuntimeReplayError::CommandBatchMismatch {
                tick: report.tick,
                phase: CommandPhase::Outcome,
            });
        }
        self.runtime = staged_runtime;
        Ok(report)
    }

    pub fn world_checkpoint(&self) -> Result<WorldCheckpointV3, WorldCheckpointError> {
        self.runtime.world_checkpoint()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RuntimeReplayError {
    Runtime(RuntimeFatalError),
    CommandBatchMismatch { tick: u64, phase: CommandPhase },
    PhysicsStepInputMismatch { tick: u64 },
    ContactBatchMismatch { tick: u64 },
}

impl RuntimeReplayError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Runtime(error) => error.stable_code(),
            Self::CommandBatchMismatch { .. } => "REPLAY_COMMAND_BATCH_MISMATCH",
            Self::PhysicsStepInputMismatch { .. } => "REPLAY_PHYSICS_STEP_INPUT_MISMATCH",
            Self::ContactBatchMismatch { .. } => "REPLAY_CONTACT_BATCH_MISMATCH",
        }
    }
}

impl Display for RuntimeReplayError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "replay runtime failed: {error}"),
            Self::CommandBatchMismatch { tick, phase } => {
                write!(
                    formatter,
                    "recorded {phase:?} command batch diverged at tick {tick}"
                )
            }
            Self::PhysicsStepInputMismatch { tick } => {
                write!(
                    formatter,
                    "recorded physics step input diverged at tick {tick}"
                )
            }
            Self::ContactBatchMismatch { tick } => {
                write!(formatter, "recorded contact batch diverged at tick {tick}")
            }
        }
    }
}

impl Error for RuntimeReplayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::CommandBatchMismatch { .. }
            | Self::PhysicsStepInputMismatch { .. }
            | Self::ContactBatchMismatch { .. } => None,
        }
    }
}

impl From<RuntimeFatalError> for RuntimeReplayError {
    fn from(error: RuntimeFatalError) -> Self {
        Self::Runtime(error)
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new(
            RuntimeBootstrapV3::neutral_empty().expect("neutral empty bootstrap is valid"),
            AuthorityRegistry::new(),
        )
        .expect("neutral empty runtime is valid")
    }
}

struct ClosedIngressExecution {
    batch: ClosedIngressBatchV1,
    mapping_receipts: Vec<InputMappingReceiptV1>,
    derived_commands: Vec<WorldCommand>,
    pending_interactions: Vec<PendingInteractionIntent>,
    deduplicated: u64,
}

struct PlayerActionMapping {
    receipts: Vec<InputMappingReceiptV1>,
    commands: Vec<WorldCommand>,
    pending_interactions: Vec<PendingInteractionIntent>,
}

#[derive(Clone, Debug)]
struct InteractionOutcomeRoute {
    system_id: SystemId,
    stream_id: CommandStreamId,
}

#[derive(Clone, Debug)]
struct PendingInteractionIntent {
    controlled_body_id: PersistentId,
    source_id: next_contracts::InputSourceId,
    source_sequence: u64,
    payload_hash: ContentHash,
}

#[derive(Clone, Debug)]
struct BuiltInInteractionOutcome {
    proposal: crate::outcome::OutcomeProposal,
    source_id: next_contracts::InputSourceId,
    source_sequence: u64,
    payload_hash: ContentHash,
}

fn close_ingress(
    tick: u64,
    following_tick: u64,
    admission: &RuntimeAdmissionLimitsV1,
    controllers: &PlayerControllerRegistryV1,
    interaction_enabled: bool,
    checkpoint: &mut IngressCheckpointV1,
) -> Result<ClosedIngressExecution, RuntimeFatalError> {
    if checkpoint.current_tick != tick {
        return Err(RuntimeFatalError::IngressCheckpointCorrupt);
    }
    let next_generation = checkpoint
        .current_generation
        .checked_add(1)
        .ok_or(RuntimeFatalError::IngressGenerationExhausted)?;

    let mut samples = checkpoint.current_samples.clone();
    for sample in &mut samples {
        let bytes = sample.canonical_bytes()?;
        *sample = InputSampleV1::from_canonical_bytes(
            &bytes,
            CanonicalDecodeLimits::default(),
            admission,
        )?;
    }
    samples.sort_by(|left, right| {
        left.sort_key()
            .expect("validated input sample has a canonical sort key")
            .cmp(
                &right
                    .sort_key()
                    .expect("validated input sample has a canonical sort key"),
            )
    });
    let before_dedup = samples.len();
    samples.dedup_by(|left, right| {
        left.canonical_bytes()
            .expect("validated sample is canonical")
            == right
                .canonical_bytes()
                .expect("validated sample is canonical")
    });
    let deduplicated = count(
        before_dedup
            .checked_sub(samples.len())
            .ok_or(RuntimeFatalError::TraceCountExhausted)?,
    )?;

    let mut collisions: BTreeMap<
        (SchemaId, next_contracts::InputSourceId, u64),
        BTreeSet<ContentHash>,
    > = BTreeMap::new();
    for sample in &samples {
        collisions
            .entry((
                sample.source_class.clone(),
                sample.source_id,
                sample.source_sequence,
            ))
            .or_default()
            .insert(sample.payload_hash()?);
    }
    let collision_keys = collisions
        .iter()
        .filter_map(|(key, hashes)| {
            (hashes.len() > 1).then_some((key.0.as_str().to_owned(), key.1, key.2))
        })
        .collect::<BTreeSet<_>>();

    let mut assignments = Vec::new();
    let mut equivalence_receipts = Vec::new();
    for (key, hashes) in &collisions {
        if hashes.len() > 1 {
            equivalence_receipts.push(IngressEquivalenceReceiptV1::for_input_collision(
                tick,
                &key.0,
                key.1,
                key.2,
                hashes.clone(),
            )?);
        }
    }
    for sample in &samples {
        let key = (
            sample.source_class.as_str().to_owned(),
            sample.source_id,
            sample.source_sequence,
        );
        if !collision_keys.contains(&key) {
            assignments.push(IngressAssignmentV1::from_sample(
                checkpoint.current_generation,
                tick,
                sample,
            )?);
        }
    }
    assignments.sort();
    equivalence_receipts.sort();

    let mut mapping = map_player_actions(
        tick,
        &samples,
        &collision_keys,
        controllers,
        interaction_enabled,
    )?;
    sort_command_batch(&mut mapping.commands)?;
    let body = ClosedIngressBatchBodyV1 {
        schema_version: CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
        queue_generation: checkpoint.current_generation,
        assigned_tick: tick,
        input_samples: samples,
        completion_signals: Vec::new(),
        input_assignments: assignments,
        completion_assignments: Vec::new(),
        equivalence_receipts,
    };
    let batch = ClosedIngressBatchV1::from_body(body)?;
    batch.validate(admission)?;

    checkpoint.current_tick = following_tick;
    checkpoint.current_generation = next_generation;
    checkpoint.current_samples = std::mem::take(&mut checkpoint.next_samples);
    checkpoint.last_closed_batch_hash = Some(batch.batch_hash);
    checkpoint.validate(admission)?;

    Ok(ClosedIngressExecution {
        batch,
        mapping_receipts: mapping.receipts,
        derived_commands: mapping.commands,
        pending_interactions: mapping.pending_interactions,
        deduplicated,
    })
}

fn accept_closed_ingress(
    tick: u64,
    following_tick: u64,
    admission: &RuntimeAdmissionLimitsV1,
    controllers: &PlayerControllerRegistryV1,
    interaction_enabled: bool,
    checkpoint: &mut IngressCheckpointV1,
    batch: ClosedIngressBatchV1,
) -> Result<ClosedIngressExecution, RuntimeFatalError> {
    batch.validate(admission)?;
    if checkpoint.current_tick != tick
        || batch.body.assigned_tick != tick
        || batch.body.queue_generation != checkpoint.current_generation
    {
        return Err(RuntimeFatalError::IngressCheckpointCorrupt);
    }
    let next_generation = checkpoint
        .current_generation
        .checked_add(1)
        .ok_or(RuntimeFatalError::IngressGenerationExhausted)?;

    if !checkpoint.current_samples.is_empty() {
        let mut queued = checkpoint.current_samples.clone();
        queued.sort_by(|left, right| {
            left.sort_key()
                .expect("validated queued input has a sort key")
                .cmp(
                    &right
                        .sort_key()
                        .expect("validated queued input has a sort key"),
                )
        });
        queued.dedup_by(|left, right| {
            left.canonical_bytes().expect("validated queued input")
                == right.canonical_bytes().expect("validated queued input")
        });
        if queued != batch.body.input_samples {
            return Err(RuntimeFatalError::IngressCheckpointCorrupt);
        }
    }

    let assigned = batch
        .body
        .input_assignments
        .iter()
        .map(|assignment| {
            (
                assignment.source_class.as_str().to_owned(),
                assignment.source_id,
                assignment.source_sequence,
                assignment.payload_hash,
            )
        })
        .collect::<BTreeSet<_>>();
    let mut collision_keys = BTreeSet::new();
    for sample in &batch.body.input_samples {
        let key = (
            sample.source_class.as_str().to_owned(),
            sample.source_id,
            sample.source_sequence,
            sample.payload_hash()?,
        );
        if !assigned.contains(&key) {
            collision_keys.insert((key.0, key.1, key.2));
        }
    }
    if collision_keys.len() != batch.body.equivalence_receipts.len() {
        return Err(RuntimeFatalError::IngressCheckpointCorrupt);
    }
    let mut mapping = map_player_actions(
        tick,
        &batch.body.input_samples,
        &collision_keys,
        controllers,
        interaction_enabled,
    )?;
    sort_command_batch(&mut mapping.commands)?;

    checkpoint.current_tick = following_tick;
    checkpoint.current_generation = next_generation;
    checkpoint.current_samples = std::mem::take(&mut checkpoint.next_samples);
    checkpoint.last_closed_batch_hash = Some(batch.batch_hash);
    checkpoint.validate(admission)?;

    Ok(ClosedIngressExecution {
        batch,
        mapping_receipts: mapping.receipts,
        derived_commands: mapping.commands,
        pending_interactions: mapping.pending_interactions,
        deduplicated: 0,
    })
}

fn map_player_actions(
    tick: u64,
    samples: &[InputSampleV1],
    collision_keys: &BTreeSet<(String, next_contracts::InputSourceId, u64)>,
    controllers: &PlayerControllerRegistryV1,
    interaction_enabled: bool,
) -> Result<PlayerActionMapping, RuntimeFatalError> {
    enum MappedAction {
        Movement {
            binding: PlayerControllerBindingV1,
            sequence: u64,
            direction_q15: [i16; 2],
            receipt_index: usize,
        },
        Interaction {
            binding: PlayerControllerBindingV1,
            sequence: u64,
            payload_hash: ContentHash,
        },
        Noop,
    }

    let mut receipts = Vec::new();
    let mut mapped = BTreeMap::new();
    for sample in samples {
        let key = (
            sample.source_class.as_str().to_owned(),
            sample.source_id,
            sample.source_sequence,
        );
        if collision_keys.contains(&key) {
            continue;
        }
        let payload_hash = sample.payload_hash()?;
        let mapped_action = map_player_action_sample(sample, controllers, interaction_enabled);
        let (code, action) = match mapped_action {
            Ok(action) => (InputMappingCodeV1::Accepted, Some(action)),
            Err(code) => (code, None),
        };
        let receipt_index = receipts.len();
        receipts.push(InputMappingReceiptV1 {
            assigned_tick: tick,
            source_id: sample.source_id,
            source_sequence: sample.source_sequence,
            payload_hash,
            code,
            derived_command_id: None,
        });
        if let Some(action) = action {
            let controller_id = match &action {
                MappedPlayerAction::Movement { binding, .. }
                | MappedPlayerAction::Interaction { binding }
                | MappedPlayerAction::Noop { binding } => binding.controller_id,
            };
            let action = match action {
                MappedPlayerAction::Movement {
                    binding,
                    direction_q15,
                } => MappedAction::Movement {
                    binding: binding.clone(),
                    sequence: sample.source_sequence,
                    direction_q15,
                    receipt_index,
                },
                MappedPlayerAction::Interaction { binding } => MappedAction::Interaction {
                    binding: binding.clone(),
                    sequence: sample.source_sequence,
                    payload_hash,
                },
                MappedPlayerAction::Noop { .. } => MappedAction::Noop,
            };
            mapped.insert(controller_id, action);
        }
    }

    let mut commands = Vec::with_capacity(mapped.len());
    let mut pending_interactions = Vec::new();
    for action in mapped.into_values() {
        match action {
            MappedAction::Movement {
                binding,
                sequence,
                direction_q15,
                receipt_index,
            } => {
                let command = WorldCommand::physical(
                    binding.command_stream_id,
                    binding.principal,
                    sequence,
                    tick,
                    binding.controlled_body_id,
                    PhysicalCommandV1::SetCapsuleLocomotionIntent { direction_q15 },
                )?;
                receipts[receipt_index].derived_command_id = Some(command.compute_command_id()?);
                commands.push(command);
            }
            MappedAction::Interaction {
                binding,
                sequence,
                payload_hash,
            } => pending_interactions.push(PendingInteractionIntent {
                controlled_body_id: binding.controlled_body_id,
                source_id: binding.source_id,
                source_sequence: sequence,
                payload_hash,
            }),
            MappedAction::Noop => {}
        }
    }
    receipts.sort_by_key(|receipt| {
        (
            receipt.assigned_tick,
            receipt.source_id,
            receipt.source_sequence,
            receipt.payload_hash,
        )
    });
    Ok(PlayerActionMapping {
        receipts,
        commands,
        pending_interactions,
    })
}

enum MappedPlayerAction<'a> {
    Movement {
        binding: &'a PlayerControllerBindingV1,
        direction_q15: [i16; 2],
    },
    Interaction {
        binding: &'a PlayerControllerBindingV1,
    },
    Noop {
        binding: &'a PlayerControllerBindingV1,
    },
}

fn map_player_action_sample<'a>(
    sample: &InputSampleV1,
    controllers: &'a PlayerControllerRegistryV1,
    interaction_enabled: bool,
) -> Result<MappedPlayerAction<'a>, InputMappingCodeV1> {
    if sample.source_class.as_str() != PLAYER_ACTION_SOURCE_CLASS
        || sample.payload_schema_id.as_str() != PLAYER_ACTION_FRAME_SCHEMA_ID
        || sample.payload_schema_version != u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION)
    {
        return Err(InputMappingCodeV1::PayloadSchemaUnsupported);
    }
    let Some(binding) = controllers.bindings.get(&sample.source_id) else {
        return Err(InputMappingCodeV1::PrincipalUnbound);
    };
    let frame = PlayerActionFrameV1::from_canonical_bytes(
        &sample.payload,
        CanonicalDecodeLimits::default(),
    )
    .map_err(|_| InputMappingCodeV1::FrameInvalid)?;
    if frame.controller_id != binding.controller_id
        || frame.logical_frame_sequence != sample.source_sequence
    {
        return Err(InputMappingCodeV1::FrameInvalid);
    }
    if frame.action_map_hash != binding.action_map_hash
        || frame.action_map_revision != binding.action_map_revision
    {
        return Err(InputMappingCodeV1::ActionMapStale);
    }
    if frame.context_stack_hash != binding.context_stack_hash
        || frame.context_stack_revision != binding.context_stack_revision
    {
        return Err(InputMappingCodeV1::ContextStackStale);
    }
    if frame.actions.is_empty() {
        return Err(InputMappingCodeV1::ActionUnmapped);
    }
    let has_movement = frame
        .actions
        .iter()
        .any(|action| action.action_id.as_str() == CORE_MOVE_ACTION_ID);
    let has_interaction = frame
        .actions
        .iter()
        .any(|action| action.action_id.as_str() == CORE_INTERACT_ACTION_ID);
    let has_unknown = frame.actions.iter().any(|action| {
        !matches!(
            action.action_id.as_str(),
            CORE_MOVE_ACTION_ID | CORE_INTERACT_ACTION_ID
        )
    });
    if has_unknown || (has_movement && has_interaction) {
        return Err(InputMappingCodeV1::ActionUnmapped);
    }
    if has_interaction {
        if !interaction_enabled || frame.actions.len() != 1 {
            return Err(if interaction_enabled {
                InputMappingCodeV1::ValueOutOfProfile
            } else {
                InputMappingCodeV1::ActionUnmapped
            });
        }
        let action = &frame.actions[0];
        return match (action.phase, action.value) {
            (PlayerActionPhaseV1::Started, PlayerActionValueV1::Digital(true)) => {
                Ok(MappedPlayerAction::Interaction { binding })
            }
            (
                PlayerActionPhaseV1::Completed | PlayerActionPhaseV1::Cancelled,
                PlayerActionValueV1::Digital(false),
            ) => Ok(MappedPlayerAction::Noop { binding }),
            _ => Err(InputMappingCodeV1::ValueOutOfProfile),
        };
    }
    let mut direction = None;
    for action in &frame.actions {
        let PlayerActionValueV1::Vector2Q15(value) = action.value else {
            return Err(InputMappingCodeV1::ValueOutOfProfile);
        };
        let accepted = match action.phase {
            PlayerActionPhaseV1::Started | PlayerActionPhaseV1::Performed => {
                matches!(
                    value,
                    [32_767, 0] | [-32_767, 0] | [0, 32_767] | [0, -32_767]
                )
            }
            PlayerActionPhaseV1::Completed | PlayerActionPhaseV1::Cancelled => value == [0, 0],
        };
        if !accepted {
            return Err(InputMappingCodeV1::ValueOutOfProfile);
        }
        direction = Some(value);
    }
    Ok(MappedPlayerAction::Movement {
        binding,
        direction_q15: direction
            .expect("nonempty validated action frame has a final movement value"),
    })
}

fn resolve_interaction_outcome_route(
    principals: &PrincipalRegistryV1,
    streams: &CommandStreamRegistryV1,
    authority: &AuthorityRegistry,
) -> Option<InteractionOutcomeRoute> {
    let system_id = SystemId::new(PLAYER_INTERACTION_SYSTEM_ID)
        .expect("built-in interaction system identifier is valid");
    let principal = IssuerPrincipal::InternalSystem(system_id.clone());
    let capability = CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
        .expect("built-in RPG capability identifier is valid");
    if !principals.is_active(&principal)
        || !authority.is_authenticated(&principal)
        || !authority
            .grants(&principal)
            .is_some_and(|grants| grants.contains(&capability))
    {
        return None;
    }
    let stream_id = streams
        .entries
        .iter()
        .find(|(key, _)| {
            key.principal == principal && key.stream_slot == 0 && key.stream_epoch == 0
        })
        .map(|(_, stream_id)| *stream_id)?;
    Some(InteractionOutcomeRoute {
        system_id,
        stream_id,
    })
}

fn build_interaction_outcomes(
    intents: &[PendingInteractionIntent],
    route: Option<&InteractionOutcomeRoute>,
    physics: &ReferencePhysicsWorld,
    rpg: &RpgState,
    authoritative_revision: u64,
) -> Result<Vec<BuiltInInteractionOutcome>, RuntimeFatalError> {
    let Some(route) = route else {
        if intents.is_empty() {
            return Ok(Vec::new());
        }
        return Err(RuntimeFatalError::InternalIdentityCollision);
    };
    let rpg_snapshot = rpg.snapshot();
    let ready_state = SchemaId::new(CORE_INTERACTIVE_OBJECT_READY_STATE_ID)
        .expect("built-in interactive-object state identifier is valid");
    let activated_state = SchemaId::new(CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID)
        .expect("built-in interactive-object state identifier is valid");
    let mut outcomes = Vec::new();
    for intent in intents {
        let Some(target) =
            select_interaction_target(intent.controlled_body_id, physics, &rpg_snapshot)
        else {
            continue;
        };
        let proposal = crate::outcome::OutcomeProposal::rpg(
            route.system_id.clone(),
            route.stream_id,
            intent.source_sequence,
            RpgCommand::SetInteractiveObjectState {
                object_id: target,
                expected_state_id: ready_state.clone(),
                next_state_id: activated_state.clone(),
            },
        )
        .with_precondition_revision(authoritative_revision);
        outcomes.push(BuiltInInteractionOutcome {
            proposal,
            source_id: intent.source_id,
            source_sequence: intent.source_sequence,
            payload_hash: intent.payload_hash,
        });
    }
    Ok(outcomes)
}

fn select_interaction_target(
    controlled_body_id: PersistentId,
    physics: &ReferencePhysicsWorld,
    rpg: &RpgSnapshot,
) -> Option<PersistentId> {
    let physical_body_id = physics
        .checkpoint()
        .catalog
        .avatar_bindings
        .get(&controlled_body_id)?;
    let mut candidates = Vec::new();
    for contact in physics.snapshot().sorted_contact_continuity_states.values() {
        let other = if contact.participant_low.body_id == *physical_body_id {
            contact.participant_high.body_id
        } else if contact.participant_high.body_id == *physical_body_id {
            contact.participant_low.body_id
        } else {
            continue;
        };
        let target = other.subject_id;
        if rpg.interactive_objects.iter().any(|object| {
            object.id == target
                && object.archetype_id.as_str() == CORE_INTERACTIVE_OBJECT_ARCHETYPE_ID
                && object.state_id.as_str() == CORE_INTERACTIVE_OBJECT_READY_STATE_ID
        }) {
            candidates.push((target, contact.contact_id));
        }
    }
    candidates.sort();
    candidates.first().map(|(target, _)| *target)
}

fn sort_command_batch(commands: &mut [WorldCommand]) -> Result<(), CanonicalError> {
    let mut keys = Vec::with_capacity(commands.len());
    for command in commands.iter() {
        keys.push((
            command.stream_id,
            command.sequence,
            command.body_hash()?,
            command.claimed_command_id,
        ));
    }
    let mut indexed = commands
        .iter()
        .cloned()
        .zip(keys)
        .collect::<Vec<(WorldCommand, _)>>();
    indexed.sort_by(|left, right| left.1.cmp(&right.1));
    for (slot, (command, _)) in commands.iter_mut().zip(indexed) {
        *slot = command;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedCommand {
    command: WorldCommand,
    command_id: CommandId,
    canonical_bytes: Vec<u8>,
    order_key: CommandOrderKey,
    due: bool,
}

#[derive(Clone, Debug)]
struct QueuedCommand {
    command: WorldCommand,
    due: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidationSource {
    ExternalIngress,
    InternalOutcome,
}

#[derive(Debug)]
struct PhaseExecution {
    results: Vec<OrderedResult>,
    events: Vec<DomainEvent>,
    stage_trace: Vec<StageTraceEntry>,
    physics_step_input: Option<PhysicsStepInputV2>,
    contact_batch: Option<ClosedPhysicsContactBatchV1>,
}

#[derive(Clone, Copy)]
struct PhaseContext<'a> {
    registry: &'a CommandKindRegistry,
    authority: &'a AuthorityRegistry,
    principals: &'a PrincipalRegistryV1,
    streams: &'a CommandStreamRegistryV1,
    profile: &'a RuntimeDeterminismProfileV1,
    controllers: &'a PlayerControllerRegistryV1,
    source: ValidationSource,
    tick: u64,
    phase: CommandPhase,
    phase_revision: u64,
}

struct StagedAuthoritativeState {
    ledger: CommandLedgerV2,
    archive: CommandBodyArchiveV1,
    event_count: u64,
    revision: u64,
    rpg: RpgState,
    physics: ReferencePhysicsWorld,
    ingress: IngressCheckpointV1,
}

fn process_phase(
    context: PhaseContext<'_>,
    commands: Vec<WorldCommand>,
    staged: &mut StagedAuthoritativeState,
) -> Result<PhaseExecution, RuntimeFatalError> {
    let mut queued = due_commands(context, staged)?;
    queued.extend(commands.into_iter().map(|command| QueuedCommand {
        command,
        due: false,
    }));
    let received = count(queued.len())?;
    queued.sort_by(|left, right| left.command.cmp(&right.command));

    let mut results = Vec::new();
    let mut admission_rejected = 0_u64;
    let mut deduplicated = 0_u64;
    let mut valid = Vec::new();
    for queued in queued {
        let order_key = CommandOrderKey::from_command(&queued.command, context.registry);
        let command_id = queued.command.compute_command_id().unwrap_or_default();
        let sequence = queued.command.sequence;
        match validate_command(context, queued) {
            Ok(command) => valid.push(command),
            Err(code) => {
                admission_rejected = checked_inc(admission_rejected)?;
                results.push(OrderedResult::rejected(
                    order_key, command_id, sequence, code,
                ));
            }
        }
    }

    let mut groups: BTreeMap<CollisionKey, Vec<ValidatedCommand>> = BTreeMap::new();
    for command in valid {
        groups
            .entry((
                command.command.stream_id,
                command.command.issuer.clone(),
                command.command.sequence,
            ))
            .or_default()
            .push(command);
    }

    let mut candidates = Vec::new();
    for mut group in groups.into_values() {
        group.sort_by(compare_validated_commands);
        let mut unique: BTreeMap<Vec<u8>, ValidatedCommand> = BTreeMap::new();
        for candidate in group {
            match unique.get_mut(&candidate.canonical_bytes) {
                Some(existing) => {
                    existing.due |= candidate.due;
                    deduplicated = checked_inc(deduplicated)?;
                }
                None => {
                    unique.insert(candidate.canonical_bytes.clone(), candidate);
                }
            }
        }
        let group: Vec<_> = unique.into_values().collect();
        if group.len() > 1 {
            let collision_results = handle_collision(context, group, staged)?;
            admission_rejected = admission_rejected
                .checked_add(count(collision_results.len())?)
                .ok_or(RuntimeFatalError::TraceCountExhausted)?;
            results.extend(collision_results);
        } else if let Some(candidate) = group.into_iter().next() {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(compare_execution_candidates);
    let admitted = count(candidates.len())?;

    let mut commit_rejected = 0_u64;
    let mut committed = 0_u64;
    let mut commit_deduplicated = 0_u64;
    let mut events = Vec::new();
    let mut physical_pending = Vec::new();
    let mut physical_bodies = BTreeSet::new();
    let mut physics_step_input = None;
    let mut contact_batch = None;
    for candidate in candidates {
        match execute_candidate(context, candidate, staged, &mut physical_bodies)? {
            CandidateExecution::Result(result, trace) => {
                match trace {
                    ExecutionTrace::Rejected => commit_rejected = checked_inc(commit_rejected)?,
                    ExecutionTrace::Deduplicated => {
                        commit_deduplicated = checked_inc(commit_deduplicated)?
                    }
                    ExecutionTrace::Reserved => {}
                }
                results.push(result);
            }
            CandidateExecution::Committed(result, event) => {
                committed = checked_inc(committed)?;
                results.push(result);
                events.push(*event);
            }
            CandidateExecution::PhysicalPending(pending) => physical_pending.push(*pending),
        }
    }
    if context.source == ValidationSource::ExternalIngress {
        let physical_execution = finish_physical_step(context, physical_pending, staged)?;
        for (result, event) in physical_execution.command_results {
            committed = checked_inc(committed)?;
            results.push(result);
            if let Some(event) = event {
                events.push(event);
            }
        }
        physics_step_input = Some(physical_execution.step_input);
        contact_batch = Some(physical_execution.contact_batch);
    } else if !physical_pending.is_empty() {
        return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
    }

    results.sort();
    let admission_stage = match context.source {
        ValidationSource::ExternalIngress => TransactionStage::IngressAdmission,
        ValidationSource::InternalOutcome => TransactionStage::OutcomeAdmission,
    };
    let commit_stage = match context.source {
        ValidationSource::ExternalIngress => TransactionStage::IngressCommit,
        ValidationSource::InternalOutcome => TransactionStage::OutcomeCommit,
    };
    Ok(PhaseExecution {
        results,
        events,
        physics_step_input,
        contact_batch,
        stage_trace: vec![
            StageTraceEntry {
                stage: admission_stage,
                received,
                accepted: admitted,
                rejected: admission_rejected,
                committed: 0,
                deduplicated,
            },
            StageTraceEntry {
                stage: commit_stage,
                received: admitted,
                accepted: admitted
                    .checked_sub(commit_rejected)
                    .and_then(|value| value.checked_sub(commit_deduplicated))
                    .ok_or(RuntimeFatalError::TraceCountExhausted)?,
                rejected: commit_rejected,
                committed,
                deduplicated: commit_deduplicated,
            },
        ],
    })
}

fn due_commands(
    context: PhaseContext<'_>,
    staged: &StagedAuthoritativeState,
) -> Result<Vec<QueuedCommand>, RuntimeFatalError> {
    let mut commands = Vec::new();
    for stream in staged.ledger.streams.values() {
        for reservation in stream.pending.values() {
            if reservation.target_tick < context.tick {
                return Err(RuntimeFatalError::LedgerCorrupt(
                    CommandLedgerError::TargetTickRegression,
                ));
            }
            if reservation.target_tick == context.tick && reservation.phase == context.phase {
                let bytes = staged
                    .archive
                    .entries()
                    .get(&reservation.canonical_body_ref)
                    .ok_or(RuntimeFatalError::LedgerCorrupt(
                        CommandLedgerError::BodyReferenceMissing,
                    ))?;
                let command =
                    WorldCommand::from_canonical_bytes(bytes, CanonicalDecodeLimits::default())
                        .map_err(CommandLedgerError::from)
                        .map_err(RuntimeFatalError::LedgerCorrupt)?;
                commands.push(QueuedCommand { command, due: true });
            }
        }
    }
    Ok(commands)
}

fn validate_command(
    context: PhaseContext<'_>,
    queued: QueuedCommand,
) -> Result<ValidatedCommand, RejectionCode> {
    let command = queued.command;
    let descriptor = context
        .registry
        .descriptor(&command.payload_schema_id, command.payload_schema_version)
        .filter(|descriptor| descriptor.accepts_payload(&command.payload))
        .ok_or(RejectionCode::SchemaMismatch)?;
    if command.envelope_schema_version != COMMAND_ENVELOPE_SCHEMA_VERSION {
        return Err(RejectionCode::SchemaMismatch);
    }
    let canonical_bytes = command
        .canonical_bytes()
        .map_err(|_| RejectionCode::CanonicalCommandInvalid)?;
    let command_id = command
        .compute_command_id()
        .map_err(|_| RejectionCode::CanonicalCommandInvalid)?;
    if command
        .claimed_command_id
        .is_some_and(|claimed| claimed != command_id)
    {
        return Err(RejectionCode::CommandIdMismatch);
    }
    match context.source {
        ValidationSource::InternalOutcome
            if !matches!(command.issuer, IssuerPrincipal::InternalSystem(_)) =>
        {
            return Err(RejectionCode::InternalOutcomeIssuerRequired);
        }
        _ => {}
    }
    if !context.principals.is_active(&command.issuer) {
        return Err(RejectionCode::IssuerUnauthenticated);
    }
    let Some((stream_key, _)) = context.streams.binding(command.stream_id) else {
        return Err(RejectionCode::CommandStreamUnbound);
    };
    if stream_key.principal != command.issuer {
        return Err(RejectionCode::CommandStreamUnbound);
    }
    let grants = context
        .authority
        .grants(&command.issuer)
        .ok_or(RejectionCode::IssuerUnauthenticated)?;
    let declared: BTreeSet<_> = command
        .capability_claims
        .iter()
        .map(|claim| claim.capability_id.clone())
        .collect();
    if descriptor
        .required_capabilities()
        .iter()
        .any(|required| !declared.contains(required))
    {
        return Err(RejectionCode::CapabilityRequired);
    }
    if declared
        .iter()
        .any(|capability| !grants.contains(capability))
    {
        return Err(RejectionCode::CapabilityDenied);
    }
    Ok(ValidatedCommand {
        order_key: CommandOrderKey::new(&command, descriptor.priority_class()),
        command,
        command_id,
        canonical_bytes,
        due: queued.due,
    })
}

fn handle_collision(
    context: PhaseContext<'_>,
    candidates: Vec<ValidatedCommand>,
    staged: &mut StagedAuthoritativeState,
) -> Result<Vec<OrderedResult>, RuntimeFatalError> {
    let first = candidates
        .first()
        .expect("collision group is nonempty by construction");
    if matches!(first.command.issuer, IssuerPrincipal::InternalSystem(_)) {
        return Err(RuntimeFatalError::InternalIdentityCollision);
    }
    let stream_id = first.command.stream_id;
    let sequence = first.command.sequence;
    let stream = staged
        .ledger
        .streams
        .get(&stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?;
    if stream
        .admission_high_watermark
        .is_some_and(|high| sequence <= high)
        && !sequence_is_retained(stream, sequence)
    {
        return Ok(candidates
            .into_iter()
            .map(|candidate| {
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    sequence,
                    RejectionCode::CommandSequenceFinalized,
                )
            })
            .collect());
    }
    if stream.state == CommandStreamStateV1::CollisionLocked {
        return Ok(candidates
            .into_iter()
            .map(|candidate| {
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    sequence,
                    RejectionCode::CommandCollisionLocked,
                )
            })
            .collect());
    }

    let mut members = retained_collision_candidates(stream, sequence);
    for candidate in &candidates {
        insert_archive_identity(
            &mut staged.ledger,
            &mut staged.archive,
            &candidate.command,
            candidate.command_id,
        )?;
        members.push(CommandCollisionCandidateV1 {
            command_id: candidate.command_id,
            body_hash: candidate.command.body_hash()?,
            canonical_body_ref: candidate.command.body_hash()?,
        });
    }
    members.sort();
    members.dedup();
    let incident = CommandCollisionIncidentV1::new(
        stream_id,
        first.command.issuer.clone(),
        sequence,
        members,
    )?;
    let stream =
        staged
            .ledger
            .streams
            .get_mut(&stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?;
    if sequence_is_retained(stream, sequence) {
        stream.lock_for_collision(incident)?;
    } else {
        let root =
            empty_transaction_result_root(incident.candidates_root.as_bytes(), context.phase);
        let receipt = collision_receipt(context, stream, &incident, root)?;
        stream.append_collision_receipt(incident, receipt)?;
    }
    staged.ledger.synchronize_archive(&staged.archive)?;
    Ok(candidates
        .into_iter()
        .map(|candidate| {
            OrderedResult::rejected(
                candidate.order_key,
                candidate.command_id,
                sequence,
                RejectionCode::CommandSequenceCollision,
            )
        })
        .collect())
}

enum CandidateExecution {
    Result(OrderedResult, ExecutionTrace),
    Committed(OrderedResult, Box<DomainEvent>),
    PhysicalPending(Box<PhysicalPending>),
}

struct PhysicalPending {
    candidate: ValidatedCommand,
    intent: AcceptedLocomotionIntentV2,
}

#[derive(Clone, Copy)]
enum ExecutionTrace {
    Rejected,
    Deduplicated,
    Reserved,
}

fn execute_candidate(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
    physical_bodies: &mut BTreeSet<next_contracts::PersistentId>,
) -> Result<CandidateExecution, RuntimeFatalError> {
    let command = &candidate.command;
    let stream =
        staged
            .ledger
            .streams
            .get(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?;
    if let Some(result) = retained_result(stream, &candidate)? {
        return Ok(CandidateExecution::Result(
            result,
            ExecutionTrace::Deduplicated,
        ));
    }
    if stream
        .receipt_window
        .iter()
        .any(|receipt| receipt.subject.sequence() == command.sequence)
    {
        return collision_with_retained(context, candidate, staged);
    }
    if let Some(reservation) = stream.pending.get(&command.sequence) {
        if reservation.command_id == candidate.command_id
            && reservation.body_hash == command.body_hash()?
        {
            if !candidate.due {
                return Ok(CandidateExecution::Result(
                    OrderedResult::reserved(
                        candidate.order_key,
                        candidate.command_id,
                        command.sequence,
                    ),
                    ExecutionTrace::Deduplicated,
                ));
            }
        } else {
            return collision_with_retained(context, candidate, staged);
        }
    } else if stream
        .admission_high_watermark
        .is_some_and(|high| command.sequence <= high)
    {
        return Ok(CandidateExecution::Result(
            OrderedResult::rejected(
                candidate.order_key,
                candidate.command_id,
                command.sequence,
                RejectionCode::CommandSequenceFinalized,
            ),
            ExecutionTrace::Rejected,
        ));
    }
    match stream.state {
        CommandStreamStateV1::CollisionLocked => {
            return Ok(CandidateExecution::Result(
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                    RejectionCode::CommandCollisionLocked,
                ),
                ExecutionTrace::Rejected,
            ));
        }
        CommandStreamStateV1::Closed => {
            return Ok(CandidateExecution::Result(
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                    RejectionCode::CommandStreamClosed,
                ),
                ExecutionTrace::Rejected,
            ));
        }
        CommandStreamStateV1::Exhausted => {
            return Ok(CandidateExecution::Result(
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                    RejectionCode::CommandSequenceExhausted,
                ),
                ExecutionTrace::Rejected,
            ));
        }
        CommandStreamStateV1::Open => {}
    }

    if !candidate.due
        && staged
            .ledger
            .streams
            .get(&command.stream_id)
            .is_some_and(|stream| stream.pending.len() >= next_contracts::COMMAND_PENDING_CAPACITY)
    {
        return Ok(CandidateExecution::Result(
            OrderedResult::rejected(
                candidate.order_key,
                candidate.command_id,
                command.sequence,
                RejectionCode::CommandPendingLimit,
            ),
            ExecutionTrace::Rejected,
        ));
    }

    if !candidate.due {
        let identity_result = insert_archive_identity(
            &mut staged.ledger,
            &mut staged.archive,
            command,
            candidate.command_id,
        )?;
        if identity_result == IdentityInsertResult::Collision {
            return collision_from_identity_index(context, candidate, staged);
        }
        staged.ledger.synchronize_archive(&staged.archive)?;
        let descriptor = context
            .registry
            .descriptor(&command.payload_schema_id, command.payload_schema_version)
            .expect("pre-ledger schema validation resolved the descriptor");
        if command.phase != context.phase || !descriptor.allows_phase(command.phase) {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::CommandPhaseForbidden,
            );
        }
        match &command.payload {
            CommandPayload::Physical(_) if command.target.is_none() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::PhysicalTargetUnbound,
                );
            }
            CommandPayload::Physical(_) => {}
            _ if command.target.is_some() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::TargetNotAllowed,
                );
            }
            _ => {}
        }
        if command.target_tick < context.tick {
            return finalize_rejection(context, candidate, staged, RejectionCode::CommandExpired);
        }
        let horizon = context
            .tick
            .checked_add(u64::from(context.profile.maximum_future_command_ticks))
            .ok_or(RuntimeFatalError::TickExhausted)?;
        if command.target_tick > horizon {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::CommandFutureLimit,
            );
        }
        let reservation = CommandReservationV1::from_command(
            command,
            context.tick,
            candidate.order_key.priority_class,
            context.registry.canonical_hash(),
        )?;
        let reserve = staged
            .ledger
            .streams
            .get_mut(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?
            .reserve(reservation);
        if let Err(CommandLedgerError::TargetTickRegression) = reserve {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::CommandStreamTimeRegression,
            );
        }
        reserve?;
        if command.target_tick > context.tick {
            return Ok(CandidateExecution::Result(
                OrderedResult::reserved(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                ),
                ExecutionTrace::Reserved,
            ));
        }
    }

    if command
        .precondition_revision()
        .is_some_and(|revision| revision != context.phase_revision)
    {
        return finalize_rejection(
            context,
            candidate,
            staged,
            RejectionCode::PreconditionFailed,
        );
    }

    if let CommandPayload::Physical(PhysicalCommandV1::SetCapsuleLocomotionIntent {
        direction_q15,
    }) = &command.payload
    {
        let body_id = command
            .target
            .expect("physical target presence was validated after admission");
        let binding_matches = context.controllers.bindings.values().any(|binding| {
            binding.principal == command.issuer
                && binding.command_stream_id == command.stream_id
                && binding.controlled_body_id == body_id
        });
        if !binding_matches {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalTargetUnbound,
            );
        }
        let Some(physics_body_id) = staged
            .physics
            .checkpoint()
            .catalog
            .avatar_bindings
            .get(&body_id)
            .copied()
        else {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalTargetUnbound,
            );
        };
        let Some(body) = staged
            .physics
            .snapshot()
            .sorted_body_states
            .get(&physics_body_id)
        else {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalTargetUnbound,
            );
        };
        if !body.active {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalBodyInactive,
            );
        }
        if !physical_bodies.insert(body_id) {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalIntentAlreadyAssigned,
            );
        }
        let intent = AcceptedLocomotionIntentV2 {
            causal_command_id: candidate.command_id,
            controlled_target_id: body_id,
            body_id: physics_body_id,
            target_gameplay_tick: context.tick,
            direction_q15: *direction_q15,
        };
        return Ok(CandidateExecution::PhysicalPending(Box::new(
            PhysicalPending { candidate, intent },
        )));
    }

    let before_rpg = staged.rpg.snapshot();
    let (event, after_rpg) = match &command.payload {
        CommandPayload::Noop => (
            DomainEvent::command_committed(
                context.tick,
                context.phase,
                candidate.command_id,
                command.sequence,
            )?,
            staged.rpg.clone(),
        ),
        CommandPayload::Rpg(rpg_command) => {
            let mut next_rpg = staged.rpg.clone();
            let rpg_event = match next_rpg.apply(rpg_command) {
                Ok(event) => event,
                Err(error) => {
                    return finalize_rejection(
                        context,
                        candidate,
                        staged,
                        rpg_rejection_code(error),
                    );
                }
            };
            (
                DomainEvent::rpg(
                    context.tick,
                    context.phase,
                    candidate.command_id,
                    0,
                    rpg_event,
                )?,
                next_rpg,
            )
        }
        CommandPayload::Physical(_) => {
            unreachable!("physical commands return a pending step before domain execution")
        }
    };
    let provenance = event.canonical_bytes()?;
    let insert = staged
        .ledger
        .causal_identity_registry
        .compare_or_insert_provenance(
            CausalIdentityKind::DomainEvent,
            *event.event_id.as_bytes(),
            &provenance,
        )?;
    if insert == IdentityInsertResult::Collision {
        return Err(RuntimeFatalError::InternalIdentityCollision);
    }
    let event_ids = vec![event.event_id];
    let transaction_root = transaction_result_root(
        candidate.command_id.as_bytes(),
        context.phase,
        &canonical_rpg_delta(&before_rpg, &after_rpg.snapshot())?,
        &event_ids,
    );
    let receipt =
        command_receipt(
            context,
            staged.ledger.streams.get(&command.stream_id).ok_or(
                RuntimeFatalError::LedgerCorrupt(CommandLedgerError::StreamKeyMismatch),
            )?,
            command,
            candidate.command_id,
            CommandFinalResultV1::Committed,
            event_ids,
            transaction_root,
        )?;
    staged
        .ledger
        .streams
        .get_mut(&command.stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?
        .append_receipt(receipt)?;
    staged.event_count = staged
        .event_count
        .checked_add(1)
        .ok_or(RuntimeFatalError::EventCountExhausted)?;
    staged.revision = staged
        .revision
        .checked_add(1)
        .ok_or(RuntimeFatalError::RevisionExhausted)?;
    staged.rpg = after_rpg;
    Ok(CandidateExecution::Committed(
        OrderedResult::committed(candidate.order_key, candidate.command_id, command.sequence),
        Box::new(event),
    ))
}

struct PhysicalStepExecution {
    command_results: Vec<(OrderedResult, Option<DomainEvent>)>,
    step_input: PhysicsStepInputV2,
    contact_batch: ClosedPhysicsContactBatchV1,
}

fn finish_physical_step(
    context: PhaseContext<'_>,
    pending: Vec<PhysicalPending>,
    staged: &mut StagedAuthoritativeState,
) -> Result<PhysicalStepExecution, RuntimeFatalError> {
    let before_snapshot = staged.physics.snapshot().clone();
    let mut intents = pending
        .iter()
        .map(|pending| pending.intent.clone())
        .collect::<Vec<_>>();
    intents.sort_by_key(|intent| (intent.body_id, intent.causal_command_id));
    let step_input = PhysicsStepInputV2 {
        schema_version: PHYSICS_STEP_INPUT_SCHEMA_VERSION,
        world_id: before_snapshot.world_id,
        expected_world_revision: before_snapshot.world_revision,
        expected_snapshot_hash: before_snapshot.snapshot_hash()?,
        expected_catalog_hash: staged.physics.checkpoint().catalog.catalog_hash()?,
        gameplay_tick: context.tick,
        first_physics_tick: before_snapshot
            .physics_tick
            .checked_add(1)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?,
        physics_substeps: staged
            .physics
            .tick_rate_profile()
            .physics_substeps_per_gameplay_tick,
        accepted_intents: intents,
    };

    let step_result = staged.physics.step(&step_input)?;
    if step_result.applied_locomotion.len() != pending.len() {
        return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
    }
    if step_result.before_snapshot_hash != step_result.after_snapshot_hash {
        staged.revision = staged
            .revision
            .checked_add(1)
            .ok_or(RuntimeFatalError::RevisionExhausted)?;
    }
    let applied_by_command = step_result
        .applied_locomotion
        .into_iter()
        .map(|applied| (applied.causal_command_id, applied))
        .collect::<BTreeMap<_, _>>();

    let mut results = Vec::with_capacity(pending.len());
    for pending in pending {
        let applied = applied_by_command
            .get(&pending.candidate.command_id)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
        if pending.intent.body_id != applied.body_id {
            return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
        }
        let before = before_snapshot
            .sorted_body_states
            .get(&applied.body_id)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
        let after = staged
            .physics
            .snapshot()
            .sorted_body_states
            .get(&applied.body_id)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
        let command = &pending.candidate.command;
        let event = (applied.applied_displacement_micrometres[0] != 0
            || applied.applied_displacement_micrometres[2] != 0)
            .then(|| {
                DomainEvent::physical(
                    context.tick,
                    context.phase,
                    pending.candidate.command_id,
                    0,
                    PhysicalEventV1::CapsuleStepApplied {
                        body_id: pending.intent.controlled_target_id,
                        physics_tick: staged.physics.snapshot().physics_tick,
                        before: before.pose,
                        after: after.pose,
                    },
                )
            })
            .transpose()?;
        let event_ids = event
            .as_ref()
            .map_or_else(Vec::new, |event| vec![event.event_id]);
        if let Some(event) = &event {
            let provenance = event.canonical_bytes()?;
            let insert = staged
                .ledger
                .causal_identity_registry
                .compare_or_insert_provenance(
                    CausalIdentityKind::DomainEvent,
                    *event.event_id.as_bytes(),
                    &provenance,
                )?;
            if insert == IdentityInsertResult::Collision {
                return Err(RuntimeFatalError::InternalIdentityCollision);
            }
        }
        let delta = if pending.intent.direction_q15 == [0, 0] {
            Vec::new()
        } else {
            applied.canonical_bytes()?
        };
        let transaction_root = transaction_result_root(
            pending.candidate.command_id.as_bytes(),
            context.phase,
            &delta,
            &event_ids,
        );
        let receipt = command_receipt(
            context,
            staged.ledger.streams.get(&command.stream_id).ok_or(
                RuntimeFatalError::LedgerCorrupt(CommandLedgerError::StreamKeyMismatch),
            )?,
            command,
            pending.candidate.command_id,
            CommandFinalResultV1::Committed,
            event_ids,
            transaction_root,
        )?;
        staged
            .ledger
            .streams
            .get_mut(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?
            .append_receipt(receipt)?;
        if event.is_some() {
            staged.event_count = staged
                .event_count
                .checked_add(1)
                .ok_or(RuntimeFatalError::EventCountExhausted)?;
        }
        results.push((
            OrderedResult::committed(
                pending.candidate.order_key,
                pending.candidate.command_id,
                command.sequence,
            ),
            event,
        ));
    }
    Ok(PhysicalStepExecution {
        command_results: results,
        step_input,
        contact_batch: step_result.contact_batch,
    })
}

fn retained_result(
    stream: &CommandStreamLedgerV2,
    candidate: &ValidatedCommand,
) -> Result<Option<OrderedResult>, RuntimeFatalError> {
    let command = &candidate.command;
    let body_hash = command.body_hash()?;
    let Some(receipt) = stream
        .receipt_window
        .iter()
        .find(|receipt| receipt.subject.sequence() == command.sequence)
    else {
        return Ok(None);
    };
    let exact = match &receipt.subject {
        CommandReceiptSubjectV1::Command {
            command_id,
            body_hash: retained_hash,
            ..
        } => *command_id == candidate.command_id && *retained_hash == body_hash,
        CommandReceiptSubjectV1::CollisionSet { candidates, .. } => {
            candidates.iter().any(|member| {
                member.command_id == candidate.command_id && member.body_hash == body_hash
            })
        }
    };
    if !exact {
        return Ok(None);
    }
    let result = match &receipt.result {
        CommandFinalResultV1::Committed => OrderedResult::deduplicated(
            candidate.order_key.clone(),
            candidate.command_id,
            command.sequence,
        ),
        CommandFinalResultV1::Rejected { code } => OrderedResult::rejected(
            candidate.order_key.clone(),
            candidate.command_id,
            command.sequence,
            RejectionCode::from_stable_code(code.as_str()),
        ),
        CommandFinalResultV1::Collision { .. } => OrderedResult::rejected(
            candidate.order_key.clone(),
            candidate.command_id,
            command.sequence,
            RejectionCode::CommandSequenceCollision,
        ),
    };
    Ok(Some(result))
}

fn collision_with_retained(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
) -> Result<CandidateExecution, RuntimeFatalError> {
    let results = handle_collision(context, vec![candidate], staged)?;
    Ok(CandidateExecution::Result(
        results
            .into_iter()
            .next()
            .expect("one collision candidate returns one result"),
        ExecutionTrace::Rejected,
    ))
}

fn collision_from_identity_index(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
) -> Result<CandidateExecution, RuntimeFatalError> {
    if matches!(candidate.command.issuer, IssuerPrincipal::InternalSystem(_)) {
        return Err(RuntimeFatalError::InternalIdentityCollision);
    }
    let binding = staged
        .ledger
        .identity_index
        .body
        .bindings
        .get(&candidate.command_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::IdentityReferenceMissing,
        ))?;
    let members = binding
        .occurrences
        .iter()
        .map(|occurrence| CommandCollisionCandidateV1 {
            command_id: candidate.command_id,
            body_hash: occurrence.body_hash,
            canonical_body_ref: occurrence.body_hash,
        })
        .collect();
    let incident = CommandCollisionIncidentV1::new(
        candidate.command.stream_id,
        candidate.command.issuer.clone(),
        candidate.command.sequence,
        members,
    )?;
    let stream = staged
        .ledger
        .streams
        .get_mut(&candidate.command.stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?;
    if sequence_is_retained(stream, candidate.command.sequence) {
        stream.lock_for_collision(incident)?;
    } else {
        let root =
            empty_transaction_result_root(incident.candidates_root.as_bytes(), context.phase);
        let receipt = collision_receipt(context, stream, &incident, root)?;
        stream.append_collision_receipt(incident, receipt)?;
    }
    Ok(CandidateExecution::Result(
        OrderedResult::rejected(
            candidate.order_key,
            candidate.command_id,
            candidate.command.sequence,
            RejectionCode::CommandIdCollision,
        ),
        ExecutionTrace::Rejected,
    ))
}

fn finalize_rejection(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
    code: RejectionCode,
) -> Result<CandidateExecution, RuntimeFatalError> {
    let command = &candidate.command;
    let root = empty_transaction_result_root(candidate.command_id.as_bytes(), context.phase);
    let receipt =
        command_receipt(
            context,
            staged.ledger.streams.get(&command.stream_id).ok_or(
                RuntimeFatalError::LedgerCorrupt(CommandLedgerError::StreamKeyMismatch),
            )?,
            command,
            candidate.command_id,
            CommandFinalResultV1::Rejected {
                code: next_contracts::SchemaId::new(code.as_str())
                    .expect("stable rejection code is a valid identifier"),
            },
            Vec::new(),
            root,
        )?;
    staged
        .ledger
        .streams
        .get_mut(&command.stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?
        .append_receipt(receipt)?;
    Ok(CandidateExecution::Result(
        OrderedResult::rejected(
            candidate.order_key,
            candidate.command_id,
            command.sequence,
            code,
        ),
        ExecutionTrace::Rejected,
    ))
}

fn command_receipt(
    context: PhaseContext<'_>,
    stream: &CommandStreamLedgerV2,
    command: &WorldCommand,
    command_id: CommandId,
    result: CommandFinalResultV1,
    event_ids: Vec<next_contracts::EventId>,
    transaction_result_root: ContentHash,
) -> Result<CommandReceiptV1, RuntimeFatalError> {
    let body_hash = command.body_hash()?;
    Ok(CommandReceiptV1 {
        schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
        finalization_ordinal: stream.finalized_receipt_count,
        subject: CommandReceiptSubjectV1::Command {
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            sequence: command.sequence,
            command_id,
            body_hash,
            canonical_body_ref: body_hash,
        },
        phase: command.phase,
        target_tick: command.target_tick,
        finalized_at_tick: context.tick,
        priority_class: CommandOrderKey::from_command(command, context.registry).priority_class,
        command_kind_registry_hash: context.registry.canonical_hash(),
        result,
        diagnostic_digest: None,
        event_ids,
        transaction_result_root,
    })
}

fn collision_receipt(
    context: PhaseContext<'_>,
    stream: &CommandStreamLedgerV2,
    incident: &CommandCollisionIncidentV1,
    transaction_result_root: ContentHash,
) -> Result<CommandReceiptV1, RuntimeFatalError> {
    Ok(CommandReceiptV1 {
        schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
        finalization_ordinal: stream.finalized_receipt_count,
        subject: CommandReceiptSubjectV1::CollisionSet {
            stream_id: incident.stream_id,
            issuer: incident.issuer.clone(),
            sequence: incident.sequence,
            candidates_root: incident.candidates_root,
            candidate_count: u32::try_from(incident.candidates.len())
                .map_err(|_| RuntimeFatalError::TraceCountExhausted)?,
            candidates: incident.candidates.clone(),
        },
        phase: context.phase,
        target_tick: context.tick,
        finalized_at_tick: context.tick,
        priority_class: u16::MAX,
        command_kind_registry_hash: context.registry.canonical_hash(),
        result: CommandFinalResultV1::Collision {
            code: next_contracts::SchemaId::new(RejectionCode::CommandSequenceCollision.as_str())
                .expect("stable collision code is valid"),
        },
        diagnostic_digest: None,
        event_ids: Vec::new(),
        transaction_result_root,
    })
}

fn insert_archive_identity(
    ledger: &mut CommandLedgerV2,
    archive: &mut CommandBodyArchiveV1,
    command: &WorldCommand,
    command_id: CommandId,
) -> Result<IdentityInsertResult, RuntimeFatalError> {
    let body_hash = command.body_hash()?;
    archive.insert_command(command)?;
    let result = ledger.identity_index.insert_occurrence(
        command_id,
        CommandIdentityOccurrenceV1 {
            body_hash,
            first_stream_id: command.stream_id,
            first_sequence: command.sequence,
        },
    )?;
    Ok(result)
}

fn sequence_is_retained(stream: &CommandStreamLedgerV2, sequence: u64) -> bool {
    stream.pending.contains_key(&sequence)
        || stream
            .receipt_window
            .iter()
            .any(|receipt| receipt.subject.sequence() == sequence)
}

fn retained_collision_candidates(
    stream: &CommandStreamLedgerV2,
    sequence: u64,
) -> Vec<CommandCollisionCandidateV1> {
    if let Some(reservation) = stream.pending.get(&sequence) {
        return vec![CommandCollisionCandidateV1 {
            command_id: reservation.command_id,
            body_hash: reservation.body_hash,
            canonical_body_ref: reservation.canonical_body_ref,
        }];
    }
    stream
        .receipt_window
        .iter()
        .find(|receipt| receipt.subject.sequence() == sequence)
        .map_or_else(Vec::new, |receipt| match &receipt.subject {
            CommandReceiptSubjectV1::Command {
                command_id,
                body_hash,
                canonical_body_ref,
                ..
            } => vec![CommandCollisionCandidateV1 {
                command_id: *command_id,
                body_hash: *body_hash,
                canonical_body_ref: *canonical_body_ref,
            }],
            CommandReceiptSubjectV1::CollisionSet { candidates, .. } => candidates.clone(),
        })
}

fn canonical_rpg_delta(
    before: &RpgSnapshot,
    after: &RpgSnapshot,
) -> Result<Vec<u8>, CanonicalError> {
    if before == after {
        return Ok(Vec::new());
    }
    let before_bytes = before.canonical_bytes()?;
    let after_bytes = after.canonical_bytes()?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.rpg-owner-delta.v1\0");
    bytes.extend_from_slice(&sha256(&before_bytes));
    bytes.extend_from_slice(&sha256(&after_bytes));
    Ok(bytes)
}

fn empty_transaction_result_root(identity: &[u8], phase: CommandPhase) -> ContentHash {
    transaction_result_root(identity, phase, &[], &[])
}

fn transaction_result_root(
    identity: &[u8],
    phase: CommandPhase,
    delta_bytes: &[u8],
    event_ids: &[next_contracts::EventId],
) -> ContentHash {
    let mut delta_preimage = Vec::new();
    delta_preimage.extend_from_slice(b"nextengine.transaction-deltas.v1\0");
    delta_preimage.extend_from_slice(&(delta_bytes.len() as u64).to_le_bytes());
    delta_preimage.extend_from_slice(delta_bytes);
    let delta_root = sha256(&delta_preimage);
    let mut event_preimage = Vec::new();
    event_preimage.extend_from_slice(b"nextengine.transaction-events.v1\0");
    event_preimage.extend_from_slice(&(event_ids.len() as u32).to_le_bytes());
    for event_id in event_ids {
        event_preimage.extend_from_slice(event_id.as_bytes());
    }
    let event_root = sha256(&event_preimage);
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.transaction-result.v1\0");
    preimage.extend_from_slice(identity);
    preimage.push(phase as u8);
    preimage.extend_from_slice(&delta_root);
    preimage.extend_from_slice(&event_root);
    content_hash_from_bytes(sha256(&preimage))
}

fn validate_bootstrap(
    bootstrap: &RuntimeBootstrapV3,
    authority: &AuthorityRegistry,
    registry: &CommandKindRegistry,
) -> Result<(), SnapshotRestoreError> {
    bootstrap.world_identity.validate()?;
    bootstrap.principal_registry.validate()?;
    bootstrap.stream_registry.validate()?;
    bootstrap.runtime_profile.validate()?;
    bootstrap.admission_limits.validate()?;
    bootstrap.tick_rate_profile.validate()?;
    bootstrap.ingress_assignment_profile.validate()?;
    bootstrap.authoritative_numeric_profile.validate()?;
    bootstrap.physics_quantization_profile.validate()?;
    bootstrap.player_controller_registry.validate()?;
    bootstrap.physics_checkpoint.validate()?;
    bootstrap
        .physics_checkpoint
        .snapshot
        .validate_profile_closure(
            &bootstrap.physics_checkpoint.catalog,
            &bootstrap.tick_rate_profile,
            &bootstrap.authoritative_numeric_profile,
            &bootstrap.physics_quantization_profile,
        )?;
    let world = bootstrap.world_identity.world_namespace;
    let profile_hash = bootstrap.runtime_profile.profile_hash()?;
    if bootstrap.world_identity.runtime_determinism_profile_hash != profile_hash
        || bootstrap.principal_registry.world_namespace != world
        || bootstrap.stream_registry.world_namespace != world
        || bootstrap.player_controller_registry.world_namespace != world
        || bootstrap.runtime_profile.command_kind_registry_hash != registry.canonical_hash()
        || bootstrap.runtime_profile.admission_limits_profile_hash
            != bootstrap.admission_limits.profile_hash()?
        || bootstrap.runtime_profile.tick_rate_profile_hash
            != bootstrap.tick_rate_profile.profile_hash()?
        || bootstrap.runtime_profile.ingress_assignment_profile_hash
            != bootstrap.ingress_assignment_profile.profile_hash()?
        || bootstrap.runtime_profile.numeric_profile_hash
            != bootstrap.authoritative_numeric_profile.profile_hash()?
        || bootstrap.runtime_profile.physics_quantization_profile_hash
            != bootstrap.physics_quantization_profile.profile_hash()?
        || bootstrap.ingress_assignment_profile.admission_limits_hash
            != bootstrap.admission_limits.profile_hash()?
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    for (principal, _) in authority.entries() {
        if !bootstrap.principal_registry.is_active(principal) {
            return Err(SnapshotRestoreError::InactivePrincipal);
        }
    }
    for key in bootstrap.stream_registry.entries.keys() {
        if !bootstrap.principal_registry.is_active(&key.principal)
            || !authority.is_authenticated(&key.principal)
        {
            return Err(SnapshotRestoreError::InactivePrincipal);
        }
    }
    for binding in bootstrap.player_controller_registry.bindings.values() {
        if !bootstrap.principal_registry.is_active(&binding.principal)
            || !authority.is_authenticated(&binding.principal)
            || bootstrap
                .stream_registry
                .binding(binding.command_stream_id)
                .is_none_or(|(key, _)| key.principal != binding.principal)
            || bootstrap
                .physics_checkpoint
                .catalog
                .avatar_bindings
                .get(&binding.controlled_body_id)
                .is_none_or(|body_id| {
                    !bootstrap
                        .physics_checkpoint
                        .snapshot
                        .sorted_body_states
                        .contains_key(body_id)
                })
        {
            return Err(SnapshotRestoreError::ControllerClosureMismatch);
        }
    }
    Ok(())
}

fn empty_physics_checkpoint(
    world_id: PhysicsWorldId,
    tick_rate_profile: &TickRateProfileV1,
    numeric_profile: &AuthoritativeNumericProfileV1,
    quantization_profile: &PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldCheckpointV1, PhysicsContractError> {
    let catalog = PhysicsWorldCatalogV1::new(
        world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1()
                .expect("built-in coordinate profile identifier is valid"),
            limits: PhysicsLimitsProfileV1::reference_v1()
                .expect("built-in limits profile identifier is valid"),
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1()
                .expect("built-in solver profile identifier is valid"),
            tick_rate_hash: tick_rate_profile.profile_hash()?,
            authoritative_numeric_hash: numeric_profile.profile_hash()?,
            quantization_hash: quantization_profile.profile_hash()?,
        },
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )?;
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(
        &catalog,
        tick_rate_profile,
        numeric_profile,
        quantization_profile,
    )?;
    PhysicsWorldCheckpointV1::new(catalog, snapshot)
}

fn register_bootstrap_identities(
    ledger: &mut CommandLedgerV2,
    principals: &PrincipalRegistryV1,
    streams: &CommandStreamRegistryV1,
) -> Result<(), SnapshotRestoreError> {
    for (principal, record) in &principals.principals {
        if let IssuerPrincipal::Player(id) = principal {
            let result = ledger.causal_identity_registry.compare_or_insert(
                CausalIdentityKey {
                    identity_kind: CausalIdentityKind::PlayerPrincipal,
                    identity_bytes: *id.as_bytes(),
                },
                record.provenance_hash,
            )?;
            if result == IdentityInsertResult::Collision {
                return Err(SnapshotRestoreError::IdentityCollision);
            }
        }
    }
    for (key, stream_id) in &streams.entries {
        let principal = key.principal.canonical_bytes()?;
        let mut provenance = Vec::new();
        provenance.extend_from_slice(streams.world_namespace.as_bytes());
        provenance.extend_from_slice(&(principal.len() as u64).to_le_bytes());
        provenance.extend_from_slice(&principal);
        provenance.extend_from_slice(&key.stream_slot.to_le_bytes());
        provenance.extend_from_slice(&key.stream_epoch.to_le_bytes());
        let result = ledger
            .causal_identity_registry
            .compare_or_insert_provenance(
                CausalIdentityKind::CommandStream,
                *stream_id.as_bytes(),
                &provenance,
            )?;
        if result == IdentityInsertResult::Collision {
            return Err(SnapshotRestoreError::IdentityCollision);
        }
    }
    Ok(())
}

fn compare_validated_commands(left: &ValidatedCommand, right: &ValidatedCommand) -> Ordering {
    left.order_key.cmp(&right.order_key)
}

fn compare_execution_candidates(left: &ValidatedCommand, right: &ValidatedCommand) -> Ordering {
    (
        left.command.stream_id,
        left.command.sequence,
        &left.order_key,
    )
        .cmp(&(
            right.command.stream_id,
            right.command.sequence,
            &right.order_key,
        ))
}

fn count(value: usize) -> Result<u64, RuntimeFatalError> {
    u64::try_from(value).map_err(|_| RuntimeFatalError::TraceCountExhausted)
}

fn checked_inc(value: u64) -> Result<u64, RuntimeFatalError> {
    value
        .checked_add(1)
        .ok_or(RuntimeFatalError::TraceCountExhausted)
}

fn rpg_rejection_code(error: RpgApplyError) -> RejectionCode {
    match error {
        RpgApplyError::AggregateNotFound(_) => RejectionCode::RpgAggregateNotFound,
        RpgApplyError::StatePreconditionFailed(_) => RejectionCode::RpgStatePreconditionFailed,
        RpgApplyError::InvariantViolation(_) => RejectionCode::RpgInvariantViolation,
        RpgApplyError::RelationshipOverflow
        | RpgApplyError::SkillProficiencyOutOfRange
        | RpgApplyError::AggregateRevisionExhausted => RejectionCode::RpgValueOutOfRange,
        _ => RejectionCode::RpgInvariantViolation,
    }
}

fn validate_rpg_snapshot(snapshot: RpgSnapshot) -> Result<RpgState, SnapshotRestoreError> {
    let canonical_bytes = snapshot.canonical_bytes()?;
    let snapshot =
        RpgSnapshot::from_canonical_bytes(&canonical_bytes, CanonicalDecodeLimits::default())?;
    Ok(RpgState::from_snapshot(snapshot)?)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CommandOrderKey {
    target_tick: u64,
    phase: CommandPhase,
    priority_class: u16,
    issuer_tag: u8,
    issuer_id_bytes: Vec<u8>,
    sequence: u64,
    command_id: CommandId,
}

impl CommandOrderKey {
    fn new(command: &WorldCommand, priority_class: u16) -> Self {
        Self {
            target_tick: command.target_tick,
            phase: command.phase,
            priority_class,
            issuer_tag: command.issuer.tag(),
            issuer_id_bytes: command.issuer.identifier_bytes().to_vec(),
            sequence: command.sequence,
            command_id: command.compute_command_id().unwrap_or_default(),
        }
    }

    fn from_command(command: &WorldCommand, registry: &CommandKindRegistry) -> Self {
        let priority_class = registry
            .descriptor(&command.payload_schema_id, command.payload_schema_version)
            .filter(|descriptor| descriptor.accepts_payload(&command.payload))
            .map_or(u16::MAX, |descriptor| descriptor.priority_class());
        Self::new(command, priority_class)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RejectionCode {
    SchemaMismatch,
    CanonicalCommandInvalid,
    CommandIdMismatch,
    CommandPhaseForbidden,
    InternalOutcomeIssuerRequired,
    IssuerUnauthenticated,
    CommandStreamUnbound,
    CapabilityRequired,
    CapabilityDenied,
    PreconditionFailed,
    TargetNotAllowed,
    PhysicalTargetUnbound,
    PhysicalBodyInactive,
    PhysicalIntentAlreadyAssigned,
    CommandExpired,
    CommandFutureLimit,
    CommandSequenceCollision,
    CommandIdCollision,
    CommandSequenceFinalized,
    CommandSequenceExhausted,
    CommandStreamTimeRegression,
    CommandPendingLimit,
    CommandCollisionLocked,
    CommandStreamClosed,
    RpgAggregateNotFound,
    RpgStatePreconditionFailed,
    RpgInvariantViolation,
    RpgValueOutOfRange,
}

impl RejectionCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SchemaMismatch => "COMMAND_SCHEMA_MISMATCH",
            Self::CanonicalCommandInvalid => "CANONICAL_COMMAND_INVALID",
            Self::CommandIdMismatch => "COMMAND_ID_MISMATCH",
            Self::CommandPhaseForbidden => "COMMAND_PHASE_FORBIDDEN",
            Self::InternalOutcomeIssuerRequired => "INTERNAL_OUTCOME_ISSUER_REQUIRED",
            Self::IssuerUnauthenticated => "ISSUER_UNAUTHENTICATED",
            Self::CommandStreamUnbound => "COMMAND_STREAM_UNBOUND",
            Self::CapabilityRequired => "CAPABILITY_REQUIRED",
            Self::CapabilityDenied => "CAPABILITY_DENIED",
            Self::PreconditionFailed => "PRECONDITION_FAILED",
            Self::TargetNotAllowed => "COMMAND_TARGET_NOT_ALLOWED",
            Self::PhysicalTargetUnbound => "PHYSICAL_TARGET_UNBOUND",
            Self::PhysicalBodyInactive => "PHYSICAL_BODY_INACTIVE",
            Self::PhysicalIntentAlreadyAssigned => "PHYSICAL_INTENT_ALREADY_ASSIGNED",
            Self::CommandExpired => "COMMAND_EXPIRED",
            Self::CommandFutureLimit => "COMMAND_FUTURE_LIMIT",
            Self::CommandSequenceCollision => "COMMAND_SEQUENCE_COLLISION",
            Self::CommandIdCollision => "COMMAND_ID_COLLISION",
            Self::CommandSequenceFinalized => "COMMAND_SEQUENCE_FINALIZED",
            Self::CommandSequenceExhausted => "COMMAND_SEQUENCE_EXHAUSTED",
            Self::CommandStreamTimeRegression => "COMMAND_STREAM_TIME_REGRESSION",
            Self::CommandPendingLimit => "COMMAND_PENDING_LIMIT",
            Self::CommandCollisionLocked => "COMMAND_COLLISION_LOCKED",
            Self::CommandStreamClosed => "COMMAND_STREAM_CLOSED",
            Self::RpgAggregateNotFound => "RPG_AGGREGATE_NOT_FOUND",
            Self::RpgStatePreconditionFailed => "RPG_STATE_PRECONDITION_FAILED",
            Self::RpgInvariantViolation => "RPG_INVARIANT_VIOLATION",
            Self::RpgValueOutOfRange => "RPG_VALUE_OUT_OF_RANGE",
        }
    }

    fn from_stable_code(code: &str) -> Self {
        [
            Self::SchemaMismatch,
            Self::CanonicalCommandInvalid,
            Self::CommandIdMismatch,
            Self::CommandPhaseForbidden,
            Self::InternalOutcomeIssuerRequired,
            Self::IssuerUnauthenticated,
            Self::CommandStreamUnbound,
            Self::CapabilityRequired,
            Self::CapabilityDenied,
            Self::PreconditionFailed,
            Self::TargetNotAllowed,
            Self::PhysicalTargetUnbound,
            Self::PhysicalBodyInactive,
            Self::PhysicalIntentAlreadyAssigned,
            Self::CommandExpired,
            Self::CommandFutureLimit,
            Self::CommandSequenceCollision,
            Self::CommandIdCollision,
            Self::CommandSequenceFinalized,
            Self::CommandSequenceExhausted,
            Self::CommandStreamTimeRegression,
            Self::CommandPendingLimit,
            Self::CommandCollisionLocked,
            Self::CommandStreamClosed,
            Self::RpgAggregateNotFound,
            Self::RpgStatePreconditionFailed,
            Self::RpgInvariantViolation,
            Self::RpgValueOutOfRange,
        ]
        .into_iter()
        .find(|candidate| candidate.as_str() == code)
        .unwrap_or(Self::RpgInvariantViolation)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandDisposition {
    Reserved,
    Committed,
    Deduplicated,
    Rejected(RejectionCode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandResult {
    pub command_id: CommandId,
    pub sequence: u64,
    pub disposition: CommandDisposition,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TransactionStage {
    IngressClose,
    IngressAdmission,
    IngressCommit,
    PhysicalStep,
    PhysicsContactPublication,
    OutcomeCollection,
    OutcomeAdmission,
    OutcomeCommit,
    SnapshotPublication,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StageTraceEntry {
    pub stage: TransactionStage,
    pub received: u64,
    pub accepted: u64,
    pub rejected: u64,
    pub committed: u64,
    pub deduplicated: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InputAdmissionError {
    Canonical(CanonicalError),
    Contract(InputContractError),
    PrincipalUnauthenticated,
    SourceUnbound,
    ResourceLimit,
}

impl InputAdmissionError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Canonical(_) | Self::Contract(_) => "INPUT_SAMPLE_INVALID",
            Self::PrincipalUnauthenticated => "INPUT_PRINCIPAL_UNAUTHENTICATED",
            Self::SourceUnbound => "INPUT_SOURCE_UNBOUND",
            Self::ResourceLimit => "INPUT_RESOURCE_LIMIT",
        }
    }
}

impl Display for InputAdmissionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for InputAdmissionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TickReport {
    pub tick: u64,
    pub results: Vec<CommandResult>,
    pub events: Vec<DomainEvent>,
    pub stage_trace: Vec<StageTraceEntry>,
    pub snapshot: RuntimeSnapshot,
    pub rpg_snapshot: RpgSnapshot,
    pub physics_snapshot: PhysicsCanonicalSnapshotV2,
    pub physics_step_input: PhysicsStepInputV2,
    pub contact_batch: ClosedPhysicsContactBatchV1,
    pub physics_checkpoint_hash: ContentHash,
    pub closed_ingress_batch: ClosedIngressBatchV1,
    pub mapping_receipts: Vec<InputMappingReceiptV1>,
    pub command_batches: Vec<ClosedCommandAdmissionBatchV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OrderedResult {
    order_key: CommandOrderKey,
    result: CommandResult,
}

impl OrderedResult {
    fn with_disposition(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
        disposition: CommandDisposition,
    ) -> Self {
        Self {
            order_key,
            result: CommandResult {
                command_id,
                sequence,
                disposition,
            },
        }
    }

    fn reserved(order_key: CommandOrderKey, command_id: CommandId, sequence: u64) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Reserved,
        )
    }

    fn committed(order_key: CommandOrderKey, command_id: CommandId, sequence: u64) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Committed,
        )
    }

    fn deduplicated(order_key: CommandOrderKey, command_id: CommandId, sequence: u64) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Deduplicated,
        )
    }

    fn rejected(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
        code: RejectionCode,
    ) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Rejected(code),
        )
    }
}

impl Ord for OrderedResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order_key.cmp(&other.order_key)
    }
}

impl PartialOrd for OrderedResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeFatalError {
    TickExhausted,
    EventCountExhausted,
    RevisionExhausted,
    TraceCountExhausted,
    IngressGenerationExhausted,
    IngressCheckpointCorrupt,
    Input(InputContractError),
    OutcomeCollection(OutcomeCollectionError),
    InternalCanonicalization(CanonicalError),
    LedgerCorrupt(CommandLedgerError),
    Physics(ReferencePhysicsError),
    PhysicalOutcomeInvariant,
    InternalIdentityCollision,
    Snapshot(SnapshotDecodeError),
}

impl RuntimeFatalError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::TickExhausted => "SIMULATION_TICK_EXHAUSTED",
            Self::EventCountExhausted => "DOMAIN_EVENT_COUNT_EXHAUSTED",
            Self::RevisionExhausted => "AUTHORITATIVE_REVISION_EXHAUSTED",
            Self::TraceCountExhausted => "STAGE_TRACE_COUNT_EXHAUSTED",
            Self::IngressGenerationExhausted => "INGRESS_QUEUE_GENERATION_EXHAUSTED",
            Self::IngressCheckpointCorrupt => "INGRESS_CHECKPOINT_CORRUPT",
            Self::Input(_) => "INGRESS_CONTRACT_CORRUPT",
            Self::OutcomeCollection(error) => error.stable_code(),
            Self::InternalCanonicalization(_) => "INTERNAL_CANONICALIZATION_FAILED",
            Self::LedgerCorrupt(_) => "COMMAND_LEDGER_CORRUPT",
            Self::Physics(error) => error.stable_code(),
            Self::PhysicalOutcomeInvariant => "PHYSICAL_OUTCOME_INVARIANT_FAILED",
            Self::InternalIdentityCollision => "INTERNAL_IDENTITY_COLLISION",
            Self::Snapshot(_) => "RUNTIME_SNAPSHOT_CLOSURE_CORRUPT",
        }
    }
}

impl Display for RuntimeFatalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.stable_code())
    }
}

impl Error for RuntimeFatalError {}

impl From<CanonicalError> for RuntimeFatalError {
    fn from(error: CanonicalError) -> Self {
        Self::InternalCanonicalization(error)
    }
}

impl From<CommandLedgerError> for RuntimeFatalError {
    fn from(error: CommandLedgerError) -> Self {
        Self::LedgerCorrupt(error)
    }
}

impl From<InputContractError> for RuntimeFatalError {
    fn from(error: InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<ReferencePhysicsError> for RuntimeFatalError {
    fn from(error: ReferencePhysicsError) -> Self {
        Self::Physics(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SnapshotRestoreError {
    Canonicalization(CanonicalError),
    Decode(SnapshotDecodeError),
    Identity(IdentityContractError),
    Input(InputContractError),
    Physics(PhysicsContractError),
    PhysicsController(ReferencePhysicsError),
    WorldCheckpoint(WorldCheckpointError),
    Ledger(CommandLedgerError),
    RpgDecode(RpgDecodeError),
    RpgState(RpgStateError),
    BootstrapClosureMismatch,
    CommandRegistryMismatch,
    InactivePrincipal,
    IdentityCollision,
    ControllerClosureMismatch,
}

impl Display for SnapshotRestoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonicalization(error) => write!(formatter, "canonicalization failed: {error}"),
            Self::Decode(error) => write!(formatter, "snapshot validation failed: {error}"),
            Self::Identity(error) => write!(formatter, "identity validation failed: {error}"),
            Self::Input(error) => write!(formatter, "input contract validation failed: {error}"),
            Self::Physics(error) => {
                write!(formatter, "physical contract validation failed: {error}")
            }
            Self::PhysicsController(error) => {
                write!(formatter, "physical controller validation failed: {error}")
            }
            Self::WorldCheckpoint(error) => {
                write!(formatter, "world checkpoint validation failed: {error}")
            }
            Self::Ledger(error) => write!(formatter, "ledger validation failed: {error}"),
            Self::RpgDecode(error) => write!(formatter, "RPG snapshot decode failed: {error}"),
            Self::RpgState(error) => write!(formatter, "RPG snapshot state failed: {error}"),
            Self::BootstrapClosureMismatch => {
                formatter.write_str("runtime bootstrap world/profile closure does not match")
            }
            Self::CommandRegistryMismatch => {
                formatter.write_str("runtime command registry hash does not match")
            }
            Self::InactivePrincipal => {
                formatter.write_str("runtime bootstrap authority or stream principal is inactive")
            }
            Self::IdentityCollision => formatter.write_str("runtime bootstrap identity collision"),
            Self::ControllerClosureMismatch => {
                formatter.write_str("runtime controller registry closure does not match")
            }
        }
    }
}

impl Error for SnapshotRestoreError {}

impl From<CanonicalError> for SnapshotRestoreError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<SnapshotDecodeError> for SnapshotRestoreError {
    fn from(error: SnapshotDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentityContractError> for SnapshotRestoreError {
    fn from(error: IdentityContractError) -> Self {
        Self::Identity(error)
    }
}

impl From<InputContractError> for SnapshotRestoreError {
    fn from(error: InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<PhysicsContractError> for SnapshotRestoreError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<ReferencePhysicsError> for SnapshotRestoreError {
    fn from(error: ReferencePhysicsError) -> Self {
        Self::PhysicsController(error)
    }
}

impl From<WorldCheckpointError> for SnapshotRestoreError {
    fn from(error: WorldCheckpointError) -> Self {
        Self::WorldCheckpoint(error)
    }
}

impl From<CommandLedgerError> for SnapshotRestoreError {
    fn from(error: CommandLedgerError) -> Self {
        Self::Ledger(error)
    }
}

impl From<RpgDecodeError> for SnapshotRestoreError {
    fn from(error: RpgDecodeError) -> Self {
        Self::RpgDecode(error)
    }
}

impl From<RpgStateError> for SnapshotRestoreError {
    fn from(error: RpgStateError) -> Self {
        Self::RpgState(error)
    }
}

#[cfg(test)]
mod tests {
    use next_contracts::{
        CORE_MOVE_ACTION_ID, CapabilityId, CommandStreamKeyV1, InputSourceId,
        NOOP_COMMAND_CAPABILITY_ID, PHYSICAL_COMMAND_CAPABILITY_ID, PLAYER_ACTION_FRAME_SCHEMA_ID,
        PLAYER_ACTION_SOURCE_CLASS, PersistentId, PhysicsBodyDescriptorV1, PhysicsBodyIdV1,
        PhysicsContactReportingV1, PhysicsGeometryV1, PhysicsMaterialDescriptorV1,
        PhysicsMotionKindV1, PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeDescriptorV1,
        PhysicsShapeIdV1, PlayerActionV1, PlayerPrincipalId, PrincipalRecordV1, PrincipalStatus,
        SchemaId, SystemId, WorldNamespaceId,
    };

    use super::*;

    struct Fixture {
        runtime: RuntimeState,
        principal: IssuerPrincipal,
        stream_id: CommandStreamId,
    }

    fn fixture() -> Fixture {
        fixture_for(
            "nextengine.runtime-fixture",
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([3; 16])),
        )
    }

    fn fixture_for(project_id: &str, principal: IssuerPrincipal) -> Fixture {
        let registry_hash = CommandKindRegistry::core_v1().canonical_hash();
        let profile = RuntimeDeterminismProfileV1::bootstrap_default(registry_hash);
        let world = WorldIdentityManifestV1::new(
            ProjectId::new(project_id).expect("project"),
            [1; 32],
            [2; 32],
            profile.profile_hash().expect("profile hash"),
        )
        .expect("world");
        let mut principals = PrincipalRegistryV1::empty(world.world_namespace);
        principals
            .register(
                principal.clone(),
                PrincipalRecordV1 {
                    provenance_hash: content_hash_from_bytes([4; 32]),
                    capability_subject_id: SchemaId::new("fixture.player").expect("subject"),
                    status: PrincipalStatus::Active,
                },
            )
            .expect("principal");
        let mut streams = CommandStreamRegistryV1::empty(world.world_namespace);
        let stream_id = streams.allocate_stream(principal.clone()).expect("stream");
        let mut authority = AuthorityRegistry::new();
        authority
            .register(
                principal.clone(),
                [CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("capability")],
            )
            .expect("authority");
        let runtime = RuntimeState::new(
            RuntimeBootstrapV3::new(world, principals, streams, profile),
            authority,
        )
        .expect("runtime");
        Fixture {
            runtime,
            principal,
            stream_id,
        }
    }

    fn command(fixture: &Fixture, sequence: u64, target_tick: u64) -> WorldCommand {
        WorldCommand::noop(
            fixture.stream_id,
            fixture.principal.clone(),
            sequence,
            target_tick,
        )
        .expect("command")
    }

    struct PhysicalFixture {
        runtime: RuntimeState,
        principal: IssuerPrincipal,
        source_id: InputSourceId,
        controller_id: PersistentId,
        physics_body_id: PhysicsBodyIdV1,
        action_map_hash: ContentHash,
        context_stack_hash: ContentHash,
    }

    fn physical_fixture() -> PhysicalFixture {
        let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([13; 16]));
        let registry_hash = CommandKindRegistry::core_v1().canonical_hash();
        let profile = RuntimeDeterminismProfileV1::bootstrap_default(registry_hash);
        let world = WorldIdentityManifestV1::new(
            ProjectId::new("nextengine.runtime-physical-fixture").expect("project"),
            [14; 32],
            [15; 32],
            profile.profile_hash().expect("profile hash"),
        )
        .expect("world");
        let mut principals = PrincipalRegistryV1::empty(world.world_namespace);
        principals
            .register(
                principal.clone(),
                PrincipalRecordV1 {
                    provenance_hash: content_hash_from_bytes([16; 32]),
                    capability_subject_id: SchemaId::new("fixture.physical-player")
                        .expect("subject"),
                    status: PrincipalStatus::Active,
                },
            )
            .expect("principal");
        let mut streams = CommandStreamRegistryV1::empty(world.world_namespace);
        let stream_id = streams.allocate_stream(principal.clone()).expect("stream");
        let mut authority = AuthorityRegistry::new();
        authority
            .register(
                principal.clone(),
                [CapabilityId::new(PHYSICAL_COMMAND_CAPABILITY_ID).expect("capability")],
            )
            .expect("authority");

        let source_id = InputSourceId::from_bytes([17; 16]);
        let controller_id = PersistentId::from_bytes([18; 16]);
        let body_id = PersistentId::from_bytes([19; 16]);
        let action_map_hash = content_hash_from_bytes([20; 32]);
        let context_stack_hash = content_hash_from_bytes([21; 32]);
        let mut bootstrap = RuntimeBootstrapV3::new(world, principals, streams, profile);
        let binding = PlayerControllerBindingV1 {
            principal: principal.clone(),
            source_id,
            controller_id,
            controlled_body_id: body_id,
            command_stream_id: stream_id,
            action_map_hash,
            action_map_revision: 1,
            context_stack_hash,
            context_stack_revision: 1,
        };
        bootstrap
            .player_controller_registry
            .bindings
            .insert(source_id, binding);
        let physics_body_id = PhysicsBodyIdV1 {
            subject_id: body_id,
            body_slot: 0,
        };
        bootstrap.physics_checkpoint = grounded_test_checkpoint(
            PhysicsWorldId::from_bytes(*bootstrap.world_identity.world_namespace.as_bytes()),
            physics_body_id,
            &bootstrap.tick_rate_profile,
            &bootstrap.authoritative_numeric_profile,
            &bootstrap.physics_quantization_profile,
        );
        let runtime = RuntimeState::new(bootstrap, authority).expect("physical runtime");
        PhysicalFixture {
            runtime,
            principal,
            source_id,
            controller_id,
            physics_body_id,
            action_map_hash,
            context_stack_hash,
        }
    }

    fn grounded_test_checkpoint(
        world_id: PhysicsWorldId,
        capsule_body_id: PhysicsBodyIdV1,
        tick_rate: &TickRateProfileV1,
        numeric: &AuthoritativeNumericProfileV1,
        quantization: &PhysicsQuantizationProfileV1,
    ) -> PhysicsWorldCheckpointV1 {
        let material_id = SchemaId::new("nextengine.physics.material.reference-zero").expect("id");
        let material = PhysicsMaterialDescriptorV1 {
            material_id: material_id.clone(),
            descriptor_revision: 1,
            static_friction_q16: 0,
            dynamic_friction_q16: 0,
            restitution_q16: 0,
            canonical_material_tags: Vec::new(),
        };
        let capsule_shape_id = PhysicsShapeIdV1 {
            body_id: capsule_body_id,
            shape_slot: 0,
        };
        let capsule_shape = PhysicsShapeDescriptorV1 {
            shape_id: capsule_shape_id,
            descriptor_revision: 1,
            local_pose: PhysicsPoseV1::default(),
            geometry: PhysicsGeometryV1::Capsule {
                radius_micrometres: 300_000,
                half_segment_micrometres: 600_000,
            },
            material_id: material_id.clone(),
            collision_layer: 0,
            collision_mask: 1,
            participation: PhysicsParticipationV1::Solid,
            contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
        };
        let capsule_pose = PhysicsPoseV1 {
            translation_micrometres: [0, 900_000, 0],
            ..PhysicsPoseV1::default()
        };
        let capsule = PhysicsBodyDescriptorV1 {
            body_id: capsule_body_id,
            descriptor_revision: 1,
            motion_kind: PhysicsMotionKindV1::Kinematic,
            initial_pose: capsule_pose,
            initial_linear_velocity_micrometres_per_second: [0; 3],
            initial_angular_velocity_q16: [0; 3],
            active: true,
            shapes: BTreeMap::from([(capsule_shape_id, capsule_shape)]),
        };
        let floor_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([22; 16]),
            body_slot: 0,
        };
        let floor_shape_id = PhysicsShapeIdV1 {
            body_id: floor_body_id,
            shape_slot: 0,
        };
        let floor_pose = PhysicsPoseV1 {
            translation_micrometres: [0, -100_000, 0],
            ..PhysicsPoseV1::default()
        };
        let floor_shape = PhysicsShapeDescriptorV1 {
            shape_id: floor_shape_id,
            descriptor_revision: 1,
            local_pose: PhysicsPoseV1::default(),
            geometry: PhysicsGeometryV1::Box {
                half_extents_micrometres: [10_000_000, 100_000, 10_000_000],
            },
            material_id: material_id.clone(),
            collision_layer: 0,
            collision_mask: 1,
            participation: PhysicsParticipationV1::Solid,
            contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
        };
        let floor = PhysicsBodyDescriptorV1 {
            body_id: floor_body_id,
            descriptor_revision: 1,
            motion_kind: PhysicsMotionKindV1::Static,
            initial_pose: floor_pose,
            initial_linear_velocity_micrometres_per_second: [0; 3],
            initial_angular_velocity_q16: [0; 3],
            active: true,
            shapes: BTreeMap::from([(floor_shape_id, floor_shape)]),
        };
        let catalog = PhysicsWorldCatalogV1::new(
            world_id,
            PhysicsWorldCatalogProfilesV1 {
                coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinate"),
                limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
                solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
                tick_rate_hash: tick_rate.profile_hash().expect("tick hash"),
                authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
                quantization_hash: quantization.profile_hash().expect("quantization hash"),
            },
            BTreeMap::from([(material_id, material)]),
            BTreeMap::from([(capsule_body_id, capsule), (floor_body_id, floor)]),
            BTreeMap::from([(capsule_body_id.subject_id, capsule_body_id)]),
        )
        .expect("catalog");
        let snapshot =
            PhysicsCanonicalSnapshotV2::genesis(&catalog, tick_rate, numeric, quantization)
                .expect("snapshot");
        PhysicsWorldCheckpointV1::new(catalog, snapshot).expect("checkpoint")
    }

    fn movement_sample(
        fixture: &PhysicalFixture,
        sequence: u64,
        phase: PlayerActionPhaseV1,
        direction_q15: [i16; 2],
        wall_time: Option<i64>,
    ) -> InputSampleV1 {
        let frame = PlayerActionFrameV1 {
            schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
            controller_id: fixture.controller_id,
            logical_frame_sequence: sequence,
            action_map_hash: fixture.action_map_hash,
            action_map_revision: 1,
            context_stack_hash: fixture.context_stack_hash,
            context_stack_revision: 1,
            actions: vec![PlayerActionV1 {
                action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("action"),
                phase,
                value: PlayerActionValueV1::Vector2Q15(direction_q15),
                semantic_occurrence_ordinal: 0,
            }],
        };
        InputSampleV1 {
            schema_version: 1,
            source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
            source_id: fixture.source_id,
            source_sequence: sequence,
            payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("schema"),
            payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
            payload: frame.canonical_bytes().expect("frame"),
            sampled_wall_time: wall_time,
        }
    }

    #[test]
    fn action_frames_move_capsule_exactly_and_wall_time_is_nonauthoritative() {
        let mut fixture = physical_fixture();
        for sequence in 0..3 {
            let sample = movement_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Performed,
                [0, 32_767],
                Some(i64::try_from(sequence).expect("small") * 999),
            );
            fixture
                .runtime
                .enqueue_input_sample(&fixture.principal, sample)
                .expect("enqueue");
            let report = fixture.runtime.run_tick([]).expect("movement tick");
            assert_eq!(report.results[0].disposition, CommandDisposition::Committed);
            assert_eq!(report.events.len(), 1);
            assert_eq!(
                report.mapping_receipts[0].code,
                InputMappingCodeV1::Accepted
            );
        }
        let body = &fixture.runtime.physics_snapshot().sorted_body_states[&fixture.physics_body_id];
        assert_eq!(body.pose.translation_micrometres, [0, 900_000, 300_000]);
        assert_eq!(fixture.runtime.physics_snapshot().physics_tick, 6);
    }

    #[test]
    fn invalid_and_colliding_input_never_reaches_command_ledger() {
        let mut fixture = physical_fixture();
        let diagonal = movement_sample(
            &fixture,
            0,
            PlayerActionPhaseV1::Performed,
            [32_767, 32_767],
            None,
        );
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, diagonal)
            .expect("enqueue diagonal");
        let invalid = fixture.runtime.run_tick([]).expect("mapping rejection");
        assert!(invalid.results.is_empty());
        assert!(invalid.events.is_empty());
        assert_eq!(
            invalid.mapping_receipts[0].code,
            InputMappingCodeV1::ValueOutOfProfile
        );
        assert_eq!(
            invalid
                .snapshot
                .command_ledger
                .streams
                .values()
                .next()
                .expect("stream")
                .finalized_receipt_count,
            0
        );

        let first = movement_sample(
            &fixture,
            1,
            PlayerActionPhaseV1::Performed,
            [0, 32_767],
            None,
        );
        let second = movement_sample(
            &fixture,
            1,
            PlayerActionPhaseV1::Performed,
            [0, -32_767],
            None,
        );
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, first)
            .expect("first");
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, second)
            .expect("second");
        let collision = fixture.runtime.run_tick([]).expect("input collision");
        assert!(collision.results.is_empty());
        assert_eq!(
            collision
                .closed_ingress_batch
                .body
                .equivalence_receipts
                .len(),
            1
        );
        assert_eq!(
            collision
                .snapshot
                .command_ledger
                .streams
                .values()
                .next()
                .expect("stream")
                .finalized_receipt_count,
            0
        );
    }

    #[test]
    fn exact_duplicate_input_closes_and_moves_once() {
        let mut fixture = physical_fixture();
        let sample = movement_sample(
            &fixture,
            0,
            PlayerActionPhaseV1::Performed,
            [0, 32_767],
            Some(1),
        );
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, sample.clone())
            .expect("first duplicate");
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, sample)
            .expect("second duplicate");
        let report = fixture.runtime.run_tick([]).expect("tick");
        assert_eq!(report.closed_ingress_batch.body.input_samples.len(), 1);
        assert_eq!(
            report.stage_trace[0],
            StageTraceEntry {
                stage: TransactionStage::IngressClose,
                received: 1,
                accepted: 1,
                rejected: 0,
                committed: 1,
                deduplicated: 1,
            }
        );
        assert_eq!(
            report.physics_snapshot.sorted_body_states[&fixture.physics_body_id]
                .pose
                .translation_micrometres,
            [0, 900_000, 100_000]
        );
    }

    #[test]
    fn input_arrival_permutations_close_to_identical_batches_and_state() {
        let mut left = physical_fixture();
        let mut right = physical_fixture();
        let first = movement_sample(
            &left,
            0,
            PlayerActionPhaseV1::Performed,
            [32_767, 0],
            Some(1),
        );
        let second = movement_sample(
            &left,
            1,
            PlayerActionPhaseV1::Performed,
            [0, 32_767],
            Some(2),
        );
        for sample in [first.clone(), second.clone()] {
            left.runtime
                .enqueue_input_sample(&left.principal, sample)
                .expect("left enqueue");
        }
        for sample in [second, first] {
            right
                .runtime
                .enqueue_input_sample(&right.principal, sample)
                .expect("right enqueue");
        }
        let left = left.runtime.run_tick([]).expect("left tick");
        let right = right.runtime.run_tick([]).expect("right tick");
        assert_eq!(left.closed_ingress_batch, right.closed_ingress_batch);
        assert_eq!(left.command_batches, right.command_batches);
        assert_eq!(left.results, right.results);
        assert_eq!(left.events, right.events);
        assert_eq!(left.physics_snapshot, right.physics_snapshot);
        assert_eq!(left.snapshot.command_ledger, right.snapshot.command_ledger);
    }

    #[test]
    fn ingress_generation_exhaustion_preserves_queued_input_and_world_state() {
        let mut fixture = physical_fixture();
        fixture.runtime.ingress_checkpoint.current_generation = u64::MAX;
        let sample = movement_sample(
            &fixture,
            0,
            PlayerActionPhaseV1::Performed,
            [0, 32_767],
            None,
        );
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, sample)
            .expect("enqueue");
        let before = fixture.runtime.clone();
        let error = fixture.runtime.run_tick([]).expect_err("exhaustion");
        assert_eq!(error.stable_code(), "INGRESS_QUEUE_GENERATION_EXHAUSTED");
        assert_eq!(fixture.runtime.snapshot(), before.snapshot());
        assert_eq!(
            fixture.runtime.physics_snapshot(),
            before.physics_snapshot()
        );
    }

    #[test]
    fn physical_retry_after_world_checkpoint_restore_never_moves_twice() {
        let mut fixture = physical_fixture();
        let sample = movement_sample(
            &fixture,
            0,
            PlayerActionPhaseV1::Performed,
            [0, 32_767],
            None,
        );
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, sample)
            .expect("enqueue");
        let first = fixture.runtime.run_tick([]).expect("first movement");
        let command = first.command_batches[0].body.envelopes[0].clone();
        let pose = first.physics_snapshot.sorted_body_states[&fixture.physics_body_id].pose;
        let receipt_count =
            first.snapshot.command_ledger.streams[&command.stream_id].finalized_receipt_count;
        let checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
        let authority = fixture.runtime.authority.clone();
        let mut restored =
            RuntimeState::restore_world_checkpoint(checkpoint, authority).expect("restore");
        let retry = restored.run_tick([command.clone()]).expect("retry");
        assert_eq!(
            retry
                .results
                .iter()
                .find(|result| result.command_id == command.claimed_command_id.expect("claim"))
                .expect("retry result")
                .disposition,
            CommandDisposition::Deduplicated
        );
        assert!(retry.events.is_empty());
        assert_eq!(
            retry.physics_snapshot.sorted_body_states[&fixture.physics_body_id].pose,
            pose
        );
        assert_eq!(
            retry.snapshot.command_ledger.streams[&command.stream_id].finalized_receipt_count,
            receipt_count
        );
    }

    #[test]
    fn commit_and_retry_are_exact_without_duplicate_event_or_receipt() {
        let mut fixture = fixture();
        let command = command(&fixture, 0, 0);
        let first = fixture.runtime.run_tick([command.clone()]).expect("commit");
        let receipt_count =
            first.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count;
        let retry = fixture.runtime.run_tick([command]).expect("retry");
        assert_eq!(
            retry.results[0].disposition,
            CommandDisposition::Deduplicated
        );
        assert!(retry.events.is_empty());
        assert_eq!(
            retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
            receipt_count
        );
    }

    #[test]
    fn domain_rejection_consumes_sequence_and_retry_returns_same_code() {
        let mut fixture = fixture();
        let mut rejected = command(&fixture, 0, 0);
        rejected
            .set_precondition_revision(Some(99))
            .expect("precondition");
        let first = fixture
            .runtime
            .run_tick([rejected.clone()])
            .expect("rejection");
        assert_eq!(
            first.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::PreconditionFailed)
        );
        let retry = fixture.runtime.run_tick([rejected]).expect("retry");
        assert_eq!(
            retry.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::PreconditionFailed)
        );
        assert_eq!(
            retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
            1
        );
    }

    #[test]
    fn phase_and_target_rejections_are_terminal_receipts() {
        let mut fixture = fixture();
        let mut wrong_phase = command(&fixture, 0, 0);
        wrong_phase.phase = CommandPhase::Outcome;
        wrong_phase.refresh_command_id().expect("changed phase");
        let mut forbidden_target = command(&fixture, 1, 0);
        forbidden_target.target = Some(PersistentId::from_bytes([8; 16]));
        forbidden_target
            .refresh_command_id()
            .expect("changed target");

        let report = fixture
            .runtime
            .run_tick([wrong_phase.clone(), forbidden_target.clone()])
            .expect("terminal validation rejections");
        assert_eq!(
            report
                .results
                .iter()
                .find(|result| result.sequence == 0)
                .expect("phase result")
                .disposition,
            CommandDisposition::Rejected(RejectionCode::CommandPhaseForbidden)
        );
        assert_eq!(
            report
                .results
                .iter()
                .find(|result| result.sequence == 1)
                .expect("target result")
                .disposition,
            CommandDisposition::Rejected(RejectionCode::TargetNotAllowed)
        );
        assert_eq!(
            report.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
            2
        );
        assert_eq!(
            fixture
                .runtime
                .run_tick([wrong_phase])
                .expect("phase retry")
                .results[0]
                .disposition,
            CommandDisposition::Rejected(RejectionCode::CommandPhaseForbidden)
        );
        assert_eq!(
            fixture
                .runtime
                .run_tick([forbidden_target])
                .expect("target retry")
                .results[0]
                .disposition,
            CommandDisposition::Rejected(RejectionCode::TargetNotAllowed)
        );
    }

    #[test]
    fn future_reservation_survives_restore_and_executes_once_when_due() {
        let mut fixture = fixture();
        let future = command(&fixture, 0, 2);
        let reserved = fixture.runtime.run_tick([future.clone()]).expect("reserve");
        assert_eq!(
            reserved.results[0].disposition,
            CommandDisposition::Reserved
        );
        let checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
        let authority = fixture.runtime.authority.clone();
        let mut restored =
            RuntimeState::restore_world_checkpoint(checkpoint, authority).expect("restore");
        assert!(restored.run_tick([]).expect("tick one").events.is_empty());
        let due = restored.run_tick([]).expect("tick two");
        assert_eq!(due.results[0].disposition, CommandDisposition::Committed);
        assert_eq!(due.events.len(), 1);
        let retry = restored.run_tick([future]).expect("retained retry");
        assert_eq!(
            retry.results[0].disposition,
            CommandDisposition::Deduplicated
        );
    }

    #[test]
    fn future_horizon_expiry_and_stream_time_regression_are_terminal() {
        let mut fixture = fixture();
        let beyond_horizon = command(&fixture, 0, 121);
        let future_limit = fixture
            .runtime
            .run_tick([beyond_horizon])
            .expect("future limit");
        assert_eq!(
            future_limit.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandFutureLimit)
        );

        let expired = command(&fixture, 1, 0);
        let expired_result = fixture.runtime.run_tick([expired]).expect("expired");
        assert_eq!(
            expired_result.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandExpired)
        );

        let first_future = command(&fixture, 2, 100);
        assert_eq!(
            fixture
                .runtime
                .run_tick([first_future])
                .expect("future reservation")
                .results[0]
                .disposition,
            CommandDisposition::Reserved
        );
        let regressing = command(&fixture, 3, 99);
        let regression = fixture
            .runtime
            .run_tick([regressing])
            .expect("stable time regression");
        assert_eq!(
            regression.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandStreamTimeRegression)
        );
        let stream = &regression.snapshot.command_ledger.streams[&fixture.stream_id];
        assert_eq!(stream.pending.len(), 1);
        assert_eq!(stream.finalized_receipt_count, 3);
    }

    #[test]
    fn external_collision_locks_stream_and_survives_restore() {
        let mut fixture = fixture();
        let first = command(&fixture, 0, 0);
        let mut second = first.clone();
        second.target_tick = 1;
        second.refresh_command_id().expect("changed body");
        let report = fixture
            .runtime
            .run_tick([first, second.clone()])
            .expect("collision");
        assert!(report.events.is_empty());
        assert_eq!(
            report.snapshot.command_ledger.streams[&fixture.stream_id].state,
            CommandStreamStateV1::CollisionLocked
        );
        assert!(report.results.iter().all(|result| {
            result.disposition
                == CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
        }));
        let receipt_count =
            report.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count;
        let authority = fixture.runtime.authority.clone();
        let checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
        let mut restored =
            RuntimeState::restore_world_checkpoint(checkpoint, authority).expect("restore");
        assert_eq!(
            restored.command_ledger.streams[&fixture.stream_id].state,
            CommandStreamStateV1::CollisionLocked
        );
        let retry = restored.run_tick([second]).expect("collision retry");
        assert_eq!(
            retry.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
        );
        assert_eq!(
            retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
            receipt_count
        );
    }

    #[test]
    fn collision_against_retained_receipt_locks_external_stream() {
        let mut fixture = fixture();
        let original = command(&fixture, 0, 0);
        fixture
            .runtime
            .run_tick([original.clone()])
            .expect("original commits");
        let mut conflicting = original;
        conflicting.target_tick = 1;
        conflicting.refresh_command_id().expect("changed body");
        let report = fixture
            .runtime
            .run_tick([conflicting])
            .expect("retained collision is stable");
        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
        );
        assert_eq!(
            report.snapshot.command_ledger.streams[&fixture.stream_id].state,
            CommandStreamStateV1::CollisionLocked
        );
        assert_eq!(
            report.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
            1
        );
    }

    #[test]
    fn internal_collision_rolls_back_the_entire_tick() {
        let mut fixture = fixture_for(
            "nextengine.runtime-internal-fixture",
            IssuerPrincipal::InternalSystem(
                SystemId::new("nextengine.system.fixture").expect("system ID"),
            ),
        );
        let before = fixture.runtime.snapshot();
        let first = command(&fixture, 0, 0);
        let mut second = first.clone();
        second.target_tick = 1;
        second.refresh_command_id().expect("changed body");
        let error = fixture
            .runtime
            .run_tick([first, second])
            .expect_err("internal collision is fatal");
        assert_eq!(error, RuntimeFatalError::InternalIdentityCollision);
        assert_eq!(fixture.runtime.snapshot(), before);
    }

    #[test]
    fn event_identity_collision_rolls_back_domain_and_ledger_staging() {
        let mut fixture = fixture();
        let command = command(&fixture, 0, 0);
        let command_id = command.compute_command_id().expect("command ID");
        let event =
            DomainEvent::command_committed(0, CommandPhase::Ingress, command_id, command.sequence)
                .expect("expected event");
        fixture
            .runtime
            .command_ledger
            .causal_identity_registry
            .compare_or_insert(
                CausalIdentityKey {
                    identity_kind: CausalIdentityKind::DomainEvent,
                    identity_bytes: *event.event_id.as_bytes(),
                },
                content_hash_from_bytes([9; 32]),
            )
            .expect("inject conflicting provenance");
        let before = fixture.runtime.snapshot();

        let error = fixture
            .runtime
            .run_tick([command])
            .expect_err("event identity collision is fatal");
        assert_eq!(error, RuntimeFatalError::InternalIdentityCollision);
        assert_eq!(fixture.runtime.snapshot(), before);
        assert_eq!(fixture.runtime.authoritative_revision(), 0);
        assert!(fixture.runtime.rpg_snapshot().items.is_empty());
    }

    #[test]
    fn pending_capacity_rejects_257th_without_archive_or_receipt_insert() {
        let mut fixture = fixture();
        let commands: Vec<_> = (0..=next_contracts::COMMAND_PENDING_CAPACITY)
            .map(|sequence| command(&fixture, sequence as u64, 120))
            .collect();
        let report = fixture
            .runtime
            .run_tick(commands)
            .expect("bounded admission");
        let stream = &report.snapshot.command_ledger.streams[&fixture.stream_id];
        assert_eq!(
            stream.pending.len(),
            next_contracts::COMMAND_PENDING_CAPACITY
        );
        assert_eq!(report.snapshot.body_archive.entries().len(), 256);
        assert_eq!(stream.finalized_receipt_count, 0);
        assert_eq!(
            report.results.last().expect("257th result").disposition,
            CommandDisposition::Rejected(RejectionCode::CommandPendingLimit)
        );
    }

    #[test]
    fn unretained_sequence_below_high_watermark_is_finalized_without_archive_insert() {
        let mut fixture = fixture();
        let report = fixture
            .runtime
            .run_tick([command(&fixture, 1, 0)])
            .expect("sequence gap commits");
        let stream = &report.snapshot.command_ledger.streams[&fixture.stream_id];
        assert_eq!(stream.admission_high_watermark, Some(1));
        assert_eq!(stream.receipt_window[0].subject.sequence(), 1);
        let archive_count = report.snapshot.body_archive.entries().len();
        let finalized = command(&fixture, 0, 0);
        let result = fixture
            .runtime
            .run_tick([finalized])
            .expect("unretained sequence is stable");
        assert_eq!(
            result.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandSequenceFinalized)
        );
        assert_eq!(result.snapshot.body_archive.entries().len(), archive_count);
        assert_eq!(
            result.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
            1
        );
    }

    #[test]
    fn maximum_sequence_exhausts_stream_but_exact_retry_remains_idempotent() {
        let mut fixture = fixture();
        let maximum = command(&fixture, u64::MAX, 0);
        let committed = fixture
            .runtime
            .run_tick([maximum.clone()])
            .expect("maximum sequence commits");
        assert_eq!(
            committed.snapshot.command_ledger.streams[&fixture.stream_id].state,
            CommandStreamStateV1::Exhausted
        );
        let retry = fixture.runtime.run_tick([maximum]).expect("maximum retry");
        assert_eq!(
            retry.results[0].disposition,
            CommandDisposition::Deduplicated
        );
        assert_eq!(
            retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
            1
        );
    }

    #[test]
    fn invalid_claim_and_unbound_stream_are_preledger_failures() {
        let mut fixture = fixture();
        let before = fixture.runtime.snapshot();
        let mut invalid = command(&fixture, 0, 0);
        invalid.claimed_command_id = Some(CommandId::from_bytes([9; 16]));
        let unbound = WorldCommand::noop(
            CommandStreamId::from_bytes([8; 16]),
            fixture.principal.clone(),
            0,
            0,
        )
        .expect("unbound");
        let report = fixture
            .runtime
            .run_tick([invalid, unbound])
            .expect("stable rejection");
        assert!(report.events.is_empty());
        assert_eq!(report.snapshot.command_ledger, before.command_ledger);
        assert_eq!(report.snapshot.body_archive, before.body_archive);
    }

    #[test]
    fn bootstrap_rejects_arbitrary_stream_id() {
        let mut fixture = fixture();
        let key = fixture
            .runtime
            .stream_registry
            .entries
            .keys()
            .next()
            .cloned()
            .expect("stream key");
        fixture.runtime.stream_registry.entries = BTreeMap::from([(
            CommandStreamKeyV1 {
                principal: key.principal,
                stream_slot: key.stream_slot,
                stream_epoch: key.stream_epoch,
            },
            CommandStreamId::from_bytes([9; 16]),
        )]);
        assert!(fixture.runtime.stream_registry.validate().is_err());
    }

    #[test]
    fn restore_rejects_world_namespace_drift_before_activation() {
        let fixture = fixture();
        let mut checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
        checkpoint
            .runtime_snapshot
            .principal_registry
            .world_namespace = WorldNamespaceId::from_bytes([9; 16]);
        assert!(
            RuntimeState::restore_world_checkpoint(checkpoint, fixture.runtime.authority).is_err()
        );
    }
}
