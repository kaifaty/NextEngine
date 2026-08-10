use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::body::BodyActuatorDefinitionV1;
use next_contracts::physics::{AppliedActuatorEffortV1, PhysicsActuatorDescriptorV1};

use crate::CompiledBodySchemaV1;

pub const ACTUATOR_TARGET_CLAMPED: u16 = 1 << 0;
pub const ACTUATOR_EFFORT_CLAMPED: u16 = 1 << 1;
pub const ACTUATOR_RATE_CLAMPED: u16 = 1 << 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JointControlStateV1 {
    pub position_microradians: i64,
    pub velocity_microradians_per_second: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ControlChannel {
    body: BodyActuatorDefinitionV1,
    physics: PhysicsActuatorDescriptorV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedPdController {
    channels: Vec<ControlChannel>,
    previous_efforts: Vec<i64>,
}

impl FixedPdController {
    pub fn new(compiled: &CompiledBodySchemaV1) -> Result<Self, MotorControlError> {
        if compiled.actuator_definitions.len() != compiled.physics_descriptors.actuators.len() {
            return Err(MotorControlError::ProfileMismatch);
        }
        let channels = compiled
            .actuator_definitions
            .iter()
            .cloned()
            .zip(compiled.physics_descriptors.actuators.iter().cloned())
            .map(|(body, physics)| {
                if body.actuator_id != physics.actuator_id || body.joint_id != physics.joint_id {
                    Err(MotorControlError::ProfileMismatch)
                } else {
                    Ok(ControlChannel { body, physics })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            previous_efforts: vec![0; channels.len()],
            channels,
        })
    }

    #[must_use]
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn reset_efforts(&mut self) {
        self.previous_efforts.fill(0);
    }

    pub fn restore_efforts(&mut self, efforts: &[i64]) -> Result<(), MotorControlError> {
        if efforts.len() != self.channels.len() {
            return Err(MotorControlError::ChannelCount);
        }
        for ((value, channel), output) in efforts
            .iter()
            .zip(&self.channels)
            .zip(&mut self.previous_efforts)
        {
            let maximum = i64::try_from(channel.physics.maximum_effort_micronewton_metres)
                .map_err(|_| MotorControlError::NumericOverflow)?;
            if value.unsigned_abs() > maximum as u64 {
                return Err(MotorControlError::EffortOutOfBounds);
            }
            *output = *value;
        }
        Ok(())
    }

    pub fn step_substep(
        &mut self,
        residual_targets_microradians: &[i64],
        joint_states: &[JointControlStateV1],
    ) -> Result<Vec<AppliedActuatorEffortV1>, MotorControlError> {
        if residual_targets_microradians.len() != self.channels.len()
            || joint_states.len() != self.channels.len()
        {
            return Err(MotorControlError::ChannelCount);
        }
        let mut output = Vec::with_capacity(self.channels.len());
        for (index, ((channel, residual), state)) in self
            .channels
            .iter()
            .zip(residual_targets_microradians)
            .zip(joint_states)
            .enumerate()
        {
            let mut flags = 0;
            let requested_target =
                i128::from(channel.body.neutral_position_microradians) + i128::from(*residual);
            let lower = i128::from(channel.physics.limit_min_microradians);
            let upper = i128::from(channel.physics.limit_max_microradians);
            let target = requested_target.clamp(lower, upper);
            if target != requested_target {
                flags |= ACTUATOR_TARGET_CLAMPED;
            }
            let position_error = target - i128::from(state.position_microradians);
            let proportional = round_div_ties_even(
                i128::from(channel.body.stiffness_q16) * position_error,
                65_536,
            );
            let damping = round_div_ties_even(
                i128::from(channel.body.damping_q16)
                    * i128::from(state.velocity_microradians_per_second),
                65_536,
            );
            let requested_effort = proportional - damping;
            let maximum = i128::from(channel.physics.maximum_effort_micronewton_metres);
            let effort_limited = requested_effort.clamp(-maximum, maximum);
            if effort_limited != requested_effort {
                flags |= ACTUATOR_EFFORT_CLAMPED;
            }
            let maximum_rate = i128::from(
                channel
                    .physics
                    .maximum_effort_rate_micronewton_metres_per_second,
            );
            let maximum_delta = round_div_ties_even(maximum_rate, 240);
            let previous = i128::from(self.previous_efforts[index]);
            let rate_limited =
                effort_limited.clamp(previous - maximum_delta, previous + maximum_delta);
            if rate_limited != effort_limited {
                flags |= ACTUATOR_RATE_CLAMPED;
            }
            let effort =
                i64::try_from(rate_limited).map_err(|_| MotorControlError::NumericOverflow)?;
            self.previous_efforts[index] = effort;
            output.push(AppliedActuatorEffortV1 {
                actuator_id: channel.physics.actuator_id.clone(),
                effort_micronewton_metres: effort,
                clamp_flags: flags,
            });
        }
        Ok(output)
    }

    #[must_use]
    pub fn previous_efforts(&self) -> &[i64] {
        &self.previous_efforts
    }
}

fn round_div_ties_even(numerator: i128, denominator: i128) -> i128 {
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
pub enum MotorControlError {
    ProfileMismatch,
    ChannelCount,
    NumericOverflow,
    EffortOutOfBounds,
}

impl MotorControlError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::ProfileMismatch => "MOTOR_CONTROL_PROFILE_MISMATCH",
            Self::ChannelCount => "MOTOR_CONTROL_CHANNEL_COUNT",
            Self::NumericOverflow => "MOTOR_CONTROL_NUMERIC_OVERFLOW",
            Self::EffortOutOfBounds => "MOTOR_CONTROL_EFFORT_OUT_OF_BOUNDS",
        }
    }
}

impl Display for MotorControlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorControlError {}

#[cfg(test)]
mod tests {
    use next_contracts::ids::PersistentId;

    use super::*;
    use crate::reference_humanoid_body_schema_v1;

    fn controller() -> FixedPdController {
        let compiled = CompiledBodySchemaV1::compile(
            &reference_humanoid_body_schema_v1(),
            PersistentId::from_bytes([9; 16]),
        )
        .expect("compile");
        FixedPdController::new(&compiled).expect("controller")
    }

    #[test]
    fn pd_golden_vector_uses_rate_clamp_before_ffi_conversion() {
        let mut controller = controller();
        let targets = vec![1_000_000; controller.channel_count()];
        let states = vec![
            JointControlStateV1 {
                position_microradians: 0,
                velocity_microradians_per_second: 0,
            };
            controller.channel_count()
        ];
        let first = controller
            .step_substep(&targets, &states)
            .expect("first substep");
        assert_eq!(first[0].effort_micronewton_metres, 25_000_000);
        assert_eq!(first[0].clamp_flags, ACTUATOR_RATE_CLAMPED);
        let second = controller
            .step_substep(&targets, &states)
            .expect("second substep");
        assert_eq!(second[0].effort_micronewton_metres, 50_000_000);
        assert_eq!(second[0].clamp_flags, ACTUATOR_RATE_CLAMPED);
    }

    #[test]
    fn target_effort_and_rate_limits_are_independent_facts() {
        let mut controller = controller();
        let targets = vec![10_000_000; controller.channel_count()];
        let states = vec![
            JointControlStateV1 {
                position_microradians: -1_500_000,
                velocity_microradians_per_second: -20_000_000,
            };
            controller.channel_count()
        ];
        let result = controller.step_substep(&targets, &states).expect("substep");
        assert_eq!(result[0].effort_micronewton_metres, 25_000_000);
        assert_eq!(
            result[0].clamp_flags,
            ACTUATOR_TARGET_CLAMPED | ACTUATOR_EFFORT_CLAMPED | ACTUATOR_RATE_CLAMPED
        );
    }

    #[test]
    fn ties_to_even_division_is_sign_symmetric() {
        assert_eq!(round_div_ties_even(5, 2), 2);
        assert_eq!(round_div_ties_even(7, 2), 4);
        assert_eq!(round_div_ties_even(-5, 2), -2);
        assert_eq!(round_div_ties_even(-7, 2), -4);
    }
}
