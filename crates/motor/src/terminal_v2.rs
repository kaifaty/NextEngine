use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_contracts::motor::MotorTerminalDispositionV1;
use next_physics_physx::{CanonicalPhysXLinkState, CanonicalPhysXSnapshotV2};

use crate::{
    BiomechanicsContactClassV1, BiomechanicsContactFrameV1, BiomechanicsSkillContactProfileV1,
    CompiledBodySchemaV2, HUMANOID_SAFETY_CONTACT_PROFILE_SHA256, MotorSafetyError,
};

pub const BIOMECHANICS_TERMINAL_SUBSTEPS: usize = 4;
pub const BIOMECHANICS_FALL_HEIGHT_MICROMETRES: i64 = 450_000;
pub const BIOMECHANICS_WORLD_BOUND_MICROMETRES: u64 = 90_000_000;
pub const BIOMECHANICS_ROOT_NORM_TOLERANCE_Q2_60: u64 = 1 << 40;
const Q2_60_ONE: i128 = 1_i128 << 60;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BiomechanicsTerminalReasonV1 {
    NonFiniteState = 1,
    JointSafety = 2,
    ContactImpact = 3,
    SelfCollision = 4,
    ForbiddenLocomotionContact = 5,
    WorldBounds = 6,
    Fall = 7,
    Timeout = 8,
}

impl BiomechanicsTerminalReasonV1 {
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::NonFiniteState => "terminal.non-finite-state",
            Self::JointSafety => "terminal.joint-safety",
            Self::ContactImpact => "terminal.contact-impact",
            Self::SelfCollision => "terminal.self-collision",
            Self::ForbiddenLocomotionContact => "terminal.forbidden-locomotion-contact",
            Self::WorldBounds => "terminal.world-bounds",
            Self::Fall => "terminal.fall",
            Self::Timeout => "terminal.timeout",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsTerminalDecisionV1 {
    pub disposition: MotorTerminalDispositionV1,
    pub reason: Option<BiomechanicsTerminalReasonV1>,
    pub decision_root: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsTerminalEvaluator {
    contact_profile_hash: ContentHash,
    articulated_binding: Option<ContentHash>,
    root_actor_token: u64,
    skill_profile: BiomechanicsSkillContactProfileV1,
    maximum_episode_motor_ticks: u64,
    latched: Option<BiomechanicsTerminalDecisionV1>,
}

impl BiomechanicsTerminalEvaluator {
    pub fn new(
        compiled: &CompiledBodySchemaV2,
        skill_profile: BiomechanicsSkillContactProfileV1,
        maximum_episode_motor_ticks: u64,
    ) -> Result<Self, BiomechanicsTerminalError> {
        if maximum_episode_motor_ticks == 0 {
            return Err(BiomechanicsTerminalError::InvalidProfile);
        }
        let root_id = compiled
            .construction_order
            .first()
            .ok_or(BiomechanicsTerminalError::InvalidProfile)?;
        let root_actor_token = *compiled
            .body_tokens
            .get(root_id)
            .ok_or(BiomechanicsTerminalError::InvalidProfile)?;
        Ok(Self {
            contact_profile_hash: content_hash_from_bytes(HUMANOID_SAFETY_CONTACT_PROFILE_SHA256),
            articulated_binding: None,
            root_actor_token,
            skill_profile,
            maximum_episode_motor_ticks,
            latched: None,
        })
    }

    pub fn new_articulated(
        compiled: &crate::CompiledBodySchemaV4,
        skill_profile: BiomechanicsSkillContactProfileV1,
        maximum_episode_motor_ticks: u64,
    ) -> Result<Self, BiomechanicsTerminalError> {
        let subject = compiled
            .articulated_subject()
            .map_err(|_| BiomechanicsTerminalError::InvalidProfile)?;
        let mut evaluator = Self::new(
            &compiled.base.base,
            skill_profile,
            maximum_episode_motor_ticks,
        )?;
        evaluator.contact_profile_hash = crate::articulated_foot_contact_profile_hash();
        let mut bytes = compiled.compiled_descriptor_hash.as_bytes().to_vec();
        bytes.extend_from_slice(subject.as_bytes());
        evaluator.articulated_binding = Some(content_hash_from_bytes(sha256(&bytes)));
        Ok(evaluator)
    }

    pub fn evaluate_motor_tick(
        &mut self,
        motor_tick: u64,
        snapshot: &CanonicalPhysXSnapshotV2,
        contact_substeps: &[BiomechanicsContactFrameV1],
        non_finite_state: bool,
        joint_safety_error: Option<MotorSafetyError>,
    ) -> Result<BiomechanicsTerminalDecisionV1, BiomechanicsTerminalError> {
        if self.latched.is_some() {
            return Err(BiomechanicsTerminalError::EpisodeFinished);
        }
        if contact_substeps.len() != BIOMECHANICS_TERMINAL_SUBSTEPS {
            return Err(BiomechanicsTerminalError::SubstepCount);
        }
        if contact_substeps.iter().any(|frame| {
            frame.skill_profile != self.skill_profile
                || frame.safety_contact_profile_hash != self.contact_profile_hash
        }) {
            return Err(BiomechanicsTerminalError::ContactProfileMismatch);
        }
        let root = unique_root(snapshot, self.root_actor_token);
        let malformed_root = root.is_none_or(|root| {
            !root_rotation_is_valid(root.rotation_q1_30, BIOMECHANICS_ROOT_NORM_TOLERANCE_Q2_60)
        });
        let hard_impact = contact_substeps.iter().any(|frame| {
            frame
                .contacts
                .iter()
                .any(|contact| contact.hard_impact_violation)
        });
        let self_collision = contact_substeps.iter().any(|frame| {
            frame
                .contacts
                .iter()
                .any(|contact| contact.class == BiomechanicsContactClassV1::SelfCollisionViolation)
        });
        let forbidden_contact = self.skill_profile == BiomechanicsSkillContactProfileV1::Locomotion
            && contact_substeps.iter().any(|frame| {
                frame
                    .contacts
                    .iter()
                    .any(|contact| contact.class == BiomechanicsContactClassV1::ForbiddenLocomotion)
            });
        let world_bounds = root.is_some_and(|root| {
            root.position_micrometres[0].unsigned_abs() >= BIOMECHANICS_WORLD_BOUND_MICROMETRES
                || root.position_micrometres[2].unsigned_abs()
                    >= BIOMECHANICS_WORLD_BOUND_MICROMETRES
        });
        let fall = root.is_some_and(|root| {
            root.position_micrometres[1] <= BIOMECHANICS_FALL_HEIGHT_MICROMETRES
                || root_tilt_is_at_least_sixty_degrees(root.rotation_q1_30)
        });
        let reason = if non_finite_state || malformed_root {
            Some(BiomechanicsTerminalReasonV1::NonFiniteState)
        } else if joint_safety_error.is_some() {
            Some(BiomechanicsTerminalReasonV1::JointSafety)
        } else if hard_impact {
            Some(BiomechanicsTerminalReasonV1::ContactImpact)
        } else if self_collision {
            Some(BiomechanicsTerminalReasonV1::SelfCollision)
        } else if forbidden_contact {
            Some(BiomechanicsTerminalReasonV1::ForbiddenLocomotionContact)
        } else if world_bounds {
            Some(BiomechanicsTerminalReasonV1::WorldBounds)
        } else if fall {
            Some(BiomechanicsTerminalReasonV1::Fall)
        } else if motor_tick >= self.maximum_episode_motor_ticks {
            Some(BiomechanicsTerminalReasonV1::Timeout)
        } else {
            None
        };
        let disposition = match reason {
            None => MotorTerminalDispositionV1::Running,
            Some(BiomechanicsTerminalReasonV1::Timeout) => MotorTerminalDispositionV1::Truncated,
            Some(_) => MotorTerminalDispositionV1::Terminated,
        };
        let decision_root = decision_root(
            self.contact_profile_hash,
            self.skill_profile,
            self.maximum_episode_motor_ticks,
            motor_tick,
            root,
            contact_substeps,
            non_finite_state,
            joint_safety_error,
            disposition,
            reason,
        );
        let decision = BiomechanicsTerminalDecisionV1 {
            disposition,
            reason,
            decision_root,
        };
        if disposition != MotorTerminalDispositionV1::Running {
            self.latched = Some(decision.clone());
        }
        Ok(decision)
    }

    pub fn reset(&mut self) {
        self.latched = None;
    }

    #[must_use]
    pub fn terminal_state_root(&self) -> ContentHash {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"nextengine.humanoid-terminal-state.v1\0");
        bytes.extend_from_slice(self.contact_profile_hash.as_bytes());
        bytes.extend_from_slice(&self.root_actor_token.to_le_bytes());
        bytes.extend_from_slice(&BIOMECHANICS_ROOT_NORM_TOLERANCE_Q2_60.to_le_bytes());
        bytes.push(self.skill_profile as u8);
        bytes.extend_from_slice(&self.maximum_episode_motor_ticks.to_le_bytes());
        if let Some(decision) = &self.latched {
            bytes.push(1);
            bytes.extend_from_slice(decision.decision_root.as_bytes());
        } else {
            bytes.push(0);
        }
        if let Some(binding) = self.articulated_binding {
            bytes.extend_from_slice(binding.as_bytes());
        }
        content_hash_from_bytes(sha256(&bytes))
    }
}

fn unique_root(
    snapshot: &CanonicalPhysXSnapshotV2,
    root_actor_token: u64,
) -> Option<&CanonicalPhysXLinkState> {
    let mut matching = snapshot
        .links
        .iter()
        .filter(|link| link.user_token == root_actor_token);
    let root = matching.next()?;
    matching.next().is_none().then_some(root)
}

fn root_tilt_is_at_least_sixty_degrees(rotation_q1_30: [i64; 4]) -> bool {
    let x = i128::from(rotation_q1_30[0]);
    let z = i128::from(rotation_q1_30[2]);
    let lateral_square = x.checked_mul(x).and_then(|value| {
        z.checked_mul(z)
            .and_then(|z_square| value.checked_add(z_square))
    });
    let Some(lateral_square) = lateral_square else {
        return true;
    };
    let up_y_q2_60 = Q2_60_ONE.checked_sub(2_i128.saturating_mul(lateral_square));
    up_y_q2_60.is_none_or(|up_y| up_y <= Q2_60_ONE / 2)
}

fn root_rotation_is_valid(rotation_q1_30: [i64; 4], tolerance_q2_60: u64) -> bool {
    let norm = rotation_q1_30.into_iter().try_fold(0_u128, |sum, value| {
        sum.checked_add(u128::from(value.unsigned_abs()).pow(2))
    });
    norm.is_some_and(|norm| norm.abs_diff(Q2_60_ONE as u128) <= u128::from(tolerance_q2_60))
}

#[allow(clippy::too_many_arguments)]
fn decision_root(
    contact_profile_hash: ContentHash,
    profile: BiomechanicsSkillContactProfileV1,
    maximum_episode_motor_ticks: u64,
    motor_tick: u64,
    root: Option<&CanonicalPhysXLinkState>,
    contact_substeps: &[BiomechanicsContactFrameV1],
    non_finite_state: bool,
    joint_safety_error: Option<MotorSafetyError>,
    disposition: MotorTerminalDispositionV1,
    reason: Option<BiomechanicsTerminalReasonV1>,
) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.humanoid-terminal-decision.v1\0");
    bytes.extend_from_slice(contact_profile_hash.as_bytes());
    bytes.push(profile as u8);
    bytes.extend_from_slice(&maximum_episode_motor_ticks.to_le_bytes());
    bytes.extend_from_slice(&motor_tick.to_le_bytes());
    bytes.push(u8::from(non_finite_state));
    if let Some(error) = joint_safety_error {
        bytes.push(1);
        let code = error.stable_code().as_bytes();
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(code);
    } else {
        bytes.push(0);
    }
    if let Some(root) = root {
        bytes.push(1);
        bytes.extend_from_slice(&root.user_token.to_le_bytes());
        for value in root.position_micrometres {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in root.rotation_q1_30 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    } else {
        bytes.push(0);
    }
    bytes.extend_from_slice(&(contact_substeps.len() as u64).to_le_bytes());
    for frame in contact_substeps {
        bytes.extend_from_slice(frame.classification_root.as_bytes());
        bytes.extend_from_slice(frame.continuity_root.as_bytes());
    }
    bytes.push(disposition as u8);
    bytes.push(reason.map_or(0, |reason| reason as u8));
    content_hash_from_bytes(sha256(&bytes))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BiomechanicsTerminalError {
    InvalidProfile,
    SubstepCount,
    ContactProfileMismatch,
    EpisodeFinished,
}

impl BiomechanicsTerminalError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::InvalidProfile => "MOTOR_TERMINAL_PROFILE_INVALID",
            Self::SubstepCount => "MOTOR_TERMINAL_SUBSTEP_COUNT",
            Self::ContactProfileMismatch => "MOTOR_TERMINAL_CONTACT_PROFILE_MISMATCH",
            Self::EpisodeFinished => "MOTOR_TERMINAL_EPISODE_FINISHED",
        }
    }
}

impl Display for BiomechanicsTerminalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for BiomechanicsTerminalError {}
