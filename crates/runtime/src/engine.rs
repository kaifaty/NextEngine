use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use next_contracts::{
    AcceptedLocomotionIntentV2, AuthoritativeNumericProfileV1,
    CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION, COMMAND_ENVELOPE_SCHEMA_VERSION,
    COMMAND_RECEIPT_SCHEMA_VERSION, CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CanonicalDecodeLimits,
    CanonicalError, CausalIdentityKey, CausalIdentityKind, ClosedCommandAdmissionBatchBodyV2,
    ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1, ClosedPhysicsContactBatchV1,
    CommandBodyArchiveV1, CommandCollisionCandidateV1, CommandCollisionIncidentV1,
    CommandFinalResultV1, CommandId, CommandIdentityOccurrenceV1, CommandLedgerError,
    CommandLedgerV2, CommandPayload, CommandPhase, CommandReceiptSubjectV1, CommandReceiptV1,
    CommandReservationV1, CommandStreamId, CommandStreamLedgerV2, CommandStreamRegistryV1,
    CommandStreamStateV1, ContentHash, CoreDialogueQuestClosureError, DomainEvent,
    IdentityInsertResult, IngressAssignmentProfileV1, IngressCheckpointV1, InputMappingReceiptV1,
    InputSampleV1, IssuerPrincipal, PHYSICS_STEP_INPUT_SCHEMA_VERSION, PhysicalCommandV1,
    PhysicalEventV1, PhysicsCanonicalSnapshotV2, PhysicsContractError, PhysicsCoordinateProfileV1,
    PhysicsLimitsProfileV1, PhysicsMotionKindV1, PhysicsQuantizationProfileV1,
    PhysicsSolverSemanticsProfileV1, PhysicsStepInputV2, PhysicsWorldCatalogProfilesV1,
    PhysicsWorldCatalogV1, PhysicsWorldCheckpointV1, PhysicsWorldId, PlayerControllerRegistryV1,
    PrincipalRegistryV1, ProjectId, RpgDefinitionRegistryV1, RpgPhysicalContactFactV1,
    RpgRuntimeBindingsV1, RpgSnapshotV2, RpgTransactionPlanV1, RuntimeAdmissionLimitsV1,
    RuntimeDeterminismProfileV1, RuntimeSnapshot, TickRateProfileV1, WorldCheckpointError,
    WorldCheckpointV4, WorldCommand, WorldIdentityManifestV1, content_hash_from_bytes,
    interaction_definition_hash, sha256,
};
use next_physics_api::{
    PhysicsBackendError, PhysicsBackendKind, PhysicsBackendPolicy, PhysicsWorldHost,
    ReferencePhysicsError, ReferencePhysicsFactory,
};
use next_rpg::{
    RpgPlanBuildError, RpgPlanMaterializeError, RpgPlanningContextV1, RpgState,
    build_transaction_plan_v1, materialize_transaction_plan_v1,
};

use crate::authority::AuthorityRegistry;
use crate::outcome::{NoOutcomes, OutcomeContext, OutcomeProvider, OutcomeSink};
use crate::registry::CommandKindRegistry;

mod affordance;
mod error;
mod ingress;
mod interaction;
mod replay;
mod result;

use affordance::resolve_dialogue_quest_binding_v2;
pub use error::{InputAdmissionError, RuntimeFatalError, SnapshotRestoreError};
use ingress::{accept_closed_ingress, close_ingress};
use interaction::{
    InteractionBuildContext, build_interaction_outcomes, resolve_interaction_outcome_route,
    rpg_physical_contact_facts,
};
pub use replay::{RuntimeReplayDriver, RuntimeReplayError};
pub use result::{
    CommandDisposition, CommandResult, CommittedRpgPlanTraceV1, RejectionCode, StageTraceEntry,
    TickReport, TransactionStage,
};
use result::{CommandOrderKey, OrderedResult};

type CollisionKey = (CommandStreamId, IssuerPrincipal, u64);

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicsLaunchOptions {
    pub backend_policy: PhysicsBackendPolicy,
}

impl PhysicsLaunchOptions {
    #[must_use]
    pub const fn new(backend_policy: PhysicsBackendPolicy) -> Self {
        Self { backend_policy }
    }

    const fn require(kind: PhysicsBackendKind) -> Self {
        Self {
            backend_policy: match kind {
                PhysicsBackendKind::Reference => PhysicsBackendPolicy::ReferenceOnly,
                PhysicsBackendKind::PhysX => PhysicsBackendPolicy::RequirePhysX,
            },
        }
    }
}

fn activate_physics(
    options: PhysicsLaunchOptions,
    checkpoint: PhysicsWorldCheckpointV1,
    tick_rate_profile: TickRateProfileV1,
    numeric_profile: AuthoritativeNumericProfileV1,
    quantization_profile: PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldHost, PhysicsBackendError> {
    match options.backend_policy {
        PhysicsBackendPolicy::ReferenceOnly => PhysicsWorldHost::activate(
            &ReferencePhysicsFactory,
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
        ),
        PhysicsBackendPolicy::RequirePhysX => activate_physx(
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
        ),
        PhysicsBackendPolicy::PreferPhysXThenReference => activate_physx(
            checkpoint.clone(),
            tick_rate_profile,
            numeric_profile.clone(),
            quantization_profile.clone(),
        )
        .or_else(|_| {
            PhysicsWorldHost::activate(
                &ReferencePhysicsFactory,
                checkpoint,
                tick_rate_profile,
                numeric_profile,
                quantization_profile,
            )
        }),
    }
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
fn activate_physx(
    checkpoint: PhysicsWorldCheckpointV1,
    tick_rate_profile: TickRateProfileV1,
    numeric_profile: AuthoritativeNumericProfileV1,
    quantization_profile: PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldHost, PhysicsBackendError> {
    PhysicsWorldHost::activate(
        &next_physics_physx::PhysXPhysicsFactory,
        checkpoint,
        tick_rate_profile,
        numeric_profile,
        quantization_profile,
    )
}

#[cfg(not(any(feature = "physx", feature = "physx-mock")))]
fn activate_physx(
    _checkpoint: PhysicsWorldCheckpointV1,
    _tick_rate_profile: TickRateProfileV1,
    _numeric_profile: AuthoritativeNumericProfileV1,
    _quantization_profile: PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldHost, PhysicsBackendError> {
    Err(PhysicsBackendError::from(
        ReferencePhysicsError::BackendUnavailable,
    ))
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
    rpg_plan_traces: Vec<CommittedRpgPlanTraceV1>,
}

#[derive(Clone, Copy)]
struct PhaseContext<'a> {
    registry: &'a CommandKindRegistry,
    authority: &'a AuthorityRegistry,
    principals: &'a PrincipalRegistryV1,
    streams: &'a CommandStreamRegistryV1,
    profile: &'a RuntimeDeterminismProfileV1,
    rpg_bindings: &'a RpgRuntimeBindingsV1,
    controllers: &'a PlayerControllerRegistryV1,
    physical_contact_facts: &'a [RpgPhysicalContactFactV1],
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
    physics: PhysicsWorldHost,
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
    let mut rpg_plan_traces = Vec::new();
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
            CandidateExecution::Committed(result, committed_events, plan_trace) => {
                committed = checked_inc(committed)?;
                results.push(result);
                events.extend(committed_events);
                if let Some(plan_trace) = plan_trace {
                    rpg_plan_traces.push(plan_trace);
                }
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
        rpg_plan_traces,
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
    Committed(
        OrderedResult,
        Vec<DomainEvent>,
        Option<CommittedRpgPlanTraceV1>,
    ),
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

    let (after_rpg, events, delta, plan_trace) = match &command.payload {
        CommandPayload::Noop => (
            staged.rpg.clone(),
            vec![DomainEvent::command_committed(
                context.tick,
                context.phase,
                candidate.command_id,
                command.sequence,
            )?],
            Vec::new(),
            None,
        ),
        CommandPayload::Rpg(rpg_command) => {
            let planning_context = RpgPlanningContextV1 {
                gameplay_tick: context.tick,
                causal_command_id: candidate.command_id,
                canonical_command_body_hash: command.body_hash()?,
                project_composition_lock_hash: context.rpg_bindings.project_composition_lock_hash,
                schema_registry_hash: context.rpg_bindings.schema_registry_hash,
                budget_policy_hash: context.rpg_bindings.budget_policy_hash,
                active_definition_policy_hashes: &context
                    .rpg_bindings
                    .active_definition_policy_hashes,
                physical_contact_facts: context.physical_contact_facts,
            };
            let plan = match build_transaction_plan_v1(&staged.rpg, rpg_command, planning_context) {
                Ok(plan) => plan,
                Err(error) => {
                    return finalize_rejection(
                        context,
                        candidate,
                        staged,
                        rpg_rejection_code(&error),
                    );
                }
            };
            let next_rpg = match materialize_transaction_plan_v1(&staged.rpg, &plan) {
                Ok(state) => state,
                Err(RpgPlanMaterializeError::PlanStale) => {
                    return finalize_rejection(
                        context,
                        candidate,
                        staged,
                        RejectionCode::RpgPlanStale,
                    );
                }
                Err(_) => {
                    return finalize_rejection(
                        context,
                        candidate,
                        staged,
                        RejectionCode::RpgTransactionAborted,
                    );
                }
            };
            let mut events = Vec::with_capacity(plan.as_contract().ordered_event_drafts.len());
            for (event_slot, draft) in plan.as_contract().ordered_event_drafts.iter().enumerate() {
                events.push(DomainEvent::rpg(
                    context.tick,
                    context.phase,
                    candidate.command_id,
                    u32::try_from(event_slot)
                        .map_err(|_| RuntimeFatalError::EventCountExhausted)?,
                    draft.event.clone(),
                )?);
            }
            let delta = canonical_rpg_plan_delta(plan.as_contract())?;
            let trace = CommittedRpgPlanTraceV1 {
                command_id: candidate.command_id,
                plan_hash: plan.as_contract().plan_hash,
            };
            (next_rpg, events, delta, Some(trace))
        }
        CommandPayload::Physical(_) => {
            unreachable!("physical commands return a pending step before domain execution")
        }
    };
    let mut next_ledger = staged.ledger.clone();
    for event in &events {
        let provenance = event.canonical_bytes()?;
        let insert = next_ledger
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
    let event_ids = events
        .iter()
        .map(|event| event.event_id)
        .collect::<Vec<_>>();
    let transaction_root = transaction_result_root(
        candidate.command_id.as_bytes(),
        context.phase,
        &delta,
        &event_ids,
    );
    let receipt = command_receipt(
        context,
        next_ledger
            .streams
            .get(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?,
        command,
        candidate.command_id,
        CommandFinalResultV1::Committed,
        event_ids,
        transaction_root,
    )?;
    next_ledger
        .streams
        .get_mut(&command.stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?
        .append_receipt(receipt)?;
    let event_increment =
        u64::try_from(events.len()).map_err(|_| RuntimeFatalError::EventCountExhausted)?;
    let next_event_count = staged
        .event_count
        .checked_add(event_increment)
        .ok_or(RuntimeFatalError::EventCountExhausted)?;
    let next_revision = staged
        .revision
        .checked_add(1)
        .ok_or(RuntimeFatalError::RevisionExhausted)?;
    staged.ledger = next_ledger;
    staged.event_count = next_event_count;
    staged.revision = next_revision;
    staged.rpg = after_rpg;
    Ok(CandidateExecution::Committed(
        OrderedResult::committed(candidate.order_key, candidate.command_id, command.sequence),
        events,
        plan_trace,
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

fn canonical_rpg_plan_delta(plan: &RpgTransactionPlanV1) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.rpg-owner-write-set.v1\0");
    bytes.extend_from_slice(
        &u32::try_from(plan.ordered_write_set.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for write in &plan.ordered_write_set {
        bytes.push(write.aggregate_kind as u8);
        bytes.extend_from_slice(write.persistent_id.as_bytes());
        bytes.extend_from_slice(&write.before_revision.to_le_bytes());
        bytes.extend_from_slice(&write.after_revision.to_le_bytes());
        bytes.extend_from_slice(write.before_state_hash.as_bytes());
        bytes.extend_from_slice(write.after_state_hash.as_bytes());
        let after = write.after.canonical_bytes()?;
        bytes.extend_from_slice(
            &u32::try_from(after.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&after);
    }
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

fn rpg_rejection_code(error: &RpgPlanBuildError) -> RejectionCode {
    match error {
        RpgPlanBuildError::Contract(error) => RejectionCode::from_stable_code(error.stable_code()),
        RpgPlanBuildError::AggregateNotFound(_) => RejectionCode::RpgAggregateNotFound,
        RpgPlanBuildError::RevisionStale { .. } => RejectionCode::RpgRevisionStale,
        RpgPlanBuildError::RevisionExhausted => RejectionCode::RpgRevisionExhausted,
        RpgPlanBuildError::TransitionInvalid => RejectionCode::RpgTransitionInvalid,
        RpgPlanBuildError::OwnershipConflict => RejectionCode::RpgOwnershipConflict,
        RpgPlanBuildError::ReservationInvalid => RejectionCode::RpgReservationInvalid,
        RpgPlanBuildError::PhysicalPreconditionMissing => {
            RejectionCode::RpgPhysicalPreconditionMissing
        }
        RpgPlanBuildError::DefinitionMismatch => RejectionCode::RpgDefinitionMismatch,
        RpgPlanBuildError::CommitmentRejected => RejectionCode::RpgCommitmentRejected,
        RpgPlanBuildError::TransactionAborted => RejectionCode::RpgTransactionAborted,
        _ => RejectionCode::RpgTransactionAborted,
    }
}

fn validate_rpg_snapshot(snapshot: RpgSnapshotV2) -> Result<RpgState, SnapshotRestoreError> {
    let canonical_bytes = snapshot.canonical_bytes()?;
    let snapshot =
        RpgSnapshotV2::from_canonical_bytes(&canonical_bytes, CanonicalDecodeLimits::default())?;
    Ok(RpgState::from_snapshot(snapshot)?)
}

#[cfg(test)]
mod tests;
