use next_contracts::ids::{ApplicationSessionId, ContentHash};
use next_contracts::platform::{PlatformEventKindV1, PlatformEventV1};
use next_contracts::session::{ApplicationSessionStatusV1, PresentationTargetKindV1};
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_contracts::world_routine::WorldRoutineSnapshotV1;
use next_reference_game::{
    ReferenceGameDriverV2, ReferenceLiveStateV2, ReferenceRunOutcomeV2, run_reference_game,
};
use std::sync::Arc;

use crate::ApplicationError;

use super::{ApplicationCoordinator, ApplicationRunOutcomeV1};
use super::{save_compatibility, save_identity};

#[derive(Clone)]
pub(super) struct PreparedRunV1 {
    pub(super) checkpoint: WorldCheckpointV4,
    pub(super) streaming: WorldStreamingSnapshotV1,
    pub(super) routine: Option<WorldRoutineSnapshotV1>,
    pub(super) summary: ApplicationRunOutcomeV1,
}

pub(crate) struct InteractivePresentationAdvanceV1 {
    pub(crate) presentation: Arc<next_contracts::presentation::PresentationSnapshotV2>,
    pub(crate) materialized_lifecycle_boundary: bool,
}

/// One published baseline-audio frame of the interactive live driver. The PCM
/// is the exact canonical stereo window the live driver mixed for
/// `simulation_tick`; it is presentation-only and never gameplay evidence.
#[derive(Clone)]
pub struct ApplicationAudioFrameV1 {
    pub audio_sequence: u64,
    pub simulation_tick: u64,
    pub pcm: Arc<[i16]>,
}

impl ApplicationCoordinator {
    pub(super) fn restore_after_crash(&mut self) -> Result<(), ApplicationError> {
        if self.durable.close_journal.is_some() {
            return Ok(());
        }
        if self.machine.state().state == ApplicationSessionStatusV1::Active {
            let request = self.lifecycle_request(
                ApplicationSessionStatusV1::Suspended,
                next_contracts::session::LifecycleReasonKindV1::SuspendRequested,
                "nextengine.session.crash-suspended",
            )?;
            self.publish_transition(
                request,
                next_runtime::SessionTransitionReferencesV1::default(),
            )?;
        }
        if self.machine.state().state != ApplicationSessionStatusV1::Suspended {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        let base = ReferenceGameDriverV2::new_with_presentation_epoch(
            self.activated_package(),
            true,
            presentation_snapshot_epoch(
                self.machine.state().session_id,
                self.activated_project.project_lock.project_lock_sha256,
            ),
        )?;
        let base_prepared = prepare_live_state(
            self.machine.state().session_id,
            base.state()?,
            self.launch.presentation_target,
        )?;
        let compatibility = save_compatibility(&self.activated_project, &base_prepared.checkpoint)?;
        match self.save_store.load_latest(&compatibility) {
            Ok(loaded) => {
                let streaming = loaded
                    .world_streaming_snapshot
                    .ok_or(ApplicationError::RecoveryIncompatible)?;
                let generation_hash = save_identity(&loaded.image.manifest)?.0;
                let candidate = base.load_saved_world_candidate(
                    loaded.checkpoint,
                    streaming,
                    loaded.world_routine_snapshot_or_none,
                )?;
                let prepared = prepare_live_state(
                    self.machine.state().session_id,
                    candidate.state()?,
                    self.launch.presentation_target,
                )?;
                let plan = self.machine.plan_save_load_publication(
                    prepared.summary.authoritative_revision,
                    generation_hash,
                )?;
                self.publish_state_plan(plan)?;
                self.prepared_run = Some(prepared);
                self.live_run = Some(candidate);
            }
            Err(next_assets::SaveLoadError::NoValidGeneration { rejected })
                if rejected.is_empty() =>
            {
                self.prepared_run = Some(base_prepared);
                self.live_run = Some(base);
            }
            Err(error) => return Err(error.into()),
        }
        Ok(())
    }

    /// Latest published baseline-audio frame for the interactive live driver.
    pub fn reference_game_live_audio(&self) -> Result<ApplicationAudioFrameV1, ApplicationError> {
        let driver = self.live_run.as_ref().ok_or(ApplicationError::NoLiveRun)?;
        Ok(ApplicationAudioFrameV1 {
            audio_sequence: driver.audio_scene().snapshot_sequence,
            simulation_tick: driver.audio_scene().simulation_tick,
            pcm: driver.audio_pcm_shared(),
        })
    }

    pub fn run_reference_game(
        &mut self,
        include_interaction: bool,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active {
            return Err(ApplicationError::CloseStateInvalid);
        }
        if self.live_run.is_some() {
            return Err(ApplicationError::LiveRunAlreadyActive);
        }
        let prepared = prepare_reference_run(
            self.machine.state().session_id,
            self.activated_package(),
            include_interaction,
            self.launch.presentation_target,
        )?;
        self.publish_prepared_run(prepared)
    }

    pub fn begin_reference_game_live(
        &mut self,
        include_interaction: bool,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active {
            return Err(ApplicationError::CloseStateInvalid);
        }
        if self.live_run.is_some() || self.prepared_run.is_some() {
            return Err(ApplicationError::LiveRunAlreadyActive);
        }
        let driver = ReferenceGameDriverV2::new_with_presentation_epoch(
            self.activated_package(),
            include_interaction,
            presentation_snapshot_epoch(
                self.machine.state().session_id,
                self.activated_project.project_lock.project_lock_sha256,
            ),
        )?;
        let state = driver.state()?;
        let prepared = prepare_live_state(
            self.machine.state().session_id,
            state,
            self.launch.presentation_target,
        )?;
        let summary = self.publish_prepared_run(prepared)?;
        self.live_run = Some(driver);
        Ok(summary)
    }

    pub fn advance_reference_game_live(
        &mut self,
        platform_events: &[PlatformEventV1],
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        self.with_platform_event_admission(platform_events, platform_events, |coordinator| {
            validate_direct_live_lifecycle_batch(platform_events)?;
            coordinator.advance_reference_game_live_admitted(platform_events)
        })
    }

    pub(crate) fn advance_reference_game_live_admitted(
        &mut self,
        platform_events: &[PlatformEventV1],
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active {
            return Err(ApplicationError::CloseStateInvalid);
        }
        let (validated, state) = {
            let driver = self.live_run.as_ref().ok_or(ApplicationError::NoLiveRun)?;
            let prepared = driver.stage_advance(platform_events)?;
            let validated = driver.validate_prepared_advance(prepared)?;
            let state = driver.validated_state(&validated)?;
            (validated, state)
        };
        let prepared = prepare_live_state(
            self.machine.state().session_id,
            state,
            self.launch.presentation_target,
        )?;
        let suspend = platform_events.iter().find(|event| {
            event.kind == next_contracts::platform::PlatformEventKindV1::SuspendRequested
        });
        let ui_suspend = validated.ui_suspend_causal_hash();
        let summary = if let Some(suspend) = suspend {
            self.suspend_from_admitted_platform_event_with_prepared_run(suspend, &prepared)?;
            let summary = prepared.summary.clone();
            self.prepared_run = Some(prepared);
            summary
        } else if let Some(causal_hash) = ui_suspend {
            self.suspend_from_committed_ui_action_with_prepared_run(causal_hash, &prepared)?;
            let summary = prepared.summary.clone();
            self.prepared_run = Some(prepared);
            summary
        } else {
            let summary = prepared.summary.clone();
            self.prepared_run = Some(prepared);
            summary
        };
        let driver = self
            .live_run
            .as_mut()
            .expect("validated live advance retains its driver");
        driver.commit_validated_advance(validated)?;
        Ok(summary)
    }

    /// Advances the interactive live driver while publishing only the
    /// immutable presentation projection on ordinary ticks. A complete
    /// authoritative state is materialized only when a suspend boundary needs
    /// an in-memory continuation point; suspend does not publish a save.
    pub(crate) fn advance_reference_game_live_presentation_shared_admitted(
        &mut self,
        platform_events: &[PlatformEventV1],
    ) -> Result<Arc<next_contracts::presentation::PresentationSnapshotV2>, ApplicationError> {
        self.advance_reference_game_live_presentation_shared_observed_admitted(platform_events)
            .map(|advance| advance.presentation)
    }

    pub(crate) fn advance_reference_game_live_presentation_shared_observed_admitted(
        &mut self,
        platform_events: &[PlatformEventV1],
    ) -> Result<InteractivePresentationAdvanceV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active {
            return Err(ApplicationError::CloseStateInvalid);
        }
        let validated = {
            let driver = self.live_run.as_ref().ok_or(ApplicationError::NoLiveRun)?;
            let prepared = driver.stage_advance(platform_events)?;
            driver.validate_prepared_advance(prepared)?
        };
        let presentation = validated.presentation_snapshot_shared()?;
        let suspend = platform_events
            .iter()
            .find(|event| event.kind == PlatformEventKindV1::SuspendRequested);
        let ui_suspend = validated.ui_suspend_causal_hash();
        let checkpoint_due = suspend.is_some() || ui_suspend.is_some();

        if checkpoint_due {
            let state = self
                .live_run
                .as_ref()
                .expect("validated live advance retains its driver")
                .validated_state(&validated)?;
            let prepared = prepare_live_state(
                self.machine.state().session_id,
                state,
                self.launch.presentation_target,
            )?;
            prepared
                .summary
                .presentation_snapshot
                .as_ref()
                .ok_or(ApplicationError::NoRunOutcome)?;
            if let Some(suspend) = suspend {
                self.suspend_from_admitted_platform_event_with_prepared_run(suspend, &prepared)?;
                self.prepared_run = Some(prepared);
            } else if let Some(causal_hash) = ui_suspend {
                self.suspend_from_committed_ui_action_with_prepared_run(causal_hash, &prepared)?;
                self.prepared_run = Some(prepared);
            } else {
                self.publish_prepared_run(prepared)?;
            }
            let driver = self
                .live_run
                .as_mut()
                .expect("validated live advance retains its driver");
            driver.commit_validated_advance(validated)?;
            return Ok(InteractivePresentationAdvanceV1 {
                presentation,
                materialized_lifecycle_boundary: true,
            });
        }

        let driver = self
            .live_run
            .as_mut()
            .expect("validated live advance retains its driver");
        driver.commit_validated_advance(validated)?;
        Ok(InteractivePresentationAdvanceV1 {
            presentation,
            materialized_lifecycle_boundary: false,
        })
    }

    /// Interactive pause menu (S5): while the declared pause suspend was
    /// active, the host consumed admitted menu-key events outside the game
    /// input stream. Queues them into the live input session for cursor-only
    /// admission at the next closed frame, so per-source sequence
    /// continuity stays exact across input the game can never observe.
    pub fn queue_host_consumed_live_input(
        &mut self,
        events: &[PlatformEventV1],
    ) -> Result<(), ApplicationError> {
        let driver = self.live_run.as_mut().ok_or(ApplicationError::NoLiveRun)?;
        driver.queue_host_consumed_platform_events(events)?;
        Ok(())
    }

    pub fn current_live_run(&self) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        let driver = self.live_run.as_ref().ok_or(ApplicationError::NoLiveRun)?;
        if let Some(prepared) = self
            .prepared_run
            .as_ref()
            .filter(|prepared| prepared.summary.ticks == driver.next_tick())
        {
            return Ok(prepared.summary.clone());
        }
        Ok(prepare_live_state(
            self.machine.state().session_id,
            driver.state()?,
            self.launch.presentation_target,
        )?
        .summary)
    }

    /// Loads the latest compatible published save into the suspended live
    /// session. The complete candidate driver and recovery closure are
    /// validated before one atomic session observation publication replaces
    /// the active world; failures retain the pre-load world byte-for-byte.
    pub fn load_latest_save_into_live_run(
        &mut self,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Suspended {
            return Err(ApplicationError::CloseStateInvalid);
        }
        let current_checkpoint = &self
            .prepared_run
            .as_ref()
            .ok_or(ApplicationError::NoRunOutcome)?
            .checkpoint;
        let compatibility = save_compatibility(&self.activated_project, current_checkpoint)?;
        let loaded = self.save_store.load_latest(&compatibility)?;
        let save_generation_hash = save_identity(&loaded.image.manifest)?.0;
        let streaming = loaded
            .world_streaming_snapshot
            .ok_or(ApplicationError::RecoveryIncompatible)?;
        let candidate = self
            .live_run
            .as_ref()
            .ok_or(ApplicationError::NoLiveRun)?
            .load_saved_world_candidate(
                loaded.checkpoint,
                streaming,
                loaded.world_routine_snapshot_or_none,
            )?;
        let prepared = prepare_live_state(
            self.machine.state().session_id,
            candidate.state()?,
            self.launch.presentation_target,
        )?;
        if prepared.summary.authoritative_revision != loaded.image.manifest.world_revision {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        let summary = self.publish_loaded_save(prepared, save_generation_hash)?;
        self.live_run = Some(candidate);
        Ok(summary)
    }

    fn publish_prepared_run(
        &mut self,
        prepared: PreparedRunV1,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        let summary = prepared.summary.clone();
        self.prepared_run = Some(prepared);
        Ok(summary)
    }

    fn publish_loaded_save(
        &mut self,
        prepared: PreparedRunV1,
        save_generation_hash: ContentHash,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        let plan = self.machine.plan_save_load_publication(
            prepared.summary.authoritative_revision,
            save_generation_hash,
        )?;
        self.publish_state_plan(plan)?;
        let summary = prepared.summary.clone();
        self.prepared_run = Some(prepared);
        Ok(summary)
    }

    pub(super) fn flush_reference_game_live_checkpoint(&mut self) -> Result<(), ApplicationError> {
        let Some(driver) = self.live_run.as_ref() else {
            return Ok(());
        };
        let live_tick = driver.next_tick();
        let prepared_tick = self
            .prepared_run
            .as_ref()
            .map(|prepared| prepared.summary.ticks);
        if prepared_tick != Some(live_tick) {
            let prepared = prepare_live_state(
                self.machine.state().session_id,
                driver.state()?,
                self.launch.presentation_target,
            )?;
            self.prepared_run = Some(prepared);
            return Ok(());
        }
        Ok(())
    }

    pub(super) fn ensure_prepared_run(&mut self) -> Result<(), ApplicationError> {
        if self.prepared_run.is_some() {
            return Ok(());
        }
        let prepared = prepare_reference_run(
            self.machine.state().session_id,
            self.activated_package(),
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
        self.prepared_run = Some(prepared);
        Ok(())
    }
}

fn validate_direct_live_lifecycle_batch(
    platform_events: &[PlatformEventV1],
) -> Result<(), ApplicationError> {
    let mut lifecycle_event_id = None;
    for event in platform_events.iter().filter(|event| {
        matches!(
            event.kind,
            PlatformEventKindV1::SuspendRequested | PlatformEventKindV1::ResumeRequested
        )
    }) {
        if lifecycle_event_id.is_some_and(|event_id| event_id != event.platform_event_id) {
            return Err(ApplicationError::CloseStateInvalid);
        }
        lifecycle_event_id = Some(event.platform_event_id);
    }
    Ok(())
}

fn prepare_live_state(
    session_id: ApplicationSessionId,
    state: ReferenceLiveStateV2,
    presentation_target: PresentationTargetKindV1,
) -> Result<PreparedRunV1, ApplicationError> {
    let ReferenceLiveStateV2 {
        checkpoint,
        checkpoint_canonical_components,
        world_streaming_snapshot,
        world_routine_snapshot_or_none,
        ticks,
        events,
        rpg_events,
        project_composition_lock_hash,
        content_manifest_hash: _,
        presentation_input_count,
        presentation_snapshot,
        driver_recovery: _,
    } = state;
    let authoritative_state_root = match world_routine_snapshot_or_none.as_ref() {
        Some(routine) => next_contracts::snapshot::world_checkpoint_with_streaming_and_routine_v1_state_root_from_canonical_components(
            &checkpoint_canonical_components,
            &world_streaming_snapshot,
            routine,
        )?,
        None => next_contracts::snapshot::world_checkpoint_with_streaming_v1_state_root_from_canonical_components(
            &checkpoint_canonical_components,
            &world_streaming_snapshot,
        )?,
    };
    let summary = ApplicationRunOutcomeV1 {
        session_id,
        project_composition_lock_hash,
        ticks,
        events,
        rpg_events,
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
        command_ledger_hash: checkpoint_canonical_components.command_ledger_hash()?,
        presentation_input_count,
        presentation_snapshot: (presentation_target != PresentationTargetKindV1::None)
            .then_some(presentation_snapshot),
    };
    Ok(PreparedRunV1 {
        checkpoint,
        streaming: world_streaming_snapshot,
        routine: world_routine_snapshot_or_none,
        summary,
    })
}

fn prepare_reference_run(
    session_id: ApplicationSessionId,
    package: next_project::ActivatedProjectPackage,
    include_interaction: bool,
    presentation_target: PresentationTargetKindV1,
) -> Result<PreparedRunV1, ApplicationError> {
    let run: ReferenceRunOutcomeV2 = run_reference_game(package, include_interaction)?;
    let (checkpoint, checkpoint_canonical_components) =
        run.runtime.world_checkpoint_with_canonical_components()?;
    let presentation_snapshot = if presentation_target == PresentationTargetKindV1::None {
        None
    } else {
        let mut extractor =
            next_presentation::PresentationExtractorV1::new_with_snapshot_epoch_and_batch_limits(
                presentation_snapshot_epoch(session_id, run.project_composition_lock_hash),
                next_contracts::project::domain_hash(
                    "nextengine.presentation-profile.b0.v1",
                    b"sdr-reference-no-optional-features",
                ),
                8,
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
    let authoritative_state_root = match run.world_routine_snapshot_or_none.as_ref() {
        Some(routine) => next_contracts::snapshot::world_checkpoint_with_streaming_and_routine_v1_state_root_from_canonical_components(
            &checkpoint_canonical_components,
            &run.world_streaming_snapshot,
            routine,
        )?,
        None => next_contracts::snapshot::world_checkpoint_with_streaming_v1_state_root_from_canonical_components(
            &checkpoint_canonical_components,
            &run.world_streaming_snapshot,
        )?,
    };
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
        command_ledger_hash: checkpoint_canonical_components.command_ledger_hash()?,
        presentation_input_count,
        presentation_snapshot,
    };
    Ok(PreparedRunV1 {
        checkpoint,
        streaming: run.world_streaming_snapshot,
        routine: run.world_routine_snapshot_or_none,
        summary,
    })
}

fn presentation_snapshot_epoch(
    session_id: ApplicationSessionId,
    project_composition_lock_hash: ContentHash,
) -> ContentHash {
    let mut preimage = Vec::with_capacity(48);
    preimage.extend_from_slice(project_composition_lock_hash.as_bytes());
    preimage.extend_from_slice(session_id.as_bytes());
    next_contracts::project::domain_hash(
        "nextengine.application-presentation-snapshot-epoch.v1",
        &preimage,
    )
}
