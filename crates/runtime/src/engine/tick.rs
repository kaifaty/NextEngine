use next_contracts::command::{CommandPayload, CommandPhase, IssuerPrincipal, WorldCommand};
use next_contracts::ids::ContentHash;
use next_contracts::input::{
    CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION, ClosedCommandAdmissionBatchBodyV2,
    ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1, IngressCheckpointV1,
    InputDerivedCommandRefV2, InputMappingCodeV1, InputMappingReceiptV1, InputMappingReceiptV2,
    InputSampleV1,
};
use next_contracts::physics::PhysicsQueryBatchV1;
use next_contracts::snapshot::{
    RuntimeSnapshotV3, WorldCheckpointCanonicalComponentsV1, WorldCheckpointError,
    WorldCheckpointV4,
};
use next_contracts::targeting::{AuthoritativeTargetingQueryV1, TargetingIntentV1};
use next_physics_api::PhysicsSceneQueryError;
use std::sync::OnceLock;

use crate::outcome::{NoOutcomes, OutcomeContext, OutcomeProvider, OutcomeSink};
use crate::stage_zone::stage_zone;

use super::error::{InputAdmissionError, RuntimeFatalError};
use super::ingress::{accept_closed_ingress, close_ingress, finalize_mapping_receipt_v2};
use super::interaction::{
    InteractionBuildContext, build_interaction_outcomes, resolve_interaction_outcome_route,
    rpg_physical_contact_facts,
};
use super::order::sort_command_batch;
use super::pipeline::{
    PhaseContext, PreparedCommandLedgerTransaction, StagedAuthoritativeState, ValidationSource,
    count, process_phase,
};
use super::result::{StageTraceEntry, TickReport, TransactionStage};
use super::state::{IngressQueueV1, RuntimeState, enqueue_input_sample_in_checkpoint};

mod preparation;

use preparation::RuntimeGenerationV1;

/// Opaque staging scope for one runtime tick.
///
/// Input ingress is copied into this scope, so admission and tick preparation
/// cannot mutate the live runtime generation.
pub struct RuntimeTickPreparation<'a> {
    runtime: &'a RuntimeState,
    base_generation: RuntimeGenerationV1,
    ingress_checkpoint: IngressCheckpointV1,
}

impl RuntimeTickPreparation<'_> {
    pub fn enqueue_input_sample(
        &mut self,
        principal: &IssuerPrincipal,
        sample: InputSampleV1,
    ) -> Result<(), InputAdmissionError> {
        enqueue_input_sample_in_checkpoint(
            &self.runtime.admission_limits,
            &self.runtime.principal_registry,
            &self.runtime.authority,
            &self.runtime.player_controller_registry,
            &mut self.ingress_checkpoint,
            principal,
            sample,
            IngressQueueV1::Current,
        )
    }

    pub fn prepare(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
    ) -> Result<PreparedRuntimeTick, RuntimeFatalError> {
        self.prepare_with_outcomes(commands, &mut NoOutcomes)
    }

    pub fn prepare_with_outcomes(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        outcome_provider: &mut impl OutcomeProvider,
    ) -> Result<PreparedRuntimeTick, RuntimeFatalError> {
        self.prepare_internal(commands, outcome_provider, None)
    }

    fn prepare_internal(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        outcome_provider: &mut impl OutcomeProvider,
        replay_ingress: Option<ClosedIngressBatchV1>,
    ) -> Result<PreparedRuntimeTick, RuntimeFatalError> {
        self.runtime.prepare_tick_internal(
            self.base_generation,
            self.ingress_checkpoint,
            commands,
            outcome_provider,
            replay_ingress,
        )
    }
}

/// A completely staged and checked next tick. The live runtime is unchanged.
pub struct PreparedRuntimeTick {
    base_generation: RuntimeGenerationV1,
    next_tick: u64,
    player_controller_registry_generation: ContentHash,
    staged: StagedAuthoritativeState,
    ledger_update: PreparedCommandLedgerTransaction,
    report: OnceLock<TickReport>,
    report_parts: PreparedTickReportParts,
    last_closed_ingress_batch: ClosedIngressBatchV1,
    last_mapping_receipts: Vec<InputMappingReceiptV1>,
    last_mapping_receipts_v2: Vec<InputMappingReceiptV2>,
    last_command_batches: Vec<ClosedCommandAdmissionBatchV2>,
}

struct PreparedTickReportParts {
    tick: u64,
    results: Vec<super::result::CommandResult>,
    events: Vec<next_contracts::command::DomainEvent>,
    stage_trace: Vec<StageTraceEntry>,
    snapshot_fields: PreparedRuntimeSnapshotFields,
    physics_step_input: next_contracts::physics::PhysicsStepInputV2,
    contact_batch: next_contracts::physics::ClosedPhysicsContactBatchV1,
    physics_checkpoint_hash: next_contracts::ids::ContentHash,
    targeting_intents: Vec<TargetingIntentV1>,
    authoritative_targeting_queries: Vec<AuthoritativeTargetingQueryV1>,
    physics_query_batch: PhysicsQueryBatchV1,
    physics_query_results: Vec<next_contracts::physics::PhysicsQueryResultV1>,
    rpg_plan_traces: Vec<super::result::CommittedRpgPlanTraceV1>,
}

/// Immutable snapshot fields copied from the prepared runtime generation.
///
/// Dynamic authoritative state stays in `StagedAuthoritativeState`; in
/// particular, the staged command ledger and body archive are not cloned a
/// second time into a public snapshot until a caller asks for the tick report.
struct PreparedRuntimeSnapshotFields {
    world_identity: next_contracts::identity::WorldIdentityManifestV1,
    principal_registry: next_contracts::identity::PrincipalRegistryV1,
    stream_registry: next_contracts::identity::CommandStreamRegistryV1,
    runtime_profile: next_contracts::identity::RuntimeDeterminismProfileV1,
    admission_limits: next_contracts::input::RuntimeAdmissionLimitsV1,
    tick_rate_profile: next_contracts::input::TickRateProfileV1,
    ingress_assignment_profile: next_contracts::input::IngressAssignmentProfileV1,
    authoritative_numeric_profile: next_contracts::physics::AuthoritativeNumericProfileV1,
    physics_quantization_profile: next_contracts::physics::PhysicsQuantizationProfileV1,
    player_controller_registry: next_contracts::input::PlayerControllerRegistryV1,
    rpg_runtime_bindings: next_contracts::rpg::RpgRuntimeBindingsV1,
}

#[derive(Clone, Copy)]
struct PreparedTickReportSources<'a> {
    next_tick: u64,
    staged: &'a StagedAuthoritativeState,
    closed_ingress_batch: &'a ClosedIngressBatchV1,
    mapping_receipts: &'a [InputMappingReceiptV1],
    mapping_receipts_v2: &'a [InputMappingReceiptV2],
    command_batches: &'a [ClosedCommandAdmissionBatchV2],
}

impl PreparedRuntimeSnapshotFields {
    fn snapshot(
        &self,
        next_tick: u64,
        staged: &StagedAuthoritativeState,
        command_ledger: next_contracts::ledger::CommandLedgerV2,
        body_archive: next_contracts::ledger::CommandBodyArchiveV1,
    ) -> RuntimeSnapshotV3 {
        RuntimeSnapshotV3 {
            next_tick,
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
            rpg_runtime_bindings: self.rpg_runtime_bindings.clone(),
            command_ledger,
            body_archive,
        }
    }

    fn into_snapshot(
        self,
        next_tick: u64,
        staged: &StagedAuthoritativeState,
        command_ledger: next_contracts::ledger::CommandLedgerV2,
        body_archive: next_contracts::ledger::CommandBodyArchiveV1,
    ) -> RuntimeSnapshotV3 {
        RuntimeSnapshotV3 {
            next_tick,
            committed_event_count: staged.event_count,
            authoritative_revision: staged.revision,
            world_identity: self.world_identity,
            principal_registry: self.principal_registry,
            stream_registry: self.stream_registry,
            runtime_profile: self.runtime_profile,
            admission_limits: self.admission_limits,
            tick_rate_profile: self.tick_rate_profile,
            ingress_assignment_profile: self.ingress_assignment_profile,
            authoritative_numeric_profile: self.authoritative_numeric_profile,
            physics_quantization_profile: self.physics_quantization_profile,
            player_controller_registry: self.player_controller_registry,
            ingress_checkpoint: staged.ingress.clone(),
            rpg_runtime_bindings: self.rpg_runtime_bindings,
            command_ledger,
            body_archive,
        }
    }
}

impl PreparedTickReportParts {
    fn report(
        &self,
        sources: PreparedTickReportSources<'_>,
        command_ledger: next_contracts::ledger::CommandLedgerV2,
        body_archive: next_contracts::ledger::CommandBodyArchiveV1,
    ) -> TickReport {
        TickReport {
            tick: self.tick,
            results: self.results.clone(),
            events: self.events.clone(),
            stage_trace: self.stage_trace.clone(),
            snapshot: self.snapshot_fields.snapshot(
                sources.next_tick,
                sources.staged,
                command_ledger,
                body_archive,
            ),
            rpg_snapshot: sources.staged.rpg.snapshot(),
            physics_snapshot: sources.staged.physics.snapshot().clone(),
            physics_step_input: self.physics_step_input.clone(),
            contact_batch: self.contact_batch.clone(),
            physics_checkpoint_hash: self.physics_checkpoint_hash,
            targeting_intents: self.targeting_intents.clone(),
            authoritative_targeting_queries: self.authoritative_targeting_queries.clone(),
            physics_query_batch: self.physics_query_batch.clone(),
            physics_query_results: self.physics_query_results.clone(),
            closed_ingress_batch: sources.closed_ingress_batch.clone(),
            mapping_receipts: sources.mapping_receipts.to_vec(),
            mapping_receipts_v2: sources.mapping_receipts_v2.to_vec(),
            command_batches: sources.command_batches.to_vec(),
            rpg_plan_traces: self.rpg_plan_traces.clone(),
        }
    }

    fn into_report(
        self,
        sources: PreparedTickReportSources<'_>,
        command_ledger: next_contracts::ledger::CommandLedgerV2,
        body_archive: next_contracts::ledger::CommandBodyArchiveV1,
    ) -> TickReport {
        TickReport {
            tick: self.tick,
            results: self.results,
            events: self.events,
            stage_trace: self.stage_trace,
            snapshot: self.snapshot_fields.into_snapshot(
                sources.next_tick,
                sources.staged,
                command_ledger,
                body_archive,
            ),
            rpg_snapshot: sources.staged.rpg.snapshot(),
            physics_snapshot: sources.staged.physics.snapshot().clone(),
            physics_step_input: self.physics_step_input,
            contact_batch: self.contact_batch,
            physics_checkpoint_hash: self.physics_checkpoint_hash,
            targeting_intents: self.targeting_intents,
            authoritative_targeting_queries: self.authoritative_targeting_queries,
            physics_query_batch: self.physics_query_batch,
            physics_query_results: self.physics_query_results,
            closed_ingress_batch: sources.closed_ingress_batch.clone(),
            mapping_receipts: sources.mapping_receipts.to_vec(),
            mapping_receipts_v2: sources.mapping_receipts_v2.to_vec(),
            command_batches: sources.command_batches.to_vec(),
            rpg_plan_traces: self.rpg_plan_traces,
        }
    }
}

impl PreparedRuntimeTick {
    #[cfg(test)]
    pub(super) fn report_is_materialized(&self) -> bool {
        self.report.get().is_some()
    }

    #[must_use]
    pub fn report(&self) -> &TickReport {
        self.report.get_or_init(|| {
            stage_zone!("TickReportMaterialize");
            let (ledger, archive) = self
                .ledger_update
                .materialize(&self.staged.ledger, &self.staged.archive);
            self.report_parts.report(
                PreparedTickReportSources {
                    next_tick: self.next_tick,
                    staged: &self.staged,
                    closed_ingress_batch: &self.last_closed_ingress_batch,
                    mapping_receipts: &self.last_mapping_receipts,
                    mapping_receipts_v2: &self.last_mapping_receipts_v2,
                    command_batches: &self.last_command_batches,
                },
                ledger,
                archive,
            )
        })
    }

    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.next_tick
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        &self.report_parts.events
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.staged.physics.snapshot()
    }

    /// Staged immutable RPG projection of the prepared tick for read-only
    /// presentation probes at the same publication boundary.
    #[must_use]
    pub fn rpg_snapshot(&self) -> next_contracts::rpg::RpgSnapshotV2 {
        self.staged.rpg.snapshot()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<(WorldCheckpointV4, WorldCheckpointCanonicalComponentsV1), WorldCheckpointError>
    {
        let report = self.report();
        WorldCheckpointV4::new_with_incrementally_validated_canonical_components(
            report.snapshot.clone(),
            report.rpg_snapshot.clone(),
            self.staged.physics.checkpoint().clone(),
        )
    }
}

/// A prepared tick bound to the runtime generation that validated it.
pub struct ValidatedRuntimeTick(PreparedRuntimeTick);

impl ValidatedRuntimeTick {
    #[must_use]
    pub fn report(&self) -> &TickReport {
        self.0.report()
    }

    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.0.next_tick()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.0.physics_snapshot()
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        self.0.events()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<(WorldCheckpointV4, WorldCheckpointCanonicalComponentsV1), WorldCheckpointError>
    {
        self.0.world_checkpoint_with_canonical_components()
    }
}

impl RuntimeState {
    #[must_use]
    pub fn tick_preparation(&self) -> RuntimeTickPreparation<'_> {
        RuntimeTickPreparation {
            runtime: self,
            base_generation: RuntimeGenerationV1::capture(self),
            ingress_checkpoint: self.ingress_checkpoint.clone(),
        }
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
        let prepared = self
            .tick_preparation()
            .prepare_with_outcomes(commands, outcome_provider)?;
        let validated = self.validate_prepared_tick(prepared)?;
        Ok(self.commit_validated_tick(validated))
    }

    pub fn validate_prepared_tick(
        &self,
        prepared: PreparedRuntimeTick,
    ) -> Result<ValidatedRuntimeTick, RuntimeFatalError> {
        if !prepared.base_generation.matches(self) {
            return Err(RuntimeFatalError::PreparedGenerationStale);
        }
        Ok(ValidatedRuntimeTick(prepared))
    }

    #[must_use]
    pub fn commit_validated_tick(&mut self, validated: ValidatedRuntimeTick) -> TickReport {
        stage_zone!("RuntimeCommit");
        let ValidatedRuntimeTick(prepared) = validated;
        debug_assert!(prepared.base_generation.matches(self));
        let PreparedRuntimeTick {
            next_tick,
            player_controller_registry_generation,
            staged,
            ledger_update,
            report,
            report_parts,
            last_closed_ingress_batch,
            last_mapping_receipts,
            last_mapping_receipts_v2,
            last_command_batches,
            ..
        } = prepared;
        let committed_player_controller_registry = report_parts
            .snapshot_fields
            .player_controller_registry
            .clone();
        let report = report.into_inner().unwrap_or_else(|| {
            let (ledger, archive) = ledger_update.materialize(&staged.ledger, &staged.archive);
            report_parts.into_report(
                PreparedTickReportSources {
                    next_tick,
                    staged: &staged,
                    closed_ingress_batch: &last_closed_ingress_batch,
                    mapping_receipts: &last_mapping_receipts,
                    mapping_receipts_v2: &last_mapping_receipts_v2,
                    command_batches: &last_command_batches,
                },
                ledger,
                archive,
            )
        });
        self.next_tick = report.snapshot.next_tick;
        self.committed_event_count = staged.event_count;
        self.authoritative_revision = staged.revision;
        self.player_controller_registry = committed_player_controller_registry;
        self.player_controller_registry_generation = player_controller_registry_generation;
        self.command_ledger = report.snapshot.command_ledger.clone();
        self.body_archive = report.snapshot.body_archive.clone();
        self.ledger_roots_dirty = false;
        self.ledger_snapshot_cache = OnceLock::new();
        self.rpg = staged.rpg;
        self.physics = staged.physics;
        self.ingress_checkpoint = staged.ingress;
        self.last_closed_ingress_batch = Some(last_closed_ingress_batch);
        self.last_mapping_receipts = last_mapping_receipts;
        self.last_mapping_receipts_v2 = last_mapping_receipts_v2;
        self.last_command_batches = last_command_batches;
        report
    }

    /// Commits a validated tick without materializing its complete public
    /// report. Interactive presentation uses this path between durable
    /// checkpoint boundaries.
    pub fn commit_validated_tick_without_report(&mut self, validated: ValidatedRuntimeTick) {
        stage_zone!("RuntimeCommit");
        let ValidatedRuntimeTick(prepared) = validated;
        debug_assert!(prepared.base_generation.matches(self));
        let PreparedRuntimeTick {
            next_tick,
            player_controller_registry_generation,
            staged,
            ledger_update,
            report,
            report_parts,
            last_closed_ingress_batch,
            last_mapping_receipts,
            last_mapping_receipts_v2,
            last_command_batches,
            ..
        } = prepared;
        let committed_player_controller_registry = report_parts
            .snapshot_fields
            .player_controller_registry
            .clone();
        if let Some(report) = report.into_inner() {
            self.command_ledger = report.snapshot.command_ledger;
            self.body_archive = report.snapshot.body_archive;
            self.ledger_roots_dirty = false;
            self.ledger_snapshot_cache = OnceLock::new();
        } else {
            drop(report_parts);
            let StagedAuthoritativeState {
                ledger,
                archive,
                event_count,
                revision,
                rpg,
                physics,
                ingress,
                ledger_delta: _,
            } = staged;
            drop(archive);
            ledger_update.commit_deferred_roots(
                &mut self.command_ledger,
                &mut self.body_archive,
                ledger,
            );
            self.ledger_roots_dirty = true;
            self.ledger_snapshot_cache = OnceLock::new();
            self.next_tick = next_tick;
            self.committed_event_count = event_count;
            self.authoritative_revision = revision;
            self.player_controller_registry = committed_player_controller_registry;
            self.player_controller_registry_generation = player_controller_registry_generation;
            self.rpg = rpg;
            self.physics = physics;
            self.ingress_checkpoint = ingress;
            self.last_closed_ingress_batch = Some(last_closed_ingress_batch);
            self.last_mapping_receipts = last_mapping_receipts;
            self.last_mapping_receipts_v2 = last_mapping_receipts_v2;
            self.last_command_batches = last_command_batches;
            return;
        }
        self.next_tick = next_tick;
        self.committed_event_count = staged.event_count;
        self.authoritative_revision = staged.revision;
        self.player_controller_registry = committed_player_controller_registry;
        self.player_controller_registry_generation = player_controller_registry_generation;
        self.rpg = staged.rpg;
        self.physics = staged.physics;
        self.ingress_checkpoint = staged.ingress;
        self.last_closed_ingress_batch = Some(last_closed_ingress_batch);
        self.last_mapping_receipts = last_mapping_receipts;
        self.last_mapping_receipts_v2 = last_mapping_receipts_v2;
        self.last_command_batches = last_command_batches;
    }

    pub(super) fn prepare_replay_ingress_tick(
        &self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: impl IntoIterator<Item = WorldCommand>,
    ) -> Result<ValidatedRuntimeTick, RuntimeFatalError> {
        let prepared = self.tick_preparation().prepare_internal(
            direct_commands,
            &mut NoOutcomes,
            Some(closed_ingress_batch),
        )?;
        self.validate_prepared_tick(prepared)
    }

    pub(super) fn preview_replay_ingress_batch(
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

    fn prepare_tick_internal(
        &self,
        base_generation: RuntimeGenerationV1,
        ingress_checkpoint: IngressCheckpointV1,
        commands: impl IntoIterator<Item = WorldCommand>,
        outcome_provider: &mut impl OutcomeProvider,
        replay_ingress: Option<ClosedIngressBatchV1>,
    ) -> Result<PreparedRuntimeTick, RuntimeFatalError> {
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
            ledger_delta: Default::default(),
            event_count: self.committed_event_count,
            revision: self.authoritative_revision,
            rpg: self.rpg.clone(),
            physics: self.physics.fork_for_staging()?,
            ingress: ingress_checkpoint,
        };

        let mut closed_ingress = {
            stage_zone!("IngressClose");
            match replay_ingress {
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
            }
        };
        let (ingress_commands, ingress_batch) = {
            stage_zone!("IngressAdmission");
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
            (ingress_commands, ingress_batch)
        };

        let ingress_revision = staged.revision;
        let ingress = {
            stage_zone!("IngressCommit");
            process_phase(
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
            )?
        };
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

        let mut built_in_resolution = {
            stage_zone!("OutcomeCollection");
            build_interaction_outcomes(
                &closed_ingress.pending_interactions,
                InteractionBuildContext {
                    route: interaction_route.as_ref(),
                    physics: &staged.physics,
                    rpg: &staged.rpg,
                    rpg_definitions: &self.rpg_definitions,
                    ledger: &staged.ledger,
                    archive: &staged.archive,
                    archive_additions: staged.ledger_delta.archive_additions(),
                    gameplay_tick: tick,
                    physical_contact_facts: &physical_contact_facts,
                    authoritative_revision: staged.revision,
                },
            )?
        };
        built_in_resolution
            .targeting
            .sort_by_key(|targeting| targeting.query.physics_query.query_id);
        let query_snapshot_selector =
            staged
                .physics
                .snapshot_hash()
                .map(|physics_snapshot_hash| {
                    next_contracts::physics::PhysicsSnapshotSelectorV1 {
                        physics_tick: staged.physics.snapshot().physics_tick,
                        completed_substep: 0,
                        physics_snapshot_hash,
                    }
                })?;
        let physics_query_batch = PhysicsQueryBatchV1::new(
            query_snapshot_selector,
            built_in_resolution
                .targeting
                .iter()
                .map(|targeting| targeting.query.physics_query.clone())
                .collect(),
        )
        .map_err(PhysicsSceneQueryError::from)?;
        let targeting_intents = built_in_resolution
            .targeting
            .iter()
            .map(|targeting| targeting.intent.clone())
            .collect::<Vec<_>>();
        let authoritative_targeting_queries = built_in_resolution
            .targeting
            .iter()
            .map(|targeting| targeting.query.clone())
            .collect::<Vec<_>>();
        let physics_query_results = built_in_resolution
            .targeting
            .iter()
            .map(|targeting| targeting.result.clone())
            .collect::<Vec<_>>();
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
            built_in_resolution
                .outcomes
                .len()
                .checked_add(external_proposals.len())
                .ok_or(RuntimeFatalError::TraceCountExhausted)?,
        )?;
        let mut outcome_commands = Vec::with_capacity(
            built_in_resolution
                .outcomes
                .len()
                .checked_add(external_proposals.len())
                .ok_or(RuntimeFatalError::TraceCountExhausted)?,
        );
        for built_in in built_in_resolution.outcomes {
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
                        && receipt.code == InputMappingCodeV1::Accepted
                })
                .ok_or(RuntimeFatalError::IngressCheckpointCorrupt)?;
            if receipt.derived_command_id.is_none() {
                receipt.derived_command_id = Some(command_id);
            }
            let receipt_v2 = closed_ingress
                .mapping_receipts_v2
                .iter_mut()
                .find(|receipt| {
                    receipt.source_id == built_in.source_id
                        && receipt.source_sequence == built_in.source_sequence
                        && receipt.payload_hash == built_in.payload_hash
                        && receipt.frame_code == InputMappingCodeV1::Accepted
                })
                .ok_or(RuntimeFatalError::IngressCheckpointCorrupt)?;
            receipt_v2.derived_commands.push(InputDerivedCommandRefV2 {
                command_ordinal: 0,
                source_action_ordinal: built_in.source_action_ordinal,
                mapper_command_slot: 0,
                command_id,
            });
            outcome_commands.push(command);
        }
        for receipt in &mut closed_ingress.mapping_receipts_v2 {
            finalize_mapping_receipt_v2(receipt)?;
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
        let outcome = {
            stage_zone!("OutcomeCommit");
            process_phase(
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
            )?
        };

        stage_zone!("SnapshotPublication");
        let ledger_update =
            std::mem::take(&mut staged.ledger_delta).prepare(&staged.ledger, &staged.archive)?;
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

        let snapshot_fields = PreparedRuntimeSnapshotFields {
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
            rpg_runtime_bindings: self.rpg_bindings.clone(),
        };
        // Incremental mutation APIs validate the exact affected archive,
        // identity, stream and receipt boundaries. Durable serialization and
        // restore still validate the complete public snapshot and historical
        // closure.

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

        let last_command_batches = vec![ingress_batch, outcome_batch];
        let report_parts = PreparedTickReportParts {
            tick,
            results,
            events,
            stage_trace,
            snapshot_fields,
            physics_step_input,
            contact_batch,
            physics_checkpoint_hash: staged.physics.checkpoint_hash()?,
            targeting_intents,
            authoritative_targeting_queries,
            physics_query_batch,
            physics_query_results,
            rpg_plan_traces,
        };

        Ok(PreparedRuntimeTick {
            base_generation,
            next_tick: following_tick,
            player_controller_registry_generation: self.player_controller_registry_generation,
            staged,
            ledger_update,
            report: OnceLock::new(),
            report_parts,
            last_closed_ingress_batch: closed_ingress.batch,
            last_mapping_receipts: closed_ingress.mapping_receipts,
            last_mapping_receipts_v2: closed_ingress.mapping_receipts_v2,
            last_command_batches,
        })
    }
}
