use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::motor::MotorObservationLayoutV1;
use next_physics_physx::CanonicalPhysXSnapshot;

use crate::CompiledBodySchemaV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorObservationBuilder {
    layout: MotorObservationLayoutV1,
    root_token: u64,
    joint_ordinals: Vec<u32>,
    effector_tokens: Vec<u64>,
    action_count: usize,
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
        values.extend(root.linear_velocity_micrometres_per_second);
        values.extend(root.angular_velocity_microradians_per_second);
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
}

impl MotorObservationError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::ProfileMismatch => "MOTOR_OBSERVATION_PROFILE_MISMATCH",
            Self::ChannelCount => "MOTOR_OBSERVATION_CHANNEL_COUNT",
            Self::MissingRoot => "MOTOR_OBSERVATION_ROOT_MISSING",
            Self::MissingJoint => "MOTOR_OBSERVATION_JOINT_MISSING",
        }
    }
}

impl Display for MotorObservationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorObservationError {}

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
}
