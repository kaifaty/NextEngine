use next_contracts::command::{IssuerPrincipal, WorldCommand};
use next_contracts::ids::{ContentHash, InputSourceId};
use next_contracts::input::{
    ActionMapManifestV1, ClosedIngressBatchV1, IngressCheckpointV1, InputContextStackV1,
    InputSampleV1,
};

use super::{PreparedRuntimeTick, RuntimeFatalError, RuntimeState};
use crate::engine::state::player_controller_registry_generation_hash;
use crate::engine::state::{IngressQueueV1, enqueue_input_sample_in_checkpoint};
use crate::outcome::{NoOutcomes, OutcomeProvider};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeGenerationV1 {
    next_tick: u64,
    authoritative_revision: u64,
    committed_event_count: u64,
    player_controller_registry_generation: ContentHash,
    ingress_checkpoint: IngressCheckpointV1,
}

impl RuntimeGenerationV1 {
    pub(super) fn capture(runtime: &RuntimeState) -> Self {
        Self {
            next_tick: runtime.next_tick,
            authoritative_revision: runtime.authoritative_revision,
            committed_event_count: runtime.committed_event_count,
            player_controller_registry_generation: runtime.player_controller_registry_generation,
            ingress_checkpoint: runtime.ingress_checkpoint.clone(),
        }
    }

    pub(super) fn matches(&self, runtime: &RuntimeState) -> bool {
        self.next_tick == runtime.next_tick
            && self.authoritative_revision == runtime.authoritative_revision
            && self.committed_event_count == runtime.committed_event_count
            && self.player_controller_registry_generation
                == runtime.player_controller_registry_generation
            && self.ingress_checkpoint == runtime.ingress_checkpoint
    }
}

/// Opaque staging scope for one runtime tick.
///
/// Input ingress is copied into this scope, so admission and tick preparation
/// cannot mutate the live runtime generation.
pub struct RuntimeTickPreparation<'a> {
    pub(super) runtime: &'a RuntimeState,
    pub(super) base_generation: RuntimeGenerationV1,
    pub(super) ingress_checkpoint: IngressCheckpointV1,
}

impl RuntimeTickPreparation<'_> {
    pub fn enqueue_input_sample(
        &mut self,
        principal: &IssuerPrincipal,
        sample: InputSampleV1,
    ) -> Result<(), crate::engine::error::InputAdmissionError> {
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

    pub(super) fn prepare_internal(
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
            None,
            None,
            None,
            None,
            None,
        )
    }
}

impl RuntimeState {
    /// Stages an input-map/context activation inside the same immutable
    /// generation as `prepared`.
    ///
    /// The target ingress queues must be empty, exactly as for direct boundary
    /// activation. Validation and snapshot publication therefore happen before
    /// any live runtime field changes; the later validated tick commit applies
    /// the controller registry together with the rest of the generation and is
    /// infallible.
    pub fn stage_player_input_configuration_activation(
        &self,
        mut prepared: PreparedRuntimeTick,
        source_id: InputSourceId,
        action_map: ActionMapManifestV1,
        context_stack: InputContextStackV1,
    ) -> Result<PreparedRuntimeTick, RuntimeFatalError> {
        if !prepared.base_generation.matches(self) {
            return Err(RuntimeFatalError::PreparedGenerationStale);
        }
        if prepared.report.get().is_some()
            || !prepared.staged.ingress.current_samples.is_empty()
            || !prepared.staged.ingress.next_samples.is_empty()
        {
            return Err(next_contracts::input::InputContractError::InvalidProfile.into());
        }
        let mut candidate = self.player_controller_registry.clone();
        candidate.activate_input_configuration(source_id, action_map, context_stack)?;
        candidate.validate()?;
        prepared.player_controller_registry_generation =
            player_controller_registry_generation_hash(&candidate)?;
        prepared
            .report_parts
            .snapshot_fields
            .player_controller_registry = candidate;
        Ok(prepared)
    }
}
