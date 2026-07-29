use next_contracts::ids::{ApplicationSessionId, ContentHash};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{ApplicationSessionStatusV1, PresentationTargetKindV1};
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_reference_game::{ReferenceRunOutcomeV1, run_reference_game};

use crate::ApplicationError;

use super::{ApplicationCoordinator, ApplicationRunOutcomeV1};

#[derive(Clone)]
pub(super) struct PreparedRunV1 {
    pub(super) checkpoint: WorldCheckpointV4,
    pub(super) streaming: WorldStreamingSnapshotV1,
    pub(super) summary: ApplicationRunOutcomeV1,
}

impl ApplicationCoordinator {
    pub fn run_reference_game(
        &mut self,
        include_interaction: bool,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active {
            return Err(ApplicationError::CloseStateInvalid);
        }
        let prepared = prepare_reference_run(
            self.machine.state().session_id,
            self.activated_project.clone(),
            include_interaction,
            self.launch.presentation_target,
        )?;
        self.record_prepared_run(&prepared)?;
        let plan = self
            .machine
            .plan_state_publication(Some(prepared.summary.authoritative_revision), None)?;
        self.publish_state_plan(plan)?;
        let summary = prepared.summary.clone();
        self.prepared_run = Some(prepared);
        Ok(summary)
    }

    fn record_prepared_run(&mut self, prepared: &PreparedRunV1) -> Result<(), ApplicationError> {
        self.record_object(prepared.checkpoint.runtime_snapshot.canonical_bytes()?);
        self.record_object(prepared.checkpoint.rpg_snapshot.canonical_bytes()?);
        self.record_object(prepared.checkpoint.physics_checkpoint.canonical_bytes()?);
        self.record_object(prepared.streaming.canonical_bytes()?);
        Ok(())
    }

    pub(super) fn ensure_prepared_run(&mut self) -> Result<(), ApplicationError> {
        if self.prepared_run.is_some() {
            return Ok(());
        }
        let prepared = prepare_reference_run(
            self.machine.state().session_id,
            self.activated_project.clone(),
            true,
            self.launch.presentation_target,
        )?;
        if self
            .machine
            .state()
            .active_runtime_revision
            .is_some_and(|revision| revision != prepared.summary.authoritative_revision)
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        self.record_prepared_run(&prepared)?;
        self.prepared_run = Some(prepared);
        Ok(())
    }
}

fn prepare_reference_run(
    session_id: ApplicationSessionId,
    project: ActivatedProjectV2,
    include_interaction: bool,
    presentation_target: PresentationTargetKindV1,
) -> Result<PreparedRunV1, ApplicationError> {
    let run: ReferenceRunOutcomeV1 = run_reference_game(project, include_interaction)?;
    let checkpoint = run.runtime.world_checkpoint()?;
    let presentation_snapshot = if presentation_target == PresentationTargetKindV1::None {
        None
    } else {
        let mut extractor = next_presentation::PresentationExtractorV1::new(
            run.project_composition_lock_hash,
            next_contracts::project::domain_hash(
                "nextengine.presentation-profile.b0.v1",
                b"sdr-reference-no-optional-features",
            ),
            8,
        )?;
        Some(
            extractor
                .extract(
                    run.ticks,
                    run.project_composition_lock_hash,
                    run.content_manifest_hash,
                    run.runtime.physics_snapshot(),
                    &run.presentation_bindings,
                )?
                .clone(),
        )
    };
    let presentation_input_count = u64::try_from(run.presentation_bindings.len())
        .map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
    let authoritative_state_root =
        next_contracts::snapshot::world_checkpoint_with_streaming_v1_state_root(
            &checkpoint.runtime_snapshot,
            &checkpoint.rpg_snapshot,
            &checkpoint.physics_checkpoint,
            &run.world_streaming_snapshot,
        )?;
    let summary = ApplicationRunOutcomeV1 {
        session_id,
        project_composition_lock_hash: run.project_composition_lock_hash,
        ticks: run.ticks,
        events: run.events,
        rpg_events: run.rpg_events,
        authoritative_revision: checkpoint.runtime_snapshot.authoritative_revision,
        authoritative_state_root: ContentHash::from_bytes(*authoritative_state_root.as_bytes()),
        command_archive_root: checkpoint
            .runtime_snapshot
            .command_ledger
            .body_archive
            .archive_root,
        command_identity_index_root: checkpoint
            .runtime_snapshot
            .command_ledger
            .identity_index
            .index_root,
        command_ledger_hash: checkpoint.runtime_snapshot.command_ledger_hash()?,
        presentation_input_count,
        presentation_snapshot,
    };
    Ok(PreparedRunV1 {
        checkpoint,
        streaming: run.world_streaming_snapshot,
        summary,
    })
}
