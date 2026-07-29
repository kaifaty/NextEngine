use std::collections::BTreeMap;

use next_contracts::{
    AuthoritativeNumericProfileV1, CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION,
    CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CanonicalDecodeLimits, CanonicalError, CausalIdentityKey,
    CausalIdentityKind, ClosedCommandAdmissionBatchBodyV2, ClosedCommandAdmissionBatchV2,
    ClosedIngressBatchV1, CommandBodyArchiveV1, CommandLedgerV2, CommandPayload, CommandPhase,
    CommandStreamLedgerV2, CommandStreamRegistryV1, ContentHash, CoreDialogueQuestClosureError,
    IdentityInsertResult, IngressAssignmentProfileV1, IngressCheckpointV1, InputMappingReceiptV1,
    InputSampleV1, IssuerPrincipal, PhysicsCanonicalSnapshotV2, PhysicsMotionKindV1,
    PhysicsQuantizationProfileV1, PhysicsWorldCheckpointV1, PhysicsWorldId,
    PlayerControllerRegistryV1, PrincipalRegistryV1, ProjectId, RpgDefinitionRegistryV1,
    RpgRuntimeBindingsV1, RpgSnapshotV2, RuntimeAdmissionLimitsV1, RuntimeDeterminismProfileV1,
    RuntimeSnapshot, TickRateProfileV1, WorldCheckpointError, WorldCheckpointV4, WorldCommand,
    WorldIdentityManifestV1, content_hash_from_bytes, interaction_definition_hash, sha256,
};
use next_physics_api::{PhysicsBackendKind, PhysicsWorldHost};
use next_rpg::RpgState;

use crate::authority::AuthorityRegistry;
use crate::outcome::{NoOutcomes, OutcomeContext, OutcomeProvider, OutcomeSink};
use crate::registry::CommandKindRegistry;

mod affordance;
mod error;
mod ingress;
mod interaction;
mod physics;
mod pipeline;
mod replay;
mod result;

use affordance::resolve_dialogue_quest_binding_v2;
pub use error::{InputAdmissionError, RuntimeFatalError, SnapshotRestoreError};
use ingress::{accept_closed_ingress, close_ingress};
use interaction::{
    InteractionBuildContext, build_interaction_outcomes, resolve_interaction_outcome_route,
    rpg_physical_contact_facts,
};
pub use physics::PhysicsLaunchOptions;
use physics::{activate_physics, empty_physics_checkpoint};
use pipeline::{PhaseContext, StagedAuthoritativeState, ValidationSource, count, process_phase};
pub use replay::{RuntimeReplayDriver, RuntimeReplayError};
pub use result::{
    CommandDisposition, CommandResult, RejectionCode, StageTraceEntry, TickReport, TransactionStage,
};

fn bootstrap_rpg_bindings(
    world_identity: &WorldIdentityManifestV1,
    runtime_profile: &RuntimeDeterminismProfileV1,
) -> RpgRuntimeBindingsV1 {
    let project_bytes = world_identity
        .canonical_bytes()
        .expect("validated world identity is canonical");
    let budget_bytes = runtime_profile
        .canonical_bytes()
        .expect("validated runtime profile is canonical");
    RpgRuntimeBindingsV1 {
        project_composition_lock_hash: domain_hash(
            b"nextengine.bootstrap-project-composition.v1\0",
            &project_bytes,
        ),
        schema_registry_hash: CommandKindRegistry::core_v1().canonical_hash(),
        budget_policy_hash: domain_hash(b"nextengine.bootstrap-rpg-budget.v1\0", &budget_bytes),
        active_definition_policy_hashes: {
            let mut hashes = vec![
                core_rpg_policy_hash(),
                bootstrap_equipment_slot_policy_hash_v1(),
            ];
            hashes.sort_unstable();
            hashes
        },
    }
}

fn core_rpg_policy_hash() -> ContentHash {
    domain_hash(
        b"nextengine.bootstrap-rpg-policy.v1\0",
        b"dialogue|quest|relationship|interactive-object",
    )
}

#[must_use]
pub fn bootstrap_equipment_slot_policy_hash_v1() -> ContentHash {
    domain_hash(
        b"nextengine.bootstrap-equipment-slot-policy.v1\0",
        CORE_EQUIPMENT_MAIN_HAND_SLOT_ID.as_bytes(),
    )
}

fn domain_hash(domain: &[u8], body: &[u8]) -> ContentHash {
    let mut bytes = Vec::with_capacity(domain.len() + body.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(body);
    content_hash_from_bytes(sha256(&bytes))
}

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
    pub rpg_bindings: RpgRuntimeBindingsV1,
    pub rpg_definitions: RpgDefinitionRegistryV1,
}

impl RuntimeBootstrapV3 {
    pub fn new(
        world_identity: WorldIdentityManifestV1,
        principal_registry: PrincipalRegistryV1,
        stream_registry: CommandStreamRegistryV1,
        runtime_profile: RuntimeDeterminismProfileV1,
    ) -> Self {
        let rpg_bindings = bootstrap_rpg_bindings(&world_identity, &runtime_profile);
        let rpg_definitions =
            RpgDefinitionRegistryV1::empty().expect("empty RPG definition registry is canonical");
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
            rpg_bindings,
            rpg_definitions,
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

    #[must_use]
    pub fn with_rpg_bindings(mut self, rpg_bindings: RpgRuntimeBindingsV1) -> Self {
        self.rpg_bindings = rpg_bindings;
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

#[derive(Debug)]
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
    physics: PhysicsWorldHost,
    last_closed_ingress_batch: Option<ClosedIngressBatchV1>,
    last_mapping_receipts: Vec<InputMappingReceiptV1>,
    last_command_batches: Vec<ClosedCommandAdmissionBatchV2>,
    command_ledger: CommandLedgerV2,
    body_archive: CommandBodyArchiveV1,
    rpg: RpgState,
    rpg_bindings: RpgRuntimeBindingsV1,
    rpg_definitions: RpgDefinitionRegistryV1,
}

impl RuntimeState {
    pub fn new(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::new_with_physics_options(bootstrap, authority, PhysicsLaunchOptions::default())
    }

    pub fn new_with_physics_options(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
        physics_options: PhysicsLaunchOptions,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::from_bootstrap(bootstrap, authority, RpgState::default(), physics_options)
    }

    pub fn with_rpg_snapshot(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
        snapshot: RpgSnapshotV2,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::with_rpg_snapshot_and_physics_options(
            bootstrap,
            authority,
            snapshot,
            PhysicsLaunchOptions::default(),
        )
    }

    pub fn with_rpg_snapshot_and_physics_options(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
        snapshot: RpgSnapshotV2,
        physics_options: PhysicsLaunchOptions,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::from_bootstrap(
            bootstrap,
            authority,
            validate_rpg_snapshot(snapshot)?,
            physics_options,
        )
    }

    fn from_bootstrap(
        bootstrap: RuntimeBootstrapV3,
        authority: AuthorityRegistry,
        rpg: RpgState,
        physics_options: PhysicsLaunchOptions,
    ) -> Result<Self, SnapshotRestoreError> {
        let registry = CommandKindRegistry::core_v1();
        validate_bootstrap(&bootstrap, &authority, &registry)?;
        validate_core_interaction_runtime_closure(&bootstrap, &rpg, &authority)?;
        let physics = activate_physics(
            physics_options,
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
            rpg_bindings: bootstrap.rpg_bindings,
            rpg_definitions: bootstrap.rpg_definitions,
        })
    }

    pub fn restore_world_checkpoint(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::restore_world_checkpoint_with_physics_options(
            checkpoint,
            authority,
            PhysicsLaunchOptions::default(),
        )
    }

    pub fn restore_world_checkpoint_with_physics_options(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
        physics_options: PhysicsLaunchOptions,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::restore_world_checkpoint_with_definitions_and_physics_options(
            checkpoint,
            authority,
            RpgDefinitionRegistryV1::empty().expect("empty RPG definition registry is canonical"),
            physics_options,
        )
    }

    pub fn restore_world_checkpoint_with_definitions(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
        rpg_definitions: RpgDefinitionRegistryV1,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::restore_world_checkpoint_with_definitions_and_physics_options(
            checkpoint,
            authority,
            rpg_definitions,
            PhysicsLaunchOptions::default(),
        )
    }

    pub fn restore_world_checkpoint_with_definitions_and_physics_options(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
        rpg_definitions: RpgDefinitionRegistryV1,
        physics_options: PhysicsLaunchOptions,
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
        Self::restore_from_parts(
            snapshot,
            rpg,
            physics_checkpoint,
            authority,
            rpg_definitions,
            physics_options,
        )
    }

    fn restore_from_parts(
        snapshot: RuntimeSnapshot,
        rpg: RpgState,
        physical: PhysicsWorldCheckpointV1,
        authority: AuthorityRegistry,
        rpg_definitions: RpgDefinitionRegistryV1,
        physics_options: PhysicsLaunchOptions,
    ) -> Result<Self, SnapshotRestoreError> {
        let registry = CommandKindRegistry::core_v1();
        let bootstrap = RuntimeBootstrapV3 {
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
            rpg_bindings: snapshot.rpg_runtime_bindings.clone(),
            rpg_definitions,
        };
        validate_bootstrap(&bootstrap, &authority, &registry)?;
        validate_core_interaction_runtime_closure(&bootstrap, &rpg, &authority)?;
        if snapshot.command_ledger.command_kind_registry_hash != registry.canonical_hash() {
            return Err(SnapshotRestoreError::CommandRegistryMismatch);
        }
        let physics = activate_physics(
            physics_options,
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
            rpg_bindings: bootstrap.rpg_bindings,
            rpg_definitions: bootstrap.rpg_definitions,
        })
    }

    fn fork_from_checkpoint(&self) -> Result<Self, SnapshotRestoreError> {
        Self::restore_from_parts(
            self.snapshot(),
            self.rpg.clone(),
            self.physics.checkpoint().clone(),
            self.authority.clone(),
            self.rpg_definitions.clone(),
            PhysicsLaunchOptions::require(self.physics.backend_kind()),
        )
    }

    pub fn world_checkpoint(&self) -> Result<WorldCheckpointV4, WorldCheckpointError> {
        WorldCheckpointV4::new(
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
            rpg_runtime_bindings: self.rpg_bindings.clone(),
            command_ledger: self.command_ledger.clone(),
            body_archive: self.body_archive.clone(),
        }
    }

    #[must_use]
    pub fn rpg_snapshot(&self) -> RpgSnapshotV2 {
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
    pub fn physics_backend_kind(&self) -> PhysicsBackendKind {
        self.physics.backend_kind()
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
            physics: self
                .physics
                .fork_from_checkpoint(self.physics.checkpoint().clone())?,
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
                rpg_bindings: &self.rpg_bindings,
                controllers: &self.player_controller_registry,
                physical_contact_facts: &[],
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
        let physical_contact_facts = rpg_physical_contact_facts(
            &contact_batch,
            staged.physics.snapshot().checkpoint_revision,
        )?;

        let built_in_outcomes = build_interaction_outcomes(
            &closed_ingress.pending_interactions,
            InteractionBuildContext {
                route: interaction_route.as_ref(),
                physics: &staged.physics,
                rpg: &staged.rpg,
                rpg_definitions: &self.rpg_definitions,
                ledger: &staged.ledger,
                archive: &staged.archive,
                gameplay_tick: tick,
                physical_contact_facts: &physical_contact_facts,
                authoritative_revision: staged.revision,
            },
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
                rpg_bindings: &self.rpg_bindings,
                controllers: &self.player_controller_registry,
                physical_contact_facts: &physical_contact_facts,
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
        let mut rpg_plan_traces = ingress.rpg_plan_traces;
        rpg_plan_traces.extend(outcome.rpg_plan_traces);

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
            rpg_runtime_bindings: self.rpg_bindings.clone(),
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
            rpg_plan_traces,
        })
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
    bootstrap.rpg_bindings.validate()?;
    bootstrap.rpg_definitions.validate()?;
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
    if bootstrap
        .rpg_definitions
        .interactions
        .iter()
        .any(|definition| {
            bootstrap
                .rpg_bindings
                .active_definition_policy_hashes
                .binary_search(&interaction_definition_hash(definition))
                .is_err()
        })
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

fn validate_core_interaction_runtime_closure(
    bootstrap: &RuntimeBootstrapV3,
    rpg: &RpgState,
    authority: &AuthorityRegistry,
) -> Result<(), SnapshotRestoreError> {
    let binding = match bootstrap.player_controller_registry.bindings.len() {
        0 => None,
        1 => {
            let controlled_character_id = bootstrap
                .player_controller_registry
                .bindings
                .values()
                .next()
                .expect("single controller exists")
                .controlled_body_id;
            resolve_dialogue_quest_binding_v2(
                rpg,
                &bootstrap.rpg_definitions,
                controlled_character_id,
            )?
        }
        _ => return Err(CoreDialogueQuestClosureError.into()),
    };
    if let Some(binding) = binding {
        if !bootstrap
            .physics_checkpoint
            .catalog
            .avatar_bindings
            .contains_key(&binding.player_id)
        {
            return Err(CoreDialogueQuestClosureError.into());
        }
        let mut npc_bodies = bootstrap
            .physics_checkpoint
            .catalog
            .bodies
            .values()
            .filter(|body| body.body_id.subject_id == binding.npc_id);
        if npc_bodies
            .next()
            .is_none_or(|body| body.motion_kind != PhysicsMotionKindV1::Static)
            || npc_bodies.next().is_some()
        {
            return Err(CoreDialogueQuestClosureError.into());
        }
    }
    if binding.is_some()
        && resolve_interaction_outcome_route(
            &bootstrap.principal_registry,
            &bootstrap.stream_registry,
            authority,
        )
        .is_none()
    {
        return Err(SnapshotRestoreError::CoreInteractionClosure(
            CoreDialogueQuestClosureError,
        ));
    }
    Ok(())
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

fn validate_rpg_snapshot(snapshot: RpgSnapshotV2) -> Result<RpgState, SnapshotRestoreError> {
    let canonical_bytes = snapshot.canonical_bytes()?;
    let snapshot =
        RpgSnapshotV2::from_canonical_bytes(&canonical_bytes, CanonicalDecodeLimits::default())?;
    Ok(RpgState::from_snapshot(snapshot)?)
}

#[cfg(test)]
mod tests;
