use next_contracts::command::{CommandPayload, CommandPhase, WorldCommand};
use next_contracts::input::{
    CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION, ClosedCommandAdmissionBatchBodyV2,
    ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1, InputDerivedCommandRefV2,
    InputMappingCodeV1,
};
use next_contracts::physics::PhysicsQueryBatchV1;
use next_contracts::snapshot::RuntimeSnapshotV3;
use next_physics_api::PhysicsSceneQueryError;

use crate::outcome::{NoOutcomes, OutcomeContext, OutcomeProvider, OutcomeSink};

use super::error::RuntimeFatalError;
use super::ingress::{accept_closed_ingress, close_ingress, finalize_mapping_receipt_v2};
use super::interaction::{
    InteractionBuildContext, build_interaction_outcomes, resolve_interaction_outcome_route,
    rpg_physical_contact_facts,
};
use super::order::sort_command_batch;
use super::pipeline::{
    PhaseContext, StagedAuthoritativeState, ValidationSource, count, process_phase,
};
use super::result::{StageTraceEntry, TickReport, TransactionStage};
use super::state::RuntimeState;

impl RuntimeState {
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

    pub(super) fn replay_closed_ingress_tick(
        &mut self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: impl IntoIterator<Item = WorldCommand>,
    ) -> Result<TickReport, RuntimeFatalError> {
        self.run_tick_internal(direct_commands, &mut NoOutcomes, Some(closed_ingress_batch))
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

        let mut built_in_resolution = build_interaction_outcomes(
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
        built_in_resolution
            .targeting
            .sort_by_key(|targeting| targeting.query.physics_query.query_id);
        let query_snapshot_selector =
            staged
                .physics
                .checkpoint()
                .snapshot
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

        staged
            .ledger
            .synchronize_archive_incremental(&staged.archive)?;
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

        let snapshot = RuntimeSnapshotV3 {
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
        self.last_mapping_receipts_v2 = closed_ingress.mapping_receipts_v2;
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
            targeting_intents,
            authoritative_targeting_queries,
            physics_query_batch,
            physics_query_results,
            closed_ingress_batch: self
                .last_closed_ingress_batch
                .clone()
                .expect("successful tick publishes its closed ingress batch"),
            mapping_receipts: self.last_mapping_receipts.clone(),
            mapping_receipts_v2: self.last_mapping_receipts_v2.clone(),
            command_batches: self.last_command_batches.clone(),
            rpg_plan_traces,
        })
    }
}
