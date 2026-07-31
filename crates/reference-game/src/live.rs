use next_contracts::command::EventPayload;
use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::input::{
    CORE_CAMERA_ORBIT_ACTION_ID, PlayerActionFrameV1, PlayerActionValueV1,
};
use next_contracts::physics::PhysicsCanonicalSnapshotV2;
use next_contracts::platform::PlatformEventV1;
use next_contracts::presentation::{
    CameraProjectionProfileV1, CameraResultSampleV1, CameraRoleV1, PresentationSnapshotV2,
    QuantizedPresentationTransformV1, ThirdPersonCameraIntentSampleV1,
};
use next_contracts::snapshot::{WorldCheckpointCanonicalComponentsV1, WorldCheckpointV4};
use next_contracts::world::WorldStreamingSnapshotV1;
use next_player::PlayerInputSessionV1;
use next_presentation::{
    CameraPresentationBindingV1, PresentationBindingV1, PresentationExtractorV1,
};
use next_runtime::{PhysicsLaunchOptions, PreparedRuntimeTick, RuntimeState, ValidatedRuntimeTick};
use next_world::WorldStreamerV1;

use crate::ReferenceGameError;
use crate::rpg::cooked_project_rpg_snapshot;
use crate::scenario::fixture_presentation_bindings;
use crate::session::{ReferenceGameSession, build_reference_game_session};

const CAMERA_ID_BYTES: [u8; 16] = [0xc0; 16];
const CAMERA_DISTANCE_MICROMETRES: u64 = 4_000_000;
const CAMERA_FOCUS_HEIGHT_MICROMETRES: i64 = 700_000;
const CAMERA_SHOULDER_MICROMETRES: i64 = 350_000;
const CAMERA_MOUSE_MILLIDEGREES_PER_UNIT: i32 = 120;
const CAMERA_TRIG_FRACTIONAL_BITS: u32 = 52;
const CAMERA_TRIG_ONE_Q52: i64 = 1_i64 << CAMERA_TRIG_FRACTIONAL_BITS;
const CAMERA_CORDIC_GAIN_INVERSE_Q52: i64 = 2_734_824_091_825_638;
// Pinned deterministic CORDIC profile: atan(2^-i) uses Q62 turns while the
// rotated sine/cosine vector uses Q52. These integers are canonical inputs.
const CAMERA_CORDIC_ATAN_TURN_Q62: [i64; 53] = [
    576_460_752_303_423_488,
    340_304_653_033_718_272,
    179_807_632_645_220_256,
    91_273_161_881_380_496,
    45_813_697_873_323_712,
    22_929_182_573_009_056,
    11_467_389_120_678_284,
    5_734_044_481_687_724,
    2_867_065_987_018_958,
    1_433_538_461_969_102,
    716_769_914_547_871,
    358_385_042_719_534,
    179_192_532_040_472,
    89_596_267_355_325,
    44_798_133_844_548,
    22_399_066_943_135,
    11_199_533_474_175,
    5_599_766_737_413,
    2_799_883_368_747,
    1_399_941_684_379,
    699_970_842_190,
    349_985_421_095,
    174_992_710_548,
    87_496_355_274,
    43_748_177_637,
    21_874_088_818,
    10_937_044_409,
    5_468_522_205,
    2_734_261_102,
    1_367_130_551,
    683_565_276,
    341_782_638,
    170_891_319,
    85_445_659,
    42_722_830,
    21_361_415,
    10_680_707,
    5_340_354,
    2_670_177,
    1_335_088,
    667_544,
    333_772,
    166_886,
    83_443,
    41_722,
    20_861,
    10_430,
    5_215,
    2_608,
    1_304,
    652,
    326,
    163,
];

#[must_use]
pub fn reference_b0_presentation_profile_hash() -> ContentHash {
    next_contracts::project::domain_hash(
        "nextengine.presentation-profile.b0.v1",
        b"sdr-reference-no-optional-features",
    )
}

pub struct ReferenceLiveStateV1 {
    pub checkpoint: WorldCheckpointV4,
    pub checkpoint_canonical_components: WorldCheckpointCanonicalComponentsV1,
    pub world_streaming_snapshot: WorldStreamingSnapshotV1,
    pub ticks: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub project_composition_lock_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub presentation_input_count: u64,
    pub presentation_snapshot: PresentationSnapshotV2,
    pub driver_recovery: ReferenceLiveDriverRecoveryV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceLiveDriverRecoveryV1 {
    pub next_logical_frame_sequence: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub camera_yaw_millidegrees: i32,
    pub camera_pitch_millidegrees: i32,
    pub camera_cut: bool,
    pub input_session_bytes: Vec<u8>,
    pub presentation_snapshot_bytes: Vec<u8>,
}

pub struct ReferenceGameDriverV1 {
    fixture: ReferenceGameSession,
    runtime: RuntimeState,
    world_streamer: WorldStreamerV1,
    input: PlayerInputSessionV1,
    presentation_bindings: Vec<PresentationBindingV1>,
    presentation_extractor: PresentationExtractorV1,
    next_logical_frame_sequence: u64,
    events: u64,
    rpg_events: u64,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    camera_cut: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReferenceGameGenerationV1 {
    next_logical_frame_sequence: u64,
    events: u64,
    rpg_events: u64,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    camera_cut: bool,
    input_last_logical_frame_sequence: Option<u64>,
    presentation_snapshot_sequence: u64,
    presentation_simulation_tick: u64,
}

impl ReferenceGameGenerationV1 {
    fn capture(driver: &ReferenceGameDriverV1) -> Result<Self, ReferenceGameError> {
        let presentation = driver.presentation_snapshot()?;
        Ok(Self {
            next_logical_frame_sequence: driver.next_logical_frame_sequence,
            events: driver.events,
            rpg_events: driver.rpg_events,
            camera_yaw_millidegrees: driver.camera_yaw_millidegrees,
            camera_pitch_millidegrees: driver.camera_pitch_millidegrees,
            camera_cut: driver.camera_cut,
            input_last_logical_frame_sequence: driver.input.last_logical_frame_sequence(),
            presentation_snapshot_sequence: presentation.snapshot_sequence,
            presentation_simulation_tick: presentation.simulation_tick,
        })
    }

    fn matches(&self, driver: &ReferenceGameDriverV1) -> bool {
        let Some(presentation) = driver.presentation_extractor.accepted_snapshot() else {
            return false;
        };
        self.next_logical_frame_sequence == driver.next_logical_frame_sequence
            && self.events == driver.events
            && self.rpg_events == driver.rpg_events
            && self.camera_yaw_millidegrees == driver.camera_yaw_millidegrees
            && self.camera_pitch_millidegrees == driver.camera_pitch_millidegrees
            && self.camera_cut == driver.camera_cut
            && self.input_last_logical_frame_sequence == driver.input.last_logical_frame_sequence()
            && self.presentation_snapshot_sequence == presentation.snapshot_sequence
            && self.presentation_simulation_tick == presentation.simulation_tick
    }
}

struct PreparedReferenceGameState {
    input: PlayerInputSessionV1,
    presentation_extractor: PresentationExtractorV1,
    next_logical_frame_sequence: u64,
    events: u64,
    rpg_events: u64,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    camera_cut: bool,
}

/// An isolated next reference-game generation with a read-only presentation
/// preview. The live driver has not changed.
pub struct PreparedReferenceGameAdvance {
    base_generation: ReferenceGameGenerationV1,
    runtime: PreparedRuntimeTick,
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
}

/// A prepared reference-game generation bound to the current runtime and
/// driver generation and ready for an infallible commit.
pub struct ValidatedReferenceGameAdvance {
    runtime: ValidatedRuntimeTick,
    state: PreparedReferenceGameState,
}

impl ValidatedReferenceGameAdvance {
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
}

impl ReferenceGameDriverV1 {
    pub fn new(
        activated_project: next_contracts::project::ActivatedProjectV2,
        include_interaction: bool,
    ) -> Result<Self, ReferenceGameError> {
        let snapshot_epoch = next_contracts::project::domain_hash(
            "nextengine.presentation-snapshot-epoch.v1",
            activated_project
                .composition_lock
                .composition_lock_sha256
                .as_bytes(),
        );
        Self::new_with_presentation_epoch(activated_project, include_interaction, snapshot_epoch)
    }

    pub fn new_with_presentation_epoch(
        activated_project: next_contracts::project::ActivatedProjectV2,
        include_interaction: bool,
        snapshot_epoch: ContentHash,
    ) -> Result<Self, ReferenceGameError> {
        let fixture = build_reference_game_session(activated_project)?;
        let rpg_snapshot = if include_interaction {
            cooked_project_rpg_snapshot(&fixture)
        } else {
            next_contracts::rpg::RpgSnapshotV2::default()
        };
        let runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            rpg_snapshot,
            PhysicsLaunchOptions::default(),
        )?;
        let initial_chunk_id = fixture
            .activated_project
            .world_partition
            .body
            .chunk_bindings
            .first()
            .ok_or(ReferenceGameError::WorldPartitionEmpty)?
            .chunk_id
            .clone();
        let world_streamer =
            WorldStreamerV1::activate(fixture.activated_project.clone(), initial_chunk_id)?;
        let input = PlayerInputSessionV1::new(
            fixture.controller_id,
            fixture.source_id,
            fixture.action_map.clone(),
            fixture.context_stack.clone(),
        )?;
        let presentation_bindings = fixture_presentation_bindings(&fixture)?;
        let presentation_extractor =
            PresentationExtractorV1::new_with_snapshot_epoch_and_batch_limits(
                snapshot_epoch,
                reference_b0_presentation_profile_hash(),
                8,
                1,
            )?;
        let mut driver = Self {
            fixture,
            runtime,
            world_streamer,
            input,
            presentation_bindings,
            presentation_extractor,
            next_logical_frame_sequence: 0,
            events: 0,
            rpg_events: 0,
            camera_yaw_millidegrees: 0,
            camera_pitch_millidegrees: -15_000,
            camera_cut: true,
        };
        driver.publish_presentation()?;
        Ok(driver)
    }

    pub fn restore(
        activated_project: next_contracts::project::ActivatedProjectV2,
        checkpoint: WorldCheckpointV4,
        world_streaming_snapshot: WorldStreamingSnapshotV1,
        recovery: ReferenceLiveDriverRecoveryV1,
    ) -> Result<Self, ReferenceGameError> {
        checkpoint.validate()?;
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
        let world_streamer =
            WorldStreamerV1::restore(fixture.activated_project.clone(), world_streaming_snapshot)?;
        let input =
            PlayerInputSessionV1::restore_from_recovery_bytes(&recovery.input_session_bytes)?;
        let expected_last_logical_frame_sequence =
            recovery.next_logical_frame_sequence.checked_sub(1);
        if input.controller_id() != fixture.controller_id
            || input.source_id() != fixture.source_id
            || input.action_map() != &fixture.action_map
            || input.context_stack() != &fixture.context_stack
            || input.last_logical_frame_sequence() != expected_last_logical_frame_sequence
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let presentation_bindings = fixture_presentation_bindings(&fixture)?;
        let (presentation_extractor, persisted_snapshot) =
            PresentationExtractorV1::begin_authoritative_recovery_from_bytes(
                &recovery.presentation_snapshot_bytes,
                reference_b0_presentation_profile_hash(),
            )?;
        if persisted_snapshot.simulation_tick != runtime.next_tick()
            || persisted_snapshot.project_composition_lock_hash
                != fixture
                    .activated_project
                    .composition_lock
                    .composition_lock_sha256
            || persisted_snapshot.content_manifest_hash
                != fixture
                    .activated_project
                    .content_manifest
                    .content_manifest_sha256
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let mut driver = Self {
            fixture,
            runtime,
            world_streamer,
            input,
            presentation_bindings,
            presentation_extractor,
            next_logical_frame_sequence: recovery.next_logical_frame_sequence,
            events: recovery.events,
            rpg_events: recovery.rpg_events,
            camera_yaw_millidegrees: recovery.camera_yaw_millidegrees,
            camera_pitch_millidegrees: recovery.camera_pitch_millidegrees,
            camera_cut: true,
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
        Ok(self.commit_validated_advance(validated))
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
        let mut presentation_extractor = self.presentation_extractor.clone();
        let mut camera_yaw_millidegrees = self.camera_yaw_millidegrees;
        let mut camera_pitch_millidegrees = self.camera_pitch_millidegrees;

        input_session.submit_platform_events(platform_events)?;
        let input = input_session.close_frame(self.next_logical_frame_sequence)?;
        let next_logical_frame_sequence = self
            .next_logical_frame_sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let mut runtime_preparation = self.runtime.tick_preparation();
        if let Some(resolved) = input.resolved {
            update_camera_state(
                &resolved.frame,
                &mut camera_yaw_millidegrees,
                &mut camera_pitch_millidegrees,
            );
            if let Some(sample) = runtime_sample_without_camera_actions(
                &resolved.frame,
                &resolved.sample,
                &input_session,
            )? {
                runtime_preparation.enqueue_input_sample(&self.fixture.principal, sample)?;
            }
        }
        let prepared_runtime = runtime_preparation.prepare([])?;
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
        presentation_extractor.extract_with_cameras(
            prepared_runtime.next_tick(),
            self.fixture
                .activated_project
                .composition_lock
                .composition_lock_sha256,
            self.fixture
                .activated_project
                .content_manifest
                .content_manifest_sha256,
            prepared_runtime.physics_snapshot(),
            &self.presentation_bindings,
            &[camera],
        )?;
        if presentation_extractor.accepted_snapshot().is_none() {
            return Err(ReferenceGameError::PresentationSnapshotMissing);
        }

        Ok(PreparedReferenceGameAdvance {
            base_generation,
            runtime: prepared_runtime,
            state: PreparedReferenceGameState {
                input: input_session,
                presentation_extractor,
                next_logical_frame_sequence,
                events,
                rpg_events,
                camera_yaw_millidegrees,
                camera_pitch_millidegrees,
                camera_cut: false,
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
        let runtime = self.runtime.validate_prepared_tick(prepared.runtime)?;
        Ok(ValidatedReferenceGameAdvance {
            runtime,
            state: prepared.state,
        })
    }

    #[must_use]
    pub fn commit_validated_advance(
        &mut self,
        validated: ValidatedReferenceGameAdvance,
    ) -> &PresentationSnapshotV2 {
        self.runtime
            .commit_validated_tick_without_report(validated.runtime);
        self.input = validated.state.input;
        self.presentation_extractor = validated.state.presentation_extractor;
        self.next_logical_frame_sequence = validated.state.next_logical_frame_sequence;
        self.events = validated.state.events;
        self.rpg_events = validated.state.rpg_events;
        self.camera_yaw_millidegrees = validated.state.camera_yaw_millidegrees;
        self.camera_pitch_millidegrees = validated.state.camera_pitch_millidegrees;
        self.camera_cut = validated.state.camera_cut;
        self.presentation_extractor
            .accepted_snapshot()
            .expect("validated reference advance contains a presentation snapshot")
    }

    pub fn state(&self) -> Result<ReferenceLiveStateV1, ReferenceGameError> {
        let presentation_input_count = u64::try_from(self.presentation_bindings.len())
            .map_err(|_| ReferenceGameError::CountOverflow)?
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let (checkpoint, checkpoint_canonical_components) =
            self.runtime.world_checkpoint_with_canonical_components()?;
        Ok(ReferenceLiveStateV1 {
            checkpoint,
            checkpoint_canonical_components,
            world_streaming_snapshot: self.world_streamer.snapshot().clone(),
            ticks: self.runtime.next_tick(),
            events: self.events,
            rpg_events: self.rpg_events,
            project_composition_lock_hash: self
                .fixture
                .activated_project
                .composition_lock
                .composition_lock_sha256,
            content_manifest_hash: self
                .fixture
                .activated_project
                .content_manifest
                .content_manifest_sha256,
            presentation_input_count,
            presentation_snapshot: self
                .presentation_extractor
                .accepted_snapshot()
                .cloned()
                .ok_or(ReferenceGameError::PresentationSnapshotMissing)?,
            driver_recovery: ReferenceLiveDriverRecoveryV1 {
                next_logical_frame_sequence: self.next_logical_frame_sequence,
                events: self.events,
                rpg_events: self.rpg_events,
                camera_yaw_millidegrees: self.camera_yaw_millidegrees,
                camera_pitch_millidegrees: self.camera_pitch_millidegrees,
                camera_cut: self.camera_cut,
                input_session_bytes: self.input.recovery_bytes()?,
                presentation_snapshot_bytes: self.presentation_extractor.recovery_bytes()?,
            },
        })
    }

    pub fn prepared_state(
        &self,
        prepared: &PreparedReferenceGameAdvance,
    ) -> Result<ReferenceLiveStateV1, ReferenceGameError> {
        let checkpoint = prepared
            .runtime
            .world_checkpoint_with_canonical_components()?;
        self.state_from_prepared_parts(checkpoint, prepared.next_tick(), &prepared.state)
    }

    pub fn validated_state(
        &self,
        validated: &ValidatedReferenceGameAdvance,
    ) -> Result<ReferenceLiveStateV1, ReferenceGameError> {
        let checkpoint = validated
            .runtime
            .world_checkpoint_with_canonical_components()?;
        self.state_from_prepared_parts(checkpoint, validated.next_tick(), &validated.state)
    }

    fn state_from_prepared_parts(
        &self,
        (checkpoint, checkpoint_canonical_components): (
            WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        ticks: u64,
        state: &PreparedReferenceGameState,
    ) -> Result<ReferenceLiveStateV1, ReferenceGameError> {
        let presentation_input_count = u64::try_from(self.presentation_bindings.len())
            .map_err(|_| ReferenceGameError::CountOverflow)?
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        Ok(ReferenceLiveStateV1 {
            checkpoint,
            checkpoint_canonical_components,
            world_streaming_snapshot: self.world_streamer.snapshot().clone(),
            ticks,
            events: state.events,
            rpg_events: state.rpg_events,
            project_composition_lock_hash: self
                .fixture
                .activated_project
                .composition_lock
                .composition_lock_sha256,
            content_manifest_hash: self
                .fixture
                .activated_project
                .content_manifest
                .content_manifest_sha256,
            presentation_input_count,
            presentation_snapshot: state
                .presentation_extractor
                .accepted_snapshot()
                .cloned()
                .ok_or(ReferenceGameError::PresentationSnapshotMissing)?,
            driver_recovery: ReferenceLiveDriverRecoveryV1 {
                next_logical_frame_sequence: state.next_logical_frame_sequence,
                events: state.events,
                rpg_events: state.rpg_events,
                camera_yaw_millidegrees: state.camera_yaw_millidegrees,
                camera_pitch_millidegrees: state.camera_pitch_millidegrees,
                camera_cut: state.camera_cut,
                input_session_bytes: state.input.recovery_bytes()?,
                presentation_snapshot_bytes: state.presentation_extractor.recovery_bytes()?,
            },
        })
    }

    pub fn cancel_recovered_controls(&mut self) -> Result<(), ReferenceGameError> {
        self.input.cancel_recovered_controls()?;
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
        self.presentation_extractor.extract_with_cameras(
            self.runtime.next_tick(),
            self.fixture
                .activated_project
                .composition_lock
                .composition_lock_sha256,
            self.fixture
                .activated_project
                .content_manifest
                .content_manifest_sha256,
            self.runtime.physics_snapshot(),
            &self.presentation_bindings,
            &[camera],
        )?;
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

fn update_camera_state(
    frame: &PlayerActionFrameV1,
    camera_yaw_millidegrees: &mut i32,
    camera_pitch_millidegrees: &mut i32,
) {
    let Some(PlayerActionValueV1::Vector2Q15([yaw, pitch])) = frame
        .actions
        .iter()
        .find(|action| action.action_id.as_str() == CORE_CAMERA_ORBIT_ACTION_ID)
        .map(|action| action.value)
    else {
        return;
    };
    let yaw_delta = i32::from(yaw).saturating_mul(CAMERA_MOUSE_MILLIDEGREES_PER_UNIT);
    let unwrapped = camera_yaw_millidegrees.saturating_add(yaw_delta);
    *camera_yaw_millidegrees = (unwrapped.saturating_add(180_000)).rem_euclid(360_000) - 180_000;
    let pitch_delta = i32::from(pitch).saturating_mul(CAMERA_MOUSE_MILLIDEGREES_PER_UNIT);
    *camera_pitch_millidegrees = camera_pitch_millidegrees
        .saturating_add(pitch_delta)
        .clamp(-75_000, 75_000);
}

fn runtime_sample_without_camera_actions(
    frame: &PlayerActionFrameV1,
    sample: &next_contracts::input::InputSampleV1,
    input: &PlayerInputSessionV1,
) -> Result<Option<next_contracts::input::InputSampleV1>, ReferenceGameError> {
    let mut runtime_frame = frame.clone();
    runtime_frame
        .actions
        .retain(|action| action.action_id.as_str() != CORE_CAMERA_ORBIT_ACTION_ID);
    if runtime_frame.actions.is_empty() {
        return Ok(None);
    }
    for (ordinal, action) in runtime_frame.actions.iter_mut().enumerate() {
        action.semantic_occurrence_ordinal =
            u32::try_from(ordinal).map_err(|_| ReferenceGameError::CountOverflow)?;
    }
    runtime_frame.validate_against(input.action_map(), input.context_stack())?;
    let mut runtime_sample = sample.clone();
    runtime_sample.payload = runtime_frame.canonical_bytes()?;
    Ok(Some(runtime_sample))
}

fn camera_orbit_offset_micrometres(
    yaw_millidegrees: i32,
    pitch_millidegrees: i32,
) -> Result<[i64; 3], ReferenceGameError> {
    let (yaw_sine, yaw_cosine) = deterministic_sin_cos_q52(yaw_millidegrees)?;
    let (pitch_sine, pitch_cosine) = deterministic_sin_cos_q52(pitch_millidegrees)?;
    let scale = i128::from(CAMERA_TRIG_ONE_Q52);
    let scale_squared = scale * scale;
    let distance = i128::from(CAMERA_DISTANCE_MICROMETRES);
    let shoulder = i128::from(CAMERA_SHOULDER_MICROMETRES);
    let yaw_sine = i128::from(yaw_sine);
    let yaw_cosine = i128::from(yaw_cosine);
    let pitch_sine = i128::from(pitch_sine);
    let pitch_cosine = i128::from(pitch_cosine);

    Ok([
        round_fixed_ratio_to_i64(
            yaw_sine * pitch_cosine * distance + yaw_cosine * shoulder * scale,
            scale_squared,
        )?,
        round_fixed_ratio_to_i64(-pitch_sine * distance, scale)?,
        round_fixed_ratio_to_i64(
            -yaw_cosine * pitch_cosine * distance + yaw_sine * shoulder * scale,
            scale_squared,
        )?,
    ])
}

fn deterministic_sin_cos_q52(angle_millidegrees: i32) -> Result<(i64, i64), ReferenceGameError> {
    if !(-180_000..=180_000).contains(&angle_millidegrees) {
        return Err(ReferenceGameError::CountOverflow);
    }
    let (reduced_angle, quadrant_sign) = if angle_millidegrees > 90_000 {
        (angle_millidegrees - 180_000, -1_i64)
    } else if angle_millidegrees < -90_000 {
        (angle_millidegrees + 180_000, -1_i64)
    } else {
        (angle_millidegrees, 1_i64)
    };
    match reduced_angle {
        0 => return Ok((0, quadrant_sign * CAMERA_TRIG_ONE_Q52)),
        90_000 => return Ok((quadrant_sign * CAMERA_TRIG_ONE_Q52, 0)),
        -90_000 => return Ok((-quadrant_sign * CAMERA_TRIG_ONE_Q52, 0)),
        _ => {}
    }

    let mut residual_turn_q62 =
        round_fixed_ratio_to_i64(i128::from(reduced_angle) * (1_i128 << 62), 360_000)?;
    let mut cosine = CAMERA_CORDIC_GAIN_INVERSE_Q52;
    let mut sine = 0_i64;
    for (shift, angle) in CAMERA_CORDIC_ATAN_TURN_Q62.iter().copied().enumerate() {
        let prior_cosine = cosine;
        let prior_sine = sine;
        if residual_turn_q62 >= 0 {
            cosine = prior_cosine - (prior_sine >> shift);
            sine = prior_sine + (prior_cosine >> shift);
            residual_turn_q62 -= angle;
        } else {
            cosine = prior_cosine + (prior_sine >> shift);
            sine = prior_sine - (prior_cosine >> shift);
            residual_turn_q62 += angle;
        }
    }
    Ok((
        (sine * quadrant_sign).clamp(-CAMERA_TRIG_ONE_Q52, CAMERA_TRIG_ONE_Q52),
        (cosine * quadrant_sign).clamp(-CAMERA_TRIG_ONE_Q52, CAMERA_TRIG_ONE_Q52),
    ))
}

fn round_fixed_ratio_to_i64(numerator: i128, denominator: i128) -> Result<i64, ReferenceGameError> {
    debug_assert!(denominator > 0);
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let rounded = if remainder.abs() * 2 >= denominator {
        quotient + numerator.signum()
    } else {
        quotient
    };
    i64::try_from(rounded).map_err(|_| ReferenceGameError::CountOverflow)
}

#[cfg(test)]
mod tests {
    use super::camera_orbit_offset_micrometres;

    #[test]
    fn integer_camera_orbit_matches_the_pinned_micrometre_golden_vectors() {
        let cases = [
            ((0, -15_000), [350_000, 1_035_276, -3_863_703]),
            ((90_000, 0), [4_000_000, 0, 350_000]),
            ((-90_000, 0), [-4_000_000, 0, -350_000]),
            ((-180_000, 0), [-350_000, 0, 4_000_000]),
            ((42_480, 12_360), [2_896_849, -856_214, -2_645_310]),
            ((-42_480, -12_360), [-2_380_590, 856_214, -3_118_042]),
            ((179_880, 75_000), [-347_831, -3_863_703, 1_036_007]),
            ((-180_000, -75_000), [-350_000, 3_863_703, 1_035_276]),
        ];
        for ((yaw, pitch), expected) in cases {
            assert_eq!(
                camera_orbit_offset_micrometres(yaw, pitch).expect("bounded camera orbit"),
                expected,
                "yaw={yaw}, pitch={pitch}"
            );
        }
    }

    #[test]
    fn integer_camera_orbit_preserves_opposite_yaw_symmetry_without_float_tolerance() {
        for pitch in [-75_000, -15_000, 0, 12_360, 75_000] {
            let forward = camera_orbit_offset_micrometres(0, pitch).expect("forward camera orbit");
            let backward =
                camera_orbit_offset_micrometres(-180_000, pitch).expect("backward camera orbit");
            assert_eq!(forward[0], -backward[0]);
            assert_eq!(forward[1], backward[1]);
            assert_eq!(forward[2], -backward[2]);
        }
    }
}
