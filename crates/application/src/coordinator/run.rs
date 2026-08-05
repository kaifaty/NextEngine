use next_assets::SessionObjectV1;
use next_contracts::ids::{ApplicationSessionId, ContentHash};
use next_contracts::platform::{PlatformEventKindV1, PlatformEventV1};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{ApplicationSessionStatusV1, PresentationTargetKindV1};
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_reference_game::{
    ReferenceGameDriverV1, ReferenceLiveDriverRecoveryV1, ReferenceLiveStateV1,
    ReferenceRunOutcomeV1, run_reference_game,
};
use std::sync::Arc;

use crate::ApplicationError;

use super::{ApplicationCoordinator, ApplicationRunOutcomeV1};

mod live_recovery;

use live_recovery::live_run_object_closure;
#[cfg(test)]
pub(super) use live_recovery::{
    live_run_evidence_payload_hashes, replace_live_run_evidence_payload,
};
pub(super) use live_recovery::{restore_live_run, validate_live_run_evidence_closure};

const LIVE_CHECKPOINT_INTERVAL_TICKS: u64 = 30;

#[derive(Clone)]
pub(super) struct PreparedRunV1 {
    pub(super) checkpoint: WorldCheckpointV4,
    pub(super) checkpoint_canonical_components:
        next_contracts::snapshot::WorldCheckpointCanonicalComponentsV1,
    pub(super) streaming: WorldStreamingSnapshotV1,
    pub(super) summary: ApplicationRunOutcomeV1,
    pub(super) driver_recovery: Option<ReferenceLiveDriverRecoveryV1>,
}

pub(crate) struct InteractivePresentationAdvanceV1 {
    pub(crate) presentation: Arc<next_contracts::presentation::PresentationSnapshotV2>,
    pub(crate) published_checkpoint: bool,
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
            self.activated_project.clone(),
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
        let driver = ReferenceGameDriverV1::new_with_presentation_epoch(
            self.activated_project.clone(),
            include_interaction,
            presentation_snapshot_epoch(
                self.machine.state().session_id,
                self.activated_project
                    .composition_lock
                    .composition_lock_sha256,
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
        } else if prepared
            .summary
            .ticks
            .is_multiple_of(LIVE_CHECKPOINT_INTERVAL_TICKS)
        {
            self.publish_prepared_run(prepared)?
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
    /// immutable presentation projection on ordinary ticks. Complete
    /// authoritative checkpoints remain forced at the declared cadence and
    /// lifecycle boundaries.
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
        let checkpoint_due = validated
            .next_tick()
            .is_multiple_of(LIVE_CHECKPOINT_INTERVAL_TICKS)
            || suspend.is_some()
            || ui_suspend.is_some();

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
                published_checkpoint: true,
            });
        }

        let driver = self
            .live_run
            .as_mut()
            .expect("validated live advance retains its driver");
        driver.commit_validated_advance(validated)?;
        Ok(InteractivePresentationAdvanceV1 {
            presentation,
            published_checkpoint: false,
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

    pub(super) fn record_prepared_run(
        &mut self,
        prepared: &PreparedRunV1,
    ) -> Result<(), ApplicationError> {
        if prepared.driver_recovery.is_some() {
            let closure = live_run_object_closure(self.machine.state().session_id, prepared)?;
            self.prepared_run_objects = closure.objects;
            self.durable.live_run_recovery_manifest_hash = Some(closure.manifest_hash);
            return Ok(());
        }
        let object_bytes = [
            prepared
                .checkpoint_canonical_components
                .runtime_snapshot_shared_bytes(),
            prepared
                .checkpoint_canonical_components
                .rpg_snapshot_shared_bytes(),
            prepared
                .checkpoint_canonical_components
                .physics_checkpoint_shared_bytes(),
            Arc::from(prepared.streaming.canonical_bytes()?),
        ];
        self.replace_prepared_run_object_bytes(object_bytes);
        self.durable.live_run_recovery_manifest_hash = None;
        Ok(())
    }

    pub(super) fn replace_prepared_run_object_bytes<B>(&mut self, object_bytes: [B; 4])
    where
        B: Into<Arc<[u8]>>,
    {
        self.prepared_run_objects = object_bytes
            .into_iter()
            .map(SessionObjectV1::new)
            .map(|object| (object.content_hash(), object.into_shared_bytes()))
            .collect();
    }

    fn publish_prepared_run(
        &mut self,
        prepared: PreparedRunV1,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        let prior_prepared_run_objects = self.prepared_run_objects.clone();
        let prior_recovery_manifest_hash = self.durable.live_run_recovery_manifest_hash;
        self.record_prepared_run(&prepared)?;
        let publication = (|| {
            let plan = self
                .machine
                .plan_state_publication(Some(prepared.summary.authoritative_revision), None)?;
            self.publish_state_plan(plan)
        })();
        if let Err(error) = publication {
            self.prepared_run_objects = prior_prepared_run_objects;
            self.durable.live_run_recovery_manifest_hash = prior_recovery_manifest_hash;
            return Err(error);
        }
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
            self.publish_prepared_run(prepared)?;
            return Ok(());
        }
        let prepared = self
            .prepared_run
            .as_ref()
            .expect("matching prepared tick exists");
        if self.machine.state().active_runtime_revision
            == Some(prepared.summary.authoritative_revision)
        {
            return Ok(());
        }
        self.publish_prepared_run(prepared.clone())?;
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
    state: ReferenceLiveStateV1,
    presentation_target: PresentationTargetKindV1,
) -> Result<PreparedRunV1, ApplicationError> {
    let ReferenceLiveStateV1 {
        checkpoint,
        checkpoint_canonical_components,
        world_streaming_snapshot,
        ticks,
        events,
        rpg_events,
        project_composition_lock_hash,
        content_manifest_hash: _,
        presentation_input_count,
        presentation_snapshot,
        driver_recovery,
    } = state;
    let authoritative_state_root =
        next_contracts::snapshot::world_checkpoint_with_streaming_v1_state_root_from_canonical_components(
            &checkpoint_canonical_components,
            &world_streaming_snapshot,
        )?;
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
        checkpoint_canonical_components,
        streaming: world_streaming_snapshot,
        summary,
        driver_recovery: Some(driver_recovery),
    })
}

fn prepare_reference_run(
    session_id: ApplicationSessionId,
    project: ActivatedProjectV2,
    include_interaction: bool,
    presentation_target: PresentationTargetKindV1,
) -> Result<PreparedRunV1, ApplicationError> {
    let run: ReferenceRunOutcomeV1 = run_reference_game(project, include_interaction)?;
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
    let authoritative_state_root =
        next_contracts::snapshot::world_checkpoint_with_streaming_v1_state_root_from_canonical_components(
            &checkpoint_canonical_components,
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
        command_ledger_hash: checkpoint_canonical_components.command_ledger_hash()?,
        presentation_input_count,
        presentation_snapshot,
    };
    Ok(PreparedRunV1 {
        checkpoint,
        checkpoint_canonical_components,
        streaming: run.world_streaming_snapshot,
        summary,
        driver_recovery: None,
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
