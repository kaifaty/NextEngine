use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::command::IssuerPrincipal;
use next_contracts::identity::{
    CommandStreamRegistryV1, PrincipalRegistryV1, RuntimeDeterminismProfileV1,
    WorldIdentityManifestV1,
};
use next_contracts::ids::InputSourceId;
use next_contracts::input::{
    ActionMapManifestV1, ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1,
    IngressAssignmentProfileV1, IngressCheckpointV1, InputContextStackV1, InputContractError,
    InputMappingReceiptV1, InputMappingReceiptV2, InputSampleV1, PlayerControllerRegistryV1,
    RuntimeAdmissionLimitsV1, TickRateProfileV1,
};
use next_contracts::ledger::{CommandBodyArchiveV1, CommandLedgerV2, CommandStreamLedgerV2};
use next_contracts::mechanics::RpgDefinitionRegistryV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PhysicsCanonicalSnapshotV2, PhysicsQuantizationProfileV1,
    PhysicsWorldCheckpointV1,
};
use next_contracts::rpg::{RpgRuntimeBindingsV1, RpgSnapshotV2};
use next_contracts::snapshot::{
    RuntimeSnapshotV3, WorldCheckpointCanonicalComponentsV1, WorldCheckpointError,
    WorldCheckpointV4,
};
use next_physics_api::{PhysicsBackendKind, PhysicsWorldHost};
use next_rpg::RpgState;

use crate::authority::AuthorityRegistry;
use crate::registry::CommandKindRegistry;

use super::bootstrap::{
    RuntimeBootstrapV3, register_bootstrap_identities, validate_bootstrap,
    validate_core_interaction_runtime_closure, validate_rpg_snapshot,
};
use super::error::{InputAdmissionError, SnapshotRestoreError};
use super::physics::{PhysicsLaunchOptions, activate_physics};

#[derive(Debug)]
pub struct RuntimeState {
    pub(super) registry: CommandKindRegistry,
    pub(super) authority: AuthorityRegistry,
    pub(super) next_tick: u64,
    pub(super) committed_event_count: u64,
    pub(super) authoritative_revision: u64,
    pub(super) world_identity: WorldIdentityManifestV1,
    pub(super) principal_registry: PrincipalRegistryV1,
    pub(super) stream_registry: CommandStreamRegistryV1,
    pub(super) runtime_profile: RuntimeDeterminismProfileV1,
    pub(super) admission_limits: RuntimeAdmissionLimitsV1,
    pub(super) tick_rate_profile: TickRateProfileV1,
    pub(super) ingress_assignment_profile: IngressAssignmentProfileV1,
    pub(super) authoritative_numeric_profile: AuthoritativeNumericProfileV1,
    pub(super) physics_quantization_profile: PhysicsQuantizationProfileV1,
    pub(super) player_controller_registry: PlayerControllerRegistryV1,
    pub(super) ingress_checkpoint: IngressCheckpointV1,
    pub(super) physics: PhysicsWorldHost,
    pub(super) last_closed_ingress_batch: Option<ClosedIngressBatchV1>,
    pub(super) last_mapping_receipts: Vec<InputMappingReceiptV1>,
    pub(super) last_mapping_receipts_v2: Vec<InputMappingReceiptV2>,
    pub(super) last_command_batches: Vec<ClosedCommandAdmissionBatchV2>,
    pub(super) command_ledger: CommandLedgerV2,
    pub(super) body_archive: CommandBodyArchiveV1,
    pub(super) rpg: RpgState,
    pub(super) rpg_bindings: RpgRuntimeBindingsV1,
    pub(super) rpg_definitions: RpgDefinitionRegistryV1,
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
            last_mapping_receipts_v2: Vec::new(),
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
        let snapshot = RuntimeSnapshotV3::from_canonical_bytes(
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
        snapshot: RuntimeSnapshotV3,
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
            last_mapping_receipts_v2: Vec::new(),
            last_command_batches: Vec::new(),
            command_ledger: snapshot.command_ledger,
            body_archive: snapshot.body_archive,
            rpg,
            rpg_bindings: bootstrap.rpg_bindings,
            rpg_definitions: bootstrap.rpg_definitions,
        })
    }

    /// Reconstructs an isolated equivalent runtime generation for transactional
    /// staging. Backend caches are rebuilt from the canonical checkpoint and
    /// never become shared mutable authority.
    pub fn fork_from_checkpoint(&self) -> Result<Self, SnapshotRestoreError> {
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

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<(WorldCheckpointV4, WorldCheckpointCanonicalComponentsV1), WorldCheckpointError>
    {
        WorldCheckpointV4::new_with_canonical_components(
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
    pub fn snapshot(&self) -> RuntimeSnapshotV3 {
        RuntimeSnapshotV3 {
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
    pub fn last_mapping_receipts_v2(&self) -> &[InputMappingReceiptV2] {
        &self.last_mapping_receipts_v2
    }

    #[must_use]
    pub fn last_command_batches(&self) -> &[ClosedCommandAdmissionBatchV2] {
        &self.last_command_batches
    }

    /// Activates an exact input-map/context revision at an ingress boundary.
    ///
    /// Both ingress queues must be empty so no sample captured against the old
    /// revision can be interpreted against the new registry entry. Revision
    /// and hash collisions fail before the live registry is changed.
    pub fn activate_player_input_configuration(
        &mut self,
        source_id: InputSourceId,
        action_map: ActionMapManifestV1,
        context_stack: InputContextStackV1,
    ) -> Result<(), InputContractError> {
        if !self.ingress_checkpoint.current_samples.is_empty()
            || !self.ingress_checkpoint.next_samples.is_empty()
        {
            return Err(InputContractError::InvalidProfile);
        }
        let mut candidate = self.player_controller_registry.clone();
        candidate.activate_input_configuration(source_id, action_map, context_stack)?;
        candidate.validate()?;
        self.player_controller_registry = candidate;
        Ok(())
    }

    pub fn enqueue_input_sample(
        &mut self,
        principal: &IssuerPrincipal,
        sample: InputSampleV1,
    ) -> Result<(), InputAdmissionError> {
        self.enqueue_input_sample_in_queue(principal, sample, IngressQueueV1::Current)
    }

    /// Enqueues an input sample on the far side of the current ingress close
    /// barrier. The sample is persisted in `next_samples` and cannot be
    /// assigned before the following gameplay tick.
    pub fn enqueue_input_sample_for_next_tick(
        &mut self,
        principal: &IssuerPrincipal,
        sample: InputSampleV1,
    ) -> Result<(), InputAdmissionError> {
        self.enqueue_input_sample_in_queue(principal, sample, IngressQueueV1::Next)
    }

    fn enqueue_input_sample_in_queue(
        &mut self,
        principal: &IssuerPrincipal,
        sample: InputSampleV1,
        queue: IngressQueueV1,
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
        let samples = match queue {
            IngressQueueV1::Current => &mut self.ingress_checkpoint.current_samples,
            IngressQueueV1::Next => &mut self.ingress_checkpoint.next_samples,
        };
        if samples.len() >= maximum {
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
        samples.push(sample);
        samples.sort_by(|left, right| {
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
}

#[derive(Clone, Copy)]
enum IngressQueueV1 {
    Current,
    Next,
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
