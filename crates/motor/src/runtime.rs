use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_contracts::motor::STAGE0_SUBSTEPS;
use next_contracts::physics::AppliedActuatorEffortV1;
use next_physics_physx::{
    CanonicalPhysXSnapshot, PhysXAdapterError, PhysXArticulationWorld, PhysXRawArticulationSnapshot,
};

use crate::{
    CompiledBodySchemaV1, FixedPdController, JointControlStateV1, MotorControlError,
    MotorObservationBuilder, MotorObservationError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorReplayFrame {
    pub motor_tick: u64,
    pub post_safety_efforts_micronewton_metres: Vec<i64>,
    pub physics_witness_hash: ContentHash,
}

pub const MAX_REPLAY_MOTOR_TICKS: usize = 3_600;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanoidMotorCheckpoint {
    pub motor_tick: u64,
    pub applied_action_microradians: Vec<i64>,
    pub command_raw: [i64; 3],
    pub previous_efforts_micronewton_metres: Vec<i64>,
    pub physics: PhysXRawArticulationSnapshot,
    pub replay_origin_physics: PhysXRawArticulationSnapshot,
    pub replay_frames: Vec<MotorReplayFrame>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorFrameResult {
    pub motor_tick: u64,
    pub applied_action_microradians: Vec<i64>,
    pub observation_raw: Vec<i64>,
    pub substep_efforts: Vec<Vec<AppliedActuatorEffortV1>>,
    pub snapshot: CanonicalPhysXSnapshot,
}

#[derive(Debug)]
pub struct DeterministicHumanoidMotor {
    compiled: CompiledBodySchemaV1,
    world: PhysXArticulationWorld,
    controller: FixedPdController,
    observation_builder: MotorObservationBuilder,
    current_snapshot: CanonicalPhysXSnapshot,
    applied_action: Vec<i64>,
    command_raw: [i64; 3],
    motor_tick: u64,
    replay_origin_physics: PhysXRawArticulationSnapshot,
    replay_frames: Vec<MotorReplayFrame>,
}

impl DeterministicHumanoidMotor {
    pub fn create(compiled: CompiledBodySchemaV1) -> Result<Self, MotorRuntimeError> {
        let mut world =
            PhysXArticulationWorld::create(compiled.physx_scene_profile, &compiled.physx_catalog)?;
        let current_snapshot = world.capture()?;
        let replay_origin_physics = world.raw_checkpoint();
        let controller = FixedPdController::new(&compiled)?;
        let observation_builder = MotorObservationBuilder::new(&compiled)?;
        Ok(Self {
            applied_action: vec![0; controller.channel_count()],
            compiled,
            world,
            controller,
            observation_builder,
            current_snapshot,
            command_raw: [0; 3],
            motor_tick: 0,
            replay_origin_physics,
            replay_frames: Vec::new(),
        })
    }

    pub fn reset(&mut self) -> Result<Vec<i64>, MotorRuntimeError> {
        self.world = PhysXArticulationWorld::create(
            self.compiled.physx_scene_profile,
            &self.compiled.physx_catalog,
        )?;
        self.current_snapshot = self.world.capture()?;
        self.controller.reset_efforts();
        self.applied_action.fill(0);
        self.command_raw = [0; 3];
        self.motor_tick = 0;
        self.replay_origin_physics = self.world.raw_checkpoint();
        self.replay_frames.clear();
        self.observation_builder
            .build(
                &self.current_snapshot,
                &self.applied_action,
                self.command_raw,
            )
            .map_err(Into::into)
    }

    pub fn step_motor_frame(
        &mut self,
        requested_action_microradians: &[i64],
        command_raw: [i64; 3],
    ) -> Result<MotorFrameResult, MotorRuntimeError> {
        if requested_action_microradians.len() != self.compiled.action_layout.channels.len() {
            return Err(MotorRuntimeError::ChannelCount);
        }
        if self.replay_frames.len() >= MAX_REPLAY_MOTOR_TICKS {
            return Err(MotorRuntimeError::ReplayCapacity);
        }
        let next_motor_tick = self
            .motor_tick
            .checked_add(1)
            .ok_or(MotorRuntimeError::TickOverflow)?;
        self.applied_action = requested_action_microradians
            .iter()
            .zip(&self.compiled.action_layout.channels)
            .map(|(value, channel)| (*value).clamp(channel.minimum_raw, channel.maximum_raw))
            .collect();
        self.command_raw = command_raw;
        let mut substep_efforts = Vec::with_capacity(STAGE0_SUBSTEPS);
        for _ in 0..STAGE0_SUBSTEPS {
            let joint_states = self
                .compiled
                .actuator_dof_ordinals
                .iter()
                .map(|ordinal| {
                    self.current_snapshot
                        .joints
                        .get(*ordinal as usize)
                        .map(|joint| JointControlStateV1 {
                            position_microradians: joint.position_microradians,
                            velocity_microradians_per_second: joint
                                .velocity_microradians_per_second,
                        })
                        .ok_or(MotorRuntimeError::ProfileMismatch)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let efforts = self
                .controller
                .step_substep(&self.applied_action, &joint_states)?;
            let effort_values = efforts
                .iter()
                .map(|effort| effort.effort_micronewton_metres)
                .collect::<Vec<_>>();
            self.current_snapshot = self.world.apply_efforts_and_step(&effort_values)?;
            substep_efforts.push(efforts);
        }
        self.motor_tick = next_motor_tick;
        let post_safety_efforts_micronewton_metres = substep_efforts
            .iter()
            .flat_map(|efforts| {
                efforts
                    .iter()
                    .map(|effort| effort.effort_micronewton_metres)
            })
            .collect();
        self.replay_frames.push(MotorReplayFrame {
            motor_tick: self.motor_tick,
            post_safety_efforts_micronewton_metres,
            physics_witness_hash: physics_witness_hash(&self.current_snapshot),
        });
        let observation_raw = self.observation_builder.build(
            &self.current_snapshot,
            &self.applied_action,
            self.command_raw,
        )?;
        Ok(MotorFrameResult {
            motor_tick: self.motor_tick,
            applied_action_microradians: self.applied_action.clone(),
            observation_raw,
            substep_efforts,
            snapshot: self.current_snapshot.clone(),
        })
    }

    #[must_use]
    pub fn checkpoint(&self) -> HumanoidMotorCheckpoint {
        HumanoidMotorCheckpoint {
            motor_tick: self.motor_tick,
            applied_action_microradians: self.applied_action.clone(),
            command_raw: self.command_raw,
            previous_efforts_micronewton_metres: self.controller.previous_efforts().to_vec(),
            physics: self.world.raw_checkpoint(),
            replay_origin_physics: self.replay_origin_physics.clone(),
            replay_frames: self.replay_frames.clone(),
        }
    }

    #[must_use]
    pub(crate) fn current_snapshot(&self) -> &CanonicalPhysXSnapshot {
        &self.current_snapshot
    }

    pub fn restore_fresh(
        &mut self,
        checkpoint: &HumanoidMotorCheckpoint,
    ) -> Result<Vec<i64>, MotorRuntimeError> {
        self.validate_checkpoint_shape(checkpoint)?;
        let mut world = PhysXArticulationWorld::create(
            self.compiled.physx_scene_profile,
            &self.compiled.physx_catalog,
        )?;
        let mut snapshot = if world.raw_checkpoint() == checkpoint.replay_origin_physics {
            world.capture()?
        } else {
            let snapshot = world.restore(&checkpoint.replay_origin_physics)?;
            if world.raw_checkpoint() != checkpoint.replay_origin_physics {
                return Err(MotorRuntimeError::RestoreDivergence);
            }
            snapshot
        };

        let channel_count = self.compiled.action_layout.channels.len();
        for frame in &checkpoint.replay_frames {
            for efforts in frame
                .post_safety_efforts_micronewton_metres
                .chunks_exact(channel_count)
            {
                snapshot = world.apply_efforts_and_step(efforts)?;
            }
            if physics_witness_hash(&snapshot) != frame.physics_witness_hash {
                return Err(MotorRuntimeError::RestoreDivergence);
            }
        }
        if world.raw_checkpoint() != checkpoint.physics {
            return Err(MotorRuntimeError::RestoreDivergence);
        }

        let mut controller = FixedPdController::new(&self.compiled)?;
        controller.restore_efforts(&checkpoint.previous_efforts_micronewton_metres)?;
        let observation = self.observation_builder.build(
            &snapshot,
            &checkpoint.applied_action_microradians,
            checkpoint.command_raw,
        )?;
        self.world = world;
        self.controller = controller;
        self.current_snapshot = snapshot;
        self.applied_action = checkpoint.applied_action_microradians.clone();
        self.command_raw = checkpoint.command_raw;
        self.motor_tick = checkpoint.motor_tick;
        self.replay_origin_physics = checkpoint.replay_origin_physics.clone();
        self.replay_frames = checkpoint.replay_frames.clone();
        Ok(observation)
    }

    fn validate_checkpoint_shape(
        &self,
        checkpoint: &HumanoidMotorCheckpoint,
    ) -> Result<(), MotorRuntimeError> {
        let channel_count = self.compiled.action_layout.channels.len();
        let effort_count = channel_count
            .checked_mul(STAGE0_SUBSTEPS)
            .ok_or(MotorRuntimeError::ReplayBounds)?;
        if checkpoint.applied_action_microradians.len() != channel_count
            || checkpoint.previous_efforts_micronewton_metres.len() != channel_count
            || checkpoint.physics.joints.len() != channel_count
            || checkpoint.replay_origin_physics.joints.len() != channel_count
            || checkpoint.physics.links.len() != checkpoint.replay_origin_physics.links.len()
            || checkpoint.replay_frames.len() > MAX_REPLAY_MOTOR_TICKS
            || checkpoint.motor_tick
                != u64::try_from(checkpoint.replay_frames.len())
                    .map_err(|_| MotorRuntimeError::ReplayBounds)?
        {
            return Err(MotorRuntimeError::ReplayBounds);
        }
        for (index, frame) in checkpoint.replay_frames.iter().enumerate() {
            let expected_tick = u64::try_from(index)
                .map_err(|_| MotorRuntimeError::ReplayBounds)?
                .checked_add(1)
                .ok_or(MotorRuntimeError::ReplayBounds)?;
            if frame.motor_tick != expected_tick
                || frame.post_safety_efforts_micronewton_metres.len() != effort_count
            {
                return Err(MotorRuntimeError::ReplayBounds);
            }
        }
        Ok(())
    }
}

pub(crate) fn physics_witness_hash(snapshot: &CanonicalPhysXSnapshot) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.motor-physics-witness.v1\0");
    bytes.extend_from_slice(&(snapshot.links.len() as u64).to_le_bytes());
    for link in &snapshot.links {
        bytes.extend_from_slice(&link.user_token.to_le_bytes());
        for values in [
            &link.position_micrometres[..],
            &link.rotation_q1_30[..],
            &link.linear_velocity_micrometres_per_second[..],
            &link.angular_velocity_microradians_per_second[..],
        ] {
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    bytes.extend_from_slice(&(snapshot.joints.len() as u64).to_le_bytes());
    for joint in &snapshot.joints {
        bytes.extend_from_slice(&joint.ordinal.to_le_bytes());
        bytes.extend_from_slice(&joint.position_microradians.to_le_bytes());
        bytes.extend_from_slice(&joint.velocity_microradians_per_second.to_le_bytes());
    }
    bytes.extend_from_slice(&(snapshot.contacts.len() as u64).to_le_bytes());
    for contact in &snapshot.contacts {
        bytes.extend_from_slice(&contact.actor_a_token.to_le_bytes());
        bytes.extend_from_slice(&contact.actor_b_token.to_le_bytes());
        for values in [
            &contact.position_micrometres[..],
            &contact.normal_q1_30[..],
            &contact.impulse_micronewton_seconds[..],
        ] {
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&contact.separation_micrometres.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

#[derive(Debug)]
pub enum MotorRuntimeError {
    Physics(PhysXAdapterError),
    Control(MotorControlError),
    Observation(MotorObservationError),
    ChannelCount,
    ProfileMismatch,
    TickOverflow,
    ReplayCapacity,
    ReplayBounds,
    RestoreDivergence,
}

impl MotorRuntimeError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Physics(error) => error.stable_code(),
            Self::Control(error) => error.stable_code(),
            Self::Observation(error) => error.stable_code(),
            Self::ChannelCount => "MOTOR_RUNTIME_CHANNEL_COUNT",
            Self::ProfileMismatch => "MOTOR_RUNTIME_PROFILE_MISMATCH",
            Self::TickOverflow => "MOTOR_RUNTIME_TICK_OVERFLOW",
            Self::ReplayCapacity => "MOTOR_RUNTIME_REPLAY_CAPACITY_EXCEEDED",
            Self::ReplayBounds => "MOTOR_RUNTIME_REPLAY_BOUNDS_INVALID",
            Self::RestoreDivergence => "MOTOR_RUNTIME_RESTORE_DIVERGENCE",
        }
    }
}

impl Display for MotorRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorRuntimeError {}

impl From<PhysXAdapterError> for MotorRuntimeError {
    fn from(value: PhysXAdapterError) -> Self {
        Self::Physics(value)
    }
}

impl From<MotorControlError> for MotorRuntimeError {
    fn from(value: MotorControlError) -> Self {
        Self::Control(value)
    }
}

impl From<MotorObservationError> for MotorRuntimeError {
    fn from(value: MotorObservationError) -> Self {
        Self::Observation(value)
    }
}

#[cfg(all(test, any(feature = "physx-sdk", feature = "mock-abi")))]
mod tests {
    use next_contracts::ids::PersistentId;

    use super::*;
    use crate::{CompiledBodySchemaV1, reference_humanoid_body_schema_v1};

    #[test]
    fn one_motor_frame_is_exactly_four_physics_substeps_and_restorable() {
        let compiled = CompiledBodySchemaV1::compile(
            &reference_humanoid_body_schema_v1(),
            PersistentId::from_bytes([8; 16]),
        )
        .expect("compile");
        let mut runtime = DeterministicHumanoidMotor::create(compiled).expect("runtime");
        let initial = runtime.checkpoint();
        let result = runtime
            .step_motor_frame(&[100_000; 23], [500_000, 0, 0])
            .expect("frame");
        assert_eq!(result.motor_tick, 1);
        assert_eq!(result.substep_efforts.len(), STAGE0_SUBSTEPS);
        assert!(
            result
                .substep_efforts
                .iter()
                .all(|efforts| efforts.len() == 23)
        );
        let restored_observation = runtime.restore_fresh(&initial).expect("fresh restore");
        let reset_observation = runtime.reset().expect("fresh reset");
        assert_eq!(restored_observation, reset_observation);
    }
}
