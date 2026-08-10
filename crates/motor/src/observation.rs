use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::motor::MotorObservationLayoutV1;
use next_physics_physx::CanonicalPhysXSnapshot;

use crate::CompiledBodySchemaV1;

const Q1_30_ONE: i64 = 1_i64 << 30;
const FLAT_LOCOMOTION_OBSERVATION_LAYOUT_ID: &str =
    "motor-observation-layout.humanoid-flat-command.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotorVelocityFrameV1 {
    World,
    RootLocal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorObservationBuilder {
    layout: MotorObservationLayoutV1,
    root_token: u64,
    joint_ordinals: Vec<u32>,
    effector_tokens: Vec<u64>,
    action_count: usize,
    velocity_frame: MotorVelocityFrameV1,
}

impl MotorObservationBuilder {
    pub fn new(compiled: &CompiledBodySchemaV1) -> Result<Self, MotorObservationError> {
        let root_id = compiled
            .construction_order
            .first()
            .ok_or(MotorObservationError::ProfileMismatch)?;
        let root_token = *compiled
            .body_tokens
            .get(root_id)
            .ok_or(MotorObservationError::ProfileMismatch)?;
        let joint_ordinals = compiled
            .physics_descriptors
            .joints
            .iter()
            .map(|joint| {
                compiled
                    .joint_dof_ordinals
                    .get(&joint.joint_id)
                    .copied()
                    .ok_or(MotorObservationError::ProfileMismatch)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            velocity_frame: if compiled.observation_layout.layout_id.as_str()
                == FLAT_LOCOMOTION_OBSERVATION_LAYOUT_ID
            {
                MotorVelocityFrameV1::RootLocal
            } else {
                MotorVelocityFrameV1::World
            },
            layout: compiled.observation_layout.clone(),
            root_token,
            joint_ordinals,
            effector_tokens: compiled.effector_tokens.values().copied().collect(),
            action_count: compiled.action_layout.channels.len(),
        })
    }

    pub fn build(
        &self,
        snapshot: &CanonicalPhysXSnapshot,
        previous_action_microradians: &[i64],
        command_raw: [i64; 3],
    ) -> Result<Vec<i64>, MotorObservationError> {
        if previous_action_microradians.len() != self.action_count {
            return Err(MotorObservationError::ChannelCount);
        }
        let root = snapshot
            .links
            .iter()
            .find(|link| link.user_token == self.root_token)
            .ok_or(MotorObservationError::MissingRoot)?;
        let mut values = Vec::with_capacity(self.layout.channels.len());
        values.extend(root.rotation_q1_30);
        match self.velocity_frame {
            MotorVelocityFrameV1::World => {
                values.extend(root.linear_velocity_micrometres_per_second);
                values.extend(root.angular_velocity_microradians_per_second);
            }
            MotorVelocityFrameV1::RootLocal => {
                values.extend(rotate_world_to_root_local_q1_30(
                    root.rotation_q1_30,
                    root.linear_velocity_micrometres_per_second,
                )?);
                values.extend(rotate_world_to_root_local_q1_30(
                    root.rotation_q1_30,
                    root.angular_velocity_microradians_per_second,
                )?);
            }
        }
        for ordinal in &self.joint_ordinals {
            values.push(
                snapshot
                    .joints
                    .get(*ordinal as usize)
                    .ok_or(MotorObservationError::MissingJoint)?
                    .position_microradians,
            );
        }
        for ordinal in &self.joint_ordinals {
            values.push(
                snapshot
                    .joints
                    .get(*ordinal as usize)
                    .ok_or(MotorObservationError::MissingJoint)?
                    .velocity_microradians_per_second,
            );
        }
        values.extend_from_slice(previous_action_microradians);
        values.extend(command_raw);
        for token in &self.effector_tokens {
            values.push(i64::from(snapshot.contacts.iter().any(|contact| {
                contact.actor_a_token == *token || contact.actor_b_token == *token
            })));
        }
        if values.len() != self.layout.channels.len() {
            return Err(MotorObservationError::ProfileMismatch);
        }
        for (value, channel) in values.iter_mut().zip(&self.layout.channels) {
            *value = (*value).clamp(channel.minimum_raw, channel.maximum_raw);
        }
        Ok(values)
    }

    #[must_use]
    pub fn layout(&self) -> &MotorObservationLayoutV1 {
        &self.layout
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotorObservationError {
    ProfileMismatch,
    ChannelCount,
    MissingRoot,
    MissingJoint,
    ArithmeticOverflow,
}

impl MotorObservationError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::ProfileMismatch => "MOTOR_OBSERVATION_PROFILE_MISMATCH",
            Self::ChannelCount => "MOTOR_OBSERVATION_CHANNEL_COUNT",
            Self::MissingRoot => "MOTOR_OBSERVATION_ROOT_MISSING",
            Self::MissingJoint => "MOTOR_OBSERVATION_JOINT_MISSING",
            Self::ArithmeticOverflow => "MOTOR_OBSERVATION_ARITHMETIC_OVERFLOW",
        }
    }
}

impl Display for MotorObservationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorObservationError {}

pub fn rotate_world_to_root_local_q1_30(
    quaternion_xyzw_q1_30: [i64; 4],
    world_vector: [i64; 3],
) -> Result<[i64; 3], MotorObservationError> {
    if quaternion_xyzw_q1_30
        .iter()
        .any(|value| !(-Q1_30_ONE..=Q1_30_ONE).contains(value))
    {
        return Err(MotorObservationError::ArithmeticOverflow);
    }
    let [x, y, z, w] = quaternion_xyzw_q1_30.map(i128::from);
    let two = |value: i128| {
        value
            .checked_mul(2)
            .ok_or(MotorObservationError::ArithmeticOverflow)
    };
    let product = |left: i128, right: i128| {
        left.checked_mul(right)
            .ok_or(MotorObservationError::ArithmeticOverflow)
    };
    let sum = |left: i128, right: i128| {
        left.checked_add(right)
            .ok_or(MotorObservationError::ArithmeticOverflow)
    };
    let difference = |left: i128, right: i128| {
        left.checked_sub(right)
            .ok_or(MotorObservationError::ArithmeticOverflow)
    };
    let coefficient = |value_q2_60: i128| round_shift_ties_even(value_q2_60, 30);
    let diagonal = |left: i128, right: i128| {
        let square_sum = sum(product(left, left)?, product(right, right)?)?;
        let reduction = coefficient(two(square_sum)?)?;
        Q1_30_ONE
            .checked_sub(reduction)
            .ok_or(MotorObservationError::ArithmeticOverflow)
    };

    let r00 = diagonal(y, z)?;
    let r01 = coefficient(two(difference(product(x, y)?, product(z, w)?)?)?)?;
    let r02 = coefficient(two(sum(product(x, z)?, product(y, w)?)?)?)?;
    let r10 = coefficient(two(sum(product(x, y)?, product(z, w)?)?)?)?;
    let r11 = diagonal(x, z)?;
    let r12 = coefficient(two(difference(product(y, z)?, product(x, w)?)?)?)?;
    let r20 = coefficient(two(difference(product(x, z)?, product(y, w)?)?)?)?;
    let r21 = coefficient(two(sum(product(y, z)?, product(x, w)?)?)?)?;
    let r22 = diagonal(x, y)?;

    let dot = |coefficients: [i64; 3]| -> Result<i64, MotorObservationError> {
        let mut value = 0_i128;
        for (coefficient, component) in coefficients.into_iter().zip(world_vector) {
            value = value
                .checked_add(
                    i128::from(coefficient)
                        .checked_mul(i128::from(component))
                        .ok_or(MotorObservationError::ArithmeticOverflow)?,
                )
                .ok_or(MotorObservationError::ArithmeticOverflow)?;
        }
        round_shift_ties_even(value, 30)
    };
    Ok([
        dot([r00, r10, r20])?,
        dot([r01, r11, r21])?,
        dot([r02, r12, r22])?,
    ])
}

fn round_shift_ties_even(value: i128, shift: u32) -> Result<i64, MotorObservationError> {
    let denominator = 1_i128
        .checked_shl(shift)
        .ok_or(MotorObservationError::ArithmeticOverflow)?;
    let quotient = value / denominator;
    let remainder = (value % denominator).unsigned_abs();
    let half = (denominator / 2) as u128;
    let adjust = remainder > half || (remainder == half && quotient.unsigned_abs() % 2 == 1);
    let rounded = if adjust {
        quotient
            .checked_add(if value.is_negative() { -1 } else { 1 })
            .ok_or(MotorObservationError::ArithmeticOverflow)?
    } else {
        quotient
    };
    i64::try_from(rounded).map_err(|_| MotorObservationError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use next_contracts::ids::PersistentId;
    use next_physics_physx::{
        CanonicalPhysXJointState, CanonicalPhysXLinkState, CanonicalPhysXSnapshot,
    };

    use super::*;
    use crate::reference_humanoid_body_schema_v1;

    #[test]
    fn observation_is_exact_layout_order_and_bounded() {
        let compiled = CompiledBodySchemaV1::compile(
            &reference_humanoid_body_schema_v1(),
            PersistentId::from_bytes([3; 16]),
        )
        .expect("compile");
        let builder = MotorObservationBuilder::new(&compiled).expect("builder");
        let snapshot = CanonicalPhysXSnapshot {
            links: vec![CanonicalPhysXLinkState {
                user_token: 1_000,
                position_micrometres: [0, 1_000_000, 0],
                rotation_q1_30: [0, 0, 0, 1 << 30],
                linear_velocity_micrometres_per_second: [30_000_000, 0, 0],
                angular_velocity_microradians_per_second: [0; 3],
            }],
            joints: (0..23)
                .map(|ordinal| CanonicalPhysXJointState {
                    ordinal,
                    position_microradians: i64::from(ordinal),
                    velocity_microradians_per_second: i64::from(ordinal) * 10,
                })
                .collect(),
            contacts: Vec::new(),
        };
        let values = builder
            .build(&snapshot, &[0; 23], [1, 2, 3])
            .expect("observation");
        assert_eq!(values.len(), builder.layout().channels.len());
        assert_eq!(values[3], 1 << 30);
        assert_eq!(values[4], 20_000_000);
        assert_eq!(&values[values.len() - 5..values.len() - 2], &[1, 2, 3]);
        assert_eq!(&values[values.len() - 2..], &[0, 0]);
    }

    #[test]
    fn root_local_transform_uses_xyzw_and_ties_to_even_integer_math() {
        let identity = rotate_world_to_root_local_q1_30(
            [0, 0, 0, 1 << 30],
            [1_000_000, -2_000_000, 3_000_000],
        )
        .expect("identity");
        assert_eq!(identity, [1_000_000, -2_000_000, 3_000_000]);

        let half_sqrt_q30 = 759_250_125;
        let local = rotate_world_to_root_local_q1_30(
            [0, half_sqrt_q30, 0, half_sqrt_q30],
            [1_000_000, 0, 0],
        )
        .expect("yaw");
        assert!(local[0].unsigned_abs() <= 1);
        assert_eq!(local[1], 0);
        assert!((local[2] - 1_000_000).unsigned_abs() <= 1);
    }
}
