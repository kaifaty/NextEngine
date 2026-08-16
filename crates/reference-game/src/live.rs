use std::sync::Arc;

use next_contracts::cognition::{AgentCognitionSnapshotV1, AgentMemorySnapshotV1};
use next_contracts::command::EventPayload;
use next_contracts::ids::{AssetId, ContentHash, PersistentId, SchemaId};
use next_contracts::input::{CORE_INTERACT_ACTION_ID, PlayerActionPhaseV1, PlayerActionValueV1};
use next_contracts::physical_animation::PhysicalAnimationSnapshotV1;
use next_contracts::physics::PhysicsCanonicalSnapshotV2;
use next_contracts::platform::PlatformEventV1;
use next_contracts::presentation::{
    CameraProjectionProfileV1, CameraResultSampleV1, CameraRoleV1, PresentationSnapshotV2,
    QuantizedPresentationTransformV1, ThirdPersonCameraIntentSampleV1,
};
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_contracts::world_activity::WorldActivitySnapshotV1;
use next_contracts::world_population::WorldPopulationSnapshotV1;
use next_contracts::world_routine::WorldRoutineSnapshotV1;
use next_player::PlayerInputSessionV1;
use next_presentation::audio_mix::AudioMixerV1;
use next_presentation::audio_scene::{
    AudioEventCueBindingV1, AudioListenerBindingV1, extract_audio_scene,
};
use next_presentation::{
    CameraPresentationBindingV1, PresentationBindingV1, PresentationExtractorV1,
};
use next_runtime::{
    PhysicsLaunchOptions, PreparedRuntimeWorldServicesTickV1, RuntimeState,
    ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1,
};
use next_world::{
    WorldActivityOwnerV1, WorldPopulationOwnerV1, WorldRoutineOwnerV1, WorldStreamerV1,
};

use crate::ReferenceGameError;
use crate::camera::{
    CAMERA_DISTANCE_MICROMETRES, CAMERA_SHOULDER_MICROMETRES, camera_orbit_offset_micrometres,
    update_camera_state,
};
use crate::dialogue::{ReferenceDialogueChoiceV1, ReferenceDialogueUiV1};
use crate::input::ReferenceUiScreenV1;
use crate::rpg::cooked_project_rpg_snapshot;
use crate::scenario::fixture_presentation_bindings;
use crate::session::{ReferenceGameSession, build_reference_game_session};

const CAMERA_ID_BYTES: [u8; 16] = [0xc0; 16];
const CAMERA_FOCUS_HEIGHT_MICROMETRES: i64 = 700_000;
const SEMANTIC_UI_RECORDS_PER_BATCH: usize = 64;

mod audio_ops;
mod bulk_time;
mod generation;
mod load;
mod recovery_state;
mod state;

use generation::ReferenceGameGenerationV1;

pub use bulk_time::{
    REFERENCE_BULK_TIME_MAX_TICKS_V1, ReferenceBulkTimeAdvanceV1, ReferenceBulkTimeStopReasonV1,
};
pub use state::{ReferenceLiveDriverRecoveryV1, ReferenceLiveStateV2};

#[must_use]
pub fn reference_b0_presentation_profile_hash() -> ContentHash {
    next_contracts::project::domain_hash(
        "nextengine.presentation-profile.b0.v1",
        b"sdr-reference-no-optional-features",
    )
}

pub struct ReferenceGameDriverV2 {
    fixture: ReferenceGameSession,
    content_generation: next_assets::PinnedContentGeneration,
    runtime: RuntimeState,
    physical_animation: next_motor::PhysicalAnimationOwnerV1,
    world_routine: WorldRoutineOwnerV1,
    world_population: WorldPopulationOwnerV1,
    world_activity: WorldActivityOwnerV1,
    cognition: next_agent::cognition::StrategicAgentOwnersV1,
    world_streamer: WorldStreamerV1,
    input: PlayerInputSessionV1,
    presentation_bindings: Vec<PresentationBindingV1>,
    presentation_extractor: PresentationExtractorV1,
    audio_clips: Arc<std::collections::BTreeMap<AssetId, next_contracts::audio::NeutralAudioV1>>,
    audio_cue_bindings: Arc<Vec<AudioEventCueBindingV1>>,
    audio_listener_binding: AudioListenerBindingV1,
    audio_mixer: AudioMixerV1,
    audio_scene: next_contracts::presentation::audio_scene::AudioSceneSnapshotV1,
    audio_pcm: Arc<[i16]>,
    next_audio_sequence: u64,
    audio_subtitle_or_none: Option<(SchemaId, u64)>,
    next_logical_frame_sequence: u64,
    events: u64,
    rpg_events: u64,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    camera_cut: bool,
    ui_screen: ReferenceUiScreenV1,
    dialogue: ReferenceDialogueUiV1,
    dialogue_entry_node_id: SchemaId,
}

struct PreparedReferenceGameState {
    input: PlayerInputSessionV1,
    physical_animation: next_motor::PhysicalAnimationOwnerV1,
    presentation_bindings: Vec<PresentationBindingV1>,
    presentation_extractor: PresentationExtractorV1,
    audio_mixer: AudioMixerV1,
    audio_scene: next_contracts::presentation::audio_scene::AudioSceneSnapshotV1,
    audio_pcm: Arc<[i16]>,
    next_audio_sequence: u64,
    audio_subtitle_or_none: Option<(SchemaId, u64)>,
    next_logical_frame_sequence: u64,
    events: u64,
    rpg_events: u64,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    camera_cut: bool,
    ui_suspend_causal_hash: Option<ContentHash>,
    ui_screen: ReferenceUiScreenV1,
    dialogue: ReferenceDialogueUiV1,
}

/// An isolated next reference-game generation with a read-only presentation
/// preview. The live driver has not changed.
pub struct PreparedReferenceGameAdvance {
    base_generation: ReferenceGameGenerationV1,
    runtime: PreparedRuntimeWorldServicesTickV1,
    state: PreparedReferenceGameState,
}

impl PreparedReferenceGameAdvance {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.runtime.next_tick()
    }

    pub fn presentation_snapshot(&self) -> Result<&PresentationSnapshotV2, ReferenceGameError> {
        self.state
            .presentation_extractor
            .accepted_snapshot()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)
    }

    pub fn presentation_snapshot_shared(
        &self,
    ) -> Result<Arc<PresentationSnapshotV2>, ReferenceGameError> {
        self.state
            .presentation_extractor
            .accepted_snapshot_shared()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)
    }
}

/// A prepared reference-game generation bound to the current runtime and
/// driver generation and ready for an infallible commit.
pub struct ValidatedReferenceGameAdvance {
    runtime: ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1,
    state: PreparedReferenceGameState,
}

impl ValidatedReferenceGameAdvance {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.runtime.next_tick()
    }

    /// Causal evidence hash of the committed `ui-back` press that requests
    /// the declared suspend transition on this advance, if any.
    #[must_use]
    pub const fn ui_suspend_causal_hash(&self) -> Option<ContentHash> {
        self.state.ui_suspend_causal_hash
    }

    pub fn presentation_snapshot(&self) -> Result<&PresentationSnapshotV2, ReferenceGameError> {
        self.state
            .presentation_extractor
            .accepted_snapshot()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)
    }

    pub fn presentation_snapshot_shared(
        &self,
    ) -> Result<Arc<PresentationSnapshotV2>, ReferenceGameError> {
        self.state
            .presentation_extractor
            .accepted_snapshot_shared()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)
    }
}

impl ReferenceGameDriverV2 {
    pub fn new(
        package: next_project::ActivatedProjectPackage,
        include_interaction: bool,
    ) -> Result<Self, ReferenceGameError> {
        let snapshot_epoch = next_contracts::project::domain_hash(
            "nextengine.presentation-snapshot-epoch.v1",
            package.project.project_lock.project_lock_sha256.as_bytes(),
        );
        Self::new_with_presentation_epoch(package, include_interaction, snapshot_epoch)
    }

    pub fn new_with_presentation_epoch(
        package: next_project::ActivatedProjectPackage,
        _include_interaction: bool,
        snapshot_epoch: ContentHash,
    ) -> Result<Self, ReferenceGameError> {
        let next_project::ActivatedProjectPackage {
            project: activated_project,
            content_generation,
        } = package;
        let fixture = build_reference_game_session(activated_project)?;
        let rpg_snapshot = cooked_project_rpg_snapshot(&fixture);
        let runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            rpg_snapshot,
            PhysicsLaunchOptions::default(),
        )?;
        let initial_chunk_id = load::initial_chunk_id(&fixture);
        let world_streamer = WorldStreamerV1::activate(
            fixture.activated_project.clone(),
            content_generation.clone(),
            initial_chunk_id,
        )?;
        let world_routine = WorldRoutineOwnerV1::activate(
            fixture.activated_project.world_routine_catalog_or_none,
            runtime.next_tick(),
        )?;
        let world_population = WorldPopulationOwnerV1::activate(
            fixture.activated_project.world_population_catalog.clone(),
            fixture.activated_project.world_navigation_catalog.clone(),
            runtime.next_tick(),
        )?;
        let cognition = fixture.initial_cognition_owners()?;
        let world_activity = fixture.initial_activity_owner()?;
        let input = PlayerInputSessionV1::new(
            fixture.controller_id,
            fixture.source_id,
            fixture.action_map.clone(),
            fixture.context_stack.clone(),
        )?;
        let physical_animation = crate::physical_animation::reference_physical_animation_owner(
            &fixture,
            runtime.physics_snapshot(),
        )?;
        let presentation_bindings = fixture_presentation_bindings(
            &fixture,
            &runtime.rpg_snapshot(),
            &physical_animation,
            runtime.physics_snapshot(),
        )?;
        let presentation_extractor =
            PresentationExtractorV1::new_with_snapshot_epoch_and_ui_batch_limits(
                snapshot_epoch,
                reference_b0_presentation_profile_hash(),
                8,
                1,
                SEMANTIC_UI_RECORDS_PER_BATCH,
            )?;
        let audio_clips = Arc::new(crate::audio::reference_audio_clip_map(
            &fixture.activated_project,
        ));
        let audio_cue_bindings = Arc::new(crate::audio::reference_audio_cue_bindings(
            &fixture.activated_project,
        )?);
        let audio_listener_binding = crate::audio::reference_audio_listener_binding(&fixture);
        let audio_mixer = AudioMixerV1::new(crate::audio::reference_audio_mix_profile()?);
        let audio_scene = extract_audio_scene(
            snapshot_epoch,
            0,
            runtime.next_tick(),
            &audio_listener_binding,
            &[],
            &audio_cue_bindings,
            &[],
            runtime.physics_snapshot(),
        )?;
        let mut driver = Self {
            dialogue_entry_node_id: crate::dialogue::reference_dialogue_entry_node_id(&fixture)?,
            fixture,
            content_generation,
            runtime,
            physical_animation,
            world_routine,
            world_population,
            world_activity,
            cognition,
            world_streamer,
            input,
            presentation_bindings,
            presentation_extractor,
            audio_clips,
            audio_cue_bindings,
            audio_listener_binding,
            audio_mixer,
            audio_scene,
            audio_pcm: Arc::from(Vec::new()),
            next_audio_sequence: 0,
            audio_subtitle_or_none: None,
            next_logical_frame_sequence: 0,
            events: 0,
            rpg_events: 0,
            camera_yaw_millidegrees: 0,
            camera_pitch_millidegrees: -15_000,
            camera_cut: true,
            ui_screen: ReferenceUiScreenV1::None,
            dialogue: ReferenceDialogueUiV1::Closed,
        };
        driver.publish_presentation()?;
        Ok(driver)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the public recovery boundary accepts each independently owned R4c snapshot"
    )]
    pub fn restore(
        package: next_project::ActivatedProjectPackage,
        checkpoint: WorldCheckpointV4,
        world_streaming_snapshot: WorldStreamingSnapshotV1,
        world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
        world_population_snapshot: WorldPopulationSnapshotV1,
        world_activity_snapshot: WorldActivitySnapshotV1,
        agent_cognition_snapshot: AgentCognitionSnapshotV1,
        agent_memory_snapshot: AgentMemorySnapshotV1,
        physical_animation_snapshot: PhysicalAnimationSnapshotV1,
        recovery: ReferenceLiveDriverRecoveryV1,
    ) -> Result<Self, ReferenceGameError> {
        checkpoint.validate()?;
        let next_project::ActivatedProjectPackage {
            project: activated_project,
            content_generation,
        } = package;
        let fixture = build_reference_game_session(activated_project)?;
        if recovery.next_logical_frame_sequence != checkpoint.runtime_snapshot.next_tick
            || recovery.events != checkpoint.runtime_snapshot.committed_event_count
            || recovery.rpg_events > recovery.events
            || recovery.camera_cut
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let runtime = RuntimeState::restore_world_checkpoint_with_definitions(
            checkpoint,
            fixture.authority.clone(),
            fixture.bootstrap.rpg_definitions.clone(),
        )?;
        let world_streamer = WorldStreamerV1::restore(
            fixture.activated_project.clone(),
            content_generation.clone(),
            world_streaming_snapshot,
        )?;
        let world_routine = WorldRoutineOwnerV1::restore(
            fixture.activated_project.world_routine_catalog_or_none,
            world_routine_snapshot_or_none,
            runtime.next_tick(),
        )?;
        let world_population = WorldPopulationOwnerV1::restore(
            fixture.activated_project.world_population_catalog.clone(),
            fixture.activated_project.world_navigation_catalog.clone(),
            world_population_snapshot,
            runtime.next_tick(),
        )?;
        let cognition = next_agent::cognition::StrategicAgentOwnersV1::restore(
            fixture.activated_project.agent_cognition_catalog.clone(),
            agent_cognition_snapshot,
            agent_memory_snapshot,
        )?;
        let world_activity = WorldActivityOwnerV1::restore(
            fixture.activated_project.world_activity_catalog.clone(),
            world_activity_snapshot,
            runtime.next_tick(),
        )?;
        let physical_animation =
            crate::physical_animation::restore_reference_physical_animation_owner(
                &fixture,
                physical_animation_snapshot,
                runtime.physics_snapshot(),
                runtime.next_tick(),
            )?;
        runtime.validate_world_routine_ledger_closure(&world_routine)?;
        runtime.validate_world_population_ledger_closure(&world_population)?;
        let input =
            PlayerInputSessionV1::restore_from_recovery_bytes(&recovery.input_session_bytes)?;
        let expected_last_logical_frame_sequence =
            recovery.next_logical_frame_sequence.checked_sub(1);
        if input.controller_id() != fixture.controller_id
            || input.source_id() != fixture.source_id
            || input.action_map() != &fixture.action_map
            // The persisted session may rest on a modal dialogue revision of
            // the same stack family (S4); the gameplay fixture stack is only
            // the genesis revision.
            || input.context_stack().stack_id != fixture.context_stack.stack_id
            || input.last_logical_frame_sequence() != expected_last_logical_frame_sequence
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let presentation_bindings = fixture_presentation_bindings(
            &fixture,
            &runtime.rpg_snapshot(),
            &physical_animation,
            runtime.physics_snapshot(),
        )?;
        let (presentation_extractor, persisted_snapshot) =
            PresentationExtractorV1::begin_authoritative_recovery_from_bytes(
                &recovery.presentation_snapshot_bytes,
                reference_b0_presentation_profile_hash(),
            )?;
        if persisted_snapshot.simulation_tick != runtime.next_tick()
            || persisted_snapshot.project_composition_lock_hash
                != fixture.activated_project.project_lock.project_lock_sha256
            || persisted_snapshot.content_manifest_hash
                != fixture
                    .activated_project
                    .content_manifest
                    .content_manifest_sha256
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let audio_clips = Arc::new(crate::audio::reference_audio_clip_map(
            &fixture.activated_project,
        ));
        let audio_cue_bindings = Arc::new(crate::audio::reference_audio_cue_bindings(
            &fixture.activated_project,
        )?);
        let audio_listener_binding = crate::audio::reference_audio_listener_binding(&fixture);
        // Authoritative recovery resets the presentation-only mixer: in-flight
        // one-shots may be omitted after a restart but never replayed
        // (SPEC-30 consumption recovery semantics).
        let audio_mixer = AudioMixerV1::new(crate::audio::reference_audio_mix_profile()?);
        let audio_scene = extract_audio_scene(
            presentation_extractor.snapshot_epoch(),
            0,
            runtime.next_tick(),
            &audio_listener_binding,
            &[],
            &audio_cue_bindings,
            &[],
            runtime.physics_snapshot(),
        )?;
        let mut driver = Self {
            dialogue_entry_node_id: crate::dialogue::reference_dialogue_entry_node_id(&fixture)?,
            fixture,
            content_generation,
            runtime,
            physical_animation,
            world_routine,
            world_population,
            world_activity,
            cognition,
            world_streamer,
            input,
            presentation_bindings,
            presentation_extractor,
            audio_clips,
            audio_cue_bindings,
            audio_listener_binding,
            audio_mixer,
            audio_scene,
            audio_pcm: Arc::from(Vec::new()),
            next_audio_sequence: 0,
            audio_subtitle_or_none: None,
            next_logical_frame_sequence: recovery.next_logical_frame_sequence,
            events: recovery.events,
            rpg_events: recovery.rpg_events,
            camera_yaw_millidegrees: recovery.camera_yaw_millidegrees,
            camera_pitch_millidegrees: recovery.camera_pitch_millidegrees,
            camera_cut: true,
            ui_screen: recovery.ui_screen,
            dialogue: recovery.dialogue,
        };
        driver.validate_recovered_camera(&persisted_snapshot)?;
        if driver.input.recovery_bytes()? != recovery.input_session_bytes {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        driver.publish_presentation()?;
        let recovered_snapshot = driver
            .presentation_extractor
            .accepted_snapshot()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)?;
        if recovered_snapshot.snapshot_epoch == persisted_snapshot.snapshot_epoch
            || recovered_snapshot.snapshot_sequence != 0
            || recovered_snapshot.presentation_profile_hash
                != reference_b0_presentation_profile_hash()
            || recovered_snapshot
                .camera_records()
                .any(|camera| !camera.cut)
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        Ok(driver)
    }

    pub fn advance(
        &mut self,
        platform_events: &[PlatformEventV1],
    ) -> Result<&PresentationSnapshotV2, ReferenceGameError> {
        let prepared = self.stage_advance(platform_events)?;
        let validated = self.validate_prepared_advance(prepared)?;
        self.commit_validated_advance(validated)
    }

    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.runtime.next_tick()
    }

    pub fn presentation_snapshot(&self) -> Result<&PresentationSnapshotV2, ReferenceGameError> {
        self.presentation_extractor
            .accepted_snapshot()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)
    }

    /// Produces one complete next live generation without changing the current
    /// driver. Only mutable input/presentation state is cloned; immutable
    /// fixture, streaming and binding state stays shared through this driver.
    pub fn stage_advance(
        &self,
        platform_events: &[PlatformEventV1],
    ) -> Result<PreparedReferenceGameAdvance, ReferenceGameError> {
        let base_generation = ReferenceGameGenerationV1::capture(self)?;
        let mut input_session = self.input.clone();
        let mut physical_animation = self.physical_animation.clone();
        let mut presentation_extractor = self.presentation_extractor.clone();
        let mut audio_mixer = self.audio_mixer.clone();
        let mut camera_yaw_millidegrees = self.camera_yaw_millidegrees;
        let mut camera_pitch_millidegrees = self.camera_pitch_millidegrees;
        let mut ui_screen = self.ui_screen;
        let mut dialogue = self.dialogue;

        // S4: keep the session context stack on the layer the dialogue state
        // requires. The queued revision applies at this advance's close_frame,
        // so at rest the session never carries a pending configuration and
        // recovery bytes stay canonical.
        crate::dialogue::sync_context_stack(&mut input_session, dialogue)?;

        input_session.submit_platform_events(platform_events)?;
        let input = input_session.close_frame(self.next_logical_frame_sequence)?;
        let pending_input_configuration = if input.configuration_changed {
            Some((
                input_session.action_map().clone(),
                input_session.context_stack().clone(),
            ))
        } else {
            None
        };
        let next_logical_frame_sequence = self
            .next_logical_frame_sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let mut runtime_preparation = self.runtime.tick_preparation();
        let mut ui_suspend_causal_hash = None;
        let mut strip_interaction_movement = false;
        let mut inject_interact = false;
        if let Some(resolved) = &input.resolved {
            update_camera_state(
                &resolved.frame,
                &mut camera_yaw_millidegrees,
                &mut camera_pitch_millidegrees,
            );
            let screen_outcome = crate::input::apply_ui_screen_actions(&resolved.frame, ui_screen);
            ui_screen = screen_outcome.screen;
            let dialogue_outcome =
                crate::dialogue::apply_dialogue_frame_actions(&resolved.frame, dialogue);
            dialogue = dialogue_outcome.state;
            // A committed `interact` press that targets the reference NPC opens
            // the dialogue instead of reaching the runtime: the press and the
            // frame's movement are consumed by the modal (S4, Q2A).
            if dialogue == ReferenceDialogueUiV1::Closed
                && resolved.frame.actions.iter().any(|action| {
                    action.action_id.as_str() == CORE_INTERACT_ACTION_ID
                        && action.phase == PlayerActionPhaseV1::Started
                        && action.value == PlayerActionValueV1::Digital(true)
                })
                && self
                    .runtime
                    .interaction_availability(
                        &crate::dialogue::reference_dialogue_interaction_id(&self.fixture)?,
                        &self.world_routine,
                    )?
                    .code
                    == next_contracts::world_routine::InteractionAvailabilityCodeV1::Available
                && crate::dialogue::dialogue_open_available(
                    &self.runtime.rpg_snapshot(),
                    self.runtime.physics_snapshot(),
                    &self.fixture,
                    &self.dialogue_entry_node_id,
                )?
            {
                dialogue = ReferenceDialogueUiV1::Open {
                    selection: ReferenceDialogueChoiceV1::Accept,
                };
                ui_screen = crate::dialogue::screen_for_dialogue(ui_screen, true);
                strip_interaction_movement = true;
            }
            // A committed `ui-back` that closes an open screen or the dialogue
            // is consumed by the presentation state and never reaches the
            // pause suspend request.
            if !screen_outcome.back_consumed_by_screen
                && !dialogue_outcome.back_consumed_by_dialogue
            {
                ui_suspend_causal_hash = crate::input::ui_suspend_causal_hash(&resolved.frame)?;
            }
        }
        // The accepted choice delivers through the production interaction path
        // on the first frame resolved again under the gameplay stack, whether
        // or not that frame carried physical input (S4, Q2A).
        if dialogue == ReferenceDialogueUiV1::AcceptPending
            && !crate::dialogue::stack_has_dialogue_layer(self.input.context_stack())
        {
            dialogue = ReferenceDialogueUiV1::Closed;
            inject_interact = true;
        }
        if let Some(sample) = crate::dialogue::dialogue_runtime_sample(
            input.resolved.as_ref(),
            &self.input,
            self.next_logical_frame_sequence,
            strip_interaction_movement,
            inject_interact,
        )? {
            runtime_preparation.enqueue_input_sample(&self.fixture.principal, sample)?;
        }
        let mut prepared_runtime = runtime_preparation
            .prepare_with_world_services_cognition_and_activity(
                [],
                &self.world_routine,
                &self.world_population,
                &self.world_activity,
                &self.cognition,
                &self.world_streamer,
            )?;
        if let Some((action_map, context_stack)) = pending_input_configuration {
            prepared_runtime = self
                .runtime
                .stage_player_input_configuration_activation_world_services(
                    prepared_runtime,
                    self.fixture.source_id,
                    action_map,
                    context_stack,
                )?;
        }
        physical_animation.advance(
            self.runtime.physics_snapshot(),
            prepared_runtime.physics_snapshot(),
            prepared_runtime.next_tick(),
        )?;
        let events = self
            .events
            .checked_add(
                u64::try_from(prepared_runtime.events().len())
                    .map_err(|_| ReferenceGameError::CountOverflow)?,
            )
            .ok_or(ReferenceGameError::CountOverflow)?;
        let rpg_events = self
            .rpg_events
            .checked_add(
                u64::try_from(
                    prepared_runtime
                        .events()
                        .iter()
                        .filter(|event| matches!(&event.payload, EventPayload::Rpg(_)))
                        .count(),
                )
                .map_err(|_| ReferenceGameError::CountOverflow)?,
            )
            .ok_or(ReferenceGameError::CountOverflow)?;
        let camera = self.camera_binding_for(
            prepared_runtime.physics_snapshot(),
            camera_yaw_millidegrees,
            camera_pitch_millidegrees,
            self.camera_cut,
        )?;
        let ui_records = crate::ui::live_semantic_ui_records(
            presentation_extractor.snapshot_epoch(),
            &self.fixture,
            &prepared_runtime.rpg_snapshot(),
            ui_screen,
            dialogue,
            ui_suspend_causal_hash,
            self.current_audio_subtitle(prepared_runtime.next_tick()),
        )?;
        let presentation_bindings = fixture_presentation_bindings(
            &self.fixture,
            &prepared_runtime.rpg_snapshot(),
            &physical_animation,
            prepared_runtime.physics_snapshot(),
        )?;
        presentation_extractor.extract_with_cameras_and_semantic_ui(
            prepared_runtime.next_tick(),
            self.fixture
                .activated_project
                .project_lock
                .project_lock_sha256,
            self.fixture
                .activated_project
                .content_manifest
                .content_manifest_sha256,
            prepared_runtime.physics_snapshot(),
            &presentation_bindings,
            &[camera],
            ui_records,
        )?;
        let audio_scene = extract_audio_scene(
            presentation_extractor.snapshot_epoch(),
            self.next_audio_sequence,
            prepared_runtime.next_tick(),
            &self.audio_listener_binding,
            &[],
            &self.audio_cue_bindings,
            prepared_runtime.events(),
            prepared_runtime.physics_snapshot(),
        )?;
        let audio_pcm: Arc<[i16]> =
            Arc::from(audio_mixer.mix_tick(&audio_scene, &self.audio_clips));
        let next_audio_sequence = self
            .next_audio_sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let audio_subtitle_or_none = crate::audio::active_subtitle_text_id(
            &audio_scene,
            &self.audio_cue_bindings,
        )
        .map_or(self.audio_subtitle_or_none.clone(), |text_id| {
            Some((
                text_id,
                prepared_runtime
                    .next_tick()
                    .saturating_add(crate::audio::REFERENCE_SUBTITLE_WINDOW_TICKS),
            ))
        });
        if presentation_extractor.accepted_snapshot().is_none() {
            return Err(ReferenceGameError::PresentationSnapshotMissing);
        }

        Ok(PreparedReferenceGameAdvance {
            base_generation,
            runtime: prepared_runtime,
            state: PreparedReferenceGameState {
                input: input_session,
                physical_animation,
                presentation_bindings,
                presentation_extractor,
                audio_mixer,
                audio_scene,
                audio_pcm,
                next_audio_sequence,
                audio_subtitle_or_none,
                next_logical_frame_sequence,
                events,
                rpg_events,
                camera_yaw_millidegrees,
                camera_pitch_millidegrees,
                camera_cut: false,
                ui_suspend_causal_hash,
                ui_screen,
                dialogue,
            },
        })
    }

    pub fn validate_prepared_advance(
        &self,
        prepared: PreparedReferenceGameAdvance,
    ) -> Result<ValidatedReferenceGameAdvance, ReferenceGameError> {
        if !prepared.base_generation.matches(self) {
            return Err(next_runtime::RuntimeFatalError::PreparedGenerationStale.into());
        }
        let runtime = self
            .runtime
            .validate_prepared_world_services_tick_with_cognition_and_activity_without_application_evidence(
                &self.world_routine,
                &self.world_population,
                &self.world_activity,
                &self.cognition,
                &self.world_streamer,
                prepared.runtime,
            )?;
        Ok(ValidatedReferenceGameAdvance {
            runtime,
            state: prepared.state,
        })
    }

    /// Commits one validated advance. Any input configuration revision applied
    /// by `close_frame` is already part of the validated runtime generation, so
    /// controller registry, authoritative tick and presentation state publish
    /// together without a fallible operation after the first live mutation.
    pub fn commit_validated_advance(
        &mut self,
        validated: ValidatedReferenceGameAdvance,
    ) -> Result<&PresentationSnapshotV2, ReferenceGameError> {
        self.runtime
            .commit_validated_world_services_tick_with_cognition_and_activity_without_application_evidence(
                &mut self.world_routine,
                &mut self.world_population,
                &mut self.world_activity,
                &mut self.cognition,
                &mut self.world_streamer,
                validated.runtime,
            )?;
        self.input = validated.state.input;
        self.physical_animation = validated.state.physical_animation;
        self.presentation_bindings = validated.state.presentation_bindings;
        self.presentation_extractor = validated.state.presentation_extractor;
        self.audio_mixer = validated.state.audio_mixer;
        self.audio_scene = validated.state.audio_scene;
        self.audio_pcm = validated.state.audio_pcm;
        self.next_audio_sequence = validated.state.next_audio_sequence;
        self.audio_subtitle_or_none = validated.state.audio_subtitle_or_none;
        self.next_logical_frame_sequence = validated.state.next_logical_frame_sequence;
        self.events = validated.state.events;
        self.rpg_events = validated.state.rpg_events;
        self.camera_yaw_millidegrees = validated.state.camera_yaw_millidegrees;
        self.camera_pitch_millidegrees = validated.state.camera_pitch_millidegrees;
        self.camera_cut = validated.state.camera_cut;
        self.ui_screen = validated.state.ui_screen;
        self.dialogue = validated.state.dialogue;
        Ok(self
            .presentation_extractor
            .accepted_snapshot()
            .expect("validated reference advance contains a presentation snapshot"))
    }

    pub fn cancel_recovered_controls(&mut self) -> Result<(), ReferenceGameError> {
        self.input.cancel_recovered_controls()?;
        Ok(())
    }

    /// Interactive pause menu (S5): queues host-consumed platform events
    /// (menu keys swallowed while the session was suspended) for cursor-only
    /// admission at the next closed input frame, keeping per-source sequence
    /// continuity exact. No control or action effects.
    pub fn queue_host_consumed_platform_events(
        &mut self,
        events: &[PlatformEventV1],
    ) -> Result<(), ReferenceGameError> {
        self.input.queue_host_consumed_platform_events(events)?;
        Ok(())
    }

    fn validate_recovered_camera(
        &self,
        persisted_snapshot: &PresentationSnapshotV2,
    ) -> Result<(), ReferenceGameError> {
        let persisted = {
            let mut cameras = persisted_snapshot.camera_records();
            let camera = cameras.next();
            camera
                .and_then(|camera| cameras.next().is_none().then_some(camera))
                .ok_or(ReferenceGameError::RecoveryInvalid)?
                .clone()
        };
        let rebuilt = self.camera_binding()?;
        if persisted.camera_id != PersistentId::from_bytes(CAMERA_ID_BYTES)
            || persisted.camera_role != CameraRoleV1::PrimaryThirdPerson
            || persisted.camera_id != rebuilt.camera_id
            || persisted.viewport != rebuilt.viewport
            || persisted.projection_profile != rebuilt.projection_profile
            || persisted.intent_sample != rebuilt.intent_sample
            || persisted.current_result_sample != rebuilt.current_result_sample
            || persisted.exposure_profile_revision != rebuilt.exposure_profile_revision
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        Ok(())
    }

    fn publish_presentation(&mut self) -> Result<&PresentationSnapshotV2, ReferenceGameError> {
        let camera = self.camera_binding()?;
        let ui_records = crate::ui::live_semantic_ui_records(
            self.presentation_extractor.snapshot_epoch(),
            &self.fixture,
            &self.runtime.rpg_snapshot(),
            self.ui_screen,
            self.dialogue,
            None,
            self.current_audio_subtitle(self.runtime.next_tick()),
        )?;
        self.presentation_extractor
            .extract_with_cameras_and_semantic_ui(
                self.runtime.next_tick(),
                self.fixture
                    .activated_project
                    .project_lock
                    .project_lock_sha256,
                self.fixture
                    .activated_project
                    .content_manifest
                    .content_manifest_sha256,
                self.runtime.physics_snapshot(),
                &self.presentation_bindings,
                &[camera],
                ui_records,
            )?;
        self.publish_audio(&[], self.runtime.next_tick())?;
        self.camera_cut = false;
        self.presentation_extractor
            .accepted_snapshot()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)
    }

    fn camera_binding(&self) -> Result<CameraPresentationBindingV1, ReferenceGameError> {
        self.camera_binding_for(
            self.runtime.physics_snapshot(),
            self.camera_yaw_millidegrees,
            self.camera_pitch_millidegrees,
            self.camera_cut,
        )
    }

    fn camera_binding_for(
        &self,
        physics_snapshot: &PhysicsCanonicalSnapshotV2,
        camera_yaw_millidegrees: i32,
        camera_pitch_millidegrees: i32,
        camera_cut: bool,
    ) -> Result<CameraPresentationBindingV1, ReferenceGameError> {
        let player = self.fixture.physics_body_id;
        let player = physics_snapshot
            .sorted_body_states
            .get(&player)
            .ok_or(ReferenceGameError::BodyMissing)?;
        let mut focus = player.pose.translation_micrometres;
        focus[1] = focus[1]
            .checked_add(CAMERA_FOCUS_HEIGHT_MICROMETRES)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let offset =
            camera_orbit_offset_micrometres(camera_yaw_millidegrees, camera_pitch_millidegrees)?;
        let translation = [
            focus[0]
                .checked_add(offset[0])
                .ok_or(ReferenceGameError::CountOverflow)?,
            focus[1]
                .checked_add(offset[1])
                .ok_or(ReferenceGameError::CountOverflow)?,
            focus[2]
                .checked_add(offset[2])
                .ok_or(ReferenceGameError::CountOverflow)?,
        ];
        Ok(CameraPresentationBindingV1::primary_third_person(
            PersistentId::from_bytes(CAMERA_ID_BYTES),
            0,
            CameraProjectionProfileV1::new(60_000, 100_000, 100_000_000)?,
            ThirdPersonCameraIntentSampleV1 {
                focus_subject_id: Some(self.fixture.body_id),
                focus_point_micrometres: focus,
                orbit_yaw_millidegrees: camera_yaw_millidegrees,
                orbit_pitch_millidegrees: camera_pitch_millidegrees,
                distance_micrometres: CAMERA_DISTANCE_MICROMETRES,
                shoulder_offset_micrometres: [CAMERA_SHOULDER_MICROMETRES, 0, 0],
            },
            CameraResultSampleV1 {
                pose: QuantizedPresentationTransformV1 {
                    translation_micrometres: translation,
                    ..QuantizedPresentationTransformV1::default()
                },
                focus_point_micrometres: focus,
            },
            self.fixture
                .activated_project
                .render_content_catalog
                .profile_revision(),
            camera_cut,
        ))
    }
}
