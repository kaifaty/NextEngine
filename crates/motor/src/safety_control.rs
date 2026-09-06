use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::body::BodyActuatorDefinitionV2;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::physics::{
    AppliedActuatorEffortV1, PhysicsActuatorDescriptorV2, PhysicsJointDescriptorV2,
};

use crate::{CompiledBodySchemaV2, JointControlStateV1};

pub const NORMALIZED_RESIDUAL_ONE_Q1_30: i64 = 1 << 30;
pub const OBSERVED_HARD_ROM_QUANTIZATION_TOLERANCE_MICRORADIANS: i64 = 10;
pub const OBSERVED_MAXIMUM_VELOCITY_QUANTIZATION_TOLERANCE_MICRORADIANS_PER_SECOND: u64 = 1_000;
pub const ACTUATOR_TARGET_SLEW_CLAMPED: u16 = 1 << 3;
pub const ACTUATOR_POWER_CLAMPED: u16 = 1 << 4;
pub const ACTUATOR_WORK_CLAMPED: u16 = 1 << 5;
pub(crate) const PHYSICS_SUBSTEPS_PER_SECOND: i128 = 240;
pub(crate) const PHYSICS_SUBSTEPS_PER_MOTOR_TICK: u8 = 4;
pub(crate) const MICRO_SCALE: i128 = 1_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JointTargetEnvelopeV1 {
    pub joint_id: SchemaId,
    pub minimum_microradians: i64,
    pub maximum_microradians: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedJointTargetV1 {
    pub actuator_id: SchemaId,
    pub target_microradians: i64,
    pub clamp_flags: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsSafetyCheckpointV1 {
    pub applied_targets_microradians: Vec<i64>,
    pub previous_efforts_micronewton_metres: Vec<i64>,
    pub positive_work_microjoules: Vec<u64>,
    pub completed_substeps: u8,
    pub motor_tick_prepared: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SafetyControlChannel {
    pub(crate) body: BodyActuatorDefinitionV2,
    pub(crate) actuator: PhysicsActuatorDescriptorV2,
    pub(crate) joint: PhysicsJointDescriptorV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsSafetyController {
    body_schema_hash: ContentHash,
    compiled_descriptor_hash: ContentHash,
    support_compiled_hash: Option<ContentHash>,
    pub(crate) channels: Vec<SafetyControlChannel>,
    applied_targets: Vec<i64>,
    previous_efforts: Vec<i64>,
    positive_work: Vec<u64>,
    completed_substeps: u8,
    motor_tick_prepared: bool,
}

impl BiomechanicsSafetyController {
    pub fn new(compiled: &CompiledBodySchemaV2) -> Result<Self, MotorSafetyError> {
        if compiled.actuator_definitions.len() != compiled.physics_descriptors.actuators.len() {
            return Err(MotorSafetyError::ProfileMismatch);
        }
        let joints = compiled
            .physics_descriptors
            .joints
            .iter()
            .map(|joint| (joint.base.joint_id.clone(), joint))
            .collect::<BTreeMap<_, _>>();
        let channels = compiled
            .actuator_definitions
            .iter()
            .cloned()
            .zip(compiled.physics_descriptors.actuators.iter().cloned())
            .map(|(body, actuator)| {
                let joint = joints
                    .get(&actuator.base.joint_id)
                    .ok_or(MotorSafetyError::ProfileMismatch)?;
                validate_channel_correspondence(&body, &actuator, joint)?;
                Ok(SafetyControlChannel {
                    body,
                    actuator,
                    joint: (*joint).clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let applied_targets = channels
            .iter()
            .map(|channel| channel.joint.neutral_position_microradians)
            .collect::<Vec<_>>();
        Ok(Self {
            body_schema_hash: compiled.body_schema_hash,
            compiled_descriptor_hash: compiled.compiled_descriptor_hash,
            support_compiled_hash: None,
            previous_efforts: vec![0; channels.len()],
            positive_work: vec![0; channels.len()],
            channels,
            applied_targets,
            completed_substeps: 0,
            motor_tick_prepared: false,
        })
    }

    /// ADR-125: exact V11 diagnostic support input, before unchanged safety.
    pub fn new_bandwidth_support(
        compiled: &crate::CompiledBodySchemaV4,
    ) -> Result<Self, MotorSafetyError> {
        compiled
            .bandwidth_subject()
            .map_err(|_| MotorSafetyError::ProfileMismatch)?;
        let mut controller = Self::new(&compiled.base.base)?;
        controller.support_compiled_hash = Some(compiled.compiled_descriptor_hash);
        Ok(controller)
    }

    #[must_use]
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    #[must_use]
    pub fn default_skill_envelopes(&self) -> Vec<JointTargetEnvelopeV1> {
        self.channels
            .iter()
            .map(|channel| JointTargetEnvelopeV1 {
                joint_id: channel.joint.base.joint_id.clone(),
                minimum_microradians: channel.joint.soft_limit_min_microradians,
                maximum_microradians: channel.joint.soft_limit_max_microradians,
            })
            .collect()
    }

    pub fn begin_motor_tick(
        &mut self,
        reference_targets_microradians: &[i64],
        normalized_residuals_q1_30: &[i64],
        skill_envelopes: &[JointTargetEnvelopeV1],
    ) -> Result<Vec<AppliedJointTargetV1>, MotorSafetyError> {
        self.begin_motor_tick_with_residual_scale_multiplier(
            reference_targets_microradians,
            normalized_residuals_q1_30,
            skill_envelopes,
            65_536,
        )
    }

    pub fn begin_motor_tick_with_residual_scale_multiplier(
        &mut self,
        reference_targets_microradians: &[i64],
        normalized_residuals_q1_30: &[i64],
        skill_envelopes: &[JointTargetEnvelopeV1],
        residual_scale_multiplier_q16: i64,
    ) -> Result<Vec<AppliedJointTargetV1>, MotorSafetyError> {
        if self.motor_tick_prepared && self.completed_substeps != PHYSICS_SUBSTEPS_PER_MOTOR_TICK {
            return Err(MotorSafetyError::TickInProgress);
        }
        if !(1..=4 * 65_536).contains(&residual_scale_multiplier_q16) {
            return Err(MotorSafetyError::InvalidNormalizedResidual);
        }
        if reference_targets_microradians.len() != self.channels.len()
            || normalized_residuals_q1_30.len() != self.channels.len()
            || skill_envelopes.len() != self.channels.len()
        {
            return Err(MotorSafetyError::ChannelCount);
        }
        let mut next_targets = Vec::with_capacity(self.channels.len());
        let mut output = Vec::with_capacity(self.channels.len());
        for (index, (((channel, reference), residual), skill)) in self
            .channels
            .iter()
            .zip(reference_targets_microradians)
            .zip(normalized_residuals_q1_30)
            .zip(skill_envelopes)
            .enumerate()
        {
            if !(-NORMALIZED_RESIDUAL_ONE_Q1_30..=NORMALIZED_RESIDUAL_ONE_Q1_30).contains(residual)
            {
                return Err(MotorSafetyError::InvalidNormalizedResidual);
            }
            if skill.joint_id != channel.joint.base.joint_id
                || skill.minimum_microradians > skill.maximum_microradians
            {
                return Err(MotorSafetyError::InvalidSkillEnvelope);
            }
            let scaled_residual = round_div_ties_even(
                i128::from(*residual)
                    .checked_mul(i128::from(channel.actuator.residual_scale_microradians))
                    .ok_or(MotorSafetyError::NumericOverflow)?,
                i128::from(NORMALIZED_RESIDUAL_ONE_Q1_30),
            );
            let scaled_residual = round_div_ties_even(
                scaled_residual
                    .checked_mul(i128::from(residual_scale_multiplier_q16))
                    .ok_or(MotorSafetyError::NumericOverflow)?,
                65_536,
            );
            let candidate = i128::from(*reference)
                .checked_add(scaled_residual)
                .ok_or(MotorSafetyError::NumericOverflow)?;
            let envelope_minimum = i128::from(channel.joint.base.limit_min_microradians)
                .max(i128::from(channel.joint.soft_limit_min_microradians))
                .max(i128::from(skill.minimum_microradians));
            let envelope_maximum = i128::from(channel.joint.base.limit_max_microradians)
                .min(i128::from(channel.joint.soft_limit_max_microradians))
                .min(i128::from(skill.maximum_microradians));
            if envelope_minimum > envelope_maximum {
                return Err(MotorSafetyError::InvalidSkillEnvelope);
            }
            let previous = i128::from(self.applied_targets[index]);
            let slew_minimum = previous
                .checked_add(i128::from(
                    channel
                        .actuator
                        .minimum_target_delta_microradians_per_motor_tick,
                ))
                .ok_or(MotorSafetyError::NumericOverflow)?;
            let slew_maximum = previous
                .checked_add(i128::from(
                    channel
                        .actuator
                        .maximum_target_delta_microradians_per_motor_tick,
                ))
                .ok_or(MotorSafetyError::NumericOverflow)?;
            let applied_minimum = envelope_minimum.max(slew_minimum);
            let applied_maximum = envelope_maximum.min(slew_maximum);
            if applied_minimum > applied_maximum {
                return Err(MotorSafetyError::InfeasibleTargetEnvelope);
            }
            let applied = candidate.clamp(applied_minimum, applied_maximum);
            let mut flags = 0;
            if !(envelope_minimum..=envelope_maximum).contains(&candidate) {
                flags |= crate::ACTUATOR_TARGET_CLAMPED;
            }
            if !(slew_minimum..=slew_maximum).contains(&candidate) {
                flags |= ACTUATOR_TARGET_SLEW_CLAMPED;
            }
            let target = i64::try_from(applied).map_err(|_| MotorSafetyError::NumericOverflow)?;
            next_targets.push(target);
            output.push(AppliedJointTargetV1 {
                actuator_id: channel.actuator.base.actuator_id.clone(),
                target_microradians: target,
                clamp_flags: flags,
            });
        }
        self.applied_targets = next_targets;
        self.positive_work.fill(0);
        self.completed_substeps = 0;
        self.motor_tick_prepared = true;
        Ok(output)
    }

    pub fn step_substep(
        &mut self,
        joint_states: &[JointControlStateV1],
    ) -> Result<Vec<AppliedActuatorEffortV1>, MotorSafetyError> {
        self.step_substep_inner(joint_states, None)
    }

    /// Ordered microNm support request; never bypasses effort or energy limits.
    pub fn step_substep_with_support_effort(
        &mut self,
        joint_states: &[JointControlStateV1],
        support_efforts_micronewton_metres: &[i64],
    ) -> Result<Vec<AppliedActuatorEffortV1>, MotorSafetyError> {
        if self.support_compiled_hash.is_none() {
            return Err(MotorSafetyError::ProfileMismatch);
        }
        if support_efforts_micronewton_metres.len() != self.channels.len() {
            return Err(MotorSafetyError::ChannelCount);
        }
        self.step_substep_inner(joint_states, Some(support_efforts_micronewton_metres))
    }

    fn step_substep_inner(
        &mut self,
        joint_states: &[JointControlStateV1],
        support: Option<&[i64]>,
    ) -> Result<Vec<AppliedActuatorEffortV1>, MotorSafetyError> {
        if !self.motor_tick_prepared {
            return Err(MotorSafetyError::TickNotPrepared);
        }
        if self.completed_substeps >= PHYSICS_SUBSTEPS_PER_MOTOR_TICK {
            return Err(MotorSafetyError::TooManySubsteps);
        }
        if joint_states.len() != self.channels.len() {
            return Err(MotorSafetyError::ChannelCount);
        }
        validate_joint_states(&self.channels, joint_states)?;
        let mut next_efforts = Vec::with_capacity(self.channels.len());
        let mut next_work = self.positive_work.clone();
        let mut output = Vec::with_capacity(self.channels.len());
        for (index, ((channel, target), state)) in self
            .channels
            .iter()
            .zip(&self.applied_targets)
            .zip(joint_states)
            .enumerate()
        {
            let position_error = i128::from(*target) - i128::from(state.position_microradians);
            let proportional = round_div_ties_even(
                i128::from(channel.actuator.stiffness_q16)
                    .checked_mul(position_error)
                    .ok_or(MotorSafetyError::NumericOverflow)?,
                65_536,
            );
            let damping = round_div_ties_even(
                i128::from(channel.actuator.damping_q16)
                    .checked_mul(i128::from(state.velocity_microradians_per_second))
                    .ok_or(MotorSafetyError::NumericOverflow)?,
                65_536,
            );
            let requested = proportional
                .checked_sub(damping)
                .and_then(|value| {
                    value.checked_add(i128::from(support.map_or(0, |values| values[index])))
                })
                .ok_or(MotorSafetyError::NumericOverflow)?;
            let (effort, flags) = intersect_effort_limits(
                channel,
                requested,
                i128::from(self.previous_efforts[index]),
                state.velocity_microradians_per_second,
                self.positive_work[index],
            )?;
            let charge = positive_work_charge(effort, state.velocity_microradians_per_second)?;
            next_work[index] = next_work[index]
                .checked_add(charge)
                .ok_or(MotorSafetyError::NumericOverflow)?;
            if next_work[index]
                > channel
                    .actuator
                    .maximum_positive_work_microjoules_per_motor_tick
            {
                return Err(MotorSafetyError::InfeasibleEffortEnvelope);
            }
            let effort = i64::try_from(effort).map_err(|_| MotorSafetyError::NumericOverflow)?;
            next_efforts.push(effort);
            output.push(AppliedActuatorEffortV1 {
                actuator_id: channel.actuator.base.actuator_id.clone(),
                effort_micronewton_metres: effort,
                clamp_flags: flags,
            });
        }
        self.previous_efforts = next_efforts;
        self.positive_work = next_work;
        self.completed_substeps += 1;
        Ok(output)
    }

    pub fn validate_observed_joint_states(
        &self,
        joint_states: &[JointControlStateV1],
    ) -> Result<(), MotorSafetyError> {
        if joint_states.len() != self.channels.len() {
            return Err(MotorSafetyError::ChannelCount);
        }
        validate_joint_states(&self.channels, joint_states)
    }

    pub fn reset(&mut self) {
        for (target, channel) in self.applied_targets.iter_mut().zip(&self.channels) {
            *target = channel.joint.neutral_position_microradians;
        }
        self.previous_efforts.fill(0);
        self.positive_work.fill(0);
        self.completed_substeps = 0;
        self.motor_tick_prepared = false;
    }

    pub fn reset_to_reference(
        &mut self,
        reference_targets_microradians: &[i64],
    ) -> Result<(), MotorSafetyError> {
        if reference_targets_microradians.len() != self.channels.len() {
            return Err(MotorSafetyError::ChannelCount);
        }
        for (target, channel) in reference_targets_microradians.iter().zip(&self.channels) {
            if *target < channel.joint.soft_limit_min_microradians
                || *target > channel.joint.soft_limit_max_microradians
            {
                return Err(MotorSafetyError::InvalidSkillEnvelope);
            }
        }
        self.applied_targets
            .copy_from_slice(reference_targets_microradians);
        self.previous_efforts.fill(0);
        self.positive_work.fill(0);
        self.completed_substeps = 0;
        self.motor_tick_prepared = false;
        Ok(())
    }

    #[must_use]
    pub fn checkpoint(&self) -> BiomechanicsSafetyCheckpointV1 {
        BiomechanicsSafetyCheckpointV1 {
            applied_targets_microradians: self.applied_targets.clone(),
            previous_efforts_micronewton_metres: self.previous_efforts.clone(),
            positive_work_microjoules: self.positive_work.clone(),
            completed_substeps: self.completed_substeps,
            motor_tick_prepared: self.motor_tick_prepared,
        }
    }

    #[must_use]
    pub fn checkpoint_root(&self) -> ContentHash {
        let mut bytes = Vec::new();
        if let Some(hash) = self.support_compiled_hash {
            bytes.extend_from_slice(b"nextengine.humanoid-support-safety-checkpoint.v1\0");
            bytes.extend_from_slice(hash.as_bytes());
        } else {
            bytes.extend_from_slice(b"nextengine.humanoid-safety-checkpoint.v1\0");
        }
        bytes.extend_from_slice(&crate::HUMANOID_SAFETY_CONTACT_PROFILE_SHA256);
        bytes.extend_from_slice(self.body_schema_hash.as_bytes());
        bytes.extend_from_slice(self.compiled_descriptor_hash.as_bytes());
        bytes.extend_from_slice(&(self.channels.len() as u64).to_le_bytes());
        for (index, channel) in self.channels.iter().enumerate() {
            push_schema_id(&mut bytes, &channel.actuator.base.actuator_id);
            push_schema_id(&mut bytes, &channel.joint.base.joint_id);
            bytes.extend_from_slice(&self.applied_targets[index].to_le_bytes());
            bytes.extend_from_slice(&self.previous_efforts[index].to_le_bytes());
            bytes.extend_from_slice(&self.positive_work[index].to_le_bytes());
        }
        bytes.push(self.completed_substeps);
        bytes.push(u8::from(self.motor_tick_prepared));
        content_hash_from_bytes(sha256(&bytes))
    }
}

fn push_schema_id(bytes: &mut Vec<u8>, value: &SchemaId) {
    let text = value.as_str().as_bytes();
    bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
    bytes.extend_from_slice(text);
}

fn validate_channel_correspondence(
    body: &BodyActuatorDefinitionV2,
    actuator: &PhysicsActuatorDescriptorV2,
    joint: &PhysicsJointDescriptorV2,
) -> Result<(), MotorSafetyError> {
    if body.actuator_id != actuator.base.actuator_id
        || body.joint_id != actuator.base.joint_id
        || body.joint_id != joint.base.joint_id
        || body.stiffness_q16 != actuator.stiffness_q16
        || body.damping_q16 != actuator.damping_q16
        || body.minimum_effort_micronewton_metres != actuator.minimum_effort_micronewton_metres
        || body.maximum_effort_micronewton_metres != actuator.maximum_effort_micronewton_metres
        || body.maximum_effort_rate_micronewton_metres_per_second
            != actuator
                .base
                .maximum_effort_rate_micronewton_metres_per_second
        || body.maximum_power_microwatts != actuator.maximum_power_microwatts
        || body.maximum_positive_work_microjoules_per_motor_tick
            != actuator.maximum_positive_work_microjoules_per_motor_tick
        || body.residual_scale_microradians != actuator.residual_scale_microradians
        || body.minimum_target_delta_microradians_per_motor_tick
            != actuator.minimum_target_delta_microradians_per_motor_tick
        || body.maximum_target_delta_microradians_per_motor_tick
            != actuator.maximum_target_delta_microradians_per_motor_tick
    {
        return Err(MotorSafetyError::ProfileMismatch);
    }
    Ok(())
}

fn validate_joint_states(
    channels: &[SafetyControlChannel],
    states: &[JointControlStateV1],
) -> Result<(), MotorSafetyError> {
    for (channel, state) in channels.iter().zip(states) {
        let observed_minimum = channel
            .joint
            .base
            .limit_min_microradians
            .saturating_sub(OBSERVED_HARD_ROM_QUANTIZATION_TOLERANCE_MICRORADIANS);
        let observed_maximum = channel
            .joint
            .base
            .limit_max_microradians
            .saturating_add(OBSERVED_HARD_ROM_QUANTIZATION_TOLERANCE_MICRORADIANS);
        if !(observed_minimum..=observed_maximum).contains(&state.position_microradians) {
            return Err(MotorSafetyError::HardRangeViolation);
        }
        if state.velocity_microradians_per_second.unsigned_abs()
            > channel
                .joint
                .base
                .maximum_velocity_microradians_per_second
                .saturating_add(
                    OBSERVED_MAXIMUM_VELOCITY_QUANTIZATION_TOLERANCE_MICRORADIANS_PER_SECOND,
                )
        {
            return Err(MotorSafetyError::VelocityViolation);
        }
    }
    Ok(())
}

fn intersect_effort_limits(
    channel: &SafetyControlChannel,
    requested: i128,
    previous: i128,
    velocity: i64,
    used_work: u64,
) -> Result<(i128, u16), MotorSafetyError> {
    let effort_minimum = i128::from(channel.actuator.minimum_effort_micronewton_metres);
    let effort_maximum = i128::from(channel.actuator.maximum_effort_micronewton_metres);
    let maximum_delta = round_div_ties_even(
        i128::from(
            channel
                .actuator
                .base
                .maximum_effort_rate_micronewton_metres_per_second,
        ),
        PHYSICS_SUBSTEPS_PER_SECOND,
    );
    let rate_minimum = previous
        .checked_sub(maximum_delta)
        .ok_or(MotorSafetyError::NumericOverflow)?;
    let rate_maximum = previous
        .checked_add(maximum_delta)
        .ok_or(MotorSafetyError::NumericOverflow)?;
    let velocity_abs = i128::from(velocity).abs();
    let power_effort_maximum = if velocity_abs == 0 {
        i128::MAX
    } else {
        i128::from(channel.actuator.maximum_power_microwatts)
            .checked_mul(MICRO_SCALE)
            .ok_or(MotorSafetyError::NumericOverflow)?
            / velocity_abs
    };
    let remaining_work = channel
        .actuator
        .maximum_positive_work_microjoules_per_motor_tick
        .checked_sub(used_work)
        .ok_or(MotorSafetyError::InfeasibleEffortEnvelope)?;
    let work_effort_maximum = if velocity_abs == 0 {
        i128::MAX
    } else {
        i128::from(remaining_work)
            .checked_mul(PHYSICS_SUBSTEPS_PER_SECOND)
            .and_then(|value| value.checked_mul(MICRO_SCALE))
            .ok_or(MotorSafetyError::NumericOverflow)?
            / velocity_abs
    };
    let mut minimum = effort_minimum.max(rate_minimum).max(-power_effort_maximum);
    let mut maximum = effort_maximum.min(rate_maximum).min(power_effort_maximum);
    if velocity > 0 {
        maximum = maximum.min(work_effort_maximum);
    } else if velocity < 0 {
        minimum = minimum.max(-work_effort_maximum);
    }
    if minimum > maximum {
        return Err(MotorSafetyError::InfeasibleEffortEnvelope);
    }
    let effort = requested.clamp(minimum, maximum);
    let mut flags = 0;
    if !(effort_minimum..=effort_maximum).contains(&requested) {
        flags |= crate::ACTUATOR_EFFORT_CLAMPED;
    }
    if !(rate_minimum..=rate_maximum).contains(&requested) {
        flags |= crate::ACTUATOR_RATE_CLAMPED;
    }
    if !(-power_effort_maximum..=power_effort_maximum).contains(&requested) {
        flags |= ACTUATOR_POWER_CLAMPED;
    }
    if (velocity > 0 && requested > work_effort_maximum)
        || (velocity < 0 && requested < -work_effort_maximum)
    {
        flags |= ACTUATOR_WORK_CLAMPED;
    }
    Ok((effort, flags))
}

fn positive_work_charge(effort: i128, velocity: i64) -> Result<u64, MotorSafetyError> {
    let product = effort
        .checked_mul(i128::from(velocity))
        .ok_or(MotorSafetyError::NumericOverflow)?;
    if product <= 0 {
        return Ok(0);
    }
    let denominator = PHYSICS_SUBSTEPS_PER_SECOND * MICRO_SCALE;
    let charge = product
        .checked_add(denominator - 1)
        .ok_or(MotorSafetyError::NumericOverflow)?
        / denominator;
    u64::try_from(charge).map_err(|_| MotorSafetyError::NumericOverflow)
}

pub(crate) fn round_div_ties_even(numerator: i128, denominator: i128) -> i128 {
    debug_assert!(denominator > 0);
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let doubled_remainder = remainder.unsigned_abs().saturating_mul(2);
    let denominator = denominator as u128;
    if doubled_remainder < denominator {
        quotient
    } else if doubled_remainder > denominator || quotient.unsigned_abs() % 2 == 1 {
        quotient + numerator.signum()
    } else {
        quotient
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotorSafetyError {
    ProfileMismatch,
    ChannelCount,
    InvalidNormalizedResidual,
    InvalidSkillEnvelope,
    InfeasibleTargetEnvelope,
    NumericOverflow,
    TickInProgress,
    TickNotPrepared,
    TooManySubsteps,
    HardRangeViolation,
    VelocityViolation,
    InfeasibleEffortEnvelope,
}

impl MotorSafetyError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::ProfileMismatch => "MOTOR_SAFETY_PROFILE_MISMATCH",
            Self::ChannelCount => "MOTOR_SAFETY_CHANNEL_COUNT",
            Self::InvalidNormalizedResidual => "MOTOR_SAFETY_RESIDUAL_INVALID",
            Self::InvalidSkillEnvelope => "MOTOR_SAFETY_SKILL_ENVELOPE_INVALID",
            Self::InfeasibleTargetEnvelope => "MOTOR_SAFETY_TARGET_ENVELOPE_EMPTY",
            Self::NumericOverflow => "MOTOR_SAFETY_NUMERIC_OVERFLOW",
            Self::TickInProgress => "MOTOR_SAFETY_TICK_IN_PROGRESS",
            Self::TickNotPrepared => "MOTOR_SAFETY_TICK_NOT_PREPARED",
            Self::TooManySubsteps => "MOTOR_SAFETY_SUBSTEP_COUNT",
            Self::HardRangeViolation => "MOTOR_SAFETY_HARD_ROM_VIOLATION",
            Self::VelocityViolation => "MOTOR_SAFETY_VELOCITY_VIOLATION",
            Self::InfeasibleEffortEnvelope => "MOTOR_SAFETY_EFFORT_ENVELOPE_EMPTY",
        }
    }
}

impl Display for MotorSafetyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorSafetyError {}
